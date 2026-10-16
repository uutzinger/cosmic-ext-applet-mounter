// SPDX-License-Identifier: MIT
//! Development-only OneDrive scenario orchestration using production identity policy.
use crate::process::{CommandRequest, CommandRunner, Executable};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub use crate::onedrive::{
    DriveType, Graph, GraphFuture, GraphTarget, Identity, MicrosoftGraph,
    OneDriveError as ProbeError, OneDriveResult as ProbeResult, RcloneFailure, RcloneOperation,
    command, complete, fingerprint, request, validate_remote,
};
#[cfg(test)]
use crate::onedrive::{graph_url, primary};

pub async fn verify<R: CommandRunner, G: Graph>(
    runner: &R,
    graph: &G,
    remote: &str,
    drive_type: DriveType,
    account: &str,
    cancel: CancellationToken,
) -> ProbeResult<Identity> {
    crate::onedrive::verify(runner, graph, remote, drive_type, account, cancel)
        .await
        .map(|verified| verified.fingerprints)
}

pub async fn authenticate<R: CommandRunner, G: Graph>(
    runner: &R,
    graph: &G,
    remote: &str,
    drive_type: DriveType,
    account: &str,
    cancel: CancellationToken,
) -> ProbeResult<Identity> {
    if !remote.starts_with("cm_proto_") {
        return Err(ProbeError::InvalidInput);
    }
    let setup = crate::onedrive::RemoteSetup::new(remote, drive_type, account)?;
    crate::onedrive::authenticate(runner, graph, &setup, cancel)
        .await
        .map(|verified| verified.fingerprints)
}

use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use uuid::Uuid;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stage {
    pub operation: String,
    pub elapsed_ms: u128,
    pub succeeded: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SampleTree {
    pub files: u64,
    pub bytes: u64,
    pub directories: u64,
    pub max_depth: usize,
    pub digest: String,
}
fn sample_inventory(root: &Path, cancel: &CancellationToken) -> ProbeResult<SampleTree> {
    use std::io::Read;
    let mut result = SampleTree {
        files: 0,
        bytes: 0,
        directories: 0,
        max_depth: 0,
        digest: String::new(),
    };
    let mut pending = vec![root.to_path_buf()];
    let mut records = Vec::new();
    while let Some(path) = pending.pop() {
        if cancel.is_cancelled() {
            return Err(ProbeError::Cancelled);
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|_| ProbeError::InvalidInput)?;
        let depth = relative.components().count();
        let metadata = io(fs::symlink_metadata(&path))?;
        if depth > 64 || records.len() >= 20_000 || metadata.file_type().is_symlink() {
            return Err(ProbeError::InvalidInput);
        }
        result.max_depth = result.max_depth.max(depth);
        let name = relative
            .to_str()
            .ok_or(ProbeError::InvalidInput)?
            .to_owned();
        if metadata.is_dir() {
            result.directories += 1;
            records.push((name, 0, String::new()));
            for entry in io(fs::read_dir(&path))? {
                if pending.len() + records.len() >= 20_000 {
                    return Err(ProbeError::InvalidInput);
                }
                pending.push(io(entry)?.path());
            }
        } else if metadata.is_file() {
            result.files += 1;
            result.bytes = result
                .bytes
                .checked_add(metadata.len())
                .ok_or(ProbeError::InvalidInput)?;
            if result.bytes > 2 * 1024 * 1024 * 1024 {
                return Err(ProbeError::InvalidInput);
            }
            let mut file = io(fs::File::open(&path))?;
            let mut hash = Sha256::new();
            let mut bytes = 0;
            let mut buffer = [0_u8; 64 * 1024];
            loop {
                if cancel.is_cancelled() {
                    return Err(ProbeError::Cancelled);
                }
                let n = io(file.read(&mut buffer))?;
                if n == 0 {
                    break;
                }
                bytes += n as u64;
                if bytes > metadata.len() {
                    return Err(ProbeError::VerificationFailed);
                }
                hash.update(&buffer[..n]);
            }
            if bytes != metadata.len() {
                return Err(ProbeError::VerificationFailed);
            }
            records.push((name, 1, format!("{}:{:x}", bytes, hash.finalize())));
        } else {
            return Err(ProbeError::InvalidInput);
        }
    }
    records.sort();
    result.digest = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&records).map_err(|_| ProbeError::InvalidState)?)
    );
    Ok(result)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub schema_version: u8,
    pub id: Uuid,
    pub remote: String,
    pub identity: Identity,
    pub initialized: bool,
    pub previewed: bool,
    pub failed: bool,
    pub owned: bool,
    pub cleaned: bool,
    pub stages: Vec<Stage>,
    pub benchmark: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_tree: Option<SampleTree>,
}
fn io<T>(result: std::io::Result<T>) -> ProbeResult<T> {
    result.map_err(|_| ProbeError::Io)
}
pub fn save(path: &Path, value: &impl Serialize) -> ProbeResult<()> {
    let tmp = path.with_extension(format!("{}.tmp", Uuid::new_v4()));
    let data = serde_json::to_vec_pretty(value).map_err(|_| ProbeError::InvalidState)?;
    io(fs::write(&tmp, data))?;
    io(fs::rename(tmp, path))
}
impl Session {
    pub fn load(root: &Path) -> ProbeResult<Self> {
        if io(fs::symlink_metadata(root))?.file_type().is_symlink() {
            return Err(ProbeError::Ownership);
        }
        for name in ["local", "work", "recovery", "uploads"] {
            let metadata = io(fs::symlink_metadata(root.join(name)))?;
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                return Err(ProbeError::Ownership);
            }
        }
        let result: Self = serde_json::from_slice(&io(fs::read(root.join("session.json")))?)
            .map_err(|_| ProbeError::InvalidState)?;
        validate_remote(&result.remote)?;
        if result.schema_version != 1
            || io(fs::read_to_string(root.join("owner")))? != result.id.to_string()
        {
            return Err(ProbeError::Ownership);
        }
        Ok(result)
    }
    pub fn new(root: &Path, remote: &str, identity: Identity) -> ProbeResult<Self> {
        validate_remote(remote)?;
        io(fs::create_dir(root))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            io(fs::set_permissions(root, fs::Permissions::from_mode(0o700)))?;
        }
        let session = Self {
            schema_version: 1,
            id: Uuid::new_v4(),
            remote: remote.into(),
            identity,
            initialized: false,
            previewed: false,
            failed: false,
            owned: false,
            cleaned: false,
            stages: vec![],
            benchmark: None,
            sample_tree: None,
        };
        io(fs::write(root.join("owner"), session.id.to_string()))?;
        for name in ["local", "work", "recovery", "uploads"] {
            io(fs::create_dir(root.join(name)))?;
        }
        session.persist(root)?;
        Ok(session)
    }
    pub fn persist(&self, root: &Path) -> ProbeResult<()> {
        save(&root.join("session.json"), self)
    }
    pub fn remote_root(&self) -> String {
        format!("{}:CloudMounter-Prototype-{}", self.remote, self.id)
    }
    pub fn tree(&self) -> String {
        format!("{}/tree", self.remote_root())
    }
    fn marker(&self) -> String {
        format!(".cloud-mounter-onedrive-anchor-{}", self.id)
    }
    pub fn connection(&self, root: &Path) -> crate::model::Connection {
        use crate::model::*;
        Connection {
            id: ConnectionId::from_uuid(self.id),
            name: "onedrive_prototype".into(),
            provider: Provider::OneDrive,
            mode: ConnectionMode::OfflineMirror(OfflineMirrorConfig {
                engine: None,
                recovery_directory: root.join("recovery"),
                sync_interval_minutes: 5,
                sync_on_metered: false,
            }),
            remote_reference: self.remote.clone(),
            remote_subpath: Some(format!("CloudMounter-Prototype-{}/tree", self.id)),
            local_path: root.join("local"),
            enabled: true,
            vpn_profile_id: None,
            disconnect_vpn_when_unused: false,
            tuning_profile: TuningProfile::Balanced,
            smb_preload_override: None,
            sftp_preload_override: None,
            onedrive_identity: None,
            teams_identity: None,
        }
    }
    pub fn plan(&self, root: &Path) -> ProbeResult<crate::sync::RcloneBisyncPlan> {
        if self.cleaned {
            return Err(ProbeError::InvalidState);
        }
        crate::sync::onedrive_prototype_plan(
            &self.connection(root),
            &root.join("work"),
            format!("{}/recovery", self.remote_root()),
        )
        .map_err(|_| ProbeError::InvalidState)
    }
}
pub struct Engine<'a, R, G> {
    pub runner: &'a R,
    pub graph: &'a G,
    pub account: &'a str,
    pub root: PathBuf,
    pub session: Session,
    pub cancellation: CancellationToken,
}
impl<R: CommandRunner, G: Graph> Engine<'_, R, G> {
    async fn exec(
        &mut self,
        operation: &str,
        args: Vec<String>,
        timeout: u64,
    ) -> ProbeResult<String> {
        let start = Instant::now();
        let values = args.iter().map(String::as_str).collect::<Vec<_>>();
        let result = command(
            self.runner,
            &values,
            timeout,
            self.cancellation.child_token(),
        )
        .await;
        self.session.stages.push(Stage {
            operation: operation.into(),
            elapsed_ms: start.elapsed().as_millis(),
            succeeded: result.is_ok(),
        });
        self.session.persist(&self.root)?;
        result
    }
    pub async fn guard(&self, access: bool) -> ProbeResult<()> {
        if self.session.cleaned {
            return Err(ProbeError::InvalidState);
        }
        if verify(
            self.runner,
            self.graph,
            &self.session.remote,
            self.session.identity.drive_type,
            self.account,
            self.cancellation.child_token(),
        )
        .await?
            != self.session.identity
        {
            return Err(ProbeError::WrongDrive);
        }
        if self.session.owned {
            let owner = command(
                self.runner,
                &[
                    "cat",
                    &format!("{}/.prototype-owner", self.session.remote_root()),
                ],
                30,
                self.cancellation.child_token(),
            )
            .await?;
            if owner.trim_end() != self.session.id.to_string() {
                return Err(ProbeError::Ownership);
            }
        }
        if access {
            let expected = self.session.id.to_string();
            if io(fs::read_to_string(
                self.root.join("local").join(self.session.marker()),
            ))?
            .trim_end()
                != expected
            {
                return Err(ProbeError::Ownership);
            }
            let remote = command(
                self.runner,
                &[
                    "cat",
                    &format!("{}/{}", self.session.tree(), self.session.marker()),
                ],
                30,
                self.cancellation.child_token(),
            )
            .await?;
            if remote.trim_end() != expected {
                return Err(ProbeError::Ownership);
            }
        }
        Ok(())
    }
    async fn setup_owned_tree(&mut self) -> ProbeResult<()> {
        self.guard(false).await?;
        self.exec("mkdir", vec!["mkdir".into(), self.session.tree()], 30)
            .await?;
        self.exec(
            "ownership",
            vec![
                "copyto".into(),
                self.root.join("owner").display().to_string(),
                format!("{}/.prototype-owner", self.session.remote_root()),
            ],
            30,
        )
        .await?;
        self.session.owned = true;
        self.session.persist(&self.root)
    }
    pub async fn setup_samples(&mut self, source: &Path) -> ProbeResult<()> {
        if !source.is_absolute() || io(fs::symlink_metadata(source))?.file_type().is_symlink() {
            return Err(ProbeError::InvalidInput);
        }
        let source = io(fs::canonicalize(source))?;
        let root = io(fs::canonicalize(&self.root))?;
        if source.starts_with(&root) || root.starts_with(&source) || !source.is_dir() {
            return Err(ProbeError::InvalidInput);
        }
        let before = sample_inventory(&source, &self.cancellation)?;
        if before.files == 0 {
            return Err(ProbeError::InvalidInput);
        }
        let destination = self.root.join("local/sample");
        self.exec(
            "sample-copy",
            vec![
                "copy".into(),
                source.display().to_string(),
                destination.display().to_string(),
                "--create-empty-src-dirs".into(),
            ],
            600,
        )
        .await?;
        if before != sample_inventory(&source, &self.cancellation)?
            || before != sample_inventory(&destination, &self.cancellation)?
        {
            return Err(ProbeError::VerificationFailed);
        }
        self.session.sample_tree = Some(before);
        self.session.persist(&self.root)?;
        io(fs::write(
            self.root.join("local/timing-change.txt"),
            "synthetic timing baseline\n",
        ))?;
        self.setup_owned_tree().await?;
        self.sync("initial-preview").await
    }
    pub async fn setup(&mut self) -> ProbeResult<()> {
        self.setup_owned_tree().await?;
        io(fs::write(
            self.root.join("local/local.txt"),
            "local initial synthetic fixture\n",
        ))?;
        self.upload("remote.txt", b"remote initial synthetic fixture\n")
            .await?;
        self.sync("initial-preview").await?;
        if self.root.join("local/remote.txt").exists() {
            return Err(ProbeError::VerificationFailed);
        }
        Ok(())
    }
    async fn upload(&mut self, name: &str, data: &[u8]) -> ProbeResult<()> {
        if name.contains('/') || name.starts_with('.') {
            return Err(ProbeError::InvalidInput);
        }
        self.guard(self.session.initialized).await?;
        let path = self.root.join("uploads").join(name);
        io(fs::write(&path, data))?;
        self.exec(
            "remote-fixture",
            vec![
                "copyto".into(),
                path.display().to_string(),
                format!("{}/{name}", self.session.tree()),
                "--ignore-times".into(),
            ],
            120,
        )
        .await?;
        Ok(())
    }
    pub async fn sync(&mut self, kind: &str) -> ProbeResult<()> {
        use crate::bisync_coordinator::{self, BisyncMode};
        if self.session.failed {
            return Err(ProbeError::InvalidState);
        }
        let mode = BisyncMode::parse(kind).ok_or(ProbeError::InvalidInput)?;
        let plan = self.session.plan(&self.root)?;
        self.guard(self.session.initialized).await?;
        let start = Instant::now();
        let result = bisync_coordinator::run_onedrive_prototype(
            &self.session.connection(&self.root),
            mode,
            &self.root.join("work"),
            &plan,
            self.runner,
            self.cancellation.child_token(),
        )
        .await;
        self.session.stages.push(Stage {
            operation: kind.into(),
            elapsed_ms: start.elapsed().as_millis(),
            succeeded: result.is_ok(),
        });
        if result.is_err() {
            self.session.failed = true;
            self.session.previewed = false;
            self.session.persist(&self.root)?;
            return Err(ProbeError::CommandFailed);
        }
        self.session.initialized = plan.work_directory.join("initial-sync-complete").is_file();
        self.session.previewed = plan
            .work_directory
            .join("initial-preview-confirmable")
            .is_file();
        self.session.persist(&self.root)
    }
    pub async fn verify_bytes(&mut self) -> ProbeResult<()> {
        if !self.session.initialized || self.session.failed {
            return Err(ProbeError::InvalidState);
        }
        self.guard(true).await?;
        self.exec(
            "verify-download",
            vec![
                "check".into(),
                self.session.tree(),
                self.root.join("local").display().to_string(),
                "--download".into(),
            ],
            900,
        )
        .await?;
        Ok(())
    }
    pub async fn scenario(&mut self) -> ProbeResult<()> {
        if !self.session.initialized || self.session.failed || self.session.sample_tree.is_some() {
            return Err(ProbeError::InvalidState);
        }
        self.verify_bytes().await?;
        io(fs::write(
            self.root.join("local/local.txt"),
            "local modified synthetic fixture with changed size\n",
        ))?;
        self.upload(
            "remote.txt",
            b"remote modified synthetic fixture with changed size\n",
        )
        .await?;
        self.sync("sync").await?;
        self.verify_bytes().await?;
        if io(fs::read(self.root.join("local/remote.txt")))?
            != b"remote modified synthetic fixture with changed size\n"
        {
            return Err(ProbeError::VerificationFailed);
        }
        io(fs::remove_file(self.root.join("local/local.txt")))?;
        self.guard(true).await?;
        self.exec(
            "remote-delete",
            vec![
                "deletefile".into(),
                format!("{}/remote.txt", self.session.tree()),
            ],
            30,
        )
        .await?;
        self.sync("sync").await?;
        self.verify_bytes().await?;
        if self.root.join("local/remote.txt").exists() {
            return Err(ProbeError::VerificationFailed);
        }
        io(fs::write(
            self.root.join("local/conflict.txt"),
            "conflict baseline",
        ))?;
        self.sync("sync").await?;
        // Explicit delay covers remote whole-second timestamps; bounded, no polling.
        tokio::time::sleep(Duration::from_millis(1100)).await;
        io(fs::write(
            self.root.join("local/conflict.txt"),
            "local conflict version",
        ))?;
        self.upload(
            "conflict.txt",
            b"remote conflict version with different length",
        )
        .await?;
        self.sync("sync").await?;
        self.verify_bytes().await?;
        let files = io(fs::read_dir(self.root.join("local")))?
            .filter_map(Result::ok)
            .filter(|p| p.file_name().to_string_lossy().starts_with("conflict"))
            .filter_map(|p| fs::read(p.path()).ok())
            .collect::<Vec<_>>();
        if !files.iter().any(|v| v == b"local conflict version")
            || !files
                .iter()
                .any(|v| v == b"remote conflict version with different length")
        {
            return Err(ProbeError::VerificationFailed);
        }
        // Above the default 10 MiB OneDrive upload chunk size.
        let payload = vec![0x5a; 16 * 1024 * 1024];
        io(fs::write(self.root.join("local/large.bin"), &payload))?;
        self.sync("sync").await?;
        self.verify_bytes().await?;
        self.benchmark().await
    }
    pub async fn benchmark(&mut self) -> ProbeResult<()> {
        let start = Instant::now();
        self.sync("sync").await?;
        let sync_ms = start.elapsed().as_millis();
        let start = Instant::now();
        self.sync("sync").await?;
        self.guard(true).await?;
        self.exec(
            "verify-checksum",
            vec![
                "check".into(),
                self.session.tree(),
                self.root.join("local").display().to_string(),
                "--checksum".into(),
            ],
            900,
        )
        .await?;
        let verified_ms = start.elapsed().as_millis();
        self.session.benchmark = Some(
            serde_json::json!({"interval_seconds":300,"sync_ms":sync_ms,"sync_and_full_checksum_verify_ms":verified_ms,
            "representative_tree":false,"scope":if self.session.sample_tree.is_some(){"operator_sample_tree"}else{"synthetic_16mib_tree"},
            "sample_tree":self.session.sample_tree,"cadence_decision":"operator_measurement_required","cadence_exercised":false}),
        );
        self.session.persist(&self.root)
    }
    pub async fn timing(&mut self) -> ProbeResult<()> {
        if self.session.sample_tree.is_none() || !self.session.initialized || self.session.failed {
            return Err(ProbeError::InvalidState);
        }
        self.benchmark().await?;
        io(fs::write(
            self.root.join("local/timing-change.txt"),
            format!("synthetic timing edit {}\n", Uuid::new_v4()),
        ))?;
        let start = Instant::now();
        self.sync("sync").await?;
        let change_ms = start.elapsed().as_millis();
        self.verify_bytes().await?;
        let initial_ms = self
            .session
            .stages
            .iter()
            .find(|s| s.operation == "initial-sync" && s.succeeded)
            .map(|s| s.elapsed_ms);
        let result = self
            .session
            .benchmark
            .as_mut()
            .ok_or(ProbeError::InvalidState)?;
        result["small_change_sync_ms"] = serde_json::json!(change_ms);
        result["initial_sync_engine_ms"] = serde_json::json!(initial_ms);
        result["source_policy"] = serde_json::json!("read_only_copy_verified_at_prepare");
        self.session.persist(&self.root)
    }
    pub async fn fault(&mut self, cancel_upload: bool) -> ProbeResult<()> {
        use crate::{
            bisync_coordinator::{self, BisyncMode},
            offline_coordinator::{self, OfflineOperationState},
        };
        if self.session.failed || !self.session.initialized {
            return Err(ProbeError::InvalidState);
        }
        let plan = self.session.plan(&self.root)?;
        self.guard(true).await?;
        io(fs::write(
            self.root.join("local/pending.bin"),
            vec![0x71; 16 * 1024 * 1024],
        ))?;
        let cancellation = self.cancellation.child_token();
        let fault = FaultRunner {
            inner: self.runner,
            cancel_upload,
            cancellation: cancellation.clone(),
            observed_expected_failure: std::sync::atomic::AtomicBool::new(false),
        };
        self.session.failed = true;
        self.session.persist(&self.root)?;
        let start = Instant::now();
        let result = bisync_coordinator::run_onedrive_prototype(
            &self.session.connection(&self.root),
            BisyncMode::Sync,
            &self.root.join("work"),
            &plan,
            &fault,
            cancellation,
        )
        .await;
        let state = offline_coordinator::read_state(&offline_coordinator::state_path(
            &self.root.join("work"),
            crate::model::ConnectionId::from_uuid(self.session.id),
        ))
        .map_err(|_| ProbeError::InvalidState)?;
        let expected = result.is_err()
            && fault
                .observed_expected_failure
                .load(std::sync::atomic::Ordering::SeqCst)
            && state.is_some_and(|s| {
                s.state
                    == if cancel_upload {
                        OfflineOperationState::Cancelled
                    } else {
                        OfflineOperationState::Failed
                    }
            });
        self.session.stages.push(Stage {
            operation: if cancel_upload {
                "cancel-upload"
            } else {
                "network-unavailable"
            }
            .into(),
            elapsed_ms: start.elapsed().as_millis(),
            succeeded: expected,
        });
        self.session.persist(&self.root)?;
        if !expected {
            return Err(ProbeError::VerificationFailed);
        }
        Ok(())
    }
    pub async fn repair_inspection(&mut self) -> ProbeResult<Value> {
        self.guard(true).await?;
        let locks = io(fs::read_dir(self.session.plan(&self.root)?.work_directory))?
            .filter_map(Result::ok)
            .filter(|p| p.file_name().to_string_lossy().ends_with(".lck"))
            .count();
        // Deliberately preserve locks, pending data and all listings; never implicit --resync.
        Ok(
            serde_json::json!({"failed":self.session.failed,"locks_preserved":locks,"pending_bytes_preserved":self.root.join("local/pending.bin").exists(),"implicit_resync":false}),
        )
    }
    pub async fn cleanup(&mut self) -> ProbeResult<()> {
        if !self.session.owned {
            return Err(ProbeError::Ownership);
        }
        self.guard(false).await?;
        self.exec(
            "cleanup",
            vec!["purge".into(), self.session.remote_root()],
            120,
        )
        .await?;
        self.session.cleaned = true;
        self.session.persist(&self.root)
    }
}

struct FaultRunner<'a, R> {
    inner: &'a R,
    cancel_upload: bool,
    cancellation: CancellationToken,
    observed_expected_failure: std::sync::atomic::AtomicBool,
}
impl<R: CommandRunner> CommandRunner for FaultRunner<'_, R> {
    fn resolve(&self, executable: Executable) -> Option<PathBuf> {
        self.inner.resolve(executable)
    }
    fn run<'a>(
        &'a self,
        mut request: CommandRequest,
        cancel: CancellationToken,
    ) -> crate::process::CommandFuture<'a> {
        Box::pin(async move {
            // The shared coordinator puts fixed global timeout options before
            // the subcommand. Match the typed argument, not positional CLI text.
            let is_bisync = request.executable == Executable::Rclone
                && request
                    .args
                    .contains(&crate::process::CommandArg::plain("bisync")?);
            let mut timer = None;
            if is_bisync {
                request = request.with_timeout(Duration::from_secs(30));
                let arguments = if self.cancel_upload {
                    vec!["--bwlimit", "64k", "--transfers", "1", "--disable", "Copy"]
                } else {
                    vec![
                        "--bind",
                        "192.0.2.1",
                        "--contimeout",
                        "1s",
                        "--timeout",
                        "1s",
                        "--retries",
                        "1",
                        "--low-level-retries",
                        "1",
                    ]
                };
                for arg in arguments {
                    request = request.arg(arg)?;
                }
                if self.cancel_upload {
                    let token = self.cancellation.clone();
                    timer = Some(tokio::spawn(async move {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        token.cancel();
                    }));
                }
            }
            let result = self.inner.run(request, cancel).await;
            if is_bisync {
                use crate::process::CommandError;
                let expected = if self.cancel_upload {
                    matches!(result, Err(CommandError::Cancelled { .. }))
                } else {
                    matches!(
                        result,
                        Err(CommandError::NonZero { .. } | CommandError::Timeout { .. })
                    )
                };
                self.observed_expected_failure
                    .store(expected, std::sync::atomic::Ordering::SeqCst);
            }
            if let Some(timer) = timer {
                timer.abort();
                let _ = timer.await;
            }
            result
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{
        CapturedOutput, CommandArg, CommandError, CommandOutput, FakeCommandRunner,
        SystemCommandRunner,
    };
    use serde_json::json;
    struct FixedGraph {
        account: &'static str,
        kind: &'static str,
        drive: &'static str,
    }
    impl Default for FixedGraph {
        fn default() -> Self {
            Self {
                account: "person@example.invalid",
                kind: "personal",
                drive: "primary",
            }
        }
    }
    impl Graph for FixedGraph {
        fn get<'a>(
            &'a self,
            target: GraphTarget<'a>,
            _token: &'a str,
            _cancel: CancellationToken,
        ) -> GraphFuture<'a> {
            Box::pin(async move {
                Ok(if matches!(target, GraphTarget::Profile) {
                    json!({"id":"synthetic-account","userPrincipalName":self.account})
                } else {
                    json!({"id":self.drive,"driveType":self.kind,"owner":{"user":{"id":"synthetic-account"}}})
                })
            })
        }
    }
    fn output(text: &str) -> CommandOutput {
        CommandOutput {
            command: "synthetic".into(),
            stdout: CapturedOutput {
                text: text.into(),
                truncated: false,
                invalid_utf8: false,
            },
            stderr: CapturedOutput {
                text: String::new(),
                truncated: false,
                invalid_utf8: false,
            },
            attempts: 1,
            duration: Duration::ZERO,
        }
    }
    fn config(remote: &str, drive: &str) -> String {
        json!({remote:{"type":"onedrive","drive_type":"personal","drive_id":drive,"token":json!({"access_token":"synthetic-access"}).to_string()}}).to_string()
    }
    fn push_verify(r: &FakeCommandRunner, remote: &str, drive: &str) {
        r.push(Ok(output("{}")));
        r.push(Ok(output(&config(remote, drive))));
    }
    fn identity() -> Identity {
        Identity {
            drive_fingerprint: fingerprint("primary"),
            account_fingerprint: fingerprint("synthetic-account"),
            drive_type: DriveType::Personal,
        }
    }
    #[test]
    fn onedrive_prototype_sample_inventory_rejects_links_oversize_and_cancellation() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("file"), "synthetic sample").unwrap();
        let cancel = CancellationToken::new();
        let before = sample_inventory(temp.path(), &cancel).unwrap();
        assert_eq!(before.files, 1);
        assert_eq!(before.bytes, 16);
        std::os::unix::fs::symlink(temp.path().join("file"), temp.path().join("link")).unwrap();
        assert_eq!(
            sample_inventory(temp.path(), &cancel),
            Err(ProbeError::InvalidInput)
        );
        fs::remove_file(temp.path().join("link")).unwrap();
        fs::File::create(temp.path().join("oversize"))
            .unwrap()
            .set_len(2 * 1024 * 1024 * 1024 + 1)
            .unwrap();
        assert_eq!(
            sample_inventory(temp.path(), &cancel),
            Err(ProbeError::InvalidInput)
        );
        cancel.cancel();
        assert_eq!(
            sample_inventory(temp.path(), &cancel),
            Err(ProbeError::Cancelled)
        );
    }
    #[test]
    fn onedrive_prototype_business_graph_target_is_fixed_and_upn_is_encoded() {
        let url = graph_url(GraphTarget::ExpectedBusinessDrive(
            "guest#EXT#@example.invalid",
        ))
        .unwrap();
        assert_eq!(url.host_str(), Some("graph.microsoft.com"));
        assert_eq!(
            url.path(),
            "/v1.0/users/guest%23EXT%23@example.invalid/drive"
        );
        assert_eq!(url.query(), Some("$select=id,driveType,owner"));
        for account in [
            "",
            "not-upn",
            "@domain",
            "name@",
            "a@b@c",
            "a@b/../me",
            "a@b?select=id",
            "a@b\nc",
        ] {
            assert!(graph_url(GraphTarget::ExpectedBusinessDrive(account)).is_err());
        }
    }
    #[tokio::test]
    async fn onedrive_prototype_business_binds_signed_in_and_expected_primary_without_profile() {
        use std::collections::VecDeque;
        use std::sync::Mutex;
        struct Drives(Mutex<VecDeque<ProbeResult<Value>>>);
        impl Graph for Drives {
            fn get<'a>(
                &'a self,
                target: GraphTarget<'a>,
                _token: &'a str,
                _cancel: CancellationToken,
            ) -> GraphFuture<'a> {
                match target {
                    GraphTarget::Folder { .. } => {
                        panic!("prototype has no selected-folder verification")
                    }
                    GraphTarget::Profile => panic!("business must not request profile access"),
                    GraphTarget::ExpectedBusinessDrive(upn) => {
                        assert_eq!(upn, "person@example.invalid")
                    }
                    GraphTarget::PrimaryDrive => assert_eq!(self.0.lock().unwrap().len(), 2),
                }
                Box::pin(async move { self.0.lock().unwrap().pop_front().unwrap() })
            }
        }
        let good = json!({"id":"primary","driveType":"business","owner":{"user":{"id":"synthetic-account"}}});
        let mut wrong_drive = good.clone();
        wrong_drive["id"] = json!("other-primary");
        let mut wrong_owner = good.clone();
        wrong_owner["owner"]["user"]["id"] = json!("other-user");
        let mut library = good.clone();
        library["driveType"] = json!("documentLibrary");
        let mut missing_owner = good.clone();
        missing_owner["owner"] = json!({});
        let mut empty_id = good.clone();
        empty_id["id"] = json!("");
        for (signed_in, expected, outcome) in [
            (good.clone(), Ok(good.clone()), None),
            (
                good.clone(),
                Ok(wrong_drive),
                Some(ProbeError::WrongAccount),
            ),
            (
                good.clone(),
                Ok(wrong_owner),
                Some(ProbeError::WrongAccount),
            ),
            (
                library.clone(),
                Ok(good.clone()),
                Some(ProbeError::WrongDrive),
            ),
            (good.clone(), Ok(library), Some(ProbeError::WrongDrive)),
            (
                missing_owner.clone(),
                Ok(good.clone()),
                Some(ProbeError::InvalidMetadata),
            ),
            (
                good.clone(),
                Ok(missing_owner),
                Some(ProbeError::InvalidMetadata),
            ),
            (
                good.clone(),
                Ok(empty_id),
                Some(ProbeError::InvalidMetadata),
            ),
            (
                good.clone(),
                Err(ProbeError::GraphHttp(403)),
                Some(ProbeError::GraphHttp(403)),
            ),
            (
                good.clone(),
                Err(ProbeError::GraphHttp(404)),
                Some(ProbeError::GraphHttp(404)),
            ),
        ] {
            let graph = Drives(Mutex::new(VecDeque::from([Ok(signed_in), expected])));
            let result = primary(
                &graph,
                "synthetic-access",
                DriveType::Business,
                "person@example.invalid",
                CancellationToken::new(),
            )
            .await;
            if let Some(error) = outcome {
                assert_eq!(result, Err(error));
            } else {
                let (id, identity) = result.unwrap();
                assert_eq!(id, "primary");
                assert_eq!(
                    identity.account_fingerprint,
                    fingerprint("synthetic-account")
                );
                assert_eq!(identity.drive_type, DriveType::Business);
            }
        }
    }
    #[tokio::test]
    async fn onedrive_prototype_business_auth_uses_default_scopes_and_rejects_cache_config() {
        let mut config: Value = serde_json::from_str(&config("cm_proto_test", "primary")).unwrap();
        config["cm_proto_test"]["drive_type"] = json!("business");
        let runner = FakeCommandRunner::default();
        runner.push(Ok(output("{}")));
        runner.push(Ok(output(r#"{"State":"drive-state","Option":{"Name":"config_driveid","Examples":[{"Value":"cache"},{"Value":"primary"}]}}"#)));
        runner.push(Ok(output(&config.to_string())));
        runner.push(Ok(output(
            r#"{"State":"confirm-state","Option":{"Name":"config_drive_ok"}}"#,
        )));
        runner.push(Ok(output(r#"{"State":""}"#)));
        runner.push(Ok(output("{}")));
        runner.push(Ok(output(&config.to_string())));
        let graph = FixedGraph {
            kind: "business",
            ..FixedGraph::default()
        };
        let result = authenticate(
            &runner,
            &graph,
            "cm_proto_test",
            DriveType::Business,
            "person@example.invalid",
            CancellationToken::new(),
        )
        .await
        .unwrap();
        assert_eq!(result.drive_type, DriveType::Business);
        for request in runner.requests() {
            let command = request.sanitized_command();
            assert!(!command.contains("User.Read"));
            assert!(!command.contains("access_scopes"));
        }
        config["cm_proto_test"]["drive_id"] = json!("cache");
        let runner = FakeCommandRunner::default();
        runner.push(Ok(output("{}")));
        runner.push(Ok(output(&config.to_string())));
        assert_eq!(
            verify(
                &runner,
                &graph,
                "cm_proto_test",
                DriveType::Business,
                "person@example.invalid",
                CancellationToken::new()
            )
            .await,
            Err(ProbeError::WrongDrive)
        );
    }
    #[tokio::test]
    async fn onedrive_prototype_verify_retry_is_read_only_and_reports_safe_failures() {
        for (failed_at, operation) in [
            (0, RcloneOperation::IdentityAccess),
            (1, RcloneOperation::Config),
            (2, RcloneOperation::OwnerRead),
            (3, RcloneOperation::AnchorRead),
            (4, RcloneOperation::DownloadVerification),
        ] {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().join("session");
            let mut session = Session::new(&root, "probe", identity()).unwrap();
            session.initialized = true;
            session.owned = true;
            fs::write(
                root.join("local").join(session.marker()),
                session.id.to_string(),
            )
            .unwrap();
            session.persist(&root).unwrap();
            let owner = session.id.to_string();
            let responses = [
                "{}".to_owned(),
                config("probe", "primary"),
                owner.clone(),
                owner,
                "".to_owned(),
            ];
            let runner = FakeCommandRunner::default();
            let mut engine = Engine {
                runner: &runner,
                graph: &FixedGraph::default(),
                account: "person@example.invalid",
                root: root.clone(),
                session,
                cancellation: CancellationToken::new(),
            };
            for response in responses.iter().take(failed_at) {
                runner.push(Ok(output(response)));
            }
            let private = output("synthetic-secret-provider-body");
            runner.push(Err(CommandError::NonZero {
                command: "synthetic-secret-command".into(),
                code: Some(3),
                stdout: private.stdout.clone(),
                stderr: private.stdout,
                attempts: 1,
            }));
            let error = engine.verify_bytes().await.unwrap_err();
            assert_eq!(
                error,
                ProbeError::RcloneFailed {
                    operation,
                    failure: RcloneFailure::NonZero(Some(3))
                }
            );
            let diagnostic = serde_json::to_string(&error).unwrap();
            assert!(!diagnostic.contains("synthetic-secret"));
            // A retry can verify this initialized session without redoing bisync.
            for response in &responses {
                runner.push(Ok(output(response)));
            }
            engine.verify_bytes().await.unwrap();
            assert!(Session::load(&root).unwrap().initialized);
            assert!(!Session::load(&root).unwrap().failed);
            assert!(runner.requests().iter().all(|request| {
                ["lsjson", "config", "cat", "check"]
                    .iter()
                    .any(|name| request.args[0] == CommandArg::plain(*name).unwrap())
            }));
        }
    }
    #[tokio::test]
    async fn onedrive_prototype_blocks_wrong_account_cache_library_and_drive_type() {
        let r = FakeCommandRunner::default();
        push_verify(&r, "probe", "primary");
        assert_eq!(
            verify(
                &r,
                &FixedGraph::default(),
                "probe",
                DriveType::Personal,
                "other@example.invalid",
                CancellationToken::new()
            )
            .await,
            Err(ProbeError::WrongAccount)
        );
        let r = FakeCommandRunner::default();
        push_verify(&r, "probe", "cache");
        assert_eq!(
            verify(
                &r,
                &FixedGraph::default(),
                "probe",
                DriveType::Personal,
                "person@example.invalid",
                CancellationToken::new()
            )
            .await,
            Err(ProbeError::WrongDrive)
        );
        let r = FakeCommandRunner::default();
        push_verify(&r, "probe", "primary");
        assert_eq!(
            verify(
                &r,
                &FixedGraph {
                    kind: "business",
                    ..FixedGraph::default()
                },
                "probe",
                DriveType::Personal,
                "person@example.invalid",
                CancellationToken::new()
            )
            .await,
            Err(ProbeError::WrongDrive)
        );
    }
    #[tokio::test]
    async fn onedrive_prototype_business_and_default_client_validation() {
        let r = FakeCommandRunner::default();
        r.push(Ok(output("{}")));
        let mut value: Value = serde_json::from_str(&config("probe", "primary")).unwrap();
        value["probe"]["drive_type"] = json!("business");
        r.push(Ok(output(&value.to_string())));
        let graph = FixedGraph {
            kind: "business",
            ..FixedGraph::default()
        };
        assert_eq!(
            verify(
                &r,
                &graph,
                "probe",
                DriveType::Business,
                "person@example.invalid",
                CancellationToken::new()
            )
            .await
            .unwrap()
            .drive_type,
            DriveType::Business
        );
        for client in [json!("custom"), json!(42)] {
            let r = FakeCommandRunner::default();
            r.push(Ok(output("{}")));
            value["probe"]["client_id"] = client;
            r.push(Ok(output(&value.to_string())));
            assert_eq!(
                verify(
                    &r,
                    &graph,
                    "probe",
                    DriveType::Business,
                    "person@example.invalid",
                    CancellationToken::new()
                )
                .await,
                Err(ProbeError::CustomClient)
            );
        }
    }
    #[tokio::test]
    async fn onedrive_prototype_auth_rejects_existing_remote_and_rolls_back_only_new_partial() {
        let r = FakeCommandRunner::default();
        r.push(Ok(output(&config("cm_proto_test", "primary"))));
        assert_eq!(
            authenticate(
                &r,
                &FixedGraph::default(),
                "cm_proto_test",
                DriveType::Personal,
                "person@example.invalid",
                CancellationToken::new()
            )
            .await,
            Err(ProbeError::ExistingRemote)
        );
        assert_eq!(r.requests().len(), 1);
        let r = FakeCommandRunner::default();
        r.push(Ok(output("{}")));
        r.push(Ok(output(
            r#"{"State":"synthetic-state","Error":"","Option":{"Name":"unexpected"}}"#,
        )));
        r.push(Ok(output(&config("cm_proto_test", "primary"))));
        r.push(Ok(output("")));
        assert_eq!(
            authenticate(
                &r,
                &FixedGraph::default(),
                "cm_proto_test",
                DriveType::Personal,
                "person@example.invalid",
                CancellationToken::new()
            )
            .await,
            Err(ProbeError::UnsupportedQuestion)
        );
        assert!(
            r.requests()
                .last()
                .unwrap()
                .sanitized_command()
                .contains("config delete cm_proto_test")
        );
    }
    #[tokio::test]
    async fn onedrive_prototype_auth_selects_graph_primary_and_redacts_continuation_state() {
        let r = FakeCommandRunner::default();
        r.push(Ok(output("{}")));
        r.push(Ok(output(r#"{"State":"private-state","Error":"","Option":{"Name":"config_driveid","Examples":[{"Value":"cache"},{"Value":"primary"}]}}"#)));
        r.push(Ok(output(&config("cm_proto_test", "primary"))));
        r.push(Ok(output(
            r#"{"State":"next-state","Error":"","Option":{"Name":"config_drive_ok"}}"#,
        )));
        r.push(Ok(output(r#"{"State":"","Error":""}"#)));
        push_verify(&r, "cm_proto_test", "primary");
        assert_eq!(
            authenticate(
                &r,
                &FixedGraph::default(),
                "cm_proto_test",
                DriveType::Personal,
                "person@example.invalid",
                CancellationToken::new()
            )
            .await
            .unwrap(),
            identity()
        );
        let requests = r.requests();
        assert!(requests[1].args.windows(2).any(|pair| {
            pair[0] == CommandArg::plain("access_scopes").unwrap()
                && pair[1] == CommandArg::plain(crate::microsoft_oauth::PERSONAL_SCOPES).unwrap()
        }));
        for request in requests {
            let text = request.sanitized_command();
            assert!(!text.contains("private-state"));
            assert!(!text.contains("synthetic-access"));
        }
    }
    #[tokio::test]
    async fn onedrive_prototype_profile_denial_stops_identity_check_without_mutation() {
        struct DeniedGraph(u16);
        impl Graph for DeniedGraph {
            fn get<'a>(
                &'a self,
                target: GraphTarget<'a>,
                _token: &'a str,
                _cancel: CancellationToken,
            ) -> GraphFuture<'a> {
                assert!(matches!(target, GraphTarget::Profile));
                Box::pin(async move { Err(ProbeError::GraphHttp(self.0)) })
            }
        }
        for status in [401, 403, 429] {
            let runner = FakeCommandRunner::default();
            push_verify(&runner, "probe", "primary");
            let result = verify(
                &runner,
                &DeniedGraph(status),
                "probe",
                DriveType::Personal,
                "person@example.invalid",
                CancellationToken::new(),
            )
            .await;
            assert_eq!(
                result,
                Err(if status == 429 {
                    ProbeError::GraphHttp(status)
                } else {
                    ProbeError::AccountProfileUnauthorized(status)
                })
            );
            let requests = runner.requests();
            assert_eq!(requests.len(), 2);
            assert_eq!(requests[1].args[0], CommandArg::plain("config").unwrap());
            assert_eq!(requests[1].args[1], CommandArg::plain("dump").unwrap());
        }
    }
    #[test]
    fn onedrive_prototype_reuses_generic_plan_without_changing_production_routing() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("session");
        let s = Session::new(&root, "probe", identity()).unwrap();
        let plan = s.plan(&root).unwrap();
        let args = &plan.service.arguments;
        assert!(args.windows(2).any(|p| p == ["--compare", "size,modtime"]));
        for flag in [
            "--ignore-size",
            "--ignore-checksum",
            "--modify-window",
            "--check-sync",
        ] {
            assert!(!args.iter().any(|v| v == flag));
        }
        // The production SharePoint profile is unchanged by the shared extraction.
        let mut teams = s.connection(&root);
        teams.provider = crate::model::Provider::Teams;
        let mut library = crate::sharepoint::parse_library_url(
            "https://example.sharepoint.com/sites/team/Documents",
        )
        .unwrap();
        library.drive_id = "synthetic-library".into();
        teams.teams_identity = Some(library);
        let sharepoint = crate::sync::rclone_bisync_plan(&teams, &root.join("work")).unwrap();
        assert!(sharepoint.verify_after_sync);
        for flag in [
            "--ignore-size",
            "--ignore-checksum",
            "--modify-window",
            "--check-sync",
        ] {
            assert!(sharepoint.service.arguments.iter().any(|v| v == flag));
        }
        assert!(!plan.verify_after_sync);
        assert!(plan.access_marker_name.is_some());
        assert!(
            plan.remote_recovery_path
                .starts_with(&format!("{}/recovery", s.remote_root()))
        );
        assert!(crate::sync::rclone_bisync_plan(&s.connection(&root), &root.join("work")).is_err());
    }
    #[tokio::test]
    async fn onedrive_prototype_bad_ownership_blocks_coordinator_and_cleanup() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("session");
        let mut s = Session::new(&root, "probe", identity()).unwrap();
        s.owned = true;
        let r = FakeCommandRunner::default();
        push_verify(&r, "probe", "primary");
        r.push(Ok(output("wrong-owner")));
        let g = FixedGraph::default();
        let mut e = Engine {
            runner: &r,
            graph: &g,
            account: "person@example.invalid",
            root,
            session: s,
            cancellation: CancellationToken::new(),
        };
        assert_eq!(e.sync("initial-preview").await, Err(ProbeError::Ownership));
        assert!(
            !r.requests()
                .iter()
                .any(|v| v.sanitized_command().contains("bisync"))
        );
        push_verify(&r, "probe", "primary");
        r.push(Ok(output("wrong-owner")));
        assert_eq!(e.cleanup().await, Err(ProbeError::Ownership));
        assert!(
            !r.requests()
                .iter()
                .any(|v| v.sanitized_command().contains("purge"))
        );
    }
    #[tokio::test]
    async fn onedrive_prototype_coordinator_failure_retains_state_and_repair_preserves_lock() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("session");
        let mut s = Session::new(&root, "probe", identity()).unwrap();
        s.owned = true;
        let r = FakeCommandRunner::default();
        push_verify(&r, "probe", "primary");
        r.push(Ok(output(&s.id.to_string())));
        r.push(Err(CommandError::Cancelled {
            command: "synthetic".into(),
        }));
        let g = FixedGraph::default();
        let mut e = Engine {
            runner: &r,
            graph: &g,
            account: "person@example.invalid",
            root: root.clone(),
            session: s,
            cancellation: CancellationToken::new(),
        };
        assert_eq!(
            e.sync("initial-preview").await,
            Err(ProbeError::CommandFailed)
        );
        assert!(Session::load(&root).unwrap().failed);
        let plan = e.session.plan(&root).unwrap();
        fs::write(plan.work_directory.join("test.lck"), b"preserve lock").unwrap();
        fs::write(root.join("local/pending.bin"), b"preserve pending").unwrap();
        e.session.initialized = true;
        push_verify(&r, "probe", "primary");
        r.push(Ok(output(&e.session.id.to_string())));
        r.push(Ok(output(&e.session.id.to_string())));
        let before = r.requests().len();
        let result = e.repair_inspection().await.unwrap();
        assert_eq!(result["locks_preserved"], 1);
        assert_eq!(
            fs::read(plan.work_directory.join("test.lck")).unwrap(),
            b"preserve lock"
        );
        assert_eq!(
            fs::read(root.join("local/pending.bin")).unwrap(),
            b"preserve pending"
        );
        assert!(
            !r.requests()[before..]
                .iter()
                .any(|v| v.sanitized_command().contains("bisync"))
        );
    }
    #[test]
    fn onedrive_prototype_refuses_malformed_state_input_and_truncated_credentials() {
        for name in ["", "x:y", "--name\nother"] {
            assert_eq!(validate_remote(name), Err(ProbeError::InvalidInput));
        }
        let temp = tempfile::tempdir().unwrap();
        assert!(Session::load(temp.path()).is_err());
        let mut out = output("{}");
        out.stdout.truncated = true;
        assert_eq!(complete(&out), Err(ProbeError::IncompleteOutput));
    }
    #[tokio::test]
    async fn onedrive_prototype_fault_injection_handles_shared_global_options() {
        let runner = FakeCommandRunner::default();
        runner.push(Ok(output("")));
        let fault = FaultRunner {
            inner: &runner,
            cancel_upload: true,
            cancellation: CancellationToken::new(),
            observed_expected_failure: std::sync::atomic::AtomicBool::new(false),
        };
        fault
            .run(
                request(
                    &[
                        "--contimeout",
                        "10s",
                        "--timeout",
                        "30s",
                        "bisync",
                        "probe:owned/tree",
                        "/tmp/local",
                    ],
                    30,
                )
                .unwrap(),
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let command = runner.requests()[0].sanitized_command();
        assert!(command.contains("--bwlimit 64k"));
        assert!(command.contains("--disable Copy"));
    }
    struct AliasRunner {
        config: PathBuf,
    }
    impl CommandRunner for AliasRunner {
        fn resolve(&self, e: Executable) -> Option<PathBuf> {
            SystemCommandRunner.resolve(e)
        }
        fn run<'a>(
            &'a self,
            mut req: CommandRequest,
            c: CancellationToken,
        ) -> crate::process::CommandFuture<'a> {
            if req.args
                == [
                    CommandArg::plain("config").unwrap(),
                    CommandArg::plain("dump").unwrap(),
                ]
            {
                return Box::pin(async { Ok(output(&config("probe", "primary"))) });
            }
            req.args.splice(
                0..0,
                [
                    CommandArg::plain("--config").unwrap(),
                    CommandArg::plain(self.config.as_os_str()).unwrap(),
                ],
            );
            SystemCommandRunner.run(req, c)
        }
        fn private_config<'a>(
            &'a self,
            request: CommandRequest,
            cancel: CancellationToken,
        ) -> crate::process::PrivateConfigFuture<'a> {
            Box::pin(async move {
                self.run(request, cancel)
                    .await
                    .map(crate::process::PrivateConfigOutput::test_output)
            })
        }
    }
    #[tokio::test]
    #[ignore = "requires installed rclone; synthetic alias filesystem and Graph fake only, no user data/network"]
    async fn onedrive_prototype_local_sample_timing_preserves_originals() {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir().unwrap();
        let cloud = temp.path().join("cloud");
        fs::create_dir(&cloud).unwrap();
        let config = temp.path().join("rclone.conf");
        fs::write(
            &config,
            format!("[probe]\ntype = alias\nremote = {}\n", cloud.display()),
        )
        .unwrap();
        let source = temp.path().join("samples");
        fs::create_dir_all(source.join(".git/objects")).unwrap();
        fs::create_dir_all(source.join("nested/empty")).unwrap();
        fs::write(
            source.join(".git/objects/synthetic"),
            b"synthetic git object",
        )
        .unwrap();
        fs::write(source.join("nested/file.py"), b"synthetic Python fixture\n").unwrap();
        fs::write(source.join("small.bin"), [0_u8; 1024]).unwrap();
        fs::set_permissions(source.join("small.bin"), fs::Permissions::from_mode(0o444)).unwrap();
        let original = sample_inventory(&source, &CancellationToken::new()).unwrap();
        let runner = AliasRunner { config };
        let graph = FixedGraph::default();
        let root = temp.path().join("session");
        let session = Session::new(&root, "probe", identity()).unwrap();
        let mut engine = Engine {
            runner: &runner,
            graph: &graph,
            account: "person@example.invalid",
            root: root.clone(),
            session,
            cancellation: CancellationToken::new(),
        };
        assert_eq!(
            engine.setup_samples(&root).await,
            Err(ProbeError::InvalidInput)
        );
        engine.setup_samples(&source).await.unwrap();
        assert_eq!(engine.session.sample_tree, Some(original.clone()));
        assert!(!engine.session.initialized);
        assert!(engine.session.previewed);
        engine.sync("initial-sync").await.unwrap();
        engine.verify_bytes().await.unwrap();
        engine.session = Session::load(&root).unwrap();
        assert_eq!(engine.scenario().await, Err(ProbeError::InvalidState));
        engine.timing().await.unwrap();
        let result = engine.session.benchmark.as_ref().unwrap();
        assert_eq!(result["scope"], "operator_sample_tree");
        assert_eq!(result["representative_tree"], false);
        assert_eq!(result["cadence_exercised"], false);
        assert!(result["small_change_sync_ms"].is_u64());
        assert!(result["initial_sync_engine_ms"].is_u64());
        assert_eq!(
            original,
            sample_inventory(&source, &CancellationToken::new()).unwrap()
        );
        assert_eq!(
            original,
            sample_inventory(&root.join("local/sample"), &CancellationToken::new()).unwrap()
        );
        engine.cleanup().await.unwrap();
        assert!(fs::read_dir(&cloud).unwrap().next().is_none());
        assert!(root.join("local/sample/nested/file.py").is_file());
    }
    #[tokio::test]
    #[ignore = "requires installed rclone; synthetic alias filesystem and Graph fake only, no user data/network"]
    async fn onedrive_prototype_local_shared_coordinator_lifecycle() {
        let temp = tempfile::tempdir().unwrap();
        let cloud = temp.path().join("cloud");
        fs::create_dir(&cloud).unwrap();
        let config = temp.path().join("rclone.conf");
        fs::write(
            &config,
            format!("[probe]\ntype = alias\nremote = {}\n", cloud.display()),
        )
        .unwrap();
        let runner = AliasRunner { config };
        let graph = FixedGraph::default();
        let root = temp.path().join("session");
        let s = Session::new(&root, "probe", identity()).unwrap();
        let mut e = Engine {
            runner: &runner,
            graph: &graph,
            account: "person@example.invalid",
            root,
            session: s,
            cancellation: CancellationToken::new(),
        };
        e.setup().await.unwrap();
        assert!(e.session.previewed);
        assert!(!e.session.initialized);
        assert_eq!(e.verify_bytes().await, Err(ProbeError::InvalidState));
        e.sync("initial-sync").await.unwrap();
        e.session = Session::load(&e.root).unwrap();
        e.verify_bytes().await.unwrap();
        e.scenario().await.unwrap();
        e.fault(true).await.unwrap();
        assert!(e.session.failed);
        assert!(
            e.repair_inspection().await.unwrap()["pending_bytes_preserved"]
                .as_bool()
                .unwrap()
        );
        e.cleanup().await.unwrap();
        assert!(e.session.cleaned);
        assert!(fs::read_dir(&cloud).unwrap().next().is_none());
    }
}
