#!/usr/bin/env python3
"""GitHub bundle installer fixtures; no real network or Flatpak changes."""
import json
import hashlib
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / 'scripts/install-flatpak-release.sh'


class FlatpakReleaseInstallerTests(unittest.TestCase):
    def flow(self, arch='x86_64', failure='', arguments=(), local_bundle=False):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            adapter = root / 'adapter'
            adapter.write_text('''#!/usr/bin/env python3
import hashlib, json, os, pathlib, sys
name, args = pathlib.Path(sys.argv[0]).name, sys.argv[1:]
with open(os.environ['CALLS'], 'a') as log:
    log.write(json.dumps([name, *args]) + '\\n')
failure = os.environ['FAILURE']
if name == 'uname':
    print(os.environ['ARCH'])
elif name == 'curl':
    if failure == 'download': sys.exit(22)
    output = pathlib.Path(args[args.index('--output') + 1])
    arch = {'amd64': 'x86_64', 'arm64': 'aarch64'}.get(os.environ['ARCH'], os.environ['ARCH'])
    filename = 'io.github.uutzinger.cosmic-ext-applet-mounter-' + arch + '-master.flatpak'
    digest = hashlib.sha256(b'fixture-bundle').hexdigest()
    if failure == 'checksum': digest = '0' * 64
    if output.name == 'SHA256SUMS':
        if failure == 'missing': filename = 'other.flatpak'
        line = digest + '  ' + filename + '\\n'
        output.write_text(line * (2 if failure == 'duplicate' else 1))
    else:
        output.write_bytes(b'fixture-bundle')
elif name == 'flatpak':
    stage = 'remote' if args[0] == 'remote-add' else ('bundle' if '--bundle' in args else 'runtime')
    if failure == stage: sys.exit(42)
''')
            adapter.chmod(0o755)
            for name in ('curl', 'uname', 'flatpak'):
                (root / name).symlink_to(adapter)
            env = dict(os.environ, PATH=f'{root}:{os.environ["PATH"]}',
                       CALLS=str(root / 'calls'), FAILURE=failure, ARCH=arch)
            if local_bundle:
                selected = {'amd64': 'x86_64', 'arm64': 'aarch64'}.get(arch, arch)
                if failure == 'wrong-cpu': selected = 'aarch64'
                bundle = root / f'io.github.uutzinger.cosmic-ext-applet-mounter-{selected}-master.flatpak'
                bundle.write_bytes(b'local-fixture-bundle')
                digest = hashlib.sha256(bundle.read_bytes()).hexdigest()
                if failure == 'checksum': digest = '0' * 64
                (root / 'SHA256SUMS').write_text(f'{digest}  {bundle.name}\n')
                arguments = (*arguments, '--bundle', str(bundle))
            # Root CI can exercise the same fake adapters without installing
            # into root's real Flatpak; production enforces desktop-user mode.
            command = ['bash', '-c', 'source "$1"; shift; require_desktop_user() { :; }; main "$@"',
                       'fixture', str(SCRIPT), *arguments]
            result = subprocess.run(command, env=env, capture_output=True, text=True, timeout=10)
            calls = [json.loads(line) for line in (root / 'calls').read_text().splitlines()] if (root / 'calls').exists() else []
            return result, calls

    def test_architecture_selection_and_cleanup(self):
        for arch, expected in (('x86_64', 'x86_64'), ('amd64', 'x86_64'),
                               ('aarch64', 'aarch64'), ('arm64', 'aarch64')):
            with self.subTest(arch=arch):
                result, calls = self.flow(arch)
                self.assertEqual(result.returncode, 0, result.stderr)
                flatpak = [c for c in calls if c[0] == 'flatpak']
                self.assertEqual(len(flatpak), 3)
                self.assertTrue(all('--user' in c for c in flatpak))
                self.assertIn('--arch=' + expected, flatpak[1])
                self.assertIn('org.freedesktop.Platform//25.08', flatpak[1])
                self.assertIn('--or-update', flatpak[2])
                self.assertTrue(flatpak[2][-1].endswith(f'-{expected}-master.flatpak'))
                self.assertFalse(Path(flatpak[2][-1]).exists())

    def test_unsupported_architecture_is_read_only(self):
        result, calls = self.flow('riscv64')
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(any(c[0] in ('curl', 'flatpak') for c in calls))

    def test_bad_or_missing_assets_never_change_flatpak(self):
        for failure in ('download', 'checksum', 'missing', 'duplicate'):
            with self.subTest(failure=failure):
                result, calls = self.flow(failure=failure)
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(any(c[0] == 'flatpak' for c in calls))

    def test_partial_failures_stop_without_success_or_rollback(self):
        for stage, count in (('remote', 1), ('runtime', 2), ('bundle', 3)):
            result, calls = self.flow(failure=stage)
            self.assertEqual(result.returncode, 42)
            self.assertEqual(len([c for c in calls if c[0] == 'flatpak']), count)
            self.assertNotIn('Installed. Add', result.stdout)
            self.assertIn('"exit_code":42', result.stdout)

    def test_explicit_version_and_stdin_arguments(self):
        result, calls = self.flow(arguments=('--version', 'v0.5.5'))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(all('/v0.5.5/' in c[-1] for c in calls if c[0] == 'curl'))
        for args, code in ((['--help'], 0), (['--version', '../bad'], 1), (['--version'], 1)):
            result = subprocess.run(['bash', '-s', '--', *args], input=SCRIPT.read_text(),
                                    capture_output=True, text=True, timeout=10)
            self.assertEqual(result.returncode, code, result.stderr)

    def test_local_candidate_uses_same_verified_install_path_without_release_download(self):
        result, calls = self.flow(local_bundle=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse(any(c[0] == 'curl' for c in calls))
        self.assertEqual(len([c for c in calls if c[0] == 'flatpak']), 3)
        for failure in ('checksum', 'wrong-cpu'):
            result, calls = self.flow(local_bundle=True, failure=failure)
            self.assertNotEqual(result.returncode, 0)
            self.assertFalse(any(c[0] in ('curl', 'flatpak') for c in calls))


if __name__ == '__main__':
    unittest.main()
