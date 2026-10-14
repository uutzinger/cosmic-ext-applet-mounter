"""Offline artifact identity/checksum policy shared by operator tools."""

import hashlib
from pathlib import Path
import re
import stat

MATRIX = [(version, arch) for version in ("22.04", "24.04", "26.04")
          for arch in ("amd64", "arm64")]
PACKAGE = "cosmic-ext-applet-mounter"


def regular_file(path):
    path = Path(path).absolute()
    if any(p.is_symlink() for p in [path, *path.parents]):
        raise ValueError("Symbolic artifact paths are not accepted")
    if not stat.S_ISREG(path.stat().st_mode):
        raise ValueError("Artifact must be a regular file")
    return path


def digest(path):
    checksum = hashlib.sha256()
    with regular_file(path).open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            checksum.update(chunk)
    return checksum.hexdigest()


def verify_debian(path, version, arch, output):
    path = regular_file(path)
    for field, expected in [("Package", PACKAGE), ("Version", version),
                            ("Architecture", arch)]:
        if output("dpkg-deb", "-f", str(path), field) != expected:
            raise ValueError("Debian package identity mismatch")
    return path


def verify_matrix(root, tag, output):
    if not re.fullmatch(r"v[0-9]+\.[0-9]+\.[0-9]+", tag):
        raise ValueError("Stable version tag required")
    root = Path(root).absolute()
    if any(p.is_symlink() for p in [root, *root.parents]) or not root.is_dir():
        raise ValueError("Matrix root must be a nonsymbolic directory")
    packages = []
    for version, arch in MATRIX:
        folder = root / f"deb-ubuntu-{version}-{arch}"
        name = f"{PACKAGE}_{tag[1:]}_ubuntu{version}_{arch}.deb"
        package, sums = folder / name, folder / "SHA256SUMS"
        if (folder.is_symlink() or not folder.is_dir()
                or {p.name for p in folder.iterdir()} != {name, "SHA256SUMS"}):
            raise ValueError("Unexpected or symbolic artifact contents")
        regular_file(sums)
        if sums.stat().st_size > 1024:
            raise ValueError("Oversized checksum metadata")
        match = re.fullmatch(r"([0-9a-f]{64})  " + re.escape(name) + r"\n?", sums.read_text())
        if not match or digest(package) != match[1]:
            raise ValueError("Artifact checksum mismatch or unsafe checksum filename")
        packages.append(verify_debian(package, tag[1:], arch, output))
    return packages
