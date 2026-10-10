# Technical debt register

Scope: Tauri desktop/Android integration of Gomoku Calculator and its native Rapfi engine. Entries below are unresolved risks or costs; investigate before redesign. Priorities reflect impact, not a mandate to implement immediately.

## TD-01 — Android engine lifecycle contract (P1)

**Mechanism:** Desktop `engine_stop` kills and reaps the child process. Android `engine_stop` only sends `YXSTOP` to a process-global native engine, while `STARTED` stays true. Search cancellation and engine teardown are conflated at the public API boundary.

**Cost/risk:** Restart/reinitialization semantics differ by platform; native threads/state may survive a UI stop. The initial Android device test passed, but stop/restart sequences have not been validated.

**Direction:** Specify whether stop means cancel search or release engine. If teardown is required, design a safe native shutdown/restart path with thread ownership; otherwise make the singleton contract explicit at the API boundary.

**Closure:** Repeated start → analyze → stop → start → analyze, app background/foreground, and shutdown tests on Android and desktop; no stale output, crash, or leaked native workers.

## TD-02 — Process-global native bridge state (P1)

**Mechanism:** Android bridge relies on global C++ stream interception and process-wide state; Rust holds global app handle/start flag and changes process working directory for Rapfi assets.

**Cost/risk:** Other native components can inherit changed I/O or CWD; concurrent use and cleanup are hard to reason about. This is a risk, not an observed user-facing failure.

**Direction:** Inventory actual global side effects and Rapfi constraints. Localize file resolution/output routing and establish explicit ownership only where practical.

**Closure:** Demonstrate isolated resource lookup, predictable output routing, and safe initialization/shutdown without unintended process-wide effects.

## TD-03 — Reproducible Rapfi source and native artifact provenance (P1)

**Mechanism:** Builds depend on a separately maintained `~/src/rapfi` checkout, its networks submodule, and generated Android `.so`/desktop executable. The expected revision is documented but the external checkout and toolchain are not fully provisioned by the repository.

**Cost/risk:** Clean machines and new agents may produce different binaries or fail without reconstructing implicit local state.

**Direction:** Establish a reproducible source-fetch/verification and artifact-generation contract (pinned revision/submodule, toolchain versions, architecture flags); avoid committing generated binaries unless justified.

**Closure:** Fresh checkout on a clean machine builds equivalent native assets with documented commands and verifiable input revisions.

## TD-04 — Android installed-data duplication (P2; measure first)

**Mechanism:** Network weights are embedded in the Rust shared library and extracted into app data on first engine start. Installed storage contains both embedded and extracted copies.

**Cost/risk:** Extra device storage and first-run writes for roughly 40 MB of weights. This may be an acceptable tradeoff because Rapfi expects file paths.

**Direction:** Measure actual installed size, startup time, and alternatives (asset file access or native loader changes) before committing to an architectural change.

**Closure:** Either show a lower-cost solution with comparable reliability, or document the duplication as an intentional constraint and retire this entry.

## TD-05 — Linux Wayland WebKit compatibility workaround (P2)

**Mechanism:** The desktop launcher forces `WEBKIT_DISABLE_DMABUF_RENDERER=1` when it detects Wayland. This mitigates a machine-observed startup/rendering issue but disables a rendering path for all Wayland users.

**Cost/risk:** Potential performance loss and an unbounded compatibility exception as WebKit/driver behavior evolves. Earlier tests also saw intermittent WebKitWebProcess hangs; do not assume the workaround fixes those.

**Direction:** Reproduce against current WebKitGTK/GPU combinations; scope the workaround to affected configurations or remove it when no longer needed.

**Closure:** Stable Wayland startup and sustained interaction without unnecessary renderer fallback on unaffected systems.

## Closed findings / not debt

- **Android runtime verification:** Real ARM64 phone installation, UI, native Rapfi and AI analysis verified on 2026-10-10. Reopen only for specific untested lifecycle cases (TD-01).
- **Android local-page refusal:** Production frontend forced `http://tauri.localhost` to HTTPS; fixed by excluding `tauri.localhost` from website HTTPS redirect, verified on device.
- **Stale Android generated assets:** Android build entrypoint prunes the known generated engine asset directory; APK inspection found zero stale engine assets.
- **Same-size weight update:** Embedded asset version now tracks content changes; startup refreshes extracted weights on version change.
- **Build entrypoint and docs:** `npm run build:android` and `NATIVE.md` cover current workflow and configurable SDK/NDK/JDK defaults.
- **NEON DotProd variants / further binary shrinking:** Optimization ideas, not demonstrated technical debt without benchmarks or meaningful avoidable cost.
