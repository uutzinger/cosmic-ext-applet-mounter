"""Offline Flatpak metadata fixtures; no actual install, build or network."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts/lib"))
from flatpak_inspection import parse_metadata

APP = "org.example.Candidate"
METADATA = """[Application]
name=org.example.Candidate
runtime=org.example.Platform/x86_64/1
sdk=org.example.Sdk/x86_64/1
command=candidate
[Context]
shared=network;
filesystems=xdg-documents:ro;
[Session Bus Policy]
org.example.Service=talk
[Environment]
FIXTURE=synthetic
"""

FAKE = r'''#!/usr/bin/env python3
import json,os,pathlib,sys
root=pathlib.Path(os.environ['FIXTURE_ROOT'])
cfg=json.loads((root/'config.json').read_text())
tool=pathlib.Path(sys.argv[0]).name
args=sys.argv[1:]
with (root/'calls').open('a') as log: log.write(json.dumps([tool,*args])+'\n')
if tool=='ostree':
 repo=pathlib.Path(args[0].split('=',1)[1])
 if args[1]=='init': repo.mkdir()
 elif args[1]=='rev-parse':
  if args[-1]!=cfg.get('ref','app/org.example.Candidate/x86_64/candidate'): sys.exit(21)
  cfg['reads']=cfg.get('reads',0)+1
  (root/'config.json').write_text(json.dumps(cfg))
  print('e'*64 if cfg.get('drift') and cfg['reads']>1 else 'd'*64)
 elif args[1]=='cat':
  if cfg.get('missing_metadata'): sys.exit(22)
  print((repo/'metadata').read_text())
  if cfg.get('baseline_mutation'): (root/'expected.ini').write_text('changed')
 else: sys.exit(91)
elif tool=='flatpak' and args[:2]==['build-import-bundle','--no-update-summary']:
 if cfg.get('import_failure'): sys.exit(23)
 repo=pathlib.Path(args[-2])
 (repo/'metadata').write_text((root/'repo/metadata').read_text())
 if cfg.get('bundle_mutation'): pathlib.Path(args[-1]).write_bytes(b'changed')
else: sys.exit(92)
'''


class FlatpakInspectionTests(unittest.TestCase):
    def fixture(self, tmp, mode="flatpak-repo", **cfg):
        root = Path(tmp)
        repo = root / "repo"
        repo.mkdir()
        (repo / "metadata").write_text(METADATA)
        (root / "expected.ini").write_text(METADATA)
        bundle = root / "candidate.flatpak"
        bundle.write_bytes(b"synthetic bundle")
        tools = root / "bin"
        tools.mkdir()
        fake = tools / "fake"
        fake.write_text(FAKE)
        fake.chmod(0o755)
        for tool in ["ostree", "flatpak"]:
            (tools / tool).symlink_to(fake)
        (root / "config.json").write_text(json.dumps(cfg))
        env = dict(os.environ, FIXTURE_ROOT=str(root), PATH=f"{tools}:{os.environ['PATH']}")
        args = [sys.executable, str(ROOT / "scripts/inspect-artifacts.py"), mode,
                str(bundle if mode == "flatpak-bundle" else repo), "--app-id", APP,
                "--flatpak-arch", "x86_64", "--branch", "candidate",
                "--expected-metadata", str(root / "expected.ini"), "--log-dir", str(root / "logs")]
        return root, env, args

    def invoke(self, env, args):
        return subprocess.run(args, env=env, capture_output=True, text=True, timeout=10)

    def test_repository_identity_and_all_policy_sections_recorded(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, env, args = self.fixture(tmp)
            result = self.invoke(env, [*args, "--expected-commit", "d" * 64])
            self.assertEqual(result.returncode, 0, result.stderr)
            receipt = json.loads((root / "logs/artifacts.json").read_text())
            self.assertIsNone(receipt["source_sha"])
            self.assertEqual(receipt["inspection"]["commit"], "d" * 64)
            self.assertEqual(receipt["inspection"]["metadata"]["Session Bus Policy"]["org.example.Service"], "talk")
            self.assertEqual(len(receipt["inspection"]["metadata_canonical_sha256"]), 64)
            self.assertNotIn("FIXTURE", result.stdout)
            self.assertNotIn("flatpak", (root / "calls").read_text())

    def test_bundle_import_is_private_ephemeral_and_no_install_or_ref_override(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, env, args = self.fixture(tmp, "flatpak-bundle")
            result = self.invoke(env, args)
            self.assertEqual(result.returncode, 0, result.stderr)
            receipt = json.loads((root / "logs/artifacts.json").read_text())
            self.assertIn("bundle_sha256", receipt["inspection"])
            self.assertFalse(list((root / "logs").glob("bundle-inspection-*")))
            calls = [json.loads(line) for line in (root / "calls").read_text().splitlines()]
            imports = [c for c in calls if c[0] == "flatpak"]
            self.assertEqual(len(imports), 1)
            self.assertEqual(imports[0][1:3], ["build-import-bundle", "--no-update-summary"])
            self.assertTrue(Path(imports[0][-2]).is_relative_to(root / "logs"))
            self.assertFalse(any("--ref=" in arg for arg in imports[0]))

    def test_metadata_identity_and_permission_differences_fail(self):
        variants = [METADATA.replace(APP, "org.example.Wrong"),
                    METADATA.replace("/x86_64/1", "/aarch64/1"),
                    METADATA.replace("Platform/x86_64/1", "Platform/x86_64/2"),
                    METADATA.replace("xdg-documents:ro;", "host;"),
                    METADATA.replace("=talk", "=own"),
                    METADATA.replace("FIXTURE=synthetic", "FIXTURE=changed"),
                    METADATA + "[System Bus Policy]\norg.example.Service=talk\n"]
        for metadata in variants:
            with self.subTest(metadata=metadata), tempfile.TemporaryDirectory() as tmp:
                root, env, args = self.fixture(tmp)
                (root / "repo/metadata").write_text(metadata)
                self.assertEqual(self.invoke(env, args).returncode, 1)
                self.assertFalse((root / "logs/artifacts.json").exists())

    def test_keyfile_duplicate_defaults_and_continuations_rejected(self):
        for text in [METADATA.replace("shared=network;", "shared=network;\nshared=ipc;"),
                     "[DEFAULT]\nname=x\n" + METADATA,
                     METADATA + " continuation=x\n", "malformed"]:
            with self.subTest(text=text), self.assertRaises(ValueError):
                parse_metadata(text)

    def test_wrong_commit_ref_drift_and_missing_metadata_fail(self):
        cases = [({"ref": "app/org.example.Other/x86_64/candidate"}, 21, []),
                 ({"drift": True}, 1, []), ({"missing_metadata": True}, 22, []),
                 ({}, 1, ["--expected-commit", "e" * 64]),
                 ({"baseline_mutation": True}, 1, [])]
        for cfg, code, flags in cases:
            with self.subTest(cfg=cfg, flags=flags), tempfile.TemporaryDirectory() as tmp:
                root, env, args = self.fixture(tmp, **cfg)
                result = self.invoke(env, [*args, *flags])
                self.assertEqual(result.returncode, code, result.stderr)
                self.assertEqual(json.loads((root / "logs/summary.json").read_text())["status"], "failed")

    def test_bundle_import_failure_and_input_mutation_fail(self):
        for cfg, code in [({"import_failure": True}, 23), ({"bundle_mutation": True}, 1),
                          ({"ref": "app/org.example.Other/x86_64/candidate"}, 21)]:
            with self.subTest(cfg=cfg), tempfile.TemporaryDirectory() as tmp:
                root, env, args = self.fixture(tmp, "flatpak-bundle", **cfg)
                self.assertEqual(self.invoke(env, args).returncode, code)
                self.assertFalse(list((root / "logs").glob("bundle-inspection-*")))
                self.assertFalse((root / "logs/artifacts.json").exists())

    def test_symbolic_baseline_and_repository_rejected(self):
        for kind in ["baseline", "repo"]:
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as tmp:
                root, env, args = self.fixture(tmp)
                path = root / ("expected.ini" if kind == "baseline" else "repo")
                saved = root / "saved"
                path.rename(saved)
                path.symlink_to(saved)
                self.assertEqual(self.invoke(env, args).returncode, 1)

    def test_missing_identity_and_cross_mode_flags_fail_before_tools(self):
        result = subprocess.run([sys.executable, str(ROOT / "scripts/inspect-artifacts.py"),
                                 "flatpak-repo", "/unused"], capture_output=True, text=True)
        self.assertEqual(result.returncode, 2)
        result = subprocess.run([sys.executable, str(ROOT / "scripts/inspect-artifacts.py"),
                                 "native", "/unused", "--app-id", APP], capture_output=True, text=True)
        self.assertEqual(result.returncode, 2)


if __name__ == "__main__":
    unittest.main()
