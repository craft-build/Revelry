---
wave: 1
depends_on: []
files_modified:
  - src-tauri/src/db/app_settings.rs
  - src-tauri/src/db/mod.rs
  - src-tauri/src/commands/app_settings.rs
  - src-tauri/src/commands/mod.rs
  - src-tauri/src/lib.rs
  - src/lib/data/api.ts
  - src/app.css
  - src/app.html
  - src/lib/stores/app.svelte.ts
  - src/lib/shiki.svelte.ts
  - src/routes/+layout.svelte
  - src/lib/components/settings/ThemeToggle.svelte
  - src/routes/settings/+page.svelte
  - src/lib/components/chat/MarkdownRenderer.svelte
autonomous: true
requirements:
  - SETT-01
  - SETT-02
---

# Phase 10: Theme & Polish

## Goal

Users can toggle between light and dark themes (SETT-01) and the app respects system appearance preference with manual override (SETT-02).

## must_haves

1. `app.css` contains an `html.light { ... }` block that overrides all 17 color tokens
2. `app.html` has an inline flash-prevention script in `<head>` that reads localStorage and sets class on `<html>` before paint
3. `AppState` has a `theme` field, `resolved_theme` getter, and `setTheme()` action that persists to both localStorage and backend
4. Settings > Appearance section contains a working three-way ThemeToggle (System/Light/Dark) that instantly changes the app theme
5. Shiki loads both `vitesse-dark` and `vitesse-light` themes and MarkdownRenderer selects the correct one based on resolved theme
6. `matchMedia` listener detects OS preference changes and auto-updates when theme is "system"
7. No flash of wrong theme on app load

---

## Task 1: Backend — app_settings DB functions + DbCommand variants + Tauri commands

<read_first>

- src-tauri/src/db/mod.rs (current DbCommand enum and handle_command match arms)
- src-tauri/src/db/migrations.rs (app_settings table schema at line 25)
- src-tauri/src/commands/mod.rs (current module list)
- src-tauri/src/commands/providers.rs (reference pattern for Tauri commands using DbWorker)
- src-tauri/src/lib.rs (current invoke_handler registration)
- gtk_ref/src/db/providers.rs (reference: save_app_setting at line 80, load_app_setting at line 90)
  </read_first>

<action>
### 1. Create `src-tauri/src/db/app_settings.rs`

Two functions, ported verbatim from `gtk_ref/src/db/providers.rs:80-97`:

```rust
use anyhow::Result;
use rusqlite::Connection;

pub fn load_app_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare_cached("SELECT value FROM app_settings WHERE key = ?1")?;
    let mut rows = stmt.query(rusqlite::params![key])?;
    match rows.next()? {
        Some(row) => Ok(Some(row.get(0)?)),
        None => Ok(None),
    }
}

pub fn save_app_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![key, value],
    )?;
    Ok(())
}
```

### 2. Add to `src-tauri/src/db/mod.rs`

Add at top of file with other `pub mod` declarations:

```rust
pub mod app_settings;
```

Add two new `DbCommand` variants in the enum (after the Provider operations block):

```rust
// App settings operations
LoadAppSetting {
    key: String,
    reply: oneshot::Sender<anyhow::Result<Option<String>>>,
},
SaveAppSetting {
    key: String,
    value: String,
    reply: oneshot::Sender<anyhow::Result<()>>,
},
```

Add two new match arms in `handle_command()` (after the `SearchMessages` arm, before the closing `}`):

```rust
DbCommand::LoadAppSetting { key, reply } => {
    let result = app_settings::load_app_setting(conn, &key);
    let _ = reply.send(result);
}
DbCommand::SaveAppSetting { key, value, reply } => {
    let result = app_settings::save_app_setting(conn, &key, &value);
    let _ = reply.send(result);
}
```

### 3. Create `src-tauri/src/commands/app_settings.rs`

Two Tauri command functions following the exact pattern from `commands/providers.rs`:

```rust
use crate::db::{DbCommand, DbWorker};
use tauri::State;
use tokio::sync::oneshot;

#[tauri::command]
pub async fn get_app_setting(
    key: String,
    db: State<'_, DbWorker>,
) -> Result<Option<String>, String> {
    let (reply_tx, reply_rx) = oneshot::channel();
    db.send(DbCommand::LoadAppSetting { key, reply: reply_tx })
        .map_err(|e| e.to_string())?;
    reply_rx.await.map_err(|e| e.to_string())?.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_app_setting(
    key: String,
    value: String,
    db: State<'_, DbWorker>,
) -> Result<(), String> {
    let (reply_tx, reply_rx) = oneshot::channel();
    db.send(DbCommand::SaveAppSetting { key, value, reply: reply_tx })
        .map_err(|e| e.to_string())?;
    reply_rx.await.map_err(|e| e.to_string())?.map_err(|e| e.to_string())
}
```

### 4. Add to `src-tauri/src/commands/mod.rs`

```rust
pub mod app_settings;
```

### 5. Register in `src-tauri/src/lib.rs`

Add to `invoke_handler` array:

```rust
commands::app_settings::get_app_setting,
commands::app_settings::set_app_setting,
```

### 6. Add to `src/lib/data/api.ts`

Two wrappers at the end of the file (before the last section comment or at bottom):

```typescript
// ── App settings ──────────────────────────────────────────────────────────────

export async function getAppSetting(key: string): Promise<string | null> {
  return invoke<string | null>("get_app_setting", { key });
}

export async function setAppSetting(key: string, value: string): Promise<void> {
  return invoke<void>("set_app_setting", { key, value });
}
```

Also add these to the import list at the top of `src/lib/stores/app.svelte.ts` — add `getAppSetting` and `setAppSetting` to the imports from `'$lib/data/api'`.
</action>

<acceptance_criteria>

- `src-tauri/src/db/app_settings.rs` exists and contains `fn load_app_setting` and `fn save_app_setting`
- `src-tauri/src/db/mod.rs` contains `pub mod app_settings;`
- `src-tauri/src/db/mod.rs` contains `LoadAppSetting { key: String, reply: ... }` and `SaveAppSetting { key: String, value: String, reply: ... }` variants
- `src-tauri/src/db/mod.rs` has match arms for `DbCommand::LoadAppSetting` and `DbCommand::SaveAppSetting` calling `app_settings::load_app_setting` and `app_settings::save_app_setting`
- `src-tauri/src/commands/app_settings.rs` exists and contains `pub async fn get_app_setting` and `pub async fn set_app_setting`
- `src-tauri/src/commands/mod.rs` contains `pub mod app_settings;`
- `src-tauri/src/lib.rs` invoke_handler contains `commands::app_settings::get_app_setting` and `commands::app_settings::set_app_setting`
- `src/lib/data/api.ts` contains `export async function getAppSetting` and `export async function setAppSetting`
- `cargo build` succeeds from `src-tauri/` directory
  </acceptance_criteria>

---

## Task 2: CSS — Light theme token overrides + surface grain light adjustment

<read_first>

- src/app.css (current @theme block with all 17 dark color tokens, surface-grain utility, base layer, components layer)
  </read_first>

<action>
### 1. Add `html.light` override block in `src/app.css`

Insert immediately after the closing `}` of the `@theme { }` block (before the `/* Base styles */` comment):

```css
/*
 * Light theme overrides
 * Activated by setting class="light" on <html>.
 * Uses same OKLCH color space, cool-toned 270 hue family.
 * Soft/off-white surfaces (Notion/Bear aesthetic).
 */
html.light {
  /* Surfaces */
  --color-background: oklch(0.97 0.003 270);
  --color-surface: oklch(0.96 0.004 270);
  --color-surface-elevated: oklch(0.94 0.005 270);
  --color-surface-hover: oklch(0.92 0.006 270);
  --color-surface-input: oklch(0.95 0.004 270);

  /* Foreground */
  --color-foreground: oklch(0.18 0.01 270);
  --color-muted: oklch(0.5 0.01 270);
  --color-subtle: oklch(0.38 0.008 270);

  /* Borders */
  --color-border: oklch(0.88 0.005 270);
  --color-border-strong: oklch(0.78 0.008 270);

  /* Accent — teal, darker for light backgrounds */
  --color-accent: oklch(0.55 0.13 195);
  --color-accent-fg: oklch(0.97 0.01 195);
  --color-accent-muted: oklch(0.88 0.05 195);

  /* Semantic */
  --color-destructive: oklch(0.55 0.2 25);
  --color-warning: oklch(0.68 0.16 80);
  --color-success: oklch(0.55 0.15 155);

  /* Chat bubble specific */
  --color-bubble-user: oklch(0.93 0.008 220);
  --color-bubble-character: oklch(0.96 0.004 270);
}
```

### 2. Add light-mode surface grain override

After the existing `.surface-grain::before { ... }` block inside `@layer utilities`, add:

```css
html.light .surface-grain::before {
  opacity: 0.3;
  background-image:
    radial-gradient(
      circle at 20% 30%,
      oklch(0.55 0.13 195 / 0.02),
      transparent 50%
    ),
    radial-gradient(
      circle at 80% 70%,
      oklch(0.55 0.13 280 / 0.015),
      transparent 50%
    );
}
```

### 3. Add global transition-colors for smooth theme switching

In the `@layer base` block, add to the `body` rule:

```css
transition:
  background-color 150ms ease-out,
  color 150ms ease-out;
```

The full `body` rule becomes:

```css
body {
  background: var(--color-background);
  color: var(--color-foreground);
  min-height: 100dvh;
  overflow: hidden;
  transition:
    background-color 150ms ease-out,
    color 150ms ease-out;
}
```

### 4. Add transition-colors to the root container in `+layout.svelte`

On the outermost `<div class="surface-grain ...">`, add `transition-colors duration-150` to the class list. This makes the surface grain and background transition smoothly on theme change.
</action>

<acceptance_criteria>

- `src/app.css` contains `html.light {` block with all 17 color token overrides (--color-background through --color-bubble-character)
- `src/app.css` contains `html.light .surface-grain::before` with reduced opacity
- `src/app.css` `body` rule contains `transition: background-color 150ms ease-out, color 150ms ease-out`
- `src/app.css` has exactly one `html.light` block (no duplicates)
- All 17 dark tokens in `@theme` have matching light tokens in `html.light`
- No changes to `@theme` block values
- No changes to `--font-*` or `--radius-*` tokens in the light block
  </acceptance_criteria>

---

## Task 3: Flash prevention + AppState theme infrastructure + Shiki dual theme

<read_first>

- src/app.html (current hardcoded class="dark" on html element)
- src/lib/stores/app.svelte.ts (AppState class, hydrate method, all imports)
- src/routes/+layout.svelte (Shiki preload, onMount, structure)
- src/lib/components/chat/MarkdownRenderer.svelte (Shiki highlighter usage, vitesse-dark hardcoded)
- src/lib/data/api.ts (to verify getAppSetting/setAppSetting added by Task 1)
  </read_first>

<action>
### 1. Modify `src/app.html`

Replace `<html lang="en" class="dark">` with `<html lang="en">` (remove class="dark").

Add this inline `<script>` block in `<head>` as the FIRST child (before the Google Fonts `<link rel="preconnect">`):

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

### 2. Create `src/lib/shiki.svelte.ts`

New file. Shared Shiki highlighter singleton that both layout and MarkdownRenderer use:

```typescript
import { createHighlighter, type Highlighter } from "shiki";

let highlighterInstance: Highlighter | null = null;
let highlighterPromise: Promise<Highlighter> | null = null;

const LANGS = [
  "javascript",
  "typescript",
  "python",
  "rust",
  "json",
  "html",
  "css",
  "bash",
  "markdown",
] as const;

const THEMES = ["vitesse-dark", "vitesse-light"] as const;

export async function getShikiHighlighter(): Promise<Highlighter> {
  if (highlighterInstance) return highlighterInstance;
  if (highlighterPromise) return highlighterPromise;

  highlighterPromise = createHighlighter({
    themes: [...THEMES],
    langs: [...LANGS],
  }).then((hl) => {
    highlighterInstance = hl;
    return hl;
  });

  return highlighterPromise;
}
```

### 3. Add theme state to `AppState` in `src/lib/stores/app.svelte.ts`

Add to the import at top of file:

```typescript
import { getAppSetting, setAppSetting } from "$lib/data/api";
```

(Note: `getAppSetting` and `setAppSetting` were added by Task 1.)

Add new state field in AppState class (after `focusProviderSelector`):

```typescript
// ── Theme state ──────────────────────────────────────────────────────────
theme = $state<"system" | "light" | "dark">("system");
```

Add new getter (after existing getters):

```typescript
get resolved_theme(): 'light' | 'dark' {
	if (this.theme === 'system') {
		if (typeof window === 'undefined') return 'dark';
		return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
	}
	return this.theme;
}
```

Add new action (after `toggleSidebar`):

```typescript
async setTheme(theme: 'system' | 'light' | 'dark') {
	this.theme = theme;
	localStorage.setItem('revelry-theme', theme);
	document.documentElement.className = this.resolved_theme;
	// Async backend persist — non-blocking, localStorage is the primary store
	try {
		await setAppSetting('theme', theme);
	} catch (err) {
		console.error('Theme persist failed:', err);
	}
}
```

In `hydrate()`, add theme loading. After the existing `Promise.allSettled` block (after `providers` is handled), add:

```typescript
// Load theme preference from backend
try {
  const savedTheme = await getAppSetting("theme");
  if (
    savedTheme === "system" ||
    savedTheme === "light" ||
    savedTheme === "dark"
  ) {
    this.theme = savedTheme;
  }
} catch (err) {
  // Theme loading is non-critical, localStorage already handled by flash-prevention script
  console.warn("Failed to load theme from backend:", err);
}
// Sync <html> class with resolved theme
document.documentElement.className = this.resolved_theme;
```

### 4. Modify `src/routes/+layout.svelte`

Replace the existing Shiki preload block in `onMount`:

Old code to replace:

```typescript
createHighlighter({
  themes: ["vitesse-dark"],
  langs: [
    "javascript",
    "typescript",
    "python",
    "rust",
    "json",
    "html",
    "css",
    "bash",
    "markdown",
  ],
})
  .then(() => {
    // Shiki engine cached internally — MarkdownRenderer will reuse it
  })
  .catch((err: unknown) => {
    console.warn("Shiki preload failed (non-critical):", err);
  });
```

Replace with:

```typescript
import { getShikiHighlighter } from "$lib/shiki.svelte";
```

And in `onMount`:

```typescript
// Preload Shiki syntax highlighting with both themes
getShikiHighlighter().catch((err: unknown) => {
  console.warn("Shiki preload failed (non-critical):", err);
});

// Listen for OS theme preference changes
const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
const onMediaChange = () => {
  if (app.theme === "system") {
    document.documentElement.className = app.resolved_theme;
  }
};
mediaQuery.addEventListener("change", onMediaChange);
```

Add the import for `getShikiHighlighter` at the top of the script. Also update the cleanup in the `onMount` return to include:

```typescript
return () => {
  window.removeEventListener("keydown", onkey);
  mediaQuery.removeEventListener("change", onMediaChange);
};
```

Also add `transition-colors duration-150` to the outermost `<div>` class:
Change:

```svelte
<div class="surface-grain flex h-dvh flex-col bg-(--color-background) text-(--color-foreground)">
```

To:

```svelte
<div class="surface-grain flex h-dvh flex-col bg-(--color-background) text-(--color-foreground) transition-colors duration-150">
```

### 5. Modify `src/lib/components/chat/MarkdownRenderer.svelte`

Replace the entire Shiki highlighter initialization. Remove the `import { createHighlighter, type Highlighter } from 'shiki';` import and the `onMount` block that creates a highlighter.

Replace with:

```typescript
import { getShikiHighlighter } from "$lib/shiki.svelte";
import { app } from "$lib/stores/app.svelte";
```

Change the `let highlighter = $state<Highlighter | null>(null);` to keep it, but replace `onMount` with:

```typescript
onMount(async () => {
  try {
    highlighter = await getShikiHighlighter();
  } catch (err) {
    console.warn("Failed to load Shiki highlighter:", err);
  }
});
```

In the `$effect` that renders markdown, change the Shiki theme selection from hardcoded `'vitesse-dark'` to dynamic:

Replace:

```typescript
hl.getLoadedThemes().includes("vitesse-dark");
```

With:

```typescript
const shikiTheme =
  app.resolved_theme === "light" ? "vitesse-light" : "vitesse-dark";
```

And use `shikiTheme` in the condition and in the `hl.codeToHtml` call:

```typescript
renderer.code = function ({ text, lang }: { text: string; lang?: string }) {
  if (lang && hl) {
    try {
      const shikiTheme =
        app.resolved_theme === "light" ? "vitesse-light" : "vitesse-dark";
      if (
        hl.getLoadedLanguages().includes(lang) &&
        hl.getLoadedThemes().includes(shikiTheme)
      ) {
        const highlighted = hl.codeToHtml(text, { lang, theme: shikiTheme });
        return `<div class="relative"><span class="lang-label">${lang}</span>${highlighted}</div>`;
      }
    } catch {
      // Fall through to plain code block
    }
  }
  const langAttr = lang ? ` class="language-${lang}"` : "";
  const langLabel = lang ? `<span class="lang-label">${lang}</span>` : "";
  return `<div class="relative">${langLabel}<pre><code${langAttr}>${escapeHtml(text)}</code></pre></div>`;
};
```

The `content` reference in the `$effect` already makes it reactive. Reading `app.resolved_theme` inside the effect also makes it reactive to theme changes, so markdown re-renders automatically when theme switches.
</action>

<acceptance_criteria>

- `src/app.html` does NOT contain `class="dark"` on the `<html>` element
- `src/app.html` contains an inline `<script>` block in `<head>` that calls `localStorage.getItem("revelry-theme")` and `document.documentElement.classList.add(resolved)`
- `src/app.html` inline script is BEFORE the Google Fonts links
- `src/lib/shiki.svelte.ts` exists and exports `getShikiHighlighter()` which loads both `vitesse-dark` and `vitesse-light` themes
- `src/lib/stores/app.svelte.ts` contains `theme = $state<'system' | 'light' | 'dark'>('system')`
- `src/lib/stores/app.svelte.ts` contains getter `get resolved_theme(): 'light' | 'dark'`
- `src/lib/stores/app.svelte.ts` contains method `async setTheme(theme: 'system' | 'light' | 'dark')`
- `src/lib/stores/app.svelte.ts` `hydrate()` calls `getAppSetting('theme')`
- `src/lib/stores/app.svelte.ts` `hydrate()` sets `document.documentElement.className = this.resolved_theme`
- `src/routes/+layout.svelte` imports `getShikiHighlighter` from `'$lib/shiki.svelte'`
- `src/routes/+layout.svelte` does NOT call `createHighlighter` directly
- `src/routes/+layout.svelte` sets up `matchMedia('(prefers-color-scheme: dark)')` change listener
- `src/routes/+layout.svelte` outermost div has `transition-colors duration-150` class
- `src/lib/components/chat/MarkdownRenderer.svelte` imports `getShikiHighlighter` from `'$lib/shiki.svelte'`
- `src/lib/components/chat/MarkdownRenderer.svelte` does NOT call `createHighlighter` directly
- `src/lib/components/chat/MarkdownRenderer.svelte` reads `app.resolved_theme` to select between `vitesse-light` and `vitesse-dark`
- `bun run check` passes (no type errors)
  </acceptance_criteria>

---

## Task 4: ThemeToggle component + Settings page integration + Polish audit

<read_first>

- src/routes/settings/+page.svelte (current appearance section stub at ~line 160)
- src/lib/stores/app.svelte.ts (verify theme, resolved_theme, setTheme exist from Task 3)
- src/lib/components/chat/MessageBubble.svelte (for transition audit)
- src/lib/components/layout/ConversationSidebar.svelte (for transition audit)
  </read_first>

<action>
### 1. Create `src/lib/components/settings/ThemeToggle.svelte`

New file with a three-way segmented button group:

```svelte
<script lang="ts">
	import { app } from '$lib/stores/app.svelte';
	import { cn } from '$lib/utils';
	import { Monitor, Sun, Moon } from '@lucide/svelte';

	type ThemeValue = 'system' | 'light' | 'dark';

	const options: { value: ThemeValue; label: string; icon: typeof Monitor; ariaLabel: string }[] = [
		{ value: 'system', label: 'System', icon: Monitor, ariaLabel: 'Follow system preference' },
		{ value: 'light', label: 'Light', icon: Sun, ariaLabel: 'Light theme' },
		{ value: 'dark', label: 'Dark', icon: Moon, ariaLabel: 'Dark theme' },
	];

	function getAnnotation(theme: ThemeValue): string {
		switch (theme) {
			case 'system':
				return `Following your system preference (${app.resolved_theme})`;
			case 'light':
				return 'Light theme active';
			case 'dark':
				return 'Dark theme active';
		}
	}
</script>

<div class="inline-flex rounded-(--radius-lg) border border-(--color-border) bg-(--color-background) p-1 gap-1"
	role="radiogroup"
	aria-label="Theme preference"
>
	{#each options as option (option.value)}
		{@const isActive = app.theme === option.value}
		<button
			type="button"
			role="radio"
			aria-checked={isActive}
			aria-label={option.ariaLabel}
			onclick={() => app.setTheme(option.value)}
			class={cn(
				'inline-flex items-center gap-1.5 rounded-(--radius-md) px-4 py-2 text-sm font-medium transition-all duration-150',
				isActive
					? 'bg-(--color-surface-elevated) text-(--color-foreground) shadow-sm'
					: 'text-(--color-muted) hover:text-(--color-foreground)'
			)}
		>
			<option.icon class="size-4" />
			{option.label}
		</button>
	{/each}
</div>
<p class="text-xs text-(--color-subtle) mt-2">
	{getAnnotation(app.theme)}
</p>
```

### 2. Modify `src/routes/settings/+page.svelte`

Replace the appearance section stub. Find this block:

```svelte
{:else if active === 'appearance'}
	<header class="mb-6">
		<h1 class="font-display text-2xl font-semibold tracking-tight">Appearance</h1>
		<p class="mt-1 text-sm text-(--color-muted)">
			Theme, density, and reading comfort.
		</p>
	</header>
	<div class="rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface) p-4 text-sm text-(--color-muted)">
		Theme controls coming soon. The current build is dark-only.
	</div>
```

Replace with:

```svelte
{:else if active === 'appearance'}
	<header class="mb-6">
		<h1 class="font-display text-2xl font-semibold tracking-tight">Appearance</h1>
		<p class="mt-1 text-sm text-(--color-muted)">
			Theme, density, and reading comfort.
		</p>
	</header>
	<div class="rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface) p-4">
		<h2 class="font-display text-base font-semibold tracking-tight text-(--color-foreground)">Theme</h2>
		<p class="text-sm text-(--color-muted) mt-1">Choose how Revelry looks. System follows your OS preference.</p>
		<div class="mt-4">
			<ThemeToggle />
		</div>
	</div>
```

Add the import at top of script:

```typescript
import ThemeToggle from "$lib/components/settings/ThemeToggle.svelte";
```

### 3. Add `transition:fade` to MessageBubble mount

Read `src/lib/components/chat/MessageBubble.svelte`. If it doesn't already have a Svelte transition on mount, add:

```typescript
import { fade } from "svelte/transition";
```

And on the outermost element of the component, add:

```svelte
transition:fade={{ duration: 150 }}
```

### 4. Polish audit — verify consistent `transition-colors` across interactive elements

This is a verification pass, not bulk changes. Check these files for interactive elements missing `transition-colors`:

- `src/lib/components/layout/ConversationSidebar.svelte` — verify all clickable items have `transition-colors`
- `src/lib/components/layout/AppHeader.svelte` — verify nav buttons have `transition-colors`
- `src/routes/+page.svelte` — verify character cards have transition
- `src/routes/characters/+page.svelte` — verify tab buttons have `transition-colors`

If any interactive element lacks `transition-colors` and it's not already using `transition-all`, add `transition-colors duration-150` to its class. Only add where it's genuinely missing — do not add duplicates or override existing transition utilities.
</action>

<acceptance_criteria>

- `src/lib/components/settings/ThemeToggle.svelte` exists
- `src/lib/components/settings/ThemeToggle.svelte` contains `role="radiogroup"` and `aria-label="Theme preference"`
- `src/lib/components/settings/ThemeToggle.svelte` contains three buttons with `role="radio"` and `aria-checked`
- `src/lib/components/settings/ThemeToggle.svelte` imports Monitor, Sun, Moon from `@lucide/svelte`
- `src/lib/components/settings/ThemeToggle.svelte` calls `app.setTheme()` on button click
- `src/routes/settings/+page.svelte` imports `ThemeToggle`
- `src/routes/settings/+page.svelte` does NOT contain the text "Theme controls coming soon"
- `src/routes/settings/+page.svelte` appearance section contains `<ThemeToggle />`
- `src/routes/settings/+page.svelte` appearance section contains heading "Theme" and description "Choose how Revelry looks. System follows your OS preference."
- `src/lib/components/chat/MessageBubble.svelte` contains `transition:fade` or `import { fade }` from svelte/transition
- `bun run check` passes
- `bun run build` succeeds
  </acceptance_criteria>

---

## Verification

After all tasks are complete, verify:

1. **Backend builds:** `cd src-tauri && cargo build` succeeds
2. **Frontend builds:** `bun run build` succeeds
3. **Type check passes:** `bun run check` exits 0
4. **No flash of wrong theme:** The inline script in `app.html` is the first element in `<head>` and synchronously sets class before paint
5. **Theme toggle works:** Settings > Appearance shows three-way toggle that changes app theme instantly
6. **System preference respected:** When set to "System", app follows OS dark/light preference
7. **Shiki dual theme:** Code blocks in markdown use `vitesse-dark` in dark mode and `vitesse-light` in light mode
8. **Persistence:** Theme preference survives app restart (stored in localStorage immediately, backed up to SQLite async)

## Cross-cutting Constraints

- **localStorage is primary, backend is backup.** The flash-prevention script reads localStorage (synchronous, available before Svelte hydrates). The backend `app_settings` table is written asynchronously as a backup. If backend write fails, theme still works on next load via localStorage.
- **No new migration.** The `app_settings` table already exists in migration #2. This phase only adds CRUD functions and IPC commands.
- **Shiki singleton.** Both layout preload and MarkdownRenderer must use `getShikiHighlighter()` from `src/lib/shiki.svelte.ts`. No component should call `createHighlighter()` directly.
- **OKLCH color space.** All light theme tokens use OKLCH, same as dark tokens. No hex or HSL values.
- **Class-based switching.** Theme is controlled entirely by the class on `<html>` (`"light"` or `"dark"`). No `@media (prefers-color-scheme)` CSS rules — the media query is only used in JS to resolve "system" to "light" or "dark".
