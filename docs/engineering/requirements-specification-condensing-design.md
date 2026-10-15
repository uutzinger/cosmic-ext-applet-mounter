# Requirements and Specifications — condensation map and plan

Type: Design

Status: Approved plan (October 15, 2026); steps 1-3 done (preamble, sections 1-5, 7, 8, 10-12, 15, 16).
This record is an observed baseline of the specification at commit 936c2f0, not a
claim about the product.

Created: October 15, 2026

Last reviewed: October 15, 2026

Scope: condensing `Requirements and Specifications.md` (2192 lines, about 130 KB,
18 numbered sections) after `Applet Description.md` was condensed. Associated
task: Task List "Requirements and Specifications condensation". The map was
produced by a read-only review; line numbers refer to the baseline and shift as
sections are edited.

## Decisions

- Keep every requirement ID (`FR-nnn[suffix]`, `NFR-nnn`, `FR-SFTP-nnn`); merge
  bodies, renumber nothing. After each step compare the sorted ID list with the
  baseline (`grep -oE 'FR-[A-Z0-9-]+|NFR-[0-9]+'`).
- IDs cited outside the specification and therefore fixed: FR-031, FR-031A,
  FR-031B, FR-031C, FR-053, FR-060A, FR-060B, FR-060C, and "4.2 Offline behavior"
  (Task List.md lines about 132-135). Update that section citation if 4.2 moves.
- The status matrix (implemented / pending acceptance / future) lives only in
  `Applet Description.md`. The specification carries no status, date, release or
  "planned" wording; such text moves to `Task List.md` or the completion notes.
- Every "shall" in a safety-critical section must survive; list any merged-away
  sentence in the commit message. Use `grep -c shall` before and after.
- Work section by section in the order below; commit after each step.

## Order of work

1. Preamble, sections 1-5, 15, 16.
2. Sections 12, 10, 11.
3. Sections 7, 8.
4. Section 13 together with 6.1 and 6.2.
5. Sections 6.3-6.5 and 6.7.
6. Section 14.
7. Section 17, then 18 with the three unnumbered dated blocks folded into 6.x.
8. Sections 9.4 and 9.5.
9. Final grep for stale tools, ID-list diff, and the Task List line 447 item.

## Safety-critical sections (verify every shall)

6.5 (FR-040-043, FR-044E/EA), 6.6 (FR-047, FR-057-062), 6.8 (FR-074-080), 6.10
(FR-093), 6.11, FR-014/015/018/018A/019/020, 9.3, 9.4, 10.1, 11.4, 12, 18.1 and the
SharePoint identity gate, the 18.3 unattended-safety gate, FR-SFTP-006, NFR-006,
NFR-007, NFR-012.

# Baseline map

Layers: REQ requirement, IMPL implementation mechanism, UI copy/layout detail, GEN generated-service detail, TEST test spec, HIST history/status narrative.

## 1. Section table
| Lines | KB | Section | Purpose | Layer |
|---|---|---|---|---|
| 3-26 | 1.3 | Preamble + "Release delivery boundary - Oct 13, 2026" | 0.5.3 vs 0.5.4 delivery, abraunegg drop, WebKit helper | HIST (delete or move to Task List) |
| 27-43 | 0.6 | 1 Purpose | intent | REQ (dup of Description Purpose) |
| 44-66 | 1.3 | 2 Goals | goals | REQ (dup) |
| 67-115 | 2.1 | 3 Current Scope (3.1 matrix, 3.2 included, 3.3 non-goals) | provider/mode matrix | REQ |
| 116-175 | 2.8 | 4 Technical Clarifications (4.1-4.5) | online limits, offline behavior, GDocs, NM VPN, settings windows | REQ (restates Description + FR-0xx) |
| 176-226 | 2.2 | 5 User Workflows (5.1-5.4) | numbered flows | REQ/narrative, dup of Description UI |
| 227-302 | 4.4 | 6.1 Panel and status (FR-001..010C) | popup behavior | REQ + UI (001A "layout revision Sep 15", 003/003A/003B row layout) |
| 303-392 | 7+ | 6.2 Connection mgmt (FR-011..020) | wizard, Detect rclone, IDs, removal | REQ + heavy UI copy (011A-011N) |
| 393-419 | 1.5 | 6.3 Dependency mgmt (FR-021..026B) | min versions, no startup probes | REQ |
| 420-471 | 3.5 | 6.4 Provider behavior (FR-027..033) | remote create, auth, OneDrive offline | REQ (+transition 031C) |
| 472-556 | 6.5 | 6.5 Online mount mgmt (FR-034..044I) | VFS, detach/remount, preload | REQ, 044A-I are very detailed (IMPL/UI) |
| 557-616 | 3.5 | 6.6 Offline mirror mgmt (FR-045..062) | preview, bisync, repair | REQ, SAFETY |
| 617-632 | 0.9 | 6.7 Legacy import (FR-063..070) | import | REQ |
| 633-664 | 2.0 | 6.8 Systemd service mgmt (FR-071..080) | unit ownership/atomic write | REQ + GEN, SAFETY |
| 665-747 | 5.5 | 6.9 VPN dependencies (FR-081..090D) | NM VPN, OpenConnect agent | REQ + IMPL; FR-086A/B/087A "planned A" |
| 748-759 | 0.7 | 6.10 Errors/logs (FR-091..096) | redaction, notices | REQ, SAFETY |
| 760-814 | 3.3 | 6.11 Sleep/wake (FR-097..102) "B; live acceptance pending" | logind inhibitor | REQ, status in title |
| 815-843 | 1.7 | 7 NFR (NFR-001..013) | quality | REQ |
| 844-885 | 2.1 | 8 System Requirements (8.1-8.3) | platform, tools table w/ versions, permissions | REQ (stale rows) |
| 886-1125 | 14.2 | 9 Architecture (9.1 modules, 9.2 runtime, 9.3 command exec, 9.4 Flatpak 951-1081 ~8 KB, 9.5 General Settings runtime 1082-1125) | design | IMPL; 9.3 is SAFETY; 9.4/9.5 largest condensation targets |
| 1126-1195 | 2.0 | 10 Configuration Model (+10.1 Validation) | config schema/validation | REQ/IMPL, SAFETY (validation) |
| 1196-1356 | 7.0 | 11 Generated Service Specs (11.1 online 1198, 11.2 offline 1274, 11.3 OneDrive 1328, 11.4 ownership 1337, 11.5 remote mgmt 1342) | unit contents | GEN |
| 1357-1390 | 2.0 | 12 State and Operation Rules | state machine | REQ |
| 1391-1576 | 11.5 | 13 UI Spec (13.1 popup 1393, 13.1A General Settings 1431, 13.2 Add/Modify/Import/help 1474-1565 ~5 KB, 13.3 errors 1566) | UI | UI copy; heavily duplicates 6.1/6.2 |
| 1577-1728 | 8.6 | 14 Testing (14.1 auto 1579, 14.2 isolated 1629, 14.3 manual 1645-1728) | test spec | TEST |
| 1729-1759 | 1.7 | 15 Current Acceptance Criteria (item 14 cites "v0.2 popup") | acceptance | TEST/HIST |
| 1760-1821 | 3.6 | 16 Decisions Required Before Development / 16.1 Resolved | approved decisions | HIST (mostly dup of 3, 6, 9) |
| 1822-1962 | 9.5 | 17 SFTP Extension - Sep 29, 2026 (17.1 reqs 1829, 17.2 setup/runtime 1878, 17.3 verification 1941) | FR-SFTP-001..016 | REQ + HIST (status para 1824-1827) |
| 1963-2192 | 15.5 | 18 SharePoint Engine "Implementation Plan, Oct 1, 2026" (18.1 identity 1977, 18.2 online 2004, 18.3 offline 2034-2090, 18.4 acceptance 2091) + 3 unnumbered dated sections: identity gate 2118, mountpoint preflight 2131, pending-change count 2143 | SharePoint + cross-engine rules | REQ + HIST/plan; unnumbered sections hold cross-engine requirements that belong in 6.x |

Note: sections 6.x total 37 KB (29%), 9: 14, 13: 11.5, 17+18: 25 (19%), 14+15+16: 14.

## 2. Duplication (canonical copy in bold)
- Product intent/goals/scope: Description (Purpose, Providers, Modes) vs spec 1-3, 4.1-4.2, 5. Canonical: **Description**; spec 1,2,5 reduce to pointers; keep 3.1 matrix and 4.x only where it adds a testable rule.
- Online/Offline behavior: 4.1/4.2 (116-141) vs 6.5/6.6 vs 12 (1357) vs 15 items 3-8 vs 16.1 item 3. Canonical: **6.5/6.6 FRs**; 4.x, 15, 16.1 become cross-refs.
- NM VPN: 4.4 (148-165), 6.9 (665-747), 8.2 (855-878), 13.2 (~1515), 16.1 (~1813), 9.4 (~1063), Description VPN. Canonical: **6.9**.
- Flatpak/host integration: 9.4 (951-1081), 9.3 (932-950), FR-080/083/084A, NFR-007, 8.3, Description Native/Flatpak. Canonical: **9.3 for exec policy; Description for stance; trim 9.4 and move rationale to docs/engineering (flatpak-host-command-inventory.md)**.
- Settings windows / General Settings: 4.5 (166), FR-001C, 9.5 (1082), 13.1A (1431), Description. Canonical: **13.1A for UI, 9.5 for IPC**; drop 4.5.
- Popup/row layout: FR-001A/003/003A/003B/010A-C vs 13.1 (1393). Canonical: **13.1**, FRs keep behavior only.
- Add/Modify wizard & field help: FR-011A..011N (303-366) vs 13.2 (1474-1565) vs FR-SFTP-002..004 vs 5.1. Canonical: **FRs for behavior (IDs kept), 13.2 for layout only**; merge.
- Preload: FR-044A-044I (498-556), 13.1A, FR-SFTP-015/016, Description "Online directory-cache preload". Canonical: **FR-044x**; SFTP items cross-ref.
- Pending-change count: unnumbered 2143-2192 vs FR-008, FR-039, 13.1, 6.1 FR-006A. Canonical: **move 2143 block into 6.5/6.6 as FR (needs new ID or attach to FR-039)**.
- Mountpoint preflight (2131) vs FR-037A (480) vs FR-014/015: merge into FR-037A.
- SharePoint identity gate (2118) vs 18.1/18.2 and 11.1: canonical 18.1 + 11.1 generated check.
- Systemd unit rules: FR-071-080 vs 9.3 vs 11.x vs 11.4 ownership. Canonical: **6.8 for rules, 11 for literal unit fields**.
- Acceptance: 14.3 manual (1645-1728), 15, 17.3, 18.4, plus Task List. Canonical: **14.3 as one acceptance matrix; per-engine pending status goes to Task List**.
- Resolved decisions 16.1 vs 3.1/6.4/9: delete after confirming each decision lives in a FR.
- Transition abraunegg text: preamble 21-24, 98, 139, FR-031C (463), FR-060C (606), 863, 868, 873, 898, 1007, 1333, 2176. Canonical: **FR-031C/060C only** (one-release transition allowed).

## 3. Stale / history / removed-tool text
- Dated headings: 11 (Oct 13), 21 (Oct 14), 232 FR-001A "(layout revision, Sep 15)", 1822 (Sep 29), 1963 (Oct 1), 2118/2131/2143 (Oct 2). Remove dates.
- Status narratives (move to Task List/Completion Notes): 11-25 (0.5.3/0.5.4 delivery); 80 (Section 17 pointer); 760 title "B; live acceptance pending"; FR-086A/086B "(A)", FR-087A "planned A" (715-725); 1136-1137 "# planned B"; 1581 (Task List section pointer); 1748 item 14 "v0.2 popup"; 1822-1827 (SFTP implemented, acceptance pending); 1963-1968 and "not yet a release-supported connection"; 1784 "Planned repository"; 862 "OneDrive-specific minimum pending"; 2109-2117 (18.4 prototype Flatpak remark).
- "Implementation Plan" tone / first-second deliverable in 18.2/18.3 (2004, 2034): plan, not requirement; convert to requirements.
- Removed Cisco client: 57, 152, 154, 736, 743-745, 1515-1516, 1813-1816. Keep only FR-090D and one sentence "Cisco-compatible = NM OpenConnect profile".
- nmcli/systemctl: 152, 417-418, 582, 651, 679, 687, 1063. Mostly "shall not" negatives; keep as a single policy line (FR-026B/FR-083/9.3) and drop repeats. (582 is FR-055A; 651 FR-078.)
- journalctl: 2173 (one negative; keep inside the pending-count FR).
- Shell wrapper / sh -c: 944, 946, 981, 1295, 1324. Negatives; consolidate into 9.3.
- abraunegg: see Dup list; table rows 863/868 and 873-875 (WebKit helper) are transition-only; preamble 15 still says "optional installed WebKit helper" (stale target-state text). Description is the check: verify it keeps the one-release transition.
- WebKit helper: 15, 868, 873-874, 946 ("Display-only manual-auth"), 1295; target state should reduce to FR-031A + one transition note.
- Release/version numbers: 0.5.3, 0.5.4, v0.2, 0.1 (16.1 item 2 "version 0.1").

## 4. Requirement IDs and external citations
- Schemes: FR-nnn[A-Z...] (about 169 distinct IDs, incl. suffix forms like 011MA, 044EA); NFR-001..013 (13); FR-SFTP-001..016 (16). Total about 185 distinct with suffixes counted.
- Ordering oddities: FR-086A/086B appear after FR-087 (715-722); FR-044EA, FR-011AA/AB/MA, FR-001A-D nested.
- Inside spec: cross-cites at 140 (FR-031C), 551 (FR-SFTP-015), 1540 (FR-011MA), 1848/1858 (FR-011K..MA, "Section 13.2"), 1911; section-number cites: 80 (Section 17), 1767 (3.1), 1858 (13.2), 2008 (18.1), 2109 (18.3).
- Outside spec (excluding archive/target): NO citations in src/, tests/, scripts/. Found:
  - Task List.md: lines 10, 132-135 (FR-031C, FR-031/031A/031B/053/060C, "4.2 Offline behavior", FR-060A-C), 447, 471 (file name only)
  - Task List Completion Notes.md: 79-80 (FR-031/031A/031B/053/060C, 031C)
  - Task List Carry Forward.md: 381 (FR-060A through FR-060C)
  - docs/engineering/onedrive-rclone-offline-contracts.md: 15 (FR-031), 121 (FR-060B)
  - docs/engineering/README.md:61, README.md:276, Applet Description.md:15, AGENTS.md:14 (file name only)
  So the externally cited IDs are only FR-031, 031A, 031B, 031C, 053, 060A, 060B, 060C, plus section "4.2". Keep these IDs stable; all others can be merged if a retired-ID note or Task List update is acceptable. Recommended: keep ALL IDs (they are cheap), merge bodies not numbers; renumber nothing.
  Also grep Task List.md for "Section 17/18/14.3" style refs before touching them (not found by the ID grep except line 133 "4.2").

## 5. Proposed target outline (about 70-75 KB, -45%)
| Section | Keeps | Est KB |
|---|---|---|
| Header + source-of-truth + (no release boundary) | pointer to Description; 3-line scope note | 0.5 |
| 1 Scope (merge 1,2,3,5) | 3.1 matrix, included/non-goals, workflows as 1-line refs | 2.5 |
| 2 Behavior clarifications (4.1-4.4) | only rules not in FRs; drop 4.5 | 1.5 |
| 3 Functional requirements (6.1-6.11) | all FR IDs; shorter wording, UI layout moved out, merge preload/pending-count/preflight/identity-gate FRs here | 24-26 |
| 4 NFR + system requirements (7, 8) | NFRs, version table, permissions (no transition rows beyond one) | 3.0 |
| 5 Architecture (9) | 9.1-9.3 intact, 9.4 cut to host-op list + permissions (move rationale to docs/engineering), 9.5 IPC contract | 7 |
| 6 Config model + validation (10) | schema, validation | 2.0 |
| 7 Generated services (11) | field-level unit contract | 5.0 |
| 8 State rules (12) | state machine | 1.8 |
| 9 UI spec (13) | layout/label contracts only | 6.5 |
| 10 Verification (14+15 merged) | auto/isolated/manual matrix, one acceptance list with level 1-6 tags | 6.5 |
| 11 Provider extensions: SFTP, SharePoint (17,18 recast as requirements) | FR-SFTP-001..016, SharePoint identity/URL/online/offline/gates, no plan/date wording | 11-12 |
Delete: 16 (after check), preamble 11-25, dates, status lines. Total about 72 KB.

Recommended order: (1) preamble/1/2/3/4/5 + 15/16 (low risk, mostly pointers); (2) 12, 10, 11 (small, mechanical); (3) 7/8; (4) 13 UI and 6.1/6.2 together (de-dup layout vs behavior); (5) 6.3-6.5 and 6.7; (6) 14; (7) 17, then 18 + unnumbered sections folded into 6.x; (8) 9.4/9.5 last; (9) final grep for stale tools, IDs preserved (diff of sorted ID list vs original), and update Task List line 447 item.

Safety-critical (extra care, verify every shall survives): 6.6 (FR-047, 057-062 preview, conflicts, repair, recovery retention), 6.5 (FR-040-043 detach, pending-write protection, FR-044E/EA unmount safeguards), 6.8 (FR-074-080 ownership marker, atomic write, rollback), 6.10 (FR-093 redaction), 6.11 sleep, 6.2 FR-014/015/018/018A/019/020 (mountpoint, removal, confirmation), 9.3 (no shell, fixed exec, timeouts, output bounds), 9.4 (host permission minimization), 10.1 validation, 11.4 ownership, 12 state rules, 18.1 + identity gate (account/library drift), 18.3 unattended safety gate, 17.1 SFTP host-key verification (FR-SFTP-006), NFR-006/007/012.
Verification trick: extract `grep -oE 'FR-[A-Z0-9-]+|NFR-[0-9]+'` before/after, and `grep -c shall`; keep a mapping list of any merged-away sentence.

