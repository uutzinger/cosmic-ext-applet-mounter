// SPDX-License-Identifier: MIT
//! Invoked by scripts/onedrive-mirror-prototype.sh; emits redacted protocol JSON only.
use cosmic_ext_applet_mounter::{
    onedrive_prototype::{
        self as probe, DriveType, Engine, MicrosoftGraph, ProbeError, ProbeResult, Session,
    },
    process::{CommandExecutionMode, RuntimeCommandRunner},
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf};
use tokio_util::sync::CancellationToken;

fn arguments() -> ProbeResult<BTreeMap<String, String>> {
    let mut args = std::env::args().skip(1);
    let mut result = BTreeMap::new();
    while let Some(key) = args.next() {
        if ![
            "--action",
            "--remote",
            "--drive",
            "--expected-account",
            "--session",
            "--files-dir",
            "--transport",
            "--report-dir",
        ]
        .contains(&key.as_str())
        {
            return Err(ProbeError::InvalidInput);
        }
        if result
            .insert(key, args.next().ok_or(ProbeError::InvalidInput)?)
            .is_some()
        {
            return Err(ProbeError::InvalidInput);
        }
    }
    Ok(result)
}
async fn run(args: &BTreeMap<String, String>) -> ProbeResult<Value> {
    let get = |k: &str| {
        args.get(k)
            .map(String::as_str)
            .ok_or(ProbeError::InvalidInput)
    };
    let action = get("--action")?;
    if args.contains_key("--files-dir") && action != "prepare" {
        return Err(ProbeError::InvalidInput);
    }
    let remote = get("--remote")?;
    let drive = DriveType::parse(get("--drive")?)?;
    let account = get("--expected-account")?;
    let mode = match get("--transport")? {
        "native" => CommandExecutionMode::Native,
        "flatpak-host" => CommandExecutionMode::FlatpakSpawnHost,
        _ => return Err(ProbeError::InvalidInput),
    };
    let report = PathBuf::from(get("--report-dir")?);
    if !report.is_absolute() || !report.is_dir() {
        return Err(ProbeError::InvalidInput);
    }
    let runner = RuntimeCommandRunner::for_mode(mode);
    let graph = MicrosoftGraph::new()?;
    let cancellation = CancellationToken::new();
    let signal = cancellation.clone();
    let watcher = tokio::spawn(async move {
        #[cfg(unix)]
        {
            if let Ok(mut terminate) =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            {
                tokio::select! { _=tokio::signal::ctrl_c()=>{}, _=terminate.recv()=>{} }
            } else {
                let _ = tokio::signal::ctrl_c().await;
            }
        }
        #[cfg(not(unix))]
        {
            let _ = tokio::signal::ctrl_c().await;
        }
        signal.cancel();
    });
    let result=async {
        if action=="auth" {return Ok(json!({"identity":probe::authenticate(&runner,&graph,remote,drive,account,cancellation.child_token()).await?,"remote_created":true}));}
        if action=="inspect" {return Ok(json!({"identity":probe::verify(&runner,&graph,remote,drive,account,cancellation.child_token()).await?}));}
        let root=if action=="prepare" {
            if args.contains_key("--session") {return Err(ProbeError::InvalidInput);}
            report.join("session")
        } else {
            let path=PathBuf::from(get("--session")?);
            if !path.is_absolute(){return Err(ProbeError::InvalidInput);}
            path
        };
        let session=if action=="prepare" {Session::new(&root,remote,probe::verify(&runner,&graph,remote,drive,account,cancellation.child_token()).await?)?}
            else {Session::load(&root)?};
        if session.remote!=remote || session.identity.drive_type!=drive {return Err(ProbeError::WrongDrive);}
        let _session_lock=cosmic_ext_applet_mounter::bisync_coordinator::acquire_operation_guard_for_state(&root.join("prototype-state"))
            .map_err(|_|ProbeError::InvalidState)?;
        let mut engine=Engine{runner:&runner,graph:&graph,account,root:root.clone(),session,cancellation:cancellation.child_token()};
        let details=match action {
            "prepare"=>{if let Some(source)=args.get("--files-dir") {engine.setup_samples(&PathBuf::from(source)).await?;} else {engine.setup().await?;} json!({"next_action":"initial-sync","sample_tree":engine.session.sample_tree})},
            "initial-sync"=>{engine.sync("initial-sync").await?;engine.verify_bytes().await?;json!({"next_action":if engine.session.sample_tree.is_some(){"timing"}else{"scenario"}})},
            "verify"=>{engine.verify_bytes().await?;json!({"next_action":"scenario"})},
            "scenario"=>{engine.scenario().await?;json!({"next_action":"fault-network_or_fault-cancel"})},
            "benchmark"=>{engine.benchmark().await?;json!({"benchmark":engine.session.benchmark})},
            "timing"=>{engine.timing().await?;json!({"benchmark":engine.session.benchmark,"next_action":"cleanup"})},
            "fault-network"=>{engine.fault(false).await?;json!({"next_action":"repair"})},
            "fault-cancel"=>{engine.fault(true).await?;json!({"next_action":"repair"})},
            "repair"=>engine.repair_inspection().await?,
            "cleanup"=>{engine.cleanup().await?;json!({"cloud_cleanup":"purged","local_evidence":"retained"})},
            _=>return Err(ProbeError::InvalidInput),
        };
        Ok(json!({"session":root,"details":details}))
    }.await;
    watcher.abort();
    let _ = watcher.await;
    result
}
#[tokio::main]
async fn main() {
    let args = match arguments() {
        Ok(v) => v,
        Err(e) => {
            println!("{}", json!({"status":"failed","error_code":e}));
            std::process::exit(2)
        }
    };
    let result = run(&args).await;
    let value = match &result {
        Ok(data) => json!({"schema_version":1,"status":"succeeded","result":data}),
        Err(code) => json!({"schema_version":1,"status":"failed","error_code":code}),
    };
    if let Some(directory) = args.get("--report-dir")
        && probe::save(&PathBuf::from(directory).join("prototype.json"), &value).is_err()
    {
        println!("{}", json!({"status":"failed","error_code":"io"}));
        std::process::exit(1);
    }
    println!("{value}");
    if result.is_err() {
        std::process::exit(1);
    }
}
