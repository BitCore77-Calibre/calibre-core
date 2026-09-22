#!/usr/bin/env bash
# Build calibre-light for mobile targets.
# Run from repo root. Requires: rustup, cargo, and for Android an NDK.
#
# iOS targets only build on macOS with Xcode command-line tools.
# Android targets build on Linux or macOS with the NDK.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/target/mobile"
mkdir -p "$OUT/ios" "$OUT/android/jniLibs/arm64-v8a" "$OUT/android/jniLibs/armeabi-v7a" "$OUT/android/jniLibs/x86_64" "$OUT/android/jniLibs/x86"

build_target() {
    local triple="$1" dest="$2"
    echo "==> building $triple"
    rustup target add "$triple" 2>/dev/null || true
    cargo build -p calibre-light --release --target "$triple"
    cp "$ROOT/target/$triple/release/libcalibre_light.so"  "$dest/" 2>/dev/null || true
    cp "$ROOT/target/$triple/release/libcalibre_light.a"   "$dest/" 2>/dev/null || true
    cp "$ROOT/target/$triple/release/libcalibre_light.dylib" "$dest/" 2>/dev/null || true
}

# --- iOS (macOS only) ---
if [[ "$(uname)" == "Darwin" ]]; then
    build_target aarch64-apple-ios          "$OUT/ios"
    build_target aarch64-apple-ios-sim      "$OUT/ios"
    echo "iOS dylibs+a in $OUT/ios"
else
    echo "Skipping iOS (not on macOS). Ada runs this on her Mac."
fi

# --- Android (needs NDK) ---
if [[ -z "${ANDROID_NDK_HOME:-}" && -z "${ANDROID_HOME:-}" ]]; then
    echo "ERROR: set ANDROID_NDK_HOME (or ANDROID_HOME with ndk/ subdir) first." >&2
    exit 1
fi

# Resolve NDK root. ANDROID_NDK_HOME typically points at the version
# directory (e.g. .../ndk/android-ndk-r27c), so use it directly.
# Otherwise scan ANDROID_HOME/ndk for the newest version.
if [[ -n "${ANDROID_NDK_HOME:-}" ]]; then
    NDK_ROOT="$ANDROID_NDK_HOME"
else
    if [[ -z "${ANDROID_HOME:-}" ]]; then
        echo "ERROR: set ANDROID_NDK_HOME or ANDROID_HOME" >&2
        exit 1
    fi
    # Trailing slash -> ls -d matches only directories.
    NDK_ROOT="$(ls -d "${ANDROID_HOME}/ndk"/*/ 2>/dev/null | sort -V | tail -1)"
    NDK_ROOT="${NDK_ROOT%/}"
fi
if [[ -z "$NDK_ROOT" || ! -d "$NDK_ROOT" ]]; then
    echo "ERROR: resolved NDK_ROOT=$NDK_ROOT is not a directory" >&2
    exit 1
fi
echo "==> NDK_ROOT=$NDK_ROOT"
HOST_TAG="linux-x86_64"
[[ "$(uname)" == "Darwin" ]] && HOST_TAG="darwin-x86_64"
TOOLCHAIN="$NDK_ROOT/toolchains/llvm/prebuilt/$HOST_TAG"

export CC_aarch64_linux_android="$TOOLCHAIN/bin/aarch64-linux-android24-clang"
export AR_aarch64_linux_android="$TOOLCHAIN/bin/llvm-ar"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$TOOLCHAIN/bin/aarch64-linux-android24-clang"

export CC_armv7_linux_androideabi="$TOOLCHAIN/bin/armv7a-linux-androideabi24-clang"
export AR_armv7_linux_androideabi="$TOOLCHAIN/bin/llvm-ar"
export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="$TOOLCHAIN/bin/armv7a-linux-androideabi24-clang"

export CC_x86_64_linux_android="$TOOLCHAIN/bin/x86_64-linux-android24-clang"
export AR_x86_64_linux_android="$TOOLCHAIN/bin/llvm-ar"
export CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="$TOOLCHAIN/bin/x86_64-linux-android24-clang"

build_target aarch64-linux-android     "$OUT/android/jniLibs/arm64-v8a"
build_target armv7-linux-androideabi   "$OUT/android/jniLibs/armeabi-v7a"
build_target x86_64-linux-android      "$OUT/android/jniLibs/x86_64"

echo
echo "Android libs in $OUT/android/jniLibs/"
echo "iOS libs in $OUT/ios/"
echo "Bindings already in crates/calibre-light/bindings/"
