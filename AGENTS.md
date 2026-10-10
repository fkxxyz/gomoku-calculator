# Gomoku Calculator — Agent Guide

## What This Project Does

Gomoku analysis UI using Rapfi. The Vue frontend runs in browsers with upstream Wasm or in Tauri desktop/Android apps with native Rapfi.

## Architecture and workflow

- `src/ai/engine.js` selects Tauri commands/events for native analysis and the existing worker/Wasm protocol for browsers.
- `src-tauri/src/lib.rs` owns desktop Rapfi subprocess integration and Android native bridge entry points. `src-tauri/android-rapfi/bridge.cpp` adapts Rapfi for Android; upstream Rapfi sources remain external and unmodified.
- `scripts/build-rapfi-linux.sh` and `scripts/build-rapfi-android.sh` build native engines from the pinned external Rapfi checkout. See `NATIVE.md` for revision, required assets, and build prerequisites.
- `npm run build:desktop:linux` builds Linux desktop; `npm run build:android` builds ARM64 Android release APK and removes obsolete generated engine assets.
- Android embeds Rapfi weights in the Rust library and extracts them to app data on first use or embedded asset-version change. Desktop packages engine files as Tauri resources.
- Preserve website HTTPS redirects while excluding `tauri.localhost`: Android Tauri serves bundled UI from HTTP and forcing HTTPS causes `ERR_CONNECTION_REFUSED`.

## Known debt

**Authoritative register:** [docs/TECHNICAL-DEBT.md](docs/TECHNICAL-DEBT.md). Read it before changing native lifecycle, global state, build provenance, resource storage, or Wayland behavior. Keep evidence and closure criteria in the register, not duplicated here.

- TD-01: Android engine stop/restart lifecycle
- TD-02: Process-global native bridge side effects
- TD-03: Rapfi build reproducibility and provenance
- TD-04: Android installed weight duplication (measurement pending)
- TD-05: Wayland WebKit renderer workaround

## Verification boundaries

- Building an APK does not prove Android runtime behavior. Android UI, Rapfi initialization and AI analysis were verified on a connected ARM64 device on 2026-10-10; lifecycle stress tests remain outstanding.
- For Android page-load failures, inspect ADB WebView interception logs before changing Tauri URL or asset settings. `http://tauri.localhost/` is the expected packaged URL.
- `src-tauri/gen/android` contains generated Android integration; changes there may be overwritten by Tauri regeneration. Prefer durable source/build configuration for fixes.
