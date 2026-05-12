# Phase 1: Native Desktop Shell — Research

**Phase:** 01-Native Desktop Shell
**Status:** Research complete
**Research Date:** 2026-05-10
**Researcher:** gsd-phase-researcher

---

## 1. Summary

Phase 1 wraps the existing SvelteKit SPA in a Tauri v2 desktop shell. No new UI features, no backend logic, no IPC commands. The scope is: initialize Tauri v2 into the existing project, switch SvelteKit from `adapter-auto` to `adapter-static` with SPA fallback, configure dev/build pipeline, scaffold capabilities, and verify the app launches and routes correctly on all platforms.

**Key finding:** This is a well-trodden path. Tauri v2's official SvelteKit guide matches our stack almost exactly (SvelteKit 2, Svelte 5, adapter-static, `ssr = false`). The main risks are: (1) the `prerender = true` interaction with `ssr = false` needs careful testing, (2) CSP configuration must accommodate both Vite HMR in dev and strict security in production, and (3) the `tauri init` interactive prompts need pre-planned answers to avoid wrong defaults. The archival `gtk_ref/Cargo.toml` confirms the Rust dependency patterns we'll need for future phases but is not modified in this phase.

**Confidence:** HIGH — all decisions are locked (D-01 through D-15), the Tauri v2 + SvelteKit integration is officially documented, and the project already has a working SPA that just needs wrapping.

---

## 2. Architectural Responsibility Map

| Requirement                         | Decision                     | Owner File                                          | Phase Plan   |
| ----------------------------------- | ---------------------------- | --------------------------------------------------- | ------------ |
| SHELL-01: Native desktop window     | D-01, D-04                   | `tauri.conf.json`                                   | 01-01        |
| SHELL-02: Full routing in webview   | D-05, D-06, D-07             | `svelte.config.js`, `+layout.ts`                    | 01-02        |
| SHELL-03: Dev and production builds | D-02, D-08, D-09, D-10, D-11 | `tauri.conf.json`, `package.json`, `vite.config.ts` | 01-01, 01-04 |
| SETT-03: Keyboard shortcuts         | Existing code, verified      | `+layout.svelte`, `MessageInput.svelte`             | 01-04        |
| CHAT-05: Enter/Ctrl+Enter send      | Existing code, verified      | `MessageInput.svelte`                               | 01-04        |
| D-12: Scaffold capability files     | D-12, D-13                   | `src-tauri/capabilities/*.json`                     | 01-03        |
| D-14: CSP (dev vs prod)             | D-14                         | `tauri.conf.json`                                   | 01-03        |
| D-15: Plugin stubs                  | D-15                         | `Cargo.toml`, `package.json`                        | 01-03        |

---

## 3. Standard Stack

### Core

| Technology               | Version       | Role                                                   | Why                                                       |
| ------------------------ | ------------- | ------------------------------------------------------ | --------------------------------------------------------- |
| Tauri v2                 | ^2.x          | Desktop shell, webview, IPC bridge                     | Locked decision — wraps SvelteKit SPA in native window    |
| @tauri-apps/cli          | ^2.x (devDep) | Build/dev CLI (`tauri dev`, `tauri build`)             | D-03: standard Tauri v2 pattern                           |
| @tauri-apps/api          | ^2.x (dep)    | Frontend API bridge (`invoke`, `convertFileSrc`, etc.) | D-03: needed for future IPC, install now as dependency    |
| @sveltejs/adapter-static | ^3.x          | SvelteKit → static SPA build output                    | D-05: Tauri requires static assets, no server             |
| Vite                     | ^8.0          | Dev server + bundler                                   | Already in project — port change only                     |
| Bun                      | 1.3.13        | Package manager, script runner                         | D-10, D-11: `bun run dev`/`bun run build` for Tauri hooks |

### Supporting

| Technology          | Role        | Why                         |
| ------------------- | ----------- | --------------------------- | ------------------------------------------- |
| Rust                | 1.95.0      | Tauri backend compilation   | Available on dev machine                    |
| cargo               | 1.95.0      | Rust package manager        | Tauri dependency resolution                 |
| tauri-plugin-fs     | ^2.x (stub) | Future filesystem access    | D-15: scaffold now, activate in later phase |
| tauri-plugin-dialog | ^2.x (stub) | Future native dialogs       | D-15: scaffold now                          |
| tauri-plugin-shell  | ^2.x (stub) | Future shell command access | D-15: scaffold now                          |
| tauri-plugin-http   | ^2.x (stub) | Future HTTP client          | D-15: scaffold now                          |
| tauri-plugin-store  | ^2.x (stub) | Future persistent KV store  | D-15: scaffold now                          |

### Alternatives Considered

| Alternative                 | Rejected Because                                                                                         |
| --------------------------- | -------------------------------------------------------------------------------------------------------- |
| Electron                    | Larger bundle, higher memory, different security model — not aligned with project's Tauri choice         |
| Tauri v1                    | v2 is current stable with better security model (capabilities), better plugin system, active development |
| adapter-node / adapter-auto | D-05 locks adapter-static; Tauri serves static files, no Node server                                     |
| SSG (full prerender)        | Dynamic routes like `chat/[id]` can't be prerendered; SPA fallback is required per D-06                  |
| pnpm/npm for Tauri hooks    | Bun is the project's package manager (bun.lock present) per D-10, D-11                                   |

---

## 4. Architecture Patterns

### System Architecture Diagram (Post-Phase 1)

```
┌─────────────────────────────────────────────────────────────┐
│                    Tauri Desktop Window                       │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │               SvelteKit SPA (webview)                    │ │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌─────────┐ │ │
│  │  │ Routes   │  │ Components│  │ AppState │  │ Types   │ │ │
│  │  │ (4 views)│  │ (Svelte 5)│  │ (runes)  │  │ (TS)    │ │ │
│  │  └──────────┘  └──────────┘  └──────────┘  └─────────┘ │ │
│  │                         │                                │ │
│  │               Mock data (Phase 1)                        │ │
│  │               Tauri IPC bridge (future phases)           │ │
│  └─────────────────────────────────────────────────────────┘ │
│                          │                                   │
│  ┌───────────────────────┴─────────────────────────────────┐ │
│  │              Tauri Rust Backend (src-tauri/)             │ │
│  │  ┌──────────────┐  ┌───────────────┐  ┌──────────────┐ │ │
│  │  │ lib.rs       │  │ tauri.conf.json│  │ capabilities │ │ │
│  │  │ (empty shell)│  │ (window config)│  │ (scaffolded) │ │ │
│  │  └──────────────┘  └───────────────┘  └──────────────┘ │ │
│  │                                                         │ │
│  │  Plugins (stubs only, no commands registered yet):      │ │
│  │  fs · dialog · shell · http · store                     │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure (After Phase 1)

```
persona/
├── .gitignore                    # ADD: src-tauri/target/, src-tauri/gen/
├── package.json                  # MODIFY: add Tauri deps + scripts
├── svelte.config.js              # MODIFY: adapter-auto → adapter-static + fallback
├── vite.config.ts                # MODIFY: port 5173 → 1420, strictPort: true
├── src/
│   ├── app.css                   # UNCHANGED (drag-region classes already present)
│   ├── routes/
│   │   ├── +layout.ts            # MODIFY: add prerender = true
│   │   ├── +layout.svelte        # UNCHANGED
│   │   ├── +page.svelte          # UNCHANGED
│   │   ├── chat/[id]/            # UNCHANGED
│   │   ├── characters/           # UNCHANGED
│   │   └── settings/             # UNCHANGED
│   └── lib/                      # UNCHANGED
├── src-tauri/                    # NEW — created by `tauri init`
│   ├── Cargo.toml                # Tauri + plugin stubs
│   ├── Cargo.lock                # Commit this
│   ├── build.rs                  # tauri_build::build()
│   ├── tauri.conf.json           # Window, CSP, dev/build config
│   ├── capabilities/             # Per-feature capability files
│   │   ├── default.json          # Core window permissions
│   │   ├── window.json           # Window management
│   │   ├── filesystem.json       # FS plugin permissions (stub)
│   │   ├── dialog.json           # Dialog plugin permissions (stub)
│   │   ├── shell.json            # Shell plugin permissions (stub)
│   │   ├── http.json             # HTTP plugin permissions (stub)
│   │   └── store.json            # Store plugin permissions (stub)
│   ├── icons/                    # App icons (generated by `tauri icon`)
│   └── src/
│       ├── main.rs               # Desktop entry point
│       └── lib.rs                # Tauri builder, plugin registration
├── gtk_ref/                      # UNCHANGED — archival reference only
└── ...
```

### Pattern Entries

**Pattern: Tauri-managed Dev Server (D-02)**

- `tauri dev` runs `beforeDevCommand` (`bun run dev`) to start Vite
- Vite serves on `http://localhost:1420` (D-09)
- Tauri loads `devUrl` into webview
- Single command: `bun tauri dev` — no separate terminal needed
- Tauri watches `src-tauri/` for Rust changes and auto-rebuilds

**Pattern: SPA with adapter-static Fallback (D-05, D-06, D-07)**

- `adapter-static({ fallback: 'index.html' })` generates a single `index.html` at the root
- All SvelteKit routes handled client-side — no server-side rendering
- `ssr = false` + `prerender = true` in root `+layout.ts` — the standard adapter-static SPA configuration
- Tauri loads `build/index.html` in production — the fallback ensures dynamic routes (`chat/[id]`) work because client-side router handles them

**Pattern: Per-Feature Capability Files (D-12, D-13)**

- Each capability file in `src-tauri/capabilities/` is a JSON file with: `identifier`, `description`, `windows`, `permissions`
- Only the `main` window exists in Phase 1
- Permissions are scaffolded but mostly empty/minimal — they'll be populated per-phase as commands are added
- All capability files in `capabilities/` directory are auto-loaded unless explicitly filtered in `tauri.conf.json`

**Pattern: Dual CSP (D-14)**

- `devCsp`: permissive — allows `localhost:1420`, `ws://localhost:*` (Vite HMR), `eval`/`unsafe-eval` (Vite needs it), `inline scripts`
- `csp` (production): strict — `default-src 'self'`, no `eval`, no inline scripts (Tauri auto-injects nonces/hashes), `connect-src ipc: http://ipc.localhost`
- Tauri auto-modifies CSP at compile time to add nonces and hashes for bundled code — we only configure the base policy

### Anti-Patterns

**Anti-Pattern: Using `adapter-auto` with Tauri**

- `adapter-auto` may select adapter-node or adapter-vercel depending on environment — wrong for Tauri which needs static files
- D-05 explicitly locks `adapter-static`

**Anti-Pattern: Prerendering all routes individually**

- Dynamic routes like `chat/[id]` can't be prerendered — they'd need every possible `id`
- The `fallback: 'index.html'` pattern handles this cleanly — it's the SPA solution

**Anti-Pattern: Opening DevTools in production**

- D-04: disable DevTools in production builds to prevent users from inspecting sensitive runtime state
- DevTools are still available in dev mode for debugging

**Anti-Pattern: Loose CSP in production**

- Allowing `eval`, `unsafe-inline`, or wildcard origins in production CSP undermines the security boundary
- Even though the app is local-first, XSS in the webview (e.g., from AI-generated markdown with `<script>` tags) could escalate to system access

**Anti-Pattern: Not committing Cargo.lock**

- Cargo.lock MUST be committed for deterministic Rust builds — Tauri docs explicitly recommend this

---

## 5. Don't Hand-Roll

| What                      | Use Instead                             | Why                                                                                                                   |
| ------------------------- | --------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| Tauri project scaffold    | `tauri init` CLI                        | D-01: generates correct `Cargo.toml`, `build.rs`, `main.rs`, `lib.rs`, `tauri.conf.json`, `capabilities/default.json` |
| App icons                 | `tauri icon` CLI                        | Generates all required sizes (png, ico, icns) from a source image                                                     |
| CSP nonce/hash injection  | Tauri's built-in compile-time injection | Tauri automatically adds nonces and hashes for bundled scripts/styles — don't try to manually compute them            |
| Webview window management | Tauri's `app.windows` config            | D-04: configure size, title, devtools via `tauri.conf.json`, not via Rust code                                        |
| SPA routing in webview    | adapter-static fallback                 | D-06: don't try to configure Tauri custom protocol redirects — the SPA fallback handles it                            |

---

## 6. Runtime State Inventory

Phase 1 adds no new runtime state. The existing `AppState` in `src/lib/stores/app.svelte.ts` continues to work unchanged with mock data. The Tauri Rust backend in Phase 1 is a minimal shell — no commands, no state, no services.

| State                 | Location                        | Owner     | Phase 1 Change                                                   |
| --------------------- | ------------------------------- | --------- | ---------------------------------------------------------------- |
| `AppState` (reactive) | `src/lib/stores/app.svelte.ts`  | SvelteKit | NONE — continues using mock data                                 |
| Window state          | `tauri.conf.json`               | Tauri     | NEW — window title/size/devtools config                          |
| Tauri app handle      | `src-tauri/src/lib.rs`          | Rust      | NEW — minimal builder, no state yet                              |
| Build config          | `tauri.conf.json`               | Tauri     | NEW — devUrl, beforeDevCommand, beforeBuildCommand, frontendDist |
| Capabilities          | `src-tauri/capabilities/*.json` | Tauri     | NEW — scaffolded, mostly empty permissions                       |

---

## 7. Common Pitfalls

### Pitfall 1: `prerender = true` + `ssr = false` interaction

**Risk:** MEDIUM
**Details:** SvelteKit's `adapter-static` docs recommend `ssr = false` for SPA mode. Adding `prerender = true` at the root layout tells the adapter to attempt prerendering, but with `ssr = false` and a fallback, it should produce a single `index.html`. However, if a page tries to use `load` functions with `prerender = true` while `ssr = false`, the load functions won't have access to Tauri APIs during build time.
**Mitigation:** Our app doesn't use `load` functions for data fetching — all data comes from `AppState` (mock data now, Tauri IPC later). The Tauri official SvelteKit guide recommends SPA mode without prerendering, but D-07 explicitly adds `prerender = true`. Test that the build produces a working `build/index.html` with the fallback. If prerendering causes issues (e.g., load functions trying to call Tauri APIs at build time), remove `prerender = true` — the fallback alone is sufficient for SPA mode.

### Pitfall 2: Vite HMR port must match `devUrl`

**Risk:** HIGH
**Details:** If Vite serves on port 1420 but `tauri.conf.json` `devUrl` points to a different port, the webview loads nothing. Both must be 1420.
**Mitigation:** Change `vite.config.ts` server.port to 1420 AND set `tauri.conf.json` build.devUrl to `http://localhost:1420`. Use `strictPort: true` to prevent Vite from silently switching ports if 1420 is occupied.

### Pitfall 3: `frontendDist` path must match adapter-static output

**Risk:** HIGH
**Details:** `adapter-static` outputs to `build/` by default. If `tauri.conf.json` `frontendDist` points to the wrong directory (e.g., `../dist`), production builds load nothing.
**Mitigation:** Set `frontendDist` to `"../build"` — this is relative to `src-tauri/tauri.conf.json`, so it points to the project root's `build/` directory. Verify by running `bun run build` and checking that `build/index.html` exists.

### Pitfall 4: CSP blocking Vite HMR in dev mode

**Risk:** HIGH
**Details:** A strict production CSP applied in dev will block Vite's WebSocket connection for HMR, causing the webview to not hot-reload. Symptoms: page loads but doesn't update on file changes.
**Mitigation:** Use the `devCsp` field in `tauri.conf.json` to set a permissive CSP for development. If `devCsp` is not set, the `csp` value applies to both dev and production. MUST set `devCsp` separately.

### Pitfall 5: Plugin registration mismatch between Rust and JS

**Risk:** MEDIUM
**Details:** If a plugin is listed in `Cargo.toml` but not registered in `lib.rs`'s builder, or if the JS `@tauri-apps/plugin-*` package is installed but the Rust plugin isn't initialized, calls will silently fail or throw runtime errors.
**Mitigation:** In Phase 1, we only scaffold — add plugins to `Cargo.toml` AND register them in `lib.rs` builder chain, AND install JS packages. Don't register any commands yet (that's future phases), but the plugin `.plugin()` calls must be present.

### Pitfall 6: `.gitignore` missing `src-tauri/target/`

**Risk:** MEDIUM
**Details:** The existing `.gitignore` has `/target/` which matches `gtk_ref/target/` but NOT `src-tauri/target/` (since it's `src-tauri/target/`, not `/target/` at root). The Rust build artifacts would be committed.
**Mitigation:** Add `src-tauri/target/` to `.gitignore`. Also add `src-tauri/gen/` (generated schema files) — these are build artifacts.

### Pitfall 7: `tauri init` overwriting existing files

**Risk:** LOW
**Details:** `tauri init` is designed to add to existing projects — it creates `src-tauri/` without touching the existing frontend. But if `src-tauri/` already exists, it may prompt or fail.
**Mitigation:** Ensure `src-tauri/` doesn't exist before running `tauri init`. The project currently doesn't have it.

### Pitfall 8: macOS code signing in development

**Risk:** LOW
**Details:** On macOS, `tauri dev` works without code signing. For production builds, you need a Developer ID certificate for notarization. Phase 1 doesn't require distribution builds — just verification that `tauri build` produces a working `.app` on the dev machine.
**Mitigation:** For Phase 1 verification, unsigned debug builds are sufficient. Code signing/notarization is a Phase 11 concern.

### Pitfall 9: Window `devtools: false` and the Cargo feature flag

**Risk:** MEDIUM
**Details:** Per Tauri v2 docs, the `devtools` config option works in debug builds by default. The `devtools` Cargo feature flag, if added, ENABLES DevTools in release builds. To DISABLE DevTools in production, just set `devtools: false` in the window config and do NOT add the `devtools` feature flag to `Cargo.toml`. Adding the feature flag would make it harder to disable DevTools.
**Mitigation:** Set `devtools: false` in window config. Do NOT add `"devtools"` to Tauri's features in `Cargo.toml`. In debug builds, DevTools remain accessible regardless of this config (which is desired for development). In release builds, DevTools will be disabled.

### Pitfall 10: `beforeDevCommand` blocking behavior

**Risk:** LOW
**Details:** By default, `beforeDevCommand` has `wait: false` — it doesn't wait for the command to finish before starting the Tauri app. If the dev server isn't ready when Tauri tries to load the URL, the webview may show an error briefly before the server is up.
**Mitigation:** Vite starts fast (typically < 500ms). The default `wait: false` is correct — Tauri will retry loading the URL. If this causes issues, use the object form: `{ "script": "bun run dev", "wait": true }` to force Tauri to wait for the dev server to exit (but this would be wrong since `vite dev` is long-running). The correct approach is to keep `wait: false` (the default) — Tauri handles the dev server lifecycle.

---

## 8. Code Examples

### 8.1 `tauri init` Expected Prompts and Answers

```
✔ What is your app name? Revelry
✔ What should the window title be? Revelry
✔ Where are your web assets located? ../build
✔ What is the url of your dev server? http://localhost:1420
✔ What is your frontend dev command? bun run dev
✔ What is your frontend build command? bun run build
```

**Note:** After `tauri init`, we'll need to manually adjust several values in `tauri.conf.json` (window size, CSP, etc.) since the interactive prompts only cover basics.

### 8.2 `tauri.conf.json` (Target Configuration)

```json
{
  "$schema": "https://raw.githubusercontent.com/tauri-apps/tauri/dev/crates/tauri-cli/schema.json",
  "productName": "Revelry",
  "version": "0.1.0",
  "identifier": "com.revelry.app",
  "build": {
    "beforeDevCommand": "bun run dev",
    "beforeBuildCommand": "bun run build",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../build"
  },
  "app": {
    "security": {
      "csp": {
        "default-src": "'self'",
        "connect-src": "ipc: http://ipc.localhost",
        "img-src": "'self' asset: http://asset.localhost blob: data:",
        "style-src": "'unsafe-inline' 'self'",
        "font-src": "'self'"
      },
      "devCsp": {
        "default-src": "'self'",
        "connect-src": "ipc: http://ipc.localhost http://localhost:1420 ws://localhost:1421",
        "img-src": "'self' asset: http://asset.localhost blob: data:",
        "style-src": "'unsafe-inline' 'self'",
        "font-src": "'self'",
        "script-src": "'self' 'unsafe-inline' 'unsafe-eval' http://localhost:1420"
      }
    },
    "windows": [
      {
        "title": "Revelry",
        "width": 1200,
        "height": 800,
        "devtools": false,
        "resizable": true
      }
    ]
  },
  "bundle": {
    "active": true,
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

**Key notes on this config:**

- `devtools: false` disables DevTools in production. In dev mode, DevTools are always available regardless of this setting.
- `devCsp` includes `unsafe-eval` and `unsafe-inline` for scripts — these are essential for dev mode HMR.
- Production `csp` does NOT include `unsafe-eval` or `unsafe-inline` for scripts. Tauri auto-injects nonces/hashes for bundled scripts at compile time.
- `style-src` includes `'unsafe-inline'` in both dev and production because Tailwind v4 generates inline styles and Svelte scoped styles are inline.
- `connect-src` includes `ipc: http://ipc.localhost` — this is required for Tauri's IPC bridge to work.
- The `identifier` uses reverse-DNS notation: `com.revelry.app`.

### 8.3 `src-tauri/Cargo.toml` (Target with Plugin Stubs)

```toml
[package]
name = "revelry"
version = "0.1.0"
description = "AI roleplay desktop application"
authors = ["you"]
edition = "2021"

[lib]
name = "revelry_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-fs = "2"
tauri-plugin-dialog = "2"
tauri-plugin-shell = "2"
tauri-plugin-http = "2"
tauri-plugin-store = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

**Note:** Version pinning should use the latest stable Tauri v2 versions at time of implementation. The `"2"` shorthand resolves to the latest 2.x.y.

### 8.4 `src-tauri/src/lib.rs` (Minimal Shell with Plugin Stubs)

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_store::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 8.5 `src-tauri/src/main.rs` (Desktop Entry Point)

```rust
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    revelry_lib::run()
}
```

### 8.6 `src-tauri/build.rs`

```rust
fn main() {
    tauri_build::build()
}
```

### 8.7 `svelte.config.js` (Modified)

```javascript
import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      fallback: "index.html",
    }),
    alias: {
      $lib: "src/lib",
      "$lib/*": "src/lib/*",
    },
  },
};

export default config;
```

### 8.8 `vite.config.ts` (Modified — Port 1420)

```typescript
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],
  server: {
    port: 1420,
    strictPort: true,
  },
});
```

**Note:** `strictPort: true` ensures Vite fails loudly if port 1420 is occupied, rather than silently switching to another port (which would break the Tauri devUrl).

### 8.9 `src/routes/+layout.ts` (Modified)

```typescript
export const ssr = false;
export const prerender = true;
```

### 8.10 `package.json` Scripts Addition

```json
{
  "scripts": {
    "tauri:dev": "tauri dev",
    "tauri:build": "tauri build",
    "tauri:preview": "tauri build && tauri dev --no-watch",
    "tauri:check": "tauri info"
  }
}
```

### 8.11 `package.json` Dependencies Addition

```json
{
  "devDependencies": {
    "@tauri-apps/cli": "^2"
  },
  "dependencies": {
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-fs": "^2",
    "@tauri-apps/plugin-dialog": "^2",
    "@tauri-apps/plugin-shell": "^2",
    "@tauri-apps/plugin-http": "^2",
    "@tauri-apps/plugin-store": "^2"
  }
}
```

### 8.12 Capability Files

**`src-tauri/capabilities/default.json`** (core permissions for the main window):

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Core permissions for the main Revelry window",
  "windows": ["main"],
  "permissions": [
    "core:path:default",
    "core:event:default",
    "core:window:default",
    "core:app:default",
    "core:resources:default",
    "core:menu:default",
    "core:tray:default"
  ]
}
```

**`src-tauri/capabilities/window.json`**:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "window",
  "description": "Window management permissions",
  "windows": ["main"],
  "permissions": [
    "core:window:allow-set-title",
    "core:window:allow-close",
    "core:window:allow-set-size",
    "core:window:allow-set-position",
    "core:window:allow-set-fullscreen",
    "core:window:allow-minimize",
    "core:window:allow-maximize",
    "core:window:allow-unminimize",
    "core:window:allow-unmaximize",
    "core:window:allow-is-maximized",
    "core:window:allow-is-minimized",
    "core:window:allow-is-fullscreen",
    "core:window:allow-start-dragging",
    "core:window:allow-toggle-maximize"
  ]
}
```

**`src-tauri/capabilities/filesystem.json`** (stub — minimal permissions):

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "filesystem",
  "description": "Filesystem access permissions — stub for future phases",
  "windows": ["main"],
  "permissions": ["fs:default"]
}
```

**`src-tauri/capabilities/dialog.json`** (stub):

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "dialog",
  "description": "Native dialog permissions — stub for future phases",
  "windows": ["main"],
  "permissions": ["dialog:default"]
}
```

**`src-tauri/capabilities/shell.json`** (stub):

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "shell",
  "description": "Shell command permissions — stub for future phases",
  "windows": ["main"],
  "permissions": ["shell:default"]
}
```

**`src-tauri/capabilities/http.json`** (stub):

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "http",
  "description": "HTTP client permissions — stub for future phases",
  "windows": ["main"],
  "permissions": ["http:default"]
}
```

**`src-tauri/capabilities/store.json`** (stub):

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "store",
  "description": "Persistent store permissions — stub for future phases",
  "windows": ["main"],
  "permissions": ["store:default"]
}
```

### 8.13 `.gitignore` Addition

```
# Tauri build artifacts
src-tauri/target/
src-tauri/gen/
```

---

## 9. State of the Art

### Tauri v2 (Stable, 2024-2025)

Tauri v2 is the current stable release with significant improvements over v1:

- **Capabilities system** replaces v1's allowlist — fine-grained, per-window, per-plugin permission scoping in `capabilities/` directory
- **Plugin architecture** is standardized — official plugins (`tauri-plugin-*`) follow consistent patterns
- **Mobile support** added (iOS/Android) — but our project is desktop-only
- **Security model** is stronger — CSP enforcement, prototype freezing, isolation pattern option
- **Build system** uses `tauri-build` consistently across all targets

### SvelteKit adapter-static SPA Pattern (2025-2026)

The officially recommended Tauri + SvelteKit pattern:

1. `adapter-static({ fallback: 'index.html' })`
2. `ssr = false` in root `+layout.ts`
3. `frontendDist: "../build"` in `tauri.conf.json`
4. No prerendering needed for SPA mode, but D-07 adds `prerender = true`

This is a well-documented, well-tested pattern. The Tauri docs explicitly cover SvelteKit.

### Tauri Plugin Ecosystem (2025-2026)

All five plugins we're stubbing are officially maintained and cross-platform:

- `tauri-plugin-fs` — filesystem access with scope restrictions
- `tauri-plugin-dialog` — native OS dialogs (open/save/message)
- `tauri-plugin-shell` — spawn processes, sidecar support
- `tauri-plugin-http` — HTTP client in Rust, exposed to frontend
- `tauri-plugin-store` — persistent key-value storage

Each has both Rust crate (`tauri-plugin-*`) and JS wrapper (`@tauri-apps/plugin-*`).

---

## 10. Assumptions Log

| ID   | Assumption                                                                                                                               | Confidence | If Wrong                                                                                                                  |
| ---- | ---------------------------------------------------------------------------------------------------------------------------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------- |
| A-01 | `tauri init` generates `src-tauri/` with `lib.rs` + `main.rs` pattern (not just `main.rs`)                                               | HIGH       | Manually create `lib.rs` per project structure docs                                                                       |
| A-02 | `adapter-static` default output directory is `build/`                                                                                    | HIGH       | Check SvelteKit docs; set custom `outDir` if different                                                                    |
| A-03 | Bun 1.3.13 supports `@tauri-apps/cli` and Tauri's `beforeDevCommand`                                                                     | HIGH       | Tauri docs list bun as supported package manager                                                                          |
| A-04 | `prerender = true` with `ssr = false` produces a working SPA build with fallback                                                         | MEDIUM     | May need to remove `prerender = true` — the fallback alone is sufficient                                                  |
| A-05 | No `load` functions in the existing app use Tauri APIs                                                                                   | HIGH       | Verified: all data comes from `AppState` / mock data                                                                      |
| A-06 | The `.drag-region` CSS class works with Tauri's default window setup                                                                     | HIGH       | Tauri respects `-webkit-app-region: drag` in its webview                                                                  |
| A-07 | DevTools are disabled in production builds when `devtools: false` is set in window config (without needing the `devtools` Cargo feature) | MEDIUM     | Test during 01-04 verification; adding `devtools` feature would ENABLE them in release, which is opposite of what we want |
| A-08 | `strictPort: true` in Vite config won't cause issues with `tauri dev`                                                                    | HIGH       | This is the recommended approach per Tauri docs                                                                           |
| A-09 | The `tauri-plugin-store` `default` permission set is safe to include as a stub                                                           | HIGH       | Default permissions are intentionally minimal per plugin design                                                           |

---

## 11. Open Questions

| ID    | Question                                                                                                                                                                 | Impact                                                                        | Resolution Path                                                                                                                                                                            |
| ----- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| OQ-01 | Should `prerender = true` be added to `+layout.ts` per D-07, or is `ssr = false` + fallback sufficient? D-07 locks it, but the Tauri SvelteKit guide doesn't include it. | Build output may differ; could cause load-function issues in future           | Test both configurations during Plan 01-02. If `prerender = true` causes issues, note the deviation from D-07.                                                                             |
| OQ-02 | What should the `identifier` (reverse-DNS) be for the Tauri app?                                                                                                         | Affects bundle ID, app data directory paths, macOS app identity               | Recommend `com.revelry.app` unless the user has a specific domain. This appears in Info.plist on macOS and determines the `~/Library/Application Support/com.revelry.app/` data directory. |
| OQ-03 | Should `tauri info` output be captured as a verification artifact?                                                                                                       | Confirms correct Rust/Node/Tauri versions on each platform                    | Include in Plan 01-04 verification steps.                                                                                                                                                  |
| OQ-04 | Does the project need app icons for Phase 1, or can we use the Tauri default icons?                                                                                      | `bundle.icon` array must have at least one entry for `tauri build` to succeed | `tauri init` generates default icons. Can also use `tauri icon` with a custom source image. Default icons are sufficient for Phase 1.                                                      |
| OQ-05 | Should `src-tauri/gen/` (auto-generated schema files) be gitignored?                                                                                                     | Build artifact — should not be committed                                      | Yes, add to `.gitignore`. Tauri regenerates these on every build.                                                                                                                          |

---

## 12. Environment Availability

| Tool           | Version       | Status         | Notes                                                                    |
| -------------- | ------------- | -------------- | ------------------------------------------------------------------------ |
| Rust           | 1.95.0        | ✅ Available   | `rustc --version` confirmed                                              |
| Cargo          | 1.95.0        | ✅ Available   | `cargo --version` confirmed                                              |
| Bun            | 1.3.13        | ✅ Available   | `bun --version` confirmed                                                |
| Node.js        | (via Bun)     | ✅ Available   | Bun includes Node.js compatibility                                       |
| Tauri CLI      | Not installed | ⚠️ To install  | `bun add -D @tauri-apps/cli@latest`                                      |
| Tauri API      | Not installed | ⚠️ To install  | `bun add @tauri-apps/api@latest`                                         |
| adapter-static | Not installed | ⚠️ To install  | `bun add -D @sveltejs/adapter-static`                                    |
| Platform       | macOS         | ✅ Dev machine | Linux/Windows verification deferred to Plan 01-04 cross-platform testing |

**Prerequisites for `tauri init`:**

1. Rust toolchain ✅
2. Bun ✅
3. `@tauri-apps/cli` installed as devDependency
4. Existing SvelteKit project with `bun install` already run

**Rust version requirement:** Tauri v2 requires Rust 1.77.2+. Our Rust 1.95.0 exceeds this. ✅

---

## 13. Validation Architecture

### Phase 1 Success Criteria (from ROADMAP.md)

1. ✅ User can launch the app as a native desktop window
2. ✅ All SvelteKit routes (home, chat, characters, settings) work inside the Tauri webview
3. ✅ App runs correctly in both `tauri dev` and production `tauri build`
4. ✅ Keyboard shortcuts (F9 sidebar, Enter/Ctrl+Enter send) still work

### Validation Plan per Plan

| Plan                             | Validation Method                                                       | Pass Criteria                                                                                  |
| -------------------------------- | ----------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| 01-01: Initialize Tauri          | `tauri dev` launches window; window shows app UI                        | Window opens with title "Revelry", size 1200×800, SvelteKit routes load                        |
| 01-02: Switch to adapter-static  | `bun run build` produces `build/index.html`; all routes work in webview | Navigate to `/`, `/chat/[id]`, `/characters`, `/settings` — all render correctly               |
| 01-03: Configure capabilities    | `tauri build` succeeds; capability files present and valid              | Build completes without permission errors; `src-tauri/capabilities/` has all 7 files           |
| 01-04: Verify dev and production | Both `tauri dev` and `tauri build` produce working app                  | F9 sidebar toggle works; Enter/Ctrl+Enter send works; routes all work; window drags via header |

### Validation Checks for Each Criterion

**SHELL-01 (Native desktop window):**

- [ ] `bun tauri dev` opens a native window (not browser)
- [ ] Window title is "Revelry"
- [ ] Window size is approximately 1200×800
- [ ] Window is resizable
- [ ] DevTools are NOT accessible in production build (`tauri build`)

**SHELL-02 (Full routing in webview):**

- [ ] Navigate to `/` — home view loads
- [ ] Navigate to `/chat/[id]` — chat view loads with conversation
- [ ] Navigate to `/characters` — character library loads
- [ ] Navigate to `/settings` — settings page loads
- [ ] Client-side navigation between routes works (no full page reload)
- [ ] Browser back/forward navigation works within webview

**SHELL-03 (Dev and production builds):**

- [ ] `bun tauri dev` starts and loads the app with HMR
- [ ] `bun tauri build` produces platform installer (macOS: `.dmg` or `.app`)
- [ ] Production build runs without console errors
- [ ] CSP doesn't break production app functionality

**SETT-03 / CHAT-05 (Keyboard shortcuts):**

- [ ] F9 toggles sidebar open/closed
- [ ] Enter sends message in chat input
- [ ] Ctrl+Enter sends message in chat input (depending on settings)
- [ ] Shift+Enter inserts newline in chat input

### Automated Validation

No test infrastructure exists yet (no `*.test.ts` files, no Vitest/Jest). For Phase 1, validation is manual — run the app and check the criteria. Future phases should add automated tests.

**`tauri check` (D-08 `tauri:check` script):**

- Runs `tauri info` which outputs environment details
- This is a diagnostic tool, not a test suite
- Useful for CI environment verification in future phases

---

## 14. Security Domain

### Threat Model for Phase 1

| Threat                                   | Mitigation                                                                       | Phase 1 Status                                                       |
| ---------------------------------------- | -------------------------------------------------------------------------------- | -------------------------------------------------------------------- |
| XSS via AI-generated content (future)    | Strict production CSP prevents inline script execution                           | CSP configured; actual markdown rendering deferred to Phase 6        |
| DevTools exposing sensitive state        | `devtools: false` in production window config                                    | Config set — need to verify it works without `devtools` feature flag |
| CSP bypass in production                 | Tauri auto-injects nonces/hashes; no `unsafe-eval` for scripts in production CSP | CSP configured per D-14                                              |
| Supply chain attack via malicious plugin | All plugins are official Tauri plugins (`@tauri-apps/plugin-*`)                  | Low risk — using official ecosystem                                  |
| WebView escape via `window.__TAURI__`    | `withGlobalTauri: false` (default) — API not exposed globally                    | Default is safe                                                      |
| Filesystem access via webview            | Capability system restricts which commands the webview can call                  | Stub permissions only; no sensitive scopes defined yet               |

### CSP Details

**Development CSP (`devCsp`):**

```
default-src 'self';
connect-src ipc: http://ipc.localhost http://localhost:1420 ws://localhost:1421;
img-src 'self' asset: http://asset.localhost blob: data:;
style-src 'unsafe-inline' 'self';
font-src 'self';
script-src 'self' 'unsafe-inline' 'unsafe-eval' http://localhost:1420;
```

- `unsafe-eval` and `unsafe-inline` for scripts are REQUIRED for Vite HMR
- `ws://localhost:1421` for Vite WebSocket HMR connection
- This is only active during `tauri dev` — never in production builds

**Production CSP (`csp`):**

```
default-src 'self';
connect-src ipc: http://ipc.localhost;
img-src 'self' asset: http://asset.localhost blob: data:;
style-src 'unsafe-inline' 'self';
font-src 'self';
```

- NO `unsafe-eval` for scripts — Tauri auto-injects nonces/hashes for bundled code
- NO `script-src` directive explicitly — Tauri adds it at compile time with hashes
- `style-src 'unsafe-inline'` is needed because Tailwind v4 generates inline styles
- `connect-src ipc: http://ipc.localhost` required for Tauri's IPC bridge

### Capability Security Boundaries

Phase 1 capabilities are intentionally minimal:

- `default.json`: Core permissions only (`core:path:default`, `core:event:default`, `core:window:default`, etc.)
- `window.json`: Window manipulation permissions (drag, resize, minimize, maximize, etc.)
- Plugin stubs (`filesystem.json`, `dialog.json`, `shell.json`, `http.json`, `store.json`): Only `*:default` permissions — these are the safe defaults defined by each plugin

**Security invariant:** No capability file grants access to user data paths, system directories, or unrestricted shell execution in Phase 1. Specific scopes will be added per-phase as needed.

---

## 15. Sources

| Source                                  | URL                                            | What It Informed                                                               |
| --------------------------------------- | ---------------------------------------------- | ------------------------------------------------------------------------------ |
| Tauri v2 — Create a Project             | https://v2.tauri.app/start/create-project/     | `tauri init` CLI usage, manual setup steps                                     |
| Tauri v2 — Configuration Reference      | https://v2.tauri.app/reference/config/         | Full `tauri.conf.json` schema, WindowConfig, SecurityConfig, CSP, capabilities |
| Tauri v2 — SvelteKit Integration        | https://v2.tauri.app/start/frontend/sveltekit/ | adapter-static config, SSR disable, frontendDist path                          |
| Tauri v2 — CSP Guide                    | https://v2.tauri.app/security/csp/             | CSP directive requirements, nonce/hash injection, dev vs production CSP        |
| Tauri v2 — Capabilities                 | https://v2.tauri.app/security/capabilities/    | Capability file format, permission entries, platform scoping                   |
| Tauri v2 — Permissions                  | https://v2.tauri.app/security/permissions/     | Permission identifier format, scope configuration                              |
| Tauri v2 — Project Structure            | https://v2.tauri.app/start/project-structure/  | `src-tauri/` layout, `lib.rs` vs `main.rs`, `capabilities/` directory          |
| Tauri v2 — Develop Guide                | https://v2.tauri.app/develop/                  | Dev/build workflow, `tauri dev`, source control guidance                       |
| Tauri v2 — Features & Plugins           | https://v2.tauri.app/plugin/                   | Official plugin list, support table, version requirements                      |
| Project: `01-CONTEXT.md`                | (local)                                        | All locked decisions D-01 through D-15                                         |
| Project: `REQUIREMENTS.md`              | (local)                                        | SHELL-01, SHELL-02, SHELL-03, SETT-03, CHAT-05                                 |
| Project: `STACK.md`                     | (local)                                        | Svelte 5, SvelteKit, Vite, Bun, Tailwind v4, Bits.ui stack details             |
| Project: `ARCHITECTURE.md`              | (local)                                        | System overview, layers, data flow, anti-patterns                              |
| Project: `STRUCTURE.md`                 | (local)                                        | Directory layout, key file locations, where to add code                        |
| Project: `CONVENTIONS.md`               | (local)                                        | Naming, code style, import organization                                        |
| Project: `01-UI-SPEC.md`                | (local)                                        | Design system baseline, drag region CSS, no UI changes                         |
| Project: `gtk_ref/Cargo.toml`           | (local)                                        | Rust dependency patterns for reference (not modified in Phase 1)               |
| Project: `svelte.config.js`             | (local)                                        | Current adapter-auto config                                                    |
| Project: `vite.config.ts`               | (local)                                        | Current port 5173 config                                                       |
| Project: `package.json`                 | (local)                                        | Current scripts and dependencies                                               |
| Project: `src/routes/+layout.ts`        | (local)                                        | Current `ssr = false`, `prerender = false`                                     |
| Project: `src/routes/+layout.svelte`    | (local)                                        | App shell with keyboard shortcuts, drag region                                 |
| Project: `src/lib/stores/app.svelte.ts` | (local)                                        | Central AppState — no changes needed                                           |
| Project: `src/app.css`                  | (local)                                        | `.drag-region` / `.no-drag` CSS classes already present                        |
| Project: `.gitignore`                   | (local)                                        | Current gitignore — needs `src-tauri/target/` and `src-tauri/gen/`             |

---

_Research completed: 2026-05-10_
