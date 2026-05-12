# Phase 1: Native Desktop Shell — Pattern Mapping

**Generated:** 2026-05-10
**Phase:** 01-Native Desktop Shell
**Input:** 01-CONTEXT.md, 01-RESEARCH.md, ARCHITECTURE.md, STRUCTURE.md, CONVENTIONS.md

---

## 1. File Inventory

| #   | File                                     | Action | Role                                | Data Flow                              |
| --- | ---------------------------------------- | ------ | ----------------------------------- | -------------------------------------- |
| F1  | `src-tauri/Cargo.toml`                   | NEW    | Rust dependency manifest            | Build input → cargo resolver           |
| F2  | `src-tauri/tauri.conf.json`              | NEW    | Tauri runtime config                | Build input → tauri-build → bundle     |
| F3  | `src-tauri/src/lib.rs`                   | NEW    | Tauri builder + plugin registration | App entry → runtime init               |
| F4  | `src-tauri/src/main.rs`                  | NEW    | Desktop process entry point         | OS → main → lib::run                   |
| F5  | `src-tauri/build.rs`                     | NEW    | Tauri build script                  | cargo build → tauri_build::build()     |
| F6  | `src-tauri/capabilities/default.json`    | NEW    | Core capability permissions         | Runtime → permission check             |
| F7  | `src-tauri/capabilities/window.json`     | NEW    | Window management permissions       | Runtime → permission check             |
| F8  | `src-tauri/capabilities/filesystem.json` | NEW    | FS permissions (stub)               | Runtime → permission check             |
| F9  | `src-tauri/capabilities/dialog.json`     | NEW    | Dialog permissions (stub)           | Runtime → permission check             |
| F10 | `src-tauri/capabilities/shell.json`      | NEW    | Shell permissions (stub)            | Runtime → permission check             |
| F11 | `src-tauri/capabilities/http.json`       | NEW    | HTTP permissions (stub)             | Runtime → permission check             |
| F12 | `src-tauri/capabilities/store.json`      | NEW    | Store permissions (stub)            | Runtime → permission check             |
| F13 | `src-tauri/icons/*`                      | NEW    | App icons                           | Build → bundle resources               |
| F14 | `svelte.config.js`                       | MODIFY | SvelteKit adapter config            | Build → adapter-static → static output |
| F15 | `vite.config.ts`                         | MODIFY | Vite dev server config              | Dev server → Tauri webview URL         |
| F16 | `package.json`                           | MODIFY | JS dependency manifest + scripts    | Build/dev toolchain                    |
| F17 | `src/routes/+layout.ts`                  | MODIFY | SvelteKit layout config             | SvelteKit adapter → prerender behavior |
| F18 | `.gitignore`                             | MODIFY | VCS ignore rules                    | Git → exclude Rust build artifacts     |

---

## 2. Pattern Mapping by File

### F1 — `src-tauri/Cargo.toml` (NEW)

**Role:** Rust dependency manifest — declares the Tauri runtime, plugin stubs, and serde for IPC serialization.

**Closest codebase analog:** `gtk_ref/Cargo.toml`

**Analog excerpt:**

```gtk_ref/Cargo.toml#L1-5
[package]
name = "revelry"
version = "0.1.0"
edition = "2021"
```

**Key patterns from analog:**

- Same `name = "revelry"`, `version = "0.1.0"`, `edition = "2021"` — project identity carries over
- `serde` with `derive` feature appears in both — needed for IPC command argument/result serialization in Tauri just as it was for domain model serialization in gtk_ref
- `serde_json` appears in both — Tauri IPC uses JSON serialization under the hood
- `[build-dependencies]` section exists in both — `glib-build-tools` + `relm4-icons-build` in gtk_ref maps to `tauri-build` in the new file

**Divergences from analog:**

- **No UI crate dependencies** — gtk_ref uses `gtk4`, `libadwaita`, `relm4`; Tauri uses the system webview instead
- **No AI/DB/Security crates yet** — `rig-core`, `rusqlite`, `aes-gcm`, `argon2`, `keyring` deferred to later phases
- **No `tokio`/`futures`** — Tauri v2 uses its own async runtime; explicit tokio not needed for the shell phase
- **Plugin dependency pattern** — Tauri v2 uses `tauri-plugin-*` crates (fs, dialog, shell, http, store) instead of gtk_ref's direct library dependencies
- **`[lib]` section** — Tauri requires `crate-type = ["staticlib", "cdylib", "rlib"]` for cross-platform linking; gtk_ref has no `[lib]` section (binary-only)

**Target pattern:**

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

**Conventions applied:**

- Version shorthand `"2"` resolves to latest 2.x.y — matches gtk_ref's approach of pinning major version and letting Cargo resolve (e.g., `rig-core = "0.36.0"` was full pin; Tauri ecosystem convention is major-only for v2)
- `serde` version `"1"` — major-only pinning, consistent with Tauri examples and gtk_ref's `serde = "1.0.228"` (which is also effectively 1.x)

---

### F2 — `src-tauri/tauri.conf.json` (NEW)

**Role:** Tauri runtime configuration — window properties, CSP, dev/build commands, frontend dist path.

**Closest codebase analog:** No direct analog exists. The closest conceptual analog is `vite.config.ts` — both configure the dev server (port) and build output.

**Informed by existing pattern:**

```vite.config.ts#L1-11
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	server: {
		port: 5173,
		strictPort: false
	}
});
```

**Key cross-references from existing code:**

- `vite.config.ts` port `5173` → `tauri.conf.json` `devUrl` must match the **modified** port `1420`
- `package.json` scripts `"dev": "vite dev"` and `"build": "vite build"` → `beforeDevCommand`/`beforeBuildCommand` use the same commands (via `bun run dev` / `bun run build`)
- `svelte.config.js` adapter-static `fallback: 'index.html'` → `frontendDist: "../build"` points to adapter-static's output directory
- The `+layout.ts` `ssr = false` + `prerender = true` pair ensures adapter-static generates the SPA-compatible output that `frontendDist` references

**Target pattern (full config from RESEARCH §8.2):**

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
      "csp": { ... },
      "devCsp": { ... }
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
    "icon": [ ... ]
  }
}
```

**Conventions applied:**

- `identifier` uses reverse-DNS: `com.revelry.app` — standard Tauri convention, mirrors how `package.json` uses `"name": "revelry-ui"`
- Window size 1200×800 from D-04 — matches desktop-class design spec dimensions
- `devtools: false` from D-04 — production security; dev mode ignores this setting

---

### F3 — `src-tauri/src/lib.rs` (NEW)

**Role:** Tauri builder — registers plugins, launches application. This is the structural equivalent of gtk_ref's `app.rs` initialization.

**Closest codebase analog:** `gtk_ref/src/app.rs` (Relm4 component init) — both are the "wire everything together" entry points.

**Analog pattern (from ARCHITECTURE.md):**

> `AppModel` init: creates GTK window, initializes DB worker, loads providers/characters/conversations from SQLite, sets up component controllers

**Key pattern mapping:**
| gtk_ref init step | Tauri equivalent | Phase 1 scope |
|---|---|---|
| Create GTK window | `tauri.conf.json` windows config | ✅ (config, not code) |
| Init DB worker | — | ❌ Deferred |
| Load providers/secrets | — | ❌ Deferred |
| Set up UI controllers | — | ❌ Not needed (webview) |
| Register services | `Builder::plugin()` | ✅ Plugin stubs only |

**Target pattern:**

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

**Conventions applied:**

- `#[cfg_attr(mobile, tauri::mobile_entry_point)]` — Tauri v2 standard attribute for cross-platform support; mirrors gtk_ref's `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` in main.rs as a platform-conditional attribute
- `.expect()` with message — matches gtk_ref's error handling style (uses `anyhow`/`thiserror` in services; `expect` at process boundaries)
- Plugin registration order: fs → dialog → shell → http → store — alphabetical grouping by capability domain, matching the capability file naming convention
- Future `.invoke_handler()` registration will follow the same builder chain pattern

---

### F4 — `src-tauri/src/main.rs` (NEW)

**Role:** Desktop process entry point — minimal wrapper that delegates to `lib::run()`.

**Closest codebase analog:** `gtk_ref/src/main.rs`

**Analog pattern:**

> Initializes tracing, GTK icons, creates RelmApp, runs AppModel

The Tauri version is simpler — no tracing setup, no icon init, no component model. The `main()` function just calls the library entry point.

**Target pattern:**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    revelry_lib::run()
}
```

**Conventions applied:**

- `windows_subsystem = "windows"` attribute — prevents console window on Windows release builds; identical pattern to Tauri standard template
- Library crate name `revelry_lib` matches `[lib] name = "revelry_lib"` in Cargo.toml — follows Rust convention of `lib_name::function()`
- No tracing init yet — gtk_ref uses `tracing-subscriber`; this will be added in a future phase when the Rust backend has meaningful work to log

---

### F5 — `src-tauri/build.rs` (NEW)

**Role:** Tauri build script — generates embedded assets from `tauri.conf.json` and capabilities at compile time.

**Closest codebase analog:** `gtk_ref/` build.rs (uses `glib-build-tools` and `relm4-icons-build`)

**Analog pattern:** gtk_ref's build script compiles resources (GTK icons, UI files) into the binary. Tauri's build script does the same — it embeds `tauri.conf.json`, capability files, and icon assets into the Rust binary via `tauri_build::build()`.

**Target pattern:**

```rust
fn main() {
    tauri_build::build()
}
```

**Conventions applied:**

- Single function call — Tauri build script is intentionally minimal; all configuration is in `tauri.conf.json` and capabilities, not in build.rs code
- Mirrors gtk_ref's pattern of build script as thin bridge to framework build tooling

---

### F6–F12 — `src-tauri/capabilities/*.json` (NEW)

**Role:** Per-feature permission scoping — each file declares what its domain can do in the Tauri runtime.

**Closest codebase analog:** No direct analog. Conceptually closest is the SvelteKit routing structure — each route owns its page, each capability file owns its permission scope.

**Informed by existing pattern:**

```src/lib/components/ui/index.ts
// Barrel file pattern: each domain directory has an index.ts that
// re-exports its contents. Capabilities follow the same per-domain
// ownership principle — each file owns its permission scope.
```

**Target structure (all 7 files follow identical schema):**

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "<domain>",
  "description": "<domain description>",
  "windows": ["main"],
  "permissions": [ ... ]
}
```

**Capability files and their permission scope:**

| File              | Identifier   | Phase 1 Permissions                | Future Expansion            |
| ----------------- | ------------ | ---------------------------------- | --------------------------- |
| `default.json`    | `default`    | `core:*:default` (7 entries)       | —                           |
| `window.json`     | `window`     | `core:window:allow-*` (15 entries) | Custom title bar drag       |
| `filesystem.json` | `filesystem` | `fs:default`                       | Read/write app data dirs    |
| `dialog.json`     | `dialog`     | `dialog:default`                   | Open/save dialogs           |
| `shell.json`      | `shell`      | `shell:default`                    | Open URL in browser         |
| `http.json`       | `http`       | `http:default`                     | AI provider HTTP requests   |
| `store.json`      | `store`      | `store:default`                    | App preferences persistence |

**Conventions applied:**

- `"windows": ["main"]` — single window in Phase 1; future phases may add windows (e.g., pop-out chat)
- `identifier` matches filename — `window.json` → `"window"`, follows the naming convention from STRUCTURE.md (filenames map to their domain identifier)
- Stub files use `*:default` permissions — the minimal permission set that doesn't error. Expanded in the phase that needs them (per D-12, D-13)
- `$schema` path `../gen/schemas/desktop-schema.json` — relative to `capabilities/` directory, points to Tauri's generated schema (exists after first build)

---

### F13 — `src-tauri/icons/*` (NEW)

**Role:** App icons for the native window, taskbar, and installer bundles.

**Closest codebase analog:** `static/favicon.png` — the existing web app favicon.

**Pattern:** Generated by `tauri icon <source-image>` from a single high-resolution source. The CLI produces all required sizes: 32×32, 128×128, 128×128@2x, .icns (macOS), .ico (Windows).

**Conventions applied:**

- No manual icon creation — use the Tauri CLI generator
- The existing `static/favicon.png` can serve as the source image if no dedicated icon exists yet

---

### F14 — `svelte.config.js` (MODIFY)

**Role:** SvelteKit build adapter configuration — switches from server-adaptable output to static SPA output.

**Current pattern:**

```svelte.config.js#L1-14
import adapter from '@sveltejs/adapter-auto';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: vitePreprocess(),
	kit: {
		adapter: adapter(),
		alias: {
			$lib: 'src/lib',
			'$lib/*': 'src/lib/*'
		}
	}
};

export default config;
```

**Target pattern:**

```svelte.config.js
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: vitePreprocess(),
	kit: {
		adapter: adapter({
			fallback: 'index.html'
		}),
		alias: {
			$lib: 'src/lib',
			'$lib/*': 'src/lib/*'
		}
	}
};

export default config;
```

**Diff analysis:**

| Aspect       | Before             | After                                 | Rationale                                                            |
| ------------ | ------------------ | ------------------------------------- | -------------------------------------------------------------------- |
| Import       | `adapter-auto`     | `adapter-static`                      | D-05: app targets Tauri desktop, never a hosted server               |
| Adapter call | `adapter()`        | `adapter({ fallback: 'index.html' })` | D-06: SPA client-side routing — single HTML entry handles all routes |
| Preprocess   | `vitePreprocess()` | `vitePreprocess()`                    | ✅ Unchanged                                                         |
| Alias        | `$lib`/`$lib/*`    | `$lib`/`$lib/*`                       | ✅ Unchanged                                                         |
| JSDoc        | `@type` annotation | `@type` annotation                    | ✅ Unchanged — follows CONVENTIONS.md pattern                        |

**Data flow impact:**

- Before: `adapter-auto` may produce server-compatible output (adapter-node, adapter-vercel) depending on environment
- After: `adapter-static` always produces `build/` directory with static files + `index.html` fallback
- The `fallback: 'index.html'` ensures dynamic routes (`/chat/[id]`) work because SvelteKit's client router handles them — no need to prerender every possible `id`

**Conventions preserved:**

- JSDoc `@type` annotation on `config` — matches CONVENTIONS.md "Used on the Svelte config" pattern
- 2-space indent, tabs for `kit` nested properties — matches Prettier defaults observed in existing file
- `$lib` alias maintained — existing imports throughout `src/` rely on this

---

### F15 — `vite.config.ts` (MODIFY)

**Role:** Vite dev server configuration — aligns the dev port with Tauri's expected dev URL.

**Current pattern:**

```vite.config.ts#L1-11
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	server: {
		port: 5173,
		strictPort: false
	}
});
```

**Target pattern:**

```vite.config.ts
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	server: {
		port: 1420,
		strictPort: true
	}
});
```

**Diff analysis:**

| Aspect              | Before                       | After                        | Rationale                                                                                                                                  |
| ------------------- | ---------------------------- | ---------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| `server.port`       | `5173`                       | `1420`                       | D-09: Tauri conventional dev port; must match `tauri.conf.json` `devUrl`                                                                   |
| `server.strictPort` | `false`                      | `true`                       | D-09: if port 1420 is occupied, Vite must fail loudly rather than silently switching ports (which would break Tauri's `devUrl` connection) |
| Plugins             | `tailwindcss(), sveltekit()` | `tailwindcss(), sveltekit()` | ✅ Unchanged                                                                                                                               |
| Imports             | All 3 imports                | All 3 imports                | ✅ Unchanged                                                                                                                               |

**Critical coupling (Pitfall 2 from RESEARCH §7):**
The Vite port (`vite.config.ts`), the dev URL (`tauri.conf.json` `devUrl`), and the devCsp `connect-src` must all agree on port `1420`. If they diverge:

- Vite serves on 1420, Tauri tries to load from 1420 → ✅ works
- Vite silently switches to 1421 (strictPort: false), Tauri still loads 1420 → ❌ blank webview
- Vite serves on 1420, devCsp doesn't allow `localhost:1420` → ❌ HMR blocked

This is why `strictPort: true` is essential.

**Conventions preserved:**

- Plugin order: `tailwindcss()` before `sveltekit()` — unchanged; this order matters because Tailwind v4 processes CSS before SvelteKit transforms
- Tab indentation for nested properties — matches Prettier defaults in existing file
- No new imports needed — this is a value-only modification

---

### F16 — `package.json` (MODIFY)

**Role:** JS dependency manifest and script definitions — adds Tauri CLI, API, and plugin packages.

**Current pattern:**

```package.json#L1-35
{
  "name": "revelry-ui",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "description": "Revelry — AI roleplay desktop application UI (SvelteKit + Svelte 5 + Bits.ui)",
  "scripts": {
    "dev": "vite dev",
    "build": "vite build",
    "preview": "vite preview",
    "check": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json",
    "check:watch": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json --watch",
    "format": "prettier --write .",
    "lint": "prettier --check ."
  },
  "devDependencies": {
    "@sveltejs/adapter-auto": "^7.0",
    ...
  },
  "dependencies": {
    "@lucide/svelte": "^1.14",
    ...
  }
}
```

**Additions:**

**Scripts (D-08):**

```json
"tauri:dev": "tauri dev",
"tauri:build": "tauri build",
"tauri:preview": "tauri build && tauri dev --no-watch",
"tauri:check": "tauri info"
```

**devDependencies:**
| Package | Version | Replaces/Add | Rationale |
|---------|---------|-------------|-----------|
| `@sveltejs/adapter-auto` | `^7.0` | **REMOVE** | Replaced by adapter-static |
| `@sveltejs/adapter-static` | `^3` | **ADD** | D-05: permanent switch for Tauri |
| `@tauri-apps/cli` | `^2` | **ADD** | D-03: Tauri dev/build CLI tooling |

**dependencies:**
| Package | Version | Rationale |
|---------|---------|-----------|
| `@tauri-apps/api` | `^2` | D-03: Tauri IPC bridge (invoke, events) |
| `@tauri-apps/plugin-fs` | `^2` | D-15: FS plugin JS wrapper |
| `@tauri-apps/plugin-dialog` | `^2` | D-15: Dialog plugin JS wrapper |
| `@tauri-apps/plugin-shell` | `^2` | D-15: Shell plugin JS wrapper |
| `@tauri-apps/plugin-http` | `^2` | D-15: HTTP plugin JS wrapper |
| `@tauri-apps/plugin-store` | `^2` | D-15: Store plugin JS wrapper |

**Cross-reference with F1 (Cargo.toml):**
Each `@tauri-apps/plugin-*` JS package has a corresponding `tauri-plugin-*` Rust crate. The JS and Rust sides must match:

- `@tauri-apps/plugin-fs` ↔ `tauri-plugin-fs` (Cargo.toml)
- `@tauri-apps/plugin-dialog` ↔ `tauri-plugin-dialog` (Cargo.toml)
- etc.

This is **Pitfall 5** from RESEARCH §7: "Plugin registration mismatch between Rust and JS" — every plugin registered in `lib.rs` must have both the Rust crate and JS package installed, and both must be the same major version.

**Conventions applied:**

- Script naming: `tauri:*` prefix groups all Tauri commands — consistent with existing `check:watch` colon-separated namespacing
- Version pinning: `^2` major-only — matches Cargo.toml's `"2"` convention; Bun/npm will resolve to latest 2.x.y
- `adapter-auto` **removed** (not just kept unused) — D-05 is a permanent switch; keeping both would create ambiguity about the target

---

### F17 — `src/routes/+layout.ts` (MODIFY)

**Role:** SvelteKit root layout configuration — adds prerender flag for adapter-static compatibility.

**Current pattern:**

```src/routes/+layout.ts#L1-7
/**
 * This is rendered as a desktop-style SPA. Disabling SSR keeps the runes-based
 * state model simple and matches the local-first philosophy from the spec —
 * there is no server, just a packaged app.
 */
export const ssr = false;
export const prerender = false;
```

**Target pattern:**

```src/routes/+layout.ts
/**
 * This is rendered as a desktop-style SPA. Disabling SSR keeps the runes-based
 * state model simple and matches the local-first philosophy from the spec —
 * there is no server, just a packaged app.
 */
export const ssr = false;
export const prerender = true;
```

**Diff analysis:**

| Aspect        | Before  | After   | Rationale                                           |
| ------------- | ------- | ------- | --------------------------------------------------- |
| `ssr`         | `false` | `false` | ✅ Unchanged — SPA pattern                          |
| `prerender`   | `false` | `true`  | D-07: required by `adapter-static` for SPA mode     |
| JSDoc comment | Present | Present | ✅ Unchanged — preserves the architecture rationale |

**Why `prerender = true` is needed (Pitfall 1 from RESEARCH §7):**

- `adapter-static` requires either `prerender = true` or a `fallback` option on every route
- With `ssr = false` + `prerender = true` + `adapter({ fallback: 'index.html' })`:
  - Static pages (/, /characters, /settings) are pre-rendered as HTML
  - Dynamic routes (/chat/[id]) fall back to `index.html` → client-side router handles them
  - Without `prerender = true`, `adapter-static` throws: "All routes must be prerendered or have a fallback"
- The `ssr = false` + `prerender = true` combination is not contradictory: `prerender = true` tells SvelteKit to generate HTML shells at build time; `ssr = false` means those shells don't run server-side logic — they're just empty HTML that hydrates client-side

**Conventions preserved:**

- JSDoc comment explaining the architectural decision — CONVENTIONS.md notes comments serve as documentation
- `export const` for each config variable — SvelteKit layout convention
- `false` → `true` is the only change; no restructuring

---

### F18 — `.gitignore` (MODIFY)

**Role:** VCS ignore rules — adds Tauri-specific build artifacts.

**Current pattern:**

```gitignore
# Rust build artifacts
/target/
node_modules
# IDE
.idea/
.vscode/
*.swp
*.swo
.svelte*
# macOS
.DS_Store

# Environment
.env
```

**Additions:**

```
# Tauri build artifacts
src-tauri/target/
src-tauri/gen/
```

**Pattern analysis:**

- `/target/` already exists — this covers the gtk_ref Rust artifacts at the project root
- `src-tauri/target/` is needed because `src-tauri/` is a separate Cargo project with its own `target/` directory
- `src-tauri/gen/` contains Tauri's auto-generated schema files and bindings — not source code, should not be committed

**Conventions applied:**

- Section comment style: `# <category>` followed by entries — matches existing `# Rust build artifacts`, `# IDE`, `# macOS`, `# Environment` pattern
- Trailing slashes on directories — `/target/`, `src-tauri/target/`, `src-tauri/gen/` — consistent with existing `/target/` pattern

---

## 3. Cross-File Dependency Map

```
package.json ──────────────────────────────────────────────────┐
  │ @sveltejs/adapter-auto → REMOVE                           │
  │ @sveltejs/adapter-static → ADD                            │
  │ @tauri-apps/cli → ADD                                      │
  │ @tauri-apps/api → ADD                                      │
  │ @tauri-apps/plugin-* → ADD                                 │
  │ scripts.tauri:* → ADD                                      │
  │                                                            │
  ▼                                                            │
svelte.config.js ────── adapter-auto → adapter-static ─────────┤
  │                     fallback: 'index.html'                 │
  │                                                            │
  ├──► +layout.ts ─── prerender: false → true                  │
  │     (adapter-static requires prerender = true              │
  │      or per-route fallback config)                         │
  │                                                            │
  └──► vite.config.ts ─ port: 5173 → 1420 ────────────────────┤
       strictPort: false → true                                │
         │                                                     │
         │  port must match                                    │
         ▼                                                     ▼
       tauri.conf.json ──── devUrl: http://localhost:1420 ─────┤
         │                                                     │
         │  devCsp.connect-src must include                    │
         │  localhost:1420 and ws://localhost:1421             │
         │                                                     │
         ├──► Cargo.toml ─── tauri + plugin crates             │
         │     (must match @tauri-apps/plugin-* in package.json)
         │                                                     │
         ├──► lib.rs ──── .plugin() calls must match           │
         │     Cargo.toml [dependencies] entries               │
         │                                                     │
         └──► capabilities/ ── permissions must match          │
               Cargo.toml plugin entries                        │
                                                              │
.gitignore ◄──────────────────────────────────────────────────┘
  │ ADD: src-tauri/target/
  │ ADD: src-tauri/gen/
```

---

## 4. Critical Coupling Points

### 4.1 Port Alignment Triangle

Three files must agree on port `1420`:

| File              | Field                             | Value                            |
| ----------------- | --------------------------------- | -------------------------------- |
| `vite.config.ts`  | `server.port`                     | `1420`                           |
| `tauri.conf.json` | `build.devUrl`                    | `http://localhost:1420`          |
| `tauri.conf.json` | `app.security.devCsp.connect-src` | includes `http://localhost:1420` |

**Failure mode:** If Vite serves on a different port than Tauri expects, the webview shows a blank page (connection refused) or HMR fails (CSP blocks WebSocket).

### 4.2 Adapter-Prerender Contract

Two files must agree on SPA mode:

| File                    | Field       | Value                                        |
| ----------------------- | ----------- | -------------------------------------------- |
| `svelte.config.js`      | `adapter`   | `adapter-static({ fallback: 'index.html' })` |
| `src/routes/+layout.ts` | `prerender` | `true`                                       |

**Failure mode:** If `prerender` stays `false`, adapter-static throws at build time: "All routes must be prerendered or have a fallback." If `fallback` is missing, dynamic routes like `/chat/[id]` return 404 in production.

### 4.3 Plugin Registration Triangle

Three locations must agree for each plugin:

| Location                        | Example (fs plugin)                |
| ------------------------------- | ---------------------------------- |
| `Cargo.toml` `[dependencies]`   | `tauri-plugin-fs = "2"`            |
| `src-tauri/src/lib.rs`          | `.plugin(tauri_plugin_fs::init())` |
| `package.json` `[dependencies]` | `@tauri-apps/plugin-fs = "^2"`     |

**Failure mode:** If a plugin is in Cargo.toml but not registered in lib.rs, the Rust side compiles but the JS side can't invoke it. If registered in lib.rs but not in Cargo.toml, compilation fails. If in package.json but not Cargo.toml, JS imports resolve but `invoke()` fails at runtime.

### 4.4 Build Output Path

| File               | Field                | Value                                  |
| ------------------ | -------------------- | -------------------------------------- |
| `svelte.config.js` | `adapter`            | `adapter-static` → outputs to `build/` |
| `tauri.conf.json`  | `build.frontendDist` | `../build` (relative to `src-tauri/`)  |

**Failure mode:** If `frontendDist` doesn't point to adapter-static's actual output directory, Tauri can't find the frontend files in production builds.

---

## 5. Implementation Order

The files must be created/modified in a specific order due to dependencies:

| Step | File                        | Action                                      | Depends on                                |
| ---- | --------------------------- | ------------------------------------------- | ----------------------------------------- |
| 1    | `package.json`              | Add adapter-static + Tauri deps             | —                                         |
| 2    | `svelte.config.js`          | Switch adapter                              | Step 1 (adapter-static installed)         |
| 3    | `src/routes/+layout.ts`     | Add prerender = true                        | Step 2 (adapter-static active)            |
| 4    | `vite.config.ts`            | Change port + strictPort                    | —                                         |
| 5    | `bun install`               | Install new deps                            | Step 1                                    |
| 6    | `src-tauri/`                | `tauri init` (creates F1–F13)               | Steps 1–4 (dev/build commands must exist) |
| 7    | `src-tauri/tauri.conf.json` | Manual adjustments (CSP, window size, etc.) | Step 6                                    |
| 8    | `src-tauri/Cargo.toml`      | Add plugin stubs                            | Step 6                                    |
| 9    | `src-tauri/src/lib.rs`      | Add plugin registrations                    | Step 8                                    |
| 10   | `src-tauri/capabilities/`   | Create 7 capability files                   | Step 6                                    |
| 11   | `.gitignore`                | Add src-tauri/target/ and src-tauri/gen/    | Step 6                                    |
| 12   | Verification                | `bun run tauri:dev`                         | All steps                                 |

**Rationale:** Steps 1–4 prepare the frontend for Tauri (static output, correct port). Step 5 installs dependencies. Step 6 scaffolds the Rust backend. Steps 7–10 customize the scaffold. Step 11 keeps the repo clean. Step 12 validates the full pipeline.

---

## 6. Existing Pattern Reuse Summary

| Pattern                                                  | Source                               | Applied To                 | Adaptation                                                           |
| -------------------------------------------------------- | ------------------------------------ | -------------------------- | -------------------------------------------------------------------- |
| Package identity (`name: "revelry"`, `version: "0.1.0"`) | `gtk_ref/Cargo.toml`                 | F1 `src-tauri/Cargo.toml`  | Direct carry-over                                                    |
| `serde` with `derive` feature                            | `gtk_ref/Cargo.toml`                 | F1 `src-tauri/Cargo.toml`  | Required for Tauri IPC serialization                                 |
| `serde_json`                                             | `gtk_ref/Cargo.toml`                 | F1 `src-tauri/Cargo.toml`  | Required for Tauri IPC JSON transport                                |
| `[build-dependencies]` section                           | `gtk_ref/Cargo.toml`                 | F1 `src-tauri/Cargo.toml`  | `glib-build-tools` → `tauri-build`                                   |
| Platform-conditional attributes                          | `gtk_ref/src/main.rs`                | F4 `src-tauri/src/main.rs` | `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` |
| Builder/initialization pattern                           | `gtk_ref/src/app.rs`                 | F3 `src-tauri/src/lib.rs`  | Relm4 component init → Tauri Builder chain                           |
| JSDoc `@type` annotation on config                       | `svelte.config.js`                   | F14                        | Preserved unchanged                                                  |
| Barrel/per-domain file organization                      | `src/lib/components/ui/index.ts`     | F6–F12                     | Each capability file = one domain                                    |
| Section comments in gitignore                            | `.gitignore`                         | F18                        | `# Tauri build artifacts` follows `# Rust build artifacts`           |
| `export const` page options                              | `src/routes/+layout.ts`              | F17                        | Add `prerender = true` alongside existing `ssr = false`              |
| Plugin order by domain                                   | CONVENTIONS.md (import organization) | F3 `lib.rs`                | fs → dialog → shell → http → store                                   |

---

_Pattern mapping complete. Ready for plan execution._
