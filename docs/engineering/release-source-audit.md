# 0.5.3 release source review

Type: Audit
Status: In progress — not final source approval
Created: October 12, 2026
Last reviewed: October 12, 2026

## Scope and sources

Supports the final native/packaging gate in [Task List](../../Task%20List.md).
Authoritative inputs are the complete current worktree, Cargo/lock metadata,
Debian packaging, Flatpak manifests, release workflows/scripts, and
[draft notes](../releases/v0.5.3.md). Requirements and the maintained host-command
inventory retain their own implementation and package-acceptance boundaries.

## Current findings

- The migration is a large multi-session change: 55 tracked changed/deleted
  paths plus 28 untracked files at initial review. Untracked source modules,
  tests, agent guidance and moved documents are required source, not disposable
  build output. Do not finalize with only `git commit -am`.
- Cargo/lock/Debian versions match 0.5.3; AppStream is development and notes
  are Draft. The lock delta adds direct libc ownership without changing its
  resolved version. Existing published download/screenshot URLs remain 0.5.2.
- Tag pushes and ordinary manual builds cannot publish through the updated
  workflow. Explicit publication requires the six builds and the stable-version/
  finalized-notes preflight. This is source/fixture verification, not hosted CI.
- Native 0.5.3 amd64 candidate compilation/package tests and actual installed
  upgrade/UI/preservation receipts passed. Broad all-feature tests for the latest
  source passed October 12: 385 passed, zero failed, 12 ignored. Default package
  tests and all-feature coverage remain distinct from ignored live tests.
- The final Flatpak still pins v0.5.2 intentionally. It is not migration evidence.
  Remaining effective-permission, coordinator-path, helper and VPN gates cannot
  be waived by native success.
- Updated stale draft-note text to reflect the actual versioned upgrade receipt.
- Filename-based credential-file checks found no matching private key/token/
  environment/rclone-config artifacts outside ignored build output. This is not
  a complete content-level secret audit or proof that all documentation is public.

## October 12 content/reference review result

- Enumerated 114 existing tracked/nonignored-untracked source files, excluding
  deleted old paths and ignored package/cache/log output. Text scan covered
  private-key, GitHub-token, JWT, AWS-key and literal credential signatures.
  Two literal matches are synthetic passwords in test-only SMB/SFTP fixtures;
  no confirmed credential was found. Binary fixture contents, remote history and
  novel secret formats are outside this signature scan.
- All inspected inline local Markdown file destinations and heading anchors
  resolve. External URLs, reference-style links, public-site rendering and
  remote availability are not established by this check.
- Moved 0.5.0 and 0.5.1 notes are byte-identical to their historical HEAD versions.
  New documentation/modules/tests are required intended source. Ignored build
  output is not part of source finalization.
- Production entry-point spot checks confirm the restricted bisync request
  parser; fixed inspection operations/budgets; default executable feature gates;
  and test-only legacy nmcli/systemctl/systemd-analyze/private-bus paths. Settings
  and authentication helper launches remain separate documented UI actions.
  This checks these architectural boundaries, not every changed line.
- Organization-specific URLs/home paths remain in project history and some
  test examples. They are not authentication secrets, but are a public-document
  privacy consideration. Retain historical receipts rather than silently
  rewriting them; publication review must acknowledge this retained context.

Private diagnostic report: `/tmp/cloud-mounter-source-review-20261012.json`.
No production code or accepted runtime behavior changed during this review.

## Immutable candidate-to-final sequence (proposed)

1. Complete source review and checks, then obtain approval to commit all intended
   source and moved/new documents. Keep package outputs/logs out of that commit.
2. With separate push/tag approval, use an immutable candidate commit or
   prerelease tag such as `v0.5.3-rc1` for candidate Flatpak/hosted acceptance.
   Do not use `scripts/build_release.sh` to create the final stable tag while
   notes and AppStream metadata are still provisional.
3. After candidate acceptance, finalize notes/AppStream and disposition decisions
   in a new commit, then create the final immutable `v0.5.3` tag with approval.
   Never move the candidate or final tag to conceal a later change.
4. Build the final tag and verify identity plus any behavior invalidated by the
   candidate-to-final delta. Implementation/permission changes require affected
   lifecycle tests again. Only separately approved manual publication follows.

The candidate is not the final artifact. Final Flatpak identity/acceptance stays
open even if candidate tests pass. This sequence is a proposed resolution of the
current draft-notes/immutable-tag ordering conflict; no tag or push is authorized.

## Remaining review

October 12 staged inclusion review: all intended new modules/tests/moved documents
are present; no target/artifacts/Flatpak build-output paths are staged. The 88
runtime/build/test/packaging source files compared against the tested native
0.5.3 snapshot are byte-identical. Remaining differences are documentation review
receipts. The staged whitespace check exposed three Markdown hard-break spaces
in the newly included inventory; replaced them with blank-line separation.
Restage that fix and the review receipts before final staged check/commit.

Final cached whitespace check passed after user restaging. Candidate source was
committed locally as `efbf73e` on October 12 under explicit approval. No tag/push
was made. Implementation identity is immutable; any receipt follow-up is a
separate documentation commit, not an amendment or claim of final acceptance.

- Latest all-feature test gate is recorded as passed; ignored live tests are not
  counted as passes and retain their separate prior/remaining acceptance gates.
- Focused content/reference and architecture-boundary checks above passed with
  stated coverage limits; they are not a full security or line-by-line audit.
- Verify a complete clean source snapshot includes compile-time fixtures and
  `.github` workflow input required by the new Debian contract test.
- Staged inclusion/candidate-source comparison is complete; final restaging and
  whitespace check are still required. Commit/tag/push approval remains separate;
  the present
  worktree is not an immutable release identity and has not received full line-by-
  line/security review merely because tests pass.

Detailed receipts belong in [completion notes](../../Task%20List%20Completion%20Notes.md).
