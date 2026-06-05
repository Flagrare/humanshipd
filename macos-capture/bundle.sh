#!/usr/bin/env bash
# Wrap the AX probe in a minimal, code-signed .app bundle so it has its OWN TCC
# identity (CFBundleIdentifier). Launched via `open`, the app becomes its own
# responsible process — escaping the terminal-responsibility attribution that
# makes a bare `cargo run` binary fail Accessibility with -25204.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP="$ROOT/target/HumanshipdProbe.app"
BIN_NAME="HumanshipdProbe"

cargo build -p humanshipd-macos-capture

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS"
cp "$ROOT/target/debug/humanshipd-macos-capture" "$APP/Contents/MacOS/$BIN_NAME"

cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Humanshipd AX Probe</string>
  <key>CFBundleDisplayName</key><string>Humanshipd AX Probe</string>
  <key>CFBundleIdentifier</key><string>dev.humanshipd.axprobe</string>
  <key>CFBundleExecutable</key><string>HumanshipdProbe</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleVersion</key><string>0.0.0</string>
  <key>CFBundleShortVersionString</key><string>0.0.0</string>
  <key>LSMinimumSystemVersion</key><string>13.0</string>
  <key>LSUIElement</key><true/>
</dict>
</plist>
PLIST

# Ad-hoc sign with a stable identifier so TCC can attribute a grant to it.
codesign --force --sign - --identifier dev.humanshipd.axprobe "$APP"

echo "Built: $APP"
echo "Run:   open \"$APP\"   (grant Accessibility on first launch, then open again)"
echo "Log:   /tmp/humanshipd-axprobe.log"
