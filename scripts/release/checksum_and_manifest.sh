#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "$0")" && pwd)/_common.sh"

banner "Checksums and manifest (version $VERSION)"

# Checksums
pushd "$DIST_DIR" >/dev/null
if command -v shasum >/dev/null 2>&1; then
  find . -type f -maxdepth 4 -print0 | xargs -0 shasum -a 256 > "SHA256SUMS" || true
elif command -v sha256sum >/dev/null 2>&1; then
  find . -type f -maxdepth 4 -print0 | xargs -0 sha256sum > "SHA256SUMS" || true
else
  echo "No sha256 tool found; skipping checksums" >&2
fi
popd >/dev/null

# Manifest
COMMIT="$(git -C "$ROOT_DIR" rev-parse --short HEAD 2>/dev/null || echo unknown)"
DATE_ISO="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
cat >"$DIST_DIR/manifest.json" <<JSON
{
  "version": "$VERSION",
  "commit": "$COMMIT",
  "features": "${FEATURES}",
  "built_at": "$DATE_ISO"
}
JSON

echo "Wrote: $DIST_DIR/manifest.json"

