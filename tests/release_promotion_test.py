"""Offline promotion policy and package-fixture tests; no GitHub mutations."""

import copy
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("promotion", ROOT / "scripts/promote_release.py")
promotion = importlib.util.module_from_spec(spec)
spec.loader.exec_module(promotion)


class PromotionTests(unittest.TestCase):
    def setUp(self):
        self.sha = "a" * 40
        self.repo = "example/project"
        self.run = {"repository": {"full_name": self.repo},
                    "head_repository": {"full_name": self.repo},
                    "head_sha": self.sha, "path": ".github/workflows/build-deb.yml",
                    "event": "workflow_dispatch", "status": "completed", "conclusion": "success"}
        self.artifacts = [{"name": f"deb-ubuntu-{version}-{arch}", "expired": False}
                          for version, arch in promotion.MATRIX]

    def test_exact_successful_run_passes(self):
        promotion.validate_run(self.run, self.artifacts, self.repo, self.sha)

    def test_remote_lightweight_and_annotated_tag_identity(self):
        commit = json.dumps({"object": {"type": "commit", "sha": self.sha}})
        annotated = json.dumps({"object": {"type": "tag", "sha": "b" * 40}})
        for responses in [[commit], [annotated, commit]]:
            with patch.object(promotion, "output", side_effect=responses):
                promotion.verify_remote_tag(self.repo, "v0.5.4", self.sha)
        with patch.object(promotion, "output", return_value=commit), self.assertRaises(ValueError):
            promotion.verify_remote_tag(self.repo, "v0.5.4", "c" * 40)

    def test_inspection_and_approved_publication_reuse_downloaded_artifacts(self):
        def fake_output(*args):
            if args[:2] == ("git", "status"):
                return ""
            if args[0] == "git":
                return self.sha
            if args[:3] == ("gh", "api", f"repos/{self.repo}/git/ref/tags/v0.5.4"):
                return json.dumps({"object": {"type": "commit", "sha": self.sha}})
            if args[0] == "gh":
                return json.dumps({"total_count": 6, "artifacts": self.artifacts}
                                  if "artifacts?" in args[-1] else self.run)
            return self.identity(*args)

        for publish in [False, True]:
            commands = []

            def fake_run(args, **kwargs):
                commands.append(args)
                if args[:3] == ["gh", "run", "download"]:
                    folder = Path(args[-1])
                    folder.mkdir()
                    version_arch = folder.name.removeprefix("deb-ubuntu-").replace("-", "_")
                    package = folder / f"cosmic-ext-applet-mounter_0.5.4_ubuntu{version_arch}.deb"
                    package.write_bytes(b"fixture")
                    (folder / "SHA256SUMS").write_text(
                        f"{hashlib.sha256(package.read_bytes()).hexdigest()}  {package.name}\n")
                if args[:3] == ["gh", "release", "create"]:
                    self.assertEqual(len([p for p in args if p.endswith(".deb")]), 6)
                    sums = next(Path(p) for p in args if p.endswith("/SHA256SUMS"))
                    self.assertEqual(len(sums.read_text().splitlines()), 6)
                return subprocess.CompletedProcess(args, 0)

            argv = ["promote_release.py", "v0.5.4", "123", "--repo", self.repo]
            if publish:
                argv += ["--publish", "--accepted"]
            with self.subTest(publish=publish), patch("sys.argv", argv), \
                    patch.object(promotion, "output", side_effect=fake_output), \
                    patch.object(promotion.subprocess, "run", side_effect=fake_run):
                promotion.main()
            self.assertEqual(len([c for c in commands if c[:3] == ["gh", "run", "download"]]), 6)
            self.assertEqual(len([c for c in commands if c[:3] == ["gh", "release", "create"]]), int(publish))
            self.assertFalse(any("workflow" in c or "cargo" in c or "dpkg-buildpackage" in c for c in commands))

    def test_untrusted_or_unfinished_runs_fail(self):
        for key, value in [("head_sha", "b" * 40), ("path", ".github/workflows/other.yml"),
                           ("status", "in_progress"), ("conclusion", "failure"),
                           ("event", "pull_request"), ("repository", {"full_name": "other/project"}),
                           ("head_repository", {"full_name": "fork/project"})]:
            with self.subTest(key=key):
                run = copy.deepcopy(self.run)
                run[key] = value
                with self.assertRaises(ValueError):
                    promotion.validate_run(run, self.artifacts, self.repo, self.sha)

    def test_missing_duplicate_extra_expired_artifacts_fail(self):
        expired = copy.deepcopy(self.artifacts)
        expired[0]["expired"] = True
        for artifacts in [self.artifacts[:-1], self.artifacts + [self.artifacts[0]],
                          [self.artifacts[0]] * 6, expired]:
            with self.subTest(artifacts=artifacts), self.assertRaises(ValueError):
                promotion.validate_run(self.run, artifacts, self.repo, self.sha)

    def fixtures(self, root):
        for version, arch in promotion.MATRIX:
            folder = root / f"deb-ubuntu-{version}-{arch}"
            folder.mkdir()
            p = folder / f"cosmic-ext-applet-mounter_0.5.4_ubuntu{version}_{arch}.deb"
            p.write_bytes(b"synthetic package fixture")
            (folder / "SHA256SUMS").write_text(f"{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.name}\n")

    def identity(self, *args):
        return {"Package": "cosmic-ext-applet-mounter", "Version": "0.5.4",
                "Architecture": "arm64" if "arm64" in args[-2] else "amd64"}[args[-1]]

    def test_checksums_and_identities_pass_without_build(self):
        with tempfile.TemporaryDirectory() as tmp, patch.object(promotion, "output", self.identity):
            root = Path(tmp)
            self.fixtures(root)
            self.assertEqual(len(promotion.verify_packages(root, "v0.5.4")), 6)

    def test_checksum_tamper_extra_file_and_symlink_fail(self):
        for kind in ["tamper", "extra", "symlink", "unsafe-sum"]:
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                self.fixtures(root)
                folder = root / "deb-ubuntu-22.04-amd64"
                package = next(folder.glob("*.deb"))
                if kind == "tamper":
                    package.write_bytes(b"tampered")
                elif kind == "extra":
                    (folder / "unexpected").touch()
                elif kind == "symlink":
                    package.unlink()
                    package.symlink_to("/dev/null")
                else:
                    (folder / "SHA256SUMS").write_text("0" * 64 + "  ../../outside\n")
                with patch.object(promotion, "output", self.identity), self.assertRaises(ValueError):
                    promotion.verify_packages(root, "v0.5.4")

    def test_wrong_package_identity_fails(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.fixtures(root)
            with patch.object(promotion, "output", return_value="wrong"), self.assertRaises(ValueError):
                promotion.verify_packages(root, "v0.5.4")

    def test_unapproved_publish_and_invalid_inputs_fail_before_network(self):
        for args in [["v0.5.4", "123", "--publish"], ["../v0.5.4", "123"],
                     ["v0.5.4", "-1"], ["v0.5.4", "123", "--publish", "--accepted", "--source-sha", self.sha]]:
            with self.subTest(args=args):
                result = subprocess.run(["python3", str(ROOT / "scripts/promote_release.py"), *args],
                                        capture_output=True, text=True)
                self.assertEqual(result.returncode, 2, result.stderr)

    def test_workflows_separate_build_and_promotion(self):
        build = (ROOT / ".github/workflows/build-deb.yml").read_text()
        publish = (ROOT / ".github/workflows/publish-release.yml").read_text()
        script = (ROOT / "scripts/build_release.sh").read_text()
        self.assertNotIn("  push:", build)
        self.assertNotIn("gh release", build)
        self.assertNotIn("publish_after_acceptance", build)
        for obsolete in ["dpkg-buildpackage", "cargo build", "needs: build"]:
            self.assertNotIn(obsolete, publish)
        self.assertIn("--publish --accepted", publish)
        self.assertIn("build_run_id:", publish)
        self.assertIn("actions: read", publish)
        self.assertIn("--run-id", script)
        self.assertNotIn("--wait-for-existing", script)

    def test_local_release_builds_once_or_reuses_run_without_dispatch(self):
        for reuse in [False, True]:
            with self.subTest(reuse=reuse), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                (root / "scripts").mkdir()
                for name in ["build_release.sh", "run_github_linux_build.sh", "promote_release.py"]:
                    (root / "scripts" / name).write_bytes((ROOT / "scripts" / name).read_bytes())
                fake = root / "fake"
                fake.write_text('''#!/usr/bin/env python3
import json, os, pathlib, sys
name=pathlib.Path(sys.argv[0]).name
args=sys.argv[1:]
with open(os.environ["COMMAND_LOG"], "a") as log:
    log.write(json.dumps([name,*args])+"\\n")
if name=="cargo":
    print(json.dumps({"packages":[{"name":"cosmic-ext-applet-mounter","version":"0.5.4"}]}))
elif name=="dpkg-parsechangelog":
    print("0.5.4")
elif name=="git":
    if args[0]=="branch": print("master")
    elif args[0]=="ls-remote": sys.exit(2)
    elif args[0]=="rev-parse":
        if "--verify" in args and any(a.startswith("refs/tags/") for a in args): sys.exit(1)
        print("a"*40)
elif name=="gh":
    if args[:2]==["repo","view"]: print("example/project")
    elif args[0]=="api":
        if "artifacts?" in args[-1]:
            print(json.dumps({"total_count":6,"artifacts":[{"name":f"deb-ubuntu-{v}-{a}","expired":False} for v in ["22.04","24.04","26.04"] for a in ["amd64","arm64"]]}))
        else:
            print(json.dumps({"repository":{"full_name":"example/project"},"head_repository":{"full_name":"example/project"},"head_sha":"a"*40,"path":".github/workflows/build-deb.yml","event":"workflow_dispatch","status":"completed","conclusion":"success"}))
    elif args[:2]==["run","list"]: print("123")
''')
                fake.chmod(0o755)
                for name in ["cargo", "git", "gh", "dpkg-parsechangelog"]:
                    (root / name).symlink_to(fake)
                env = os.environ.copy()
                for key in ["DOWNLOAD_DIR", "WORKFLOW", "REF"]:
                    env.pop(key, None)
                log = root / "commands.jsonl"
                env.update(PATH=f"{root}:{env['PATH']}", COMMAND_LOG=str(log))
                args = ["bash", str(root / "scripts/build_release.sh")]
                if reuse:
                    args += ["--run-id", "123"]
                result = subprocess.run(args, env=env, capture_output=True, text=True)
                self.assertEqual(result.returncode, 0, result.stderr)
                commands = [json.loads(line) for line in log.read_text().splitlines()]
                dispatches = [c for c in commands if c[:3] == ["gh", "workflow", "run"]]
                self.assertEqual(len(dispatches), 0 if reuse else 1)
                self.assertFalse(any(c[:3] == ["gh", "release", "create"] for c in commands))


if __name__ == "__main__":
    unittest.main()
