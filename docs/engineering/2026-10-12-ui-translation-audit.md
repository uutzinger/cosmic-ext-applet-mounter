# UI translation audit — October 12, 2026

App-owned text displayed in the popup, settings/editor, tooltips, folder picker,
operation notices, validation results, dependency warnings, and recovery messages
now comes from `i18n/en/cosmic_ext_applet_mounter.ftl` through `fl!`.
English is currently the only catalog. This work prepares additional translations;
it does not claim translated-language or rendered UI acceptance.

## Scope and changes

- Reviewed production Rust expressions and messages flowing from library modules
  into the UI: storage-engine setup, online mounts, offline synchronization and
  Repair, SharePoint verification, VPN readiness, systemd control, configuration
  storage, legacy import, preload, and sleep.
- Included the separate GTK OneDrive authentication helper. The applet sends
  its four already-localized labels in a JSON environment value; direct launches
  read the same four plain messages from the packaged English catalog. Native,
  Debian, and both Flatpak install recipes include that catalog, without adding
  a Python Fluent dependency or changing the two-file authentication arguments.
- Extracted operation summaries and actionable errors with interpolation
  arguments. Provider names, user values, filenames, and diagnostic details
  remain arguments rather than becoming part of a translation key.
- Localized helper labels and fragments: Preview/Sync stages, yes/no, import
  state, picker titles/examples, readiness details, startup-policy rollback
  notices, and dependency names.
- Configuration validation formats each typed error as a localized message.
  Save errors join those messages instead of exposing a Rust `Debug` dump.
- Retention warnings use Fluent numeric plural selection instead of Rust-built
  English suffixes.
- Sleep warning visibility uses a typed attention flag carried by the sleep
  event and private runtime IPC. It no longer depends on English text prefixes.
  The IPC flag has a serde default for older payloads.

## Intentional untranslated values

These are data or internal contracts rather than app-owned UI prose:

- Saved user connection/profile names and user-entered values.
- Executable/provider product names, paths, URLs, UUIDs, remote references,
  command arguments, D-Bus member/property names, and systemd state tokens.
- Numeric input examples, counts, punctuation, and status glyphs.
- Third-party/system error details interpolated inside localized explanations.
- English substrings used to classify external rclone/onedrive responses. They
  must continue matching the programs' output rather than the desktop language.
- Ownership markers, rclone filters, initialization artifacts, restricted worker
  success/probe records, and other machine-readable payloads.
- Internal cancellation/deadline sentinels consumed into typed preload outcomes,
  panic assertions, developer logs, excluded test fixtures, and examples.

Persisted warning strings retain the language in which they were generated
until a later operation replaces them.

## Verification

`python3 scripts/check-ui-translations.py` checks catalog key uniqueness and
literal Rust message references. Compilation additionally validates Fluent
syntax and interpolation arguments. The checker does not classify arbitrary
Rust literals as UI text; that classification was reviewed during this audit.

The completed audit has 1,291 English messages and 1,443 literal Rust references,
with no missing or duplicate keys. Formatting, locked all-target compilation,
and strict all-target Clippy pass. The full native host suite passed 358
nonignored tests, including both socket tests. Regression coverage includes
singular/plural retention warnings, attention-state propagation with non-English
sleep text, and backward-compatible runtime IPC payloads.
Three Python helper tests passed without opening GTK or authenticating: localized
handoff, incomplete handoff rejection, and source/installed fallback catalog
discovery. The existing `just localization-check` guard now also checks catalog
integrity and excludes test-module notices from its production scan.

Installed/native and final Flatpak UI smoke checks remain part of release
acceptance. Future translations should review message length and wrapping in
both the popup and connection editor.


## October 19 release acceptance follow-up

The October 12 helper and message-count sections above are historical. The
WebKit helper and its helper-only labels were retired for 0.5.4; no helper is
shipped in current7069ae5 candidates. Current source localization receipt
 dev-localization-check-uor7nshy passed with one English catalog, 1,363 fallback
messages and 1,576 Rust references. The added disabled-mirror Save notice passed
source checks and rendered in the installed replacement Debian VM. Final
Debian/Flatpak catalogs were compared byte-for-byte with source.

Consolidated step6 reuses native settings/editor and current Flatpak panel/popup
observations plus earlier installed recovery/legacy wording. This is scoped
English UI evidence, not a full visual/accessibility or additional-language
acceptance claim. No new catalog/code edit or repeat passing guard required.
Final release checks remain in the current acceptance map.
