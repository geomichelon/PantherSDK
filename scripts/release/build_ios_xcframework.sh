#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_common.sh"

banner "Build iOS xcframework (version $VERSION)"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found; skipping iOS build" >&2
  exit 0
fi
if ! command -v xcodebuild >/dev/null 2>&1; then
  echo "xcodebuild not found; skipping iOS build" >&2
  exit 0
fi

rustup target add aarch64-apple-ios aarch64-apple-ios-sim >/dev/null 2>&1 || true

pushd "$ROOT_DIR" >/dev/null
cargo build -p panther-ffi --features "$FEATURES" --release --target aarch64-apple-ios
cargo build -p panther-ffi --features "$FEATURES" --release --target aarch64-apple-ios-sim
popd >/dev/null

IOS_LIB_DEVICE="$ROOT_DIR/target/aarch64-apple-ios/release/libpanther_ffi.a"
IOS_LIB_SIM="$ROOT_DIR/target/aarch64-apple-ios-sim/release/libpanther_ffi.a"

if [ ! -f "$IOS_LIB_DEVICE" ] || [ ! -f "$IOS_LIB_SIM" ]; then
  echo "iOS static libraries not found; check cargo output" >&2
  exit 0
fi

# Generate header
if command -v cbindgen >/dev/null 2>&1; then
  mkdir -p "$INCLUDE_DIR"
  (cd "$ROOT_DIR/crates/panther-ffi" && cbindgen --crate panther-ffi --output "$INCLUDE_DIR/panther.h" --config "$ROOT_DIR/cbindgen.toml")
else
  echo "cbindgen not found; header will be missing in xcframework" >&2
fi

OUT_DIR="$DIST_DIR/ios"
mkdir -p "$OUT_DIR"

<<<<<<< HEAD
# Ensure we start from a clean xcframework dir to avoid partial overwrites
rm -rf "$OUT_DIR/PantherSDK.xcframework"

=======
>>>>>>> origin/main
xcodebuild -create-xcframework \
  -library "$IOS_LIB_SIM" -headers "$INCLUDE_DIR" \
  -library "$IOS_LIB_DEVICE" -headers "$INCLUDE_DIR" \
  -output "$OUT_DIR/PantherSDK.xcframework"

echo "Created: $OUT_DIR/PantherSDK.xcframework"
<<<<<<< HEAD
=======

>>>>>>> origin/main
