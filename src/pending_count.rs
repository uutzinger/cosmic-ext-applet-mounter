// SPDX-License-Identifier: MIT

//! Read-only pending-upload observations for an active rclone VFS mount.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;
use tokio_util::sync::CancellationToken;

use crate::fl;
use crate::process::{CommandRequest, CommandRunner, Executable};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OnlinePendingCount {
    pub files: u64,
    pub uploading: u64,
}

pub async fn rclone_online_pending<R: CommandRunner>(
    runner: &R,
    rc_socket: &Path,
) -> Result<OnlinePendingCount, String> {
    let queue = rc_request(rc_socket, "vfs/queue")?;
    let stats = rc_request(rc_socket, "vfs/stats")?;
    let queue = runner
        .run(queue, CancellationToken::new())
        .await
        .map_err(|_| fl!("pending-count-queue-unavailable"))?;
    let stats = runner
        .run(stats, CancellationToken::new())
        .await
        .map_err(|_| fl!("pending-count-cache-unavailable"))?;
    if queue.stdout.truncated
        || queue.stdout.invalid_utf8
        || stats.stdout.truncated
        || stats.stdout.invalid_utf8
    {
        return Err(fl!("pending-count-response-incomplete"));
    }
    parse_rclone_pending(&queue.stdout.text, &stats.stdout.text)
}

fn rc_request(rc_socket: &Path, method: &str) -> Result<CommandRequest, String> {
    CommandRequest::new(Executable::Rclone)
        .arg("rc")
        .and_then(|request| request.arg("--unix-socket"))
        .and_then(|request| request.arg(rc_socket.as_os_str()))
        .and_then(|request| request.arg(method))
        .map(|request| {
            request
                .with_timeout(Duration::from_secs(5))
                .with_output_limit(256 * 1024)
        })
        .map_err(|_| fl!("pending-count-request-failed"))
}

fn parse_rclone_pending(queue: &str, stats: &str) -> Result<OnlinePendingCount, String> {
    let queue: Value =
        serde_json::from_str(queue).map_err(|_| fl!("pending-count-queue-unreadable"))?;
    let stats: Value =
        serde_json::from_str(stats).map_err(|_| fl!("pending-count-cache-unreadable"))?;
    let entries = queue
        .get("queue")
        .and_then(Value::as_array)
        .ok_or_else(|| fl!("pending-count-queue-missing"))?;
    let disk_cache = stats
        .get("diskCache")
        .ok_or_else(|| fl!("pending-count-cache-missing"))?;
    let errors = disk_cache
        .get("erroredFiles")
        .and_then(Value::as_u64)
        .ok_or_else(|| fl!("pending-count-cache-unreadable"))?;
    let out_of_space = disk_cache
        .get("outOfSpace")
        .and_then(Value::as_bool)
        .ok_or_else(|| fl!("pending-count-cache-unreadable"))?;
    if errors > 0 || out_of_space {
        return Err(fl!("pending-count-cache-errors"));
    }
    let uploading = entries
        .iter()
        .filter(|entry| entry.get("uploading").and_then(Value::as_bool) == Some(true))
        .count() as u64;
    Ok(OnlinePendingCount {
        files: entries.len() as u64,
        uploading,
    })
}

/// A successful coordinated Preview can add a bounded, structured size/modtime
/// estimate. It counts differing file paths, not transfers, conflicts or bytes.
pub async fn rclone_preview_snapshot<R: CommandRunner>(
    runner: &R,
    remote: &str,
    local: &Path,
    filters: &Path,
    cancellation: CancellationToken,
) -> Result<u64, String> {
    if cancellation.is_cancelled() {
        return Err(fl!("microsoft-oauth-cancelled"));
    }
    async fn listing<R: CommandRunner>(
        runner: &R,
        target: &str,
        filters: &Path,
        cancellation: CancellationToken,
    ) -> Result<String, String> {
        let request = CommandRequest::new(Executable::Rclone)
            .arg("lsjson")
            .and_then(|request| request.arg(target))
            .and_then(|request| request.arg("--recursive"))
            .and_then(|request| request.arg("--files-only"))
            .and_then(|request| request.arg("--no-mimetype"))
            .and_then(|request| request.arg("--filter-from"))
            .and_then(|request| request.arg(filters.as_os_str()))
            .map_err(|_| fl!("pending-count-request-failed"))?
            .with_timeout(Duration::from_secs(30))
            .with_output_limit(4 * 1024 * 1024);
        let output = runner
            .run(request, cancellation)
            .await
            .map_err(|_| fl!("pending-count-rclone-mirror-unavailable"))?;
        if output.stdout.truncated || output.stdout.invalid_utf8 {
            return Err(fl!("pending-count-response-incomplete"));
        }
        Ok(output.stdout.text)
    }
    let remote = listing(runner, remote, filters, cancellation.child_token()).await?;
    let local = listing(
        runner,
        &local.display().to_string(),
        filters,
        cancellation.child_token(),
    )
    .await?;
    if cancellation.is_cancelled() {
        return Err(fl!("microsoft-oauth-cancelled"));
    }
    preview_inventory_difference(&remote, &local)
}

fn preview_inventory_difference(remote: &str, local: &str) -> Result<u64, String> {
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Item {
        path: String,
        size: u64,
        mod_time: String,
        is_dir: bool,
    }
    fn inventory(text: &str) -> Result<std::collections::BTreeMap<String, (u64, i128)>, String> {
        let items: Vec<Item> =
            serde_json::from_str(text).map_err(|_| fl!("pending-count-response-incomplete"))?;
        let mut map = std::collections::BTreeMap::new();
        for item in items {
            if item.is_dir
                || item.path.is_empty()
                || item.path.starts_with('/')
                || item
                    .path
                    .split('/')
                    .any(|part| matches!(part, "" | "." | ".."))
            {
                return Err(fl!("pending-count-response-incomplete"));
            }
            let timestamp = item
                .mod_time
                .parse::<jiff::Timestamp>()
                .map_err(|_| fl!("pending-count-response-incomplete"))?
                .as_nanosecond();
            if map.insert(item.path, (item.size, timestamp)).is_some() {
                return Err(fl!("pending-count-response-incomplete"));
            }
        }
        Ok(map)
    }
    let remote = inventory(remote)?;
    let local = inventory(local)?;
    let names = remote
        .keys()
        .chain(local.keys())
        .collect::<std::collections::BTreeSet<_>>();
    Ok(names
        .into_iter()
        .filter(|name| match (remote.get(*name), local.get(*name)) {
            (Some((rs, rt)), Some((ls, lt))) => rs != ls || (rt - lt).abs() >= 1_000_000_000,
            _ => true,
        })
        .count() as u64)
}

#[derive(serde::Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PreviewCount {
    pub version: u8,
    pub connection_id: crate::model::ConnectionId,
    pub observed_unix_seconds: u64,
    pub files: Option<u64>,
}

pub fn preview_count_path(work_directory: &Path) -> std::path::PathBuf {
    work_directory.join("preview-pending.json")
}

#[derive(Debug, Clone, Copy)]
pub struct PreviewPendingCount {
    pub files: u64,
    pub age: Duration,
}

/// Read an estimate published by an explicit Preview, never start another dry run.
pub async fn rclone_preview_pending(
    work_directory: &Path,
    id: crate::model::ConnectionId,
) -> Result<PreviewPendingCount, String> {
    use tokio::io::AsyncReadExt;
    let read = async {
        let path = preview_count_path(work_directory);
        let metadata = tokio::fs::symlink_metadata(&path).await.ok()?;
        if !metadata.is_file() || metadata.len() > 4096 {
            return None;
        }
        let file = tokio::fs::File::open(path).await.ok()?;
        let mut bytes = Vec::new();
        file.take(4097).read_to_end(&mut bytes).await.ok()?;
        if bytes.len() > 4096 {
            return None;
        }
        let count: PreviewCount = serde_json::from_slice(&bytes).ok()?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_secs();
        let age = now.checked_sub(count.observed_unix_seconds)?;
        if count.version != 1 || count.connection_id != id || age > 180 {
            return None;
        }
        Some(PreviewPendingCount {
            files: count.files?,
            age: Duration::from_secs(age),
        })
    };
    tokio::time::timeout(Duration::from_secs(2), read)
        .await
        .ok()
        .flatten()
        .ok_or_else(|| fl!("onedrive-preview-count-unavailable"))
}

/// The OneDrive CLI reports directional pending items separately. A non-clean
/// status with missing counters is unknown, never a verified zero.
pub fn parse_onedrive_mirror_status(output: &str) -> Result<u64, String> {
    let status = output
        .lines()
        .find_map(|line| line.trim().strip_prefix("Overall status:"))
        .map(str::trim)
        .ok_or_else(|| fl!("pending-count-mirror-unreadable"))?;
    match status {
        "IN SYNC" => Ok(0),
        "NOT IN SYNC" => {
            let remote = status_count(output, "Pending remote items:")
                .ok_or_else(|| fl!("pending-count-mirror-unreadable"))?;
            let local = status_count(output, "Pending local items:")
                .ok_or_else(|| fl!("pending-count-mirror-unreadable"))?;
            Ok(remote.saturating_add(local))
        }
        _ => Err(fl!("pending-count-mirror-indeterminate")),
    }
}

/// Bisync logs per-side detected changes before applying them. Require its
/// success marker so a partial or unrecognized dry-run cannot claim zero.
pub fn parse_rclone_mirror_status(output: &str) -> Result<u64, String> {
    if !output.contains("Bisync successful") {
        return Err(fl!("pending-count-mirror-unreadable"));
    }
    let mut path1 = None;
    let mut path2 = None;
    for line in output.lines() {
        for (label, slot) in [("Path1:", &mut path1), ("Path2:", &mut path2)] {
            if let Some(rest) = line.split_once(label).map(|(_, rest)| rest)
                && let Some((count, _)) = rest.trim().split_once(" changes:")
            {
                *slot = count.trim().parse::<u64>().ok();
            }
        }
    }
    if let (Some(left), Some(right)) = (path1, path2) {
        return Ok(left.saturating_add(right));
    }
    if path1.is_some() || path2.is_some() {
        return Err(fl!("pending-count-mirror-unreadable"));
    }
    if output.contains("No changes found") {
        return Ok(0);
    }
    let skipped_copies = output
        .lines()
        .filter(|line| line.contains("Skipped copy as --dry-run is set"))
        .count() as u64;
    if skipped_copies > 0 {
        return Ok(skipped_copies);
    }
    if output.matches("There was nothing to transfer").count() >= 2 {
        return Ok(0);
    }
    Err(fl!("pending-count-mirror-unreadable"))
}

#[derive(Deserialize)]
struct LiveProgressState {
    version: u8,
    invocation_id: String,
}

#[derive(Default)]
struct MirrorProgress {
    started: bool,
    succeeded: bool,
    failed: bool,
    path1: Option<u64>,
    path2: Option<u64>,
    completed: u64,
}

impl MirrorProgress {
    fn observe(&mut self, message: &str, object: Option<&str>) {
        if message.contains("Synching Path1") {
            *self = Self {
                started: true,
                ..Self::default()
            };
        }
        if !self.started {
            return;
        }
        if message.contains("Bisync successful") {
            self.succeeded = true;
        }
        if message.contains("Bisync critical error")
            || message.contains("Bisync aborted")
            || message.contains("Bisync failed")
        {
            self.failed = true;
        }
        for (label, slot) in [("Path1:", &mut self.path1), ("Path2:", &mut self.path2)] {
            if let Some(rest) = message.split_once(label).map(|(_, rest)| rest)
                && let Some((count, _)) = rest.trim().split_once(" changes:")
            {
                *slot = count.trim().parse::<u64>().ok();
            }
        }
        if object.is_some() && (message.starts_with("Copied (") || message.starts_with("Deleted")) {
            self.completed = self.completed.saturating_add(1);
        }
    }

    fn pending(self) -> Result<u64, String> {
        if self.succeeded {
            return Ok(0);
        }
        if self.failed {
            return Err(fl!("pending-count-rclone-mirror-unavailable"));
        }
        let total = match (self.path1, self.path2) {
            (Some(left), Some(right)) => left.saturating_add(right),
            _ => return Err(fl!("pending-count-mirror-unreadable")),
        };
        Ok(total
            .saturating_sub(self.completed)
            .max(u64::from(total > 0)))
    }
}

/// Read only app-owned structured rclone output for the current systemd
/// invocation, so an earlier successful run cannot make a new run look clean.
pub async fn rclone_mirror_live_pending(
    work_directory: &Path,
    service: &str,
) -> Result<u64, String> {
    let invocation = crate::services::systemd_user_invocation_id(service)
        .await
        .map_err(|_| fl!("pending-count-rclone-mirror-unavailable"))?;
    if invocation.len() != 32 || !invocation.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(fl!("pending-count-rclone-mirror-unavailable"));
    }
    rclone_mirror_progress_for_invocation(work_directory, &invocation)
}

fn rclone_mirror_progress_for_invocation(
    work_directory: &Path,
    invocation: &str,
) -> Result<u64, String> {
    let state_path = crate::mirror_script::live_progress_state_path(work_directory);
    let state_metadata = std::fs::metadata(&state_path)
        .map_err(|_| fl!("pending-count-rclone-mirror-unavailable"))?;
    if state_metadata.len() > 4096 {
        return Err(fl!("pending-count-response-incomplete"));
    }
    let state: LiveProgressState = serde_json::from_slice(
        &std::fs::read(state_path).map_err(|_| fl!("pending-count-rclone-mirror-unavailable"))?,
    )
    .map_err(|_| fl!("pending-count-mirror-unreadable"))?;
    if state.version != 1 || state.invocation_id != invocation {
        return Err(fl!("pending-count-rclone-mirror-unavailable"));
    }

    let log = File::open(crate::mirror_script::live_progress_log_path(work_directory))
        .map_err(|_| fl!("pending-count-rclone-mirror-unavailable"))?;
    let mut reader = BufReader::new(log);
    let mut line = String::new();
    let mut progress = MirrorProgress::default();
    loop {
        line.clear();
        let bytes = reader
            .read_line(&mut line)
            .map_err(|_| fl!("pending-count-rclone-mirror-unavailable"))?;
        if bytes == 0 {
            break;
        }
        if line.len() > 1024 * 1024 {
            return Err(fl!("pending-count-response-incomplete"));
        }
        let complete = line.ends_with('\n');
        let record: Value = match serde_json::from_str(&line) {
            Ok(record) => record,
            Err(_) if !complete => break,
            Err(_) => return Err(fl!("pending-count-mirror-unreadable")),
        };
        let message = record
            .get("msg")
            .and_then(Value::as_str)
            .ok_or_else(|| fl!("pending-count-mirror-unreadable"))?;
        progress.observe(message, record.get("object").and_then(Value::as_str));
    }
    progress.pending()
}

fn status_count(output: &str, label: &str) -> Option<u64> {
    output.lines().find_map(|line| {
        line.trim()
            .strip_prefix(label)
            .and_then(|value| value.trim().parse().ok())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{CapturedOutput, CommandOutput, FakeCommandRunner};

    fn response(stdout: &str) -> CommandOutput {
        CommandOutput {
            command: "rclone rc".into(),
            stdout: CapturedOutput {
                text: stdout.into(),
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

    #[test]
    fn queue_counts_queued_and_uploading_but_not_clean_cache_files() {
        let count = parse_rclone_pending(
            r#"{"queue":[{"name":"one","uploading":false},{"name":"two","uploading":true}]}"#,
            r#"{"diskCache":{"files":100,"bytesUsed":123456,"erroredFiles":0,"outOfSpace":false}}"#,
        )
        .unwrap();
        assert_eq!(count.files, 2);
        assert_eq!(count.uploading, 1);
        assert!(
            parse_rclone_pending(r#"{"queue":[]}"#, r#"{"diskCache":{"erroredFiles":1}}"#,)
                .is_err()
        );
        assert!(parse_rclone_pending("{}", r#"{"diskCache":{}}"#).is_err());
    }

    #[test]
    fn onedrive_status_requires_a_classified_bidirectional_count() {
        assert_eq!(
            parse_onedrive_mirror_status("Overall status: IN SYNC").unwrap(),
            0
        );
        assert_eq!(
            parse_onedrive_mirror_status(
                "Pending remote items: 2\nPending local items: 3\nOverall status: NOT IN SYNC\n"
            )
            .unwrap(),
            5
        );
        assert!(parse_onedrive_mirror_status("Overall status: NOT IN SYNC").is_err());
        assert!(parse_onedrive_mirror_status("Overall status: INDETERMINATE").is_err());
    }

    #[test]
    fn bisync_status_requires_complete_change_summary() {
        assert_eq!(parse_rclone_mirror_status(
            "INFO: Path1: 2 changes: 1 new, 1 newer\nINFO: Path2: 3 changes: 2 new, 1 deleted\nINFO: Bisync successful"
        ).unwrap(), 5);
        assert_eq!(
            parse_rclone_mirror_status("INFO: No changes found\nINFO: Bisync successful").unwrap(),
            0
        );
        assert!(parse_rclone_mirror_status("INFO: Path1: 0 changes: 0 new").is_err());
        assert!(
            parse_rclone_mirror_status("INFO: Path1: 1 changes: 1 new\nINFO: Bisync successful")
                .is_err()
        );
        assert_eq!(parse_rclone_mirror_status(
            "INFO: There was nothing to transfer\nNOTICE: example.txt: Skipped copy as --dry-run is set (size 16)\nINFO: Bisync successful"
        ).unwrap(), 1);
    }

    #[test]
    fn live_mirror_uses_only_structured_state_for_current_invocation() {
        let temp = tempfile::tempdir().unwrap();
        let invocation = "0123456789abcdef0123456789abcdef";
        std::fs::write(
            crate::mirror_script::live_progress_state_path(temp.path()),
            format!(r#"{{"version":1,"invocation_id":"{invocation}"}}"#),
        )
        .unwrap();
        std::fs::write(
            crate::mirror_script::live_progress_log_path(temp.path()),
            concat!(
                r#"{"level":"info","msg":"Synching Path1 remote: with Path2 local"}"#,
                "\n",
                r#"{"level":"info","msg":"Path1: 2 changes: 2 new"}"#,
                "\n",
                r#"{"level":"info","msg":"Path2: 1 changes: 1 new"}"#,
                "\n",
                r#"{"level":"info","msg":"Copied (new)","object":"first.txt"}"#,
                "\n"
            ),
        )
        .unwrap();
        assert_eq!(
            rclone_mirror_progress_for_invocation(temp.path(), invocation).unwrap(),
            2
        );
        use std::io::Write;
        let mut log = std::fs::OpenOptions::new()
            .append(true)
            .open(crate::mirror_script::live_progress_log_path(temp.path()))
            .unwrap();
        write!(log, r#"{{"level":"info","msg":"incomplete""#).unwrap();
        assert_eq!(
            rclone_mirror_progress_for_invocation(temp.path(), invocation).unwrap(),
            2
        );
        std::fs::write(
            crate::mirror_script::live_progress_log_path(temp.path()),
            concat!(
                r#"{"level":"info","msg":"Synching Path1 remote: with Path2 local"}"#,
                "\n",
                r#"{"level":"info","msg":"Path1: 2 changes: 2 new"}"#,
                "\n",
                r#"{"level":"info","msg":"Path2: 1 changes: 1 new"}"#,
                "\n",
                r#"{"level":"info","msg":"Bisync successful"}"#,
                "\n"
            ),
        )
        .unwrap();
        assert_eq!(
            rclone_mirror_progress_for_invocation(temp.path(), invocation).unwrap(),
            0
        );
        std::fs::write(
            crate::mirror_script::live_progress_log_path(temp.path()),
            concat!(
                r#"{"level":"info","msg":"Synching Path1 remote: with Path2 local"}"#,
                "\n",
                r#"{"level":"info","msg":"Path1: 2 changes: 2 new"}"#,
                "\n",
                r#"{"level":"info","msg":"Path2: 1 changes: 1 new"}"#,
                "\n",
                r#"{"level":"error","msg":"Bisync aborted"}"#,
                "\n"
            ),
        )
        .unwrap();
        assert!(rclone_mirror_progress_for_invocation(temp.path(), invocation).is_err());
        assert!(
            rclone_mirror_progress_for_invocation(temp.path(), "ffffffffffffffffffffffffffffffff")
                .is_err()
        );
    }

    #[tokio::test]
    async fn online_count_reads_only_the_private_queue_and_health() {
        let runner = FakeCommandRunner::default();
        runner.push(Ok(response(
            r#"{"queue":[{"name":"one","uploading":true}]}"#,
        )));
        runner.push(Ok(response(
            r#"{"diskCache":{"erroredFiles":0,"outOfSpace":false}}"#,
        )));
        let count = rclone_online_pending(&runner, Path::new("/run/user/1000/private.sock"))
            .await
            .unwrap();
        assert_eq!(count.files, 1);
        let requests = runner.requests();
        assert_eq!(requests.len(), 2);
        assert!(requests[0].sanitized_command().contains("vfs/queue"));
        assert!(requests[1].sanitized_command().contains("vfs/stats"));
        assert!(
            requests
                .iter()
                .all(|request| request.sanitized_command().contains("--unix-socket"))
        );
    }
    fn inventory_item(path: &str, size: u64, time: &str) -> Value {
        serde_json::json!({"Path":path,"Size":size,"ModTime":time,"IsDir":false})
    }

    #[test]
    fn onedrive_runtime_preview_inventory_counts_differences_without_logs() {
        let remote = serde_json::json!([
            inventory_item("same", 1, "2026-10-15T10:00:00Z"),
            inventory_item("remote", 1, "2026-10-15T10:00:00Z"),
            inventory_item("size", 2, "2026-10-15T10:00:00Z"),
            inventory_item("time", 1, "2026-10-15T10:00:00Z"),
            inventory_item("zone", 1, "2026-10-15T10:00:00Z")
        ]);
        let local = serde_json::json!([
            inventory_item("same", 1, "2026-10-15T10:00:00.999999999Z"),
            inventory_item("local", 1, "2026-10-15T10:00:00Z"),
            inventory_item("size", 1, "2026-10-15T10:00:00Z"),
            inventory_item("time", 1, "2026-10-15T10:00:01Z"),
            inventory_item("zone", 1, "2026-10-15T03:00:00-07:00")
        ]);
        assert_eq!(
            preview_inventory_difference(&remote.to_string(), &local.to_string()).unwrap(),
            4
        );
        assert_eq!(preview_inventory_difference("[]", "[]").unwrap(), 0);
        let duplicate = serde_json::json!([
            inventory_item("same", 1, "2026-10-15T10:00:00Z"),
            inventory_item("same", 1, "2026-10-15T10:00:00Z")
        ]);
        for invalid in [
            "broken".into(),
            "[{}]".into(),
            duplicate.to_string(),
            serde_json::json!([inventory_item("../escape", 1, "2026-10-15T10:00:00Z")]).to_string(),
            serde_json::json!([inventory_item("bad-date", 1, "not a date")]).to_string(),
        ] {
            assert!(preview_inventory_difference(&invalid, "[]").is_err());
        }
    }

    #[tokio::test]
    async fn onedrive_runtime_preview_report_rejects_stale_foreign_partial_and_linked_data() {
        let root = tempfile::tempdir().unwrap();
        let id = crate::model::ConnectionId::new();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let path = preview_count_path(root.path());
        for (version, owner, time, files, ok) in [
            (1, id, now, Some(4), true),
            (1, id, now - 181, Some(0), false),
            (1, crate::model::ConnectionId::new(), now, Some(0), false),
            (2, id, now, Some(0), false),
            (1, id, now, None, false),
            (1, id, now + 60, Some(0), false),
        ] {
            let report = PreviewCount {
                version,
                connection_id: owner,
                observed_unix_seconds: time,
                files,
            };
            std::fs::write(&path, serde_json::to_vec(&report).unwrap()).unwrap();
            assert_eq!(rclone_preview_pending(root.path(), id).await.is_ok(), ok);
        }
        std::fs::write(&path, "{broken").unwrap();
        assert!(rclone_preview_pending(root.path(), id).await.is_err());
        std::fs::write(&path, "x".repeat(4097)).unwrap();
        assert!(rclone_preview_pending(root.path(), id).await.is_err());
        std::fs::remove_file(&path).unwrap();
        std::os::unix::fs::symlink(root.path(), &path).unwrap();
        assert!(rclone_preview_pending(root.path(), id).await.is_err());
    }

    #[tokio::test]
    async fn onedrive_runtime_preview_snapshot_uses_bounded_read_only_filtered_requests() {
        let runner = crate::process::FakeCommandRunner::default();
        for _ in 0..2 {
            runner.push(Ok(response("[]")));
        }
        assert_eq!(
            rclone_preview_snapshot(
                &runner,
                "synthetic:Test",
                Path::new("/tmp/synthetic-local"),
                Path::new("/tmp/synthetic-filters"),
                CancellationToken::new()
            )
            .await
            .unwrap(),
            0
        );
        assert_eq!(runner.requests().len(), 2);
        for request in runner.requests() {
            assert!(request.sanitized_command().starts_with("rclone lsjson "));
            assert!(request.sanitized_command().contains("--filter-from"));
            assert_eq!(request.timeout, Duration::from_secs(30));
        }
        let runner = crate::process::FakeCommandRunner::default();
        let mut incomplete = response("[]");
        incomplete.stdout.truncated = true;
        runner.push(Ok(incomplete));
        assert!(
            rclone_preview_snapshot(
                &runner,
                "synthetic:Test",
                Path::new("/tmp/synthetic-local"),
                Path::new("/tmp/synthetic-filters"),
                CancellationToken::new()
            )
            .await
            .is_err()
        );
        assert_eq!(runner.requests().len(), 1);
    }
    #[tokio::test]
    #[ignore = "requires installed rclone; temporary local directories only, no user config or network"]
    async fn onedrive_runtime_local_preview_metadata_uses_real_rclone() {
        let root = tempfile::tempdir().unwrap();
        let remote = root.path().join("synthetic-remote");
        let local = root.path().join("synthetic-local");
        std::fs::create_dir_all(&remote).unwrap();
        std::fs::create_dir_all(&local).unwrap();
        for directory in [&remote, &local] {
            std::fs::write(directory.join("same"), "same").unwrap();
        }
        std::fs::write(remote.join("remote-only"), "remote").unwrap();
        std::fs::write(local.join("local-only"), "local").unwrap();
        std::fs::write(remote.join("hidden.one"), "notebook").unwrap();
        let filters = root.path().join("filters");
        std::fs::write(
            &filters,
            crate::sync::rclone_bisync_filter_file(crate::model::Provider::OneDrive),
        )
        .unwrap();
        assert_eq!(
            rclone_preview_snapshot(
                &crate::process::SystemCommandRunner,
                &remote.display().to_string(),
                &local,
                &filters,
                CancellationToken::new()
            )
            .await
            .unwrap(),
            2
        );
        assert!(!remote.join("local-only").exists());
        assert!(!local.join("remote-only").exists());
    }
}
