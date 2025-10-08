#!/usr/bin/env bash
set -euo pipefail

if ! command -v npx >/dev/null 2>&1; then
  echo "Please install Node.js and npm (npx)." >&2
  exit 1
fi

cd "$(dirname "$0")"

APP_DIR=PantherSampleReactNative
if [ ! -d "$APP_DIR" ]; then
  npx react-native init $APP_DIR --version latest
fi

cp -f ios/PantherModule.m "$APP_DIR/ios/" || true
mkdir -p "$APP_DIR/android/app/src/main/cpp"
cp -f android/panther_jni.c "$APP_DIR/android/app/src/main/cpp/panther_jni.c"
mkdir -p "$APP_DIR/android/app/src/main/java/com/example/panther"
cp -f android/PantherModule.java "$APP_DIR/android/app/src/main/java/com/example/panther/PantherModule.java"
cp -f Panther.ts "$APP_DIR/"
echo "Now wire native modules into RN project per README."
