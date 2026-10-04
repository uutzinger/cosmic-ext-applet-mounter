// SPDX-License-Identifier: MIT

//! Render the host-side bisync runner used by manual and scheduled mirrors.

use std::path::PathBuf;

use crate::process::CommandError;
use crate::services::ServiceSpec;
use crate::sync::RcloneBisyncPlan;

const OWNER_MARKER: &str = ".cosmic-mounter-owner";

#[must_use]
pub fn script_path(plan: &RcloneBisyncPlan) -> PathBuf {
    plan.work_directory.join("managed-bisync.sh")
}

#[must_use]
pub fn service_spec(plan: &RcloneBisyncPlan) -> ServiceSpec {
    let mut service = plan.service.clone();
    service.executable = PathBuf::from("/usr/bin/sh");
    service.arguments = vec![script_path(plan).display().to_string(), "scheduled".into()];
    service
}

fn shell_quote(value: &str) -> Result<String, CommandError> {
    if value.contains(['\0', '\n', '\r']) {
        return Err(CommandError::InvalidArgument);
    }
    Ok(format!("'{}'", value.replace('\'', "'\\''")))
}

pub fn render(plan: &RcloneBisyncPlan) -> Result<String, CommandError> {
    let owner = plan.connection_id.to_string();
    let executable = shell_quote(&plan.service.executable.display().to_string())?;
    let remote_root = shell_quote(&plan.remote_recovery_path)?;
    let local_root = shell_quote(&plan.recovery_directory.display().to_string())?;
    let owner = shell_quote(&owner)?;
    let preflight = if let Some(command) = &plan.preflight_command {
        let mut parts = vec![shell_quote(&command.executable.display().to_string())?];
        parts.extend(
            command
                .arguments
                .iter()
                .map(|argument| shell_quote(argument))
                .collect::<Result<Vec<_>, _>>()?,
        );
        format!("{}\n", parts.join(" "))
    } else {
        String::new()
    };
    let mut arguments = Vec::new();
    let mut index = 0;
    while index < plan.service.arguments.len() {
        let argument = &plan.service.arguments[index];
        if argument == "--backup-dir1" || argument == "--backup-dir2" {
            let variable = if argument == "--backup-dir1" {
                "\"$remote_backup\""
            } else {
                "\"$local_backup\""
            };
            if index + 1 >= plan.service.arguments.len() {
                return Err(CommandError::InvalidArgument);
            }
            arguments.push(shell_quote(argument)?);
            arguments.push(variable.to_owned());
            index += 2;
        } else {
            arguments.push(shell_quote(argument)?);
            index += 1;
        }
    }
    if !plan
        .service
        .arguments
        .iter()
        .any(|arg| arg == "--backup-dir1")
        || !plan
            .service
            .arguments
            .iter()
            .any(|arg| arg == "--backup-dir2")
    {
        return Err(CommandError::InvalidArgument);
    }
    let arguments = arguments.join(" ");
    let access_check = if let Some(name) = &plan.access_marker_name {
        let local = shell_quote(&plan.path2_local.join(name).display().to_string())?;
        let remote = shell_quote(&format!(
            "{}/{}",
            plan.path1_remote.trim_end_matches('/'),
            name
        ))?;
        format!(
            "local_access={local}\n\
remote_access={remote}\n\
[ -f \"$local_access\" ] && [ ! -L \"$local_access\" ] || {{ echo 'SharePoint mirror access marker is missing or unsafe' >&2; exit 1; }}\n\
[ \"$(cat \"$local_access\")\" = \"$owner\" ] || {{ echo 'SharePoint mirror access marker changed' >&2; exit 1; }}\n\
case \"$mode\" in\n\
  initial-preview|initial-sync) ;;\n\
  *) remote_access_owner=$(\"$rclone\" cat \"$remote_access\") || {{ echo 'SharePoint remote access marker is missing' >&2; exit 1; }}\n\
     [ \"$remote_access_owner\" = \"$owner\" ] || {{ echo 'SharePoint remote access marker changed' >&2; exit 1; }} ;;\n\
esac\n"
        )
    } else {
        String::new()
    };
    let verification = if plan.verify_after_sync {
        let remote = shell_quote(&plan.path1_remote)?;
        let local = shell_quote(&plan.path2_local.display().to_string())?;
        let filters = shell_quote(&plan.filters_file.display().to_string())?;
        format!(
            "# SharePoint may rewrite Office bytes after the first upload.\n\
case \"$mode\" in\n\
  initial-sync) set -- {arguments} '--suffix' \"-$stamp\" ;;\n\
esac\n\
\"$rclone\" \"$@\"\n\
\"$rclone\" check {remote} {local} '--checksum' '--filter-from' {filters}\n"
        )
    } else {
        String::new()
    };

    Ok(format!(
        "#!/bin/sh\n# Cloud Mounter managed bisync script: {owner}\n\
set -eu\n\
{preflight}\
rclone={executable}\n\
remote_root={remote_root}\n\
local_root={local_root}\n\
owner={owner}\n\
mode=${{1:-scheduled}}\n\
day=$(date -u +%F)\n\
stamp=$(date -u +%Y%m%dT%H%M%S.%N)\n\
remote_backup=$remote_root/$day\n\
local_backup=$local_root/$day\n\
set -- {} '--suffix' \"-$stamp\"\n\
case \"$mode\" in\n\
  initial-preview) set -- \"$@\" '--resync' '--dry-run' ;;\n\
  preview) set -- \"$@\" '--dry-run' ;;\n\
  initial-sync) set -- \"$@\" '--resync' ;;\n\
  sync|scheduled) ;;\n\
  *) echo 'invalid managed bisync mode' >&2; exit 2 ;;\n\
esac\n\
{access_check}\
case \"$mode\" in\n\
  *preview) exec \"$rclone\" \"$@\" ;;\n\
esac\n\
[ ! -L \"$local_backup\" ] || {{ echo 'recovery path is a symlink' >&2; exit 1; }}\n\
mkdir -p -- \"$local_backup\"\n\
if [ -e \"$local_backup/{OWNER_MARKER}\" ]; then\n\
  [ \"$(cat \"$local_backup/{OWNER_MARKER}\")\" = \"$owner\" ] || exit 1\n\
elif [ -n \"$(find \"$local_backup\" -mindepth 1 -print -quit)\" ]; then\n\
  echo 'recovery directory is not applet-owned' >&2; exit 1\n\
fi\n\
printf '%s\\n' \"$owner\" > \"$local_backup/{OWNER_MARKER}\"\n\
remote_marker=$remote_backup/{OWNER_MARKER}\n\
\"$rclone\" mkdir \"$remote_backup\"\n\
remote_owner=$(\"$rclone\" cat \"$remote_marker\" 2>/dev/null || true)\n\
if [ -n \"$remote_owner\" ]; then\n\
  [ \"$remote_owner\" = \"$owner\" ] || exit 1\n\
elif [ -n \"$(\"$rclone\" lsf \"$remote_backup\")\" ]; then\n\
  echo 'remote recovery directory is not applet-owned' >&2; exit 1\n\
fi\n\
printf '%s\\n' \"$owner\" | \"$rclone\" rcat \"$remote_marker\"\n\
\"$rclone\" \"$@\"\n\
{verification}\
cutoff=$(date -u -d '32 days ago' +%F)\n\
for candidate in \"$local_root\"/[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]; do\n\
  [ -d \"$candidate\" ] || continue\n\
  [ ! -L \"$candidate\" ] || continue\n\
  name=${{candidate##*/}}\n\
  [ \"$name\" \\< \"$cutoff\" ] || continue\n\
  [ \"$(cat \"$candidate/{OWNER_MARKER}\" 2>/dev/null || true)\" = \"$owner\" ] || continue\n\
  rm -r -- \"$candidate\" || echo 'local recovery cleanup deferred' >&2\n\
done\n\
\"$rclone\" lsf --dirs-only --max-depth 1 \"$remote_root\" | while IFS= read -r entry; do\n\
  name=${{entry%/}}\n\
  case \"$name\" in [0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]) ;; *) continue ;; esac\n\
  [ \"$name\" \\< \"$cutoff\" ] || continue\n\
  target=$remote_root/$name\n\
  [ \"$(\"$rclone\" cat \"$target/{OWNER_MARKER}\" 2>/dev/null || true)\" = \"$owner\" ] || continue\n\
  \"$rclone\" purge \"$target\" || echo 'remote recovery cleanup deferred' >&2\n\
done\n",
        arguments
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        Connection, ConnectionId, ConnectionMode, OfflineMirrorConfig, Provider, TuningProfile,
    };
    use crate::sync::rclone_bisync_plan;
    use std::path::Path;
    use std::process::Command;
    use uuid::Uuid;

    #[test]
    fn shell_quote_rejects_line_breaks_and_escapes_apostrophes() {
        assert_eq!(shell_quote("a'b").unwrap(), "'a'\\''b'");
        assert!(shell_quote("a\nb").is_err());
    }

    #[test]
    #[ignore = "requires host rclone and exercises disposable local mirror data"]
    fn live_runner_preserves_backups_and_prunes_only_marked_old_directories() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let remote = root.join("remote");
        let local = root.join("local");
        let recovery = root.join("recovery");
        let work = root.join("work");
        std::fs::create_dir_all(remote.join("share")).unwrap();
        std::fs::create_dir_all(&local).unwrap();
        std::fs::create_dir_all(&recovery).unwrap();
        std::fs::write(remote.join("share/delete-me.txt"), "keep a recovery copy\n").unwrap();
        std::fs::write(local.join("local-only.txt"), "from local\n").unwrap();
        let config = root.join("rclone.conf");
        std::fs::write(
            &config,
            format!(
                "[mirror_probe]\ntype = alias\nremote = {}\n",
                remote.display()
            ),
        )
        .unwrap();
        let connection = Connection {
            id: ConnectionId::from_uuid(Uuid::new_v4()),
            name: "Disposable mirror".into(),
            provider: Provider::Sftp,
            mode: ConnectionMode::OfflineMirror(OfflineMirrorConfig {
                recovery_directory: recovery.clone(),
                sync_interval_minutes: 15,
                sync_on_metered: false,
            }),
            remote_reference: "mirror_probe".into(),
            remote_subpath: Some("share".into()),
            local_path: local.clone(),
            enabled: true,
            vpn_profile_id: None,
            disconnect_vpn_when_unused: false,
            tuning_profile: TuningProfile::Balanced,
            smb_preload_override: None,
            sftp_preload_override: None,
            teams_identity: None,
        };
        let plan = rclone_bisync_plan(&connection, Path::new(&work)).unwrap();
        std::fs::create_dir_all(&plan.work_directory).unwrap();
        std::fs::write(&plan.filters_file, "+ **\n").unwrap();
        let script = script_path(&plan);
        std::fs::write(&script, render(&plan).unwrap()).unwrap();
        let run = |mode: &str| {
            let output = Command::new("/usr/bin/sh")
                .arg(&script)
                .arg(mode)
                .env("RCLONE_CONFIG", &config)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{mode}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        };
        run("initial-preview");
        assert!(!local.join("delete-me.txt").exists());
        run("initial-sync");
        assert!(local.join("delete-me.txt").exists());
        assert!(remote.join("share/local-only.txt").exists());
        std::fs::remove_file(remote.join("share/delete-me.txt")).unwrap();
        run("sync");
        assert!(!local.join("delete-me.txt").exists());
        let dated = std::fs::read_dir(&recovery)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| path.is_dir())
            .unwrap();
        assert!(std::fs::read_dir(&dated).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("delete-me")
        }));
        let old_local = recovery.join("2000-01-01");
        let unowned = recovery.join("2000-01-02");
        std::fs::create_dir_all(&old_local).unwrap();
        std::fs::create_dir_all(&unowned).unwrap();
        std::fs::write(old_local.join(OWNER_MARKER), connection.id.to_string()).unwrap();
        std::fs::write(old_local.join("old.txt"), "old").unwrap();
        std::fs::write(unowned.join("user.txt"), "untouched").unwrap();
        let old_remote = remote
            .join(".cosmic-mounter-recovery")
            .join(connection.id.to_string())
            .join("2000-01-01");
        std::fs::create_dir_all(&old_remote).unwrap();
        std::fs::write(old_remote.join(OWNER_MARKER), connection.id.to_string()).unwrap();
        std::fs::write(old_remote.join("old.txt"), "old").unwrap();
        run("sync");
        assert!(!old_local.exists());
        assert!(!old_remote.exists());
        assert!(unowned.join("user.txt").exists());
        assert!(dated.exists());
    }
}
