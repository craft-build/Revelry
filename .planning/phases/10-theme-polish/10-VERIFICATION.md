# Phase 10 Verification — Theme Polish

**Phase goal:** Users can toggle between light and dark themes, and the app respects system appearance preference with manual override.

**Requirement IDs:** SETT-01, SETT-02

**Date:** 2026-05-19

---

## Must-Have Checklist

### 1. `app.css` contains an `html.light { ... }` block that overrides all color tokens

**Status: PASS**

`src/app.css` contains a complete `html.light { ... }` block (lines ~73-109). Both the `@theme { }` default block and the `html.light` block override the same 18 color tokens:

| Token                      | @theme (dark)   | html.light      |
| -------------------------- | --------------- | --------------- |
| `--color-background`       | oklch(0.16 ...) | oklch(0.97 ...) |
| `--color-surface`          | oklch(0.19 ...) | oklch(0.96 ...) |
| `--color-surface-elevated` | oklch(0.22 ...) | oklch(0.94 ...) |
| `--color-surface-hover`    | oklch(0.24 ...) | oklch(0.92 ...) |
| `--color-surface-input`    | oklch(0.21 ...) | oklch(0.95 ...) |
| `--color-foreground`       | oklch(0.96 ...) | oklch(0.18 ...) |
| `--color-muted`            | oklch(0.65 ...) | oklch(0.5 ...)  |
| `--color-subtle`           | oklch(0.45 ...) | oklch(0.38 ...) |
| `--color-border`           | oklch(0.28 ...) | oklch(0.88 ...) |
| `--color-border-strong`    | oklch(0.36 ...) | oklch(0.78 ...) |
| `--color-accent`           | oklch(0.78 ...) | oklch(0.55 ...) |
| `--color-accent-fg`        | oklch(0.18 ...) | oklch(0.97 ...) |
| `--color-accent-muted`     | oklch(0.32 ...) | oklch(0.88 ...) |
| `--color-destructive`      | oklch(0.66 ...) | oklch(0.55 ...) |
| `--color-warning`          | oklch(0.78 ...) | oklch(0.68 ...) |
| `--color-success`          | oklch(0.72 ...) | oklch(0.55 ...) |
| `--color-bubble-user`      | oklch(0.26 ...) | oklch(0.93 ...) |
| `--color-bubble-character` | oklch(0.21 ...) | oklch(0.96 ...) |

The light-mode `.surface-grain::before` override is also present. All tokens use OKLCH color space, matching the project convention.

---

### 2. `app.html` has an inline flash-prevention script in `<head>` that reads localStorage and sets class on `<html>` before paint

**Status: PASS**

`src/app.html` contains an inline `<script>` block as the first child of `<head>` (before any stylesheets or fonts). The script:

1. Reads `localStorage.getItem("revelry-theme")` (defaults to `"system"`)
2. If `"system"`, checks `window.matchMedia("(prefers-color-scheme: light)")` to resolve
3. Sets `document.documentElement.classList.add(resolved)` — adds `"light"` or `"dark"`
4. Falls back to `"dark"` on any error

This runs synchronously before paint, preventing a flash of wrong theme.

---

### 3. `AppState` has a `theme` field, `resolved_theme` getter, and `setTheme()` action that persists to both localStorage and backend

**Status: PASS**

Verified in `src/lib/stores/app.svelte.ts`:

- **`theme` field:** `theme = $state<"system" | "light" | "dark">("system")` (line ~56)
- **`resolved_theme` getter:** Returns `"light"` or `"dark"`, resolving `"system"` via `window.matchMedia("(prefers-color-scheme: light)")`. Falls back to `"dark"` in SSR (line ~98)
- **`setTheme()` action:** Sets `this.theme`, writes to `localStorage.setItem("revelry-theme", theme)`, updates `document.documentElement.className`, and async-persists via `apiSetAppSetting("theme", theme)` (lines ~273-282)

Backend persistence chain verified:

- `apiSetAppSetting` in `src/lib/data/api.ts` invokes `set_app_setting` Tauri command
- `src-tauri/src/commands/app_settings.rs` maps to `DbCommand::SaveAppSetting`
- `src-tauri/src/db/app_settings.rs` upserts into `app_settings` table
- `src-tauri/src/db/migrations.rs` defines `CREATE TABLE app_settings (key TEXT PRIMARY KEY, value TEXT NOT NULL)`

On app load, `hydrate()` calls `apiGetAppSetting("theme")` and syncs `document.documentElement.className` with the resolved theme.

---

### 4. Settings > Appearance section contains a working three-way ThemeToggle (System/Light/Dark) that instantly changes the app theme

**Status: PASS**

Verified in:

- **`src/routes/settings/+page.svelte`:** Contains an "Appearance" section (`active === 'appearance'`) with header "Theme", description, and `<ThemeToggle />` component (lines ~131-139)
- **`src/lib/components/settings/ThemeToggle.svelte`:** Renders a three-way segmented control with:
  - System (Monitor icon) — `aria-label="Follow system preference"`
  - Light (Sun icon) — `aria-label="Light theme"`
  - Dark (Moon icon) — `aria-label="Dark theme"`
  - Uses `role="radiogroup"` and `role="radio"` with `aria-checked` for accessibility
  - Calls `app.setTheme(option.value)` on click
  - Shows annotation text below indicating current state (e.g., "Following your system preference (dark)")

Theme changes propagate instantly via `setTheme()` which updates `document.documentElement.className` synchronously.

---

### 5. Shiki loads both `vitesse-dark` and `vitesse-light` themes and MarkdownRenderer selects the correct one based on resolved theme

**Status: PASS**

Verified in:

- **`src/lib/shiki.svelte.ts`:** Creates a singleton Shiki highlighter loaded with both `"vitesse-dark"` and `"vitesse-light"` themes and 9 languages. Exported via `getShikiHighlighter()`.
- **`src/lib/components/chat/MarkdownRenderer.svelte`:** In the `$effect` that re-renders markdown, the code block handler reads `app.resolved_theme` and selects:
  - `'vitesse-light'` when `app.resolved_theme === 'light'`
  - `'vitesse-dark'` otherwise (dark)
  - Checks `hl.getLoadedThemes().includes(shikiTheme)` before using Shiki, with fallback to plain code block
- **`src/routes/+layout.svelte`:** Preloads the Shiki highlighter on mount via `getShikiHighlighter().catch(...)` so both themes are available when needed.

---

### 6. `matchMedia` listener detects OS preference changes and auto-updates when theme is "system"

**Status: PASS**

Verified in `src/routes/+layout.svelte` (onMount):

```ts
const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
const onMediaChange = () => {
  if (app.theme === "system") {
    document.documentElement.className = app.resolved_theme;
  }
};
mediaQuery.addEventListener("change", onMediaChange);
```

The listener is properly cleaned up in the returned cleanup function. When `app.theme === 'system'` and the OS preference changes, `app.resolved_theme` re-evaluates (it reads `matchMedia` live), and `document.documentElement.className` is updated to match.

---

### 7. No flash of wrong theme on app load

**Status: PASS**

Verified by the flash-prevention mechanism:

1. **Before paint:** The inline `<script>` in `app.html` `<head>` runs synchronously, reads `localStorage("revelry-theme")`, resolves system preference via `matchMedia`, and sets the correct class on `<html>` before any CSS renders.
2. **After JS hydration:** `app.hydrate()` loads the theme from backend and calls `document.documentElement.className = this.resolved_theme`, ensuring consistency even if backend has a different value.
3. **CSS transitions:** `body` has `transition: background-color 150ms ease-out, color 150ms ease-out` for smooth theme switching, but no initial flash because the class is set before first paint.

---

## Requirement Cross-Reference

| Requirement ID | Description                                                    | Status | Evidence                                                                                                                                                                                                  |
| -------------- | -------------------------------------------------------------- | ------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| SETT-01        | Users can toggle between light and dark themes                 | PASS   | ThemeToggle component in Settings > Appearance with three-way control (System/Light/Dark); `app.setTheme()` updates localStorage + backend + DOM class instantly                                          |
| SETT-02        | App respects system appearance preference with manual override | PASS   | `resolved_theme` getter resolves `"system"` to OS preference via `matchMedia`; flash-prevention script resolves system before paint; `matchMedia` listener in layout auto-updates on OS preference change |

---

## Files Verified

| File                                              | Status   | Notes                                                                                             |
| ------------------------------------------------- | -------- | ------------------------------------------------------------------------------------------------- |
| `src/app.css`                                     | Modified | Contains `html.light { ... }` block with 18 color token overrides + light-mode `.surface-grain`   |
| `src/app.html`                                    | Modified | Inline flash-prevention script in `<head>` before any resources                                   |
| `src/lib/stores/app.svelte.ts`                    | Modified | `theme` field, `resolved_theme` getter, `setTheme()` action, `hydrate()` loads theme from backend |
| `src/lib/shiki.svelte.ts`                         | New      | Singleton Shiki highlighter loaded with both `vitesse-dark` and `vitesse-light`                   |
| `src/routes/+layout.svelte`                       | Modified | `matchMedia` listener for OS preference changes, Shiki preload                                    |
| `src/lib/components/settings/ThemeToggle.svelte`  | New      | Three-way segmented control with System/Light/Dark, accessible radio group                        |
| `src/routes/settings/+page.svelte`                | Modified | Appearance section with ThemeToggle                                                               |
| `src/lib/components/chat/MarkdownRenderer.svelte` | Modified | Selects Shiki theme based on `app.resolved_theme`                                                 |
| `src/lib/components/chat/MessageBubble.svelte`    | Modified | Uses `MarkdownRenderer` which respects theme                                                      |
| `src/lib/data/api.ts`                             | Modified | `getAppSetting` and `setAppSetting` API wrappers                                                  |
| `src-tauri/src/db/app_settings.rs`                | New      | `load_app_setting` / `save_app_setting` with SQLite upsert                                        |
| `src-tauri/src/commands/app_settings.rs`          | New      | `get_app_setting` / `set_app_setting` Tauri commands                                              |
| `src-tauri/src/db/mod.rs`                         | Modified | `DbCommand::LoadAppSetting` / `DbCommand::SaveAppSetting` variants + dispatch                     |
| `src-tauri/src/lib.rs`                            | Modified | Registers `get_app_setting` / `set_app_setting` commands                                          |

---

## Summary

**All 7 must-haves: PASS**

Phase 10 is fully implemented. The theme system works end-to-end:

- **Flash-free loading** via inline script in `<head>`
- **Three-way toggle** (System / Light / Dark) in Settings > Appearance
- **System preference tracking** via `matchMedia` listener
- **Full light theme** with 18 OKLCH color token overrides
- **Backend persistence** via `app_settings` table through Tauri IPC
- **Code block theming** via Shiki dual-theme support in MarkdownRenderer

**SETT-01: PASS**
**SETT-02: PASS**
