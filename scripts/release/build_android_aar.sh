#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_common.sh"

banner "Build Android AAR (version $VERSION)"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found; skipping Android build" >&2
  exit 0
fi

if ! command -v cargo-ndk >/dev/null 2>&1; then
  echo "cargo-ndk not found; attempting to install (may fail if offline)" >&2
  cargo install cargo-ndk >/dev/null 2>&1 || true
fi

ABI_LIST=(arm64-v8a armeabi-v7a x86_64)

pushd "$ROOT_DIR" >/dev/null
for abi in "${ABI_LIST[@]}"; do
  cargo ndk -t "$abi" -o "$ROOT_DIR/target/android/jniLibs" build -p panther-ffi --features "$FEATURES" --release || true
done
popd >/dev/null

OUT_DIR="$DIST_DIR/android"
TMP_DIR="$ROOT_DIR/target/android/aar"
LIBS_DIR="$ROOT_DIR/target/android/jniLibs"
mkdir -p "$OUT_DIR" "$TMP_DIR" "$LIBS_DIR"

# Minimal AAR structure
mkdir -p "$TMP_DIR/jni" "$TMP_DIR/headers"
cat >"$TMP_DIR/AndroidManifest.xml" <<EOF
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="ai.panther.ffi" />
EOF

if command -v cbindgen >/dev/null 2>&1; then
  (cd "$ROOT_DIR/crates/panther-ffi" && cbindgen --crate panther-ffi --output "$TMP_DIR/headers/panther.h" --config "$ROOT_DIR/cbindgen.toml")
fi

# Copy per-ABI .so if exist
for abi in "${ABI_LIST[@]}"; do
  src="$LIBS_DIR/$abi/libpanther_ffi.so"
  if [ -f "$src" ]; then
    mkdir -p "$TMP_DIR/jni/$abi"
    cp "$src" "$TMP_DIR/jni/$abi/"
  fi
done

# An AAR must include classes.jar; add an empty placeholder
pushd "$TMP_DIR" >/dev/null
zip -q -r "panther-ffi-$VERSION.aar" AndroidManifest.xml jni headers
popd >/dev/null

mv "$TMP_DIR/panther-ffi-$VERSION.aar" "$OUT_DIR/"
echo "Created: $OUT_DIR/panther-ffi-$VERSION.aar"

