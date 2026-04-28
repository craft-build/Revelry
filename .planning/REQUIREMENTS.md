# Requirements: Persona

**Defined:** 2025-04-27
**Core Value:** Users can engage in uncensored, private AI character roleplay with full data sovereignty

## v1 Requirements

### Core Infrastructure (INFRA)

- [ ] **INFRA-01**: Tauri 2.0 project scaffold with React 18 + TypeScript 5
- [ ] **INFRA-02**: SQLite database schema with migrations (characters, conversations, messages, providers)
- [ ] **INFRA-03**: Type sharing between Rust backend and TypeScript frontend
- [ ] **INFRA-04**: OS keychain integration for secure API key storage
- [ ] **INFRA-05**: Tauri capabilities configuration for v2 security model
- [ ] **INFRA-06**: Error handling framework with structured error types

### Character Management (CHAR)

- [ ] **CHAR-01**: Character card creation with basic fields (name, description, personality)
- [ ] **CHAR-02**: Character card editing with all CharacterCardV2 fields
- [ ] **CHAR-03**: PNG import with embedded V2 metadata extraction
- [ ] **CHAR-04**: JSON import for V1/V2 card formats
- [ ] **CHAR-05**: PNG export with embedded metadata
- [ ] **CHAR-06**: JSON export for V2 format
- [ ] **CHAR-07**: Character grid view with avatar, name, and tags
- [ ] **CHAR-08**: Character search via SQLite FTS5
- [ ] **CHAR-09**: Character deletion with confirmation

### AI Provider Integration (PROV)

- [ ] **PROV-01**: Single OpenAI-compatible provider configuration
- [ ] **PROV-02**: API key storage in OS keychain (never in database)
- [ ] **PROV-03**: Base URL configuration for custom endpoints
- [ ] **PROV-04**: Default model selection per provider
- [ ] **PROV-05**: Connection testing with minimal completion request
- [ ] **PROV-06**: Streaming SSE response handling via Tauri Channels
- [ ] **PROV-07**: Generation parameters (temperature, top_p, max_tokens)
- [ ] **PROV-08**: Error handling for rate limits, auth failures, timeouts

### Chat System (CHAT)

- [ ] **CHAT-01**: Message display with user/character attribution
- [ ] **CHAT-02**: Streaming token display with real-time updates
- [ ] **CHAT-03**: Multi-line message input with send on Ctrl+Enter
- [ ] **CHAT-04**: Message editing for user messages
- [ ] **CHAT-05**: Message regeneration for character responses
- [ ] **CHAT-06**: Message deletion with cascade
- [ ] **CHAT-07**: Conversation history persistence
- [ ] **CHAT-08**: Conversation creation from character first_mes
- [ ] **CHAT-09**: Conversation resumption from history
- [ ] **CHAT-10**: Token counter and context utilization indicator

### UI Shell (UI)

- [ ] **UI-01**: GNOME-style header bar with view switcher
- [ ] **UI-02**: Collapsible sidebar for conversation/character navigation
- [ ] **UI-03**: Three primary views: Chat, Characters, Settings
- [ ] **UI-04**: Light/dark/system theme support
- [ ] **UI-05**: Adwaita-inspired color palette and spacing
- [ ] **UI-06**: Keyboard shortcut support (Ctrl+N new chat, Ctrl+B toggle sidebar)
- [ ] **UI-07**: Responsive layout adaptation for different window sizes

### Prompt System (PROMPT)

- [ ] **PROMPT-01**: System prompt assembly from character card
- [ ] **PROMPT-02**: Context window management with configurable budget
- [ ] **PROMPT-03**: Conversation history injection into prompt
- [ ] **PROMPT-04**: Character description and scenario inclusion
- [ ] **PROMPT-05**: Message example (mes_example) formatting

## v2 Requirements

### Conversation Branching (BRANCH)

- **BRANCH-01**: Parent-child message tree storage
- **BRANCH-02**: Branch creation on message edit/regenerate
- **BRANCH-03**: Branch navigation UI (previous/next sibling)
- **BRANCH-04**: Visual branch tree display

### Advanced Features (ADV)

- **ADV-01**: Multi-provider support with quick switching
- **ADV-02**: Prompt presets (save/load parameter profiles)
- **ADV-03**: Lorebook editor with keyword matching
- **ADV-04**: Lorebook recursion and insertion ordering
- **ADV-05**: Group chat with multiple characters
- **ADV-06**: Regex scripting for response post-processing
- **ADV-07**: Full-text search across all conversations
- **ADV-08**: Batch character import/export

### Settings & Polish (POLISH)

- **POLISH-01**: Application settings persistence
- **POLISH-02**: Custom keyboard shortcuts
- **POLISH-03**: Data export/backup functionality
- **POLISH-04**: Data import/restore functionality

## v3 Requirements

### Security & Enterprise (SEC)

- **SEC-01**: Vault Mode with AES-256-GCM database encryption
- **SEC-02**: Passphrase-based key derivation (Argon2id)
- **SEC-03**: Optional startup passphrase requirement

### Platform Expansion (PLATFORM)

- **PLATFORM-01**: Linux packaging (Flatpak, AppImage)
- **PLATFORM-02**: Accessibility audit and screen reader support
- **PLATFORM-03**: Internationalization (i18n) infrastructure

### Ecosystem (ECO)

- **ECO-01**: Plugin system architecture
- **ECO-02**: Plugin API documentation
- **ECO-03**: Community plugin marketplace integration

## Out of Scope

| Feature | Reason |
|---------|--------|
| Mobile application | Desktop-first design; mobile would require separate architecture |
| Built-in AI models | Requires significant ML infrastructure; user-provided API only |
| Cloud synchronization | Contradicts local-first, privacy-centric core value |
| Content moderation | User-controlled content policy; no corporate restrictions |
| Video generation | Text-only AI interactions; out of domain |
| Real-time multiplayer | Focus is AI roleplay, not human-human chat |
| Web-based version | Tauri desktop is the product; web would be separate codebase |
| Account system | No mandatory registration; local-first by design |
| Telemetry/analytics | Privacy-first; no data collection unless explicitly opted-in |

## Traceability

### Phase 1: Foundation

| Requirement | Phase | Status |
|-------------|-------|--------|
| INFRA-01 | Phase 1 | Pending |
| INFRA-02 | Phase 1 | Pending |
| INFRA-03 | Phase 1 | Pending |
| INFRA-04 | Phase 1 | Pending |
| INFRA-05 | Phase 1 | Pending |
| INFRA-06 | Phase 1 | Pending |
| PROV-02 | Phase 1 | Pending |
| UI-01 | Phase 1 | Pending |
| UI-02 | Phase 1 | Pending |
| UI-03 | Phase 1 | Pending |
| UI-05 | Phase 1 | Pending |
| UI-07 | Phase 1 | Pending |

### Phase 2: Core Features

| Requirement | Phase | Status |
|-------------|-------|--------|
| CHAR-01 | Phase 2 | Pending |
| CHAR-02 | Phase 2 | Pending |
| CHAR-03 | Phase 2 | Pending |
| CHAR-04 | Phase 2 | Pending |
| CHAR-05 | Phase 2 | Pending |
| CHAR-06 | Phase 2 | Pending |
| CHAR-07 | Phase 2 | Pending |
| CHAR-09 | Phase 2 | Pending |
| PROV-01 | Phase 2 | Pending |
| PROV-03 | Phase 2 | Pending |
| PROV-04 | Phase 2 | Pending |
| PROV-05 | Phase 2 | Pending |
| PROV-06 | Phase 2 | Pending |
| PROV-07 | Phase 2 | Pending |
| PROV-08 | Phase 2 | Pending |
| CHAT-01 | Phase 2 | Pending |
| CHAT-02 | Phase 2 | Pending |
| CHAT-03 | Phase 2 | Pending |
| CHAT-04 | Phase 2 | Pending |
| CHAT-05 | Phase 2 | Pending |
| CHAT-06 | Phase 2 | Pending |
| CHAT-07 | Phase 2 | Pending |
| CHAT-08 | Phase 2 | Pending |
| CHAT-09 | Phase 2 | Pending |
| CHAT-10 | Phase 2 | Pending |
| UI-04 | Phase 2 | Pending |
| UI-06 | Phase 2 | Pending |
| PROMPT-01 | Phase 2 | Pending |
| PROMPT-02 | Phase 2 | Pending |
| PROMPT-03 | Phase 2 | Pending |
| PROMPT-04 | Phase 2 | Pending |
| PROMPT-05 | Phase 2 | Pending |

### Phase 3: Advanced Features

| Requirement | Phase | Status |
|-------------|-------|--------|
| CHAR-08 | Phase 3 | Pending |
| BRANCH-01 | Phase 3 | Pending |
| BRANCH-02 | Phase 3 | Pending |
| BRANCH-03 | Phase 3 | Pending |
| BRANCH-04 | Phase 3 | Pending |
| ADV-01 | Phase 3 | Pending |
| ADV-02 | Phase 3 | Pending |
| ADV-03 | Phase 3 | Pending |
| ADV-04 | Phase 3 | Pending |
| ADV-07 | Phase 3 | Pending |

### Phase 4: Polish & Distribution

| Requirement | Phase | Status |
|-------------|-------|--------|
| POLISH-01 | Phase 4 | Pending |
| POLISH-02 | Phase 4 | Pending |
| POLISH-03 | Phase 4 | Pending |
| POLISH-04 | Phase 4 | Pending |

**Coverage Summary:**

| Category | Count | Requirements |
|----------|-------|--------------|
| INFRA | 6 | INFRA-01 to INFRA-06 |
| CHAR | 9 | CHAR-01 to CHAR-09 |
| PROV | 8 | PROV-01 to PROV-08 |
| CHAT | 10 | CHAT-01 to CHAT-10 |
| UI | 7 | UI-01 to UI-07 |
| PROMPT | 5 | PROMPT-01 to PROMPT-05 |
| **v1 Total** | **45** | — |

**Phase Distribution:**
- Phase 1 (Foundation): 12 requirements (INFRA ×6, PROV-02, UI ×5)
- Phase 2 (Core Features): 29 requirements (CHAR ×8, PROV ×7, CHAT ×10, UI-04, UI-06, PROMPT ×5)
- Phase 3 (Advanced Features): 4 requirements (CHAR-08, BRANCH-01 to BRANCH-04, ADV-01, ADV-02, ADV-03, ADV-04, ADV-07)
  - *Note: BRANCH and ADV requirements are v2 features included in Phase 3*
- Phase 4 (Polish & Distribution): 4 v2 requirements (POLISH-01 to POLISH-04)

**Coverage:** 45/45 v1 requirements mapped ✓
**Unmapped: 0**

---
*Requirements defined: 2025-04-27*
*Last updated: 2025-04-27 after initial definition*
