#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_common.sh"

banner "Package headers (version $VERSION)"

if command -v cbindgen >/dev/null 2>&1; then
  mkdir -p "$INCLUDE_DIR"
  (cd "$ROOT_DIR/crates/panther-ffi" && cbindgen --crate panther-ffi --output "$INCLUDE_DIR/panther.h" --config "$ROOT_DIR/cbindgen.toml")
else
  echo "cbindgen not found; cannot generate header" >&2
fi

pushd "$DIST_DIR" >/dev/null
zip -q -r "c-headers-$VERSION.zip" "include" || true
popd >/dev/null

echo "Headers packaged in: $DIST_DIR/include and c-headers-$VERSION.zip"

