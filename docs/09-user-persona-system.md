<!-- Revelry Design Specification v2.0 — Section 9 -->
<!-- Last updated: 2026-05-20 -->

# User Persona System

## 9.1 Purpose and Problem Statement

In most AI roleplay platforms, the user is an invisible, undefined presence. The AI character knows nothing about who it is talking to beyond the raw text of user messages. This creates several problems: flat interactions where AI characters default to treating every user the same way, context waste as users re-introduce themselves manually at the start of each conversation, identity fragmentation for users who play multiple characters across different roleplays, and missed narrative potential when the AI cannot proactively reference the user's identity details. Revelry's User Persona System allows users to define, manage, and switch between distinct roleplay identities that shape how AI characters perceive and respond to them.

## 9.2 User Persona Data Model

### 9.2.1 Schema Definition

A User Persona is a structured record that defines the user's roleplay identity. Unlike AI character cards, which follow the CharacterCardV2 specification, user personas have a simpler but equally important schema designed around the question: Who am I in this story? The schema includes the following fields:

| **Field** | **Required** | **Token Budget** | **Description** |
|---|---|---|---|
| name | Yes | Always included | The persona's display name, used as {{user}} replacement and in UI elements |
| appearance | Yes | Always included | Physical description: height, build, hair, eyes, clothing, distinguishing features |
| personality | Yes | Always included | Core personality traits, emotional tendencies, behavioral patterns |
| background | Recommended | Always if present | The persona's history, origin, and formative experiences |
| role_description | Optional | Always if present | Occupation, social role, or narrative function within the story world |
| relationships | Optional | Always if present | Template string describing relationship to AI characters, supports {{char}} |
| speech_style | Optional | Always if present | How the persona speaks: vocabulary, accent, verbal tics |
| tags | Optional | Not injected | Categorization tags for organization and filtering |
| is_default | Optional | Not injected | Marks one persona as the default for new conversations |
| extensions | Optional | Varies | Extensible namespace for future features and integrations |

### 9.2.2 Template Variables

User personas support template variables that are resolved at prompt assembly time. The {{user}} variable resolves to the active persona's name, the {{char}} variable resolves to the active AI character's name, and {{persona.appearance}}, {{persona.personality}}, {{persona.background}}, and {{persona.role}} resolve to the corresponding persona fields. These variables can be used within the persona's own relationships field and are also available in AI character card fields that support templating, creating a bidirectional reference system.

## 9.3 Prompt Integration

### 9.3.1 Prompt Assembly with User Personas

The user persona system inserts an additional layer into the prompt assembly, positioned between the character definition and the conversation history. The complete prompt structure becomes: System Prompt, then Character Card (sticky), then Character Sample Dialogue (sticky), then Lorebook Entries (activated), then User Persona Block, then Post-History Instructions, then Conversation History, and finally the Current User Message.

### 9.3.2 User Persona Block Construction

The user persona block is assembled as a structured text segment injected into the messages array as a system-level message positioned after the character definition and lorebook entries but before the conversation history begins. Only non-empty fields are included, ensuring that the token budget is not wasted on blank entries. The block is always included when a persona is active for the conversation; if no persona is assigned, the block is omitted entirely, and the {{user}} variable resolves to a configurable default (typically User).

### 9.3.3 Token Budget Impact

User persona data competes for the same context window budget as character cards and lorebook entries. The persona block is treated as a semi-sticky element: it is included in every request, but if the total sticky content exceeds a configurable threshold (default: 50% of context window), the persona block is compressed using the following strategy: Full inclusion (all fields), Trimmed (speech_style and role_description dropped, background truncated to 200 tokens), Minimal (only name, appearance, and personality), or Essential (only name). The compression level is automatically determined by the context manager, and a visual indicator in the chat UI shows the current level.

## 9.4 Persona Assignment and Switching

### 9.4.1 Assignment Hierarchy

Persona assignment follows a resolution hierarchy, from most specific to least specific: Per-conversation override (always wins if set), Per-character default (set on the AI character card), Global default persona (the user's marked default persona), and No persona / raw User (fallback if nothing is configured). This hierarchy allows users to set a sensible global default, override it for specific characters, and further override it for individual conversations.

### 9.4.2 Per-Character Default Personas

AI character cards gain an optional extension field that specifies the default user persona for conversations with that character. When a user starts a new conversation with this character, the application automatically selects the specified persona. If the referenced persona has been deleted, the application falls back to the global default and shows a notification.

### 9.4.3 Mid-Conversation Switching

Users can switch their active persona at any point during a conversation by clicking the persona indicator in the chat header (`PersonaSwitcher` component), which shows the current persona name and avatar. A dropdown appears listing all available personas with search and filtering. After selection, all subsequent AI requests use the new persona's data in the prompt. An optional local system message (not sent to the AI) can be inserted into the conversation for tracking purposes.

## 9.5 Persona UI Components

### 9.5.1 Persona Library View

The persona library is accessed from the Characters View through a tab switcher at the top of the content area: Characters and My Personas. Switching to My Personas displays the user's persona library in the same grid layout used for AI characters, using `PersonaCard` components. Each persona card displays an `Avatar`, the name, the role_description (or first line of personality), tags as `Tag` components, a default badge, and hover-revealed actions (edit, set as default, duplicate, delete).

### 9.5.2 Persona Editor

The persona editor is implemented as a `PersonaEditorDialog` with a tabbed layout aligned with the persona's field structure: Identity (name, avatar, role description), Appearance and Personality (appearance, personality, speech style), Background and Relationships (background, relationships with {{char}} hint), and Settings (tags, set as default, extensions). Each text area includes a character count indicator and a token estimate, helping users understand how much context budget their persona will consume.

### 9.5.3 Persona Switcher

The `PersonaSwitcher` component is a compact dropdown that appears when the user clicks the persona indicator in the chat header. It lists all available personas with the active one highlighted, a No Persona option always listed last, and quick-access buttons for Create New Persona and Manage Personas. Keyboard navigation supports Up/Down arrows for selection, Enter to confirm, and Escape to cancel.
