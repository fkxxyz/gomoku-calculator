#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
export ANDROID_HOME="${ANDROID_HOME:-$HOME/src/android-sdk}"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ANDROID_HOME/ndk/28.2.13676358}"
export JAVA_HOME="${JAVA_HOME:-/usr/lib/jvm/java-21-openjdk}"
"$root/scripts/build-rapfi-android.sh"
# Tauri's Android resource copy does not prune assets removed from the bundle config.
# Only remove this known generated directory, never arbitrary application assets.
rm -rf "$root/src-tauri/gen/android/app/src/main/assets/resources/engine"
cd "$root"
npm run tauri android build -- --apk --target aarch64 "$@"
