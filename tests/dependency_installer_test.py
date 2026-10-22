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
        self.assertIn("--onedriver", result.stdout)
        self.assertNotIn("--onedrive ", result.stdout)
        self.assertNotIn("--onedrive-auth-helper", result.stdout)

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
            main "$@"
        """, *arguments)
        self.assertEqual(result.returncode, 0, result.stderr)
        return [line for line in result.stdout.splitlines() if line.startswith("step-")]

    def test_default_and_all_retain_engines_without_helper(self):
        expected = ["step-base", "step-rclone", "step-onedriver"]
        self.assertEqual(self.flow(), expected)
        self.assertEqual(self.flow("--all"), expected)

    def test_obs_repository_matches_distribution_not_package_manager_family(self):
        for distro, version, expected in [
            ("pop", "24.04", "xUbuntu_24.04"),
            ("ubuntu", "24.04", "xUbuntu_24.04"),
            ("ubuntu", "22.04", "xUbuntu_22.04"),
            ("debian", "12", "Debian_12"),
            ("opensuse-tumbleweed", "20261018", "openSUSE_Tumbleweed"),
            ("pop", "99.04", ""),
            ("unknown", "12", ""),
        ]:
            with self.subTest(distro=distro, version=version):
                result = self.run_shell('''
                    read_os_release() {
                        case "$1" in
                            ID) printf '%s' "$test_distro" ;;
                            VERSION_ID) printf '%s' "$test_version" ;;
                        esac
                    }
                    test_distro="$1"
                    test_version="$2"
                    obs_repo_suffix
                ''', distro, version)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(result.stdout, expected)

    def test_engine_selections_remain_independent(self):
        for option, expected in [("--rclone", "step-rclone"),
                                 ("--onedriver", "step-onedriver"), ("--base", "step-base")]:
            self.assertEqual(self.flow(option), [expected])

    def test_retired_helper_options_fail_before_privileged_work(self):
        for option in ["--onedrive", "--onedrive-auth-helper", "--skip-onedrive-auth-helper"]:
            result = self.run_shell("""
                require_sudo() { echo unexpected-sudo; return 99; }
                pkg_install() { echo unexpected-install; return 99; }
                main "$@"
            """, option)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("Unknown option", result.stderr)
            self.assertNotIn("unexpected-", result.stdout)

    def test_selected_engine_package_failure_propagates(self):
        result = self.run_shell("""
            require_supported_distro() { :; }
            require_sudo() { :; }
            install_rclone() { return 17; }
            main --rclone
        """)
        self.assertEqual(result.returncode, 17)


if __name__ == "__main__":
    unittest.main()
