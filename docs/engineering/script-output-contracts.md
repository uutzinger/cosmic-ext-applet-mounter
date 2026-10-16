# Script output and resumption contracts

Type: Contracts

Status: Accepted interface, implemented and fixture-verified; new hosted acceptance pending

Created: October 13, 2026

Last reviewed: October 16, 2026

Scope: Hosted/local tooling, setup reporting, Debian/native/Flatpak inspection
and OneDrive guard-investigation receipts.
Associated task: script-tooling P1–P3. No installation, package matrix or publication
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

### P3.3 verified candidate reuse contract

`build-flatpak.sh` accepts explicit `--reuse-receipt PATH/artifacts.json`, with
the same manifest/source/architecture/branch/output arguments. Default invocation
still refuses any existing output. Reuse requires a schema-2 candidate receipt
and its sibling successful flatpak-build summary; legacy/incomplete/failed
receipts are not adopted. Verify checkout/input fingerprints, selected manifest,
identity, recorded runtime/tool provenance and build options, canonical output
locations, exactly the expected bundle/metadata files and their hashes. Require
repository and bundle ref/commit/full metadata to match the recorded results.
Run read-only `ostree fsck --quiet` without deletion/repair options before reuse;
this adds object-integrity checking to P3.2 metadata comparison.

Reuse does not invoke flatpak-builder (even for --version), build-export or
build-bundle, and does not require current installed build runtimes. Their original
recorded commits and tool versions remain provenance; current inspector versions
are recorded separately, not substituted as build evidence. Input/options mismatch,
corruption, missing output, prior failure or cancellation fails with no rebuild,
overwrite, cleanup or retry. A partial candidate requires diagnosis; a newly
authorized build uses a different destination.

The new private report links the prior receipt path/hash and records verification
results without changing that receipt or original outputs (repository checking
may use tool locks). Recheck source/artifact/receipt stability before success.
Receipts are trusted local comparison baselines, not signed attestations: editing
both artifacts and their entire baseline cannot be detected cryptographically.
No installed acceptance, runtime availability, signatures or publication claim.
Summary schema remains version 1; reusable Flatpak candidate artifacts.json uses
version 2. Other inspector/promotion receipt schemas are unchanged.

### P3.2 offline Flatpak inspection contract

The shared inspector adds `flatpak-repo` and `flatpak-bundle`. Both require
explicit --app-id, --flatpak-arch, --branch and --expected-metadata, with optional
--expected-commit. The selected expected keyfile is the caller's comparison
baseline, not automatically an approved permission policy. Compare all sections
and keys (including Context, bus policies, Environment and extension settings),
not just requested finish-args or a permission subset. Require expected and
actual application/runtime/SDK identities and architecture to agree. Reject
duplicate/default/malformed metadata, unsafe identities and symbolic inputs.

Read the selected local OSTree app ref and its /metadata without executing app
code. Bundle inspection imports into a private temporary repository only, never
an installed application, configured remote or user/system installation. Fixed
local flatpak/ostree commands do not fetch network data or install runtimes.
Record selected ref/commit, canonical metadata hash, expected-file hash and
bundle hash when applicable. Confirm ref/input stability across inspection.
The canonical metadata fingerprint is distinct from the raw keyfile byte hash.

Candidate build/export reuses the same functions: exported repository and bundled
ref must match each other and the built metadata, before a success receipt is
written. This establishes export consistency, not independent permission approval.
Report full metadata only in private receipts, not console output (Environment
can contain sensitive values). Installed overrides/dynamic permissions, object
payload integrity, signatures, provenance and store acceptance remain outside
metadata inspection. P3.3 reuse must not mistake this for full repository fsck.

### P3.1 explicit-manifest build/export contract

`bash scripts/build-flatpak.sh --manifest PATH --source-sha SHA --arch ARCH
--branch BRANCH --output-dir NEW_DIRECTORY` builds an unaccepted candidate.
The manifest must be JSON inside a clean Git checkout at the expected SHA;
referenced JSON/source/patch inputs must be tracked inside that checkout.
Git sources require full commit pins, remote file/archive inputs require SHA-256,
and local regular-file inputs are hashed. Generated Cargo inline sources require
string contents and destination filenames; shell/script command lists require
strings. Their enclosing tracked JSON is fingerprinted, including all contents
and commands. Mutable tags alone, directory sources,
unsupported input forms and symbolic paths fail closed. This intentionally does
not accept existing prototype/loosely pinned manifests automatically.

The wrapper records packaging-checkout identity separately from manifest engine/
application source pins. It invokes fixed flatpak-builder build, flatpak
build-export and build-bundle operations, with no installation, launch, runtime
substitution, dependency generation, cleanup, signing or submission. Required
runtime/SDK/BaseApp and declared SDK extensions are checked read-only and their
installed commits recorded. --offline disables source
downloads; otherwise pinned-source downloads are permitted, not runtime installs.
Build state lives under the new candidate directory, so offline operation needs
sources already available to the builder; no cache population is implied.

The requested branch must match any manifest branch; architecture is explicit.
Tool versions, input hashes and declared finish-args are recorded. Built metadata
is retained separately (never equated with declared permissions); application,
runtime and SDK identities must match. The source input fingerprint is rechecked
before success. Build/export/bundle failures retain private logs/partial output,
never a success receipt or implicit retry. Output parents must exist, destinations
must be new and outside the tracked source inputs, and a log directory cannot be
inside the candidate output. --timeout-seconds applies per command (1–86400,
default 7200); local child cleanup uses TaskReport.

A final artifacts.json records the exported bundle hash, built metadata hash,
input/runtime/tool identity and local repository location. It does not certify
repository integrity, reproducibility, dependency closure, installed behavior or
store acceptance. Shared inspection checks export consistency; existing outputs
are refused unless P3.3 --reuse-receipt is explicitly selected and verified.

### P2 standalone setup reporting contract

Both setup scripts accept optional `--log-dir NEW_DIRECTORY` (parent must exist).
Default behavior and dependency/service selections stay unchanged. Logging mirrors
stdout/stderr to private task.log **and keeps existing console output visible**:
unlike noninteractive dev tasks, installers must not hide sudo/package prompts or
instructions. Input and shell tracing are never recorded by the reporter; raw
command diagnostics may still be sensitive and logs must not be published.
No checkout-relative modules, Python, network bootstrap or automatic retries are
introduced. The same embedded Bash block is parity-tested in both standalone files.
Help does not create a report or perform mutations.

Opt-in reporting creates a new private directory, preserves exit status and writes
atomic summary.json on exit: schema_version, task, status, exit_code, log_path,
summary_path and next_action. Initial status is running with a null exit_code;
SIGINT/SIGTERM exit 130/143 and mark cancelled. Reporting does not promise child
group cleanup, rollback or transactional package/service operations. Partial
changes can remain after failure and require inspection, never implicit retry.
No source/run provenance is inferred for a downloaded standalone script. Reject
existing/symbolic log destinations and control characters in report paths.

### P2 development task contract

`bash scripts/dev.sh <task>` runs one allowlisted just recipe from the repository
root: fmt-check, localization-check, check, lint, test, verify, metadata-check,
metadata-check-strict, build or build-release. No arbitrary arguments, overrides,
installation, cleanup, launch or publication tasks are accepted. The justfile
remains authoritative, including its documented nonfatal metadata warnings.
Build tasks build when explicitly selected; they do not promise cached acceptance.
The shared reporter retains stdout/stderr privately and prints phase/final status
only. `--log-dir` requires a new directory; `--timeout-seconds` is bounded to
1–86400 (default 7200). Cancellation terminates the local recipe process group.
Summary fields additionally record `recipe` and `source_dirty`; source_sha is
the checkout HEAD, not proof that dirty inputs equal that revision. No build
provenance or installed acceptance is implied. Underlying just exit codes are
preserved; timeout is a failed task, not a successful check.

### P2 offline artifact inspection contract

The standalone inspector accepts an explicit `debian-matrix`, `debian` or
`native` mode and expected version/architecture where applicable. It performs
no network requests, builds, installation or executable launch. Debian metadata
is read with `dpkg-deb`, never package maintainer scripts. Matrix inspection
shares the promotion verifier and requires its six expected checksummed targets.
Single-package inspection validates package/version/architecture; native inspection
records bytes only, not ABI, runtime or UI acceptance. Reject symbolic/nonregular
inputs and unsafe version/path components. Stream hashes rather than loading
large packages into memory. Input files are not modified; private task receipts
are the only output writes. Receipts describe observed bytes, not trusted build
provenance; source/run identity stays unknown unless verified by the calling
build/promotion workflow. Artifact payload/dependency and installed acceptance
remain separate checks. P2 wrappers and deployment reporting have their own
contracts above and do not expand byte inspection into installed acceptance.

Use fake CLI fixtures for dispatch correlation, stale/ambiguous runs, exit codes,
noisy logs, cancellation, download validation, cache reuse and partial-tag resume.
Read-only live metadata checks are optional and distinct. A real hosted build or
publication is not required to test this control flow. P2/P3 wrapper and setup
reporting fixtures add their stated coverage; runtime authentication remains
outside this script-tooling scope.


## OneDrive guard investigation schema 2 (October 15, 2026)

The existing guard wrapper opts into the extended experiment with `--round-trip`;
its shared `summary.json` remains schema 1. New `results.json` uses schema 2 and is
checkpointed before/after operations, including exceptions/cancellation. Legacy
runs remain schema 1 and must not be interpreted as original-byte evidence.

Fields include status/verdict/issues, drive label/remote, source SHA and dirty
flag (unknown outside Git), SHA-256 of the two implementation modules, opaque
rclone version text (`rclone_version = {format: "text", output: ...}`), observation
interval and timestamps, baseline/source manifests,
round-trip original/remote/download evidence, profile flags and stages, individual
command exit codes and counters, and per-folder ownership/cleanup outcomes.
Source SHA on a dirty checkout is not proof of exact source bytes; module hashes
identify the implementation. Drive label still comes from the operator; no new
account-identity verification is implied. Tokens here are random cleanup markers,
not authentication credentials. Sample names remain in private local evidence.

Transfers use rclone's structured JSON `stats` fields (bytes/transfers/errors),
never localized status prose; diagnostic copies are explicitly separate from
bisync counters. Local SHA-256 and byte lengths establish actual downloaded
content independently of remote QuickXorHash and size claims. Missing statistics,
missing hash evidence, byte rewrites, failed operations and unobserved requested
web edits cannot pass. A compatibility bundle passing does not identify the
smallest necessary settings; its verdict remains inconclusive pending targeted
follow-up. No whole-drive operation, mounted UI or installed-service acceptance.

Each profile uses fresh copies of one baseline and a distinct random cloud tree.
Ownership is established by uploading a marker before bulk data and verified by
reading its exact token before purge. Failed ownership confirmation leaves the
folder recorded for operator review. Cancellation reaps the active child through
TaskReport; cleanup is bounded to 60 seconds per read/purge and partial evidence
is retained. `--keep` is an explicit cleanup exception. Local evidence is retained.
The input sample directory is never a transfer destination, and its final snapshot
must equal its initial snapshot. Observation intervals are bounded 0..60 seconds;
no assertion about delayed changes beyond the measured window is made.


## OneDrive phase-3 prototype receipts (October 15, 2026)

The fixed-action wrapper uses shared TaskReport schema 1, real exit status,
source HEAD/dirty flag, selected action/transport, private log and an
artifact_receipt_path to the Rust example's `prototype.json`. The Cargo invocation
is quiet so account arguments cannot enter its echoed command line. No tokens,
account names, Graph bodies or rclone config dumps are printed by the example.
It captures bounded credential responses through the restricted
CommandRunner private_config method, now shared with production Microsoft OAuth
(`src/microsoft_oauth.rs`). Identity/setup now delegates to production
`src/onedrive.rs`; scenario orchestration and the prototype example remain
development-gated. Receipt schema/actions and the `cm_proto_` creation restriction
are unchanged.
Capture is restricted to rclone config dump/create/update
with a 1 MiB cap and at most 300 seconds. The private response has opaque Debug
output, no serialization/raw public fields and a bounded JSON decoder. Both output
bodies are replaced with opaque text on command failure. Ordinary command output
remains redacted. It emits protocol error codes rather than raw provider errors.
Profile `/me` refusals use `account_profile_unauthorized` with HTTP 401/403;
other Graph HTTP errors retain `graph_http`. This distinction does not infer the
provider's underlying cause. New personal prototype OAuth requests the default
OneDrive file scopes plus `User.Read`; business uses rclone's default scopes
without an override. Business identity requires equality of configured drive,
signed-in `/me/drive` and expected-UPN `/users/{UPN}/drive`, with matching nonempty
owner user IDs and business types. Personal still uses profile `/me`. Typed Graph
targets build only fixed-host URLs, encoding the validated business UPN as one
path segment. Denied/malformed responses fail closed; no JWT-claim identity
fallback is used. Inspect never changes consent on existing remotes.
`rclone_failed` contains fixed `operation` and `failure` fields. Operations
distinguish identity access, config, owner/anchor reads, download verification and
other engine requests. Failures distinguish missing executable, invalid argument,
spawn, timeout and nonzero exit (with optional numeric code). Cancellation retains
its cancellation code. No command text, arguments or stdout/stderr enter this
diagnostic. Shared coordinator failures retain their existing generic code.

`prototype.json` schema 1 records succeeded/failed, result/error_code and, on a
successful prepare/session action, its retained session path. Private session.json
schema 1 retains a UUID, remote reference, identity fingerprints, initialization
observed from shared coordinator markers, failed/owned/cleaned flags, operation
durations/outcomes and a labelled synthetic benchmark if measured. Session
ownership and shared access markers are validated; a setup failure leaves partial
session evidence in the prepare report directory. Shared coordinator state and
bisync logs/listings are retained under the session work root. No source SHA
claims clean or immutable inputs when source_dirty=true.

Later actions require the same explicit remote/drive/account and absolute session
path. A shared operation guard serializes each CLI session action, in addition to
the coordinator lock around each transaction. Prepare never automatically confirms initial sync; the separate initial-sync
action requires shared Preview evidence. Failed sessions block ordinary sync;
the separate verify action requires an initialized, nonfailed session and reruns
identity/ownership/access and downloaded-byte checks without another bisync.
Repair only inspects pending data/locks and no implicit rebuild is offered.
Explicit cleanup rechecks drive/account/root ownership and purges only that UUID
cloud tree, retaining local evidence. A forced-host transport run is not final
installed Flatpak or browser-portal acceptance.

Sample timing extension: prepare accepts an absolute --files-dir only for a new
session. It reads the source into a session-local sample copy with bounded rclone
copy, compares source-before/source-after/copy SHA-256 tree inventories, then
creates the cloud owner/tree and Preview. Links/special files, non-UTF-8 names,
overlapping paths, depth over 64, over 20,000 entries, over 2 GiB and empty samples
are rejected. Existing session schema 1 files remain readable: optional sample_tree
stores aggregate files/bytes/directories/depth and a digest, never source paths,
filenames or contents. Normal scenario rejects sample sessions.

The timing action requires initialized/nonfailed sample state. It reuses the
guarded planner/coordinator for two quiet transactions (one followed by full
checksum verification), changes only a separate synthetic timing file, measures
that sync and independently verifies downloaded bytes. Benchmark scope is
operator_sample_tree, with sample_tree aggregates, small_change_sync_ms and
initial_sync_engine_ms. The latter is read from the successful initial-sync
stage and excludes entry guards; action timings include guards. Source_policy
is read_only_copy_verified_at_prepare. Representative_tree=false and
cadence_exercised=false explicitly prohibit inferring representative performance
or five-minute service behavior from a small project baseline. Cleanup retains
local sample/evidence and purges only the owned UUID cloud root.


## Flatpak read-only preflight (October 16)

`build-flatpak.sh --preflight --manifest PATH --arch ARCH --branch BRANCH`
uses task `flatpak-preflight`, common schema-1 summary and a private schema-1
`preflight.json` receipt. No source SHA/output directory is required for inspection;
output/reuse arguments are rejected. The receipt observes source/manifest pins,
dirty state, typed runtime/SDK/extension and BaseApp refs/commits, existing user and
system application ref/commit/metadata, and read-only global/application overrides.
Tool invocations have 20-second deadlines and shared bounded capture/cancellation.

`status=succeeded` means inspection completed. `ready_for_candidate_build=false`
plus blockers is a valid inspection result; `acceptance=not-run` and
`installation=not-performed` prevent promotion to acceptance. Unexpected errors
fail the task. No build, installation, launch, override write, service or cloud
operation is reachable from this mode. Candidate/reuse validation now compares
full typed refs (`runtime/...` versus BaseApp `app/...`) and includes declared SDK
extensions; inaccurate older provenance cannot silently satisfy a changed ref set.


## Local Git transport for candidate builds (October 16)

Optional `--local-source-repo PATH` validates the selected application's pinned
commit with bounded read-only Git, then gives only the builder child an appended
Git config environment rewriting that exact logical HTTPS app URL to the local
Git repository. The manifest's full commit and input hashes remain authoritative;
no worktree/directory source or mutable ref is accepted, and no global Git config,
source manifest URL, dependency transport or installed application is changed.
The additive `source_transport` receipt records manifest-url or local-git plus
repository/logical URL/commit. Existing schema-2 reuse retains original provenance
without requiring the local repository. The option is rejected for preflight/reuse.
The shared task runner accepts an optional child environment; existing callers
inherit unchanged behavior and no environment values are printed by the runner.
