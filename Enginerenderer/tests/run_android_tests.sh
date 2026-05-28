#!/usr/bin/env bash
# Cross-compile the Android GPU backend integration tests, push the binary to
# the device connected over adb, and execute it.
#
# Usage:
#     ./tests/run_android_tests.sh                     # all tests
#     ./tests/run_android_tests.sh full_frame          # filter
#
# Requirements:
# - rustup target aarch64-linux-android installed
# - Android NDK linker configured in .cargo/config.toml
# - adb in PATH, exactly one device connected

set -euo pipefail

TARGET="aarch64-linux-android"
TEST_NAME="android_gpu_backend"
DEVICE_DIR="/data/local/tmp"
FILTER="${1:-}"

cd "$(dirname "$0")/.."

echo "==> Building test binary for ${TARGET}"
ARTIFACT_JSON=$(cargo test \
	--target "${TARGET}" \
	--test "${TEST_NAME}" \
	--no-run \
	--message-format=json 2>/dev/null \
	| grep -E '"profile":\{[^}]*"test":true' \
	| tail -1)

if [ -z "${ARTIFACT_JSON}" ]; then
	echo "ERROR: could not locate test artifact" >&2
	exit 1
fi

EXE=$(echo "${ARTIFACT_JSON}" | sed -E 's/.*"executable":"([^"]+)".*/\1/')
echo "==> Test binary: ${EXE}"

REMOTE="${DEVICE_DIR}/$(basename "${EXE}")"

echo "==> Pushing to device"
adb push "${EXE}" "${REMOTE}" >/dev/null
adb shell chmod 755 "${REMOTE}"

echo "==> Running on device"
if [ -n "${FILTER}" ]; then
	adb shell "${REMOTE}" "${FILTER}"
else
	adb shell "${REMOTE}"
fi
