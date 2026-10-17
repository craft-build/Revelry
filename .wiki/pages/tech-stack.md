---
type: Reference
description: Languages, frameworks, key dependencies, package layout, and entry points for the Revelry Tauri + SvelteKit + Rust desktop app.
tags:
  - tech-stack
  - dependencies
  - tauri
  - svelte
  - rust
  - package-layout
timestamp: 2026-07-12T18:26:11Z
---

# Tech Stack

Revelry is an AI roleplay desktop app: a **Tauri v2** shell wrapping a **SvelteKit 2 + Svelte 5** TypeScript frontend and a **Rust** backend. The frontend never touches the database, filesystem, or LLM APIs directly — everything flows through typed Tauri IPC commands.

## Languages & Runtimes

- **Rust** — edition 2021, crate `revelry_lib` (`src-tauri/Cargo.toml:6`)
- **TypeScript** — strict mode (`tsconfig.json:11`)
- **Bun** — package manager (`bun.lock`)

## Frontend Frameworks & Libraries

| Library                                         | Version | Purpose                                                                  |
| ----------------------------------------------- | ------- | ------------------------------------------------------------------------ |
| SvelteKit                                       | ^2.69.2 | Framework, SSR disabled (`src/routes/+layout.ts` sets `ssr=false`)       |
| Svelte                                          | ^5.56.4 | Component framework (runes mode)                                         |
| Vite                                            | ^8.1.4  | Bundler; dev server at port 1420 (`vite.config.ts`)                      |
| `@sveltejs/adapter-static`                      | ^3.0.10 | Static export (fallback `index.html`)                                    |
| Tailwind CSS                                    | ^4.3.2  | Styling via `@tailwindcss/vite`                                          |
| bits-ui                                         | ^2.18.1 | Headless UI primitives                                                   |
| `@lucide/svelte`                                | ^1.24.0 | Icons                                                                    |
| `marked`                                        | ^18.0.6 | Markdown rendering                                                       |
| `shiki`                                         | ^4.3.1  | Syntax highlighting                                                      |
| `dompurify`                                     | ^3.4.12 | DOM sanitization                                                         |
| `@tauri-apps/api`                               | ^2.11.1 | Tauri IPC bindings                                                       |
| `@tauri-apps/plugin-*`                          | ^2.*    | fs, dialog, shell, http, store                                           |
| `clsx` + `tailwind-merge` + `tailwind-variants` | —       | Class utilities                                                          |
| Prettier                                        | ^3.9.5  | Sole formatter (`prettier-plugin-svelte`, `prettier-plugin-tailwindcss`) |

## Backend Crates

| Crate                                     | Version     | Purpose                                      |
| ----------------------------------------- | ----------- | -------------------------------------------- |
| `tauri`                                   | 2           | Desktop framework (feature `protocol-asset`) |
| `tokio`                                   | 1           | Async runtime (`rt-multi-thread`)            |
| `rusqlite`                                | 0.40        | SQLite (bundled)                             |
| `rusqlite_migration`                      | 2.6         | Schema migrations                            |
| `rig-core`                                | 0.40        | Unified LLM client (15 provider types)       |
| `tiktoken-rs`                             | 0.12        | Token counting                               |
| `keyring` + `keyring-core`                | 4.0 / 1.0   | OS credential store                          |
| `serde` + `serde_json`                    | 1.0         | Serialization                                |
| `thiserror`                               | 2           | Typed `AppError` enum                        |
| `anyhow`                                  | 1           | Card import/export errors                    |
| `jiff`                                    | 0.2         | Datetime (serde)                             |
| `uuid`                                    | 1 (v4)      | IDs                                          |
| `futures`                                 | 0.3         | Future utilities                             |
| `tokio-util`                              | 0.7         | `CancellationToken`                          |
| `regex`                                   | 1           | Regex scripts                                |
| `png` + `base64`                          | 0.18 / 0.22 | Character card PNG I/O                       |
| `tauri-plugin-fs/dialog/shell/http/store` | 2.x         | Tauri plugins                                |
| `tauri-plugin-webdriver-automation`       | 0.1.3       | E2E (debug only)                             |

## Fonts

- **Geist** — sans-serif body (`src/app.html`)
- **Fraunces** — serif display for character names (`.font-display`)
- **Geist Mono** — monospace

## Styling System

- Tailwind v4 `@theme` tokens in **OKLCH** color space (`src/app.css:14`)
- Convention: `bg-(--color-surface)` shorthand (not `bg-surface` or `theme()`)
- Dark theme default (`<html class="dark">` in `src/app.html`)
- Custom utilities: `.font-display`, `.surface-grain`, `.streaming-caret`, `.drag-region`, `.no-drag`

## Package Layout

### Frontend (`src/`)

```
src/
├── app.css / app.html / app.d.ts        # Tailwind tokens, HTML shell, type augmentations
├── lib/
│   ├── components/{ui,layout,chat,characters,personas,lorebooks,providers,settings}/
│   ├── stores/app.svelte.ts            # AppState singleton (Svelte 5 runes)
│   ├── stores/toasts.svelte.ts
│   ├── data/api.ts                     # invoke wrappers + IS_DEV mocks
│   ├── data/mock.ts                    # Dev-mode mock implementations
│   ├── types/index.ts                  # Domain types mirroring Rust serde
│   ├── utils/index.ts                  # cn(), formatRelativeTime(), estimateTokens(), avatarInitials()
│   └── shiki.svelte.ts                 # Syntax highlighting
└── routes/{+page,+layout}.svelte, chat/[id]/, characters/, settings/
```

### Backend (`src-tauri/src/`)

```
src-tauri/src/
├── lib.rs               # Tauri builder, DbWorker start, invoke_handler
├── main.rs              # Entry point (windows_subsystem closure)
├── error.rs             # AppError enum (thiserror + Serialize)
├── commands/            # 13 command modules (one per domain)
├── domain/              # Pure data types (serde camelCase)
├── service/             # PromptEngine, token counting, regex, lorebook, card_io
├── ai/mod.rs            # AiService (rig-core streaming)
├── db/                  # DbWorker actor + CRUD modules + migrations
└── secrets/mod.rs       # KeyringStore (OS keychain)
```

Tauri capabilities live in `src-tauri/capabilities/*.json` (default, filesystem, dialog, shell, http, store, window).

## Entry Points

- **Rust**: `src-tauri/src/main.rs:4` → calls `revelry_lib::run()` → Tauri builder in `src-tauri/src/lib.rs:34`
- **Frontend HTML shell**: `src/app.html` (font links, theme preload)
- **Root layout**: `src/routes/+layout.svelte` (hydrates AppState, binds F9 sidebar toggle)
- **Bundler**: Vite dev server at port 1420 (`vite.config.ts`); Tauri dev URL matches.

## Commands

```sh
bun dev              # Frontend dev (port 5173, dev mocks)
bun run build        # Production frontend build
cargo check         # Rust type-check (in src-tauri/)
cargo test          # Rust tests
bun tauri dev       # Full app (Rust + frontend)
bun tauri build     # Production desktop build
```
