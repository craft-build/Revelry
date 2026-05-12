---
phase: 01-native-desktop-shell
plan: 01
subsystem: infra
tags: [tauri, sveltekit, adapter-static, desktop, vite, cargo]

requires: []
provides:
  - "Tauri v2 desktop shell wrapping SvelteKit SPA"
  - "adapter-static with fallback for SPA client-side routing"
  - "Vite dev server on port 1420 aligned with Tauri devUrl"
  - "Rust backend scaffold with 5 Tauri plugin stubs registered"
  - "CSP configuration (strict production, permissive dev)"
  - "Tauri dev/build/preview/check scripts in package.json"
affects: [02-persistent-data, 03-provider-setup, all subsequent phases]

tech-stack:
  added:
    [
      "@tauri-apps/cli@2.11.1",
      "@tauri-apps/api@2.11.0",
      "@sveltejs/adapter-static@3.0.10",
      "@tauri-apps/plugin-fs@2.5.1",
      "@tauri-apps/plugin-dialog@2.7.1",
      "@tauri-apps/plugin-shell@2.3.5",
      "@tauri-apps/plugin-http@2.5.9",
      "@tauri-apps/plugin-store@2.4.3",
      "tauri@2",
      "tauri-plugin-fs@2",
      "tauri-plugin-dialog@2",
      "tauri-plugin-shell@2",
      "tauri-plugin-http@2",
      "tauri-plugin-store@2",
      "tauri-build@2",
      "serde@1",
      "serde_json@1",
    ]
  patterns:
    [
      Tauri v2 plugin registration,
      adapter-static SPA with fallback,
      dual CSP config,
    ]

key-files:
  created:
    [
      "src-tauri/Cargo.toml",
      "src-tauri/tauri.conf.json",
      "src-tauri/src/lib.rs",
      "src-tauri/src/main.rs",
      "src-tauri/build.rs",
      "src-tauri/capabilities/default.json",
    ]
  modified:
    [
      "package.json",
      "svelte.config.js",
      "vite.config.ts",
      "src/routes/+layout.ts",
      ".gitignore",
    ]

key-decisions:
  - "adapter-static with fallback: 'index.html' for SPA routing"
  - "prerender.handleUnseenRoutes: 'ignore' for dynamic routes like /chat/[id]"
  - "tauri-plugin-store uses Builder::default().build() not init() (v2 API)"
  - "Port 1420 with strictPort: true for Tauri alignment"

patterns-established:
  - "Plugin registration triangle: Cargo.toml deps + lib.rs .plugin() + package.json @tauri-apps/plugin-* must all match"
  - "CSP dual config: strict production CSP + permissive devCsp for HMR"
  - "SPA prerender pattern: ssr=false + prerender=true + handleUnseenRoutes=ignore"

requirements-completed: [SHELL-01, SHELL-02, SHELL-03]

duration: 45min
completed: 2026-05-10
---

# Phase 1: Native Desktop Shell Summary — Plan 01-01

**Tauri v2 desktop shell bootstrapped with adapter-static SPA, port-aligned Vite dev server, and 5 plugin stubs registered**

## Performance

- **Duration:** ~45 min
- **Tasks:** 12 (11 from plan + 1 fix commit)
- **Files modified:** ~15

## Accomplishments

- Full Tauri v2 scaffold (src-tauri/) with Rust backend and SvelteKit frontend
- SvelteKit switched from adapter-auto to adapter-static with `fallback: 'index.html'`
- Vite dev server port aligned to 1420 with strictPort for Tauri devUrl
- 5 Tauri plugin stubs registered in Cargo.toml, lib.rs, and package.json
- CSP configured with strict production and permissive dev (HMR-friendly)
- App successfully launches as native desktop window via `bun tauri dev`

## Task Commits

1. **Task 01: Install Tauri and adapter-static dependencies** - `e246395` (feat)
2. **Task 02: Switch SvelteKit to adapter-static** - `943d7b6` (feat)
3. **Task 03: Set prerender = true in +layout.ts** - `fa84fd0` (feat)
4. **Task 04: Change Vite port to 1420** - `18cccc1` (feat)
5. **Task 05: Run tauri init** - `c14bd31` (feat)
6. **Task 06: Configure tauri.conf.json** - `97606e3` (feat)
7. **Task 07: Configure Cargo.toml** - `9d1ee49` (feat)
8. **Task 08: Write lib.rs with plugins** - `759da53` (feat)
9. **Task 09: Write main.rs** - `548c6ce` (feat)
10. **Task 10: Write build.rs** - `ffd242c` (feat)
11. **Task 11: Add Tauri artifacts to .gitignore** - `47dcf5b` (feat)
12. **Fix: Prerender SPA build + store plugin API** - `0258561` (fix)

## Files Created/Modified

- `package.json` — Added Tauri deps, plugin packages, and tauri:\* scripts; removed adapter-auto
- `svelte.config.js` — Switched to adapter-static with fallback; added prerender.handleUnseenRoutes
- `vite.config.ts` — Port 1420, strictPort: true
- `src/routes/+layout.ts` — Added `prerender = true`
- `src-tauri/Cargo.toml` — Package config with 5 plugin crates, lib target, serde
- `src-tauri/tauri.conf.json` — Full config: identifier, window props, CSP, bundle
- `src-tauri/src/lib.rs` — Plugin registration (fs, dialog, shell, http, store)
- `src-tauri/src/main.rs` — Entry point with Windows subsystem attribute
- `src-tauri/build.rs` — tauri_build::build()
- `src-tauri/capabilities/default.json` — Core permissions (7 core:default entries)
- `.gitignore` — Added src-tauri/target/, src-tauri/gen/, build/

## Decisions Made

- Used `handleUnseenRoutes: "ignore"` in svelte.config.js (not a layout export) because SvelteKit's prerender config lives in kit config, not route modules
- Used `tauri_plugin_store::Builder::default().build()` instead of `init()` — the store plugin v2 API uses Builder pattern

## Deviations from Plan

### Auto-fixed Issues

**1. Prerender build failure for dynamic route /chat/[id]**

- **Found during:** Task 12 (verification — `bun run build`)
- **Issue:** `prerender = true` at root layout caused SvelteKit to fail on dynamic routes not discoverable by crawling
- **Fix:** Added `kit.prerender.handleUnseenRoutes: "ignore"` to svelte.config.js; reverted invalid `handleUnseenRoutes` export from +layout.ts
- **Files modified:** svelte.config.js, src/routes/+layout.ts
- **Verification:** `bun run build` succeeds, build/index.html exists

**2. tauri_plugin_store::init() not found**

- **Found during:** Task 12 (verification — `cargo check`)
- **Issue:** Store plugin v2 API doesn't expose top-level `init()`, uses Builder pattern
- **Fix:** Changed to `tauri_plugin_store::Builder::default().build()`
- **Files modified:** src-tauri/src/lib.rs
- **Verification:** `cargo check` passes

**3. default.json capabilities had plugin permissions mixed in**

- **Found during:** Review of staged changes
- **Issue:** tauri init scaffold put all permissions in default.json; plan 01-02 requires per-feature files
- **Fix:** Updated default.json to only have 7 core:\*:default permissions
- **Files modified:** src-tauri/capabilities/default.json
- **Verification:** cargo check passes

---

**Total deviations:** 3 auto-fixed
**Impact on plan:** All fixes necessary for build correctness. No scope creep.

## Issues Encountered

- SvelteKit v2 prerender doesn't support `handleUnseenRoutes` as a layout export — must be in svelte.config.js `kit.prerender` config
- tauri-plugin-store v2.4.x uses `Builder::default().build()` instead of `init()`

## Next Phase Readiness

- Tauri shell fully operational — `bun tauri dev` launches native window with SvelteKit SPA
- Ready for Plan 01-02 (capabilities) and Wave 2 (verification plans 01-03, 01-04)

---

_Phase: 01-native-desktop-shell_
_Completed: 2026-05-10_
