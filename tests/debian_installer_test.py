#!/usr/bin/env python3
"""Standalone installer tests: fake network, OS metadata and package commands."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / 'scripts/install-debian.sh'


class DebianInstallerTests(unittest.TestCase):
    def flow(self, distro='pop', version='24.04', arch='amd64', failure='', apt_status=0):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            release = root / 'os-release'
            release.write_text(f'ID={distro}\nVERSION_ID={version}\n')
            adapter = root / 'adapter'
            adapter.write_text('''#!/usr/bin/env python3
import hashlib, json, os, pathlib, sys
name = pathlib.Path(sys.argv[0]).name
args = sys.argv[1:]
with open(os.environ['CALLS'], 'a') as log:
    log.write(json.dumps([name, *args]) + '\\n')
failure = os.environ['FAILURE']
if name == 'dpkg':
    print(os.environ['ARCH'])
elif name == 'curl':
    if failure == 'download': sys.exit(22)
    output = pathlib.Path(args[args.index('--output') + 1])
    filename = 'cosmic-ext-applet-mounter_0.5.4_ubuntu' + os.environ['OS_VERSION'] + '_' + os.environ['ARCH'] + '.deb'
    digest = hashlib.sha256(b'fixture-package').hexdigest()
    if failure == 'checksum': digest = '0' * 64
    if output.name == 'SHA256SUMS':
        line = digest + '  ' + filename + '\\n'
        output.write_text(line * (2 if failure == 'duplicate' else 1))
    else:
        output.write_bytes(b'fixture-package')
elif name == 'dpkg-deb':
    fields = {'Package': 'cosmic-ext-applet-mounter', 'Version': '0.5.4', 'Architecture': os.environ['ARCH']}
    print('wrong' if failure == args[-1] else fields[args[-1]])
elif name in ('apt-get', 'sudo'):
    sys.exit(int(os.environ['APT_STATUS']))
''')
            adapter.chmod(0o755)
            for name in ('curl', 'dpkg', 'dpkg-deb', 'apt-get', 'sudo'):
                (root / name).symlink_to(adapter)
            env = dict(os.environ, PATH=f'{root}:{os.environ["PATH"]}',
                       CALLS=str(root / 'calls'), FAILURE=failure, ARCH=arch,
                       OS_VERSION=version, APT_STATUS=str(apt_status))
            # Production reads /etc/os-release; only this sourced test adapter
            # substitutes a synthetic distribution file.
            shell = '''source "$1"
eval "$(declare -f detect_platform | sed '1s/detect_platform/detect_fixture_platform/')"
fixture_release="$2"
detect_platform() { detect_fixture_platform "$fixture_release"; }
'''
            result = subprocess.run(['bash', '-c', shell + '\nmain', 'test', str(SCRIPT), str(release)],
                                    env=env, capture_output=True, text=True, timeout=10)
            calls = [json.loads(line) for line in (root / 'calls').read_text().splitlines()] if (root / 'calls').exists() else []
            return result, calls

    def test_supported_targets(self):
        for distro in ('ubuntu', 'pop'):
            for version in ('22.04', '24.04', '26.04'):
                for arch in ('amd64', 'arm64'):
                    with self.subTest(distro=distro, version=version, arch=arch):
                        result, calls = self.flow(distro, version, arch)
                        self.assertEqual(result.returncode, 0, result.stderr)
                        apt = [call for call in calls if call[0] in ('apt-get', 'sudo')]
                        self.assertEqual(len(apt), 1)
                        self.assertIn(f'_ubuntu{version}_{arch}.deb', apt[0][-1])
                        self.assertIn('--reinstall', apt[0])
                        self.assertFalse(Path(apt[0][-1]).exists())

    def test_unsupported_systems_do_not_download_or_install(self):
        for args in (('debian', '12', 'amd64'), ('ubuntu', '25.10', 'amd64'), ('pop', '24.04', 'i386')):
            result, calls = self.flow(*args)
            self.assertNotEqual(result.returncode, 0)
            self.assertFalse(any(c[0] in ('curl', 'apt-get', 'sudo') for c in calls))

    def test_failed_verification_never_installs(self):
        for failure in ('download', 'checksum', 'duplicate', 'Package', 'Version', 'Architecture'):
            with self.subTest(failure=failure):
                result, calls = self.flow(failure=failure)
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(any(c[0] in ('apt-get', 'sudo') for c in calls))

    def test_apt_failure_propagates(self):
        result, _ = self.flow(apt_status=42)
        self.assertEqual(result.returncode, 42)
        self.assertIn('"exit_code":42', result.stdout)
        self.assertNotIn('Installed. Add', result.stdout)

    def test_stdin_help_and_invalid_version(self):
        for arguments, expected in ((['--help'], 0), (['--version', '../bad'], 1), (['--version'], 1)):
            result = subprocess.run(['bash', '-s', '--', *arguments], input=SCRIPT.read_text(),
                                    capture_output=True, text=True, timeout=10)
            self.assertEqual(result.returncode, expected, result.stderr)


if __name__ == '__main__':
    unittest.main()
