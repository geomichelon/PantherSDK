#!/usr/bin/env bash
set -euo pipefail

# Refresh the iOS XCFramework and copy it into the sample's central folder.

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# Features used by the sample (override by exporting FEATURES="...")
: "${FEATURES:=metrics-inmemory storage-inmemory validation validation-openai validation-ollama}"

# Ensure release script is executable and build the xcframework into dist/<version>/ios
chmod +x "$ROOT_DIR/scripts/release/build_ios_xcframework.sh" "$ROOT_DIR/scripts/release/_common.sh" 2>/dev/null || true
FEATURES="$FEATURES" "$ROOT_DIR/scripts/release/build_ios_xcframework.sh"

# Resolve workspace version
VERSION=$(awk '
  $0 ~ /^\[workspace\.package\]/ { inpkg=1; next }
  inpkg && $1 ~ /^version/ { gsub(/\"/, ""); gsub(/version *= */, ""); print $0; exit }
' "$ROOT_DIR/Cargo.toml" | tr -d '[:space:]')

SRC="$ROOT_DIR/dist/$VERSION/ios/PantherSDK.xcframework"
DST_DIR="$ROOT_DIR/samples/swift/frameworkIOS"
DST="$DST_DIR/PantherSDKIOS.xcframework"

if [ ! -d "$SRC" ]; then
  echo "XCFramework not found at: $SRC" >&2
  exit 1
fi

# Flatten copy so Info.plist sits at the root of PantherSDKIOS.xcframework
rm -rf "$DST"
mkdir -p "$DST"
rsync -a "$SRC/" "$DST/"

echo "Refreshed: $DST"
find "$DST" -maxdepth 2 -print

