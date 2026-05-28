#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
WORKSPACE_DIR="$(cd "$CRATE_DIR/../../.." && pwd)"

PROFILE="${PROFILE:-release}"
ANDROID_API="${ANDROID_API:-24}"
TARGETS=("aarch64-linux-android" "x86_64-linux-android")

if ! command -v cargo-ndk >/dev/null 2>&1; then
    echo "cargo-ndk not found. Install it with: cargo install cargo-ndk" >&2
    exit 1
fi

OUT_BASE="$WORKSPACE_DIR/target/android-libs"
mkdir -p "$OUT_BASE"

cd "$WORKSPACE_DIR"
cargo ndk \
    -p "ruxel-mobile-android" \
    --platform "$ANDROID_API" \
    $(printf -- "-t %s " "${TARGETS[@]}") \
    --output-dir "$OUT_BASE" \
    build "--$PROFILE"

echo "Native libraries copied to: $OUT_BASE"
echo "Manifest: $CRATE_DIR/AndroidManifest.xml"
