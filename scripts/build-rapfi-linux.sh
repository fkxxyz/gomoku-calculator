#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
rapfi="${RAPFI_SOURCE:-$HOME/src/rapfi}"
expected=3c94c2a976f24a0dd1c5517623e9ab6fffe66bd7
actual="$(git -C "$rapfi" rev-parse HEAD)"
if [[ "$actual" != "$expected" ]]; then echo "Rapfi revision mismatch: $actual (expected $expected)" >&2; exit 1; fi
cmake -S "$rapfi/Rapfi" -B "$root/src-tauri/rapfi-build" -DCMAKE_BUILD_TYPE=Release -DENABLE_LTO=ON
cmake --build "$root/src-tauri/rapfi-build" --parallel
out="$root/src-tauri/resources/engine"
mkdir -p "$out"
cp "$root/src-tauri/rapfi-build/pbrain-rapfi" "$out/rapfi"
cp "$rapfi/Networks/mix9svq/"*.lz4 "$out/"
cp "$rapfi/Networks/classical/model210901.bin" "$out/"
cp "$rapfi/Networks/config-example/gomocalc-mix9svq.toml" "$out/config.toml"
cp "$rapfi/Networks/config-example/gomocalc-classical220723.toml" "$out/classical220723.toml"
cp "$rapfi/Networks/config-example/gomocalc-classical210901.toml" "$out/classical210901.toml"
