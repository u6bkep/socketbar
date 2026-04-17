#!/usr/bin/env bash
# Build the Socketbar native host and register its Firefox manifest.
# The extension itself is loaded separately via about:debugging.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo not found — install rustup from https://rustup.rs" >&2
  exit 1
fi

echo "Building release binary…"
cargo build --release --manifest-path "$HERE/host/Cargo.toml"

"$HERE/host/target/release/socketbar-host" install

echo
echo "Load the extension:"
echo "  Firefox → about:debugging#/runtime/this-firefox → Load Temporary Add-on…"
echo "  Select: $HERE/extension/manifest.json"
