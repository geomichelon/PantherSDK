#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"
APP_DIR="$(cd "$(dirname "$0")/.." && pwd)/panther_sample_flutter"

pushd "$ROOT_DIR" >/dev/null

# Android (arm64, x86_64)
if ! command -v cargo-ndk >/dev/null 2>&1; then
  cargo install cargo-ndk >/dev/null 2>&1 || true
fi
rustup target add aarch64-linux-android x86_64-linux-android >/dev/null 2>&1 || true
cargo ndk -t arm64-v8a -o "$APP_DIR/android/app/src/main/jniLibs" build -p panther-ffi --features "metrics storage" --release || true
cargo ndk -t x86_64     -o "$APP_DIR/android/app/src/main/jniLibs" build -p panther-ffi --features "metrics storage" --release || true

# iOS Simulator staticlib (optional for simulator runs)
rustup target add aarch64-apple-ios-sim >/dev/null 2>&1 || true
cargo build -p panther-ffi --features "metrics storage" --release --target aarch64-apple-ios-sim || true
mkdir -p "$APP_DIR/ios/libs" || true
cp "target/aarch64-apple-ios-sim/release/libpanther_ffi.a" "$APP_DIR/ios/libs/" || true

# Header for iOS bridge code
if ! command -v cbindgen >/dev/null 2>&1; then
  cargo install cbindgen >/dev/null 2>&1 || true
fi
mkdir -p "$ROOT_DIR/bindings/include"
pushd crates/panther-ffi >/dev/null
cbindgen --crate panther-ffi --output "$ROOT_DIR/bindings/include/panther.h" --config "$ROOT_DIR/cbindgen.toml"
popd >/dev/null

echo "Flutter: Rust artifacts prepared."
popd >/dev/null

