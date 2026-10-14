"""Local Flatpak ref/metadata comparison; never install or execute an app."""

import configparser
import hashlib
import json
from pathlib import Path
import re
import tempfile

from artifact_inspection import digest, regular_file


def parse_metadata(text):
    if len(text.encode()) > 1024 * 1024:
        raise ValueError("Flatpak metadata exceeds the size budget")
    parser = configparser.ConfigParser(interpolation=None, delimiters=("=",),
                                       comment_prefixes=("#",), strict=True)
    parser.optionxform = str
    # GKeyFile is not Python INI: do not accept inherited DEFAULT values or
    # multiline continuations that could hide a policy difference.
    for line in text.splitlines():
        if line[:1].isspace() and line.strip() and not line.lstrip().startswith("#"):
            raise ValueError("Multiline metadata values are not supported")
    try:
        parser.read_string(text)
    except configparser.Error as error:
        raise ValueError("Malformed or duplicate Flatpak metadata") from error
    if parser.defaults() or "[DEFAULT]" in text:
        raise ValueError("Inherited metadata defaults are not supported")
    data = {section: dict(parser.items(section, raw=True)) for section in parser.sections()}
    if "Application" not in data:
        raise ValueError("Application metadata required")
    return data


def validate_identity(appid, arch, branch, commit=None):
    if (not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_-]*(?:\.[A-Za-z_][A-Za-z0-9_-]*){2,}", appid)
            or arch not in ("x86_64", "aarch64")
            or not re.fullmatch(r"[A-Za-z0-9_][A-Za-z0-9_.-]*", branch)
            or (commit is not None and not re.fullmatch(r"[0-9a-f]{64}", commit))):
        raise ValueError("Unsafe Flatpak ref or commit identity")


def expected_policy(text, appid, arch):
    metadata = parse_metadata(text)
    application = metadata["Application"]
    if application.get("name") != appid:
        raise ValueError("Expected metadata application identity differs")
    for key in ("runtime", "sdk"):
        value = application.get(key, "")
        parts = value.split("/")
        if len(parts) != 3 or parts[1] != arch or not all(re.fullmatch(r"[A-Za-z0-9_.-]+", p) for p in parts):
            raise ValueError("Explicit matching runtime/SDK architecture required")
    return metadata


def repository_directory(path):
    root = Path(path).absolute()
    if any(p.is_symlink() for p in [root, *root.parents]) or not root.is_dir():
        raise ValueError("Local repository must be a nonsymbolic directory")
    return root.resolve()


def inspect_repository(path, appid, arch, branch, expected_text, output, expected_commit=None):
    validate_identity(appid, arch, branch, expected_commit)
    expected = expected_policy(expected_text, appid, arch)
    root = repository_directory(path)
    ref = f"app/{appid}/{arch}/{branch}"
    commit = output("ostree", f"--repo={root}", "rev-parse", ref)
    if not re.fullmatch(r"[0-9a-f]{64}", commit) or (expected_commit and commit != expected_commit):
        raise ValueError("Exported commit does not match expected identity")
    actual = parse_metadata(output("ostree", f"--repo={root}", "cat", commit, "/metadata"))
    if actual != expected:
        raise ValueError("Exported metadata/permissions differ from the selected comparison baseline")
    if output("ostree", f"--repo={root}", "rev-parse", ref) != commit:
        raise ValueError("Repository ref changed during inspection")
    canonical = json.dumps(actual, sort_keys=True, separators=(",", ":")).encode()
    return {"ref": ref, "commit": commit, "metadata": actual,
            "metadata_canonical_sha256": hashlib.sha256(canonical).hexdigest()}


def inspect_bundle(path, appid, arch, branch, expected_text, output, scratch_parent,
                   expected_commit=None):
    validate_identity(appid, arch, branch, expected_commit)
    expected_policy(expected_text, appid, arch)
    bundle = regular_file(path).resolve()
    before = digest(bundle)
    with tempfile.TemporaryDirectory(prefix="bundle-inspection-", dir=scratch_parent) as temporary:
        repo = Path(temporary) / "repo"
        output("ostree", f"--repo={repo}", "init", "--mode=archive")
        output("flatpak", "build-import-bundle", "--no-update-summary", str(repo), str(bundle))
        observed = inspect_repository(repo, appid, arch, branch, expected_text, output, expected_commit)
    if digest(bundle) != before:
        raise ValueError("Bundle changed during inspection")
    observed["bundle_sha256"] = before
    return observed
