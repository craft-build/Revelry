---
type: Decision
description: "Conventions, patterns, and rationale: Svelte 5 runes, AppState singleton, Tauri IPC, DbWorker, keyring, OKLCH styling, error handling, testing."
tags:
  - conventions
  - decisions
  - svelte-runes
  - tauri-ipc
  - oklch
  - error-handling
timestamp: 2026-07-12T18:26:11Z
---

# Design Decisions

Conventions and rationale for Revelry, primarily codified in `AGENTS.md` at the project root.

## Single Source of Truth — `AppState`

All frontend state lives in one Svelte 5 rune-based class instance (`app`) in `src/lib/stores/app.svelte.ts`. This mirrors the Elm/Relm4 AppModel pattern from the design spec: components read from and write to `app`; `app` methods call `api.ts`; nothing else calls `invoke()`.

- No legacy `svelte/store` — never `writable()`, `readable()`, or `derived()`.
- State fields use `$state()`; computed values use `$derived`/`get` getters; side effects use `$effect()`.

## Svelte 5 Runes Are Mandatory

| Do                                              | Don't                                           |
| ----------------------------------------------- | ----------------------------------------------- |
| `$state()` for reactive state                   | `writable()` / `readable()` from `svelte/store` |
| `$derived` / `get` for computed                 | `derived()` from `svelte/store`                 |
| `$effect()` for side effects                    | `$:` reactive statements                        |
| `$props()` for component props                  | `export let` (legacy)                           |
| `children?: Snippet` + `{@render children?.()}` | `<slot />` (legacy)                             |
| `.svelte.ts` files (support runes)              | plain `.ts` files (do not)                      |

## Component & Variant Conventions

- Props: `let { class: className, children, ...rest }: Props = $props();`
- Every component accepting `class` merges it through `cn()` (from `$lib/utils`).
- Components exposing variant helpers use a `<script lang="ts" module>` block exporting e.g. `buttonVariants` via `tv()` so consumers can import it without importing the component.
- UI primitives are re-exported from barrel files (`src/lib/components/ui/index.ts`, per-scope `index.ts`).

## TypeScript Strict Mode

`tsconfig.json:11` sets `"strict": true`. Avoid `any`; use types from `$lib/types`.

## Backend Is Not Optional

The Rust backend is live and wired, not planned. Any feature involving data persistence, AI calls, file I/O, or API keys requires implementing both the Rust command(s) and the frontend wiring. Dev-mode mocks in `api.ts` are supplementary convenience, not a replacement.

## Tauri IPC Conventions

- **Command naming**: Rust functions are snake_case (e.g. `generate_character`); the invoke name is that snake_case name. Tauri v2 auto-converts arg names to camelCase for the frontend.
- **Serde**: Rust domain types use `#[derive(Serialize)]` + `#[serde(rename_all = "camelCase")]`.
- **Streaming chunks**: `#[serde(tag = "event", content = "data")]` enums must produce JSON matching the TypeScript discriminated union exactly (both camelCase). Tested via serde unit tests (see `src-tauri/src/ai/mod.rs` tests).

## Rust Patterns

- **DbWorker actor** — never call `rusqlite` directly; send `DbCommand` via `db.send()` and await the oneshot reply.
- **Keyring** — `KeyringStore` calls are synchronous; always wrap in `tokio::task::spawn_blocking()`.
- **Cancellation** — chat and generation streams use `CancelState` / `GenerateCancelState` (`Mutex<Option<CancellationToken>>`). Starting a new stream cancels the existing one.
- **Errors** — everything flows through `AppError` (thiserror) which `Serialize`s to a display string; the frontend catches these as string errors from `invoke()`.

## No Secrets in Code or DB

API keys live exclusively in the OS keychain via `KeyringStore`. The frontend never sees raw keys — only `has_api_key: boolean`. Never hardcode keys; never send keys to the frontend.

## Styling System

- **Tailwind v4** with `@theme` tokens in **OKLCH** color space (`src/app.css`).
- Project convention: `bg-(--color-surface)` shorthand (not `bg-surface` or `theme()`).
- **Dark theme is default** (`<html class="dark">` in `src/app.html`); light theme is stubbed.
- When adding color tokens, use OKLCH for consistency.
- Custom utilities: `.font-display`, `.surface-grain`, `.streaming-caret`, `.drag-region`, `.no-drag`.
- **Body scroll disabled** (`overflow: hidden` in `app.css`) — scroll happens inside component containers for the desktop app feel.

## SSR Disabled

`src/routes/+layout.ts` sets `export const ssr = false` and `export const prerender = false`. This is a desktop SPA built with `@sveltejs/adapter-static`.

## Formatting & Linting

- **Frontend**: Prettier is the sole formatter (`prettier-plugin-svelte`, `prettier-plugin-tailwindcss`). Run `bun run format` before committing style changes.
- **Rust**: `cargo fmt` (standard rustfmt) in `src-tauri/`.
- No ESLint config, no `CONTRIBUTING.md`, no `justfile` — TypeScript strict + Prettier `--check` and `cargo clippy` cover the gaps.

## Testing

- **Rust**: `#[cfg(test)]` module blocks inside source files (not separate test dirs). Examples in `src-tauri/src/ai/mod.rs` and `src-tauri/src/domain/*.rs`. Run `cargo test` in `src-tauri/`.
- **Frontend**: no dedicated test runner documented; `bun run check` (svelte-check) is the type-check gate.

## Dev-Mode Mock Fallback

`api.ts` gates mocks behind `IS_DEV` (`!("__TAURI_INTERNALS__" in window)`). Not every function has a mock; functions without mocks throw in dev mode, which is acceptable for features requiring the backend (keyring, file I/O). Add mocks only for features meant to be usable during frontend-only development.

## Notable Decisions

- **Single `AppState`** rather than many stores — one source of truth, simpler data flow.
- **Backend not optional** — features must ship both sides.
- **OKLCH colors** — perceptual uniformity across the palette.
- **Streaming chunk type parity** — Rust serde tag enums must match the TS union byte-for-byte.
