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
| [Script output/resumption contracts](script-output-contracts.md) | Contracts | Accepted P1 receipt/logging, exact run selection, cancellation and safe resumption boundaries; fixture verification separate from hosted acceptance. |

See the [agent guide](../../AGENTS.md) for naming and record-creation rules and
the [task list](../../Task%20List.md) for current work. Detailed execution history
is retained in [completion notes](../../Task%20List%20Completion%20Notes.md).
