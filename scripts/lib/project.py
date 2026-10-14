"""Project-specific constants shared by the release tooling.

Adapt this file (and the hosted workflow it names) when reusing the scripts in
another project. tests/project_constants_test.py checks PACKAGE against Cargo.toml.
"""

PACKAGE = "cosmic-ext-applet-mounter"
WORKFLOW = "build-deb.yml"
WORKFLOW_PATH = f".github/workflows/{WORKFLOW}"
MATRIX = [(version, arch) for version in ("22.04", "24.04", "26.04")
          for arch in ("amd64", "arm64")]
ARTIFACT_COUNT = len(MATRIX)
