"""Offline contract checks for the registry vendor inputs in the source Flatpak."""

import json
from pathlib import Path
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]


class CargoSourceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.packages = tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]
        cls.sources = json.loads((ROOT / "packaging/flatpak/cargo-sources.json").read_text())

    def test_registry_archives_exactly_match_locked_packages_and_checksums(self):
        expected = {
            f"cargo/vendor/{package['name']}-{package['version']}": package
            for package in self.packages if package.get("source", "").startswith("registry+")
        }
        archives = [source for source in self.sources if source.get("type") == "archive"]
        actual = {source["dest"]: source for source in archives}
        self.assertEqual(len(archives), len(actual), "duplicate registry destinations")
        self.assertEqual(set(actual), set(expected), "missing or obsolete crate archives")
        for destination, package in expected.items():
            with self.subTest(crate=destination):
                source = actual[destination]
                self.assertEqual(source["sha256"], package["checksum"])
                self.assertEqual(source["archive-type"], "tar-gzip")
                self.assertEqual(source["url"],
                    f"https://static.crates.io/crates/{package['name']}/{package['name']}-{package['version']}.crate")

    def test_vendor_checksum_files_and_registry_replacement_match_archives(self):
        archives = {source["dest"]: source["sha256"] for source in self.sources
                    if source.get("type") == "archive"}
        checksums = [source for source in self.sources
                     if source.get("type") == "inline" and source.get("dest-filename") == ".cargo-checksum.json"]
        actual = {source["dest"]: json.loads(source["contents"])["package"] for source in checksums}
        self.assertEqual(len(checksums), len(actual), "duplicate vendor checksum files")
        self.assertEqual({dest: checksum for dest, checksum in actual.items() if dest in archives}, archives)
        # Git dependency vendor checksum files use package=null rather than a registry archive hash.
        for dest, checksum in actual.items():
            if dest not in archives:
                self.assertIsNone(checksum, dest)
        configs = [source for source in self.sources
                   if source.get("dest") == "cargo" and source.get("dest-filename") == "config"]
        self.assertEqual(len(configs), 1)
        config = tomllib.loads(configs[0]["contents"])["source"]
        self.assertEqual(config["crates-io"]["replace-with"], "vendored-sources")
        self.assertEqual(config["vendored-sources"]["directory"], "cargo/vendor")


if __name__ == "__main__":
    unittest.main()
