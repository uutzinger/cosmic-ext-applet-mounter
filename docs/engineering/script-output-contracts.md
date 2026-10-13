# Script output and resumption contracts

Type: Contracts

Status: Accepted interface, implemented and fixture-verified; new hosted acceptance pending

Created: October 13, 2026

Last reviewed: October 13, 2026

Scope: Hosted Debian build/retrieval, release preparation and artifact promotion.
Associated task: script-tooling P1. No installation, package matrix or publication
is authorized merely by implementing this interface.

## Common receipt

Each started task creates a private, unique directory under `target/task-runs/`
(or a new explicit `--log-dir`). It contains `task.log` and atomically updated
`summary.json`. Never overwrite a previous task directory. Files are private;
logs may contain provider diagnostics and are not copied to chat automatically.
Invalid CLI arguments can fail before a task is started.
Hosted promotion uploads only summary/checksum receipts (14-day retention), not
private raw logs. Raw runner logs have the runner's lifetime; do not claim those
private files remain downloadable after teardown. No tokens or raw command
arguments belong in public receipts.

Schema version 1 fields: `task`, `status` (running/succeeded/failed/cancelled),
`phase`, `source_sha`, `ref`, `repository`, `run_id`, `run_url`, `request_id`,
`exit_code`, `log_path`, `summary_path`, `artifact_receipt_path`, `next_action`,
`updated_at`. Unknown/not-applicable values are null. Failure/cancellation retain
identities already discovered; incomplete operations never acquire success state.
Artifact receipts contain source/run identity and paths/SHA-256 of verified files.
Promotion receipts instead record names, hashes and server run origins when
downloaded temporary files are deleted; they never advertise deleted temp paths.

Console output is phase/status changes plus one final compact JSON result.
Successful compile/CLI output stays in the log. Failures retain real process exit
codes and point to the private log; inspect a bounded, redacted excerpt only
when needed rather than automatically printing potentially sensitive stderr.
Signals terminate/reap local child groups and record 130/143. Remote CI is not
implicitly cancelled; record the run/request identity so an operator can resume.

## Hosted build selection and monitoring

New dispatches carry a unique request ID exposed in the workflow run title.
Select only a matching trusted repository/workflow/ref/source run. Never select
an unrelated newest run. More than one matching run fails closed. After an
ambiguous/failed dispatch, do not automatically dispatch again. Resume using
the report's run ID or request ID. Legacy wait-for-existing requires an
unambiguous exact-source run and cannot trigger a build.

Poll structured status at a bounded interval (default 30 seconds), with an
overall timeout. Print status changes, not continuously refreshing job tables.
Download only after successful completion. Verify package identities/checksums,
write an artifact receipt and publish the staged download directory atomically.
Existing directories are reused only when their receipt/run/source and files
verify; never silently overwrite or mix different builds.

## Release preparation and promotion

Tag creation remains separately approved. Default preparation refuses existing
tags. Explicit `--resume` requires the original source and a selected run; existing
matching tags are reused without recreation/push. Network failures are not treated
as proof that a remote tag is absent. No resume automatically rebuilds or moves a
tag. Reuse validation precedes new tag mutation.

Promotion remains inspection-only by default. Publication requires explicit
`--publish --accepted`. Persistent receipts identify verified package bytes, not
deleted temporary paths. No compilation/build dispatch occurs in promotion.
Existing release assets must not be replaced automatically; ambiguous publication
failure requires checking remote state before another attempt.

The common implementation is Python standard-library code in
`scripts/lib/task_output.py`. Stable Bash entry points delegate fixed task names
to `scripts/lib/release_tasks.py`; this supersedes the initial proposed Bash-only
reporting helper and avoids separate shell JSON/state implementations.

## Verification boundary

Use fake CLI fixtures for dispatch correlation, stale/ambiguous runs, exit codes,
noisy logs, cancellation, download validation, cache reuse and partial-tag resume.
Read-only live metadata checks are optional and distinct. A real hosted build or
publication is not required to test this control flow. Local/native and Flatpak
wrappers remain P2/later tasks; installers/runtime auth are outside P1.
