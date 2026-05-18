# Phase 7: Rich Output & Cards - Context

**Gathered:** 2026-05-16
**Status:** Ready for planning

<domain>
## Phase Boundary

AI responses render as rich markdown (bold, italic, code blocks, lists, headers), and users can import/export character cards as TavernAI V2 PNG files. This phase delivers: (1) replacing the naive `renderInlineMarkdown()` in MessageBubble with a real markdown renderer, (2) porting the card I/O service from gtk_ref for PNG tEXt chunk read/write, and (3) wiring import/export to the character library UI. No new capabilities — real AI streaming, real token counting, and conversation branching are separate phases.

</domain>

<decisions>
## Implementation Decisions

### Markdown Rendering

- **D-01:** Frontend JS library for markdown rendering — NOT backend pulldown-cmark-to-HTML. The frontend receives raw markdown text and renders it client-side. This keeps the streaming path simple (text tokens) and avoids sending large HTML payloads over IPC.
- **D-02:** Show raw text during streaming, render rich markdown only when the response completes. No re-rendering on every token. The streaming caret (`▍`) displays on raw text; once `is_streaming` becomes false, the message renders as formatted markdown.
- **D-03:** Full GFM (GitHub Flavored Markdown) support — tables, task lists, strikethrough, fenced code blocks with language labels, in addition to standard bold/italic/headers/lists/links/blockquotes.
- **D-04:** Shiki for syntax highlighting in code blocks. VS Code-quality highlighting, dark theme compatible, lazy-loaded to manage bundle size.
- **D-05:** Standard italic for single-asterisk narration text. No custom narration styling — roleplay `*narration*` naturally maps to italic, which is the expected behavior.

### Card Import UX

- **D-06:** Import triggered by a button on the Characters library page (alongside existing "New Character" button). Uses Tauri's native file picker via `tauri_plugin_dialog` (already installed). Accepts `.png` and `.json` files.
- **D-07:** After parsing, open the CharacterEditorDialog pre-filled with imported data. User reviews/edits fields, then saves. This catches import errors before committing to the database.
- **D-08:** Import errors shown in a detailed error dialog — specific messages for: missing `chara` tEXt chunk, invalid JSON, V1 format not supported, file too large (>10MB), unsupported format.

### Card Export UX

- **D-09:** Export triggered from the kebab menu on CharacterCard (alongside existing Edit and Delete items). Natural location — always accessible from the library overview.
- **D-10:** Export format is PNG only (TavernAI V2). Character's avatar image with card JSON embedded in a `chara` tEXt chunk. Compatible with SillyTavern, Agnai, and other TavernAI V2 consumers.
- **D-11:** Native Tauri save dialog (`tauri_plugin_dialog`) for choosing save location and filename. Pre-filled with character name as filename suggestion.

### Agent's Discretion

- Which JS markdown library to use (marked, markdown-it, etc.) — choose based on GFM support, streaming capability, and bundle size
- Exact MarkdownMessage component structure and how it replaces `renderInlineMarkdown()` in MessageBubble
- Shiki theme selection for dark mode
- How to handle code block language detection when no language is specified
- Whether imported avatar images get copied to the asset directory or stored inline
- Loading states during import parsing and export file generation
- Whether the import button has an icon (Upload icon from lucide already imported in characters page)

</decisions>

<canonical_refs>

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Reference Implementation (MUST read — patterns to port)

- `gtk_ref/src/service/card_io.rs` — Full PNG tEXt chunk read/write, V2 spec parsing, V1 migration, base64 encoding, avatar embedding. Battle-tested with comprehensive tests.
- `gtk_ref/src/service/markdown.rs` — Markdown-to-Pango rendering using pulldown-cmark. Port the parsing logic but output HTML instead of Pango. Security escaping pattern (prevent XSS from AI responses).

### Prior Phase Decisions (carry-forward)

- `.planning/phases/06-smart-prompts/06-CONTEXT.md` — Prompt engine, context budget display, streaming transport patterns
- `.planning/phases/05-live-conversations/05-CONTEXT.md` — Streaming Channel pattern, message persistence timing, MessageBubble edit/regenerate actions
- `.planning/phases/03-editors/03-CONTEXT.md` — CharacterEditorDialog pattern, avatar upload via Tauri, kebab menu on CharacterCard
- `.planning/phases/02-persistent-data/02-CONTEXT.md` — DbWorker pattern, Tauri command design (D-15/D-16), UUID text PKs, JSON in TEXT columns
- `.planning/phases/01-native-desktop-shell/01-CONTEXT.md` — Tauri plugin setup, `tauri_plugin_dialog` and `tauri_plugin_fs` already installed

### Active Backend (will be extended)

- `src-tauri/src/lib.rs` — Command registration, plugin initialization. Add `render_markdown`, `import_card`, `export_card` commands.
- `src-tauri/src/service/mod.rs` — Service module registry. Add `card_io.rs` module.
- `src-tauri/src/commands/` — Existing command handlers. Add card I/O commands.
- `src-tauri/src/domain/` — Domain models. May need `CharaCardV2Spec`, `CharacterV1` types for import parsing.

### Active Frontend (will be modified)

- `src/lib/components/chat/MessageBubble.svelte` — Replace `renderInlineMarkdown()` with real markdown rendering. Add conditional: raw text while streaming, rendered markdown when complete.
- `src/routes/characters/+page.svelte` — Add import button, wire import action. Already imports `Upload` icon from lucide.
- `src/lib/components/characters/CharacterCard.svelte` — Add "Export" item to kebab menu.
- `src/lib/data/api.ts` — Add `import_card()`, `export_card()` invoke wrappers.
- `src/lib/stores/app.svelte.ts` — Add `importCharacter()` action (parse, open editor).
- `src/app.css` — May need markdown prose styling utilities.

### Project Planning

- `.planning/ROADMAP.md` — Phase 7 definition, success criteria, plan list
- `.planning/REQUIREMENTS.md` — CHAT-06, CHAR-06, CHAR-07 requirements
- `.planning/PROJECT.md` — Project vision and constraints

</canonical_refs>

<code_context>

## Existing Code Insights

### Reusable Assets

- **`CharacterEditorDialog`**: Already exists as a modal for creating/editing characters. Can be opened in "import review" mode with pre-filled fields from parsed card data.
- **`KebabMenu`/`KebabMenuItem`**: UI primitives already used on CharacterCard for Edit/Delete. Add Export item following the same pattern.
- **`upload_avatar` Tauri command**: File picker + asset storage pattern already works. Can be reused/adapted for card import file selection.
- **`tauri_plugin_dialog`**: Already installed and configured for file dialogs.
- **`MessageBubble.svelte`**: Already has `is_streaming` check — `message.is_streaming` drives the streaming caret. Natural conditional point for raw-vs-rendered switch.
- **`DeleteConfirmationDialog`**: Existing dialog component — pattern for the import error dialog.

### Established Patterns

- **Tauri IPC via invoke()**: Commands registered in `lib.rs`, called from `api.ts`. Flat command names, async, `Result<T, String>`.
- **AppState actions**: CRUD methods with optimistic updates and revert-on-error (see `createCharacter`, `updateCharacter`).
- **Barrel exports**: New components exported from `src/lib/components/`.
- **Tailwind v4 theme tokens**: Markdown prose styling should use existing `--color-*` tokens for consistency.

### Integration Points

- **`src-tauri/src/service/mod.rs`**: Add `pub mod card_io;` — port from `gtk_ref/src/service/card_io.rs`
- **`src-tauri/src/lib.rs`**: Register `import_card`, `export_card` commands in `invoke_handler`
- **`src-tauri/Cargo.toml`**: Add `png` crate for PNG tEXt chunk read/write (already in gtk_ref Cargo.toml)
- **`src/lib/components/chat/MessageBubble.svelte`**: Replace `renderInlineMarkdown()` call with markdown component
- **`src/routes/characters/+page.svelte`**: Add import button next to "New Character"
- **`src/lib/components/characters/CharacterCard.svelte`**: Add Export to kebab menu

</code_context>

<specifics>
## Specific Ideas

- Show raw text during streaming with the `▍` caret, then swap to rendered markdown when complete — this avoids the visual glitch of partial markup rendering
- Exported PNG should use the character's avatar as the image — if no avatar, generate a placeholder (solid color with initials)
- Import button on library page should use the `Upload` lucide icon (already imported in characters page)

</specifics>

<deferred>
## Deferred Ideas

- Drag-and-drop card import — could be added as an enhancement to the library page button
- JSON standalone export option — PNG-only for now, JSON export can be added if users request it
- Bulk import/export — importing/exporting multiple cards at once is a future enhancement
- Custom narration styling for asterisk text — standard italic is sufficient for now

</deferred>

---

_Phase: 07-rich-output-cards_
_Context gathered: 2026-05-16_
