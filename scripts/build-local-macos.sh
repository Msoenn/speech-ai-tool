#!/usr/bin/env bash
#
# Build the macOS app locally and code-sign it with a STABLE self-signed
# identity so that macOS TCC grants (Accessibility for the global hotkey +
# auto-paste, and Microphone) persist across rebuilds.
#
# Why this exists: CI/GitHub releases are only ad-hoc signed, so their code
# identity (cdhash) changes on every build and macOS silently invalidates the
# Accessibility grant after each update. Signing every LOCAL build with one
# fixed certificate gives a stable Designated Requirement, so you grant
# Accessibility once and it sticks.
#
# One-time setup (creates the cert) is documented in docs/macos-local-signing.md.
# This script only *uses* an identity that already exists in your keychain.
#
# Usage:
#   scripts/build-local-macos.sh            # build, sign, leave bundle in target/
#   scripts/build-local-macos.sh --install  # also copy the app into /Applications
set -euo pipefail

IDENTITY="${SAT_SIGNING_IDENTITY:-Speech AI Tool Local Signing}"
BUNDLE_ID="com.speech-ai-tool.app"

cd "$(dirname "$0")/.."
ROOT="$(pwd)"
ENTITLEMENTS="$ROOT/src-tauri/Entitlements.plist"

# Fail early with a clear message if the signing identity is missing.
if ! security find-identity -p codesigning | grep -q "$IDENTITY"; then
  echo "error: code-signing identity '$IDENTITY' not found in your keychain." >&2
  echo "       Run the one-time setup in docs/macos-local-signing.md first," >&2
  echo "       or set SAT_SIGNING_IDENTITY to an identity you do have." >&2
  exit 1
fi

echo "==> Building (pnpm tauri build)…"
# --bundles app: build only the .app (skip the .dmg). The dmg step relocates
#   the .app into the disk image and cleans bundle/macos, leaving nothing to
#   sign here; the .app is all a local install needs.
# createUpdaterArtifacts=false: updater artifacts require the CI-only updater
#   signing key (TAURI_SIGNING_PRIVATE_KEY), which a local install doesn't need.
pnpm tauri build --bundles app --config '{"bundle":{"createUpdaterArtifacts":false}}'

APP="$ROOT/src-tauri/target/release/bundle/macos/Speech AI Tool.app"
if [ ! -d "$APP" ]; then
  # Fall back to a universal-target layout if that's how it was built.
  APP="$(/usr/bin/find "$ROOT/src-tauri/target" -maxdepth 4 -name 'Speech AI Tool.app' -path '*/bundle/macos/*' 2>/dev/null | head -1)"
fi
[ -d "$APP" ] || { echo "error: could not find built .app bundle" >&2; exit 1; }

echo "==> Signing $APP with '$IDENTITY'…"
codesign --force --sign "$IDENTITY" \
  --identifier "$BUNDLE_ID" \
  --entitlements "$ENTITLEMENTS" \
  "$APP"
codesign --verify --strict --verbose=2 "$APP"
echo "==> Designated Requirement (TCC pins the grant to this):"
codesign -d -r- "$APP" 2>&1 | grep -i designated || true

if [ "${1:-}" = "--install" ]; then
  echo "==> Installing to /Applications…"
  pkill -f "Speech AI Tool" 2>/dev/null || true
  sleep 1
  rm -rf "/Applications/Speech AI Tool.app"
  cp -R "$APP" "/Applications/Speech AI Tool.app"
  echo "==> Installed. Launch it from /Applications."
fi

echo "==> Done."
