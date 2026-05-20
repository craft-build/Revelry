# Phase 10: Theme & Polish — Summary

**Status:** Complete
**Date:** 2026-05-19
**Commits:** 4

## What was done

### Task 1: Backend — app_settings DB functions + DbCommand variants + Tauri commands

- Created `src-tauri/src/db/app_settings.rs` with `load_app_setting` and `save_app_setting`
- Added `LoadAppSetting` and `SaveAppSetting` variants to `DbCommand` enum
- Created `src-tauri/src/commands/app_settings.rs` with `get_app_setting` and `set_app_setting` Tauri commands
- Added TypeScript API wrappers `getAppSetting()` and `setAppSetting()` to `api.ts`
- No new migration needed — `app_settings` table exists since Phase 2

### Task 2: CSS — Light theme token overrides + surface grain light adjustment

- Added `html.light { }` block with 17 OKLCH color token overrides (cool-toned 270 hue family, soft/off-white surfaces)
- Added `html.light .surface-grain::before` with reduced opacity (0.3) and cooler gradients
- Added `transition: background-color 150ms ease-out, color 150ms ease-out` to `body`

### Task 3: Flash prevention + AppState theme infrastructure + Shiki dual theme

- Replaced `class="dark"` on `<html>` with inline flash-prevention script that reads `localStorage("revelry-theme")` and sets class before paint
- Created `src/lib/shiki.svelte.ts` singleton that loads both `vitesse-dark` and `vitesse-light` themes
- Added `theme`, `resolved_theme`, and `setTheme()` to AppState
- `hydrate()` loads theme from backend and syncs `<html>` class
- Layout sets up `matchMedia('(prefers-color-scheme: dark)')` change listener
- MarkdownRenderer uses `app.resolved_theme` to select correct Shiki theme

### Task 4: ThemeToggle component + Settings page integration + Polish audit

- Created `ThemeToggle.svelte` with three-way segmented button (System/Light/Dark)
- Uses proper `role="radiogroup"` / `role="radio"` / `aria-checked` accessibility
- Replaced settings page appearance stub with working ThemeToggle
- Added `transition:fade={{ duration: 150 }}` to MessageBubble
- Polish audit confirmed `transition-colors` present on all interactive elements

## Verification

- `cargo build` succeeds (29 pre-existing warnings, 0 errors)
- `bun run check` passes (0 errors, 48 pre-existing warnings)
- `bun run build` succeeds

## Key Decisions

- **localStorage primary, backend backup.** Flash-prevention script reads localStorage synchronously before paint. Backend `app_settings` table written async as backup.
- **Class-based switching.** Theme controlled by class on `<html>` (`"light"` or `"dark"`). No `@media (prefers-color-scheme)` CSS rules — media query only used in JS to resolve "system".
- **Shiki singleton.** Both layout preload and MarkdownRenderer use `getShikiHighlighter()` from `src/lib/shiki.svelte.ts`. No component calls `createHighlighter()` directly.
