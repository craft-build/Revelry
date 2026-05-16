---
phase: 7
slug: rich-output-cards
status: draft
shadcn_initialized: false
preset: none
created: 2026-05-16
---

# Phase 7 — UI Design Contract

> Visual and interaction contract for the Rich Output & Cards phase. Replaces the naive `renderInlineMarkdown()` in MessageBubble with a real GFM markdown renderer. Adds TavernAI V2 PNG card import/export to the character library. Markdown renders on completion only — raw text during streaming. Card import opens a pre-filled CharacterEditorDialog for review before saving. All design tokens, typography, color, spacing, and radii are inherited from Phase 1's locked design system.

---

## Design System

**Inherited from Phase 1.** No changes to tooling, tokens, or component library.

| Property          | Value                                                           |
| ----------------- | --------------------------------------------------------------- |
| Tool              | none (no shadcn — project uses Bits.ui + Tailwind v4 directly)  |
| Component library | Bits.ui ^2.18 (headless Svelte primitives)                      |
| Icon library      | @lucide/svelte ^1.14                                            |
| Font              | Geist (sans), Fraunces (serif/display), Geist Mono (mono)       |
| Styling           | Tailwind CSS v4 with custom OKLCH theme tokens in `src/app.css` |
| Variant system    | tailwind-variants ^3.2 (`tv()`)                                 |

**Reference:** `.planning/phases/01-native-desktop-shell/01-UI-SPEC.md` § Design System — all unchanged.

---

## Spacing Scale

**Inherited from Phase 1.** No changes. 4px-multiple scale (4, 8, 12, 16, 20, 24, 32, 40, 48, 64).

**Phase 7 additions:**

| Token                  | Value             | Usage                                                           |
| ---------------------- | ----------------- | --------------------------------------------------------------- |
| `prose-body-gap`       | 16px (space-y-4)  | Vertical gap between block-level markdown elements (paragraphs, lists, etc.) |
| `prose-heading-gap`    | 8px (mt-2)        | Gap between heading and following paragraph                     |
| `code-block-pad`       | 16px (p-4)        | Padding inside fenced code blocks                               |
| `code-block-radius`    | 0.75rem           | Border radius for code block containers (matches `--radius-lg`) |
| `inline-code-pad`      | 2px 6px (px-1.5 py-0.5) | Padding for inline `code` elements                        |
| `table-cell-pad`       | 8px 12px (px-3 py-2) | Padding for GFM table cells                               |
| `blockquote-border`    | 3px (border-l-3)  | Left border width for blockquotes                               |
| `blockquote-pad`       | 8px 16px (pl-4 py-2) | Left padding + vertical padding for blockquotes             |

All new spacing uses existing Tailwind token values. No custom spacing values introduced.

Exceptions: none

---

## Typography

**Inherited from Phase 1.** Consolidated to 4 sizes and 2 weights per design system rules.

**Type scale (4 sizes, 2 weights):**

| Role                                                       | Size        | Weight         | Line Height     | Font Family                             |
| ---------------------------------------------------------- | ----------- | -------------- | --------------- | --------------------------------------- |
| Body (chat bubbles, input, error messages)                 | 15px        | 400 (regular)  | 1.625 (relaxed) | Geist (sans)                            |
| Heading (speaker names, card titles, dialog title)         | 14px–18px\* | 600 (semibold) | 1.2             | Fraunces (serif/display) / Geist (sans) |
| Label / small (sidebar items, nav, metadata, field labels) | 12px (xs)   | 400 (regular)  | 1.25            | Geist (sans)                            |
| Textarea body / edit textarea                              | 14px (sm)   | 400 (regular)  | 1.625           | Geist (sans)                            |

\* Dialog title: 18px Geist (sans), weight 600. Card titles / speaker names: 14px Fraunces (serif/display), weight 600.

**Phase 7 role breakdown — markdown prose typography:**

| Element                         | Size    | Weight | Line Height | Font Family  | Notes                                  |
| ------------------------------- | ------- | ------ | ----------- | ------------ | -------------------------------------- |
| Markdown body text (paragraphs) | 15px    | 400    | 1.75        | Geist sans   | Slightly taller line-height than plain bubble for prose readability |
| Markdown h1                     | 24px    | 600    | 1.2         | Geist sans   | Top margin: 24px; first-child: no margin |
| Markdown h2                     | 20px    | 600    | 1.25        | Geist sans   | Top margin: 20px; first-child: no margin |
| Markdown h3                     | 16px    | 600    | 1.3         | Geist sans   | Top margin: 16px; first-child: no margin |
| Markdown h4–h6                  | Not styled separately — h3 is the deepest heading rendered. h4+ render as bold paragraph text. |
| Markdown strong                 | 15px    | 600    | 1.75        | Geist sans   | Inherits parent size/line-height        |
| Markdown em                     | 15px    | 400    | 1.75        | Geist sans   | `text-(--color-muted)` per D-05 narration styling |
| Markdown inline code            | 13px    | 400    | 1.75        | Geist Mono   | Slightly smaller than body; monospace   |
| Markdown fenced code block      | 13px    | 400    | 1.5         | Geist Mono   | Shiki-highlighted                       |
| Markdown blockquote             | 15px    | 400    | 1.75        | Geist sans   | Muted foreground, left accent border    |
| Markdown list items (ul/ol)     | 15px    | 400    | 1.75        | Geist sans   | Left padding: 24px for nesting          |
| Markdown GFM table              | 14px    | 400    | 1.5         | Geist sans   | Slightly compact for tabular data       |
| Markdown GFM task list checkbox | 14px    | —      | —           | —            | Native checkbox, accent-colored when checked |
| Markdown link                   | 15px    | 400    | 1.75        | Geist sans   | `text-(--color-accent) hover:underline` |
| Markdown horizontal rule        | —       | —      | —           | —            | `border-(--color-border)` 1px line      |
| Code block language label       | 12px    | 400    | 1.0         | Geist sans   | Top-right of code block, `text-(--color-subtle)` uppercase |

**Phase 7 role breakdown — card import/export typography:**

| Element                          | Size    | Weight | Font Family | Notes                                   |
| -------------------------------- | ------- | ------ | ----------- | --------------------------------------- |
| "Import card" button label       | 12px    | 500    | Geist sans  | Inherited from Button secondary variant  |
| "Export" kebab menu item label   | 14px    | 400    | Geist sans  | Inherited from KebabMenuItem pattern     |
| Import error dialog heading      | 18px    | 600    | Geist sans  | Inherited from Dialog title pattern      |
| Import error dialog body         | 14px    | 400    | Geist sans  | Standard dialog body size                |
| Import error detail text         | 12px    | 400    | Geist Mono  | Monospace for technical error details    |

---

## Color

All colors defined as OKLCH values in `src/app.css` @theme block. Dark theme only — light theme deferred to Phase 10.

**Inherited from Phase 5/6.** The full color table is documented in `05-UI-SPEC.md` § Color and `01-UI-SPEC.md` § Color. Not repeated here — no new base tokens.

**Phase 7 markdown prose colors:**

| Element                    | Color Token                    | Usage                                        |
| -------------------------- | ------------------------------ | -------------------------------------------- |
| Markdown body text         | `--color-foreground`           | Default prose text                            |
| Markdown em / italic       | `--color-muted`                | Muted text for narration (D-05)              |
| Markdown strong            | `--color-foreground`           | Bold text uses normal foreground, weight 600 |
| Markdown link              | `--color-accent`               | Links are accent-colored                     |
| Markdown inline code bg    | `--color-surface-elevated`     | Background for `code` elements               |
| Markdown inline code text  | `--color-accent`               | Monospace inline code is accent-tinted       |
| Markdown fenced code bg    | `--color-surface`              | Dark background for code blocks              |
| Markdown fenced code border| `--color-border`               | Border around code block container           |
| Markdown blockquote border | `--color-border-strong`        | Left border on blockquotes                   |
| Markdown blockquote text   | `--color-muted`                | Quoted text is visually recessed             |
| Markdown HR                | `--color-border`               | Horizontal rule color                        |
| Markdown table header bg   | `--color-surface-elevated`     | Header row background in GFM tables          |
| Markdown table border      | `--color-border`               | Table cell borders                           |
| Markdown table header text | `--color-foreground`           | Header row text, weight 600                  |
| Code language label        | `--color-subtle`               | Language indicator in top-right of code block |
| Shiki line number gutter   | `--color-subtle`               | Line numbers (if displayed)                  |

**Phase 7 card import/export colors:**

| Element                     | Color Token              | Usage                                       |
| --------------------------- | ------------------------ | ------------------------------------------- |
| Import error border         | `--color-destructive/20` | Error dialog border                         |
| Import error bg             | `--color-destructive/10` | Error dialog highlight area                 |
| Import error icon           | `--color-destructive`    | AlertTriangle icon in error dialog          |
| Export loading spinner      | `--color-accent`         | Spinner while generating PNG                |
| Import loading spinner      | `--color-accent`         | Spinner while parsing card file             |

**Accent reserved for (updated list — adds Phase 7 items):**

All Phase 1–6 accent reservations carry forward. Phase 7 adds:

- Markdown link text color
- Markdown inline code text color
- Import button (existing secondary variant — uses accent indirectly)
- Export loading spinner

---

## Copywriting Contract

### CTAs and Button Labels

| Element                          | Copy              | Context                                         |
| -------------------------------- | ----------------- | ----------------------------------------------- |
| "Import card" button             | "Import card"     | Characters library header, `Upload` icon prefix |
| "Export" kebab menu item         | "Export"          | CharacterCard kebab menu, `Download` icon prefix|
| Import error dialog dismiss      | "Close"           | Error dialog close button                       |
| Import error dialog title        | "Import failed"   | Error dialog heading                            |

### Empty States

No new empty states. The existing character library empty state ("No characters yet.") remains unchanged. Card import adds data — it does not create new empty states.

### Error States

**Import error dialog (per D-08):**

When card import fails, a modal error dialog shows the specific error:

| Error Condition              | Heading         | Body                                                                             |
| ---------------------------- | --------------- | -------------------------------------------------------------------------------- |
| Missing `chara` tEXt chunk   | Import failed   | "This PNG file does not contain a character card. Only TavernAI V2 format is supported." |
| Invalid JSON in card data    | Import failed   | "The character card data is corrupted or invalid. The file may be damaged."      |
| V1 format detected           | Import failed   | "TavernAI V1 cards are not supported. Please use a V2 format card."              |
| File too large (>10MB)       | Import failed   | "The file is too large to import. Maximum size is 10 MB."                        |
| Unsupported file format      | Import failed   | "Unsupported file format. Please use a PNG file with an embedded character card."|
| Unknown / generic error      | Import failed   | "Could not read the character card: {error detail}"                              |

**Error dialog visual contract:**

```
┌──────────────────────────────────────────────────────────────┐
│  Import failed                                            ✕  │
│                                                              │
│  ⚠ This PNG file does not contain a character card. Only     │
│    TavernAI V2 format is supported.                          │
│                                                              │
│                                            [ Close ]         │
└──────────────────────────────────────────────────────────────┘
```

Uses the existing `Dialog` component with destructive-tinted inline alert (same pattern as Phase 3 `DeleteConfirmationDialog`, but non-destructive styling — no red confirm button).

### Destructive Actions

No destructive actions in Phase 7. Import is additive (creates a new character). Export is read-only (reads character data, writes PNG). Neither action modifies or deletes existing data.

The import review flow (D-07) opens the `CharacterEditorDialog` pre-filled with imported data. The user can cancel at this point — no data is written until they explicitly click "Save" in the editor.

---

## Component Inventory

### New UI Primitives (`src/lib/components/ui/`)

None. All Phase 7 components use existing primitives.

### New Domain Components (`src/lib/components/`)

| Component                         | Location                                        | Purpose                                                       |
| --------------------------------- | ----------------------------------------------- | ------------------------------------------------------------- |
| `MarkdownRenderer.svelte`         | `src/lib/components/chat/MarkdownRenderer.svelte` | Renders sanitized GFM markdown HTML with Shiki code blocks  |
| `ImportErrorDialog.svelte`        | `src/lib/components/characters/ImportErrorDialog.svelte` | Modal showing specific import error details (D-08)  |

### Modified Existing Components

| Component                     | Change                    | What Changes                                                                                                                                                                                                                                                                                                 |
| ----------------------------- | ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `MessageBubble.svelte`        | Replace naive rendering   | (1) Remove `renderInlineMarkdown()` function entirely. (2) Replace `{@html renderInlineMarkdown(message.content)}` with conditional rendering: when `message.is_streaming` → show raw text with streaming-caret (existing behavior); when streaming completes → render `<MarkdownRenderer content={message.content} />`. (3) Remove `whitespace-pre-wrap` from the rendered-markdown path (markdown handles its own whitespace). (4) Keep `whitespace-pre-wrap` for the streaming/raw-text path only. |
| `characters/+page.svelte`     | Wire import button        | (1) Add onclick handler to existing "Import card" button: open native file picker via `tauri_plugin_dialog`, call backend `import_card` command, on success open `CharacterEditorDialog` in "create" mode pre-filled with parsed data (D-07), on error open `ImportErrorDialog` (D-08). (2) Add `import_error_open` state, `import_error_message` state. (3) Add `ImportErrorDialog` to the template. |
| `CharacterCard.svelte`        | Add export kebab item     | (1) Add `onexport?: (id: string) => void` prop. (2) Add `Download` icon import from lucide. (3) Add `<KebabMenuItem>` between Edit and the separator: `<Download class="size-3.5" /> Export`. (4) Add `<KebabMenuSeparator />` before the destructive Delete item to group: Edit | Export — separator — Delete. |
| `AppState` (`app.svelte.ts`)  | Import/export actions     | (1) Add `importCharacter(fileData)` action: calls backend `import_card`, returns parsed character data for editor pre-fill. (2) Add `exportCharacter(id)` action: calls backend `export_card`, opens native save dialog. (3) No new state fields — import result goes directly to editor. |
| `api.ts`                      | New invoke wrappers       | Add `importCard(filePath: string)` and `exportCard(characterId: string, savePath: string)` wrappers.                                                                                                                                                                                                          |
| `app.css`                     | Markdown prose styles     | Add `@layer components` block for `.prose-markdown` with GFM prose styling (paragraphs, headings, lists, tables, blockquotes, code blocks, horizontal rules). See § Custom Utilities below.                                                                                                                |

### What Does NOT Change

- No changes to design tokens (spacing, typography, color, radii) in `@theme` block — all prose styles are component-level
- No changes to existing page routes (only characters page gets handler wiring)
- No changes to ChatHeader, ConversationSidebar, AppHeader
- No changes to Dialog, Button, Input, Select, Tag, KebabMenu UI primitives
- No changes to MessageInput, PersonaSwitcher
- No changes to the welcome page or settings page
- No changes to streaming pipeline (Phase 5 infrastructure unchanged)
- No shadcn or third-party registry components

---

## Loading States

### Markdown Render Timing (from D-02)

| State             | Message Visual                                                  | Transition                                        |
| ----------------- | --------------------------------------------------------------- | ------------------------------------------------- |
| **Streaming**     | Raw text with `whitespace-pre-wrap`, streaming-caret `▍` suffix | Existing behavior — no change                     |
| **Just completed**| Raw text visible for ~1 frame, then markdown renders            | Swap is fast — `marked.parse()` + DOMPurify is synchronous |
| **Rendered**      | Full GFM markdown with Shiki-highlighted code blocks            | Stable state until message is edited              |
| **Code highlight**| Code blocks render plaintext first, then Shiki applies async    | Brief flash possible — mitigated by Shiki preloading on app init |

**No skeleton/placeholder during markdown render.** `marked.parse()` is synchronous and fast (< 5ms for typical AI responses). The swap from raw text to rendered markdown is effectively instantaneous.

**Shiki lazy-loading:** Shiki's grammar files are loaded asynchronously. On first code block render, there may be a brief plaintext display before highlighting applies. Mitigated by:
1. Preloading Shiki engine on app startup (in `+layout.svelte` `$effect`)
2. Highlighting is applied in a `$effect` that runs after initial render
3. Code blocks use the same `--color-surface` background in both states — only the text colors change

### Card Import States

| State              | Visual                                                                                        |
| ------------------ | --------------------------------------------------------------------------------------------- |
| **Idle**           | "Import card" button in header — normal secondary variant                                      |
| **File picking**   | Native Tauri file dialog open. App UI is accessible but button shows loading state            |
| **Parsing**        | "Import card" button shows `Loader2` spinner, label changes to "Importing…"                    |
| **Success**        | `CharacterEditorDialog` opens in "create" mode with imported fields pre-filled. Button resets. |
| **Error**          | `ImportErrorDialog` opens with specific error message. Button resets.                         |

### Card Export States

| State              | Visual                                                                                        |
| ------------------ | --------------------------------------------------------------------------------------------- |
| **Idle**           | "Export" item in CharacterCard kebab menu                                                      |
| **Generating PNG** | Kebab menu closes. No visible loading state (export is fast — < 500ms for typical cards). If the operation takes longer, the kebab menu re-opens with the Export item showing a `Loader2` spinner and disabled state. |
| **Save dialog**    | Native Tauri save dialog opens with pre-filled filename (`{character.name}.png`)               |
| **Saved**          | Dialog closes. No confirmation toast — file is saved.                                          |
| **Cancelled**      | User cancelled save dialog. No error shown.                                                    |
| **Error**          | If export fails, show a brief inline error: the kebab menu re-opens with Export item showing an `AlertTriangle` icon and the item is disabled. Click elsewhere to dismiss. |

---

## Interaction Patterns

### New Patterns in Phase 7

| Pattern                    | Trigger                                    | Behavior                                                                                                                                                                 | Notes                                                   |
| -------------------------- | ------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------- |
| **Import card**            | Click "Import card" button                 | Native file picker opens (`.png` and `.json` filter). On file selection, backend `import_card` parses the file. On success: `CharacterEditorDialog` opens pre-filled (D-07). On error: `ImportErrorDialog` opens (D-08). | Uses `tauri_plugin_dialog` (already installed) |
| **Review & save import**   | CharacterEditorDialog after import         | User reviews/edits imported character data (name, description, personality, scenario, greeting, dialogues, avatar). Click "Save" to persist. Click "Cancel" to discard import. | Standard `CharacterEditorDialog` in "create" mode |
| **Export card**            | Click "Export" in CharacterCard kebab menu | Backend `export_card` generates PNG with embedded `chara` tEXt chunk. Native save dialog opens pre-filled with `{character.name}.png`. User picks location and saves.   | Uses `tauri_plugin_dialog` (already installed) |
| **View rendered markdown** | Automatic on streaming completion          | When `message.is_streaming` transitions from `true` to `false`, the raw text in MessageBubble is replaced by the rendered markdown output. No user action needed.       | Seamless swap — raw text during streaming, rich markdown when done |

### Markdown Render Conditional (from D-02)

Inside `MessageBubble.svelte`, the bubble content rendering becomes:

```
{#if message.is_streaming}
  <!-- Raw text with streaming caret -->
  <p class="whitespace-pre-wrap break-words streaming-caret">
    {message.content}
  </p>
{:else}
  <!-- Rendered GFM markdown -->
  <div class="prose-markdown">
    <MarkdownRenderer content={message.content} />
  </div>
{/if}
```

This replaces the current `{@html renderInlineMarkdown(message.content)}` which runs unconditionally.

### Kebab Menu Layout Change (CharacterCard)

The existing kebab menu has two items: Edit, Delete (with separator between). Phase 7 adds Export:

```
<KebabMenu>
  <KebabMenuItem onclick={edit}>
    <Pencil /> Edit
  </KebabMenuItem>
  <KebabMenuItem onclick={export}>
    <Download /> Export
  </KebabMenuItem>
  <KebabMenuSeparator />
  <KebabMenuItem destructive onclick={delete}>
    <Trash2 /> Delete
  </KebabMenuItem>
</KebabMenu>
```

Edit and Export are grouped as non-destructive actions. Separator before Delete (destructive).

### Deferred Interactions (Not in Phase 7)

| Pattern                          | Deferred To | Reason                                    |
| -------------------------------- | ----------- | ----------------------------------------- |
| Drag-and-drop card import        | Future      | Button-based import per D-06              |
| JSON standalone export           | Future      | PNG-only per D-10                         |
| Bulk import/export               | Future      | Single card per action per CONTEXT.md     |
| Custom narration styling         | Future      | Standard italic per D-05                  |
| Markdown streaming render        | Future      | Raw text during streaming per D-02        |
| Message branching / forking      | Phase 8     | Branch navigation UI is a separate phase  |

---

## Custom Utilities

### `.prose-markdown` (NEW — added to `src/app.css`)

A `@layer components` block for GFM markdown prose styling inside chat bubbles. Applied to the wrapper `<div>` around `<MarkdownRenderer>` output.

```css
@layer components {
  .prose-markdown {
    /* Body text */
    & p {
      margin-bottom: 16px;
      line-height: 1.75;
    }
    & p:last-child {
      margin-bottom: 0;
    }

    /* Headings */
    & h1 {
      font-size: 24px;
      font-weight: 600;
      line-height: 1.2;
      margin-top: 24px;
      margin-bottom: 8px;
    }
    & h1:first-child { margin-top: 0; }

    & h2 {
      font-size: 20px;
      font-weight: 600;
      line-height: 1.25;
      margin-top: 20px;
      margin-bottom: 8px;
    }
    & h2:first-child { margin-top: 0; }

    & h3 {
      font-size: 16px;
      font-weight: 600;
      line-height: 1.3;
      margin-top: 16px;
      margin-bottom: 8px;
    }
    & h3:first-child { margin-top: 0; }

    & h4, & h5, & h6 {
      font-size: 15px;
      font-weight: 600;
      margin-top: 16px;
      margin-bottom: 4px;
    }

    /* Strong / Bold */
    & strong {
      font-weight: 600;
    }

    /* Emphasis / Italic — narration text uses muted color (D-05) */
    & em {
      color: var(--color-muted);
    }

    /* Inline code */
    & code:not(pre code) {
      font-family: var(--font-mono);
      font-size: 13px;
      background: var(--color-surface-elevated);
      color: var(--color-accent);
      padding: 2px 6px;
      border-radius: var(--radius-sm);
    }

    /* Fenced code blocks */
    & pre {
      background: var(--color-surface);
      border: 1px solid var(--color-border);
      border-radius: var(--radius-lg);
      padding: 16px;
      margin: 16px 0;
      overflow-x: auto;
      position: relative;
    }
    & pre code {
      font-family: var(--font-mono);
      font-size: 13px;
      line-height: 1.5;
      background: transparent;
      color: var(--color-foreground);
      padding: 0;
    }

    /* Code block language label */
    & pre .lang-label {
      position: absolute;
      top: 8px;
      right: 12px;
      font-family: var(--font-sans);
      font-size: 12px;
      color: var(--color-subtle);
      text-transform: uppercase;
      letter-spacing: 0.05em;
      pointer-events: none;
    }

    /* Links */
    & a {
      color: var(--color-accent);
      text-decoration: none;
    }
    & a:hover {
      text-decoration: underline;
    }

    /* Blockquotes */
    & blockquote {
      border-left: 3px solid var(--color-border-strong);
      padding-left: 16px;
      padding-top: 4px;
      padding-bottom: 4px;
      margin: 16px 0;
      color: var(--color-muted);
    }
    & blockquote p {
      margin-bottom: 0;
    }

    /* Unordered lists */
    & ul {
      list-style-type: disc;
      padding-left: 24px;
      margin: 8px 0;
    }
    & ul li {
      margin-bottom: 4px;
      line-height: 1.75;
    }
    & ul li::marker {
      color: var(--color-subtle);
    }

    /* Ordered lists */
    & ol {
      list-style-type: decimal;
      padding-left: 24px;
      margin: 8px 0;
    }
    & ol li {
      margin-bottom: 4px;
      line-height: 1.75;
    }
    & ol li::marker {
      color: var(--color-subtle);
    }

    /* Nested lists */
    & ul ul, & ol ol, & ul ol, & ol ul {
      margin-top: 4px;
      margin-bottom: 4px;
    }

    /* Horizontal rule */
    & hr {
      border: none;
      border-top: 1px solid var(--color-border);
      margin: 16px 0;
    }

    /* GFM Tables */
    & table {
      width: 100%;
      border-collapse: collapse;
      margin: 16px 0;
      font-size: 14px;
    }
    & th {
      background: var(--color-surface-elevated);
      font-weight: 600;
      text-align: left;
      padding: 8px 12px;
      border: 1px solid var(--color-border);
    }
    & td {
      padding: 8px 12px;
      border: 1px solid var(--color-border);
    }
    & tr:hover td {
      background: var(--color-surface-hover);
    }

    /* GFM Task lists */
    & ul:has(> li > input[type="checkbox"]) {
      list-style-type: none;
      padding-left: 8px;
    }
    & input[type="checkbox"] {
      accent-color: var(--color-accent);
      margin-right: 8px;
      vertical-align: middle;
    }

    /* GFM Strikethrough */
    & del {
      text-decoration: line-through;
      color: var(--color-subtle);
    }

    /* Images (AI responses may include markdown images) */
    & img {
      max-width: 100%;
      border-radius: var(--radius-md);
      margin: 8px 0;
    }
  }
}
```

**Note:** All colors reference existing `--color-*` tokens. No new color values. The `prose-markdown` class scopes all styles to avoid leaking into non-markdown UI.

---

## Registry Safety

**No changes.** No shadcn, no third-party registries. All components remain custom-built using Bits.ui + Tailwind v4.

**New npm dependencies (frontend only):**

| Package     | Version  | Purpose                                    | Bundle Impact           |
| ----------- | -------- | ------------------------------------------ | ----------------------- |
| `marked`    | ^15.x    | GFM markdown parser (D-01, D-03)           | ~40KB minified          |
| `dompurify` | ^3.x     | HTML sanitizer for AI-generated content    | ~20KB minified          |
| `shiki`     | ^3.x     | Syntax highlighting for code blocks (D-04) | ~200KB lazy-loaded      |

**Security note on `dompurify`:** All markdown output passes through DOMPurify before rendering via `{@html}`. This prevents XSS from malicious AI-generated content (e.g., `<script>` tags, `javascript:` URLs, event handlers). DOMPurify is the industry-standard sanitizer for this use case.

**Shiki lazy-loading:** Shiki and its grammar files are loaded on demand. The initial app bundle does not include Shiki. On first code block render, Shiki's wasm engine and relevant language grammars are fetched. Subsequent renders use the cached engine. Preloading is triggered on app startup (in `+layout.svelte` via `$effect`) to minimize the first-render delay.

| Registry        | Blocks Used | Safety Gate    |
| --------------- | ----------- | -------------- |
| shadcn official | none        | not applicable |
| Third-party     | none        | not applicable |

---

## Phase 7 Specifics

### Discretionary Decisions Made

The following decisions were in "Claude's Discretion" in CONTEXT.md and are hereby declared:

1. **Markdown library: `marked` (not `markdown-it`):** `marked` is chosen over `markdown-it` for GFM rendering (D-01, D-03). Rationale: (a) Lighter bundle (~40KB vs ~60KB). (b) GFM support is built-in, no plugins needed. (c) Synchronous API — `marked.parse(text)` returns HTML immediately, no async/Promise overhead. (d) Well-maintained, 33K+ GitHub stars, used by npm, VuePress, and others. (e) Since we render only on completion (D-02), streaming capability is irrelevant — both libraries handle this equally. (f) `marked` has a simpler extension API which is sufficient for our needs (custom renderer for code blocks to integrate Shiki).

2. **Sanitization: DOMPurify:** All markdown HTML output passes through `DOMPurify.sanitize(html)` before rendering via `{@html}`. AI-generated content is untrusted — it may contain `<script>` tags, event handlers, or malicious URLs. DOMPurify strips these while preserving safe HTML. Configuration: `ALLOWED_TAGS` includes standard GFM elements (p, h1–h6, strong, em, code, pre, a, ul, ol, li, blockquote, table, thead, tbody, tr, th, td, hr, img, input[type=checkbox], del, br, span). `ALLOWED_ATTR` includes href (with URL validation), class, language data attributes for Shiki.

3. **MarkdownRenderer component structure:** `src/lib/components/chat/MarkdownRenderer.svelte` is a new component with:
   - Props: `content: string` (raw markdown text)
   - Internal flow: `marked.parse(content)` → `DOMPurify.sanitize(html)` → `{@html sanitizedHtml}`
   - `$effect` on `content` changes: re-parse and sanitize
   - Code block post-processing: after initial render, `$effect` scans for `<pre><code class="language-xxx">` blocks and applies Shiki highlighting asynchronously, replacing the plaintext with highlighted HTML
   - No Shadow DOM — styles are applied via the `.prose-markdown` parent class in `app.css`

4. **Shiki theme: `vitesse-dark`:** The Shiki theme for syntax highlighting is `vitesse-dark`. Rationale: (a) Warm, low-contrast palette that complements the app's OKLCH surface colors. (b) Not blue-tinted like VS Code's default `dark-plus`. (c) Popular in developer tooling (Vite, Nuxt). (d) Good language support. (e) Only one theme loaded — dark theme only per project constraint (light theme deferred to Phase 10).

5. **Code blocks without language specification:** When a fenced code block has no language tag (e.g., ` ``` ` instead of ` ```rust `), the code block renders in a monospace `<pre>` container with `--color-surface` background but **no Shiki highlighting**. Plain monospace text with `--color-foreground` color. No language label is shown. This avoids loading unnecessary grammars and prevents Shiki errors on unknown languages.

6. **Avatar storage for imported cards:** When importing a card that includes an embedded avatar image (base64 in the card JSON), the avatar is saved to the same asset directory used by the avatar upload flow (`upload_avatar` Tauri command). The backend decodes the base64 data, writes it to the assets directory, and returns the file path. The `CharacterEditorDialog` receives this path as the pre-filled `avatar_path`. This reuses the existing avatar storage infrastructure — no new storage mechanism.

7. **Import button loading state:** The "Import card" button enters a loading state (spinner + "Importing…" label) while the backend parses the card file. The button is disabled during this state to prevent double-clicks. On success, the editor opens and the button resets. On error, the error dialog opens and the button resets. Typical parse time: < 100ms for PNG tEXt extraction and JSON decoding.

8. **Export operation is fire-and-forget:** When the user clicks "Export" in the kebab menu, the export operation runs and opens the native save dialog. There is no success confirmation (no toast, no status message). The save dialog itself confirms the action — the user chose where to save. If the export fails (e.g., disk full, permission denied), an inline error shows on the kebab menu item briefly. This matches the pattern of other desktop applications (file → save as).

### Frontend Change Scope Summary

1. **MarkdownRenderer.svelte** — New component: ~80 lines. Takes `content` prop, parses with `marked`, sanitizes with DOMPurify, renders HTML, applies Shiki to code blocks.
2. **ImportErrorDialog.svelte** — New component: ~40 lines. Dialog with destructive-tinted alert, specific error message, "Close" button.
3. **MessageBubble.svelte** — ~20 lines changed: remove `renderInlineMarkdown()`, add streaming/complete conditional, render `<MarkdownRenderer>` when complete.
4. **characters/+page.svelte** — ~30 lines added: import handler, import state, `ImportErrorDialog` instance, wiring to existing "Import card" button.
5. **CharacterCard.svelte** — ~8 lines added: `onexport` prop, `Download` icon import, Export kebab menu item.
6. **app.svelte.ts** — ~15 lines added: `importCharacter()` action, `exportCharacter()` action.
7. **api.ts** — ~10 lines added: `importCard()`, `exportCard()` invoke wrappers.
8. **app.css** — ~150 lines added: `.prose-markdown` component styles.
9. **types/index.ts** — ~5 lines added: `ImportResult` type for parsed card data.

Total: ~350 lines of frontend code. The remaining work is backend (Rust) — porting `card_io.rs` and wiring Tauri commands.

### New Icon Imports

| Icon       | From               | Used In                      |
| ---------- | ------------------ | ---------------------------- |
| `Download` | `@lucide/svelte`   | CharacterCard kebab menu     |

`Upload` icon is already imported in `characters/+page.svelte` — reused for the import button (already wired in the template, just needs the onclick handler).

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS
- [ ] Dimension 2 Visuals: PASS
- [ ] Dimension 3 Color: PASS
- [ ] Dimension 4 Typography: PASS
- [ ] Dimension 5 Spacing: PASS
- [ ] Dimension 6 Registry Safety: PASS

**Approval:** pending
