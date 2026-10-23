# Engineering records

These documents retain inventories, safety contracts, and verification findings.
Documentation status does not establish implementation or final-package acceptance.

| Record | Type | Status and purpose |
|---|---|---|
| [Application design](../../Design.md) | Design and source architecture guide | October 22: implemented 0.5.5 process/module structure, Online/Offline lifecycles, host/Flatpak boundaries and all 36 Rust source files. Root placement explicitly requested by operator; source description is separate from acceptance. |
| [Bisync coordinator baseline](2026-10-10-bisync-coordinator-inventory.md) | Inventory | October 10 historical baseline and equivalence contract for the shell-to-Rust coordinator migration. |
| [Flatpak host-command inventory](flatpak-host-command-inventory.md) | Inventory | October 20: 0.5.5 legacy executable removal and typed owned-unit retirement implemented; installed upgrade pending. October 19 source/23-grant and installed override reconciliation complete; scoped worker/VPN gates accepted by evidence reuse. Retained host authority/owners/removal criteria explicit; COSMIC exception review deferred to 0.5.5; GitHub final release acceptance open. |
| [Offline Mirror Repair contracts](offline-mirror-repair-contracts.md) | Contracts | Accepted engine-specific recovery and pause requirements. October 18 installed test exposed hidden Repair for failed manual coordinator with paused units; typed cached-state fix and focused regressions passed, replacement 82c1d7a installed. Disposable personal Repair and subsequent checksum-verified sync accepted; preserved 17-file baseline and paused units verified. Other lifecycle acceptance remains open. |
| [UI translation audit](2026-10-12-ui-translation-audit.md) | Audit | October 12 baseline retained; October 19 follow-up records helper retirement, passing English catalog guard, packaged catalog parity and scoped installed wording/UI observations. Additional-language and exhaustive accessibility acceptance not claimed. |
| [GUI action layout](gui-action-layout-inventory.md) | Inventory and proposed contract | October 21 source baseline covers Settings and every Add/Modify provider-mode layout. Remote manager, Refresh removal, Test/Verify merge and Add/Modify layouts implemented; shared action policy and blue/grey/red disabled-style wiring implemented with synthetic transition tests; October 22 Google repair/history audit, Box shared-flow repair and SharePoint whole-library Offline support and runtime ownership recovery added; explicit shared-remote Confirm Save label and Google root-first/fast-list repair with visible private-OAuth recommendation implemented; scoped native button, Box browser/setup, Google/Box mirror and Settings reconnection acceptance recorded; remaining native/package gaps are tracked in the October 22 acceptance reconciliation. |
| [0.5.4 release acceptance map](release-054-acceptance-audit.md) | Audit | Scoped steps 2–6 reviewed/accepted at recorded source/native/prototype/Flatpak levels. Candidates 7069ae5 verified; native notice and focused Flatpak VPN accepted. Unchanged tests reused, legacy live skipped, unrun limits explicit. Step-7 preparation complete; October 20 publisher checks and hosted x86 VM install/panel/settings/editor smoke passed. Separate clean-profile check skipped by operator with limits recorded; ARM installed unverified. v0.5.4 published from aa25846; all ten public asset downloads and installer URLs verified. Completed records archived, live limitations retained; COSMIC submission deferred. |
| [COSMIC submission readiness](cosmic-submission-audit.md) | Audit | Internal candidate facts, public template checks and open maintainer questions. Published 0.5.5 app/tag/manifest/provenance identities refreshed; scoped exact x86_64 installed acceptance and limits recorded. 23 declared grants, inherited x11 and external dependency model documented. Operator-authored COSMIC PR/maintainer review pending. |
| [Release source review](release-source-audit.md) | Audit | 0.5.3 worktree review in progress; source inclusion and immutable candidate/final ordering, not final source approval. |
| [Development cleanup inventory](development-cleanup-inventory.md) | Inventory | October 13 approved narrow cleanup: obsolete legacy units/empty unlisted test folders archived reversibly; all listed connections retained. |
| [Requirements condensation plan](requirements-specification-condensing-design.md) | Design | October 15 approved map, decisions and section order for condensing `Requirements and Specifications.md`; all nine steps done. |
| [Script/tooling inventory](script-tooling-inventory.md) | Inventory | October 13 tooling inventory; October 20 dependency installer removes legacy --onedrive while retaining supported engines and build Python. |
| [Script output/resumption contracts](script-output-contracts.md) | Contracts | P1/P2 tooling; P3 immutable-manifest export, metadata comparison and schema-2 reuse; October 20 per-CPU hosted Flatpak promotion/candidate-only receipts with strict exact-tag publication. Includes October 15 OneDrive guard schema-2 evidence. Fixtures/synthetic checks remain separate from installed acceptance. |
| [OneDrive Offline mirror on rclone](onedrive-rclone-offline-contracts.md) | Contracts | Accepted October 14 design; phase-4 implementation and scoped phase-6 Flatpak migration smoke closed October 18. Installed 82c1d7a pins 67b3aec; guard/Repair preservation, idle wake and manual VPN checks accepted alongside earlier personal/corporate/SharePoint evidence. Native/Debian smoke accepted October 19; legacy live handoff/monitor/transfer waived by operator, with unrun coverage explicit in acceptance map. Broader release acceptance remains open. 0.5.4 retains legacy support/deprecation; 0.5.5 removal source is implemented with installed upgrade acceptance pending. Chronology in completion notes. |
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
