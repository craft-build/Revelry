# Roadmap: Revelry

## Overview

Revelry's v1 journey delivers a complete AI roleplay desktop application built in native Rust/GTK4. The path starts with a runnable app shell and provider connectivity, builds the character system that fuels all roleplay, delivers the core streaming chat experience with conversation branching, adds persona identity injection and template resolution, implements context engineering so long conversations stay coherent, and finishes with generation controls, regex scripting, and data portability. Each phase delivers a verifiable user capability — nothing is "infrastructure only."

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: App Shell & Provider Foundation** - Runnable GTK4/Adwaita app with adaptive layout, provider configuration, and secure API key storage
- [x] **Phase 2: Character System & Library** - CharacterCardV2 CRUD, import/export, and searchable character library
- [ ] **Phase 3: AI Chat & Streaming** - Streaming AI conversations with branching, message CRUD, and markdown rendering
- [x] **Phase 4: Personas & Template Resolution** - User personas with structured fields, assignment hierarchy, and full template variable resolution
- [x] **Phase 5: Context Engineering & Lorebook** - Token budgeting, layered prompt assembly, and keyword-activated lorebook system
- [ ] **Phase 6: Generation Controls, Regex & Data** - AI parameter presets, regex post-processing, and backup/restore

## Phase Details

### Phase 1: App Shell & Provider Foundation
**Goal**: Users can launch a native GNOME-style desktop app, configure AI providers, and verify connectivity — the foundation everything else builds on.
**Depends on**: Nothing (first phase)
**Requirements**: UISL-01, UISL-02, UISL-03, UISL-04, UISL-05, UISL-06, PROV-01, PROV-02, PROV-05
**Success Criteria** (what must be TRUE):
  1. User can launch the app and see the main window with Chat, Characters, Settings view switcher in the header bar
  2. User can resize the window and see the sidebar adapt between side-by-side and overlay layouts, and toggle the sidebar with F9
  3. User can configure at least one AI provider (e.g., OpenAI, Anthropic, or Ollama) and verify the connection works
  4. User's API keys are stored in the OS credential store, never visible in the application's database or config files
  5. User can access settings in a categorized AdwPreferencesWindow and navigate all actions via keyboard
**Plans**: 3 plans

Plans:
- [x] 01-01-PLAN.md — Project scaffold, domain types, and DB Worker foundation
- [x] 01-02-PLAN.md — GTK4/Adwaita app shell with adaptive layout and empty states
- [x] 01-03-PLAN.md — Provider config, keyring storage, and settings UI

**UI hint**: yes

### Phase 2: Character System & Library
**Goal**: Users can create, import, export, and browse AI character cards following the CharacterCardV2 spec — the core data that fuels all roleplay.
**Depends on**: Phase 1
**Requirements**: CHAR-01, CHAR-02, CHAR-03, CHAR-04, CHAR-05, CHAR-06, CHAR-07
**Success Criteria** (what must be TRUE):
  1. User can create a new character card with all CharacterCardV2 fields and see it appear in the library
  2. User can import a character card from PNG (embedded metadata) and JSON files and see it appear in the library
  3. User can export any character as both PNG (with embedded metadata) and JSON formats
  4. User can browse characters in a visual grid, filter by tags, and search by name or description with instant matching
**Plans**: 3 plans

Plans:
- [x] 02-01-PLAN.md — Character data layer: domain model, DB schema, CRUD, and FTS5 search
- [x] 02-02-PLAN.md — Import/export service and CharactersPanel with grid, search, filter, tabs
- [x] 02-03-PLAN.md — Character editor and AppModel wiring for end-to-end operations

**UI hint**: yes

### Phase 3: AI Chat & Streaming
**Goal**: Users can have streaming AI conversations with characters and manage conversation history with branching — the core product experience.
**Depends on**: Phase 1, Phase 2
**Requirements**: PROV-03, PROV-04, CONV-01, CONV-02, CONV-03, CONV-04, CONV-05, CONV-06, CONV-07, CONV-08, CONV-09, CONV-10
**Success Criteria** (what must be TRUE):
  1. User can start a new conversation with a character (seeing its first message) or resume an existing conversation from the list
  2. User can send messages and see AI responses stream token-by-token, with user messages right-aligned and character messages left-aligned
  3. User can cancel an in-progress AI generation and see the partial response preserved in conversation history
  4. User can edit a previous message or regenerate an AI response to create a branch, and navigate between branches with visual indicators
  5. User can delete any message from conversation history and see AI responses rendered with markdown formatting (bold, italic, headers, lists)
**Plans**: 3 plans

Plans:
- [ ] 03-01-PLAN.md — Conversation data layer, AI service, prompt assembly, and Pango markdown
- [ ] 03-02-PLAN.md — ChatPanel upgrade with streaming, message list, input bar, and sidebar
- [ ] 03-03-PLAN.md — Branching (edit, regenerate, navigate), message delete, and error handling

**UI hint**: yes

### Phase 4: Personas & Template Resolution
**Goal**: Users can create and assign personas that inject identity into AI prompts, with full template variable resolution — transforming chat into roleplay.
**Depends on**: Phase 2, Phase 3
**Requirements**: PERS-01, PERS-02, PERS-03, PERS-04, PERS-05, PERS-06, CHAR-08
**Success Criteria** (what must be TRUE):
  1. User can create, edit, and delete personas with structured fields (name, appearance, personality, background, role, relationships, speech style)
  2. User can assign a persona as global default, per-character default, or per-conversation override, and the correct persona is used in AI prompts
  3. Active persona data appears in the AI prompt as a structured block between character definition and conversation history
  4. Template variables {{user}}, {{char}}, and {{persona.*}} resolve correctly in character card fields at prompt assembly time
  **Plans**: 3 plans

Plans:
- [x] 04-01-PLAN.md — Persona domain model, database schema (M007), CRUD, template resolver, persona block builder
- [x] 04-02-PLAN.md — Persona editor wizard, persona library list, settings + character editor dropdowns
- [x] 04-03-PLAN.md — Chat header persona selector, assignment hierarchy resolution, prompt injection into AI stream

**UI hint**: yes

### Phase 5: Context Engineering & Lorebook
**Goal**: Token budgeting and lorebook system ensure long conversations stay coherent with proper context management — the intelligence layer that keeps characters in character.
**Depends on**: Phase 3, Phase 4
**Requirements**: CCTX-01, CCTX-02, CCTX-03, CCTX-04, CCTX-05, CCTX-06, CCTX-07
**Success Criteria** (what must be TRUE):
  1. User can see how much of the context window is consumed via a context utilization indicator
  2. AI prompts are assembled with the correct priority layers: sticky system+character → activated lorebooks → semi-sticky persona → truncated history
  3. User can add lorebook entries to a character with keywords, and those entries activate and inject into context when keywords appear in conversation
  4. Lorebook activation respects recursive depth limits and token budgets so world info doesn't consume the entire context
**Plans**: 3 plans

Plans:
- [x] 05-01-PLAN.md — Token counting service and lorebook data layer
- [x] 05-02-PLAN.md — Budget-aware prompt assembly and lorebook activation engine
- [x] 05-03-PLAN.md — Context utilization UI, lorebook editor, and provider settings

**UI hint**: yes

### Phase 6: Generation Controls, Regex & Data
**Goal**: Users can fine-tune AI generation parameters, apply regex post-processing to AI responses, and backup/restore all application data.
**Depends on**: Phase 3, Phase 5
**Requirements**: GENR-01, GENR-02, GENR-03, GENR-04, RGXO-01, RGXO-02, RGXO-03, RGXO-04, DATA-01, DATA-02, DATA-03
**Success Criteria** (what must be TRUE):
  1. User can create generation parameter presets (temperature, top_p, max_tokens, penalties, stop sequences) and assign them at character or conversation level
  2. Built-in starter presets (Creative, Balanced, Precise) are available for new users
  3. User can define regex find-and-replace scripts per character that modify AI responses before display, with ReDoS vulnerability validation
  4. User can test regex scripts on sample text and see before/after preview
  5. User can export all data as SQL dump or ZIP archive, and import from a previously exported backup
**Plans**: 3 plans

Plans:
- [ ] 06-01-PLAN.md — Generation preset and regex script data layer, DB schema M9, CRUD, built-in seeding, regex validation service
- [ ] 06-02-PLAN.md — Preset editor, Settings preset page, assignment hierarchy UI, AiService generation parameter injection
- [ ] 06-03-PLAN.md — Regex editor with test panel, regex execution pipeline, backup/restore SQL and ZIP

**UI hint**: yes

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. App Shell & Provider Foundation | 3/3 | Complete | 2026-04-29 |
| 2. Character System & Library | 3/3 | Complete | 2026-04-30 |
| 3. AI Chat & Streaming | 0/3 | Not started | - |
| 4. Personas & Template Resolution | 3/3 | Complete | 2026-05-07 |
| 5. Context Engineering & Lorebook | 3/3 | Complete | 2026-05-08 |
| 6. Generation Controls, Regex & Data | 0/3 | Not started | - |
