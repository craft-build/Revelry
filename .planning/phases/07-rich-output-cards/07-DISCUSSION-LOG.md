# Phase 7: Rich Output & Cards - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-05-16
**Phase:** 07-rich-output-cards
**Areas discussed:** Markdown rendering approach, Markdown styling & features, Card import UX & flow, Card export UX & flow

---

## Markdown Rendering Approach

### Rendering engine

| Option                          | Description                                                                      | Selected |
| ------------------------------- | -------------------------------------------------------------------------------- | -------- |
| Backend → HTML (pulldown-cmark) | Rust converts markdown to sanitized HTML, frontend receives ready-to-render HTML |          |
| Frontend JS library             | Frontend receives raw markdown text and renders client-side                      | ✓        |
| Hybrid: backend + frontend      | Backend parses with pulldown-cmark, frontend applies final styling/classes       |          |

**User's choice:** Frontend JS library
**Notes:** Keeps the streaming path simple (text tokens), avoids large HTML payloads over IPC. The backend still sends raw markdown text via the existing Channel streaming pattern.

### Streaming behavior

| Option                                 | Description                                                               | Selected |
| -------------------------------------- | ------------------------------------------------------------------------- | -------- |
| Re-render on every token               | Each token triggers fresh markdown-to-HTML pass on accumulated text       |          |
| Render only on complete blocks         | Wait for paragraph/block boundaries before rendering                      |          |
| Show raw during stream, render on done | Keep plain-text during streaming, render rich markdown only on completion | ✓        |

**User's choice:** Show raw during stream, render on done
**Notes:** Simplest approach. The existing streaming caret (`▍`) works on raw text. Once `is_streaming` becomes false, the message renders as formatted markdown. Avoids flicker from partial markup.

---

## Markdown Styling & Features

### Feature scope

| Option            | Description                                                                | Selected |
| ----------------- | -------------------------------------------------------------------------- | -------- |
| Full GFM          | Tables, task lists, strikethrough, fenced code blocks with language labels | ✓        |
| CommonMark only   | Bold, italic, headers, lists, code blocks, blockquotes, links              |          |
| Extended + extras | GFM plus syntax highlighting, math/KaTeX, footnotes, custom containers     |          |

**User's choice:** Full GFM
**Notes:** Covers nearly everything AI models generate. Tables and strikethrough are common in AI responses.

### Syntax highlighting

| Option          | Description                                                               | Selected |
| --------------- | ------------------------------------------------------------------------- | -------- |
| Shiki           | VS Code-quality highlighting, dark theme compatible, lazy-loaded (~200KB) | ✓        |
| Prism/hljs      | Lighter-weight highlighters, smaller bundle                               |          |
| No highlighting | Monospace font, distinct background, no language-specific coloring        |          |

**User's choice:** Shiki
**Notes:** Lazy-loaded to manage bundle size. Dark theme compatible with the app's dark-first design.

### Narration styling

| Option                 | Description                                                             | Selected |
| ---------------------- | ----------------------------------------------------------------------- | -------- |
| Standard italic        | Single asterisks render as italic, same as normal markdown              | ✓        |
| Custom narration style | Detect single-asterisk text and apply special style (color, serif font) |          |

**User's choice:** Standard italic
**Notes:** Roleplay `*narration*` naturally maps to italic. No special treatment needed.

---

## Card Import UX & Flow

### Import trigger location

| Option              | Description                                                 | Selected |
| ------------------- | ----------------------------------------------------------- | -------- |
| Library page button | "Import" button on Characters tab alongside "New Character" | ✓        |
| Drag-and-drop       | Drop PNG/JSON onto library page                             |          |
| Both                | Button + drag-and-drop                                      |          |

**User's choice:** Library page button
**Notes:** Uses Tauri's native file picker. The `Upload` lucide icon is already imported in the characters page.

### Post-import flow

| Option                | Description                                                                | Selected |
| --------------------- | -------------------------------------------------------------------------- | -------- |
| Open editor to review | Parse card, open CharacterEditorDialog pre-filled, user reviews then saves | ✓        |
| Save directly         | Parse and save to library immediately, edit later                          |          |
| Quick confirmation    | Lightweight dialog showing name/avatar/tags, not full editor               |          |

**User's choice:** Open editor to review
**Notes:** Catches import errors before committing. Reuses the existing CharacterEditorDialog.

### Error display

| Option                    | Description                                                                   | Selected |
| ------------------------- | ----------------------------------------------------------------------------- | -------- |
| Error dialog with details | Specific messages for missing chara chunk, invalid JSON, file too large, etc. | ✓        |
| Generic error message     | Simple "Failed to import" message                                             |          |

**User's choice:** Error dialog with details
**Notes:** Helps users understand what went wrong. Follows the existing DeleteConfirmationDialog pattern for modal dialogs.

---

## Card Export UX & Flow

### Export trigger location

| Option               | Description                                                      | Selected |
| -------------------- | ---------------------------------------------------------------- | -------- |
| Kebab menu item      | Add "Export" to CharacterCard kebab menu (alongside Edit/Delete) | ✓        |
| Inside editor dialog | Export button inside CharacterEditorDialog                       |          |
| Both locations       | Kebab menu and editor dialog                                     |          |

**User's choice:** Kebab menu item
**Notes:** Always accessible from library overview. Follows existing kebab menu pattern.

### Export format

| Option                 | Description                                   | Selected |
| ---------------------- | --------------------------------------------- | -------- |
| PNG (TavernAI V2) only | Avatar image with embedded JSON in tEXt chunk | ✓        |
| PNG or JSON choice     | User picks format at export time              |          |

**User's choice:** PNG (TavernAI V2) only
**Notes:** Standard TavernAI V2 format compatible with SillyTavern, Agnai, etc. JSON can be added later if requested.

### Save mechanism

| Option                 | Description                                                               | Selected |
| ---------------------- | ------------------------------------------------------------------------- | -------- |
| Native save dialog     | Tauri's `tauri_plugin_dialog` save dialog, pre-filled with character name | ✓        |
| Auto-save to Downloads | Save to default location with toast notification                          |          |

**User's choice:** Native save dialog
**Notes:** User controls location and filename. Pre-filled with character name as suggestion.

---

## Agent's Discretion

- Which JS markdown library (marked, markdown-it, etc.) — agent selects based on GFM support and bundle size
- Exact MarkdownMessage component structure
- Shiki theme selection for dark mode
- Code block language detection when unspecified
- Imported avatar image storage (asset directory vs inline)
- Loading states during import/export
- Import button icon (Upload icon already imported)

## Deferred Ideas

- Drag-and-drop card import — enhancement to library page button
- JSON standalone export — PNG-only for now
- Bulk import/export — future enhancement
- Custom narration styling — standard italic sufficient
