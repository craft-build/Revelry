# Phase 10 Research: Theme & Polish

**Phase:** 10-theme-polish
**Status:** Research complete
**Date:** 2026-05-19

---

## Summary

Phase 10 adds light/dark theme switching with system preference detection, plus a full visual polish pass. The implementation is well-scoped: CSS token overrides, one new component, two new backend Tauri commands, and an audit of all existing components for transitions and consistency. The `app_settings` table already exists in the schema with no Tauri commands wired to it yet — this phase fills that gap.

---

## 1. What Must Be True (Success Criteria → Requirements)

| #   | Success Criterion                                              | Requirement IDs |
| --- | -------------------------------------------------------------- | --------------- |
| 1   | User can toggle between dark and light themes                  | SETT-01         |
| 2   | App respects system appearance preference with manual override | SETT-02         |

Both are fully covered by the three-way toggle (System / Light / Dark) with `matchMedia` detection.

---

## 2. Backend: app_settings Tauri Commands

### Current State

- **Table exists:** `app_settings` (key TEXT PRIMARY KEY, value TEXT NOT NULL) — created in migration 2 (`src-tauri/src/db/migrations.rs:25`)
- **No Tauri commands:** The table has zero IPC commands wired to it. No `DbCommand` variants, no `commands/app_settings.rs`, no registration in `lib.rs`.
- **Reference implementation exists:** `gtk_ref/src/db/providers.rs:79-97` has `save_app_setting()` and `load_app_setting()` functions — directly portable.
- **No new migration needed:** The table already exists.

### What Must Be Added

1. **`src-tauri/src/db/app_settings.rs`** — Two functions: `load_app_setting(conn, key) -> Result<Option<String>>` and `save_app_setting(conn, key, value) -> Result<()>`. Port from `gtk_ref/src/db/providers.rs:79-97` verbatim (same SQL, same pattern).

2. **`src-tauri/src/db/mod.rs`** — Add `pub mod app_settings;` and two new `DbCommand` variants:
   - `LoadAppSetting { key: String, reply: oneshot::Sender<anyhow::Result<Option<String>>> }`
   - `SaveAppSetting { key: String, value: String, reply: oneshot::Sender<anyhow::Result<()>> }`
   - Add match arms in `handle_command()` calling the new module functions.

3. **`src-tauri/src/commands/app_settings.rs`** — New file with two `#[tauri::command]` functions:
   - `get_app_setting(key: &str, db: State<'_, DbWorker>) -> Result<Option<String>, String>`
   - `set_app_setting(key: &str, value: &str, db: State<'_, DbWorker>) -> Result<(), String>`
   - Pattern: identical to existing commands (e.g., `commands/providers.rs`).

4. **`src-tauri/src/commands/mod.rs`** — Add `pub mod app_settings;`

5. **`src-tauri/src/lib.rs`** — Register in `invoke_handler`:
   - `commands::app_settings::get_app_setting`
   - `commands::app_settings::set_app_setting`

6. **`src/lib/data/api.ts`** — Add two wrappers:
   - `getAppSetting(key: string): Promise<string | null>` → `invoke("get_app_setting", { key })`
   - `setAppSetting(key: string, value: string): Promise<void>` → `invoke("set_app_setting", { key, value })`

### Pitfalls

- Tauri v2 uses camelCase arguments in TypeScript that auto-convert to snake_case for Rust. The `key` and `value` params match on both sides (single words, no case conversion needed).
- The `app_settings` table uses `ON CONFLICT(key) DO UPDATE` (upsert) — no need for insert-vs-update branching.

---

## 3. Frontend: Theme State

### AppState Changes (`src/lib/stores/app.svelte.ts`)

Add to `AppState` class:

```typescript
// New field
theme = $state<'system' | 'light' | 'dark'>('system');

// New derived getter
get resolved_theme(): 'light' | 'dark' {
  if (this.theme === 'system') {
    return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
  }
  return this.theme;
}

// New action
async setTheme(theme: 'system' | 'light' | 'dark') {
  this.theme = theme;
  localStorage.setItem('revelry-theme', theme);
  document.documentElement.className = this.resolved_theme;
  // Async backend persist — non-blocking
  setAppSetting('theme', theme).catch(err => console.error('Theme persist failed:', err));
}
```

### Hydration Changes

In `hydrate()`, add theme loading:

- Call `getAppSetting('theme')` alongside existing parallel loads.
- If backend returns a value, set `this.theme = value`.
- After hydration, sync `<html>` class with `resolved_theme`.
- Register `matchMedia('(prefers-color-scheme: dark)')` change listener via `$effect()` to re-resolve when OS preference changes (only matters when `theme === 'system'`).

### matchMedia Listener Pattern

Need a reactive mechanism that:

1. Listens to `prefers-color-scheme` changes at the app level.
2. When OS preference changes and `theme === 'system'`, re-applies the resolved class to `<html>`.
3. Best location: `+layout.svelte` using `onMount` + `addEventListener` on `matchMedia`, or an `$effect()` in AppState that watches `resolved_theme` and applies the class.

**Recommended:** Add a `$effect()` in AppState or `+layout.svelte` that:

```typescript
$effect(() => {
  const resolved = app.resolved_theme;
  document.documentElement.className = resolved;
});
```

Plus a `matchMedia` listener in `+layout.svelte` `onMount` that calls `document.documentElement.className = app.resolved_theme` when the media query changes (triggering Svelte reactivity by reading the getter).

---

## 4. CSS: Light Theme Token Overrides

### Current State (`src/app.css`)

- Single `@theme {}` block defines all dark tokens (17 color tokens, 3 font tokens, 4 radius tokens).
- All components reference tokens via `var(--color-*)` or `bg-(--color-*)` — theme switching propagates automatically once tokens are overridden.
- No `.light` or `.dark` selector blocks exist yet.

### Implementation

Add after the `@theme {}` block:

```css
html.light {
  --color-background: oklch(0.97 0.003 270);
  --color-surface: oklch(0.96 0.004 270);
  /* ... all 17 color tokens from 10-UI-SPEC.md ... */
}
```

**Key design decisions (from 10-CONTEXT.md):**

- Cool-toned palette, 270 hue family (D-01)
- Soft/off-white surfaces, oklch 0.94–0.97 (D-02) — Notion/Bear aesthetic, not Linear/Vercel
- Same OKLCH color space (D-03)
- Teal accent hue 195 carries over, darker for light backgrounds (D-04)

### What Does NOT Change

- Typography tokens (fonts) — same in both themes
- Radius tokens — same in both themes
- `@layer base`, `@layer utilities`, `@layer components` rules — all reference `var(--color-*)` so they adapt automatically
- `.prose-markdown` — uses token references, adapts automatically
- `.surface-grain::before` — hardcoded OKLCH values. Needs adjustment for light mode (lower opacity or different radial gradient colors)

### Surface Grain Adjustment

The `.surface-grain::before` pseudo-element uses hardcoded oklch values:

```css
radial-gradient(circle at 20% 30%, oklch(0.78 0.13 195 / 0.04), transparent 50%),
radial-gradient(circle at 80% 70%, oklch(0.78 0.13 280 / 0.03), transparent 50%);
```

These work for dark mode. For light mode, reduce opacity further or use different hue accents. Either:

- Override in `html.light .surface-grain::before` with lighter values, OR
- Make the gradients reference CSS variables (more complex, possibly overkill)

**Recommendation:** Add a `html.light .surface-grain::before` override with reduced opacity (0.02 / 0.015) since the visual effect is already subtle.

---

## 5. Flash Prevention (`app.html`)

### Current State

```html
<html lang="en" class="dark"></html>
```

Hardcoded `class="dark"`. No inline script.

### Implementation

Replace `class="dark"` with no class (removed). Add inline `<script>` in `<head>` before Google Fonts:

```html
<script>
  (function () {
    try {
      var theme = localStorage.getItem("revelry-theme") || "system";
      var resolved;
      if (theme === "system") {
        resolved = window.matchMedia("(prefers-color-scheme: light)").matches
          ? "light"
          : "dark";
      } else {
        resolved = theme;
      }
      document.documentElement.classList.add(resolved);
    } catch (e) {
      document.documentElement.classList.add("dark");
    }
  })();
</script>
```

**Requirements:**

- Must run before any CSS renders (before Google Fonts link, before `%sveltekit.head%`)
- No modules, no async, no external deps — pure synchronous IIFE
- Fallback to `dark` on error (matches current hardcoded behavior)
- The `@theme {}` block applies unconditionally; `html.light` overrides apply only when class is present

---

## 6. Shiki: Dual Theme Loading

### Current State — CRITICAL DUPLICATION BUG

There is a **duplicate Shiki highlighter initialization**:

1. **`+layout.svelte`** (line ~25): Creates a highlighter with `vitesse-dark` + 9 languages. But the result is thrown away (`.then(() => { /* Shiki engine cached internally */ })`) — it's just a preload.
2. **`MarkdownRenderer.svelte`** (line ~20): Creates its **own** highlighter with the same `vitesse-dark` + same 9 languages. This is the one actually used for rendering.

The layout preload and MarkdownRenderer create **separate** highlighter instances. The preload in layout is meant to warm the Shiki cache so MarkdownRenderer doesn't block, but since `createHighlighter` returns a new object each time, both calls initialize independently. This means:

- Double the WASM initialization time
- Double the grammar loading
- The layout preload's highlighter is never reused

### Implementation

1. **`+layout.svelte`:** Load both themes:

   ```typescript
   createHighlighter({
     themes: ["vitesse-dark", "vitesse-light"],
     langs: [
       /* same 9 langs */
     ],
   }).then((hl) => {
     // Store globally (e.g., window.__shiki or a Svelte context/store)
   });
   ```

   Theme `vitesse-light` is confirmed available in the installed shiki package.

2. **Share the highlighter** with MarkdownRenderer. Options:
   - **Option A (simplest):** Store in a module-level variable in a shared `.svelte.ts` file that both import.
   - **Option B:** Svelte context. Overkill for a singleton.
   - **Option C:** Keep the current pattern but accept the duplication and just add `vitesse-light` to both.

   **Recommendation:** Option A — create `src/lib/shiki.svelte.ts` that exports a `$state<Highlighter | null>(null)`. Layout sets it on creation. MarkdownRenderer reads from it (or creates its own if null for robustness).

3. **MarkdownRenderer:** Accept or read the resolved theme and select the appropriate Shiki theme name:

   ```typescript
   const shikiTheme =
     resolvedTheme === "light" ? "vitesse-light" : "vitesse-dark";
   ```

4. **Re-render on theme change:** Since `content` is already reactive via `$effect()`, and the highlighter/theme selection will also be reactive, markdown will re-render when theme changes. The `$effect()` that produces `renderedHtml` needs to depend on the resolved theme.

---

## 7. ThemeToggle Component

### Location

`src/lib/components/settings/ThemeToggle.svelte`

### Structure (from 10-UI-SPEC.md)

- Plain HTML three-way segmented button group (no bits-ui primitives needed)
- Icons: Monitor (System), Sun (Light), Moon (Dark) — all from @lucide/svelte
- Active state: `bg-(--color-surface-elevated) text-(--color-foreground) shadow-sm`
- Inactive state: `text-(--color-muted) hover:text-(--color-foreground)`
- Dynamic annotation below: changes based on selection

### Accessibility

- `role="radiogroup"` on container with `aria-label="Theme preference"`
- Each button: `role="radio"`, `aria-checked`, `aria-label`
- Keyboard navigation: arrow keys between buttons

---

## 8. Settings Page Integration

### Current State

```svelte
{:else if active === 'appearance'}
  <!-- header -->
  <div class="...">
    Theme controls coming soon. The current build is dark-only.
  </div>
```

Single stub div. Replace with:

- Theme sub-header
- Description text
- ThemeToggle component
- Dynamic annotation

### No New Sections

Appearance section currently only contains the theme stub. The "Theme, density, and reading comfort." description text stays. Only the stub div is replaced.

---

## 9. Polish Audit: Transitions & Animations

### Current Transition Coverage

| Component                    | Transition                                  | Status               |
| ---------------------------- | ------------------------------------------- | -------------------- |
| ConversationSidebar          | `transition-[width] duration-200`           | Has width transition |
| Settings nav buttons         | `transition-colors`                         | Has color transition |
| CharacterCard                | `transition-all`                            | Has transitions      |
| PersonaCard                  | `transition-all`                            | Has transitions      |
| LorebookCard                 | `transition-all`                            | Has transitions      |
| Regex editor items           | `transition-colors`                         | Has color transition |
| Provider editor icon buttons | `transition-colors`                         | Has color transition |
| Home page character cards    | `transition-all` + `hover:-translate-y-0.5` | Good                 |
| Tabs (characters page)       | `transition-colors`                         | Has color transition |

### Missing Transitions (Need Addition)

| Target                | Current State                           | Needed                                                                  |
| --------------------- | --------------------------------------- | ----------------------------------------------------------------------- | ------- |
| Theme switch (global) | No global `transition-colors`           | Add to body/container — `transition-colors duration-150`                |
| Message appear        | No animation                            | Add `transition:fade` on MessageBubble mount (Svelte 5 `transition:fade | slide`) |
| Route cross-fade      | No animation                            | Add Svelte `transition:fade` on route outlet                            |
| Popover/dialog open   | bits-ui defaults                        | Verify consistent timing                                                |
| Focus rings           | Existing `:focus-visible` in base layer | Verify visibility in light theme                                        |

### Animation Philosophy (D-10)

- Subtle and fast: 150–200ms
- "Felt, not seen" — support the writing/prose experience
- Content-first — animations must not distract

### Recommended Approach

1. **Global theme transition:** Add `transition-colors duration-150` to the root container in `+layout.svelte` (the `surface-grain` div). This makes the entire app smoothly crossfade when theme changes. Every element using token colors transitions automatically.

2. **Message appear:** Use Svelte's `transition:fade={{ duration: 150 }}` on `MessageBubble` mount. Light enough to not distract.

3. **Route cross-fade:** Use Svelte's `transition:fade={{ duration: 150 }}` on the `<main>` content area. SvelteKit doesn't natively support route transitions, but wrapping the `{@render children?.()}` output with a keyed transition works.

4. **Sidebar:** Already has `transition-[width] duration-200` — verify this looks good in light mode.

---

## 10. Spacing & Consistency Audit

### Known Inconsistencies to Check

| Area                                    | Concern                                                                        |
| --------------------------------------- | ------------------------------------------------------------------------------ |
| Card padding                            | CharacterCard, PersonaCard, LorebookCard should have identical `p-4`           |
| Message input padding vs bubble padding | Verify alignment                                                               |
| Sidebar item padding                    | All items use `px-2 py-2` consistently                                         |
| Settings section header margin          | All use `mb-6` — verify                                                        |
| Dialog header/body/footer padding       | Verify alignment across all dialogs                                            |
| Icon sizes                              | Mix of `size-3`, `size-3.5`, `size-4`, `size-5`, `size-6` — verify intentional |
| Button sizes                            | Verify consistency across all button groups                                    |

### Audit Method

Visual inspection + grep for spacing classes. Not a code change task — primarily verification that existing components follow the 8-point scale.

---

## 11. Light Theme Specific Checks

### Elements That Need Light-Mode Verification

| Element                          | Concern                                                                      |
| -------------------------------- | ---------------------------------------------------------------------------- |
| `.surface-grain::before`         | Hardcoded oklch accent colors — too visible on light bg?                     |
| `::selection`                    | Uses `--color-accent-muted` bg — verify contrast on light                    |
| `::-webkit-scrollbar-thumb`      | Uses `--color-border` — verify visibility                                    |
| `mark` (FTS5 highlights)         | Uses `oklch(from var(--color-accent) l c h / 0.2)` — should work but verify  |
| Avatar overlay (`bg-black/40`)   | Black overlay on hover — works in both themes                                |
| Streaming caret (`color-accent`) | Should be fine — accent is defined for both themes                           |
| Code block inline `color-accent` | Verify accent readability on `--color-surface-elevated` light bg             |
| Toggle switch dot (`bg-white`)   | Hardcoded white — fine in both themes since toggle track changes             |
| Provider status indicators       | Green check, red X — verify `--color-success`/`--color-destructive` contrast |

### Focus Ring Verification

Current `:focus-visible` ring: `2px solid var(--color-accent)` with `outline-offset: 2px`.

- Dark theme: accent is `oklch(0.78 0.13 195)` (bright teal) — visible on dark bg
- Light theme: accent is `oklch(0.55 0.13 195)` (darker teal) — must verify visibility on `oklch(0.97)` backgrounds

Should be fine — the light accent has enough contrast against the near-white surface. But worth manual verification.

---

## 12. Files to Create

| File                                             | Purpose                                               |
| ------------------------------------------------ | ----------------------------------------------------- |
| `src-tauri/src/db/app_settings.rs`               | `load_app_setting` / `save_app_setting` SQL functions |
| `src-tauri/src/commands/app_settings.rs`         | `get_app_setting` / `set_app_setting` Tauri commands  |
| `src/lib/components/settings/ThemeToggle.svelte` | Three-way segmented button group                      |
| `src/lib/shiki.svelte.ts` (recommended)          | Shared Shiki highlighter singleton                    |

## 13. Files to Modify

| File                                              | Changes                                                                                          |
| ------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `src-tauri/src/db/mod.rs`                         | Add `pub mod app_settings;`, two `DbCommand` variants, two match arms                            |
| `src-tauri/src/commands/mod.rs`                   | Add `pub mod app_settings;`                                                                      |
| `src-tauri/src/lib.rs`                            | Register 2 new commands in `invoke_handler`                                                      |
| `src/lib/data/api.ts`                             | Add `getAppSetting()`, `setAppSetting()` wrappers                                                |
| `src/lib/stores/app.svelte.ts`                    | Add `theme` field, `resolved_theme` getter, `setTheme()` action, theme hydration                 |
| `src/app.css`                                     | Add `html.light { ... }` block with 17 token overrides, optional `.surface-grain` light override |
| `src/app.html`                                    | Remove `class="dark"`, add inline flash-prevention script                                        |
| `src/routes/+layout.svelte`                       | Load both Shiki themes, set up `matchMedia` listener, add global `transition-colors`             |
| `src/routes/settings/+page.svelte`                | Replace stub with ThemeToggle component                                                          |
| `src/lib/components/chat/MarkdownRenderer.svelte` | Read resolved theme, select `vitesse-dark` or `vitesse-light`                                    |
| Various components                                | Polish audit: add missing `transition-colors`, verify spacing                                    |

---

## 14. Risk Assessment

| Risk                                   | Likelihood                    | Impact                        | Mitigation                                                              |
| -------------------------------------- | ----------------------------- | ----------------------------- | ----------------------------------------------------------------------- |
| Flash of wrong theme on load           | Low (script-based prevention) | High (jarring UX)             | Inline script in `<head>` before CSS — must be first thing              |
| Shiki highlighter duplication          | High (already exists)         | Medium (perf)                 | Refactor to shared singleton                                            |
| Token contrast issues in light mode    | Medium                        | Medium (unreadable text)      | Manual verification of all 17 tokens against both backgrounds           |
| `matchMedia` listener leak             | Low                           | Low (memory)                  | Proper cleanup in `onMount` return                                      |
| Backend persist failure                | Low                           | Low (theme resets on restart) | localStorage as primary, backend as async backup — graceful degradation |
| Surface grain visibility in light mode | Medium                        | Low (cosmetic)                | Override with reduced opacity                                           |

---

## 15. Implementation Order Recommendation

**Wave 1: Backend + CSS Foundation (parallel)**

1. Add `app_settings.rs` DB functions + DbCommand variants + Tauri commands + registration
2. Add `html.light { ... }` token overrides in `app.css`
3. Add flash-prevention script in `app.html`

**Wave 2: Frontend Theme Infrastructure (depends on Wave 1)** 4. Add API wrappers in `api.ts` 5. Add theme state to AppState (`theme`, `resolved_theme`, `setTheme`, hydration) 6. Set up `matchMedia` listener in `+layout.svelte` 7. Refactor Shiki to shared singleton with dual themes 8. Update MarkdownRenderer for theme-aware rendering

**Wave 3: ThemeToggle UI + Polish (depends on Wave 2)** 9. Create `ThemeToggle.svelte` component 10. Wire into settings page appearance section 11. Add global `transition-colors` for smooth theme switching 12. Polish audit: message appear animation, route transitions, spacing verification

---

## 16. Questions for Planning

These are resolved by CONTEXT.md decisions but worth confirming:

1. **Dark `@theme {}` tokens remain the default?** Yes — D-13 says keep `@theme {}` for dark defaults, add `.light` overrides. The `@theme` block applies unconditionally; `html.light` overrides only when class is present. This means dark mode is the "zero class" state, which matches the flash-prevention fallback.

2. **Should the Shiki highlighter be a shared singleton or kept duplicated?** Shared singleton is better for perf and to avoid double WASM init. But the duplication is a pre-existing issue, not introduced by this phase. The phase can fix it as part of the Shiki dual-theme work.

3. **Route transitions via Svelte or CSS?** Svelte `transition:fade` is simpler and more reliable. CSS route transitions are fragile with SvelteKit's rendering model. Recommend Svelte transitions.

---

_Research complete. Ready for planning._
