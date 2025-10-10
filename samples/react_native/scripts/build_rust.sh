#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"
APP_DIR="$(cd "$(dirname "$0")/.." && pwd)/PantherSampleReactNative"

pushd "$ROOT_DIR" >/dev/null

# iOS (simulator staticlib)
rustup target add aarch64-apple-ios-sim >/dev/null 2>&1 || true
cargo build -p panther-ffi --features "metrics storage" --release --target aarch64-apple-ios-sim
mkdir -p "$APP_DIR/ios/libs" || true
cp "target/aarch64-apple-ios-sim/release/libpanther_ffi.a" "$APP_DIR/ios/libs/" || true

# Android (shared libs)
if ! command -v cargo-ndk >/dev/null 2>&1; then
  cargo install cargo-ndk >/dev/null 2>&1 || true
fi
rustup target add aarch64-linux-android x86_64-linux-android >/dev/null 2>&1 || true
cargo ndk -t arm64-v8a -o "$APP_DIR/android/app/src/main/jniLibs" build -p panther-ffi --features "metrics storage validation validation-openai validation-ollama" --release || true
cargo ndk -t x86_64     -o "$APP_DIR/android/app/src/main/jniLibs" build -p panther-ffi --features "metrics storage validation validation-openai validation-ollama" --release || true

# Header
if ! command -v cbindgen >/dev/null 2>&1; then
  cargo install cbindgen >/dev/null 2>&1 || true
fi
mkdir -p "$ROOT_DIR/bindings/include"
pushd crates/panther-ffi >/dev/null
cbindgen --crate panther-ffi --output "$ROOT_DIR/bindings/include/panther.h" --config "$ROOT_DIR/cbindgen.toml"
popd >/dev/null

echo "React Native: Rust artifacts prepared."
popd >/dev/null
