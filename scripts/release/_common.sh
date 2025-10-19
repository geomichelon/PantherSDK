#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"

# Read version from workspace Cargo.toml unless VERSION provided
if [ -z "${VERSION:-}" ]; then
  VERSION=$(awk '
    $0 ~ /^\[workspace\.package\]/ { inpkg=1; next }
    inpkg && $1 ~ /^version/ { gsub(/\"/, ""); gsub(/version *= */, ""); print $0; exit }
  ' "$ROOT_DIR/Cargo.toml" | tr -d '[:space:]')
  VERSION=${VERSION:-0.0.0}
fi

FEATURES=${FEATURES:-"metrics-inmemory storage-inmemory validation validation-openai validation-ollama"}
DIST_DIR="$ROOT_DIR/dist/$VERSION"
INCLUDE_DIR="$DIST_DIR/include"

mkdir -p "$DIST_DIR" "$INCLUDE_DIR"

banner() { echo; echo "==== $* ===="; }

