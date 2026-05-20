# Phase 10: Theme & Polish - Context

**Gathered:** 2026-05-19
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can choose light or dark theme, and the app feels polished. This phase delivers:
- Light theme color tokens (OKLCH, cool-toned) paired with existing dark tokens
- Three-way theme toggle (System / Light / Dark) in Settings > Appearance
- System appearance detection with manual override
- Theme persistence via app_settings backend
- Flash-of-wrong-theme prevention on load
- Full visual polish audit: transitions, animations, spacing, hover states, focus rings

Requirements: SETT-01 (toggle between dark/light), SETT-02 (respects system preference with manual override)

NOT in scope: New features, new pages, backend logic beyond theme persistence, distribution/packaging.

</domain>

<decisions>
## Implementation Decisions

### Light Theme Aesthetic
- **D-01:** Light theme uses cool-toned palette — same 270 hue family as dark theme, not warm or paper-like
- **D-02:** Soft/off-white surface lightness (~oklch 0.94–0.97 range). Not stark white. Softer contrast, easier on eyes. Think Notion/Bear light mode, not Vercel/Linear bright white
- **D-03:** All tokens remain in OKLCH color space for consistency with existing dark tokens
- **D-04:** Teal accent (hue 195) carries over to light theme — same accent color adjusted for light backgrounds

### Theme Toggle UX
- **D-05:** Three-way toggle: System / Light / Dark. Default is System (follows OS preference)
- **D-06:** Toggle lives in Settings > Appearance section only (no header or sidebar quick-toggle)
- **D-07:** Visual form: button group (three adjacent buttons: System | Light | Dark). Active state highlighted
- **D-08:** Replaces the current stub text ("Theme controls coming soon. The current build is dark-only.")

### Polish Scope
- **D-09:** Full visual audit — not just theme toggle. Includes: transitions, animations, spacing consistency, hover states, focus rings across all components
- **D-10:** Animation style: subtle and fast (150–200ms). Felt, not seen. Content-first philosophy — animations support the prose/writing experience, not distract from it
- **D-11:** Specific transition targets: sidebar open/close slide, message appear animations, page route cross-fade
- **D-12:** Spacing/consistency audit: verify uniform padding, margin, radius usage across all components

### Theme Switching Mechanics
- **D-13:** CSS strategy: class-based token override. Keep `@theme {}` for dark defaults (current tokens). Add `.light` selector block that overrides every token with light values. Toggle swaps class on `<html>` element
- **D-14:** Flash prevention: inline `<script>` in `app.html` reads stored preference from localStorage before Svelte hydrates. Sets class immediately — no flash of wrong theme
- **D-15:** Shiki syntax highlighting: pair `vitesse-dark` (existing) with `vitesse-light` for light mode. MarkdownRenderer and layout preloader both need to load both themes
- **D-16:** Backend: add Tauri commands for `get_app_setting` / `set_app_setting` using the existing `app_settings` table (key/value). Theme persisted as `theme` key with values: `system`, `light`, `dark`
- **D-17:** AppState gains a `theme` field (`$state<'system' | 'light' | 'dark'>`) and derived getter for resolved theme (accounts for system preference when set to `system`)

### Agent's Discretion
- Exact OKLCH lightness/chroma values for each light token (within the soft/off-white, cool-toned, 270-hue parameters)
- Easing curves for transitions (ease-out, cubic-bezier, etc.) — keep them subtle
- Which specific components get message appear animations vs. which stay instant
- How the sidebar slide animation is implemented (CSS transition on width vs. transform)
- Whether page route transitions use Svelte `transition:` directives or CSS
- Order of polish audit (theme first, then animations, or interleaved)

</decisions>

<canonical_refs>
## Canonical References

### Prior Phase Decisions (carry-forward)
- `.planning/phases/07-rich-output-cards/07-CONTEXT.md` — Shiki vitesse-dark decision (D-04), MarkdownRenderer architecture
- `.planning/phases/01-native-desktop-shell/01-CONTEXT.md` — Initial dark theme, OKLCH token strategy
- `.planning/phases/04-provider-setup/04-CONTEXT.md` — Settings page structure, Tauri command patterns

### Active Backend (will be extended)
- `src-tauri/src/db/migrations.rs` — `app_settings` table schema (key TEXT PRIMARY KEY, value TEXT NOT NULL)
- `src-tauri/src/cmd/` — Existing Tauri command pattern for adding new IPC commands

### Active Frontend (will be modified)
- `src/app.css` — `@theme {}` block with dark OKLCH tokens (must add `.light` overrides)
- `src/app.html` — `<html class="dark">` hardcoded (must add inline flash-prevention script)
- `src/lib/stores/app.svelte.ts` — AppState singleton (must add theme field + persistence)
- `src/lib/components/chat/MarkdownRenderer.svelte` — Shiki hardcoded to `vitesse-dark`
- `src/routes/+layout.svelte` — Shiki preloader hardcoded to `vitesse-dark`
- `src/routes/settings/+page.svelte` — Appearance section stub to replace with actual toggle

### Reference Implementation
- `gtk_ref/src/db/providers.rs` — `save_app_setting` / `load_app_setting` pattern (lines 195–213, test coverage)

### Project Planning
- `.planning/PROJECT.md` — Key Decisions table, dark-by-default decision, OKLCH convention
- `.planning/REQUIREMENTS.md` — SETT-01, SETT-02 requirements

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `app.css @theme {}` block: All dark tokens already defined as CSS custom properties. Light tokens just need matching overrides in a `.light` block
- `app_settings` table: Backend key/value store already exists in schema — just needs CRUD Tauri commands
- Settings page navigation: Already has Appearance section with Palette icon, stubbed content — ready to fill in
- bits-ui components: Dialog, Select primitives available for building the toggle UI if needed

### Established Patterns
- Tauri command pattern: `#[tauri::command]` functions in `src-tauri/src/cmd/`, registered in `main.rs`
- AppState rune pattern: `$state` fields with `hydrate()` loading and persistence methods
- `cn()` utility: Used everywhere for class merging — theme-dependent classes can flow through this
- CSS custom property pattern: All components use `var(--color-*)` / `bg-(--color-*)` — theme switching propagates automatically once tokens are overridden

### Integration Points
- `app.html <html>` element: Where theme class lives (currently `class="dark"`)
- `app.css @theme {}`: Where token overrides go (add `.light` block after `@theme`)
- `AppState.hydrate()`: Where theme preference loads from backend on startup
- `+layout.svelte`: Where Shiki preloader needs to load both themes
- `MarkdownRenderer.svelte`: Where active Shiki theme is selected based on resolved theme

</code_context>

<specifics>
## Specific Ideas

- Light mode should feel like "the same app in a different light" — not a completely different aesthetic
- Soft/off-white surfaces (~Notion/Bear) rather than stark white (~Linear/Vercel)
- Animations should be "felt not seen" — support the writing/prose experience, not distract from it
- The existing `transition-colors` on hover states is a good baseline — build on that pattern

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 10-theme-polish*
*Context gathered: 2026-05-19*
