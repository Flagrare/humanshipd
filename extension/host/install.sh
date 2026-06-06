#!/usr/bin/env bash
# Register the humanshipd native messaging host so the browser extension can
# reach the local host binary.
#
# Usage: bash extension/host/install.sh <EXTENSION_ID>
#   Get <EXTENSION_ID> from chrome://extensions after loading the unpacked
#   extension (Developer mode → Load unpacked → select the extension/ folder).
set -euo pipefail

EXT_ID="${1:-}"
if [ -z "$EXT_ID" ]; then
  echo "usage: bash extension/host/install.sh <EXTENSION_ID>"
  exit 2
fi

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cargo build -p humanshipd-host
HOST_BIN="$ROOT/target/debug/humanshipd-host"

# Chrome's per-user native messaging host directory (macOS).
DEST_DIR="$HOME/Library/Application Support/Google/Chrome/NativeMessagingHosts"
mkdir -p "$DEST_DIR"
cat > "$DEST_DIR/dev.humanshipd.host.json" <<JSON
{
  "name": "dev.humanshipd.host",
  "description": "humanshipd native messaging host",
  "path": "$HOST_BIN",
  "type": "stdio",
  "allowed_origins": ["chrome-extension://$EXT_ID/"]
}
JSON

echo "Installed native host manifest:"
echo "  $DEST_DIR/dev.humanshipd.host.json"
echo "  → host binary: $HOST_BIN"
echo "  → allowed extension: $EXT_ID"
echo "Restart Chrome, then use the extension popup."
