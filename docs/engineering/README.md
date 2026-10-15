# Engineering records

These documents retain inventories, safety contracts, and verification findings.
Documentation status does not establish implementation or final-package acceptance.

| Record | Type | Status and purpose |
|---|---|---|
| [Bisync coordinator baseline](2026-10-10-bisync-coordinator-inventory.md) | Inventory | October 10 historical baseline and equivalence contract for the shell-to-Rust coordinator migration. |
| [Flatpak host-command inventory](flatpak-host-command-inventory.md) | Inventory | Source commands/permissions/docs reconciled and native inspection UI accepted October 12; final effective grants and worker/VPN/package acceptance remain pending. |
| [Offline Mirror Repair contracts](offline-mirror-repair-contracts.md) | Contracts | Accepted engine-specific recovery and pause requirements; implementation/acceptance evidence is tracked separately. |
| [UI translation audit](2026-10-12-ui-translation-audit.md) | Audit | October 12 source and native-test findings, intentional untranslated data, and remaining rendered/final-package checks. |
| [Release source review](release-source-audit.md) | Audit | 0.5.3 worktree review in progress; source inclusion and immutable candidate/final ordering, not final source approval. |
| [Development cleanup inventory](development-cleanup-inventory.md) | Inventory | October 13 approved narrow cleanup: obsolete legacy units/empty unlisted test folders archived reversibly; all listed connections retained. |
| [Script/tooling inventory](script-tooling-inventory.md) | Inventory | October 13 active/archived script classification, caller evidence, stale Rust contract finding and prioritized existing/update/create tooling plan. |
| [Script output/resumption contracts](script-output-contracts.md) | Contracts | P1/P2 tooling; P3 immutable-manifest export, metadata comparison and schema-2 verified reuse without rebuilding. Includes October 15 OneDrive guard schema-2 evidence. Fixtures/synthetic checks remain separate from installed acceptance. |
| [OneDrive Offline mirror on rclone](onedrive-rclone-offline-contracts.md) | Contracts | October 14 accepted design for dropping `abraunegg/onedrive`: configuration, setup, guards (generic profiles verified on both sample trees October 15), scheduling, legacy handling and acceptance criteria; implementation not started. |
| [Release, packaging and documentation procedures](release-packaging-procedures-contracts.md) | Contracts | October 13 relocation of build/artifact policy, release gates, native/Debian/Flatpak/GitHub procedures, release-note rules and package script bindings from the agent guide; read on demand. |

## Rules

Engineering records are maintained design and verification documents: inventories,
contracts, audits, designs, and decisions. Keep them under
`docs/engineering/`, separate from the repository's primary product,
requirements, task, and deployment documents.

- Use lowercase ASCII filenames with hyphens: subject-kind.md. Prefer the kinds
  inventory, contracts, audit, design, and decision. Choose a descriptive subject
  and retain the same filename while maintaining the same record.
- Keep dates in document metadata for maintained records. For a separate frozen
  baseline or historical snapshot, use YYYY-MM-DD-subject-kind.md and state which
  current record or task it supports. Do not create a new dated copy for every
  small update or silently overwrite an accepted historical baseline.
- Create the directory and its README.md index with the first record. List each
  record's purpose, type, status, and relative link in the index. Avoid deeper
  subdirectories until the number of records makes them useful.
- Track reviewed engineering records in version control. Compiled binaries,
  packages, caches, and transient raw reports/logs belong in the build tool's
  output area, such as `target/`, or a task-specific temporary directory.
  They are not engineering records and must not be mixed into that directory.

## Creating and maintaining records

1. Create a record when work needs a durable baseline inventory, interface or
   safety contract, architecture decision, or audit beyond a checklist entry.
   Use existing records for routine follow-ups instead of creating redundant
   documents. Establish a required baseline or contract before its implementation.
2. Begin with a title, type, status, creation date, and last-reviewed date. Identify
   the scope, associated task, applicable requirements, and authoritative source
   files or interfaces. State whether the text describes an observed baseline,
   a proposed design, an accepted contract, or verified behavior.
3. Record assumptions, relevant versions/capabilities, ownership, dependencies,
   decisions, and unresolved issues. For inventories, classify each item and its
   disposition. For contracts, define permitted operations, invariants, failure
   handling, and acceptance criteria. For audits, state the method, coverage,
   intentional exceptions, findings, and corrective actions.
4. Separate documentation status from implementation and acceptance status.
   Identify what was checked in source, automated tests, native installations,
   live systems, and final sandboxed packages. Include dates and build identities
   when relevant; do not turn an accepted design into a claim of tested behavior.
5. Link the record from the appropriate task and the directory index. Put durable
   product requirements into `Requirements and Specifications.md`, or reference an accepted
   contract there. Keep detailed execution chronology in `Task List Completion Notes.md`
   and avoid copying the same checklist or transcript into multiple records.
6. Update maintained records when related implementation or decisions change.
   Preserve frozen baselines and mark superseded records with links to their
   successors. Maintain references if a record is renamed and verify relative
   links from its actual directory. Review for secrets and private diagnostic
   data before recording evidence.

See the [agent guide](../../AGENTS.md) for the short summary and the [task list](../../Task%20List.md) for current work. Detailed execution history
is retained in [completion notes](../../Task%20List%20Completion%20Notes.md).
