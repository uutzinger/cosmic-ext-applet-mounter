// SPDX-License-Identifier: MIT

//! Disposable engine integration. No saved applet config or remote credentials.
use cosmic_ext_applet_mounter::{
    bisync_coordinator::{BisyncMode, run},
    model::{
        Connection, ConnectionId, ConnectionMode, OfflineMirrorConfig, Provider, TuningProfile,
    },
    process::{
        CommandArg, CommandFuture, CommandRequest, CommandRunner, Executable, SystemCommandRunner,
    },
    sync::rclone_bisync_plan,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
};
use tokio_util::sync::CancellationToken;

struct IsolatedRunner {
    config: PathBuf,
}
impl CommandRunner for IsolatedRunner {
    fn resolve(&self, executable: Executable) -> Option<PathBuf> {
        SystemCommandRunner.resolve(executable)
    }
    fn run<'a>(
        &'a self,
        mut request: CommandRequest,
        cancellation: CancellationToken,
    ) -> CommandFuture<'a> {
        request.args.splice(
            0..0,
            [
                CommandArg::plain("--config").unwrap(),
                CommandArg::plain(self.config.as_os_str()).unwrap(),
            ],
        );
        SystemCommandRunner.run(request, cancellation)
    }
}
struct Server(Child);
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn recovery_files(root: &Path, prefix: &str) -> Vec<PathBuf> {
    fs::read_dir(root)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .flat_map(|entry| fs::read_dir(entry.path()).unwrap().filter_map(Result::ok))
        .filter(|entry| entry.file_name().to_string_lossy().starts_with(prefix))
        .map(|entry| entry.path())
        .collect()
}

async fn lifecycle(sftp: bool) {
    let root = tempfile::tempdir().unwrap();
    let remote = root.path().join("remote");
    fs::create_dir_all(remote.join("share")).unwrap();
    let config = root.path().join("rclone.conf");
    let mut server = None;
    if sftp {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let child = Command::new("rclone")
            .args(["serve", "sftp", "--no-auth", "--addr"])
            .arg(format!("127.0.0.1:{port}"))
            .arg("--cache-dir")
            .arg(root.path().join("cache"))
            .arg("--dir-cache-time")
            .arg("0s")
            .arg(&remote)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        server = Some(Server(child));
        for _ in 0..100 {
            if tokio::net::TcpStream::connect(("127.0.0.1", port))
                .await
                .is_ok()
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        fs::write(&config, format!("[probe]\ntype = sftp\nhost = 127.0.0.1\nport = {port}\nuser = disposable\nkey_use_agent = false\ndisable_hashcheck = true\n")).unwrap();
    } else {
        fs::write(
            &config,
            format!("[probe]\ntype = alias\nremote = {}\n", remote.display()),
        )
        .unwrap();
    }
    let runner = IsolatedRunner { config };
    let connection = Connection {
        id: ConnectionId::new(),
        name: "Disposable coordinator lifecycle".into(),
        provider: if sftp {
            Provider::Sftp
        } else {
            Provider::GoogleDrive
        },
        mode: ConnectionMode::OfflineMirror(OfflineMirrorConfig {
            recovery_directory: root.path().join("recovery"),
            sync_interval_minutes: 15,
            sync_on_metered: true,
        }),
        remote_reference: "probe".into(),
        remote_subpath: Some("share".into()),
        local_path: root.path().join("local"),
        enabled: true,
        vpn_profile_id: None,
        disconnect_vpn_when_unused: false,
        tuning_profile: TuningProfile::Balanced,
        smb_preload_override: None,
        sftp_preload_override: None,
        teams_identity: None,
    };
    let work = root.path().join("state");
    let plan = rclone_bisync_plan(&connection, &work).unwrap();
    fs::write(remote.join("share/remote.txt"), "remote-one").unwrap();
    for mode in [BisyncMode::InitialPreview, BisyncMode::InitialSync] {
        run(&connection, mode, &work, &runner, CancellationToken::new())
            .await
            .unwrap();
        if mode == BisyncMode::InitialPreview {
            assert!(!connection.local_path.join("remote.txt").exists());
            assert!(!plan.work_directory.join("initial-sync-complete").exists());
        }
    }
    assert_eq!(
        fs::read_to_string(connection.local_path.join("remote.txt")).unwrap(),
        "remote-one"
    );
    fs::write(connection.local_path.join("local.txt"), "local-one").unwrap();
    run(
        &connection,
        BisyncMode::Preview,
        &work,
        &runner,
        CancellationToken::new(),
    )
    .await
    .unwrap();
    assert!(!remote.join("share/local.txt").exists());
    run(
        &connection,
        BisyncMode::Sync,
        &work,
        &runner,
        CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(
        fs::read_to_string(remote.join("share/local.txt")).unwrap(),
        "local-one"
    );
    let remote_recovery = remote
        .join(".cosmic-mounter-recovery")
        .join(connection.id.to_string());
    for content in ["local-two-longer", "local-three-even-longer"] {
        // Make modtime changes visible even on second-resolution SFTP servers.
        tokio::time::sleep(Duration::from_millis(1100)).await;
        fs::write(connection.local_path.join("local.txt"), content).unwrap();
        run(
            &connection,
            BisyncMode::Sync,
            &work,
            &runner,
            CancellationToken::new(),
        )
        .await
        .unwrap();
        assert_eq!(
            fs::read_to_string(remote.join("share/local.txt")).unwrap(),
            content
        );
    }
    let backups = recovery_files(&remote_recovery, "local.txt-");
    assert_eq!(backups.len(), 2, "repeated backups must be unique");
    let contents = backups
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>();
    assert!(contents.contains(&"local-one".to_owned()));
    assert!(contents.contains(&"local-two-longer".to_owned()));
    for area in [&plan.recovery_directory, &remote_recovery] {
        for day in ["2000-01-01", "2000-01-02", "2000-02-30"] {
            fs::create_dir_all(area.join(day)).unwrap();
            fs::write(area.join(day).join("preserved.txt"), "old").unwrap();
        }
        fs::write(
            area.join("2000-01-01/.cosmic-mounter-owner"),
            connection.id.to_string(),
        )
        .unwrap();
        fs::write(
            area.join("2000-02-30/.cosmic-mounter-owner"),
            connection.id.to_string(),
        )
        .unwrap();
    }
    fs::remove_file(remote.join("share/remote.txt")).unwrap();
    run(
        &connection,
        BisyncMode::Sync,
        &work,
        &runner,
        CancellationToken::new(),
    )
    .await
    .unwrap();
    assert!(!connection.local_path.join("remote.txt").exists());
    assert_eq!(
        recovery_files(&plan.recovery_directory, "remote.txt-").len(),
        1
    );
    for area in [&plan.recovery_directory, &remote_recovery] {
        assert!(!area.join("2000-01-01").exists());
        assert!(area.join("2000-01-02/preserved.txt").exists());
        assert!(area.join("2000-02-30/preserved.txt").exists());
    }
    drop(server);
}

#[tokio::test]
#[ignore = "requires host rclone; all data and configuration are disposable"]
async fn alias_local_full_coordinator_lifecycle() {
    lifecycle(false).await;
}

#[tokio::test]
#[ignore = "requires host rclone and a loopback SFTP listener; no external credentials"]
async fn loopback_sftp_full_coordinator_lifecycle() {
    lifecycle(true).await;
}
