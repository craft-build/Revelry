# Requirements: Revelry

**Defined:** 2026-05-10
**Core Value:** Users can have immersive, streaming AI conversations with custom characters through personas — and the whole thing runs locally as a native desktop app.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Shell

- [x] **SHELL-01**: User can launch the app as a native desktop window (macOS, Linux, Windows)
- [x] **SHELL-02**: App runs the existing SvelteKit SPA in a Tauri webview with full routing
- [x] **SHELL-03**: App builds and runs correctly in both dev mode (`tauri dev`) and production mode (`tauri build`)

### Database

- [x] **DB-01**: App initializes SQLite database in platform-appropriate data directory on first launch
- [x] **DB-02**: App runs schema migrations automatically on startup
- [x] **DB-03**: All database operations execute off the main thread to prevent UI freezes

### Characters

- [x] **CHAR-01**: User can view a library of characters with search and filtering
- [x] **CHAR-02**: User can create a new character with name, description, personality, scenario, greeting message, and example dialogues
- [x] **CHAR-03**: User can edit an existing character's fields
- [x] **CHAR-04**: User can delete a character
- [x] **CHAR-05**: User can assign an avatar to a character (upload image or default initials)
- [ ] **CHAR-06**: User can import a character card from a TavernAI V2 PNG file
- [ ] **CHAR-07**: User can export a character card as a TavernAI V2 PNG file

### Personas

- [x] **PERS-01**: User can view a library of personas with search and filtering
- [x] **PERS-02**: User can create a new persona with name, description, and avatar
- [x] **PERS-03**: User can edit an existing persona's fields
- [x] **PERS-04**: User can delete a persona
- [x] **PERS-05**: User can switch the active persona for a conversation via the persona switcher

### Conversations

- [x] **CONV-01**: User can create a new conversation with a character
- [x] **CONV-02**: User can view a list of conversations with character info, last message preview, and relative timestamps
- [x] **CONV-03**: User can search conversations by text
- [x] **CONV-04**: User can delete a conversation
- [x] **CONV-05**: User can view conversation messages with auto-scroll
- [ ] **CONV-06**: User can fork a conversation from any message to create a parallel branch
- [ ] **CONV-07**: User can navigate between conversation branches
- [ ] **CONV-08**: User can search across all conversations and messages with full-text search

### Chat

- [ ] **CHAT-01**: User can send a message and receive a streaming token-by-token AI response with typewriter animation
- [ ] **CHAT-02**: User can stop generation mid-stream
- [ ] **CHAT-03**: User can regenerate the last AI response
- [ ] **CHAT-04**: User can edit a sent message and resend from that point
- [x] **CHAT-05**: User can send messages via Enter (or Ctrl+Enter based on settings)
- [ ] **CHAT-06**: AI responses are rendered as formatted markdown (bold, italic, code blocks, lists, headers)

### Prompt Engineering

- [ ] **PRMP-01**: System prompt is assembled from character card, active persona, and conversation history
- [ ] **PRMP-02**: Prompt assembly respects context window budget (token limit) for the selected model
- [ ] **PRMP-03**: User can see current context utilization (tokens used / token limit) in the message input area
- [ ] **PRMP-04**: Lorebook entries are injected into the prompt when their keywords match the conversation
- [ ] **PRMP-05**: Lorebook injection is recursive (entries can reference other entries) with depth limiting

### Providers

- [x] **PROV-01**: User can add an AI provider with name, type (OpenAI, Anthropic, custom OpenAI-compatible, etc.), and API key
- [x] **PROV-02**: User can edit provider configuration
- [x] **PROV-03**: User can delete a provider
- [x] **PROV-04**: API keys are stored in the OS-native keychain (never on disk)
- [x] **PROV-05**: User can select a provider and model for a conversation
- [x] **PROV-06**: User can configure generation parameters (temperature, top_p, max_tokens, frequency_penalty, presence_penalty)

### Lorebook

- [ ] **LORB-01**: User can create lorebook entries with keywords, content, and activation strategy (constant, keyword-triggered)
- [ ] **LORB-02**: User can edit lorebook entries
- [ ] **LORB-03**: User can delete lorebook entries
- [ ] **LORB-04**: User can assign lorebooks to characters and conversations
- [ ] **LORB-05**: User can set ordering and token budget allocation for lorebook entries
- [ ] **LORB-06**: User can test keyword activation by entering sample text in the lorebook editor

### Regex Scripts

- [ ] **REGX-01**: User can create regex scripts with find/replace patterns and enable/disable toggles
- [ ] **REGX-02**: User can edit regex scripts
- [ ] **REGX-03**: User can delete regex scripts
- [ ] **REGX-04**: Regex scripts are applied in order to AI output before display
- [ ] **REGX-05**: Invalid regex patterns are rejected on save with a clear error message

### Settings

- [ ] **SETT-01**: User can toggle between dark and light themes
- [ ] **SETT-02**: App respects system appearance preference with manual override option
- [x] **SETT-03**: User can view and modify keyboard shortcuts

### Design Document

- [ ] **DOC-01**: Design specification (docs/) is updated to replace GTK4/GNOME references with SvelteKit/Tauri equivalents

### Distribution

- [ ] **DIST-01**: App builds and distributes for macOS
- [ ] **DIST-02**: App builds and distributes for Linux
- [ ] **DIST-03**: App builds and distributes for Windows

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Autoupdate

- **AUTO-01**: App checks for updates and notifies user
- **AUTO-02**: App can update itself (or prompt user to download update)

### System Tray

- **TRAY-01**: App can minimize to system tray
- **TRAY-02**: App shows status in system tray icon

### Import/Export

- **BKUP-01**: User can export all app data (conversations, characters, personas, settings) as a backup file
- **BKUP-02**: User can import a backup file to restore app data

## Out of Scope

| Feature                        | Reason                                                               |
| ------------------------------ | -------------------------------------------------------------------- |
| Vault mode (encryption)        | Was speculative in original spec; not a real requirement             |
| Cloud sync / multi-device      | Conflicts with local-first privacy proposition; enormous complexity  |
| Mobile app                     | Desktop-only; Tauri targets desktop; mobile is different UX paradigm |
| Web-hosted deployment          | Tauri native distribution only; no server-side rendering             |
| Built-in character marketplace | Requires moderation, hosting, payments, legal compliance             |
| Voice / TTS / STT              | Different domain; out of scope for text-focused roleplay             |
| Image generation               | Different API integration, cost model, and UI patterns               |
| OAuth / social login           | Single-user local app; no server auth needed                         |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase    | Status  |
| ----------- | -------- | ------- |
| SHELL-01    | Phase 1  | Done    |
| SHELL-02    | Phase 1  | Done    |
| SHELL-03    | Phase 1  | Done    |
| DB-01       | Phase 2  | Done    |
| DB-02       | Phase 2  | Done    |
| DB-03       | Phase 2  | Done    |
| CHAR-01     | Phase 2  | Done    |
| CHAR-02     | Phase 3  | Done    |
| CHAR-03     | Phase 3  | Done    |
| CHAR-04     | Phase 3  | Done    |
| CHAR-05     | Phase 3  | Done    |
| CHAR-06     | Phase 7  | Pending |
| CHAR-07     | Phase 7  | Pending |
| PERS-01     | Phase 2  | Done    |
| PERS-02     | Phase 3  | Done    |
| PERS-03     | Phase 3  | Done    |
| PERS-04     | Phase 3  | Done    |
| PERS-05     | Phase 2  | Done    |
| CONV-01     | Phase 2  | Done    |
| CONV-02     | Phase 2  | Done    |
| CONV-03     | Phase 2  | Done    |
| CONV-04     | Phase 2  | Done    |
| CONV-05     | Phase 2  | Done    |
| CONV-06     | Phase 8  | Pending |
| CONV-07     | Phase 8  | Pending |
| CONV-08     | Phase 8  | Pending |
| CHAT-01     | Phase 5  | Pending |
| CHAT-02     | Phase 5  | Pending |
| CHAT-03     | Phase 5  | Pending |
| CHAT-04     | Phase 5  | Pending |
| CHAT-05     | Phase 1  | Done    |
| CHAT-06     | Phase 7  | Pending |
| PRMP-01     | Phase 6  | Pending |
| PRMP-02     | Phase 6  | Pending |
| PRMP-03     | Phase 6  | Pending |
| PRMP-04     | Phase 9  | Pending |
| PRMP-05     | Phase 9  | Pending |
| PROV-01     | Phase 4  | Complete |
| PROV-02     | Phase 4  | Complete |
| PROV-03     | Phase 4  | Complete |
| PROV-04     | Phase 4  | Complete |
| PROV-05     | Phase 4  | Complete |
| PROV-06     | Phase 4  | Complete |
| LORB-01     | Phase 9  | Pending |
| LORB-02     | Phase 9  | Pending |
| LORB-03     | Phase 9  | Pending |
| LORB-04     | Phase 9  | Pending |
| LORB-05     | Phase 9  | Pending |
| LORB-06     | Phase 3  | Done    |
| REGX-01     | Phase 9  | Pending |
| REGX-02     | Phase 9  | Pending |
| REGX-03     | Phase 9  | Pending |
| REGX-04     | Phase 9  | Pending |
| REGX-05     | Phase 9  | Pending |
| SETT-01     | Phase 10 | Pending |
| SETT-02     | Phase 10 | Pending |
| SETT-03     | Phase 1  | Done    |
| DOC-01      | Phase 11 | Pending |
| DIST-01     | Phase 11 | Pending |
| DIST-02     | Phase 11 | Pending |
| DIST-03     | Phase 11 | Pending |

**Coverage:**

- v1 requirements: 60 total
- Mapped to phases: 60
- Unmapped: 0 ✓

---

_Requirements defined: 2026-05-10_
_Last updated: 2026-05-12 after Phase 3 planning — traceability alignment_
