# Native Rapfi integration

The Vue application keeps the existing `src/ai/engine.js` protocol. In Tauri it uses Rust commands and Rapfi's standard input/output; in browsers it keeps the upstream Wasm path.

## Linux

Run `npm ci --include=dev`, then `npm run build:desktop:linux`. This rebuilds the native Rapfi binary from `~/src/rapfi`, requires Git revision `3c94c2a976f24a0dd1c5517623e9ab6fffe66bd7`, and copies the network files from that checkout. The networks submodule must be initialized. The Tauri Rust dependency graph is pinned by `src-tauri/Cargo.lock`.

The Rapfi executable and network weights are generated assets, intentionally excluded from Git. The Tauri resource directory contains the generated executable, model files and configuration files. For direct development use, run `npm run build:rapfi:linux` before `npm run tauri dev`.

The repository does not include the upstream browser Wasm artifacts (`public/build`); browser analysis needs the upstream Wasm build separately. Android ARM64 uses a native Rapfi shared library and a C++ bridge rather than a child process. Build with `npm run build:android`; the script builds and strips Rapfi, clears stale generated Android engine assets, and creates a release APK. Defaults: SDK `~/src/android-sdk`, NDK `28.2.13676358`, JDK 21; override using `ANDROID_HOME`, `ANDROID_NDK_HOME`, and `JAVA_HOME`. The Rapfi checkout and pinned revision are the same as for Linux. The Android APK embeds network weights in the Rust library; startup extracts them to application data, refreshing when the compiled asset version changes. Android ARM64 device testing passed on 2026-10-10: bundled UI loads, native Rapfi starts, and AI analysis works. Stop/restart lifecycle stress testing remains open (see `docs/TECHNICAL-DEBT.md`, TD-01). Windows/macOS cross-target builds have not been validated.
