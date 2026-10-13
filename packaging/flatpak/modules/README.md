# OneDrive WebKit helper support modules

Status: Candidate module definitions; not yet included in the final manifest.
Reviewed: October 13, 2026.

These source-built executables support WebKit's process sandbox. They are app-
local runtime dependencies, not host commands or new host permissions.

| Module | Pinned source | License observations | Verification |
|---|---|---|---|
| bubblewrap | [0.13.0](https://github.com/containers/bubblewrap/releases/tag/v0.13.0) | Core source declares LGPL-2.0-or-later; retain upstream COPYING/per-file notices. | Source digest matches official release asset; built in SDK 25.08 and version probe loads in Platform 25.08. |
| xdg-dbus-proxy | [0.1.9](https://github.com/flatpak/xdg-dbus-proxy/releases/tag/0.1.9) | LGPL-2.1-or-later; retain upstream COPYING/per-file notices. | Source digest matches official release asset; built in SDK 25.08 and version probe loads in Platform 25.08. |

Meson packaging excludes manual pages/completions/test binaries; tests are not
claimed as passing. Module definitions retain upstream license files. The local
probe installed only into target/webkitgtk-2541-compat-20261013/tools-prefix.
Combined unstripped probe executables total about 178 KiB; this is not final
package size and excludes WebKit/PyGObject. Actual nested sandbox behavior is
still subject to WebKit GUI/live acceptance, not a --version check.

WebKit must retain ENABLE_BUBBLEWRAP_SANDBOX=ON and use the final app-local
/app/bin/bwrap and /app/bin/xdg-dbus-proxy. Do not copy host binaries, add setuid
installation steps, disable WebKit sandboxing, or silently broaden grants.
Supply Ruby only for compilation/generation and remove it from the final payload.
Security-update/source/license maintenance belongs to the app when these tools
and WebKit are bundled; SDK updates alone do not update bundled copies.

See [release source audit](../../../docs/engineering/release-source-audit.md)
and the [task list](../../../Task%20List.md) for remaining helper build/import/
authentication and immutable-final-source acceptance. No installed app or accepted
runtime/base has changed merely by adding these module definitions.
