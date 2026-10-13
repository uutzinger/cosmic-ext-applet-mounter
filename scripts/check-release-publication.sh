#!/usr/bin/env bash
# Read-only publication preflight. Run from the accepted source checkout.
set -euo pipefail
python3 - "${1:?Supply the accepted v-version tag}" <<'PY'
from pathlib import Path
import re
import subprocess
import sys
import xml.etree.ElementTree as ET

tag = sys.argv[1]
if not re.fullmatch(r'v[0-9]+\.[0-9]+\.[0-9]+', tag):
    raise SystemExit('Publication requires a stable v-major.minor.patch tag')
version = tag[1:]
package_section = Path('Cargo.toml').read_text().split('[package]', 1)[1].split('\n[', 1)[0]
cargo_version = re.search(r'^version\s*=\s*"([^"]+)"', package_section, re.MULTILINE).group(1)
lock_package = next(block for block in Path('Cargo.lock').read_text().split('[[package]]')
                    if re.search(r'^name = "cosmic-ext-applet-mounter"$', block, re.MULTILINE))
lock_version = re.search(r'^version = "([^"]+)"', lock_package, re.MULTILINE).group(1)
debian = subprocess.check_output(['dpkg-parsechangelog', '-S', 'Version'], text=True).strip()
latest = ET.parse('resources/app.metainfo.xml').find('./releases/release')
if cargo_version != version or lock_version != version or debian != version or latest.get('version') != version:
    raise SystemExit('Tag, Cargo, Debian, and latest AppStream versions must match')
if latest.get('type', 'stable') != 'stable':
    raise SystemExit('Finalize the AppStream release status before publication')
notes = Path('docs/releases') / (tag + '.md')
if not notes.is_file() or re.search(r'\bdraft\b', notes.read_text(), re.IGNORECASE):
    raise SystemExit('Finalize the versioned release notes after acceptance before publication')
print(f'Publication metadata/notes preflight passed for {tag}; acceptance is a manual gate.')
PY
