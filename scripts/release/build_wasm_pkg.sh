#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_common.sh"

banner "Build WASM npm package (version $VERSION)"

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "wasm-pack not found; skipping wasm build" >&2
  exit 0
fi

OUT_DIR="$DIST_DIR/npm"
mkdir -p "$OUT_DIR"

pushd "$ROOT_DIR/crates/panther-wasm" >/dev/null
wasm-pack build --release --target bundler --out-dir "$OUT_DIR" || true
popd >/dev/null

echo "WASM package in: $OUT_DIR"

