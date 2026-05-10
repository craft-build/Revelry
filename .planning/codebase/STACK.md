# Technology Stack

**Analysis Date:** 2026-05-10

## Languages

**Primary:**
- TypeScript — SvelteKit UI layer (`src/lib/`, `src/routes/`), type definitions (`src/lib/types/index.ts`), stores (`src/lib/stores/app.svelte.ts`), utilities (`src/lib/utils/index.ts`)

**Secondary:**
- Rust — GTK4/libadwaita reference backend (`gtk_ref/src/`), domain models, AI service, database layer, secrets
- Svelte 5 (`.svelte` components) — UI components (`src/lib/components/`, `src/routes/`)
- CSS (Tailwind v4) — Theme tokens and base styles (`src/app.css`)
- HTML — App shell (`src/app.html`)

## Runtime

**Environment:**
- Bun (lockfile: `bun.lock` present)
- Node 20+ (required per `README.md`)
- Rust (edition 2021 per `gtk_ref/Cargo.toml`)

**Package Manager:**
- Bun — `bun.lock` present at project root
- Cargo — `Cargo.lock` present in `gtk_ref/`
- Lockfile: present (both `bun.lock` and `gtk_ref/Cargo.lock`)

## Frameworks

**Core:**
- SvelteKit ^2.59 — SPA framework for the UI layer; SSR disabled (`src/routes/+layout.ts`)
- Svelte ^5.55 — Component framework using runes (`$state`, `$derived`, `$effect`)
- Vite ^8.0 — Build tool and dev server (`vite.config.ts`)
- GTK4 ^0.11.3 + libadwaita ^0.9.1 — Native UI for Rust reference backend (`gtk_ref/Cargo.toml`)
- Relm4 ^0.11.0 — Elm-architecture declarative UI framework for GTK4 (`gtk_ref/Cargo.toml`)
- Tailwind CSS ^4.3 — Utility-first CSS framework (`src/app.css`, `vite.config.ts`)

**Testing:**
- Rust built-in `#[test]` — Unit tests in `gtk_ref/src/` (e.g., `provider_test.rs`, `secrets/mod.rs`)
- svelte-check ^4.4 — Type checking for Svelte components (`package.json` scripts)

**Build/Dev:**
- Vite ^8.0 — Dev server, HMR, production builds
- @tailwindcss/vite ^4.3 — Tailwind v4 Vite plugin
- @sveltejs/vite-plugin-svelte ^7.1 — Svelte Vite plugin
- glib-build-tools ^0.22.0 — GResource compilation for GTK4 (`gtk_ref/build.rs`)
- relm4-icons-build ^0.11.0 — Icon bundling for Relm4 (`gtk_ref/build.rs`)

## Key Dependencies

**Critical:**
- bits-ui ^2.18 — Headless Svelte component library (Avatar, Popover, Tooltip, Tabs); full styling control for Adwaita aesthetic (`package.json`)
- rig-core ^0.36.0 — Provider-agnostic AI abstraction; unified clients for 15+ providers with streaming support (`gtk_ref/Cargo.toml`)
- rusqlite ^0.39.0 — SQLite bindings with `bundled` feature for zero-config cross-platform builds (`gtk_ref/Cargo.toml`)
- @lucide/svelte ^1.14 — Icon library for Svelte UI (`package.json`)

**Infrastructure:**
- tokio ^1.52 — Async runtime for rig-core AI calls and background DB operations (`gtk_ref/Cargo.toml`)
- keyring ^3.6.3 — OS-native credential storage (macOS Keychain, GNOME Keyring, Windows Credential Manager) (`gtk_ref/Cargo.toml`)
- tiktoken-rs ^0.11.0 — BPE tokenization for context window budget tracking (`gtk_ref/Cargo.toml`)
- pulldown-cmark ^0.13.3 — Markdown→events parser for AI response rendering (`gtk_ref/Cargo.toml`)
- aes-gcm ^0.10.3 — AES-256-GCM encryption for vault mode (`gtk_ref/Cargo.toml`)
- argon2 ^0.5.3 — Key derivation (Argon2id) for vault encryption (`gtk_ref/Cargo.toml`)
- clsx ^2.1 + tailwind-merge ^3.5 — Class name merging for components (`src/lib/utils/index.ts`)
- tailwind-variants ^3.2 — Variant-based component styling (`package.json`)
- tracing ^0.1.44 + tracing-subscriber ^0.3.23 — Structured logging in Rust backend (`gtk_ref/Cargo.toml`)
- anyhow ^1.0.102 + thiserror ^2.0.18 — Error handling in Rust (`gtk_ref/Cargo.toml`)

## Configuration

**Environment:**
- SSR disabled via `src/routes/+layout.ts` (`export const ssr = false`)
- `.env` file listed in `.gitignore` (exists but forbidden to read)
- OS keychain for API keys (never stored in DB or config files — security invariant from `gtk_ref/src/secrets/mod.rs`)
- Provider base URLs configured per-provider with defaults in `gtk_ref/src/domain/provider.rs`

**Build:**
- `vite.config.ts` — Vite configuration with SvelteKit + Tailwind plugins, port 5173
- `svelte.config.js` — SvelteKit config with adapter-auto and `$lib` alias
- `tsconfig.json` — TypeScript strict mode, bundler module resolution
- `gtk_ref/Cargo.toml` — Rust dependencies with platform-specific keyring features
- `gtk_ref/build.rs` — GTK icon/resource bundling

## Platform Requirements

**Development:**
- Node 20+ or Bun runtime
- Network access for initial package install (Bits.ui, Tailwind v4, SvelteKit, lucide)
- GTK4/libadwaita development libraries (for building `gtk_ref/`)
- Rust toolchain (for building `gtk_ref/`)

**Production:**
- Desktop target: Tauri webview embedding the SvelteKit SPA (planned, per `README.md`)
- Alternative: Any HTTP backend exposing data shapes from `src/lib/types/index.ts`
- Rust backend: GTK4/libadwaita native desktop app (Linux, macOS, Windows)
- Local-first: SQLite database stored in platform data directory (`dirs` crate, XDG on Linux, ~/Library on macOS, %APPDATA% on Windows)

---

*Stack analysis: 2026-05-10*
