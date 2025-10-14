#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_common.sh"

banner "Build Python wheel (version $VERSION)"

if ! command -v maturin >/dev/null 2>&1; then
  echo "maturin not found; please install (pip install maturin)" >&2
  exit 0
fi

OUT_DIR="$DIST_DIR/python"
mkdir -p "$OUT_DIR"

pushd "$ROOT_DIR" >/dev/null
maturin build -m crates/panther-py/Cargo.toml --release -o "$OUT_DIR" || true
popd >/dev/null

echo "Wheels in: $OUT_DIR"

