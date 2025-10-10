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
LOG_FILE="$SAMPLE_DIR/build_rust.log"

mkdir -p "$SAMPLE_DIR/libs" "$SAMPLE_DIR/include"

# Fast path: if artifacts already exist, do nothing (keeps Xcode clean & fast)
if [ -f "$LIB_DST" ] && [ -f "$HDR_DST" ]; then
  echo "Using prebuilt lib/header (skip Rust build)." >"$LOG_FILE"
  exit 0
fi

# If cargo is not available, fail only if we don't have artifacts
if ! command -v cargo >/dev/null 2>&1; then
  echo "WARN: cargo not found on PATH; attempting to use prebuilt artifacts" >>"$LOG_FILE"
  if [ -f "$LIB_DST" ] && [ -f "$HDR_DST" ]; then
    echo "Using prebuilt lib/header." >>"$LOG_FILE"
    exit 0
  fi
  echo "ERROR: Missing prebuilt lib/header and cargo is unavailable." >>"$LOG_FILE"
  exit 1
fi

build_rust() {
  rustup target add aarch64-apple-ios-sim >/dev/null 2>&1 || true
  # Build quietly to a log to avoid noisy Xcode issue list
  (cd "$ROOT_DIR" && cargo build -p panther-ffi --features "metrics-inmemory storage-inmemory validation validation-openai validation-ollama" --release --target aarch64-apple-ios-sim) >"$LOG_FILE" 2>&1
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
  echo "WARN: cargo build failed; using existing lib if present (see $LOG_FILE)" >>"$LOG_FILE"
  if [ ! -f "$LIB_DST" ]; then
    echo "ERROR: No existing lib found; cannot continue (see $LOG_FILE)" >>"$LOG_FILE"
    exit 0  # Non-fatal to avoid breaking previews; link will fail if truly missing
  fi
fi

# Generate/update header (best-effort)
gen_header || true

echo "Rust FFI ready (using prebuilt or freshly built)." >>"$LOG_FILE"
