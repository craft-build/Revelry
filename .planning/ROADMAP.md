# Roadmap: Revelry

## Overview

Transform the existing SvelteKit frontend scaffold into a full cross-platform Tauri desktop app by porting the Rust backend from `gtk_ref/`, wiring it to the UI via Tauri IPC, and replacing all mock data with real persistence and AI streaming. Each phase delivers a working vertical capability — from launching the app in a native window, through real AI conversations, to full feature parity with the original design.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Native Desktop Shell** - App runs as a Tauri window with full SvelteKit routing
- [ ] **Phase 2: Persistent Data** - Characters, personas, and conversations load from SQLite instead of mock data
- [ ] **Phase 3: Provider Setup** - Users configure AI providers with API keys stored in OS keychain
- [ ] **Phase 4: Live Conversations** - Real AI streaming replaces mock responses; users chat with AI through characters
- [ ] **Phase 5: Smart Prompts** - Budget-aware prompt assembly and real token counting
- [ ] **Phase 6: Rich Output & Cards** - Markdown rendering and TavernAI V2 card import/export
- [ ] **Phase 7: Editors** - Create and edit characters, personas, and lorebooks
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

**Wave 1** *(autonomous — project scaffold + security layer)*
- [ ] 01-01: Initialize Tauri v2 + Switch SvelteKit to adapter-static
- [ ] 01-02: Configure Tauri v2 capabilities and permissions *(blocked on 01-01)*

**Wave 2** *(manual — dev and production verification)*
- [ ] 01-03: Verify dev mode — routes, keyboard shortcuts, drag region *(blocked on 01-01, 01-02)*
- [ ] 01-04: Verify production build — `tauri build`, DevTools, CSP *(blocked on 01-01, 01-02)*

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
**Plans**: TBD

Plans:
- [ ] 02-01: Port domain models from gtk_ref/ and define SQLite schema
- [ ] 02-02: Implement DbService with background-thread worker and migrations
- [ ] 02-03: Create Tauri commands for character, persona, and conversation CRUD
- [ ] 02-04: Replace mock data in SvelteKit frontend with invoke() calls
**UI hint**: yes

### Phase 3: Provider Setup
**Goal**: Users can configure AI providers with API keys securely stored in the OS keychain
**Mode:** mvp
**Depends on**: Phase 2
**Requirements**: PROV-01, PROV-02, PROV-03, PROV-04, PROV-05, PROV-06
**Success Criteria** (what must be TRUE):
  1. User can add, edit, and delete AI providers in the settings page
  2. API keys are stored in the OS-native keychain and never written to disk
  3. User can select a provider and model for a conversation
  4. User can configure generation parameters (temperature, top_p, max_tokens)
  5. App handles keyring unavailability on Linux gracefully
**Plans**: TBD

Plans:
- [ ] 03-01: Implement KeyringStore with OS-native keychain integration
- [ ] 03-02: Port provider domain model and DB persistence
- [ ] 03-03: Create Tauri commands for provider CRUD
- [ ] 03-04: Wire settings page to real provider backend with keyring
**UI hint**: yes

### Phase 4: Live Conversations
**Goal**: Users have real streaming AI conversations through characters, replacing mock responses
**Mode:** mvp
**Depends on**: Phase 3
**Requirements**: CHAT-01, CHAT-02, CHAT-03, CHAT-04
**Success Criteria** (what must be TRUE):
  1. User can send a message and receive a streaming token-by-token AI response
  2. User can stop generation mid-stream
  3. User can regenerate the last AI response with the same or different provider
  4. User can edit a sent message and regenerate from that point
**Plans**: TBD

Plans:
- [ ] 04-01: Port AiService from gtk_ref/ with rig-core streaming
- [ ] 04-02: Implement Tauri Channel-based streaming (Rust → SvelteKit)
- [ ] 04-03: Replace mock streaming with real AI invoke in chat view
- [ ] 04-04: Implement stop generation and regenerate commands
- [ ] 04-05: Implement edit/resend functionality
**UI hint**: yes

### Phase 5: Smart Prompts
**Goal**: AI conversations use budget-aware prompt assembly with real token counting so responses are contextually appropriate
**Mode:** mvp
**Depends on**: Phase 4
**Requirements**: PRMP-01, PRMP-02, PRMP-03
**Success Criteria** (what must be TRUE):
  1. System prompt is assembled from character card, active persona, and conversation history
  2. Prompt stays within the model's context window budget
  3. User sees accurate context utilization (tokens used / limit) in the message input
**Plans**: TBD

Plans:
- [ ] 05-01: Port PromptEngine from gtk_ref/ with budget-aware assembly
- [ ] 05-02: Integrate tiktoken-rs for real token counting
- [ ] 05-03: Wire token count display to real data in message input
- [ ] 05-04: Integrate prompt engine into the streaming chat flow

### Phase 6: Rich Output & Cards
**Goal**: AI responses render as rich markdown, and users can import/export character cards
**Mode:** mvp
**Depends on**: Phase 5
**Requirements**: CHAT-06, CHAR-06, CHAR-07
**Success Criteria** (what must be TRUE):
  1. AI responses display with formatted markdown (bold, italic, code blocks, lists, headers)
  2. User can import a character from a TavernAI V2 PNG file
  3. User can export a character as a TavernAI V2 PNG file that loads in other apps
**Plans**: TBD

Plans:
- [ ] 06-01: Port MarkdownService from gtk_ref/ (pulldown-cmark)
- [ ] 06-02: Replace naive rendering with real markdown in MessageBubble
- [ ] 06-03: Port CardIO service from gtk_ref/ (PNG tEXt chunk read/write)
- [ ] 06-04: Wire card import/export to character library UI
**UI hint**: yes

### Phase 7: Editors
**Goal**: Users can create and edit characters, personas, and lorebooks through guided editor UIs
**Mode:** mvp
**Depends on**: Phase 6
**Requirements**: CHAR-02, CHAR-03, CHAR-04, CHAR-05, PERS-02, PERS-03, PERS-04, LORB-06
**Success Criteria** (what must be TRUE):
  1. User can create a new character with all fields (name, description, personality, scenario, greeting, dialogues) and avatar
  2. User can edit and delete existing characters
  3. User can create, edit, and delete personas
  4. User can test lorebook keyword activation in the editor
**Plans**: TBD

Plans:
- [ ] 07-01: Build character editor wizard (Bits.ui Dialog, multi-step form)
- [ ] 07-02: Build persona editor (Bits.ui Dialog)
- [ ] 07-03: Build lorebook editor with keyword testing
- [ ] 07-04: Wire editors to Tauri CRUD commands
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
- [ ] 08-01: Extend message schema for tree structure (parent/child)
- [ ] 08-02: Implement conversation forking and branch navigation
- [ ] 08-03: Implement FTS5 full-text search across conversations
- [ ] 08-04: Build branch navigation and search UI
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
**Plans**: TBD

Plans:
- [ ] 10-01: Implement theme toggle with CSS variables and system preference detection
- [ ] 10-02: Light theme token definitions and testing
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

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Native Desktop Shell | 0/4 | Planned | - |
| 2. Persistent Data | 0/4 | Not started | - |
| 3. Provider Setup | 0/4 | Not started | - |
| 4. Live Conversations | 0/5 | Not started | - |
| 5. Smart Prompts | 0/4 | Not started | - |
| 6. Rich Output & Cards | 0/4 | Not started | - |
| 7. Editors | 0/4 | Not started | - |
| 8. Advanced Conversations | 0/4 | Not started | - |
| 9. Power Features | 0/5 | Not started | - |
| 10. Theme & Polish | 0/2 | Not started | - |
| 11. Spec Update & Distribution | 0/3 | Not started | - |
