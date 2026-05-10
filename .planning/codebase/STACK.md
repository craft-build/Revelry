# Technology Stack

**Analysis Date:** 2026-05-10

## Languages

**Primary:**
- TypeScript — SvelteKit UI layer (`src/lib/`, `src/routes/`), type definitions (`src/lib/types/index.ts`), stores (`src/lib/stores/app.svelte.ts`), utilities (`src/lib/utils/index.ts`)
- Svelte 5 (`.svelte` components) — UI components (`src/lib/components/`, `src/routes/`)
- CSS (Tailwind v4) — Theme tokens and base styles (`src/app.css`)
- HTML — App shell (`src/app.html`)

**Secondary — Archival Reference (`gtk_ref/`):**
- Rust — GTK4/libadwaita reference backend (`gtk_ref/src/`), domain models, AI service, database layer, secrets. Preserved as patterns for future Tauri backend; NOT currently active.

## Runtime

**Active Environment:**
- Bun (lockfile: `bun.lock` present)
- Browser — SPA runtime, SSR disabled
- Node 20+ (alternative runtime, required per `README.md`)

**Archival Reference Environment (`gtk_ref/`):**
- Rust (edition 2021 per `gtk_ref/Cargo.toml`) — NOT currently active

**Package Manager:**
- Bun — `bun.lock` present at project root (active)
- Cargo — `Cargo.lock` present in `gtk_ref/` (archival reference only)

## Frameworks

**Active Core:**
- SvelteKit ^2.59 — SPA framework for the UI layer; SSR disabled (`src/routes/+layout.ts`)
- Svelte ^5.55 — Component framework using runes (`$state`, `$derived`, `$effect`)
- Vite ^8.0 — Build tool and dev server (`vite.config.ts`)
- Tailwind CSS ^4.3 — Utility-first CSS framework (`src/app.css`, `vite.config.ts`)

**Active Testing:**
- svelte-check ^4.4 — Type checking for Svelte components (`package.json` scripts)

**Active Build/Dev:**
- @tailwindcss/vite ^4.3 — Tailwind v4 Vite plugin
- @sveltejs/vite-plugin-svelte ^7.1 — Svelte Vite plugin

**Archival Reference Frameworks (`gtk_ref/`) — NOT currently active:**
- GTK4 ^0.11.3 + libadwaita ^0.9.1 — Native UI for Rust reference backend (`gtk_ref/Cargo.toml`)
- Relm4 ^0.11.0 — Elm-architecture declarative UI framework for GTK4 (`gtk_ref/Cargo.toml`)
- Rust built-in `#[test]` — Unit tests in `gtk_ref/src/` (e.g., `provider_test.rs`, `secrets/mod.rs`)
- glib-build-tools ^0.22.0 — GResource compilation for GTK4 (`gtk_ref/build.rs`)
- relm4-icons-build ^0.11.0 — Icon bundling for Relm4 (`gtk_ref/build.rs`)

## Key Dependencies

**Active — Critical:**
- bits-ui ^2.18 — Headless Svelte component library (Avatar, Popover, Tooltip, Tabs); full styling control for Adwaita aesthetic (`package.json`)
- @lucide/svelte ^1.14 — Icon library for Svelte UI (`package.json`)
- clsx ^2.1 + tailwind-merge ^3.5 — Class name merging for components (`src/lib/utils/index.ts`)
- tailwind-variants ^3.2 — Variant-based component styling (`package.json`)

**Archival Reference Dependencies (`gtk_ref/`) — Preserved as patterns for future Tauri work, NOT currently in use:**
- rig-core ^0.36.0 — Provider-agnostic AI abstraction; unified clients for 15+ providers with streaming support (`gtk_ref/Cargo.toml`)
- rusqlite ^0.39.0 — SQLite bindings with `bundled` feature for zero-config cross-platform builds (`gtk_ref/Cargo.toml`)
- tokio ^1.52 — Async runtime for rig-core AI calls and background DB operations (`gtk_ref/Cargo.toml`)
- keyring ^3.6.3 — OS-native credential storage (macOS Keychain, GNOME Keyring, Windows Credential Manager) (`gtk_ref/Cargo.toml`)
- tiktoken-rs ^0.11.0 — BPE tokenization for context window budget tracking (`gtk_ref/Cargo.toml`)
- pulldown-cmark ^0.13.3 — Markdown→events parser for AI response rendering (`gtk_ref/Cargo.toml`)
- aes-gcm ^0.10.3 — AES-256-GCM encryption for vault mode (`gtk_ref/Cargo.toml`)
- argon2 ^0.5.3 — Key derivation (Argon2id) for vault encryption (`gtk_ref/Cargo.toml`)
- tracing ^0.1.44 + tracing-subscriber ^0.3.23 — Structured logging in Rust backend (`gtk_ref/Cargo.toml`)
- anyhow ^1.0.102 + thiserror ^2.0.18 — Error handling in Rust (`gtk_ref/Cargo.toml`)

## Configuration

**Active:**
- SSR disabled via `src/routes/+layout.ts` (`export const ssr = false`)
- `.env` file listed in `.gitignore` (exists but forbidden to read)
- `vite.config.ts` — Vite configuration with SvelteKit + Tailwind plugins, port 5173
- `svelte.config.js` — SvelteKit config with adapter-auto and `$lib` alias
- `tsconfig.json` — TypeScript strict mode, bundler module resolution

**Archival Reference (`gtk_ref/`) — NOT currently active:**
- `gtk_ref/Cargo.toml` — Rust dependencies with platform-specific keyring features
- `gtk_ref/build.rs` — GTK icon/resource bundling
- OS keychain pattern for API keys (security invariant from `gtk_ref/src/secrets/mod.rs` — will inform future Tauri backend)
- Provider base URL defaults (pattern from `gtk_ref/src/domain/provider.rs` — will inform future Tauri backend)

## Platform Requirements

**Active Development:**
- Node 20+ or Bun runtime
- Browser — SPA runtime for development and current production
- Network access for initial package install (Bits.ui, Tailwind v4, SvelteKit, lucide)

**Archival Reference Build Only (`gtk_ref/`) — NOT required for active development:**
- GTK4/libadwaita development libraries (only if building `gtk_ref/`)
- Rust toolchain (only if building `gtk_ref/`)

**Production — Current:**
- Browser SPA — current deployment target

**Production — Planned:**
- Tauri desktop app — webview embedding the SvelteKit SPA (per `README.md`)
- Local-first: SQLite database stored in platform data directory (pattern from `gtk_ref/` `dirs` crate, XDG on Linux, ~/Library on macOS, %APPDATA% on Windows — will inform future Tauri backend)

---

*Stack analysis: 2026-05-10*
