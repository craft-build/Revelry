# Roadmap: Persona

**Project:** Persona - Tauri 2.0 Desktop AI Roleplay Application  
**Version:** v1.0.0  
**Granularity:** Coarse (4 phases)  
**Created:** 2025-04-27

---

## Goal

Deliver a production-ready native desktop AI roleplay application with GNOME-inspired design, local-first data sovereignty, and full CharacterCardV2 interoperability.

---

## Phases

- [ ] **Phase 1: Foundation** - Infrastructure, database schema, and UI shell scaffolding
- [ ] **Phase 2: Core Features** - Characters, chat, streaming AI, and provider integration
- [ ] **Phase 3: Advanced Features** - Branching conversations, lorebooks, multi-provider, search
- [ ] **Phase 4: Polish & Distribution** - UI refinement, code signing, and CI/CD pipeline

---

## Phase Details

### Phase 1: Foundation
**Goal:** Establish secure, type-safe infrastructure with working database and UI shell

**Depends on:** Nothing (first phase)

**Requirements:** INFRA-01, INFRA-02, INFRA-03, INFRA-04, INFRA-05, INFRA-06, PROV-02, UI-01, UI-02, UI-03, UI-05, UI-07, USER-01, USER-02, USER-03, USER-04, USER-05, USER-06, USER-07, USER-08

**Success Criteria** (what must be TRUE):
1. Application launches with GNOME-style header bar, sidebar, and three view areas (Chat, Characters, Settings)
2. SQLite database initializes with proper schema and runs migrations automatically
3. API keys are stored in and retrieved from OS keychain (Windows Credential Manager/macOS Keychain)
4. TypeScript types sync with Rust types without manual duplication
5. Tauri 2.0 security capabilities are configured and enforced (no blanket permissions)
6. Error handling framework returns structured, typed errors to frontend
7. Window is responsive and adapts layout to different window sizes
8. User can create, edit, and delete personas with name, appearance, personality, and background
9. User can set a global default persona for new conversations
10. Persona block is correctly injected into prompt assembly
11. `{{user}}` variable resolves to active persona name in prompts
12. Chat header displays active persona indicator with avatar and name

**Plans:** 3 plans

Plans:
- [ ] 01-01-PLAN.md — Infrastructure scaffold, DB, types, keychain, capabilities
- [ ] 01-02-PLAN.md — GNOME UI shell with header bar, sidebar, views, Adwaita theme
- [ ] 01-03-PLAN.md — Persona CRUD system, editor, avatar, prompt block, indicator

**UI hint:** yes

---

### Phase 2: Core Features
**Goal:** Users can create characters, configure AI providers, and engage in streaming conversations

**Depends on:** Phase 1

**Requirements:** CHAR-01, CHAR-02, CHAR-03, CHAR-04, CHAR-05, CHAR-06, CHAR-07, CHAR-09, PROV-01, PROV-03, PROV-04, PROV-05, PROV-06, PROV-07, PROV-08, CHAT-01, CHAT-02, CHAT-03, CHAT-04, CHAT-05, CHAT-06, CHAT-07, CHAT-08, CHAT-09, CHAT-10, UI-04, UI-06, PROMPT-01, PROMPT-02, PROMPT-03, PROMPT-04, PROMPT-05, USER-09, USER-10, USER-11, USER-12, USER-13

**Success Criteria** (what must be TRUE):
1. User can create a character with name, description, personality, and see it in a grid view
2. User can import CharacterCardV2 from PNG or JSON files with all metadata preserved
3. User can export characters as PNG with embedded metadata or JSON files
4. User can configure an OpenAI-compatible provider with custom base URL and model
5. Connection test returns success/failure with clear error messages
6. Starting a chat displays the character's first message and allows user reply
7. AI responses stream token-by-token in real-time with smooth display
8. User can edit their own messages, triggering regeneration of the AI response
9. User can swipe/regenerate AI responses to see alternatives
10. Token counter shows current context usage vs configured budget
11. Conversation history persists and can be resumed from the sidebar
12. Theme toggle works (light/dark/system) across all views
13. Keyboard shortcuts function (Ctrl+N new chat, Ctrl+B toggle sidebar)
14. Prompt correctly assembles system prompt, character context, and conversation history
15. User can switch personas mid-conversation via persona switcher popover
16. Per-conversation persona assignment persists and restores on resumption
17. Persona library view accessible in Characters tab with "My Personas" tab
18. Persona editor shows token estimates for all fields
19. Token budget accounts for persona block with compression when needed

**Plans:** TBD

**UI hint:** yes

---

### Phase 3: Advanced Features
**Goal:** Power users can manage complex conversations with branching, lorebooks, and multi-provider support

**Depends on:** Phase 2

**Requirements:** CHAR-08, BRANCH-01, BRANCH-02, BRANCH-03, BRANCH-04, ADV-01, ADV-02, ADV-03, ADV-04, ADV-07, USER-14, USER-15, USER-16, USER-17, USER-18, USER-19, USER-20

**Note:** BRANCH, ADV, and USER-v2 are v2 requirements; Phase 3 delivers Tier 2 features

**Success Criteria** (what must be TRUE):
1. User can search characters using FTS5 with instant results
2. User can search across all conversations with full-text search
3. Editing or regenerating a message creates a new branch (parent-child relationship)
4. User can navigate between branches using previous/next sibling controls
5. Visual tree view displays conversation branches and their relationships
6. User can configure multiple providers and switch between them per-conversation
7. User can save/load prompt presets with custom parameter profiles
8. User can create lorebook entries with keywords and content
9. Lorebook entries automatically inject when keywords match in conversation
10. Lorebook supports recursion (entries referencing other entries) and configurable insertion order
11. User can assign default persona to characters (auto-applied on new chat)
12. Lorebook entries can activate based on persona fields (appearance, personality, etc.)
13. User can import personas from SpicyChat and SillyTavern formats
14. User can export personas to JSON format (batch or selective)
15. User can search personas using FTS5
16. Conversation sidebar shows persona avatar for each conversation

**Plans:** TBD

**UI hint:** yes

---

### Phase 4: Polish & Distribution
**Goal:** Production-ready application with signed binaries and automated distribution

**Depends on:** Phase 3

**Requirements:** POLISH-01, POLISH-02, POLISH-03, POLISH-04

**Note:** POLISH are v2 requirements; Phase 4 delivers Tier 2-3 polish features

**Success Criteria** (what must be TRUE):
1. Application settings persist across sessions (window size, theme, shortcuts)
2. Custom keyboard shortcuts can be configured and saved
3. User can export/backup all data (characters, conversations, settings)
4. User can import/restore data from backup files
5. macOS binary is code-signed and notarized for Gatekeeper compliance
6. Windows binary is code-signed for SmartScreen compliance
7. CI/CD pipeline builds and packages for Windows, macOS, and Linux on every release
8. Application passes basic accessibility checks (keyboard navigation, screen reader labels)
9. UI has consistent spacing, typography, and visual polish matching GNOME HIG

**Plans:** TBD

**UI hint:** yes

---

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation | 0/3 | Planned | - |
| 2. Core Features | 0/0 | Not started | - |
| 3. Advanced Features | 0/0 | Not started | - |
| 4. Polish & Distribution | 0/0 | Not started | - |

---

## Coverage Summary

**v1 Requirements:** 58 total

| Category | Count |
|----------|-------|
| INFRA | 6 |
| USER | 13 |
| CHAR | 9 |
| PROV | 8 |
| CHAT | 10 |
| UI | 7 |
| PROMPT | 5 |
| **Total** | **58** |

**Phase Distribution:**

| Phase | v1 Requirements | v2 Requirements | Total |
|-------|-----------------|-----------------|-------|
| Phase 1: Foundation | 19 (INFRA ×6, USER ×8, PROV-02, UI ×4) | 0 | 19 |
| Phase 2: Core Features | 34 (CHAR ×8, PROV ×7, CHAT ×10, UI ×2, PROMPT ×5, USER ×5) | 0 | 34 |
| Phase 3: Advanced Features | 1 (CHAR-08) | 16 (BRANCH ×4, ADV ×5, USER-v2 ×7) | 17 |
| Phase 4: Polish & Distribution | 0 | 4 (POLISH ×4) | 4 |

**v2 Requirements Included:**
- BRANCH-01 to BRANCH-04: Conversation branching
- ADV-01 to ADV-04, ADV-07: Multi-provider, presets, lorebooks, FTS5 search
- POLISH-01 to POLISH-04: Settings, shortcuts, import/export

**Coverage:** 45/45 v1 requirements mapped ✓
**All v1 requirements assigned to Phases 1-2**

---

## Dependency Graph

```
Phase 1: Foundation
├── INFRA (scaffold, database, types, keychain, security, errors)
├── USER Foundation (persona CRUD, default, prompt injection, indicator)
├── PROV-02 (API key storage foundation)
└── UI Shell (header, sidebar, views, responsive)
    ↓
Phase 2: Core Features
├── CHAR (creation, editing, import/export, grid, deletion)
├── PROV (configuration, streaming, parameters, errors)
├── CHAT (display, streaming input, editing, regeneration, persistence)
├── USER Extended (switcher, per-conversation, library, token budget)
├── UI Polish (themes, shortcuts, responsive refinements)
└── PROMPT (assembly, context management, {{user}} resolution)
    ↓
Phase 3: Advanced Features
├── Search (character FTS5, conversation FTS5, persona FTS5)
├── USER Advanced (per-character default, persona-aware lorebook, import/export)
├── BRANCH (tree storage, navigation, UI)
├── ADV (multi-provider, presets, lorebooks)
└── Complex State Management
    ↓
Phase 4: Polish & Distribution
├── Settings Persistence
├── Data Import/Export
├── Code Signing (macOS, Windows)
├── CI/CD Pipeline
└── Accessibility & Polish
```

---

## Critical Pitfalls by Phase

### Phase 1: Foundation
- **SQLite Thread Safety:** Use `Mutex<Connection>` or connection pool; never share across async
- **Tauri 2.0 Permissions:** Configure `capabilities/default.json` with explicit permissions; avoid blanket grants
- **API Key Security:** Use `keyring` crate exclusively; never store in database, localStorage, or config files
- **Type Sharing:** Use `ts-rs` or `specta` for automated Rust→TypeScript type generation
- **User Persona Schema:** Design persona table with proper FTS5 index from the start; avatar storage path management
- **Prompt Injection Order:** Persona block must be positioned after character card but before conversation history
- **Single Default Constraint:** Enforce at application level that only one persona can be `is_default=1`

### Phase 2: Core Features
- **IPC Payload Limits:** Paginate conversation loading; use Channels for streaming, not large payloads
- **Streaming Channels:** Use `tauri::ipc::Channel<StreamEvent>` with Token/Complete/Error events
- **Message Swiping:** Implement efficient state updates; avoid re-rendering entire conversation tree
- **Memory Leaks:** Always unlisten to Tauri events in useEffect cleanup
- **Persona Switching State:** Ensure Zustand/TanStack Query state updates correctly when persona changes mid-conversation
- **Token Budget Calculation:** Persona block must be accounted for before conversation history truncation
- **Template Variable Resolution:** `{{user}}` and `{{char}}` must resolve consistently across all prompt components

### Phase 3: Advanced Features
- **Complex State Management:** Separate ephemeral UI state (Zustand) from persistent state (SQLite)
- **Lorebook Performance:** Benchmark keyword scanning; consider indexing for large lorebooks (1000+ entries)
- **Branching Queries:** Design efficient parent-child tree queries; avoid N+1 problems
- **Persona-Aware Lorebook:** Extend keyword matching to scan persona fields; avoid double-processing
- **Import/Export Compatibility:** Handle format variations between SpicyChat, SillyTavern, and Persona schemas
- **FTS5 Persona Search:** Create virtual table for persona search from the start, not as migration

### Phase 4: Polish & Distribution
- **Code Signing:** Configure early in CI; test signing in development, not just at release
- **macOS Notarization:** Requires Apple Developer account and specific entitlements
- **Windows SmartScreen:** Requires code signing certificate from trusted CA

---

## Phase Mapping Details

### Phase 1: Foundation (19 requirements)
| Requirement | Description |
|-------------|-------------|
| INFRA-01 | Tauri 2.0 project scaffold with React 18 + TypeScript 5 |
| INFRA-02 | SQLite database schema with migrations |
| INFRA-03 | Type sharing between Rust backend and TypeScript frontend |
| INFRA-04 | OS keychain integration for secure API key storage |
| INFRA-05 | Tauri capabilities configuration for v2 security model |
| INFRA-06 | Error handling framework with structured error types |
| PROV-02 | API key storage in OS keychain |
| UI-01 | GNOME-style header bar with view switcher |
| UI-02 | Collapsible sidebar for conversation/character navigation |
| UI-03 | Three primary views: Chat, Characters, Settings |
| UI-05 | Adwaita-inspired color palette and spacing |
| UI-07 | Responsive layout adaptation for different window sizes |
| USER-01 | Persona creation with name, appearance, personality, background |
| USER-02 | Persona editing with all fields |
| USER-03 | Persona avatar upload and management |
| USER-04 | Global default persona selection |
| USER-05 | Persona deletion with confirmation |
| USER-06 | Persona block injection into prompt assembly |
| USER-07 | `{{user}}` variable resolution to active persona name |
| USER-08 | Persona indicator in chat header (avatar + name) |

### Phase 2: Core Features (34 requirements)
| Requirement | Description |
|-------------|-------------|
| CHAR-01 | Character card creation with basic fields |
| CHAR-02 | Character card editing with all CharacterCardV2 fields |
| CHAR-03 | PNG import with embedded V2 metadata extraction |
| CHAR-04 | JSON import for V1/V2 card formats |
| CHAR-05 | PNG export with embedded metadata |
| CHAR-06 | JSON export for V2 format |
| CHAR-07 | Character grid view with avatar, name, and tags |
| CHAR-09 | Character deletion with confirmation |
| PROV-01 | Single OpenAI-compatible provider configuration |
| PROV-03 | Base URL configuration for custom endpoints |
| PROV-04 | Default model selection per provider |
| PROV-05 | Connection testing with minimal completion request |
| PROV-06 | Streaming SSE response handling via Tauri Channels |
| PROV-07 | Generation parameters (temperature, top_p, max_tokens) |
| PROV-08 | Error handling for rate limits, auth failures, timeouts |
| CHAT-01 | Message display with user/character attribution |
| CHAT-02 | Streaming token display with real-time updates |
| CHAT-03 | Multi-line message input with send on Ctrl+Enter |
| CHAT-04 | Message editing for user messages |
| CHAT-05 | Message regeneration for character responses |
| CHAT-06 | Message deletion with cascade |
| CHAT-07 | Conversation history persistence |
| CHAT-08 | Conversation creation from character first_mes |
| CHAT-09 | Conversation resumption from history |
| CHAT-10 | Token counter and context utilization indicator |
| UI-04 | Light/dark/system theme support |
| UI-06 | Keyboard shortcut support |
| PROMPT-01 | System prompt assembly from character card |
| PROMPT-02 | Context window management with configurable budget |
| PROMPT-03 | Conversation history injection into prompt |
| PROMPT-04 | Character description and scenario inclusion |
| PROMPT-05 | Message example (mes_example) formatting |
| USER-09 | Persona switcher popover for mid-conversation switching |
| USER-10 | Per-conversation persona assignment storage |
| USER-11 | Persona library view in Characters tab ("My Personas") |
| USER-12 | Persona editor sliding panel with token estimates |
| USER-13 | Token budget awareness for persona block |

### Phase 3: Advanced Features (17 requirements)
| Requirement | Description |
|-------------|-------------|
| CHAR-08 | Character search via SQLite FTS5 |
| BRANCH-01 | Parent-child message tree storage |
| BRANCH-02 | Branch creation on message edit/regenerate |
| BRANCH-03 | Branch navigation UI (previous/next sibling) |
| BRANCH-04 | Visual branch tree display |
| ADV-01 | Multi-provider support with quick switching |
| ADV-02 | Prompt presets (save/load parameter profiles) |
| ADV-03 | Lorebook editor with keyword matching |
| ADV-04 | Lorebook recursion and insertion ordering |
| ADV-07 | Full-text search across all conversations |
| USER-14 | Per-character default persona assignment |
| USER-15 | Persona-aware lorebook activation |
| USER-16 | Import personas from SpicyChat format |
| USER-17 | Import from SillyTavern user profiles |
| USER-18 | Persona JSON export (batch and selective) |
| USER-19 | Persona search via SQLite FTS5 |
| USER-20 | Persona avatar in conversation sidebar entries |

### Phase 4: Polish & Distribution (4 requirements)
| Requirement | Description |
|-------------|-------------|
| POLISH-01 | Application settings persistence |
| POLISH-02 | Custom keyboard shortcuts |
| POLISH-03 | Data export/backup functionality |
| POLISH-04 | Data import/restore functionality |

**Coverage:** 58/58 v1 requirements mapped ✓

---

## v2 Features (Deferred)

The following features are documented in REQUIREMENTS.md but intentionally deferred to v2:

- **ADV-05:** Group chat with multiple characters
- **ADV-06:** Regex scripting for response post-processing
- **ADV-08:** Batch character import/export
- **SEC-01/02/03:** Vault Mode with AES-256-GCM encryption
- **PLATFORM-01/02/03:** Linux packaging, accessibility, i18n
- **ECO-01/02/03:** Plugin system and marketplace

---

*Last updated: 2025-04-28 — User Persona System integrated*
