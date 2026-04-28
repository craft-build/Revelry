# Persona — Supplemental Design Document: User Persona System

> **Document Type**: Supplement to *Persona AI Roleplay Desktop App Design Specification v1.0*
> **Date**: April 28, 2026
> **Status**: Design Phase

---

## 1. Introduction

### 1.1 Purpose

The primary design specification for Persona covers AI character cards, conversation management, provider integration, and the GNOME-inspired UI in depth. However, it does not address a critical half of the roleplay equation: **who the user is within the narrative**. This supplemental document specifies the **User Persona System**, which allows users to define, manage, and switch between distinct roleplay identities that shape how AI characters perceive and respond to them.

### 1.2 Problem Statement

In most AI roleplay platforms, the user is an invisible, undefined presence. The AI character knows nothing about who it is talking to beyond the raw text of user messages. This creates several problems:

- **Flat interactions**: Without knowing the user's identity, AI characters default to treating every user the same way, producing generic responses that lack the specificity and depth that make roleplay compelling.
- **Context waste**: Users often re-introduce themselves manually at the start of each conversation ("I'm a tall elf with silver hair..."), consuming valuable context window tokens with repetitive information.
- **Identity fragmentation**: Users who play multiple characters across different roleplays have no structured way to switch between their different roleplay identities, leading to inconsistent characterization.
- **Missed narrative potential**: When the AI knows who the user is — their personality, appearance, history, and relationship to the character — it can proactively reference these details, creating richer and more immersive interactions.

Platforms like SpicyChat.ai address this with their "Personas" feature, allowing users to create named identities that are automatically included in the AI's context. Persona (the application) needs an equivalent system that is more powerful, more flexible, and better integrated with the GNOME design philosophy.

### 1.3 Design Goals

1. **Seamless identity management**: Users should be able to create, edit, and switch between user personas with minimal friction, ideally without leaving the active conversation view.
2. **Rich identity definition**: A user persona should encode at least as much detail as an AI character card — name, appearance, personality, background, and behavioral patterns — giving the AI a complete picture of who it is interacting with.
3. **Context-efficient integration**: Persona data must be injected into the AI prompt efficiently, using the same token budget awareness that governs character card and lorebook injection, so that user identity information enhances rather than crowds the conversation.
4. **Per-character assignment**: Different AI characters may interact with different user personas (e.g., a fantasy character interacts with the user's "elven ranger" persona, while a sci-fi character interacts with their "starship captain" persona). The system should support per-character default personas and per-conversation overrides.
5. **Compatibility**: Where possible, the user persona format should be compatible with or importable from SpicyChat personas, SillyTavern user profiles, and other platform conventions.

---

## 2. User Persona Data Model

### 2.1 Schema Definition

A User Persona is a structured record that defines the user's roleplay identity. Unlike AI character cards, which follow the CharacterCardV2 specification, user personas have a simpler but equally important schema designed around the question: *"Who am I in this story?"*

```
UserPersona {
  id:                 UUID          // Unique identifier
  name:               string        // Display name (required)
  avatar_url:         string?       // Path to avatar image file
  appearance:         string        // Physical description and distinguishing features
  personality:        string        // Core personality traits, mannerisms, speech patterns
  background:         string        // History, origin story, relevant past experiences
  role_description:   string?       // Specific role/occupation within the narrative
  relationships:      string?       // Predefined relationships to characters (template)
  speech_style:       string?       // How this persona talks (vocabulary, tone, quirks)
  notes:              string?       // Creator notes (not injected into prompt)
  tags:               string[]      // Categorization tags (e.g., "fantasy", "sci-fi", "modern")
  is_default:         boolean       // Whether this is the default persona for new chats
  extensions:         object?       // Namespace for app-specific or future extensions
  created_at:         datetime
  updated_at:         datetime
}
```

### 2.2 Field Descriptions

| Field | Required | Token Budget | Description |
|-------|----------|-------------|-------------|
| `name` | Yes | Always included | The persona's display name. This is used in prompt construction as `{{user}}` replacement and in UI elements. Unlike AI character names which appear in chat headers, the persona name is used internally for prompt assembly and in the conversation's user attribution. |
| `appearance` | Yes | Always included | Physical description: height, build, hair, eyes, clothing, distinguishing features. This gives the AI character visual cues to reference in descriptions and reactions. A well-crafted appearance field enables the AI to naturally notice and comment on the user's looks, enhancing immersion. |
| `personality` | Yes | Always included | Core personality traits, emotional tendencies, behavioral patterns, and interpersonal style. This shapes how the AI character interprets the user's actions and how it responds to them. A shy persona will elicit different behavior from a character than a bold one, even with identical user messages. |
| `background` | Recommended | Always included if present | The persona's history, origin, and formative experiences. This provides narrative context that the AI can weave into the story, referencing shared history or reacting to revealed past events. It also helps the AI maintain consistency in how it treats the user across sessions. |
| `role_description` | Optional | Always included if present | The persona's occupation, social role, or narrative function within the story world. Examples: "Knight of the Silver Order," "Freelance starship mechanic," "College student majoring in literature." This provides a quick framing that helps the AI understand the persona's capabilities and social position. |
| `relationships` | Optional | Always included if present | A template string describing the persona's default relationship to AI characters. Supports template variables like `{{char}}` which are resolved per-conversation. Example: `"{{user}} is {{char}}'s childhood friend who moved away years ago and has just returned."` This auto-populates relationship context without manual setup for each character. |
| `speech_style` | Optional | Always included if present | Describes how the persona speaks: vocabulary level, accent indicators, verbal tics, typical sentence structure. This helps the AI understand and mirror the user's character voice, and AI characters can react to the user's speech patterns in-character. |
| `tags` | Optional | Not injected | Used purely for organization and filtering in the persona library. Tags help users quickly find the right persona for a given roleplay scenario. |
| `is_default` | Optional | Not injected | Marks one persona as the default for new conversations. Only one persona can be the default at a time. Setting a new default automatically clears the previous default flag. |
| `extensions` | Optional | Varies | Extensible namespace for future features and third-party integrations. Reserved keys: `persona.accent_color` (UI theme), `persona.avatar_frame`, `persona.sillytavern_profile` (compatibility). |

### 2.3 Comparison with AI Character Cards

User personas and AI character cards serve complementary but structurally different roles in the roleplay system. Understanding these differences is important for avoiding design confusion:

| Aspect | AI Character Card | User Persona |
|--------|------------------|--------------|
| **Represents** | The AI's roleplay identity | The user's roleplay identity |
| **Prompt position** | System prompt + character context | User context block (after character, before history) |
| **Controls** | How the AI speaks and behaves | How the AI perceives and responds to the user |
| **First message** | Has `first_mes` (AI initiates) | No equivalent (user initiates or responds) |
| **Sample dialogue** | Has `mes_example` | Has `speech_style` (simpler, non-dialogue) |
| **Lorebook** | Yes (character's world knowledge) | No (uses the character's lorebook) |
| **Per-conversation** | One AI character per chat | One user persona per chat |
| **Portability format** | PNG+JSON (CharacterCardV2) | JSON only (no PNG embedding needed) |

### 2.4 Template Variables

User personas support template variables that are resolved at prompt assembly time. These variables enable dynamic, context-aware prompt construction without manual customization per character:

| Variable | Resolves To | Example |
|----------|------------|---------|
| `{{user}}` | The active persona's name | "Elara" |
| `{{char}}` | The active AI character's name | "Thorin" |
| `{{persona.appearance}}` | The active persona's appearance text | "A tall elf with silver hair..." |
| `{{persona.personality}}` | The active persona's personality text | "Quiet and observant, slow to trust..." |
| `{{persona.background}}` | The active persona's background text | "Raised in the northern forests..." |
| `{{persona.role}}` | The active persona's role description | "Ranger of the Northern Watch" |

These variables can be used within the persona's own `relationships` field (e.g., `"{{user}} is {{char}}'s former apprentice"`) and are also available in AI character card fields that support templating, creating a bidirectional reference system.

---

## 3. Prompt Integration

### 3.1 Prompt Assembly with User Personas

The prompt assembly process in the main design specification constructs the AI context from system prompt, character card data, lorebook entries, and conversation history. The user persona system inserts an additional layer into this assembly, positioned between the character definition and the conversation history. The complete prompt structure becomes:

```
┌─────────────────────────────────────┐
│  System Prompt                       │  ← Global or character-specific
├─────────────────────────────────────┤
│  Character Card (sticky)             │  ← name, description, personality, scenario
├─────────────────────────────────────┤
│  Character Sample Dialogue (sticky)  │  ← mes_example
├─────────────────────────────────────┤
│  Lorebook Entries (activated)        │  ← Keyword-matched world info
├─────────────────────────────────────┤
│  ★ User Persona Block ★              │  ← NEW: User identity context
├─────────────────────────────────────┤
│  Post-History Instructions           │  ← Character's post_history_instructions
├─────────────────────────────────────┤
│  Conversation History                │  ← Messages with token budget management
├─────────────────────────────────────┤
│  Current User Message                │  ← The message being sent now
└─────────────────────────────────────┘
```

### 3.2 User Persona Block Construction

The user persona block is assembled as a structured text segment injected into the messages array as a system-level message (role: "system") positioned after the character definition and lorebook entries but before the conversation history begins. The block is constructed as follows:

```
[User Identity]
Name: {{persona.name}}
Appearance: {{persona.appearance}}
Personality: {{persona.personality}}
Background: {{persona.background}}
{% if persona.role_description %}
Role: {{persona.role_description}}
{% endif %}
{% if persona.relationships %}
Relationship to {{char}}: {{persona.relationships}}
{% endif %}
{% if persona.speech_style %}
Speech Style: {{persona.speech_style}}
{% endif %}
```

Only non-empty fields are included, ensuring that the token budget is not wasted on blank entries. The block is always included when a persona is active for the conversation; if no persona is assigned, the block is omitted entirely, and the `{{user}}` variable resolves to a configurable default (typically "User").

### 3.3 Token Budget Impact

User persona data competes for the same context window budget as character cards and lorebook entries. The context manager must account for the persona block's token consumption when calculating how much conversation history fits. The persona block is treated as a **semi-sticky** element: it is included in every request (like the character card), but if the total sticky content exceeds a configurable threshold (default: 50% of context window), the persona block is compressed using the following strategy:

1. **Full inclusion**: All fields included (default for most scenarios)
2. **Trimmed**: `speech_style` and `role_description` are dropped; `background` is truncated to 200 tokens
3. **Minimal**: Only `name`, `appearance`, and `personality` are included
4. **Essential**: Only `name` is included (same as no persona)

The compression level is automatically determined by the context manager based on the remaining token budget after accounting for the system prompt, character card, and activated lorebook entries. A visual indicator in the chat UI shows the current compression level so users understand why the AI may not be referencing their full persona details.

### 3.4 Persona and `{{user}}` Variable Resolution

The `{{user}}` template variable is used throughout the prompt assembly pipeline and must consistently resolve to the active persona's name. This affects:

- **Character card fields**: The character's `description`, `scenario`, `first_mes`, and `mes_example` may contain `{{user}}` references. These are resolved at prompt assembly time using the active persona's name.
- **Lorebook entries**: Lorebook content may reference `{{user}}`, resolved to the active persona's name.
- **System prompt**: Custom system prompts may reference `{{user}}`.
- **Post-history instructions**: May reference `{{user}}`.

When the user switches personas mid-conversation, all `{{user}}` references in subsequent prompt assemblies resolve to the new persona's name. The conversation history itself is not rewritten — historical messages retain their original content. However, the AI will naturally adapt to the new identity through the updated persona block in subsequent turns.

---

## 4. Persona Assignment & Switching

### 4.1 Assignment Hierarchy

Persona assignment follows a resolution hierarchy, from most specific to least specific:

```
1. Per-conversation override  →  Always wins if set
2. Per-character default      →  Set on the AI character card
3. Global default persona     →  The user's marked default persona
4. No persona (raw "User")   →  Fallback if nothing is configured
```

This hierarchy allows users to set a sensible global default, override it for specific characters (e.g., always use the "Elven Ranger" persona with fantasy characters), and further override it for individual conversations (e.g., using a different persona for a specific story branch).

### 4.2 Per-Character Default Personas

AI character cards gain an optional extension field that specifies the default user persona for conversations with that character:

```json
{
  "extensions": {
    "persona": {
      "default_user_persona_id": "uuid-of-elven-ranger",
      "default_user_persona_name": "Elara the Ranger"
    }
  }
}
```

When a user starts a new conversation with this character, the application automatically selects the specified persona. The user can still override this selection before or during the conversation. If the referenced persona has been deleted, the application falls back to the global default and shows a notification: *"The character's default persona 'Elara the Ranger' was not found. Using your global default instead."*

### 4.3 Mid-Conversation Switching

Users can switch their active persona at any point during a conversation. This is a common roleplay pattern where the user wants to portray a character transformation, reveal a hidden identity, or simply correct a persona mismatch. The switching mechanism is designed to be quick and non-disruptive:

1. The user clicks the persona indicator in the chat header (showing the current persona name and avatar)
2. A popover appears listing all available personas with search and filtering
3. The user selects a new persona
4. The popover closes and the persona indicator updates
5. A subtle system message is optionally inserted into the conversation: *"[Persona switched from 'Elara' to 'Mysterious Stranger']"*
6. All subsequent AI requests use the new persona's data in the prompt

The optional system message is configurable: users can choose to insert it always, never, or be prompted each time. This message is not sent to the AI — it is a local UI annotation that helps the user track persona changes when reviewing conversation history.

### 4.4 Persona Indicator in Chat View

The chat header includes a compact persona indicator showing the currently active user persona. This indicator consists of:

- **Avatar thumbnail**: The persona's avatar image (32×32 px, circular crop)
- **Name**: The persona's display name, truncated with ellipsis if too long
- **Dropdown chevron**: Indicates clickability

Clicking the indicator opens the persona switcher popover. Hovering reveals a tooltip with the persona's name and a brief excerpt of their appearance field. The indicator is always visible in the chat header, positioned to the left of the AI character's name, creating a visual pairing that reinforces the dual-identity nature of the roleplay.

When no persona is active, the indicator shows a generic user icon and the text "No Persona" with a different visual treatment (lower opacity, dashed border) to signal that the user may want to assign one.

---

## 5. UI Design

### 5.1 Persona Library View

The persona library is accessed from the Characters View through a tab switcher at the top of the content area: **"Characters"** | **"My Personas"**. Switching to "My Personas" displays the user's persona library in the same grid layout used for AI characters, maintaining visual consistency.

Each persona card in the grid displays:
- **Avatar**: Larger circular crop with the persona's accent color as a subtle border
- **Name**: Prominent, same typographic style as character names
- **Role/Tagline**: The `role_description` field (or first line of `personality` if no role)
- **Tags**: Small pill-shaped tags for categorization
- **Default badge**: A small "Default" indicator if this is the global default persona
- **Hover actions**: Edit, Set as Default, Duplicate, Delete

The GNOME design principles apply here: the grid uses generous whitespace, cards have subtle elevation on hover, and actions are revealed progressively rather than cluttering the resting state. The search bar and tag filters from the Characters tab are shared and functional across both tabs.

### 5.2 Persona Editor

The persona editor follows the same sliding panel pattern as the character card editor, emerging from the right side of the screen. It uses a tabbed layout aligned with the persona's field structure:

**Tab 1 — Identity**
- Name (text input, required)
- Avatar (image upload with circular preview)
- Role Description (text input, optional tagline)

**Tab 2 — Appearance & Personality**
- Appearance (multi-line text area with character count)
- Personality (multi-line text area with character count)
- Speech Style (text input, shorter field)

**Tab 3 — Background & Relationships**
- Background (multi-line text area, the longest field)
- Relationships (text area with `{{char}}` variable hint)

**Tab 4 — Settings**
- Tags (tag input with autocomplete)
- Set as Default (toggle)
- Extensions (collapsible JSON editor for advanced users)

Each text area includes a character count indicator and a token estimate, helping users understand how much context budget their persona will consume. The token estimate uses the same tokenizer as the chat engine, ensuring accuracy.

A **Preview** button at the bottom of the editor opens a modal showing exactly how the persona block will appear in the assembled prompt, with the current AI character's name substituted for `{{char}}`. This helps users verify that their persona text reads well in context before saving.

### 5.3 Persona Switcher Popover

The persona switcher is a compact popover that appears when the user clicks the persona indicator in the chat header. It is designed for speed: the user should be able to switch personas in under two seconds.

```
┌──────────────────────────────────┐
│  Switch Persona            [⚙]  │
│  ─────────────────────────────── │
│  🔍 Search personas...          │
│  ─────────────────────────────── │
│  🟢 Elara the Ranger     Default│
│     Ranger of the Northern Watch │
│  ─────────────────────────────── │
│  ○  Captain Voss                │
│     Freelance starship mechanic │
│  ─────────────────────────────── │
│  ○  Mysterious Stranger         │
│     Unknown origins              │
│  ─────────────────────────────── │
│  ○  No Persona                  │
│     Chat without a persona      │
│  ─────────────────────────────── │
│  + Create New Persona            │
│  ✎ Manage Personas              │
└──────────────────────────────────┘
```

- The active persona is marked with a filled circle (🟢) and highlighted background
- "No Persona" is always listed as the last option, allowing the user to disable the persona system entirely for the current conversation
- "Create New Persona" opens the persona editor in a new panel
- "Manage Personas" navigates to the Persona Library in the Characters view
- The gear icon (⚙) opens a mini-settings popover with options: "Insert switch message," "Show token usage," and "Auto-assign per character"
- Keyboard navigation: Up/Down arrows to select, Enter to confirm, Escape to cancel

### 5.4 Conversation Sidebar Enhancements

The conversation sidebar is enhanced to show which persona is assigned to each conversation. Each conversation entry in the sidebar now includes a small avatar indicator (16×16 px) representing the active persona, positioned to the right of the AI character's avatar. This provides at-a-glance identification of which persona is being used in each conversation, which is especially useful when the user has multiple conversations with the same AI character but different user personas.

Hovering over the persona avatar indicator shows the persona name in a tooltip. Right-clicking opens a context menu with "Switch Persona" as an option, allowing persona changes without opening the conversation.

---

## 6. Database Schema

### 6.1 user_personas Table

```sql
CREATE TABLE user_personas (
    id              TEXT PRIMARY KEY,         -- UUID
    name            TEXT NOT NULL,             -- Display name
    avatar_url      TEXT,                      -- Path to avatar image
    appearance      TEXT NOT NULL DEFAULT '',   -- Physical description
    personality     TEXT NOT NULL DEFAULT '',   -- Personality traits
    background      TEXT DEFAULT '',            -- History and origin
    role_description TEXT DEFAULT '',           -- Occupation/role
    relationships   TEXT DEFAULT '',            -- Relationship template with {{char}}
    speech_style    TEXT DEFAULT '',            -- Speech patterns
    notes           TEXT DEFAULT '',            -- Creator notes (not injected)
    tags            TEXT DEFAULT '[]',          -- JSON array of tag strings
    is_default      INTEGER DEFAULT 0,          -- Boolean: global default flag
    extensions      TEXT DEFAULT '{}',          -- JSON object for extensions
    created_at      TEXT NOT NULL,              -- ISO 8601 timestamp
    updated_at      TEXT NOT NULL               -- ISO 8601 timestamp
);
```

### 6.2 Schema Changes to Existing Tables

The following alterations are needed to the schema defined in the primary design specification:

**conversations table** — Add column:

```sql
ALTER TABLE conversations ADD COLUMN user_persona_id TEXT REFERENCES user_personas(id);
```

This stores the per-conversation persona override. When NULL, the application resolves the persona using the assignment hierarchy (per-character default → global default → no persona).

**characters table** — Add extension field:

The `extensions` JSON column gains a new key:

```json
{
  "persona": {
    "default_user_persona_id": "uuid-here"
  }
}
```

No schema migration is needed for this since `extensions` is already a JSON column. The application code handles the new key with a null check.

### 6.3 Migration Script

```sql
-- Migration: 004_add_user_personas.sql

CREATE TABLE user_personas (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    avatar_url      TEXT,
    appearance      TEXT NOT NULL DEFAULT '',
    personality     TEXT NOT NULL DEFAULT '',
    background      TEXT DEFAULT '',
    role_description TEXT DEFAULT '',
    relationships   TEXT DEFAULT '',
    speech_style    TEXT DEFAULT '',
    notes           TEXT DEFAULT '',
    tags            TEXT DEFAULT '[]',
    is_default      INTEGER DEFAULT 0,
    extensions      TEXT DEFAULT '{}',
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

ALTER TABLE conversations ADD COLUMN user_persona_id TEXT REFERENCES user_personas(id);

-- FTS index for persona search
CREATE VIRTUAL TABLE user_personas_fts USING fts5(
    name, appearance, personality, background, role_description,
    content=user_personas, content_rowid=rowid
);
```

---

## 7. Import/Export & Compatibility

### 7.1 SpicyChat Persona Import

SpicyChat exports personas in a JSON format that can be mapped to Persona's schema. The import adapter handles the following field mappings:

| SpicyChat Field | Persona Field | Notes |
|----------------|---------------|-------|
| `name` | `name` | Direct mapping |
| `avatar` | `avatar_url` | Downloaded and cached locally |
| `description` | `personality` + `appearance` | Split by heuristic: first paragraph → appearance, rest → personality |
| `greeting` | *(not mapped)* | SpicyChat personas have a greeting; Persona does not need this |
| `scenario` | `background` | Maps to background context |

SpicyChat personas are imported as a batch operation from the Settings → Data Management panel. The user uploads the SpicyChat export JSON, previews the detected personas, and selects which ones to import. Imported personas are tagged with "imported:spicychat" for traceability.

### 7.2 SillyTavern User Profile Import

SillyTavern stores user profile information in a simpler format (typically just a name and a brief description). The import adapter maps:

| SillyTavern Field | Persona Field | Notes |
|------------------|---------------|-------|
| `name` | `name` | Direct mapping |
| `description` | `personality` | Single field in SillyTavern |

Since SillyTavern user profiles are minimal, the imported persona will have empty `appearance`, `background`, and other fields. The user is encouraged to fill these in through the editor after import.

### 7.3 Persona JSON Export Format

User personas are exported as a JSON file with the following structure:

```json
{
  "spec": "persona_user_v1",
  "spec_version": "1.0",
  "exported_at": "2026-04-28T12:00:00Z",
  "exported_by": "Persona Desktop v1.0",
  "personas": [
    {
      "name": "Elara the Ranger",
      "appearance": "A tall elf woman with silver hair and emerald green eyes...",
      "personality": "Quiet and observant, slow to trust but fiercely loyal once bonded...",
      "background": "Raised in the northern forests by a reclusive order of rangers...",
      "role_description": "Ranger of the Northern Watch",
      "relationships": "{{user}} is {{char}}'s traveling companion on a dangerous quest.",
      "speech_style": "Speaks in short, measured sentences. Uses nature metaphors.",
      "tags": ["fantasy", "elf", "ranger"],
      "extensions": {}
    }
  ]
}
```

This format is designed to be both human-readable and machine-parseable. The `spec` field distinguishes it from AI character card files, preventing confusion during import. Batch export exports all personas; selective export allows the user to choose which personas to include.

### 7.4 Avatar Handling

Persona avatars are stored as files in the application's data directory (`{data_dir}/avatars/personas/`) with filenames derived from the persona's UUID. During import, avatar URLs from other platforms are downloaded and cached locally. During export, avatar files are base64-encoded and embedded in the JSON export for portability. The application maintains a thumbnail cache (64×64 and 32×32 variants) for fast rendering in the persona grid and switcher popover.

---

## 8. Advanced Features

### 8.1 Persona-Aware Lorebook Activation

Lorebook entries on AI character cards can be configured to activate based on the active user persona's data, not just the conversation text. This enables characters to have special knowledge or reactions that only surface when interacting with specific types of user personas. For example, a vampire character might have a lorebook entry that activates when the user persona's `appearance` field contains the word "vampire" or "pale," causing the character to recognize the user as a fellow vampire.

This is implemented by extending the lorebook keyword matching algorithm to scan the active persona's text fields in addition to the recent conversation messages. A new activation mode, `activation_on_persona`, is added to the lorebook entry schema:

```json
{
  "extensions": {
    "persona": {
      "activation_on_persona": true,
      "persona_scan_fields": ["appearance", "personality", "background", "role_description"]
    }
  }
}
```

When `activation_on_persona` is true and the persona's scanned fields contain the entry's keywords, the lorebook entry is activated even if the keywords haven't appeared in the conversation yet. This creates a "first impression" effect where the AI character immediately responds to the user's identity.

### 8.2 Persona Evolution Tracking

Over long roleplay campaigns, a user's persona may change significantly — a peasant becomes a knight, a stranger becomes a trusted ally. Persona Evolution Tracking allows users to create timestamped snapshots of their persona at different points in a campaign, with each snapshot linked to a specific conversation or message.

The implementation uses a new table:

```sql
CREATE TABLE persona_snapshots (
    id              TEXT PRIMARY KEY,
    persona_id      TEXT NOT NULL REFERENCES user_personas(id),
    conversation_id TEXT REFERENCES conversations(id),
    message_id      TEXT REFERENCES messages(id),
    snapshot_data   TEXT NOT NULL,    -- JSON: full persona state at this point
    label           TEXT DEFAULT '',  -- User-provided label, e.g., "After coronation"
    created_at      TEXT NOT NULL
);
```

Users can create a snapshot manually through the persona editor ("Save Snapshot") or automatically at configurable intervals (every N messages, or when the user edits the persona mid-conversation). Viewing a conversation's history automatically loads the persona snapshot that was active at each message, so the context is always historically accurate.

This is a **Tier 3** feature, planned for Phase 4 of the development roadmap.

### 8.3 Persona Inheritance

Users can create a new persona by inheriting from an existing one, then overriding specific fields. This is useful for creating persona variants (e.g., "Elara — Winter Outfit" which inherits all of Elara's personality and background but changes the appearance and adds a seasonal note). The inheritance relationship is tracked in the database:

```sql
ALTER TABLE user_personas ADD COLUMN parent_persona_id TEXT REFERENCES user_personas(id);
```

When a child persona is active, the prompt assembly resolves fields by checking the child first; if a field is empty, it falls back to the parent's value. This ensures that variant personas don't require duplicating large amounts of text. The persona editor shows inherited fields with a visual indicator ("Inherited from: Elara the Ranger") and allows the user to override any field.

This is a **Tier 3** feature, planned for Phase 4 of the development roadmap.

### 8.4 Dual-Persona Mode

In some roleplay scenarios, the user may want to portray two distinct identities simultaneously — for example, a character who is secretly a double agent, or twins switching places. Dual-Persona Mode allows the user to assign two personas to a single conversation and toggle between them using a keyboard shortcut (Ctrl+Tab by default).

In Dual-Persona Mode, the persona indicator in the chat header shows both personas with a visual toggle. The active persona is highlighted, and switching is instant. Each message sent while a persona is active is tagged with that persona's identity in the message metadata. The AI receives the active persona's data in the prompt block, so it reacts differently depending on which identity the user is currently portraying.

This is a **Tier 3** feature, planned for Phase 4 or later, pending user demand validation during beta testing.

---

## 9. Feature Priority & Roadmap Integration

The User Persona System features are integrated into the existing development roadmap as follows:

### Tier 1 — MVP (Phase 1-2)

| Feature | Phase | Rationale |
|---------|-------|-----------|
| Basic persona CRUD (create, read, update, delete) | Phase 1 | Core functionality, required for any persona usage |
| Persona block in prompt assembly | Phase 1 | The entire system is useless without prompt integration |
| `{{user}}` variable resolution | Phase 1 | Required for character card compatibility |
| Persona indicator in chat header | Phase 1 | Users need to see which persona is active |
| Persona switcher popover | Phase 2 | Essential for switching between personas |
| Per-conversation persona assignment | Phase 2 | Core assignment model |
| Global default persona | Phase 2 | Reduces friction for single-persona users |
| Persona library in Characters view | Phase 2 | Organization and management |
| Token budget awareness for persona block | Phase 2 | Prevents context overflow |

### Tier 2 — Follow-Up (Phase 3)

| Feature | Phase | Rationale |
|---------|-------|-----------|
| Per-character default persona | Phase 3 | Reduces manual switching for multi-character use |
| Persona-aware lorebook activation | Phase 3 | Enables "first impression" narrative effects |
| Import from SpicyChat / SillyTavern | Phase 3 | Migration support for existing users |
| Persona JSON export | Phase 3 | Portability and backup |
| Mid-conversation persona switching | Phase 3 | Common roleplay pattern |
| Persona editor with token estimates | Phase 3 | Helps users optimize context usage |
| Conversation sidebar persona indicators | Phase 3 | At-a-glance identity tracking |

### Tier 3 — Long-Term (Phase 4+)

| Feature | Phase | Rationale |
|---------|-------|-----------|
| Persona evolution tracking | Phase 4 | Advanced campaign management |
| Persona inheritance | Phase 4 | Reduces duplication for variant personas |
| Dual-persona mode | Phase 4+ | Niche but powerful narrative tool |
| Persona compression levels | Phase 4 | Fine-grained context budget control |
| Relationship template builder | Phase 4+ | Guided UI for creating `{{char}}` relationships |

---

## 10. Open Questions

The following design decisions remain open and should be resolved during implementation or after user testing:

1. **Persona block format**: Should the persona block use the structured key-value format described in Section 3.2, or should it be rendered as a natural-language paragraph? The key-value format is token-efficient and unambiguous, but a narrative paragraph may produce more natural AI responses. This should be A/B tested during beta.

2. **Anonymous persona mode**: Should there be a mode where the user's persona data is included in the prompt but the AI character is instructed to not explicitly acknowledge the user's appearance or background unless the user mentions it first? This could reduce unwanted AI behavior where characters comment on the user's looks in every first message.

3. **Persona sharing**: Should user personas be shareable like AI character cards? There may be community interest in sharing well-crafted user personas (e.g., "Generic Fantasy Hero," "Cyberpunk Runner"), but this raises questions about the personal nature of user identity data.

4. **Multi-user conversations**: If group chat is extended to support multiple human users (each with their own persona), the prompt architecture needs significant changes. This is out of scope for the current design but should be considered in the data model.

5. **Persona versioning**: Should edits to a persona create version history, allowing users to revert to a previous version? This is related to the Persona Evolution Tracking feature but operates at the persona definition level rather than the conversation level.
