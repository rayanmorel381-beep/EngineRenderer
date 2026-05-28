#!/usr/bin/env bash
# Cross-compile the Android GPU micro-benchmarks, push the binary to the
# device connected over adb, and execute it.
#
# Usage:
#     ./benches/run_android_bench.sh
#
# Requirements:
# - rustup target aarch64-linux-android installed
# - Android NDK linker configured in .cargo/config.toml
# - adb in PATH, exactly one device connected

set -euo pipefail

TARGET="aarch64-linux-android"
BENCH_NAME="android_gpu"
DEVICE_DIR="/data/local/tmp"

cd "$(dirname "$0")/.."

echo "==> Building bench binary for ${TARGET} (release)"
ARTIFACT_JSON=$(cargo bench \
	--target "${TARGET}" \
	--bench "${BENCH_NAME}" \
	--no-run \
	--message-format=json 2>/dev/null \
	| grep -E '"executable":"[^"]*android_gpu-' \
	| tail -1)

if [ -z "${ARTIFACT_JSON}" ]; then
	echo "ERROR: could not locate bench artifact" >&2
	exit 1
fi

EXE=$(echo "${ARTIFACT_JSON}" | sed -E 's/.*"executable":"([^"]+)".*/\1/')
echo "==> Bench binary: ${EXE}"

REMOTE="${DEVICE_DIR}/$(basename "${EXE}")"

echo "==> Pushing to device"
adb push "${EXE}" "${REMOTE}" >/dev/null
adb shell chmod 755 "${REMOTE}"

echo "==> Running on device"
adb shell "${REMOTE}"
