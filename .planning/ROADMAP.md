# Roadmap: Revelry

## Overview

Transform the existing SvelteKit frontend scaffold into a full cross-platform Tauri desktop app by porting the Rust backend from `gtk_ref/`, wiring it to the UI via Tauri IPC, and replacing all mock data with real persistence and AI streaming. Each phase delivers a working vertical capability — from launching the app in a native window, through real AI conversations, to full feature parity with the original design.

## Phases

**Phase Numbering:**

- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Native Desktop Shell** - App runs as a Tauri window with full SvelteKit routing ✅ 2026-05-11
- [x] **Phase 2: Persistent Data** - Characters, personas, and conversations load from SQLite instead of mock data ✅ 2026-05-11
- [x] **Phase 3: Editors** - Create and edit characters, personas, and lorebooks ✅ 2026-05-12
- [x] **Phase 4: Provider Setup** - Users configure AI providers with API keys stored in OS keychain ✅ 2026-05-13
- [x] **Phase 5: Live Conversations** - Real AI streaming replaces mock responses; users chat with AI through characters ✅ 2026-05-16
- [x] **Phase 6: Smart Prompts** - Budget-aware prompt assembly and real token counting ✅ 2026-05-16
- [x] **Phase 7: Rich Output & Cards** - Markdown rendering and TavernAI V2 card import/export ✅ 2026-05-18
- [ ] **Phase 8: Advanced Conversations** - Branching, navigation, and full-text search
- [ ] **Phase 9: Power Features** - Lorebook context injection and regex scripting
- [ ] **Phase 10: Theme & Polish** - Dark/light toggle and final refinements
- [ ] **Phase 11: Spec Update & Distribution** - Updated design doc and cross-platform builds

## Phase Details

### Phase 1: Native Desktop Shell

**Goal**: The app runs as a native desktop window on macOS, Linux, and Windows with full SvelteKit routing working
**Mode:** mvp
**Depends on**: Nothing (first phase)
**Requirements**: SHELL-01, SHELL-02, SHELL-03, SETT-03, CHAT-05
**Success Criteria** (what must be TRUE):

1. User can launch the app as a native desktop window
2. All SvelteKit routes (home, chat, characters, settings) work inside the Tauri webview
3. App runs correctly in both `tauri dev` and production `tauri build`
4. Keyboard shortcuts (F9 sidebar, Enter/Ctrl+Enter send) still work
   **Plans**: 4 plans in 2 waves

Plans:

**Wave 1** _(autonomous — project scaffold + security layer)_

- [ ] 01-01: Initialize Tauri v2 + Switch SvelteKit to adapter-static
- [ ] 01-02: Configure Tauri v2 capabilities and permissions _(blocked on 01-01)_

**Wave 2** _(manual — dev and production verification)_

- [ ] 01-03: Verify dev mode — routes, keyboard shortcuts, drag region _(blocked on 01-01, 01-02)_
- [ ] 01-04: Verify production build — `tauri build`, DevTools, CSP _(blocked on 01-01, 01-02)_

**Cross-cutting constraints:**

- Port alignment: Vite `server.port` (1420) must match Tauri `devUrl` — single point of failure across 01-01
- Adapter-prerender contract: `svelte.config.js` adapter + `+layout.ts` prerender must stay consistent
- Plugin registration triangle: `Cargo.toml` + `lib.rs` + `package.json` must all include the same plugins
  **UI hint**: yes

### Phase 2: Persistent Data

**Goal**: Characters, personas, and conversations load from and save to a real SQLite database instead of mock data
**Mode:** mvp
**Depends on**: Phase 1
**Requirements**: DB-01, DB-02, DB-03, CHAR-01, PERS-01, PERS-05, CONV-01, CONV-02, CONV-03, CONV-04, CONV-05
**Success Criteria** (what must be TRUE):

1. App initializes SQLite database on first launch in the platform-appropriate directory
2. User can view characters and personas loaded from the database (not mock data)
3. User can create and view conversations that persist across app restarts
4. Database operations never freeze the UI
   **Plans**: 4 plans in 4 waves

Plans:

**Wave 1** _(autonomous — domain models + schema + DbWorker)_

- [x] 02-01: Port domain models from gtk_ref/ and define SQLite schema

**Wave 2** _(autonomous — CRUD functions + row mappers, blocked on Wave 1)_

- [x] 02-02: Implement DbService with background-thread worker and migrations _(blocked on 02-01)_

**Wave 3** _(autonomous — Tauri IPC commands, blocked on Wave 2)_

- [x] 02-03: Create Tauri commands for character, persona, and conversation CRUD _(blocked on 02-02)_

**Wave 4** _(autonomous — frontend wiring, blocked on Wave 3)_

- [x] 02-04: Replace mock data in SvelteKit frontend with invoke() calls _(blocked on 02-03)_

**Cross-cutting constraints:**

- DbCommand enum: 02-01 defines the enum, 02-02 adds match arms, 02-03 uses them in Tauri commands — must stay in sync
- Tauri v2 command naming: Rust function names are snake_case, invoke() must use matching snake_case strings (no camelCase conversion)
- Type alignment: Rust serde output must match TypeScript types — 02-04 Task 1 aligns, all subsequent type changes must maintain consistency
  **UI hint**: yes

### Phase 3: Editors

**Goal**: Users can create and edit characters, personas, and lorebooks through guided editor UIs
**Mode:** mvp
**Depends on**: Phase 2
**Requirements**: CHAR-02, CHAR-03, CHAR-04, CHAR-05, PERS-02, PERS-03, PERS-04, LORB-06
**Success Criteria** (what must be TRUE):

1. User can create a new character with all fields (name, description, personality, scenario, greeting, dialogues) and avatar
2. User can edit and delete existing characters
3. User can create, edit, and delete personas
4. User can test lorebook keyword activation in the editor
   **Plans**: 4 plans in 3 waves

Plans:

**Wave 1** _(autonomous — backend + UI primitives + character editor)_

- [x] 03-01: Backend foundation (lorebook domain + migration + Tauri commands) + UI primitives (Dialog, Textarea, Select, KebabMenu, StepperHeader, AvatarUpload) + Character Editor wizard + DeleteConfirmationDialog + wiring

**Wave 2** _(autonomous — persona + lorebook editors, blocked on Wave 1)_

- [x] 03-02: Persona Editor dialog + PersonaCard kebab menu + persona avatar*path wiring *(blocked on 03-01)\_
- [x] 03-03: Lorebook Editor with keyword testing + LorebookCard + Lorebooks tab _(blocked on 03-01)_

**Wave 3** _(autonomous — integration + polish, blocked on Waves 1 and 2)_

- [x] 03-04: Avatar URL resolution + end-to-end integration + polish _(blocked on 03-01, 03-02, 03-03)_

**Cross-cutting constraints:**

- AppState CRUD API: 03-01 defines all actions (createCharacter, updateCharacter, deleteCharacter, createPersona, updatePersona, deletePersona, createLorebook, updateLorebook, deleteLorebook, uploadAvatar, lorebooks state) — 03-02 and 03-03 only consume, not redefine
- Shared UI primitives: Dialog, Textarea, Select, KebabMenu, StepperHeader, AvatarUpload, DeleteConfirmationDialog all created in 03-01 — 03-02 and 03-03 must import from `src/lib/components/ui/`
- Schema migration: 03-01 adds migration #2 (lorebooks, lorebook_entries refactor, character_lorebooks junction, persona avatar_path) — all subsequent plans assume this schema exists
- Type alignment: Rust serde output must match TypeScript types — 03-01 adds Lorebook, LorebookEntry, KeywordTestResult types; all subsequent plans must maintain consistency
- `deletingEntity` type: 03-01 defines as `'character' | 'persona'`, 03-03 extends to include `'lorebook'` — must stay in sync across plans
  **UI hint**: yes

### Phase 4: Provider Setup

**Goal**: Users can configure AI providers with API keys securely stored in the OS keychain
**Mode:** mvp
**Depends on**: Phase 3
**Requirements**: PROV-01, PROV-02, PROV-03, PROV-04, PROV-05, PROV-06
**Success Criteria** (what must be TRUE):

1. User can add, edit, and delete AI providers in the settings page
2. API keys are stored in the OS-native keychain and never written to disk
3. User can select a provider and model for a conversation
4. User can configure generation parameters (temperature, top_p, max_tokens)
5. App handles keyring unavailability on Linux gracefully
   **Plans**: 2 plans in 2 waves

Plans:

**Wave 1** _(autonomous — backend keyring + provider CRUD pipeline)_

- [x] 04-01: KeyringStore + ProviderConfig domain + migration + CRUD + Tauri commands

**Wave 2** _(autonomous — frontend UI + settings wiring, blocked on Wave 1)_

- [x] 04-02: ProviderEditorDialog + ModelEntryRow + KeyringUnavailableBanner + settings page wiring _(blocked on 04-01)_

**Cross-cutting constraints:**

- ProviderType serde: Rust uses `#[serde(rename_all = "snake_case")]` to produce "openai" etc. matching TypeScript ProviderType union values — any change to either side breaks round-trip
- Keyring v4 init: `keyring::use_native_store(true)` MUST be called before any `Entry::new()` — Plan 01 Task 1, never moved or removed
- API key security: has_api_key returns boolean only; never expose key in IPC responses — enforced across both plans
- Type alignment: Provider interface in TypeScript (models array, has_api_key, model_name) must match Rust ProviderConfig serde output
  **UI hint**: yes

### Phase 5: Live Conversations

**Goal**: Users have real streaming AI conversations through characters, replacing mock responses
**Mode:** mvp
**Depends on**: Phase 4
**Requirements**: CHAT-01, CHAT-02, CHAT-03, CHAT-04
**Success Criteria** (what must be TRUE):

1. User can send a message and receive a streaming token-by-token AI response
2. User can stop generation mid-stream
3. User can regenerate the last AI response with the same or different provider
4. User can edit a sent message and regenerate from that point
   **Plans**: 5 plans in 5 waves

Plans:

**Wave 1** _(autonomous — AiService port + rig-core 0.37)_

- [x] 05-01: Port AiService from gtk_ref/ with rig-core 0.37 streaming + StreamChunk + CancelState

**Wave 2** _(autonomous — Tauri Channel commands + DB extensions, blocked on Wave 1)_

- [x] 05-02: Implement Tauri Channel-based streaming commands (start_chat, stop_generation, regenerate, edit_and_resend) + DB extensions

**Wave 3** _(autonomous — frontend wiring, blocked on Wave 2)_

- [x] 05-03: Replace mock streaming with real AI invoke in chat view + wire stop button + disable input

**Wave 4** _(autonomous — error rendering + regenerate, blocked on Wave 3)_

- [x] 05-04: Inline error rendering + stop reason badges + regenerate hover action in MessageBubble

**Wave 5** _(manual — edit mode + end-to-end verification, blocked on Wave 4)_

- [x] 05-05: Implement edit/resend functionality with inline edit mode + human verification

**Cross-cutting constraints:**

- rig-core version: must use 0.37 (NOT 0.36 from gtk_ref) — MultiTurnStreamItem::Text replaces StreamAssistantItem (RESEARCH Pitfall 1)
- StreamChunk serde: `tag = "event", content = "data"` must match TypeScript discriminated union — both sides must stay in sync
- CancelState: single CancellationToken in managed state — only one active stream at a time; start_chat cancels any existing stream first (RESEARCH Pitfall 4)
- API key security: keys loaded via KeyringStore::load_key() inside spawn_blocking — never passed through Channel or IPC responses
- Type alignment: Rust StreamChunk serde camelCase output must match TS StreamChunk type (Phase 2 D-08)
  **UI hint**: yes

### Phase 6: Smart Prompts

**Goal**: AI conversations use budget-aware prompt assembly with real token counting so responses are contextually appropriate
**Mode:** mvp
**Depends on**: Phase 5
**Requirements**: PRMP-01, PRMP-02, PRMP-03
**Success Criteria** (what must be TRUE):

1. System prompt is assembled from character card, active persona, and conversation history
2. Prompt stays within the model's context window budget
3. User sees accurate context utilization (tokens used / limit) in the message input
   **Plans**: TBD

Plans:

- [x] 06-01: Port PromptEngine from gtk_ref/ with budget-aware assembly
- [x] 06-02: Integrate tiktoken-rs for real token counting
- [x] 06-03: Wire token count display to real data in message input
- [ ] 06-04: Integrate prompt engine into the streaming chat flow

### Phase 7: Rich Output & Cards

**Goal**: AI responses render as rich markdown, and users can import/export character cards
**Mode:** mvp
**Depends on**: Phase 6
**Requirements**: CHAT-06, CHAR-06, CHAR-07
**Success Criteria** (what must be TRUE):

1. AI responses display with formatted markdown (bold, italic, code blocks, lists, headers)
2. User can import a character from a TavernAI V2 PNG file
3. User can export a character as a TavernAI V2 PNG file that loads in other apps
   **Plans**: TBD

Plans:

**Wave 1** _(autonomous — markdown + card I/O backend, parallel)_

- [x] 07-01: Frontend Markdown Rendering Pipeline (marked + DOMPurify + Shiki) ✓ 2026-05-18
- [x] 07-02: Port Card I/O Backend from gtk_ref/ (PNG tEXt chunk read/write)

**Wave 2** _(autonomous — Shiki preload + frontend card UI, blocked on Wave 1)_

- [x] 07-03: Shiki Preloading in Root Layout ✓ 2026-05-18 _(was blocked on 07-01)_
- [x] 07-04: Wire Card Import/Export to Frontend UI ✓ 2026-05-18 _(was blocked on 07-02)_
      **UI hint**: yes

### Phase 8: Advanced Conversations

**Goal**: Users can branch conversations and search across all conversations
**Mode:** mvp
**Depends on**: Phase 7
**Requirements**: CONV-06, CONV-07, CONV-08
**Success Criteria** (what must be TRUE):

1. User can fork a conversation from any message to create a parallel branch
2. User can navigate between branches in a conversation
3. User can search across all conversations and messages and get relevant results
   **Plans**: TBD

Plans:

- [ ] 08-01: Schema + Domain + DB Functions (Backend Foundation)
- [ ] 08-02: DbCommand Wiring + Tauri Commands + Frontend Types + API (IPC Layer)
- [ ] 08-03: Frontend State + Branch-Aware Loading (State Layer)
- [ ] 08-04: UI Components (Visual Layer)
      **UI hint**: yes

### Phase 9: Power Features

**Goal**: Lorebook context injection and regex scripting give users advanced control over AI behavior
**Mode:** mvp
**Depends on**: Phase 8
**Requirements**: LORB-01, LORB-02, LORB-03, LORB-04, LORB-05, PRMP-04, PRMP-05, REGX-01, REGX-02, REGX-03, REGX-04, REGX-05
**Success Criteria** (what must be TRUE):

1. Lorebook entries inject context into prompts when keywords match
2. Lorebook injection is recursive with depth limiting
3. User can assign lorebooks to characters and conversations with budget controls
4. Regex scripts apply ordered transformations to AI output
5. Invalid regex patterns are rejected on save
   **Plans**: TBD

Plans:

- [ ] 09-01: Port LorebookService from gtk_ref/ with keyword matching and recursive insertion
- [ ] 09-02: Integrate lorebook into PromptEngine
- [ ] 09-03: Port RegexService from gtk_ref/ with validation
- [ ] 09-04: Integrate regex scripts into AI response pipeline
- [ ] 09-05: Wire lorebook and regex to editor UIs and conversation flow

### Phase 10: Theme & Polish

**Goal**: Users can choose light or dark theme, and the app feels polished
**Mode:** mvp
**Depends on**: Phase 9
**Requirements**: SETT-01, SETT-02
**Success Criteria** (what must be TRUE):

1. User can toggle between dark and light themes
2. App respects system appearance preference with manual override
   **Plans**: 2 plans in 2 waves

Plans:

**Wave 1** _(autonomous — backend: keyring + domain + migration + CRUD + commands)_

- [x] 04-01: KeyringStore + ProviderConfig domain + migration #3 + CRUD + Tauri commands

**Wave 2** _(autonomous — frontend: editor dialog + settings wiring + ChatHeader selector, blocked on Wave 1)_

- [ ] 04-02: ProviderEditorDialog + ModelEntryRow + KeyringUnavailableBanner + settings wiring + ChatHeader provider/model selector

**Cross-cutting constraints:**

- API key security: keys stored only in OS keychain via KeyringStore; never in SQLite, IPC responses, or logs
- keyring v4 split: `keyring = "4.0"` + `keyring-core = "1.0"` — both crates required; store init before use
- ProviderType serde: Rust uses `#[serde(rename_all = "snake_case")]` to match TypeScript values
- Conversation model_name write path: update_conversation_provider function persists both provider_id and model_name together
  **UI hint**: yes

### Phase 11: Spec Update & Distribution

**Goal**: Design spec reflects the SvelteKit/Tauri architecture, and the app builds for all platforms
**Mode:** mvp
**Depends on**: Phase 10
**Requirements**: DOC-01, DIST-01, DIST-02, DIST-03
**Success Criteria** (what must be TRUE):

1. Design specification has all GTK4/GNOME references replaced with SvelteKit/Tauri equivalents
2. App builds and runs on macOS
3. App builds and runs on Linux
4. App builds and runs on Windows
   **Plans**: TBD

Plans:

- [ ] 11-01: Update design specification (GTK4/GNOME → SvelteKit/Tauri)
- [ ] 11-02: Cross-platform build testing and fixes
- [ ] 11-03: Distribution packaging (DMG/AppImage/MSI)

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10 → 11

| Phase                          | Plans Complete | Status      | Completed  |
| ------------------------------ | -------------- | ----------- | ---------- |
| 1. Native Desktop Shell        | 4/4            | Complete    | 2026-05-11 |
| 2. Persistent Data             | 4/4            | Complete    | 2026-05-11 |
| 3. Editors                     | 4/4            | Complete    | 2026-05-12 |
| 4. Provider Setup              | 2/2            | Complete    | 2026-05-13 |
| 5. Live Conversations          | 5/5            | Complete    | 2026-05-16 |
| 6. Smart Prompts               | 3/4            | In Progress | 2026-05-16 |
| 7. Rich Output & Cards         | 4/4            | Complete    | 2026-05-18 |
| 8. Advanced Conversations      | 0/4            | Not started | -          |
| 9. Power Features              | 0/5            | Not started | -          |
| 10. Theme & Polish             | 0/2            | Not started | -          |
| 11. Spec Update & Distribution | 0/3            | Not started | -          |
