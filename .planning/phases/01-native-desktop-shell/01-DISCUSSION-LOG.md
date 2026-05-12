# Phase 1: Native Desktop Shell - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-10
**Phase:** 01-Native Desktop Shell
**Areas discussed:** Tauri Init Approach, Adapter Strategy, Dev Workflow, Capabilities Scope

---

## Tauri Init Approach

### Q1: How to bootstrap the Tauri backend into the existing project?

| Option           | Description                                                                                                                                        | Selected |
| ---------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| `tauri init` CLI | Run `tauri init` inside the existing project. Detects existing frontend, creates `src-tauri/` with Cargo.toml + tauri.conf.json. Least disruptive. | ✓        |
| Manual setup     | Hand-craft `src-tauri/Cargo.toml`, `tauri.conf.json`, and `main.rs` from Tauri v2 docs. Full control, more effort.                                 |          |
| You decide       | Let Claude pick what fits best.                                                                                                                    |          |

**User's choice:** `tauri init` CLI

### Q2: Which dev workflow should `tauri.conf.json` point to?

| Option                   | Description                                                                                              | Selected |
| ------------------------ | -------------------------------------------------------------------------------------------------------- | -------- |
| Tauri-managed dev server | Let `tauri dev` start the Vite dev server internally via `beforeDevCommand`. One command for everything. | ✓        |
| Separate dev servers     | Point `tauri.conf.json` at `http://localhost:5173` without `beforeDevCommand`. Two commands to start.    |          |
| You decide               | Let Claude pick.                                                                                         |          |

**User's choice:** Tauri-managed dev server

### Q3: How should we install the Tauri CLI and Rust integration?

| Option               | Description                                                                                          | Selected |
| -------------------- | ---------------------------------------------------------------------------------------------------- | -------- |
| npm Tauri packages   | `@tauri-apps/cli` as devDep + `@tauri-apps/api` as dep. Standard Tauri v2 pattern, project-portable. | ✓        |
| Global `cargo-tauri` | No npm Tauri packages, just the global Rust CLI. Simpler package.json, less portable.                |          |
| You decide           | Let Claude pick.                                                                                     |          |

**User's choice:** npm Tauri packages (`@tauri-apps/cli` devDep + `@tauri-apps/api` dep)

### Q4: Should we customize the window configuration at init time or accept defaults?

| Option                      | Description                                                                             | Selected |
| --------------------------- | --------------------------------------------------------------------------------------- | -------- |
| Accept defaults, tune later | Get the shell running first with default window size, title, etc.                       |          |
| Configure window upfront    | Set app title "Revelry", window size 1200×800, disable devtools in production, set CSP. | ✓        |
| You decide                  | Let Claude pick.                                                                        |          |

**User's choice:** Configure window upfront

---

## Adapter Strategy

### Q1: Should we switch to `adapter-static` permanently, or keep the option for browser-only development?

| Option                               | Description                                                                                                 | Selected |
| ------------------------------------ | ----------------------------------------------------------------------------------------------------------- | -------- |
| Permanent switch to `adapter-static` | The app's destiny is a Tauri desktop app, not a hosted SPA. No reason to keep `adapter-auto`.               | ✓        |
| Conditional via separate configs     | Keep `adapter-auto` as default, use `adapter-static` only during Tauri builds. Two build paths to maintain. |          |
| You decide                           | Let Claude pick.                                                                                            |          |

**User's choice:** Permanent switch to `adapter-static`

### Q2: How should we handle the `adapter-static` prerender configuration?

| Option                             | Description                                                                                                     | Selected |
| ---------------------------------- | --------------------------------------------------------------------------------------------------------------- | -------- |
| Prerender a single `fallback.html` | All routes handled client-side by SvelteKit router. Simplest SPA approach.                                      |          |
| Prerender all routes               | Add `prerender = true` to root layout. Generates actual HTML files per route. More overhead for dynamic routes. |          |

**User's choice:** Initially "Prerender all routes", then revised to "Single `fallback.html`" after realizing dynamic routes like `chat/[id]` can't be prerendered at build time.

**Notes:** User asked about whether `ssr = true` would be better for Tauri desktop apps. Discussion confirmed that `ssr = false` is correct because: (1) load functions will call `invoke()` which has no Tauri runtime at build time, (2) dynamic routes can't be pre-rendered anyway, (3) desktop apps load from local disk so there's no network latency that SSR would solve.

### Q3: Should we also set `prerender = true` at the root level alongside `ssr = false`?

| Option                                          | Description                                                                 | Selected |
| ----------------------------------------------- | --------------------------------------------------------------------------- | -------- |
| Yes — set both in root `+layout.ts`             | Standard `adapter-static` SPA pattern: SSR off, prerender on with fallback. | ✓        |
| No — only configure adapter's `fallback` option | Some versions handle this automatically. Less explicit.                     |          |
| You decide                                      | Let Claude pick.                                                            |          |

**User's choice:** Yes — set both `prerender = true` and `ssr = false` in root `+layout.ts`

---

## Dev Workflow

### Q1: What Tauri scripts should we add to `package.json`?

| Option           | Description                                                                        | Selected |
| ---------------- | ---------------------------------------------------------------------------------- | -------- |
| Minimal          | Just `tauri dev` and `tauri build`. Keep it lean.                                  |          |
| Full dev tooling | `tauri:dev`, `tauri:build`, `tauri:preview`, `tauri:check`. Covers more workflows. | ✓        |
| You decide       | Let Claude pick.                                                                   |          |

**User's choice:** Full dev tooling

### Q2: Keep port 5173 or switch to Tauri's default 1420?

| Option         | Description                                                           | Selected |
| -------------- | --------------------------------------------------------------------- | -------- |
| Keep port 5173 | Already configured in `vite.config.ts`. No changes needed.            |          |
| Switch to 1420 | Tauri's conventional dev port. Avoids conflicts with other Vite apps. | ✓        |
| You decide     | Let Claude pick.                                                      |          |

**User's choice:** Switch to 1420

### Q3: How should `beforeDevCommand` be configured?

| Option        | Description                                                                     | Selected |
| ------------- | ------------------------------------------------------------------------------- | -------- |
| `npm run dev` | Uses existing SvelteKit dev script via npm. Standard.                           |          |
| `bun run dev` | Matches the project's Bun package manager (`bun.lock` present). Faster startup. | ✓        |
| You decide    | Let Claude pick.                                                                |          |

**User's choice:** `bun run dev`

### Q4: What should the `beforeBuildCommand` be?

| Option          | Description                                                       | Selected |
| --------------- | ----------------------------------------------------------------- | -------- |
| `bun run build` | Consistent with dev command choice. Build SvelteKit SPA with Bun. | ✓        |
| `npm run build` | More portable for CI/CD. Slightly slower.                         |          |
| You decide      | Let Claude pick.                                                  |          |

**User's choice:** `bun run build`

---

## Capabilities Scope

### Q1: How should we define the Tauri v2 capability schema for Phase 1?

| Option                             | Description                                                                                     | Selected |
| ---------------------------------- | ----------------------------------------------------------------------------------------------- | -------- |
| Minimal — window permissions only  | Just `core:window:default` and `core:webview:default`. Expand later.                            |          |
| Scaffold all expected capabilities | Create files for window, IPC, filesystem, shell, dialog, etc. with permissions ready to enable. | ✓        |
| You decide                         | Let Claude pick.                                                                                |          |

**User's choice:** Scaffold all expected capabilities upfront

### Q2: How should the capability files be organized?

| Option                       | Description                                                                            | Selected |
| ---------------------------- | -------------------------------------------------------------------------------------- | -------- |
| Single capability file       | One `default.json` with all permission groups. Simple but noisy as permissions grow.   |          |
| Per-feature capability files | Separate files: `window.json`, `ipc.json`, `filesystem.json`, etc. Cleaner separation. | ✓        |
| You decide                   | Let Claude pick.                                                                       |          |

**User's choice:** Per-feature capability files

### Q3: How restrictive should the CSP be?

| Option                                 | Description                                                                       | Selected |
| -------------------------------------- | --------------------------------------------------------------------------------- | -------- |
| Permissive dev, restrictive production | Dev: allow localhost + eval for HMR. Production: strict self/tauri:// only.       | ✓        |
| Uniform strict CSP                     | Same strict policy in both dev and production. More secure but HMR complications. |          |
| You decide                             | Let Claude pick.                                                                  |          |

**User's choice:** Permissive dev CSP, strict production CSP

### Q4: Should we add Tauri plugin dependencies now or defer to each phase?

| Option               | Description                                                                                                                                                                                    | Selected |
| -------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| Add plugin stubs now | Include `tauri-plugin-fs`, `tauri-plugin-dialog`, `tauri-plugin-shell`, `tauri-plugin-http`, `tauri-plugin-store` in both `Cargo.toml` and `package.json`. Each phase activates what it needs. | ✓        |
| Add only when needed | Each phase adds its own plugins. Keeps dependencies minimal but more file churn per phase.                                                                                                     |          |
| You decide           | Let Claude pick.                                                                                                                                                                               |          |

**User's choice:** Add plugin stubs now

---

## Claude's Discretion

None — all decisions were explicitly made by the user.

## Deferred Ideas

None — discussion stayed within phase scope.
