# Phase 1: Native Desktop Shell - Context

**Gathered:** 2026-05-10
**Status:** Ready for planning

<domain>
## Phase Boundary

The app runs as a native Tauri desktop window on macOS, Linux, and Windows with full SvelteKit routing working inside the Tauri webview. This phase delivers the Tauri shell that wraps the existing SvelteKit SPA — no new UI features, no backend logic, no IPC commands. The entire scope is: initialize Tauri v2, configure the adapter, wire the dev/build pipeline, and verify the app launches and routes correctly.

</domain>

<decisions>
## Implementation Decisions

### Tauri Init Approach
- **D-01:** Use `tauri init` CLI to bootstrap `src-tauri/` into the existing project — standard approach, least disruptive to the existing `src/` structure
- **D-02:** Use Tauri-managed dev server — `tauri dev` runs everything via `beforeDevCommand`; single command to start both Rust backend and Vite frontend
- **D-03:** Install `@tauri-apps/cli` as devDependency and `@tauri-apps/api` as dependency in root `package.json` — standard Tauri v2 pattern, project-portable
- **D-04:** Configure window properties at init time — app title "Revelry", initial window size 1200×800, disable devtools in production, set CSP

### Adapter Strategy
- **D-05:** Permanent switch from `adapter-auto` to `adapter-static` — the app's target is Tauri desktop, never a hosted server
- **D-06:** Single `fallback: 'index.html'` for SPA client-side routing — handles dynamic routes like `chat/[id]` cleanly without per-route prerender complexity
- **D-07:** Set `prerender = true` alongside `ssr = false` in root `+layout.ts` — standard `adapter-static` SPA pattern. SSR disabled because all data comes from the Rust backend at runtime via `invoke()`, not from a build-time pre-render step

### Dev Workflow
- **D-08:** Add full dev tooling scripts to `package.json`: `tauri:dev`, `tauri:build`, `tauri:preview`, `tauri:check`
- **D-09:** Switch Vite dev server port from 5173 to 1420 — Tauri's conventional dev port; set `devUrl` in `tauri.conf.json` to `http://localhost:1420`
- **D-10:** `bun run dev` for `beforeDevCommand` — matches the project's Bun package manager (`bun.lock` present)
- **D-11:** `bun run build` for `beforeBuildCommand` — consistent with the Bun toolchain choice

### Capabilities Scope
- **D-12:** Scaffold all expected Tauri v2 capability files upfront — window, IPC, filesystem, shell, dialog, http, store. Permissions enabled only as needed per phase
- **D-13:** Per-feature capability files in `src-tauri/capabilities/` — `window.json`, `ipc.json`, `filesystem.json`, `shell.json`, `dialog.json`, `http.json`, `store.json`. Each file owns its permission scope for clean separation and per-phase auditing
- **D-14:** Permissive dev CSP, strict production CSP — dev allows `localhost` connections and `eval` for Vite HMR; production restricts to `self` and `tauri://` origins only, no `eval`, no inline scripts
- **D-15:** Add Tauri plugin stubs to both `Cargo.toml` and `package.json` now — `tauri-plugin-fs`, `tauri-plugin-dialog`, `tauri-plugin-shell`, `tauri-plugin-http`, `tauri-plugin-store` with their JS wrappers (`@tauri-apps/plugin-*`). Each phase activates what it needs

### Claude's Discretion
None — all decisions were explicitly made by the user.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Planning
- `.planning/ROADMAP.md` — Phase 1 definition, plans list (01-01 through 01-04), success criteria
- `.planning/REQUIREMENTS.md` — SHELL-01, SHELL-02, SHELL-03, SETT-03, CHAT-05 requirements
- `.planning/PROJECT.md` — Project context, constraints, tech stack decisions

### Codebase Architecture
- `.planning/codebase/STACK.md` — Technology stack: SvelteKit, Svelte 5, Vite, Tailwind v4, Bits.ui, Bun runtime
- `.planning/codebase/ARCHITECTURE.md` — System overview, component responsibilities, layers, data flow
- `.planning/codebase/CONVENTIONS.md` — Naming, code style, import organization, module design
- `.planning/codebase/STRUCTURE.md` — Directory layout, key file locations, where to add new code

### Key Source Files
- `svelte.config.js` — Current adapter-auto config (switching to adapter-static)
- `vite.config.ts` — Current port 5173 (switching to 1420)
- `package.json` — Current scripts and dependencies (adding Tauri packages and scripts)
- `src/routes/+layout.ts` — Current `ssr = false` (adding `prerender = true`)
- `src/routes/+layout.svelte` — App shell with keyboard shortcuts
- `src/lib/stores/app.svelte.ts` — Central AppState (reference for integration points)

### Archival Reference
- `gtk_ref/Cargo.toml` — Rust dependency patterns for future Tauri backend (reference only, not modified in this phase)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **SvelteKit SPA shell**: Fully functional UI with routing, state, mock data — wraps directly into Tauri webview without changes to UI code
- **`+layout.ts`**: Already has `ssr = false` set — just needs `prerender = true` added
- **`vite.config.ts`**: Already configured with SvelteKit + Tailwind plugins — just needs port change
- **`src/routes/`**: All four routes (`/`, `/chat/[id]`, `/characters`, `/settings`) already work — Tauri webview will load them as-is

### Established Patterns
- **Svelte 5 runes**: `$state`, `$derived`, `$effect` for reactive state — no changes needed for Tauri
- **Mock data pattern**: All UI flows work without a backend — ideal for Phase 1 since no IPC commands are added yet
- **Bits.ui + Tailwind**: Headless component styling — works identically in Tauri webview as in browser

### Integration Points
- **`svelte.config.js`**: Adapter swap — `adapter-auto` → `adapter-static` with fallback
- **`package.json`**: New scripts and Tauri dependencies
- **`vite.config.ts`**: Port change 5173 → 1420
- **New `src-tauri/` directory**: Rust backend shell — `main.rs`/`lib.rs`, `Cargo.toml`, `tauri.conf.json`, capabilities/
- **`src/routes/+layout.ts`**: Add `prerender = true`

</code_context>

<specifics>
## Specific Ideas

- Window title: "Revelry"
- Window size: 1200×800
- DevTools disabled in production builds
- Bun as the package manager and script runner (consistent across dev and build)
- Port 1420 for Tauri dev server (Tauri convention)

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 01-Native Desktop Shell*
*Context gathered: 2026-05-10*
