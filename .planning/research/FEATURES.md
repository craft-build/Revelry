# Feature Landscape: AI Roleplay Applications

**Domain:** AI Character Roleplay Desktop Applications
**Researched:** 2025-04-27
**Overall Confidence:** HIGH

## Executive Summary

The AI roleplay application ecosystem is mature with well-established patterns. The primary competitors are **Character.ai** (cloud-first, proprietary), **SillyTavern** (open-source, feature-dense desktop), and **RisuAI** (cross-platform with advanced scripting). The **CharacterCardV2** specification serves as the de facto interchange format, enabling character portability across platforms.

For Persona (a Tauri-based desktop app), the key differentiator is **GNOME-inspired minimalist design** combined with **local-first data sovereignty**, targeting users who want SillyTavern's power without its complexity, and Character.ai's ease without its restrictions.

---

## Table Stakes

Features users expect from any AI roleplay application. Missing these = users leave.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Character Cards (V2)** | Ecosystem standard for character portability | Medium | Must support V1/V2 import/export (PNG + JSON). Fields: name, description, personality, scenario, first_mes, mes_example |
| **Streaming Responses** | Real-time feedback reduces perceived latency | Low | OpenAI API streaming via SSE. Essential for good UX |
| **Conversation History** | Users expect to resume past chats | Low | SQLite storage with message threading |
| **OpenAI-Compatible API** | Universal backend support | Medium | Single provider MVP; multi-provider later. Supports 20+ backends |
| **Message Swiping** | Compare/regenerate responses | Low | Core UX pattern in all competitors |
| **Basic Prompting** | System prompt, character context | Medium | system_prompt, post_history_instructions per CharacterCardV2 |
| **Search/Filter Characters** | Manage growing character libraries | Low | Basic text search; tags in Tier 2 |

### CharacterCardV2 Ecosystem Requirements (HIGH Confidence)

Per the [Character Card V2 Specification](https://github.com/malfoyslastname/character-card-spec-v2), any serious roleplay app **MUST** support:

**V1 Legacy Fields (always present):**
- `name`: Character name
- `description`: Character description and world info
- `personality`: Brief personality summary
- `scenario`: Circumstances/context of dialogue
- `first_mes`: First message (greeting)
- `mes_example`: Example dialogue blocks

**V2 New Fields (when spec === 'chara_card_v2'):**
- `spec_version`: "2.0"
- `creator_notes`: Usage instructions (never in prompt)
- `system_prompt`: Character-specific system prompt override
- `post_history_instructions`: UJB/Jailbreak replacement (high weight)
- `alternate_greetings`: Array of alternative first messages
- `character_book`: Embedded lorebook entries
- `tags`: Categorization tags (UI only, not prompt)
- `creator`: Author name
- `character_version`: Version string
- `extensions`: Custom key-value storage (preserve unknown fields)

**CharacterBook Format:**
```typescript
{
  name?: string,
  description?: string,
  scan_depth?: number,
  token_budget?: number,
  recursive_scanning?: boolean,
  extensions: Record<string, any>,
  entries: Array<{
    keys: string[],           // Trigger keywords
    content: string,          // Inserted text
    extensions: Record<string, any>,
    enabled: boolean,
    insertion_order: number,  // Lower = higher in prompt
    case_sensitive?: boolean,
    name?: string,            // Display name (not in prompt)
    priority?: number,        // Discard order if budget exceeded
    selective?: boolean,      // Require secondary_keys
    secondary_keys?: string[],
    constant?: boolean,       // Always insert within budget
    position?: 'before_char' | 'after_char'
  }>
}
```

**Import/Export Formats:**
- **PNG**: Character data embedded in image metadata (base64 JSON in tEXt chunk)
- **JSON**: Direct JSON file with .json extension
- Must preserve all V2 fields on re-export

---

## Differentiating Features

Features that set Persona apart from competitors.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **GNOME Minimalist Design** | Content-first, reduced cognitive load | Medium | Adwaita palette, 4px grid, symbolic icons. Differentiator vs SillyTavern's density |
| **Conversation Branching** | Explore "what if" scenarios | High | Parent-child message model with tree visualization. SillyTavern has basic checkpoints |
| **Lorebook Editor** | Dynamic context injection | High | Keyword matching, recursion, token budgeting. Critical for complex RP |
| **Multi-Provider Support** | Freedom from vendor lock-in | Medium | Provider presets with model lists. After MVP |
| **Prompt Presets** | Shareable prompt engineering | Low | Save/load system prompt + post-history combinations |
| **Conversation Search (FTS5)** | Find past conversations | Medium | SQLite FTS5 for full-text search |
| **Group Chat** | Multi-character interactions | High | Multiple characters in one conversation. Tier 3 |
| **Regex Scripting** | Response post-processing | Medium | Pattern-based text transformation. Power user feature |
| **Vault Mode (Encryption)** | Maximum privacy | Medium | AES-256-GCM encrypted chats. Tier 3 differentiator |
| **No Cloud/Account Required** | Data sovereignty | N/A | Architectural decision, not a feature. Major differentiator vs Character.ai |

---

## Tier Classification

Based on PROJECT.md requirements, organized by implementation phase:

### Tier 1: MVP (Must Have)

| Feature | Purpose | Dependencies |
|---------|---------|--------------|
| Core chat with streaming | Basic functionality | OpenAI API client |
| Single OpenAI-compatible provider | Backend connectivity | API key management |
| Basic character card CRUD | Character management | CharacterCardV2 parser |
| SQLite + migrations | Data persistence | rusqlite |
| GNOME UI shell | Application frame | Tauri + React |
| Message swiping | Response comparison | Message versioning |
| Import/Export (JSON) | Character portability | File system access |

### Tier 2: Core Differentiators

| Feature | Purpose | Dependencies |
|---------|---------|--------------|
| Conversation branching | Explore alternatives | Parent-child message model |
| Multiple provider support | Backend flexibility | Provider abstraction layer |
| Prompt preset system | Shareable configurations | Settings management |
| CharacterCardV2 PNG import/export | Ecosystem compatibility | PNG metadata parsing |
| Lorebook editor | Dynamic context | CharacterBook implementation |
| Conversation search | Content discovery | FTS5 integration |

### Tier 3: Advanced Features

| Feature | Purpose | Dependencies |
|---------|---------|--------------|
| Group chat | Multi-character RP | Group orchestration logic |
| Regex scripting | Response processing | Regex engine + UI |
| Vault Mode (encryption) | Privacy hardening | Encryption at rest |
| Accessibility + i18n | Inclusivity | WCAG compliance, i18n framework |

---

## OpenAI API Integration Patterns

### Standard Request Format (HIGH Confidence)

```typescript
// OpenAI Chat Completions API
POST /v1/chat/completions
{
  "model": "gpt-4o-mini",
  "messages": [
    { "role": "system", "content": "system_prompt + character context" },
    { "role": "user", "content": "User message" },
    { "role": "assistant", "content": "Previous assistant response" }
  ],
  "stream": true,
  "temperature": 0.8,
  "max_tokens": 1024,
  "presence_penalty": 0,
  "frequency_penalty": 0
}
```

### Streaming Response Handling

```typescript
// Server-Sent Events format
data: {"id":"...","object":"chat.completion.chunk","choices":[{"delta":{"content":"Hello"}}]}
data: {"id":"...","object":"chat.completion.chunk","choices":[{"delta":{"content":" there"}}]}
data: [DONE]
```

### Compatible Providers (20+)

- **Cloud**: OpenAI, Anthropic (Claude), Google (Gemini), Cohere, Mistral
- **Aggregators**: OpenRouter, DeepInfra
- **Local**: Ollama, LM Studio, KoboldCPP, text-generation-webui, TabbyAPI

**Recommendation**: Start with single provider, abstract for multi-provider later.

---

## Conversation Branching UX Patterns

### SillyTavern Approach

**Checkpoints**: Clone chat up to specific message
- "Create Branch": Clone and switch to new chat
- "Create Checkpoint": Clone but stay in current
- "Back to parent": Navigate up the tree

**Limitations**: Linear navigation, no visual tree

### Recommended Persona Approach

**Tree-Based Model:**
```
Conversation Tree
├── Message 1 (root)
│   ├── Message 2a (branch A)
│   │   └── Message 3a
│   └── Message 2b (branch B)
│       └── Message 3b
```

**UI Patterns:**
- **Branch indicator**: Small icon on messages with branches
- **Branch selector**: Dropdown/popover to switch between branches
- **Tree view**: Optional sidebar showing full conversation tree
- **Merge**: Ability to copy messages between branches

**Database Model:**
```typescript
interface Message {
  id: string;
  parent_id: string | null;  // null for root
  branch_id: string;         // Conversation branch identifier
  content: string;
  role: 'user' | 'assistant';
  created_at: Date;
}
```

---

## Lorebook/World Info Systems

### Core Concepts (HIGH Confidence)

**Lorebooks** (also called World Info, Memory Books) dynamically insert context based on keywords found in conversation.

**Key Features from SillyTavern/RisuAI:**

1. **Keyword Matching**
   - Primary keys trigger entry insertion
   - Secondary keys for selective filtering (AND ANY, AND ALL, NOT ANY, NOT ALL)
   - Regex support for advanced matching
   - Case sensitivity option
   - Whole word matching

2. **Insertion Control**
   - **Insertion Order**: Priority within token budget (lower = higher in prompt)
   - **Position**: before_char, after_char, @depth, example messages
   - **Constant entries**: Always inserted (within budget)
   - **Probability**: Random chance of insertion (0-100%)

3. **Token Budget**
   - Context percentage or absolute token limit
   - Entries sorted by order, discarded if budget exceeded
   - Recursive entries counted against budget

4. **Recursion**
   - Entry content can trigger other entries
   - Configurable recursion depth
   - Prevents infinite loops

5. **Timed Effects (SillyTavern Advanced)**
   - **Sticky**: Stay active for N messages after trigger
   - **Cooldown**: Cannot trigger for N messages after activation
   - **Delay**: Cannot trigger until N messages exist

6. **Inclusion Groups**
   - Mutually exclusive entries
   - Random or priority-based selection
   - Group scoring for specificity

### Implementation Complexity

| Component | Complexity | Notes |
|-----------|------------|-------|
| Keyword scanning | Low | Scan last N messages for keys |
| Entry sorting | Low | By insertion_order |
| Token budgeting | Medium | Requires token counting |
| Recursion | Medium | Graph traversal with cycle detection |
| Regex matching | Medium | JavaScript-compatible regex |
| Timed effects | High | State tracking per chat |
| Vector storage (RisuAI) | High | Embeddings for semantic matching |

### Simplified MVP Approach

For Tier 2, implement core features:
- Keyword matching (plain text + case sensitivity)
- Insertion ordering
- Token budgeting (approximate with character count / 4)
- Basic recursion (2-3 levels max)
- Skip: Timed effects, inclusion groups, vector storage

---

## Anti-Features

Features to explicitly NOT build, with rationale.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Cloud Sync** | Violates local-first principle | Export/import for backup; optional future peer sync |
| **Content Moderation/Filtering** | User-controlled content policy | Provide clear UI that content is user-controlled |
| **Built-in AI Models** | Adds 2-10GB to binary | Require external provider (Ollama, API) |
| **Mobile App** | Desktop-first design mandate | Responsive web view for v2 consideration |
| **Multi-User Chat** | Scope creep; focus on AI RP | Single user + AI characters only |
| **Image/Video Generation** | Scope expansion | Focus on text excellence; integrate external tools if needed |
| **Social Features** | Privacy concerns, scope creep | No sharing, no public profiles, no discovery |
| **Real-time Voice** | High complexity, niche use case | Optional TTS integration for output only |

---

## Feature Dependencies

```
CharacterCardV2 Support
├── Basic Fields (name, description, etc.)
├── system_prompt
├── post_history_instructions
├── alternate_greetings
└── character_book
    └── Lorebook System
        ├── Keyword matching
        ├── Token budgeting
        └── Recursion

Conversation System
├── Linear Messages
└── Branching (requires)
    ├── Parent-child relationships
    ├── Tree navigation UI
    └── Message versioning

Provider System
├── Single Provider (MVP)
└── Multi-Provider (requires)
    ├── Provider abstraction
    ├── Model lists per provider
    └── Settings per provider
```

---

## Competitive Analysis

### Character.ai
**Strengths:** Polished UX, discoverability, mobile apps
**Weaknesses:** Cloud-only, strict filters, no local control
**Persona Advantage:** Local-first, uncensored, data ownership

### SillyTavern
**Strengths:** Extremely feature-rich, multi-provider, extensible
**Weaknesses:** Complex/dense UI, steep learning curve, Node.js heavy
**Persona Advantage:** GNOME simplicity, Tauri performance, native feel

### RisuAI
**Strengths:** Tauri-based (like Persona!), plugins, advanced scripting
**Weaknesses:** Busy UI, less popular than SillyTavern
**Persona Advantage:** Better design philosophy, focused feature set

### Agnai.chat
**Strengths:** Web-based, service model
**Weaknesses:** Self-hosted complexity, smaller ecosystem
**Persona Advantage:** Desktop-native, zero-config

---

## Sources

- Character Card V2 Specification: https://github.com/malfoyslastname/character-card-spec-v2 (HIGH confidence)
- SillyTavern Documentation: https://docs.sillytavern.app/ (HIGH confidence)
- RisuAI Repository: https://github.com/kwaroran/RisuAI (MEDIUM confidence)
- OpenAI API Reference: https://platform.openai.com/docs/api-reference (HIGH confidence)

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Table Stakes | HIGH | Clear ecosystem standards from CharacterCardV2 |
| Differentiators | MEDIUM | Subject to user validation; GNOME design unproven in this space |
| Architecture | HIGH | Well-established patterns from SillyTavern/RisuAI |
| Implementation Complexity | MEDIUM | Tauri/React well-understood; branching/lorebook need care |
| Competitive Position | MEDIUM | Validation needed with target users |

## Gaps to Address

1. **User Research**: Validate GNOME design appeal vs power-user expectations
2. **Performance**: Test Tauri streaming performance with large context windows
3. **Character Import**: Verify PNG metadata extraction works across card sources
4. **Lorebook UX**: Design intuitive editor for complex entry management
