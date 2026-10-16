#!/usr/bin/env python3
"""Installer policy tests using fake adapters only; never install packages."""
import pathlib
import subprocess
import unittest

INSTALLER = pathlib.Path(__file__).resolve().parents[1] / "scripts/install-dependencies.sh"


class DependencyInstallerTests(unittest.TestCase):
    def test_stdin_help_preserves_curl_pipe_bash_usage(self):
        result = subprocess.run(["bash", "-s", "--", "--help"], input=INSTALLER.read_text(),
                                capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("--onedrive-auth-helper", result.stdout)

    def run_shell(self, body, *arguments):
        return subprocess.run(
            ["bash", "-c", 'source "$1"; shift; ' + body, "installer-test", str(INSTALLER), *arguments],
            capture_output=True, text=True, timeout=10,
        )

    def flow(self, *arguments):
        result = self.run_shell("""
            require_supported_distro() { :; }
            require_sudo() { :; }
            print_summary() { :; }
            install_base() { echo step-base; }
            install_rclone() { echo step-rclone; }
            install_onedriver() { echo step-onedriver; }
            install_onedrive() { echo step-onedrive-current; }
            install_onedrive_auth_helper() { echo step-helper; }
            main "$@"
        """, *arguments)
        self.assertEqual(result.returncode, 0, result.stderr)
        return [line for line in result.stdout.splitlines() if line.startswith("step-")]

    def test_default_and_all_include_helper(self):
        expected = ["step-base", "step-rclone", "step-onedriver", "step-onedrive-current", "step-helper"]
        self.assertEqual(self.flow(), expected)
        self.assertEqual(self.flow("--all"), expected)

    def test_rclone_only_does_not_install_legacy_engines_or_helper(self):
        self.assertEqual(self.flow("--rclone"), ["step-rclone"])

    def test_current_onedrive_still_checks_helper(self):
        self.assertEqual(self.flow("--onedrive"), ["step-onedrive-current", "step-helper"])

    def test_helper_only_and_skip_do_not_touch_other_engines(self):
        self.assertEqual(self.flow("--onedrive-auth-helper"), ["step-helper"])
        self.assertEqual(self.flow("--onedrive", "--skip-onedrive-auth-helper"), ["step-onedrive-current"])
        self.assertEqual(self.flow("--base"), ["step-base"])

    def test_verified_package_mappings_and_post_install_probe(self):
        expected = {
            "debian": "python3 python3-gi gir1.2-gtk-3.0 gir1.2-webkit2-4.1",
            "fedora": "python3 python3-gobject gtk3 webkit2gtk4.1",
            "arch": "python python-gobject gtk3 webkit2gtk-4.1",
        }
        for distro, packages in expected.items():
            with self.subTest(distro=distro):
                result = self.run_shell("""
                    detect_distro() { printf '%s' "$task_test_distro"; }
                    task_test_distro="$1"
                    task_test_probes=0
                    onedrive_auth_helper_available() {
                        task_test_probes=$((task_test_probes + 1))
                        [[ "$task_test_probes" -gt 1 ]]
                    }
                    pkg_update_index() { echo index; }
                    pkg_install() { printf 'packages:%s\\n' "$*"; }
                    install_onedrive_auth_helper
                    [[ "$task_test_probes" -eq 2 ]]
                """, distro)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertIn("packages:" + packages, result.stdout)

    def test_existing_runtime_does_not_invoke_package_manager(self):
        result = self.run_shell("""
            onedrive_auth_helper_available() { return 0; }
            pkg_update_index() { return 99; }
            pkg_install() { return 99; }
            install_onedrive_auth_helper
        """)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_failed_import_after_install_is_actionable_failure(self):
        result = self.run_shell("""
            detect_distro() { echo debian; }
            onedrive_auth_helper_available() { return 1; }
            pkg_update_index() { :; }
            pkg_install() { :; }
            install_onedrive_auth_helper
        """)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("--skip-onedrive-auth-helper", result.stderr)

    def test_manual_distributions_do_not_guess_or_install_packages(self):
        for distro in ["opensuse", "gentoo"]:
            result = self.run_shell("""
                task_test_distro="$1"
                detect_distro() { echo "$task_test_distro"; }
                onedrive_auth_helper_available() { return 1; }
                pkg_install() { return 99; }
                install_onedrive_auth_helper
            """, distro)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("manual browser/response-URL", result.stderr)

    def test_package_manager_failure_propagates(self):
        result = self.run_shell("""
            detect_distro() { echo debian; }
            onedrive_auth_helper_available() { return 1; }
            pkg_update_index() { :; }
            pkg_install() { return 17; }
            install_onedrive_auth_helper
        """)
        self.assertEqual(result.returncode, 17)


if __name__ == "__main__":
    unittest.main()
