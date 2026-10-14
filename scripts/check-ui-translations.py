#!/usr/bin/env python3
"""Check Fluent catalog identity and literal Rust message references.

Rust compilation additionally validates Fluent syntax and interpolation arguments.
This check does not classify arbitrary string literals as UI or protocol data.
"""
from pathlib import Path
import re
import sys


def main():
    root = Path(__file__).resolve().parents[1]
    catalogs = sorted((root / "i18n").glob("*/cosmic_ext_applet_mounter.ftl"))
    fallback = root / "i18n/en/cosmic_ext_applet_mounter.ftl"
    catalog_keys = {}
    errors = []
    for catalog in catalogs:
        keys = set()
        for line_number, line in enumerate(catalog.read_text().splitlines(), 1):
            match = re.match(r"^([A-Za-z][A-Za-z0-9_-]*)\s*=", line)
            if match:
                key = match.group(1)
                if key in keys:
                    errors.append(f"{catalog.relative_to(root)}:{line_number}: duplicate {key}")
                keys.add(key)
        catalog_keys[catalog] = keys
    if fallback not in catalog_keys:
        errors.append("English fallback catalog is missing")
    references = 0
    for source in sorted((root / "src").rglob("*.rs")):
        for match in re.finditer(r'\bfl!\s*\(\s*"([A-Za-z][A-Za-z0-9_-]*)"', source.read_text()):
            references += 1
            if match.group(1) not in catalog_keys.get(fallback, set()):
                errors.append(f"{source.relative_to(root)}: missing fallback message {match.group(1)}")
        if source.relative_to(root).as_posix() in ("src/app.rs", "src/app/sftp.rs"):
            production = re.split(r"#\[cfg\(test\)\]\s*mod tests\b", source.read_text(), maxsplit=1)[0]
            patterns = {
                "rendered widget": r'widget::(?:text::(?:body|caption|title[0-9]?)|button::(?:standard|suggested|destructive|text))\(\s*"[A-Za-z]',
                "toggler label": r'\.label\(\s*"[A-Za-z]',
                "connection notice": r'last_notice\s*=\s*Some\(\s*(?:format!\s*\(|"[A-Za-z])',
                "settings notice": r'(?:sleep_notice|preload_notice)\s*=\s*Some\(\s*(?:format!\s*\(|"[A-Za-z])',
            }
            for description, pattern in patterns.items():
                for match in re.finditer(pattern, production):
                    line = production.count("\n", 0, match.start()) + 1
                    errors.append(f"{source.relative_to(root)}:{line}: hardcoded {description}")
    for error in errors:
        print(error, file=sys.stderr)
    if errors:
        return 1
    print(f"Checked {len(catalogs)} catalog(s), {len(catalog_keys[fallback])} fallback messages, and {references} Rust references; no duplicate or missing keys.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
