# Reusing the development workflow in another project

This repository's agent-assisted workflow is built to be copied to another Rust
project (originally a COSMIC applet). It has four parts. Copy what you need, then
adapt the project-specific items listed for each.

## 1. Agent guide

[AGENTS.md](../AGENTS.md) is loaded every session, so it stays short.

- **Keep as is:** the routine verification loop, design rules, verification-level
  ladder, scripts policy, Git/approval gates, task and evidence rules.
- **Replace:** the project-variables block at the top and Part II (COSMIC APIs,
  applet/UI rules, recipes, packaging entry points).
- **On demand:** release, packaging and documentation rules live in
  [release-packaging-procedures-contracts.md](engineering/release-packaging-procedures-contracts.md)
  and the engineering-record process in the [engineering README](engineering/README.md).
  Keep that split so routine tasks do not pay for release detail.

## 2. Scripts

`scripts/` provides logged, predictable entry points: `dev.sh` (fixed `just`
recipes), the hosted-build and release scripts, artifact inspection, and
`archive-release.sh`. Every task writes a private log and a one-line JSON receipt
with the real exit status. Adaptation steps, the constants to edit
(`scripts/lib/project.py`) and the project files the tools read are in
[Reusing the tooling in another project](../scripts/README.md#reusing-the-tooling-in-another-project).

## 3. Tests

Copy the matching `tests/*_test.py` files and `tests/tooling_fixtures.py`, and make
`just test` run every `tests/*_test.py` (see this repository's justfile).
`repo_hygiene_test.py` keeps the workflow honest: entry points stay executable
and listed in the script catalog, relative doc links resolve, and the tasks
`dev.sh` allows exist in the justfile and in AGENTS.md.

## 4. Task records

Keep `Task List.md` for open work and `Task List Completion Notes.md` for dated
evidence. After each release, run `scripts/archive-release.sh` (use `--dry-run`
first) to move completed history to `docs/archive/` and carry open items forward.

## Order of adoption

1. Copy `scripts/lib`, `dev.sh`, `justfile` recipes and the tests; edit
   `scripts/lib/project.py`; get `just gate` green.
2. Copy `AGENTS.md`, replace the variables block and Part II, and trim anything
   your project does not have.
3. Add the release scripts, workflow and documentation layout only when you
   publish releases.
4. Add `.claude/settings.json` so routine commands (`dev.sh`, read-only Git) do
   not prompt, while commit, push, tag, install and publish still do.
