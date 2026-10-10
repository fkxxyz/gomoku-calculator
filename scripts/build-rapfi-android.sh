#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
rapfi="${RAPFI_SOURCE:-$HOME/src/rapfi}"
sdk="${ANDROID_HOME:-$HOME/src/android-sdk}"
ndk="${ANDROID_NDK_HOME:-$sdk/ndk/28.2.13676358}"
abi="${1:-arm64-v8a}"
expected=3c94c2a976f24a0dd1c5517623e9ab6fffe66bd7
[[ "$(git -C "$rapfi" rev-parse HEAD)" == "$expected" ]] || { echo 'Rapfi revision mismatch' >&2; exit 1; }
overlay="$root/src-tauri/android-rapfi/source"
mkdir -p "$overlay"
for entry in "$rapfi/Rapfi"/*; do
  name="${entry##*/}"
  [[ "$name" == CMakeLists.txt ]] || ln -sfn "$entry" "$overlay/$name"
done
sed -e 's/add_executable(rapfi/add_library(rapfi SHARED/' -e 's/target_link_libraries(rapfi PRIVATE pthread)/target_link_libraries(rapfi PRIVATE)/' -e 's/target_link_libraries(rapfi PRIVATE pthread atomic)/target_link_libraries(rapfi PRIVATE atomic)/' "$rapfi/Rapfi/CMakeLists.txt" > "$overlay/CMakeLists.txt"
cat >> "$overlay/CMakeLists.txt" <<'CMAKE'
target_sources(rapfi PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}/../bridge.cpp")
set_target_properties(rapfi PROPERTIES OUTPUT_NAME "rapfi")
CMAKE
cmake -S "$overlay" -B "$root/src-tauri/android-rapfi/build-$abi" \
  -DCMAKE_TOOLCHAIN_FILE="$ndk/build/cmake/android.toolchain.cmake" \
  -DANDROID_ABI="$abi" -DANDROID_PLATFORM=android-24 \
  -DCMAKE_BUILD_TYPE=Release -DENABLE_LTO=ON \
  -DUSE_NEON=ON -DNO_COMMAND_MODULES=ON
cmake --build "$root/src-tauri/android-rapfi/build-$abi" --parallel

libs="$root/src-tauri/gen/android/app/src/main/jniLibs/$abi"
mkdir -p "$libs"
cp "$root/src-tauri/android-rapfi/build-$abi/librapfi.so" "$libs/"
"$ndk/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-strip" --strip-debug "$libs/librapfi.so"
