# Phase 7 Research: Rich Output & Cards

**Researched:** 2026-05-16
**Status:** Complete

---

## Summary

Phase 7 has two independent workstreams: (1) replacing the naive `renderInlineMarkdown()` in MessageBubble with a real GFM markdown renderer, and (2) porting the card I/O service from `gtk_ref/` for TavernAI V2 PNG import/export. Both workstreams are well-understood with clear reference implementations and minimal ambiguity. The phase is low-risk — the reference `card_io.rs` is battle-tested with comprehensive tests, and the markdown rendering is a straightforward frontend library integration.

**Key finding:** The UI-SPEC has already resolved all discretionary decisions. The plan can proceed directly from the CONTEXT.md decisions and UI-SPEC specifics without further research.

---

## Research Findings

### R1: Markdown Rendering — Frontend JS Library

**Decision already made (D-01):** Frontend JS library, NOT backend Rust rendering.

**Library chosen (UI-SPEC § Discretionary Decisions):** `marked` v15.x

| Aspect            | Detail                                              |
| ----------------- | --------------------------------------------------- |
| GFM support       | Built-in, no plugins needed                         |
| Bundle size       | ~40KB minified                                      |
| API               | Synchronous: `marked.parse(text)` → HTML string     |
| Streaming         | Not needed — render only on completion (D-02)       |
| Code highlighting | Custom renderer extension for `<pre><code>` → Shiki |
| Sanitization      | DOMPurify v3.x (~20KB) after `marked.parse()`       |

**Alternatives considered and rejected:**

- `markdown-it`: Heavier (~60KB), plugin-based GFM, async overhead not needed
- Backend pulldown-cmark → HTML: Large HTML payloads over IPC, breaks streaming pipeline simplicity

**Key integration pattern:**

```
marked.parse(rawMarkdown) → DOMPurify.sanitize(html) → {@html sanitizedHtml}
```

**Svelte 5 integration note:** `marked.parse()` is synchronous and fast (<5ms typical). No need for Web Workers or async rendering. The swap from raw streaming text to rendered markdown is effectively instantaneous.

### R2: Reference `card_io.rs` — Data Flow

The `gtk_ref/src/service/card_io.rs` is a complete, battle-tested implementation with 12 test cases. It handles:

**Import flow:**

1. Read PNG bytes
2. Use `png` crate decoder to extract tEXt chunks
3. Find `chara` tEXt chunk (base64-encoded JSON)
4. Base64-decode → UTF-8 JSON string
5. Parse JSON: try V2 spec format (`{spec, spec_version, data: {...}}`) first, then flat V2, then V1
6. Normalize card fields (extract W++ personality blocks, dialogue examples, genre metadata from `description`)
7. Return `(CharacterCardV2, Option<Vec<u8>>)` — character + avatar PNG bytes

**Export flow:**

1. Serialize `CharacterCardV2` to JSON
2. Base64-encode JSON
3. If avatar bytes provided: re-encode avatar PNG with `chara` tEXt chunk added
4. If no avatar: create minimal 1×1 RGBA PNG with tEXt chunk
5. Return PNG bytes

**Critical detail — PNG reading requires consuming image data first:**
The `png` crate requires calling `reader.next_frame()` before accessing text chunks. The reference code handles this correctly. Any port must do the same.

**Domain types needed:**

- `CharaCardV2Spec` — wrapper with `spec`, `spec_version`, `data` fields
- `CharaCardV2SpecData` — the nested data payload
- `CharacterV1` — V1 format with `char_name`, `char_persona`, etc.

All three types exist in `gtk_ref/src/domain/character.rs` and can be ported directly. The Tauri backend's `domain/character.rs` already has `CharacterCardV2` but is missing `CharaCardV2Spec`, `CharaCardV2SpecData`, and `CharacterV1`.

### R3: Existing Backend vs. What's Needed

**Already exists in Tauri backend:**

- `CharacterCardV2` domain model (with serde defaults, UUID generation)
- `upload_avatar` / `delete_avatar` commands (file picker + asset storage)
- `tauri_plugin_dialog` installed and configured for file dialogs
- `tauri_plugin_fs` installed
- Avatar storage at `{app_data}/avatars/{id}.{ext}`
- Command registration pattern in `lib.rs`
- Service module pattern in `service/mod.rs`

**Must be added:**

- `CharaCardV2Spec`, `CharaCardV2SpecData`, `CharacterV1` domain types (port from gtk_ref)
- `service/card_io.rs` module (port from gtk_ref, adapt for Tauri backend)
- `commands/card_io.rs` — Tauri command handlers for `import_card`, `export_card`
- Cargo.toml: add `png` crate (for tEXt chunk read/write), `base64` crate (for encoding/decoding)
- `service/mod.rs`: add `pub mod card_io;`
- `lib.rs`: register `import_card`, `export_card` in invoke_handler

**Minor adaptation needed:** The gtk_ref uses `chrono::NaiveDateTime` for timestamps, while the Tauri backend uses `chrono::DateTime<Utc>`. The `CharaCardV2Spec::to_domain()` must be adapted to use `Utc::now()` instead of `Local::now().naive_local()`.

### R4: Frontend Modification Scope

**MessageBubble.svelte** (current: uses `renderInlineMarkdown()`):

- Remove the `renderInlineMarkdown()` function entirely
- Add conditional: `message.is_streaming` → raw text with streaming-caret (existing); `!is_streaming` → `<MarkdownRenderer content={message.content} />`
- Remove `whitespace-pre-wrap` from the rendered path (markdown handles whitespace)

**Key observation:** MessageBubble already checks `message.is_streaming` for the `streaming-caret` class. The conditional rendering point is already clear in the code.

**characters/+page.svelte:**

- The "Import card" button already exists in the template (line with `<Button variant="secondary" size="sm">` with `<Upload>` icon)
- Needs: `onclick` handler, `import_error_open` state, `import_error_message` state, `ImportErrorDialog` instance
- The handler opens native file picker, calls backend, on success opens `CharacterEditorDialog` pre-filled

**CharacterCard.svelte:**

- Add `onexport?: (id: string) => void` prop
- Add `Download` icon import from lucide
- Add Export `<KebabMenuItem>` between Edit and separator-before-Delete

**api.ts:**

- Add `importCard(filePath: string): Promise<CharacterCard>` invoke wrapper
- Add `exportCard(characterId: string, savePath: string): Promise<void>` invoke wrapper

**app.svelte.ts (AppState):**

- Add `importCharacter(filePath: string)` action
- Add `exportCharacter(id: string)` action

**types/index.ts:**

- No new types strictly needed — the parsed card data uses existing `CharacterCard` type

### R5: Shiki Integration

**Decision made (D-04, UI-SPEC):** Shiki v3.x for syntax highlighting, `vitesse-dark` theme.

**Integration approach:**

1. Install `shiki` as npm dependency
2. Create Shiki singleton/handler (lazy-loaded, initialized on first use or preloaded on app startup)
3. `MarkdownRenderer.svelte` uses `marked` custom renderer for code blocks → outputs `<pre><code class="language-xxx">` with raw text initially
4. `$effect` after render: find code blocks, apply Shiki highlighting async, replace innerHTML
5. Theme: `vitesse-dark` — only dark theme loaded (light theme deferred)

**Code blocks without language:** No Shiki highlighting, no language label. Just monospace text in `<pre>` container.

**Preloading strategy (UI-SPEC):** Trigger Shiki engine preload in `+layout.svelte` `$effect` to minimize first-render delay for code blocks.

### R6: Import/Export UX Flow

**Import flow (from D-06, D-07, D-08):**

1. User clicks "Import card" button → native file picker opens (`.png`, `.json` filters)
2. File selected → frontend calls `api.importCard(filePath)` → backend reads file, parses card
3. Backend returns parsed `CharacterCard` (with new UUID, normalized fields)
4. Frontend opens `CharacterEditorDialog` in "create" mode with pre-filled fields
5. User reviews/edits → clicks "Save" → existing `saveCharacter` flow
6. On error: `ImportErrorDialog` with specific message (D-08 error table)

**Export flow (from D-09, D-10, D-11):**

1. User clicks "Export" in CharacterCard kebab menu
2. Frontend calls backend to generate PNG → native save dialog opens with `{character.name}.png` pre-filled
3. User picks location → file saved
4. No success confirmation, no toast (fire-and-forget)

**Avatar handling for import:** The reference implementation returns `(CharacterCardV2, Option<Vec<u8>>)` — the PNG bytes ARE the avatar. The Tauri command must save these bytes to the avatar directory and set `avatar_path` on the returned character before sending to frontend.

**Avatar handling for export:** The Tauri command reads the character's `avatar_path` file (if exists) as bytes, passes to `card_io::export_to_png()`. If no avatar, a 1×1 transparent PNG is generated.

### R7: Dependencies to Add

**Cargo.toml (Rust backend):**
| Crate | Version | Purpose |
|-------|---------|---------|
| `png` | ~0.18 | PNG tEXt chunk read/write (matches gtk_ref) |
| `base64` | ~0.22 | Base64 encode/decode for chara chunk |

Note: `regex` is NOT needed — the W++ normalization in gtk_ref uses `regex`, but the Tauri backend's `CharacterCardV2` uses `DateTime<Utc>` (not `NaiveDateTime`). The normalization logic can be ported with `regex` added as a dependency, or simplified. The UI-SPEC does not mention W++ normalization as a Phase 7 requirement — it's an enhancement from the reference. Decision: include it for completeness (regex is already a common dependency).

**package.json (Frontend):**
| Package | Version | Purpose | Bundle Impact |
|---------|---------|---------|---------------|
| `marked` | ^15.x | GFM markdown parser | ~40KB |
| `dompurify` | ^3.x | HTML sanitizer | ~20KB |
| `shiki` | ^3.x | Syntax highlighting | ~200KB lazy-loaded |

### R8: Rust Crate Version Compatibility

The Tauri backend currently uses `rusqlite` v0.31 while gtk_ref uses v0.39 — but this is irrelevant for card_io (no database interaction). The card_io module is pure file I/O and JSON parsing.

The Tauri backend uses `chrono` v0.4 with `DateTime<Utc>` (not `NaiveDateTime` like gtk_ref). The `CharaCardV2Spec::to_domain()` must be adapted:

- gtk_ref: `chrono::Local::now().naive_local()`
- Tauri: `chrono::Utc::now()`

### R9: Security Considerations

1. **DOMPurify sanitization** (frontend): All markdown HTML passes through DOMPurify before `{@html}` rendering. Prevents XSS from AI-generated `<script>` tags, `javascript:` URLs, event handlers.

2. **Path traversal** (backend): The export command must validate that avatar paths are within the app data directory (existing pattern in `delete_avatar`).

3. **File size limit** (backend): 10MB max for card JSON payload (existing constant in card_io.rs).

4. **No V1 support** (D-08): The UI-SPEC error table includes "V1 format detected" error, but the reference card_io.rs actually DOES support V1 import. The CONTEXT.md defers V1 support decision. Plan should clarify: support V1 (reference already handles it) or reject it.

### R10: What Can Be Parallelized

The two workstreams are completely independent:

| Workstream         | Files                                                                                                                                                                                      | Dependency |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------- |
| Markdown rendering | `MarkdownRenderer.svelte`, `MessageBubble.svelte`, `app.css`, npm deps                                                                                                                     | None       |
| Card I/O           | `card_io.rs`, `commands/card_io.rs`, `domain/character.rs` additions, `CharacterCard.svelte`, `characters/+page.svelte`, `ImportErrorDialog.svelte`, `api.ts`, `app.svelte.ts`, Cargo deps | None       |

Both can be developed and tested independently. The plan should structure phases to exploit this.

---

## Open Questions for Planning

1. **V1 card import:** The CONTEXT.md error table lists "V1 format not supported" as an error condition, but the reference `card_io.rs` has full V1 → V2 migration. Should we support V1 import (reference code exists) or reject it (per UI-SPEC error table)? **Recommendation:** Support it — the code is already written and tested. The UI-SPEC error entry may be outdated.

2. **W++ normalization:** The reference `card_io.rs` includes sophisticated W++ extraction (personality blocks, dialogue examples, scenario metadata from description). Should this be included? **Recommendation:** Yes — port it. It significantly improves import quality for the many cards that stuff everything into `description`.

3. **Shiki preloading timing:** The UI-SPEC suggests preloading Shiki in `+layout.svelte` `$effect`. This means Shiki's WASM engine loads on every app start even if no code blocks are ever rendered. Is this acceptable? **Recommendation:** Yes — ~200KB lazy-loaded is acceptable for a desktop app. Preloading eliminates the first-code-block flash.

4. **Import command design:** Should `import_card` be one command (takes file path, returns CharacterCard) or two (one for file picking, one for parsing)? **Recommendation:** One command. The file picker is frontend-side (Tauri dialog plugin), the parsing is backend-side. `importCard(filePath: string): Promise<CharacterCard>` is clean.

5. **Export command design:** Should `export_card` handle the save dialog, or should the frontend open the save dialog and pass the chosen path? **Recommendation:** Frontend opens save dialog (via `tauri_plugin_dialog`), passes chosen path to backend `exportCard(characterId: string, savePath: string)`. This matches the existing avatar upload pattern (frontend picks file, backend processes it).

---

## Files to Read Before Planning

These files contain the implementation patterns that the plan must follow:

| File                                                  | Why                                            |
| ----------------------------------------------------- | ---------------------------------------------- |
| `gtk_ref/src/service/card_io.rs`                      | Complete reference implementation to port      |
| `gtk_ref/src/domain/character.rs`                     | `CharaCardV2Spec`, `CharacterV1` types to port |
| `src-tauri/src/domain/character.rs`                   | Existing domain model to extend                |
| `src-tauri/src/commands/avatars.rs`                   | File I/O command pattern to follow             |
| `src/lib/components/chat/MessageBubble.svelte`        | Current rendering to replace                   |
| `src/routes/characters/+page.svelte`                  | Import button wiring point                     |
| `src/lib/components/characters/CharacterCard.svelte`  | Export menu item addition                      |
| `.planning/phases/07-rich-output-cards/07-UI-SPEC.md` | Complete visual/interaction contract           |
| `.planning/phases/07-rich-output-cards/07-CONTEXT.md` | User decisions and constraints                 |

---

_Research completed: 2026-05-16_
