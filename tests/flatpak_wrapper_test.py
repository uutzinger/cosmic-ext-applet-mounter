"""Candidate build/export fixtures: no real Flatpak, network or builds."""

import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time
import unittest

ROOT = Path(__file__).resolve().parents[1]
APP = "org.example.Candidate"

FAKE = r'''#!/usr/bin/env python3
import json,os,pathlib,sys,time
root=pathlib.Path(os.environ['FIXTURE_ROOT'])
cfg=json.loads((root/'config.json').read_text())
tool=pathlib.Path(sys.argv[0]).name
args=sys.argv[1:]
with (root/'calls').open('a') as log: log.write(json.dumps([tool,*args])+'\n')
if tool=='git':
 command=args[2:]
 if command[:2]==['rev-parse','--show-toplevel']: print(root/'source')
 elif command[0]=='rev-parse':
  target=command[-1].split('^',1)[0]
  print(cfg.get('local_source_sha',target) if len(target)==40 else cfg.get('sha','a'*40))
 elif command[0]=='status': print(' M dirty' if cfg.get('dirty') else '',end='')
 elif command[0]=='ls-files': sys.exit(19 if cfg.get('untracked') else 0)
 else: sys.exit(91)
elif args==['--version']: print(tool+' fixture-1')
elif tool=='ostree':
 repo=pathlib.Path(args[0].split('=',1)[1])
 if args[1]=='init': repo.mkdir()
 elif args[1]=='rev-parse': print(cfg.get('ostree_commit','d'*64))
 elif args[1]=='cat': print((repo/'metadata').read_text())
 elif args[1]=='fsck':
  if cfg.get('sleep_fsck'):
   (root/'pid').write_text(str(os.getpid()));time.sleep(60)
  if cfg.get('mutate_on_fsck'):
   target=root/cfg['mutate_on_fsck']
   with target.open('a') as stream: stream.write(' ')
  sys.exit(28 if cfg.get('fsck_failure') else 0)
 else: sys.exit(93)
elif tool=='flatpak' and args[0]=='override': print('')
elif tool=='flatpak' and args[0]=='info':
 if '--user' in args or '--system' in args: sys.exit(1)
 if cfg.get('missing_runtime'): sys.exit(24)
 print(cfg.get('runtime_ref',args[-1]) if args[1]=='--show-ref' else 'b'*64)
elif tool=='flatpak-builder':
 if os.environ.get('GIT_CONFIG_COUNT'):
  index=int(os.environ['GIT_CONFIG_COUNT'])-1
  (root/'transport-env.json').write_text(json.dumps({
   'key':os.environ.get('GIT_CONFIG_KEY_'+str(index)),
   'value':os.environ.get('GIT_CONFIG_VALUE_'+str(index))}))
 if cfg.get('sleep'):
  (root/'pid').write_text(str(os.getpid()));time.sleep(60)
 if cfg.get('build_failure'): sys.exit(25)
 print('private-builder-output'*1000)
 build=pathlib.Path(args[-2]);build.mkdir()
 (build/'metadata').write_text('[Application]\nname='+cfg.get('appid','org.example.Candidate')+'\nruntime=org.example.Platform/x86_64/1\nsdk=org.example.Sdk/x86_64/1\n[Context]\nshared=network;\n')
 if cfg.get('mutate'): (root/'source/input.txt').write_text('changed')
elif tool=='flatpak' and args[0]=='build-export':
 if cfg.get('export_failure'): sys.exit(26)
 pathlib.Path(args[-3]).mkdir()
 (pathlib.Path(args[-3])/'metadata').write_text((pathlib.Path(args[-2])/'metadata').read_text())
 if cfg.get('export_policy_mismatch'):
  with (pathlib.Path(args[-3])/'metadata').open('a') as stream: stream.write('[Environment]\nUNEXPECTED=synthetic\n')
elif tool=='flatpak' and args[0]=='build-bundle':
 if cfg.get('bundle_failure'): sys.exit(27)
 pathlib.Path(args[-3]).write_bytes(b'fixture-bundle-not-executable')
elif tool=='flatpak' and args[0]=='build-import-bundle':
 repo=pathlib.Path(args[-2]);repo.mkdir(exist_ok=True)
 (repo/'metadata').write_text((pathlib.Path(args[-1]).parent/'build/metadata').read_text())
else: sys.exit(92)
'''


class FlatpakTests(unittest.TestCase):
    def fixture(self, tmp, **config):
        root = Path(tmp)
        source = root / "source"
        source.mkdir()
        (source / "input.txt").write_text("immutable local input")
        manifest = {"id": APP, "runtime": "org.example.Platform", "sdk": "org.example.Sdk",
                    "runtime-version": "1", "base": "org.example.Base", "base-version": "1",
                    "finish-args": ["--share=network"],
                    "modules": [{"name": "fixture", "sources": [
                        {"type": "git", "url": "https://example.invalid/repo.git", "commit": "c" * 40},
                        {"type": "file", "path": "input.txt"}]}]}
        (source / "manifest.json").write_text(json.dumps(manifest))
        (root / "config.json").write_text(json.dumps(config))
        tools = root / "bin"
        tools.mkdir()
        executable = tools / "fake"
        executable.write_text(FAKE)
        executable.chmod(0o755)
        for tool in ["git", "flatpak", "flatpak-builder", "ostree"]:
            (tools / tool).symlink_to(executable)
        env = dict(os.environ, FIXTURE_ROOT=str(root), PATH=f"{tools}:{os.environ['PATH']}")
        args = ["bash", str(ROOT / "scripts/build-flatpak.sh"), "--manifest", str(source / "manifest.json"),
                "--source-sha", "a" * 40, "--arch", "x86_64", "--branch", "candidate",
                "--output-dir", str(root / "candidate"), "--log-dir", str(root / "logs")]
        return root, env, args, manifest

    def invoke(self, env, args):
        return subprocess.run(args, env=env, capture_output=True, text=True, timeout=15)

    def calls(self, root):
        return [json.loads(line) for line in (root / "calls").read_text().splitlines()]

    def test_preflight_records_dirty_unpinned_blockers_without_building(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, env, args, manifest = self.fixture(tmp, dirty=True)
            manifest["modules"][0]["sources"][0].pop("commit")
            manifest["modules"][0]["sources"][0]["tag"] = "mutable-tag"
            (root / "source/manifest.json").write_text(json.dumps(manifest))
            index = args.index("--output-dir")
            args[index:index + 2] = []
            result = self.invoke(env, [*args, "--preflight"])
            self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
            summary = json.loads(result.stdout.splitlines()[-1])
            receipt = json.loads(Path(summary["artifact_receipt_path"]).read_text())
            self.assertFalse(receipt["ready_for_candidate_build"])
            self.assertTrue(receipt["source_dirty"])
            self.assertEqual(len(receipt["blockers"]), 2)
            self.assertEqual(receipt["acceptance"], "not-run")
            self.assertEqual(receipt["installation"], "not-performed")
            self.assertFalse((root / "candidate").exists())
            calls = self.calls(root)
            self.assertTrue(all(c[0] in ["git", "flatpak", "flatpak-builder", "ostree"] for c in calls))
            self.assertTrue(all(c[1:] == ["--version"] for c in calls if c[0] in ["flatpak-builder", "ostree"]))
            self.assertTrue(all(c[1] in ["--version", "info", "override"] for c in calls if c[0] == "flatpak"))
            self.assertTrue(all("--show" in c for c in calls if c[0] == "flatpak" and c[1] == "override"))

    def test_preflight_clean_inputs_are_ready_without_an_installed_app(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, env, args, manifest = self.fixture(tmp)
            manifest["sdk-extensions"] = ["org.example.Rust"]
            (root / "source/manifest.json").write_text(json.dumps(manifest))
            index = args.index("--output-dir")
            args[index:index + 2] = []
            index = args.index("--source-sha")
            args[index:index + 2] = []
            result = self.invoke(env, [*args, "--preflight"])
            self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
            summary = json.loads(result.stdout.splitlines()[-1])
            receipt = json.loads(Path(summary["artifact_receipt_path"]).read_text())
            self.assertTrue(receipt["ready_for_candidate_build"])
            self.assertEqual(len(receipt["runtimes"]), 4)
            self.assertTrue(all(item["ref"] is None for item in receipt["installations"]))

    def test_generated_cargo_inline_and_shell_sources_are_fingerprinted(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, env, args, manifest = self.fixture(tmp)
            generated = [
                {"type": "inline", "contents": "synthetic vendor metadata", "dest-filename": "config", "dest": "cargo"},
                {"type": "shell", "commands": ["cp -r synthetic-git-package cargo/vendor/synthetic"]},
            ]
            (root / "source/vendor.json").write_text(json.dumps(generated))
            manifest["modules"][0]["sources"].append("vendor.json")
            (root / "source/manifest.json").write_text(json.dumps(manifest))
            result = self.invoke(env, args)
            self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
            receipt = json.loads((root / "logs/artifacts.json").read_text())
            import hashlib
            self.assertEqual(receipt["input_sha256"]["vendor.json"],
                             hashlib.sha256((root / "source/vendor.json").read_bytes()).hexdigest())

    def test_malformed_inline_and_shell_sources_prevent_build(self):
        for source in [{"type": "inline", "contents": 7, "dest-filename": "config"},
                       {"type": "inline", "contents": "text"},
                       {"type": "shell", "commands": "unstructured"},
                       {"type": "shell", "commands": [None]}]:
            with self.subTest(source=source), tempfile.TemporaryDirectory() as tmp:
                root, env, args, manifest = self.fixture(tmp)
                manifest["modules"][0]["sources"].append(source)
                (root / "source/manifest.json").write_text(json.dumps(manifest))
                result = self.invoke(env, args)
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(any(call[0] == "flatpak-builder" for call in self.calls(root)))

    def test_local_source_transport_preserves_pin_and_records_builder_only_environment(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, env, args, _ = self.fixture(tmp)
            local = root / "source"
            result = self.invoke(env, [*args, "--local-source-repo", str(local)])
            self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
            receipt = json.loads((root / "logs/artifacts.json").read_text())
            self.assertEqual(receipt["source_transport"], {"kind": "local-git", "repository": str(local),
                "source_url": "https://example.invalid/repo.git", "commit": "c" * 40})
            transport = json.loads((root / "transport-env.json").read_text())
            self.assertEqual(transport, {"key": f"url.{local.as_uri()}/.insteadOf", "value": "https://example.invalid/repo.git"})
            self.assertEqual(receipt["source_pins"][0]["commit"], "c" * 40)
            self.assertFalse(any("config" in call for call in self.calls(root) if call[0] == "git"))

    def test_local_source_transport_rejects_mismatched_commit_before_build(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, env, args, _ = self.fixture(tmp, local_source_sha="d" * 40)
            result = self.invoke(env, [*args, "--local-source-repo", str(root / "source")])
            self.assertNotEqual(result.returncode, 0)
            self.assertFalse(any(call[0] == "flatpak-builder" for call in self.calls(root)))
            self.assertFalse((root / "candidate").exists())

    def test_candidate_records_inputs_runtime_metadata_and_bundle(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, env, args, _ = self.fixture(tmp)
            result = self.invoke(env, [*args, "--offline"])
            self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
            summary = json.loads(result.stdout.splitlines()[-1])
            receipt = json.loads(Path(summary["artifact_receipt_path"]).read_text())
            self.assertEqual(receipt["kind"], "unaccepted-flatpak-candidate")
            self.assertEqual(receipt["runtimes"][2]["ref"], "app/org.example.Base/x86_64/1")
            self.assertEqual(receipt["application_id"], APP)
            self.assertEqual(len(receipt["artifacts"]), 2)
            self.assertEqual(len(receipt["runtimes"]), 3)
            self.assertEqual(receipt["source_pins"][0]["commit"], "c" * 40)
            self.assertIn("input.txt", receipt["input_sha256"])
            self.assertNotIn("private-builder-output", result.stdout)
            calls = self.calls(root)
            build = [c for c in calls if c[0] == "flatpak-builder" and "--version" not in c]
            self.assertEqual(len(build), 1)
            self.assertIn("--disable-download", build[0])
            for call in calls:
                self.assertFalse(any(flag in call for flag in ["--install", "--force-clean", "--install-deps-from", "run", "remote-add"]))

    def test_identity_missing_runtime_and_input_failures_prevent_build(self):
        for config in [{"dirty": True}, {"sha": "d" * 40}, {"untracked": True},
                       {"missing_runtime": True}, {"runtime_ref": "runtime/org.example.Wrong/x86_64/1"}]:
            with self.subTest(config=config), tempfile.TemporaryDirectory() as tmp:
                root, env, args, _ = self.fixture(tmp, **config)
                result = self.invoke(env, args)
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse((root / "candidate").exists())
                self.assertFalse(any(c[0] == "flatpak-builder" and "--version" not in c for c in self.calls(root)))

    def test_unpinned_directory_sources_branch_mismatch_and_escaping_paths_fail(self):
        for kind in ["git", "archive", "dir", "branch", "escape", "symlink", "include-cycle"]:
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as tmp:
                root, env, args, manifest = self.fixture(tmp)
                sources = manifest["modules"][0]["sources"]
                if kind == "git":
                    sources[0].pop("commit")
                    sources[0]["tag"] = "mutable"
                elif kind in ["archive", "dir"]:
                    sources[0] = {"type": kind, "url": "https://example.invalid/source"}
                elif kind == "branch":
                    manifest["branch"] = "other"
                elif kind == "escape":
                    (root / "outside").write_text("outside")
                    sources[1]["path"] = "../outside"
                elif kind == "symlink":
                    (root / "source/input.txt").unlink()
                    (root / "source/input.txt").symlink_to(root / "config.json")
                else:
                    manifest["modules"] = ["manifest.json"]
                (root / "source/manifest.json").write_text(json.dumps(manifest))
                result = self.invoke(env, args)
                self.assertEqual(result.returncode, 1)
                self.assertFalse((root / "candidate").exists())

    def test_failed_build_export_bundle_or_metadata_never_has_success_receipt(self):
        for config, code in [({"build_failure": True}, 25), ({"export_failure": True}, 26),
                             ({"bundle_failure": True}, 27), ({"appid": "org.example.Wrong"}, 1),
                             ({"mutate": True}, 1), ({"export_policy_mismatch": True}, 1)]:
            with self.subTest(config=config), tempfile.TemporaryDirectory() as tmp:
                root, env, args, _ = self.fixture(tmp, **config)
                result = self.invoke(env, args)
                self.assertEqual(result.returncode, code, result.stderr)
                report = json.loads((root / "logs/summary.json").read_text())
                self.assertEqual(report["status"], "failed")
                self.assertIsNone(report["artifact_receipt_path"])
                self.assertTrue((root / "candidate").is_dir())

    def test_existing_output_and_unsupported_arguments_never_rebuild(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, env, args, _ = self.fixture(tmp)
            (root / "candidate").mkdir()
            sentinel = root / "candidate/sentinel"
            sentinel.write_text("preserve")
            self.assertEqual(self.invoke(env, args).returncode, 1)
            self.assertEqual(sentinel.read_text(), "preserve")
            self.assertFalse(any(c[0] == "flatpak-builder" and "--version" not in c for c in self.calls(root)))
            for extra in ["--install", "--force-clean", "--reuse"]:
                self.assertEqual(self.invoke(env, [*args, extra]).returncode, 2)

    def test_external_json_includes_and_pinned_archive_are_fingerprinted(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, env, args, manifest = self.fixture(tmp)
            module = manifest["modules"][0]
            module["sources"].append({"type": "archive", "url": "https://example.invalid/archive", "sha256": "e" * 64})
            (root / "source/module.json").write_text(json.dumps(module))
            manifest["modules"] = ["module.json"]
            (root / "source/manifest.json").write_text(json.dumps(manifest))
            result = self.invoke(env, args)
            self.assertEqual(result.returncode, 0, result.stderr)
            receipt = json.loads((root / "logs/artifacts.json").read_text())
            self.assertIn("module.json", receipt["input_sha256"])
            self.assertEqual(receipt["source_pins"][1]["sha256"], "e" * 64)

    def test_timeout_retains_failed_report(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, env, args, _ = self.fixture(tmp, sleep=True)
            result = self.invoke(env, [*args, "--timeout-seconds", "1"])
            self.assertEqual(result.returncode, 1)
            self.assertEqual(json.loads((root / "logs/summary.json").read_text())["status"], "failed")
            with self.assertRaises(ProcessLookupError):
                os.kill(int((root / "pid").read_text()), 0)

    def test_cancellation_reaps_builder_and_retains_failure_report(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, env, args, _ = self.fixture(tmp, sleep=True)
            process = subprocess.Popen(args, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            try:
                deadline = time.monotonic() + 6
                while not (root / "pid").exists() and time.monotonic() < deadline:
                    time.sleep(0.02)
                self.assertTrue((root / "pid").exists())
                process.send_signal(signal.SIGTERM)
                process.communicate(timeout=8)
                self.assertEqual(process.returncode, 143)
                report = json.loads((root / "logs/summary.json").read_text())
                self.assertEqual(report["status"], "cancelled")
                with self.assertRaises(ProcessLookupError):
                    os.kill(int((root / "pid").read_text()), 0)
            finally:
                if process.poll() is None:
                    process.kill()
                    process.communicate()


if __name__ == "__main__":
    unittest.main()
