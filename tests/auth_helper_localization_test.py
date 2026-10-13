"""Exercise the helper's text handoff without GTK, a browser, or authentication."""
import ast
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
HELPER = ROOT / "scripts/cosmic-ext-applet-mounter-onedrive-auth-helper"


def loader(script_path=HELPER):
    source = ast.parse(HELPER.read_text())
    selected = [node for node in source.body if
                isinstance(node, ast.FunctionDef) and node.name == "load_ui_text" or
                isinstance(node, ast.Assign) and any(isinstance(target, ast.Name) and target.id in ("APP_ID", "UI_KEYS") for target in node.targets)]
    namespace = {"__file__": str(script_path), "os": os, "json": json, "pathlib": __import__("pathlib")}
    exec(compile(ast.Module(body=selected, type_ignores=[]), str(HELPER), "exec"), namespace)
    return namespace


class HelperLocalizationTests(unittest.TestCase):
    def test_localized_handoff_is_used_without_a_catalog(self):
        namespace = loader(Path("/nonexistent/bin/helper"))
        labels = {key: "認証メッセージ" for key in namespace["UI_KEYS"]}
        with patch.dict(os.environ, {"COSMIC_MOUNTER_AUTH_UI": json.dumps(labels)}, clear=True):
            self.assertEqual(namespace["load_ui_text"](), labels)

    def test_incomplete_handoff_is_rejected(self):
        namespace = loader()
        with patch.dict(os.environ, {"COSMIC_MOUNTER_AUTH_UI": "{}"}, clear=True):
            with self.assertRaisesRegex(RuntimeError, "incomplete"):
                namespace["load_ui_text"]()

    def test_source_and_installed_fallback_match_the_fluent_catalog(self):
        namespace = loader()
        with patch.dict(os.environ, {}, clear=True):
            labels = namespace["load_ui_text"]()
        self.assertEqual(set(labels), set(namespace["UI_KEYS"]))
        with tempfile.TemporaryDirectory() as directory:
            prefix = Path(directory)
            catalog = prefix / "share" / namespace["APP_ID"] / "i18n/en/cosmic_ext_applet_mounter.ftl"
            catalog.parent.mkdir(parents=True)
            catalog.write_text((ROOT / "i18n/en/cosmic_ext_applet_mounter.ftl").read_text())
            installed = loader(prefix / "bin/helper")
            with patch.dict(os.environ, {}, clear=True):
                self.assertEqual(installed["load_ui_text"](), labels)


if __name__ == "__main__":
    unittest.main()
