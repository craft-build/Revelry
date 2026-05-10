# Feature Landscape

**Domain:** AI roleplay desktop application (Tauri + SvelteKit)

## Table Stakes

Features users expect from any AI roleplay app. Without these, users will leave.

### Streaming AI Chat
- Token-by-token streaming from AI provider — users expect real-time typewriter output, not waiting for full response
- Stop generation mid-stream — ability to abort a response
- Regenerate last response — re-roll with same or different provider
- Edit and resend — modify a sent message and regenerate from that point

**Complexity:** High (requires Tauri channel streaming from rig-core to frontend)
**Dependencies:** rig-core integration, Tauri IPC channels

### Character Cards
- Character creation with name, description, personality, scenario, greeting message, example dialogues
- Character avatars (upload or default initials)
- Character card import/export in TavernAI V2 spec (PNG-embedded JSON metadata) — the de facto standard for sharing characters across platforms
- Character search and filtering in library

**Complexity:** Medium (domain model + CRUD + PNG metadata read/write)
**Dependencies:** SQLite persistence, card I/O service

### Personas
- Persona creation with name, description, avatar — defines "who the user is" in the conversation
- Active persona per conversation — switch persona mid-conversation
- Persona library management (create, edit, delete, search)

**Complexity:** Medium (domain model + CRUD + persona switcher)
**Dependencies:** Character cards, conversation system

### Conversation Management
- Create new conversations with a character
- Conversation history persistence — all messages saved to SQLite
- Conversation list with search, timestamps, character info
- Delete conversations

**Complexity:** Medium (SQLite CRUD + message rendering)
**Dependencies:** Character cards, personas, SQLite

### Prompt Engineering
- System prompt assembly from character card, persona, lorebook entries, and conversation history
- Context window budget management — fit as much relevant context as possible within token limits
- Token counting display — show users their context utilization

**Complexity:** High (budget-aware prompt assembly, tiktoken integration)
**Dependencies:** tiktoken-rs, character/persona models, lorebook

### Provider Configuration
- Add/edit/delete AI providers (OpenAI, Anthropic, custom OpenAI-compatible, etc.)
- API key storage in OS keychain (never on disk)
- Provider-specific model selection
- Generation parameter controls (temperature, top_p, max_tokens, frequency_penalty, presence_penalty)

**Complexity:** Medium (keyring + rig-core provider setup + settings UI)
**Dependencies:** keyring crate, rig-core

### Markdown Rendering
- AI responses rendered as formatted markdown (bold, italic, code blocks, lists, headers)
- Not raw text or naive rendering — users expect rich formatting

**Complexity:** Low (pulldown-cmark on backend → HTML to frontend)
**Dependencies:** pulldown-cmark

## Differentiators

Features that give competitive advantage. Not required for launch but expected in mature products.

### Lorebook (World Info)
- Key-value entries that inject context into prompts when keywords are detected in conversation
- Activation strategies: constant, keyword-triggered, conditional
- Recursive insertion — lorebook entries can reference each other
- Per-character and per-conversation lorebooks
- Ordering and budget allocation for lorebook entries

**Complexity:** High (keyword matching, recursive insertion, budget management)
**Dependencies:** Prompt engine, token counting

### Conversation Branching
- Fork conversation from any message — create parallel timelines
- Navigate between branches — switch parent/child messages
- Branch visualization — show where branches diverge

**Complexity:** High (tree-structured message storage, branch navigation UI)
**Dependencies:** SQLite schema with parent/child message relationships

### Regex Scripting
- User-defined regex patterns applied to AI output — find/replace, formatting fixes, trigger warnings
- Multiple scripts with enable/disable toggles
- Ordered application — scripts run in sequence

**Complexity:** Medium (regex execution engine, script management UI)
**Dependencies:** Regex crate, script persistence

### Full-Text Search
- Search across all conversations and messages
- FTS5 in SQLite for fast full-text queries
- Search results with context snippets

**Complexity:** Medium (FTS5 setup, search UI)
**Dependencies:** SQLite FTS5, conversation/message data

### Theme Toggle
- Dark/light theme switching
- Respect system preference with manual override

**Complexity:** Low (CSS variables + system preference detection)
**Dependencies:** None

### Character/Persona Editors
- Multi-step wizard for creating and editing characters (replaces current card-only view)
- Form validation, preview, field grouping
- Bits.ui Dialog for modal shell

**Complexity:** Medium (form-heavy UI, validation, multi-step)
**Dependencies:** Bits.ui Dialog, character domain model

### Lorebook Editor
- CRUD UI for lorebook entries
- Keyword testing — enter sample text to see which entries activate
- Budget and ordering controls

**Complexity:** Medium (specialized CRUD UI)
**Dependencies:** Lorebook service, character model

## Anti-Features

Things to deliberately NOT build.

- **Cloud sync / multi-device** — This is a local-first app. Cloud sync adds enormous complexity (conflict resolution, auth, server costs) and conflicts with the privacy-first value proposition.
- **Built-in character marketplace** — Let users share cards as files. A marketplace requires moderation, hosting, payments, and legal compliance.
- **Voice / TTS / STT** — Significantly different domain. Out of scope for a text-focused roleplay tool.
- **Image generation** — Different API integration, cost model, and UI patterns. Users can use dedicated image tools.
- **Mobile app** — Desktop-only. Tauri targets desktop; mobile is a different UX paradigm.
- **Vault/encryption mode** — Was speculative in original spec. Not a real requirement per user.

## Feature Dependencies

```
Provider Config ──→ Streaming AI Chat ──→ Prompt Engineering
       │                  │                     │
       ▼                  ▼                     ▼
   API Keys         Conversations          Token Counting
   (keyring)        (SQLite CRUD)         (tiktoken-rs)

Character Cards ──→ Conversations ──→ Personas
       │                                  │
       ▼                                  ▼
   Card I/O                         Persona Switcher
   (PNG import/export)              (per-conversation)

Lorebook ──→ Prompt Engineering (injects context)
       │
       ▼
   Token Counting (budget for lorebook entries)

Regex Scripts ──→ Streaming AI Chat (post-process output)
Theme Toggle ──→ (independent)
FTS Search ──→ Conversations (SQLite FTS5)
Markdown ──→ Streaming AI Chat (render AI output)
Branch Nav ──→ Conversations (tree-structured messages)
Editors ──→ Characters / Personas / Lorebooks (CRUD UI)
```

## Sources

- Existing codebase: `gtk_ref/` domain models, services, and DB schema
- SillyTavern feature set (reference AI roleplay app — open source)
- Character card V2 spec (TavernAI PNG-embedded format)
- rig-core provider list (Context7: /websites/rs_rig-core)
- Design specification: `docs/Revelry_Design_Specification_v2.0.docx`
