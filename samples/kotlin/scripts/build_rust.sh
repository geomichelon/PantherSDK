#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"
APP_DIR="$(cd "$(dirname "$0")/../android/app" && pwd)"

if ! command -v cargo-ndk >/dev/null 2>&1; then
  cargo install cargo-ndk >/dev/null 2>&1 || true
fi

pushd "$ROOT_DIR" >/dev/null
rustup target add aarch64-linux-android x86_64-linux-android >/dev/null 2>&1 || true

cargo ndk -t arm64-v8a -o "$APP_DIR/src/main/jniLibs" build -p panther-ffi --features "metrics storage validation validation-openai validation-ollama" --release
cargo ndk -t x86_64     -o "$APP_DIR/src/main/jniLibs" build -p panther-ffi --features "metrics storage validation validation-openai validation-ollama" --release

if ! command -v cbindgen >/dev/null 2>&1; then
  cargo install cbindgen >/dev/null 2>&1 || true
fi
mkdir -p "$ROOT_DIR/bindings/include"
pushd crates/panther-ffi >/dev/null
cbindgen --crate panther-ffi --output "$ROOT_DIR/bindings/include/panther.h" --config "$ROOT_DIR/cbindgen.toml"
popd >/dev/null
popd >/dev/null

echo "Android shared libraries and headers prepared."
