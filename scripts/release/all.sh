#!/usr/bin/env bash
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
source "$HERE/_common.sh"

banner "Packaging all artifacts for version $VERSION"

"$HERE/build_ios_xcframework.sh" || true
"$HERE/build_android_aar.sh" || true
"$HERE/build_python_wheel.sh" || true
"$HERE/build_wasm_pkg.sh" || true
"$HERE/package_headers.sh" || true
"$HERE/checksum_and_manifest.sh" || true

echo
echo "Artifacts under: $DIST_DIR"

