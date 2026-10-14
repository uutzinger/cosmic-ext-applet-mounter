"""Standalone Bash setup/report fixtures; never call real package/service tools."""

import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = [ROOT / "scripts/install-dependencies.sh", ROOT / "scripts/setup-openconnect.sh"]


class SetupReportingTests(unittest.TestCase):
    def shell(self, script, root, body, *args, input_text=None):
        env = dict(os.environ, HOME=str(root / "user"), FIXTURE_ROOT=str(root))
        return subprocess.run(["bash", "-c", 'source "$1"; shift; ' + body,
                               "setup-fixture", str(script), *map(str, args)],
                              input=input_text, env=env, capture_output=True, text=True, timeout=10)

    def test_embedded_reporter_blocks_are_identical(self):
        def block(script):
            return script.read_text().split("# BEGIN standalone setup reporting", 1)[1].split(
                "# END standalone setup reporting", 1)[0]
        self.assertEqual(block(SCRIPTS[0]), block(SCRIPTS[1]))

    def test_standalone_stdin_help_does_not_create_logs(self):
        for script in SCRIPTS:
            with self.subTest(script=script), tempfile.TemporaryDirectory() as tmp:
                logs = Path(tmp) / "absent"
                result = subprocess.run(["bash", "-s", "--", "--log-dir", str(logs), "--help"],
                                        input=script.read_text(), capture_output=True, text=True, timeout=10)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertIn("--log-dir", result.stdout)
                self.assertFalse(logs.exists())

    def test_standalone_stdin_logged_failure_without_checkout_dependencies(self):
        for script in SCRIPTS:
            with self.subTest(script=script), tempfile.TemporaryDirectory() as tmp:
                logs = Path(tmp) / "logs"
                result = subprocess.run(["bash", "-s", "--", "--log-dir", str(logs), "--invalid-fixture"],
                                        input=script.read_text(), cwd=tmp,
                                        capture_output=True, text=True, timeout=10)
                self.assertEqual(result.returncode, 1, result.stderr)
                self.assertEqual(json.loads((logs / "summary.json").read_text())["status"], "failed")

    def test_no_logging_default_and_failed_log_sink(self):
        for script in SCRIPTS:
            with self.subTest(script=script), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                plain = self.shell(script, root, '''
                    main() { echo unchanged; }
                    setup_entry
                ''')
                self.assertEqual(plain.stdout, "unchanged\n")
                self.assertEqual(list(root.iterdir()), [])
                result = self.shell(script, root, '''
                    setup_report_task=fixture
                    tee() { command cat >/dev/null; return 19; }
                    main() { echo fixture; }
                    setup_entry --log-dir "$1"
                ''', root / "logs")
                self.assertEqual(result.returncode, 1, result.stderr)
                self.assertEqual(json.loads((root / "logs/summary.json").read_text())["status"], "failed")

    def test_opt_in_output_status_privacy_and_json_escaping(self):
        for script in SCRIPTS:
            with self.subTest(script=script), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                logs = root / 'report "quoted" \\ slash'
                result = self.shell(script, root, '''
                    setup_report_task=fixture
                    main() { printf 'visible output\\n'; printf 'visible diagnostic\\n' >&2; }
                    setup_entry --log-dir "$1"
                ''', logs)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertIn("visible output", result.stdout)
                self.assertIn("visible diagnostic", result.stderr)
                self.assertIn("exit 0", result.stdout)
                report = json.loads((logs / "summary.json").read_text())
                self.assertEqual(report["status"], "succeeded")
                self.assertEqual(report["log_path"], str(logs / "task.log"))
                log = (logs / "task.log").read_text()
                self.assertIn("visible output", log)
                self.assertIn("visible diagnostic", log)
                self.assertEqual(logs.stat().st_mode & 0o777, 0o700)
                for name in ["task.log", "summary.json"]:
                    self.assertEqual((logs / name).stat().st_mode & 0o777, 0o600)
                self.assertFalse(list(logs.glob(".summary.*")))

    def test_failure_and_signal_status_no_automatic_retry(self):
        for script in SCRIPTS:
            for action, expected, status in [("return 17", 17, "failed"),
                                              ('kill -TERM "$$"', 143, "cancelled"),
                                              ('kill -INT "$$"', 130, "cancelled")]:
                with self.subTest(script=script, action=action), tempfile.TemporaryDirectory() as tmp:
                    root = Path(tmp)
                    result = self.shell(script, root, '''
                        setup_report_task=fixture
                        main() { echo once >> "$FIXTURE_ROOT/calls"; ACTION; }
                        setup_entry --log-dir "$1"
                    '''.replace("ACTION", action), root / "logs")
                    self.assertEqual(result.returncode, expected, result.stderr)
                    report = json.loads((root / "logs/summary.json").read_text())
                    self.assertEqual((report["status"], report["exit_code"]), (status, expected))
                    self.assertEqual((root / "calls").read_text(), "once\n")

    def test_existing_link_missing_or_control_path_refused_before_mutation(self):
        for script in SCRIPTS:
            with self.subTest(script=script), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                (root / "existing").mkdir()
                (root / "link").symlink_to(root / "existing")
                for args in [("--log-dir",), ("--log-dir", root / "existing"),
                             ("--log-dir", root / "link"), ("--log-dir", "bad\npath"),
                             ("--log-dir", root / "one", "--log-dir", root / "two")]:
                    result = self.shell(script, root, '''
                        setup_report_task=fixture
                        main() { echo unsafe > "$FIXTURE_ROOT/mutated"; }
                        setup_entry "$@"
                    ''', *args)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertFalse((root / "mutated").exists())

    def test_stdin_remains_usable_and_is_not_logged(self):
        for script in SCRIPTS:
            with self.subTest(script=script), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                result = self.shell(script, root, '''
                    setup_report_task=fixture
                    main() {
                        printf 'Confirmation: ' >&2
                        local answer
                        read -r answer
                        [[ "$answer" == 'private-input' ]]
                        printf 'Confirmed\\n'
                    }
                    setup_entry --log-dir "$1"
                ''', root / "logs", input_text="private-input\n")
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertIn("Confirmation:", result.stderr)
                self.assertNotIn("private-input", (root / "logs/task.log").read_text())

    def openconnect(self, root, body, *args):
        return self.shell(SCRIPTS[1], root, '''
            setup_report_task=openconnect-setup
            detect_distro() { echo debian; }
            require_supported_distro() { :; }
            require_sudo() { :; }
            run_sudo() { printf 'package:%s\\n' "$*" >> "$FIXTURE_ROOT/calls"; return "${PACKAGE_EXIT:-0}"; }
            gsettings() { printf 'setting:%s\\n' "$*" >> "$FIXTURE_ROOT/calls"; }
            systemctl() { printf 'service:%s\\n' "$*" >> "$FIXTURE_ROOT/calls"; return "${SERVICE_EXIT:-0}"; }
        ''' + body, *args)

    def test_openconnect_install_unit_icon_and_no_start(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            result = self.openconnect(root, 'setup_entry install --log-dir "$1"', root / "logs")
            self.assertEqual(result.returncode, 0, result.stderr)
            service = root / "user/.config/systemd/user/cosmic-mounter-openconnect-agent.service"
            self.assertIn("ExecStart=/usr/bin/nm-applet", service.read_text())
            calls = (root / "calls").read_text()
            self.assertIn("network-manager-openconnect-gnome", calls)
            self.assertIn("setting:set org.gnome.nm-applet show-applet false", calls)
            self.assertIn("service:--user daemon-reload", calls)
            self.assertNotIn("--user start", calls)
            self.assertNotIn("--user enable", calls)

    def test_openconnect_all_package_manager_branches_use_fake_adapter(self):
        for distro, manager in [("debian", "apt"), ("fedora", "dnf"), ("arch", "pacman"),
                                ("opensuse", "zypper"), ("gentoo", "emerge")]:
            with self.subTest(distro=distro), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                result = self.openconnect(root, f'''
                    detect_distro() {{ echo {distro}; }}
                    setup_entry install --log-dir "$1"
                ''', root / "logs")
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertIn("package:" + manager, (root / "calls").read_text())

    def test_openconnect_package_and_reload_partial_failure(self):
        for variable in ["PACKAGE_EXIT", "SERVICE_EXIT"]:
            with self.subTest(variable=variable), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                result = self.openconnect(root, f'{variable}=23; setup_entry install --log-dir "$1"', root / "logs")
                self.assertEqual(result.returncode, 23, result.stderr)
                report = json.loads((root / "logs/summary.json").read_text())
                self.assertEqual(report["status"], "failed")
                service = root / "user/.config/systemd/user/cosmic-mounter-openconnect-agent.service"
                self.assertEqual(service.exists(), variable == "SERVICE_EXIT")

    def test_openconnect_uninstall_retains_unrelated_files_and_packages(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            folder = root / "user/.config/systemd/user"
            folder.mkdir(parents=True)
            service = folder / "cosmic-mounter-openconnect-agent.service"
            service.write_text("disposable fixture")
            sentinel = folder / "unrelated.service"
            sentinel.write_text("preserve")
            result = self.openconnect(root, 'setup_entry uninstall --log-dir "$1"', root / "logs")
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertFalse(service.exists())
            self.assertEqual(sentinel.read_text(), "preserve")
            self.assertNotIn("package:", (root / "calls").read_text())
            self.assertIn("saved VPN profile were not removed", result.stderr)

    def test_dependency_components_default_and_package_failure_with_reporting(self):
        for option, expected, code in [("", "base rclone onedriver onedrive helper", 0),
                                        ("--base", "base", 0),
                                        ("--onedrive --skip-onedrive-auth-helper", "onedrive", 0),
                                        ("--base", "base", 29)]:
            with self.subTest(option=option, code=code), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                result = self.shell(SCRIPTS[0], root, '''
                    setup_report_task=dependency-setup
                    require_supported_distro() { :; }
                    require_sudo() { :; }
                    print_summary() { :; }
                    install_base() { echo base >> "$FIXTURE_ROOT/calls"; return CODE; }
                    install_rclone() { echo rclone >> "$FIXTURE_ROOT/calls"; }
                    install_onedriver() { echo onedriver >> "$FIXTURE_ROOT/calls"; }
                    install_onedrive() { echo onedrive >> "$FIXTURE_ROOT/calls"; }
                    install_onedrive_auth_helper() { echo helper >> "$FIXTURE_ROOT/calls"; }
                    setup_entry --log-dir "$1" OPTIONS
                '''.replace("CODE", str(code)).replace("OPTIONS", option), root / "logs")
                self.assertEqual(result.returncode, code, result.stderr)
                self.assertEqual((root / "calls").read_text().split(), expected.split())


if __name__ == "__main__":
    unittest.main()
