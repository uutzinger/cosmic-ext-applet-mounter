"""Read-only publication metadata/notes preflight for an accepted v-version tag."""

from pathlib import Path
import re
import subprocess
import sys
import tomllib
import xml.etree.ElementTree as ET

sys.path.insert(0, str(Path(__file__).resolve().parent))
from project import PACKAGE


def main():
    if len(sys.argv) != 2:
        raise SystemExit("Supply the accepted v-version tag")
    tag = sys.argv[1]
    if not re.fullmatch(r"v[0-9]+\.[0-9]+\.[0-9]+", tag):
        raise SystemExit("Publication requires a stable v-major.minor.patch tag")
    version = tag[1:]
    cargo_version = tomllib.loads(Path("Cargo.toml").read_text())["package"]["version"]
    lock_versions = [p["version"] for p in tomllib.loads(Path("Cargo.lock").read_text())["package"]
                     if p["name"] == PACKAGE]
    debian = subprocess.check_output(["dpkg-parsechangelog", "-S", "Version"], text=True).strip()
    latest = ET.parse("resources/app.metainfo.xml").find("./releases/release")
    if latest is None:
        raise SystemExit("AppStream metadata has no release entry")
    if (cargo_version != version or lock_versions != [version] or debian != version
            or latest.get("version") != version):
        raise SystemExit("Tag, Cargo, Debian, and latest AppStream versions must match")
    if latest.get("type", "stable") != "stable":
        raise SystemExit("Finalize the AppStream release status before publication")
    notes = Path("docs/releases") / (tag + ".md")
    if not notes.is_file() or re.search(r"\bdraft\b", notes.read_text(), re.IGNORECASE):
        raise SystemExit("Finalize the versioned release notes after acceptance before publication")
    print(f"Publication metadata/notes preflight passed for {tag}; acceptance is a manual gate.")


if __name__ == "__main__":
    main()
