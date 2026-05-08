# Requirements: Revelry

**Defined:** 2026-04-28
**Core Value:** Users can roleplay with AI characters through any AI provider they choose — cloud or local — with complete creative freedom and complete data privacy.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### AI Providers

- [x] **PROV-01**: User can configure multiple AI providers simultaneously (OpenAI, Anthropic, Ollama, Gemini, Cohere, Mistral, Groq, DeepSeek, OpenRouter, Azure OpenAI, Together AI, HuggingFace, Perplexity, xAI, and custom OpenAI-compatible endpoints)
- [x] **PROV-02**: User can connect to any configured provider through rig's unified abstraction and receive completion responses
- [ ] **PROV-03**: User sees AI responses stream token-by-token with smooth typewriter rendering
- [ ] **PROV-04**: User can cancel an in-progress AI generation at any point (stop button or Escape key)
- [x] **PROV-05**: User's API keys are stored in the OS-native credential store (GNOME Keyring, macOS Keychain, Windows Credential Manager) and never written to database, config files, or logs

### Characters

- [x] **CHAR-01**: User can create, edit, and delete AI character cards following the CharacterCardV2 specification
- [x] **CHAR-02**: User can import character cards from PNG files with embedded V2 metadata
- [x] **CHAR-03**: User can import character cards from JSON files (V1 and V2 formats with auto-migration)
- [x] **CHAR-04**: User can export character cards as PNG files with embedded metadata
- [x] **CHAR-05**: User can export character cards as JSON files
- [x] **CHAR-06**: User can browse characters in a searchable, tag-filterable grid (GtkFlowBox)
- [x] **CHAR-07**: User can search characters by name, description, and tags using instant full-text matching
- [ ] **CHAR-08**: Template variables {{user}}, {{char}}, and {{persona.*}} are resolved at prompt assembly time in character card fields

### Conversations

- [ ] **CONV-01**: User can create a new conversation with a character (starting with character's first_mes)
- [ ] **CONV-02**: User can resume an existing conversation from the conversation list
- [ ] **CONV-03**: User can view conversation history with character messages left-aligned and user messages right-aligned
- [ ] **CONV-04**: User can edit any previous user message, creating a new branch at that point while preserving the original
- [ ] **CONV-05**: User can regenerate any AI response, creating a sibling branch
- [ ] **CONV-06**: User can navigate between conversation branches with visual branch indicators
- [ ] **CONV-07**: User can delete any message from conversation history
- [ ] **CONV-08**: AI responses are rendered as formatted text using pulldown-cmark + Pango markup (bold, italic, headers, lists)
- [ ] **CONV-09**: User can send messages via text input with Ctrl+Enter or send button
- [ ] **CONV-10**: Partial messages (from cancelled generation) are preserved and functional in history

### User Personas

- [x] **PERS-01**: User can create, edit, and delete user personas with structured fields (name, appearance, personality, background, role_description, relationships, speech_style, tags)
- [ ] **PERS-02**: Active user persona data is injected into the AI prompt as a structured block between character definition and conversation history
- [ ] **PERS-03**: Persona assignment follows a resolution hierarchy: per-conversation override → per-character default → global default persona → no persona (raw User)
- [x] **PERS-04**: User can assign a default persona to an AI character card (auto-selected when starting new conversations)
- [x] **PERS-05**: User can mark one persona as global default for new conversations without character-specific defaults
- [ ] **PERS-06**: {{user}} variable resolves to active persona's name; {{persona.*}} variables resolve to corresponding persona fields

### Context Engineering

- [x] **CCTX-01**: Prompt assembly follows a prioritized layer system: system prompt + character card (sticky) → lorebook entries (activated) → user persona block (semi-sticky) → conversation history (volatile, oldest truncated first)
- [x] **CCTX-02**: Token counting uses tiktoken-rs BPE tokenization to track context window utilization
- [x] **CCTX-03**: User can see a context utilization indicator showing how much of the context window is consumed
- [x] **CCTX-04**: User can manage lorebook entries per character with key terms, secondary terms, content, insertion position, depth, and activation state
- [x] **CCTX-05**: Lorebook entries activate when keywords appear in the recent conversation and are injected into the AI context
- [x] **CCTX-06**: Lorebook supports recursive activation where activated entries can trigger other entries up to a configurable depth limit
- [x] **CCTX-07**: Lorebook entries have token budgeting to prevent world info from consuming the entire context

### Generation Controls

- [ ] **GENR-01**: User can create, edit, and delete generation parameter presets (name, system prompt template, temperature, top_p, max_tokens, frequency_penalty, presence_penalty, stop sequences)
- [ ] **GENR-02**: User can assign a generation preset at the character level (used as default for all conversations with that character)
- [ ] **GENR-03**: User can override the generation preset at the conversation level
- [ ] **GENR-04**: Built-in starter presets (Creative, Balanced, Precise) are available for new users

### Regex Scripting

- [ ] **RGXO-01**: User can define regex find-and-replace scripts per character (find pattern, replace pattern, flags, execution order)
- [ ] **RGXO-02**: Regex scripts modify AI responses before display
- [ ] **RGXO-03**: Regex patterns are validated for ReDoS vulnerability before being applied to live conversations
- [ ] **RGXO-04**: User can test regex scripts on sample text with before/after preview panel

### Data Management

- [ ] **DATA-01**: User can export all application data as a portable SQL dump
- [ ] **DATA-02**: User can export all application data as a ZIP archive (database, configuration, avatar images)
- [ ] **DATA-03**: User can import application data from a previously exported SQL dump or ZIP archive

### UI Shell

- [ ] **UISL-01**: Application displays an AdwHeaderBar with view switcher (Chat, Characters, Settings)
- [ ] **UISL-02**: Application uses AdwOverlaySplitView for adaptive sidebar/content layout (side-by-side in wide windows, overlay in narrow)
- [ ] **UISL-03**: Sidebar contains conversation list and can be toggled with F9 keyboard shortcut
- [ ] **UISL-04**: Application follows GNOME design principles: content-first, progressive disclosure, consistent visual language via libadwaita
- [ ] **UISL-05**: All actions can be performed via keyboard navigation per GNOME accessibility standards (Ctrl+N, Ctrl+B, Ctrl+F, Escape, Ctrl+Enter, etc.)
- [x] **UISL-06**: Settings are presented in an AdwPreferencesWindow with categorized panels

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Characters (Advanced)

- **CHAR-A01**: Guided character creation wizard using multi-step AdwPreferencesWindow
- **CHAR-A02**: Import from SillyTavern persona format
- **CHAR-A03**: Import from CharacterAI community export format
- **CHAR-A04**: Batch import/export for character libraries
- **CHAR-A05**: Validated import with security checks (schema validation, PNG chunk limits, ReDoS detection)

### Conversations (Advanced)

- **CONV-A01**: FTS5 full-text search across all conversations with keyword, character name, and date range filters
- **CONV-A02**: Search results displayed in context with surrounding messages; click to navigate to that point in conversation

### User Personas (Advanced)

- **PERS-A01**: Mid-conversation persona switching via popover in chat header
- **PERS-A02**: Persona token compression (full → trimmed → minimal → essential) when token budget is tight
- **PERS-A03**: Per-conversation persona assignment (override character and global defaults)
- **PERS-A04**: Persona library view accessible from Characters view via tab switcher
- **PERS-A05**: Persona evolution tracking with timestamped snapshots linked to conversations
- **PERS-A06**: Child persona inheritance with field-level fallback resolution
- **PERS-A07**: Dual-persona mode for portraying two identities in a single conversation

### Context Engineering (Advanced)

- **CCTX-A01**: Persona-aware lorebook activation (entries activate based on active persona data fields in addition to conversation keywords)

### Group Chat

- **GRPT-01**: User can start a group chat with multiple AI characters in a single conversation
- **GRPT-02**: Each message is attributed to a specific character; AI generates responses from the appropriate character's perspective
- **GRPT-03**: User can manually trigger a specific character to speak or use auto-rotation based on conversation flow

### Security

- **SECU-01**: Vault mode encrypts SQLite database and character files with AES-256-GCM using Argon2id key derivation from user passphrase
- **SECU-02**: Application requires passphrase on startup when vault mode is enabled

## Out of Scope

| Feature | Reason |
|---------|--------|
| Cloud synchronization | Contradicts local-first, privacy-first core value; requires server infrastructure |
| Online account system | No accounts = no tracking = no data leaks; settings and data live locally |
| Built-in content moderation/filtering | Target audience explicitly rejects this; creative freedom is non-negotiable |
| Web-based UI (Electron/Tauri/WebView) | Defeats core technical differentiator; every competitor already does web UI |
| Built-in image generation | Not core to text roleplay; requires separate server infrastructure or API costs |
| Built-in TTS | Not core to text roleplay; complex integration surface (15+ providers in competitors) |
| Built-in RAG/vector storage | Lorebook covers most knowledge injection needs; embedding/chunking is huge scope |
| Extension/plugin system | SillyTavern's extension system is its most complex subsystem; ship solid core first |
| Character expression sprites | Requires sentiment classification model; complex GTK4 adaptation (no CSS overlay) |
| Visual Novel mode | Web-specific UI paradigm (full-screen sprite + text); not aligned with GNOME patterns |
| Chat translation | Not core; users can use OS-level translation tools |
| Web search integration | Not core to roleplay; requires search API keys |
| STscript/scripting engine | Enormous scope; slash commands for basic operations suffice |
| Multi-user/collaborative chat | Requires server infrastructure, auth, real-time sync; contradicts local-first |
| Subscription/monetization | Personal hobby project; free and open source |
| Mobile application | Desktop-first; GTK4 on mobile is immature; separate future project |
| Social features (marketplace, sharing) | Requires cloud infrastructure; local import/export suffices |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| UISL-01 | Phase 1 | Pending |
| UISL-02 | Phase 1 | Pending |
| UISL-03 | Phase 1 | Pending |
| UISL-04 | Phase 1 | Pending |
| UISL-05 | Phase 1 | Pending |
| UISL-06 | Phase 1 | Complete |
| PROV-01 | Phase 1 | Complete |
| PROV-02 | Phase 1 | Complete |
| PROV-05 | Phase 1 | Complete |
| CHAR-01 | Phase 2 | Complete |
| CHAR-02 | Phase 2 | Complete |
| CHAR-03 | Phase 2 | Complete |
| CHAR-04 | Phase 2 | Complete |
| CHAR-05 | Phase 2 | Complete |
| CHAR-06 | Phase 2 | Complete |
| CHAR-07 | Phase 2 | Complete |
| PROV-03 | Phase 3 | Pending |
| PROV-04 | Phase 3 | Pending |
| CONV-01 | Phase 3 | Pending |
| CONV-02 | Phase 3 | Pending |
| CONV-03 | Phase 3 | Pending |
| CONV-04 | Phase 3 | Pending |
| CONV-05 | Phase 3 | Pending |
| CONV-06 | Phase 3 | Pending |
| CONV-07 | Phase 3 | Pending |
| CONV-08 | Phase 3 | Pending |
| CONV-09 | Phase 3 | Pending |
| CONV-10 | Phase 3 | Pending |
| PERS-01 | Phase 4 | Complete (04-02) |
| PERS-02 | Phase 4 | Pending |
| PERS-03 | Phase 4 | Pending |
| PERS-04 | Phase 4 | Complete (04-02) |
| PERS-05 | Phase 4 | Complete (04-02) |
| PERS-06 | Phase 4 | Pending |
| CHAR-08 | Phase 4 | Pending |
| CCTX-01 | Phase 5 | Complete (05-02) |
| CCTX-02 | Phase 5 | Complete (05-01) |
| CCTX-03 | Phase 5 | Complete (05-03) |
| CCTX-04 | Phase 5 | Complete (05-03) |
| CCTX-05 | Phase 5 | Complete (05-02) |
| CCTX-06 | Phase 5 | Complete (05-02) |
| CCTX-07 | Phase 5 | Complete (05-02) |
| GENR-01 | Phase 6 | Pending |
| GENR-02 | Phase 6 | Pending |
| GENR-03 | Phase 6 | Pending |
| GENR-04 | Phase 6 | Pending |
| RGXO-01 | Phase 6 | Pending |
| RGXO-02 | Phase 6 | Pending |
| RGXO-03 | Phase 6 | Pending |
| RGXO-04 | Phase 6 | Pending |
| DATA-01 | Phase 6 | Pending |
| DATA-02 | Phase 6 | Pending |
| DATA-03 | Phase 6 | Pending |

**Coverage:**
- v1 requirements: 53 total
- Mapped to phases: 53
- Unmapped: 0 ✓

---
*Requirements defined: 2026-04-28*
*Last updated: 2026-04-28 after roadmap creation*
