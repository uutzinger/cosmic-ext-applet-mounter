# Historical scripts — do not run as current tooling

Local historical files moved from `archive/scripts/` on October 13, 2026, with
file-by-file SHA-256 parity. No active source, tests, packaging, recipes or current
user guides reference that old directory. All nine active scripts remain in
`scripts/`; none was retired merely because another script wraps it.

| Legacy files | Former purpose | Replacement |
|---|---|---|
| `cisco-vpn-start`, `cisco-vpn-stop`, `cisco-vpn-status` | Proprietary Cisco command/session control | Applet NetworkManager D-Bus VPN integration and documented OpenConnect setup. |
| `cloud-box-onedrive-toggle`, `cloud-box-onedrive-toggle-terminal` | Box/OneDrive shell toggles and terminal launch | Applet managed online connections and typed service control. |
| `cloud-google-toggle`, `cloud-google-toggle-terminal` | Google Drive shell toggles and terminal launch | Applet managed online connections and typed service control. |
| `ua-engr-mount`, `ua-engr-mount-terminal-stop-all` | Organization-specific share/VPN mount control | Saved applet SMB/VPN connection and managed service control. |

These scripts are frozen historical material, not tested current commands. Their
paths, assumptions and external tool requirements may be obsolete. Do not install,
source, execute or add them to build/CI dependencies. Read-only comparison is fine.

The files were already ignored by Git at the original location. The local
`.gitignore` preserves that exclusion; this README is tracked, but the legacy
content is not automatically republished. A lightweight literal-secret scan
found no matches, which is **not** a full privacy/security review. Explicit review
is required before tracking/distributing any of these files. Fresh clones may
therefore contain this README without those local historical files.

New archive candidates require caller/reference review, a replacement or reason
for retirement, preserved provenance, updated current references and recorded
verification. Do not archive an installed runtime payload or a test fixture
because it looks like an old utility. Build output belongs under `target/`, not
here. See the [current catalog](../README.md) and
[script inventory](../../docs/engineering/script-tooling-inventory.md).
