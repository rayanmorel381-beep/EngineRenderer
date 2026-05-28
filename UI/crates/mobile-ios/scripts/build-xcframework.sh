#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
WORKSPACE_DIR="$(cd "$CRATE_DIR/../../.." && pwd)"

PROFILE="${PROFILE:-release}"
DEVICE_TARGET="aarch64-apple-ios"
SIM_ARM_TARGET="aarch64-apple-ios-sim"
SIM_X86_TARGET="x86_64-apple-ios"

LIB_NAME="libruxel_mobile_ios.a"
OUT_DIR="$WORKSPACE_DIR/target/ios-xcframework"
mkdir -p "$OUT_DIR"

cd "$WORKSPACE_DIR"

for target in "$DEVICE_TARGET" "$SIM_ARM_TARGET" "$SIM_X86_TARGET"; do
    rustup target add "$target" >/dev/null 2>&1 || true
    cargo build -p ruxel-mobile-ios --target "$target" "--$PROFILE"
done

DEVICE_LIB="$WORKSPACE_DIR/target/$DEVICE_TARGET/$PROFILE/$LIB_NAME"
SIM_ARM_LIB="$WORKSPACE_DIR/target/$SIM_ARM_TARGET/$PROFILE/$LIB_NAME"
SIM_X86_LIB="$WORKSPACE_DIR/target/$SIM_X86_TARGET/$PROFILE/$LIB_NAME"

SIM_FAT="$OUT_DIR/sim/$LIB_NAME"
mkdir -p "$(dirname "$SIM_FAT")"
lipo -create "$SIM_ARM_LIB" "$SIM_X86_LIB" -output "$SIM_FAT"

XCFRAMEWORK="$OUT_DIR/EngineRenderer.xcframework"
rm -rf "$XCFRAMEWORK"
xcodebuild -create-xcframework \
    -library "$DEVICE_LIB" \
    -library "$SIM_FAT" \
    -output "$XCFRAMEWORK"

echo "XCFramework: $XCFRAMEWORK"
echo "Info.plist:  $CRATE_DIR/Info.plist"
