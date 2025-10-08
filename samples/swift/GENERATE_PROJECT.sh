#!/usr/bin/env bash
set -euo pipefail

if ! command -v xcodegen >/dev/null 2>&1; then
  echo "Please install XcodeGen: brew install xcodegen" >&2
  exit 1
fi

cd "$(dirname "$0")"
xcodegen generate
echo "Open PantherSample.xcodeproj in Xcode."
