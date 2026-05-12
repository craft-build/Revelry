# Walking Skeleton — Phase 1: Native Desktop Shell

**Created:** 2026-05-10
**Status:** Planned (not yet executed)

---

## Skeleton Definition

**The user can launch the app as a native desktop window and navigate between all SvelteKit routes.**

This exercises the full stack: Tauri shell → webview → SvelteKit routing → existing UI components.

---

## Framework

| Layer               | Technology   | Version            |
| ------------------- | ------------ | ------------------ |
| Desktop shell       | Tauri v2     | ^2 (latest stable) |
| Frontend framework  | SvelteKit    | ^2.59              |
| Component framework | Svelte 5     | ^5.55              |
| Build tool          | Vite         | ^8.0               |
| CSS framework       | Tailwind CSS | ^4.3               |
| Package manager     | Bun          | latest             |
| Backend language    | Rust         | 2021 edition       |

---

## Data Layer

**None for Phase 1 — deferred to Phase 2.**

All UI data comes from hardcoded mock data in `src/lib/data/mock.ts`. The Tauri Rust backend has no IPC commands in Phase 1 — it's a minimal shell with plugin stubs only.

---

## Auth

**None — local-only.**

The app runs entirely on the local machine. No authentication, no network calls, no user accounts. Phase 1 has no backend logic beyond the Tauri runtime.

---

## Deployment Target

| Property        | Value                                         |
| --------------- | --------------------------------------------- |
| Target          | Local desktop app                             |
| Platforms       | macOS (primary dev), Linux, Windows           |
| Distribution    | `bun tauri build` produces platform installer |
| Dev command     | `bun tauri dev`                               |
| Build command   | `bun tauri build`                             |
| Preview command | `bun run tauri:preview`                       |

---

## Directory Layout

```
persona/
├── src/                          # SvelteKit UI (existing, modified for adapter-static)
│   ├── lib/
│   │   ├── components/           # Svelte components (unchanged)
│   │   ├── data/                 # Mock data (unchanged)
│   │   ├── stores/              # AppState (unchanged)
│   │   ├── types/               # TypeScript types (unchanged)
│   │   └── utils/               # Utilities (unchanged)
│   ├── routes/                  # SvelteKit routes (unchanged except +layout.ts)
│   │   ├── +layout.svelte       # App shell with keyboard shortcuts (unchanged)
│   │   └── +layout.ts           # ssr=false, prerender=true (MODIFIED)
│   └── app.css                  # Tailwind theme + drag-region CSS (unchanged)
├── src-tauri/                   # Tauri v2 Rust backend (NEW)
│   ├── Cargo.toml               # Rust deps: tauri + plugin stubs + serde
│   ├── tauri.conf.json          # Window config, CSP, dev/build commands
│   ├── build.rs                 # Tauri build script
│   ├── src/
│   │   ├── main.rs              # Desktop entry point
│   │   └── lib.rs               # Tauri builder with plugin registrations
│   ├── capabilities/            # Per-feature permission files (7 files)
│   │   ├── default.json         # Core permissions
│   │   ├── window.json          # Window management (drag, resize, etc.)
│   │   ├── filesystem.json      # Stub: fs:default
│   │   ├── dialog.json          # Stub: dialog:default
│   │   ├── shell.json           # Stub: shell:default
│   │   ├── http.json            # Stub: http:default
│   │   └── store.json           # Stub: store:default
│   └── icons/                   # App icons (default tauri init icons)
├── package.json                 # Modified: +Tauri deps, +scripts, -adapter-auto
├── svelte.config.js             # Modified: adapter-static with fallback
├── vite.config.ts               # Modified: port 1420, strictPort true
└── .gitignore                   # Modified: +src-tauri/target/, +src-tauri/gen/
```

---

## End-to-End Path

```
User runs `bun tauri dev`
  → Tauri CLI starts
    → beforeDevCommand: `bun run dev` starts Vite on port 1420
    → Rust binary compiles and launches
      → Native desktop window opens (1200×800, title "Revelry")
        → WebView loads http://localhost:1420
          → SvelteKit SPA renders
            → User navigates: / → /chat/[id] → /characters → /settings
            → User presses F9 → sidebar toggles
            → User types in chat input → Enter sends message
            → User drags AppHeader → window moves
```

---

## Key Configuration Points

| File                    | Field                      | Value                                        | Why                                           |
| ----------------------- | -------------------------- | -------------------------------------------- | --------------------------------------------- |
| `vite.config.ts`        | `server.port`              | `1420`                                       | Tauri convention; must match devUrl           |
| `vite.config.ts`        | `server.strictPort`        | `true`                                       | Prevents silent port shift breaking Tauri     |
| `tauri.conf.json`       | `build.devUrl`             | `http://localhost:1420`                      | Must match Vite port                          |
| `tauri.conf.json`       | `build.frontendDist`       | `../build`                                   | Relative to src-tauri/; adapter-static output |
| `tauri.conf.json`       | `build.beforeDevCommand`   | `bun run dev`                                | Starts Vite before Tauri                      |
| `tauri.conf.json`       | `build.beforeBuildCommand` | `bun run build`                              | Builds SvelteKit before Rust bundling         |
| `tauri.conf.json`       | `app.windows[0].title`     | `Revelry`                                    | App identity                                  |
| `tauri.conf.json`       | `app.windows[0].width`     | `1200`                                       | Desktop-class layout width                    |
| `tauri.conf.json`       | `app.windows[0].height`    | `800`                                        | Desktop-class layout height                   |
| `tauri.conf.json`       | `app.windows[0].devtools`  | `false`                                      | Production security                           |
| `tauri.conf.json`       | `app.security.csp`         | (strict)                                     | No unsafe-eval in production                  |
| `tauri.conf.json`       | `app.security.devCsp`      | (permissive)                                 | Allows HMR in dev                             |
| `svelte.config.js`      | `adapter`                  | `adapter-static({ fallback: 'index.html' })` | Tauri target is static files                  |
| `src/routes/+layout.ts` | `prerender`                | `true`                                       | Required by adapter-static                    |
| `src/routes/+layout.ts` | `ssr`                      | `false`                                      | SPA mode; no server                           |

---

## What's NOT in the Skeleton

| Feature          | Deferred To | Reason                                            |
| ---------------- | ----------- | ------------------------------------------------- |
| SQLite database  | Phase 2     | No data persistence needed for shell verification |
| IPC commands     | Phase 2     | No backend logic in shell phase                   |
| AI streaming     | Phase 4     | No real AI calls; mock streaming works            |
| OS keychain      | Phase 3     | No API keys in shell phase                        |
| Custom app icons | Phase 11    | Default tauri init icons sufficient               |
| Autoupdate       | v2          | Deferred per STATE.md                             |
| System tray      | v2          | Deferred per STATE.md                             |

---

## Verification Checklist

- [ ] `bun tauri dev` launches a native desktop window
- [ ] Window title is "Revelry"
- [ ] Window size is approximately 1200×800
- [ ] All 4 routes render: `/`, `/chat/[id]`, `/characters`, `/settings`
- [ ] Client-side navigation works (no full reloads)
- [ ] F9 toggles sidebar
- [ ] Enter sends messages
- [ ] Shift+Enter inserts newline
- [ ] Ctrl/Cmd+Enter sends messages
- [ ] AppHeader drag region moves the window
- [ ] HMR works in dev mode
- [ ] `bun tauri build` produces a production binary
- [ ] Production binary runs correctly
- [ ] DevTools are disabled in production
- [ ] Production CSP doesn't break app functionality

---

_Walking skeleton defined for Phase 1 — Native Desktop Shell._
