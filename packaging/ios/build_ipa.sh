#!/usr/bin/env bash
set -euo pipefail

TARGET="aarch64-apple-ios"
PROFILE="${PROFILE:-release}"
CARGO_FLAGS="${CARGO_FLAGS:-}"
OUT_DIR="target/ios"

echo "=== Building horizon-client for $TARGET ($PROFILE) ==="
cargo build --target "$TARGET" --$PROFILE -p horizon-client --no-default-features --features audio,controllers $CARGO_FLAGS

APP_DIR="$OUT_DIR/Payload/Horizon.app"
rm -rf "$OUT_DIR"
mkdir -p "$APP_DIR"

BIN_PATH="target/$TARGET/$PROFILE/horizon-client"
echo "=== Packaging $BIN_PATH into $APP_DIR ==="
cp "$BIN_PATH" "$APP_DIR/Horizon"
chmod +x "$APP_DIR/Horizon"

cp packaging/ios/Info.plist "$APP_DIR/Info.plist"

if [ -f "packaging/icons/hicolor/512x512/apps/horizon.png" ]; then
    cp packaging/icons/hicolor/512x512/apps/horizon.png "$APP_DIR/AppIcon.png"
fi

if command -v codesign &>/dev/null; then
    echo "=== Ad-hoc signing Horizon.app ==="
    codesign -s - --force --deep "$APP_DIR" || true
fi

echo "=== Creating Horizon.ipa ==="
(cd "$OUT_DIR" && zip -r -9 "Horizon.ipa" Payload)

echo "=== Ready: $OUT_DIR/Horizon.ipa ==="
