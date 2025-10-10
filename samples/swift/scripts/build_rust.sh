#!/usr/bin/env bash
set -euo pipefail

# Ensure cargo/cbindgen are on PATH when run from Xcode
if [ -f "$HOME/.cargo/env" ]; then
  # shellcheck source=/dev/null
  . "$HOME/.cargo/env"
fi
export PATH="$HOME/.cargo/bin:$PATH"

ROOT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"
SAMPLE_DIR="$(cd "$(dirname "$0")/.." && pwd)"
LIB_DST="$SAMPLE_DIR/libs/libpanther_ffi.a"
HDR_DST="$SAMPLE_DIR/include/panther.h"

mkdir -p "$SAMPLE_DIR/libs" "$SAMPLE_DIR/include"

# If cargo is not available, try to proceed with existing artifacts
if ! command -v cargo >/dev/null 2>&1; then
  echo "WARN: cargo not found on PATH; using existing artifacts if present" >&2
  if [ -f "$LIB_DST" ] && [ -f "$HDR_DST" ]; then
    echo "Using prebuilt lib/header."
    exit 0
  else
    echo "ERROR: Missing prebuilt lib/header and cargo is unavailable." >&2
    exit 1
  fi
fi

build_rust() {
  rustup target add aarch64-apple-ios-sim >/dev/null 2>&1 || true
  (cd "$ROOT_DIR" && cargo build -p panther-ffi --features "metrics-inmemory storage-inmemory validation validation-openai validation-ollama" --release --target aarch64-apple-ios-sim)
}

gen_header() {
  if command -v cbindgen >/dev/null 2>&1; then
    (cd "$ROOT_DIR/crates/panther-ffi" && cbindgen --crate panther-ffi --output "$HDR_DST" --config "$ROOT_DIR/cbindgen.toml")
  else
    echo "cbindgen not found; keeping existing header at $HDR_DST" >&2
  fi
}

# Try to build; if it fails but an existing lib is present, continue
if build_rust; then
  cp "$ROOT_DIR/target/aarch64-apple-ios-sim/release/libpanther_ffi.a" "$LIB_DST"
else
  echo "WARN: cargo build failed; attempting to use existing lib at $LIB_DST" >&2
  if [ ! -f "$LIB_DST" ]; then
    echo "ERROR: No existing lib found; cannot continue" >&2
    exit 1
  fi
fi

# Generate/update header (best-effort)
gen_header || true

echo "Rust FFI built/copied. Header generated/updated if possible."
