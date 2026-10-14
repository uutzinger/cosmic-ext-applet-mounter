"""OneDrive guard check against a fake rclone; never touches a real drive."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
TOOL = ROOT / "scripts/lib/mirror_guard_check.py"

FAKE = r'''#!/usr/bin/env python3
import hashlib, json, os, pathlib, shutil, sys
root = pathlib.Path(os.environ["FAKE_ROOT"])
args = sys.argv[1:]
with (root / "calls").open("a") as log:
    log.write(json.dumps(args) + "\n")

def remote_dir(spec):
    name, _, path = spec.partition(":")
    return root / "remotes" / name / path

def files(base):
    return {p.relative_to(base).as_posix(): p for p in sorted(base.rglob("*")) if p.is_file()} if base.exists() else {}

cmd = args[0]
if cmd == "listremotes":
    for item in os.environ.get("FAKE_REMOTES", "").split(","):
        name, _, kind = item.partition("=")
        print(f"{name}: {kind}")
elif cmd == "lsjson":
    base = remote_dir(args[-1])
    entries = [{"Path": n, "Size": p.stat().st_size, "ModTime": str(p.stat().st_mtime_ns),
                "Hashes": {"sha1": hashlib.sha1(p.read_bytes()).hexdigest()}} for n, p in files(base).items()]
    print(json.dumps(entries))
elif cmd == "bisync":
    local, remote = pathlib.Path(args[1]), remote_dir(args[2])
    remote.mkdir(parents=True, exist_ok=True)
    flags = args[3:]
    lf, rf = files(local), files(remote)
    for name in sorted(set(lf) | set(rf)):
        l, r = lf.get(name), rf.get(name)
        if l and not r:
            target = remote / name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(l, target)
            if os.environ.get("FAKE_REWRITE") and name.endswith((".docx", ".xlsx")):
                target.write_bytes(target.read_bytes() + b"R")
        elif r and not l:
            (local / name).parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(r, local / name)
        elif l.read_bytes() != r.read_bytes() and "--ignore-size" not in flags:
            shutil.copy2(r, l)
elif cmd == "check":
    local, remote = pathlib.Path(args[1]), remote_dir(args[2])
    lf, rf = files(local), files(remote)
    bad = False
    for name in sorted(set(lf) | set(rf)):
        if name not in rf: print(f"+ {name}"); bad = True
        elif name not in lf: print(f"- {name}"); bad = True
        elif lf[name].read_bytes() != rf[name].read_bytes(): print(f"* {name}"); bad = True
        else: print(f"= {name}")
    sys.exit(1 if bad else 0)
elif cmd == "lsf":
    base = remote_dir(args[1])
    if not base.exists(): sys.exit(3)
    for name in files(base):
        if not (os.environ.get("FAKE_DROP_MARKER") and name.startswith(".cloud-mounter-guard-test-")):
            print(name)
elif cmd == "purge":
    shutil.rmtree(remote_dir(args[1]))
else:
    sys.exit(94)
'''


class GuardCheck(unittest.TestCase):
    def run_tool(self, tmp, *extra, remotes="od=onedrive", **env):
        root = Path(tmp)
        bin_dir = root / "bin"
        bin_dir.mkdir(exist_ok=True)
        fake = bin_dir / "rclone"
        fake.write_text(FAKE)
        fake.chmod(0o755)
        environ = dict(os.environ, PATH=f"{bin_dir}:{os.environ['PATH']}", FAKE_ROOT=str(root),
                       FAKE_REMOTES=remotes, GUARD_CHECK_ROOT=str(root), **env)
        for key in ["FAKE_REWRITE", "FAKE_DROP_MARKER"]:
            if key not in env:
                environ.pop(key, None)
        result = subprocess.run([sys.executable, str(TOOL), "--drive", "personal",
                                 "--log-dir", str(root / f"log{len(list(root.glob('log*')))}"), *extra],
                                cwd=root, env=environ, capture_output=True, text=True)
        calls = [json.loads(line) for line in (root / "calls").read_text().splitlines()] if (root / "calls").exists() else []
        logs = sorted(root.glob("log*/results.json"))
        return result, calls, (json.loads(logs[-1].read_text()) if logs else None)

    def test_stable_drive_needs_only_generic_profile_and_is_purged(self):
        with tempfile.TemporaryDirectory() as tmp:
            result, calls, summary = self.run_tool(tmp, "--remote", "od")
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual([p["profile"] for p in summary["profiles"]], ["generic"])
            self.assertEqual(summary["verdict"], "generic settings are sufficient for the stages run")
            self.assertEqual(summary["profiles"][0]["cleanup"], "purged")
            purges = [c for c in calls if c[0] == "purge"]
            self.assertEqual(len(purges), 1)
            self.assertTrue(purges[0][1].startswith("od:CloudMounter-GuardTest-"))
            self.assertTrue(summary["profiles"][0]["synthetic_office"])

    def test_rewriting_drive_is_detected_and_compat_profile_confirmed(self):
        with tempfile.TemporaryDirectory() as tmp:
            result, calls, summary = self.run_tool(tmp, "--remote", "od", FAKE_REWRITE="1")
            self.assertEqual(result.returncode, 0, result.stderr)
            generic, compat = summary["profiles"]
            self.assertFalse(generic["clean"])
            self.assertFalse(generic["stages"]["stable_without_edits"])
            self.assertTrue(compat["clean"])
            self.assertEqual(summary["verdict"], "needs the compatibility profile (Office-rewrite handling)")
            self.assertEqual(len([c for c in calls if c[0] == "purge"]), 2)

    def test_unknown_or_wrong_type_remote_makes_no_cloud_changes(self):
        for remotes in ["od=drive", "other=onedrive"]:
            with self.subTest(remotes=remotes), tempfile.TemporaryDirectory() as tmp:
                result, calls, summary = self.run_tool(tmp, "--remote", "od", remotes=remotes)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual({c[0] for c in calls}, {"listremotes"})
        with tempfile.TemporaryDirectory() as tmp:
            result, calls, _ = self.run_tool(tmp, "--remote", "od:evil")
            self.assertEqual(result.returncode, 2)
            self.assertEqual(calls, [])

    def test_purge_is_refused_when_the_marker_cannot_be_confirmed(self):
        with tempfile.TemporaryDirectory() as tmp:
            result, calls, summary = self.run_tool(tmp, "--remote", "od", FAKE_DROP_MARKER="1")
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue(summary["profiles"][0]["cleanup"].startswith("skipped"))
            self.assertFalse([c for c in calls if c[0] == "purge"])

    def test_plan_mode_only_lists_remotes(self):
        with tempfile.TemporaryDirectory() as tmp:
            result, calls, _ = self.run_tool(tmp, "--remote", "od", "--plan")
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual({c[0] for c in calls}, {"listremotes"})
            self.assertIn("plan:", result.stderr)


if __name__ == "__main__":
    unittest.main()
