#!/usr/bin/env bash
set -euo pipefail

if ! command -v flutter >/dev/null 2>&1; then
  echo "Please install Flutter SDK and ensure 'flutter' is on PATH." >&2
  exit 1
fi

cd "$(dirname "$0")"

APP_DIR=panther_sample_flutter
if [ ! -d "$APP_DIR" ]; then
  flutter create $APP_DIR
fi

mkdir -p "$APP_DIR/lib"
cp -f lib/*.dart "$APP_DIR/lib/" || true
echo "Project created in samples/flutter/$APP_DIR."
echo "Link native libraries per README to run on device/simulator."
