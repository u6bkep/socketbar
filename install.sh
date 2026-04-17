#!/usr/bin/env bash
# Build the Socketbar native host and install its Firefox manifest.
# The extension itself is loaded separately via about:debugging.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
HOST_CRATE="$HERE/host"
TEMPLATE="$HERE/host-manifest/io.socketbar.host.json.template"
DEST_DIR="$HOME/.mozilla/native-messaging-hosts"
DEST="$DEST_DIR/io.socketbar.host.json"

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo not found — install rustup from https://rustup.rs" >&2
  exit 1
fi

echo "Building release binary…"
(cd "$HOST_CRATE" && cargo build --release)
BIN="$HOST_CRATE/target/release/socketbar-host"
if [[ ! -x "$BIN" ]]; then
  echo "error: build produced no executable at $BIN" >&2
  exit 1
fi

mkdir -p "$DEST_DIR"
sed "s|__HOST_PATH__|$BIN|g" "$TEMPLATE" > "$DEST"

echo
echo "Installed:"
echo "  binary:   $BIN"
echo "  manifest: $DEST"
echo
echo "Next steps:"
echo "  1. Open Firefox → about:debugging#/runtime/this-firefox"
echo "  2. Click 'Load Temporary Add-on…'"
echo "  3. Select:  $HERE/extension/manifest.json"
echo "  4. Click the toolbar icon, or type 'lh ' in the URL bar"
echo
echo "Sanity-check the host alone:"
echo "  python3 -c 'import struct,sys; m=b\"{\\\"id\\\":\\\"1\\\",\\\"action\\\":\\\"list\\\"}\"; sys.stdout.buffer.write(struct.pack(\"@I\",len(m))+m)' \\"
echo "    | $BIN \\"
echo "    | python3 -c 'import struct,sys,json; d=sys.stdin.buffer.read(); n=struct.unpack(\"@I\",d[:4])[0]; print(len(json.loads(d[4:4+n]).get(\"ports\",[])), \"ports\")'"
