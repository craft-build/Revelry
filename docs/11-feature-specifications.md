<!-- Revelry Design Specification v2.0 — Section 11 -->
<!-- Last updated: 2026-05-20 -->

# Feature Specifications

This section details the functional requirements for each major feature area, organized by priority tier. Tier 1 features are required for the minimum viable product, Tier 2 features are planned for the first follow-up release, and Tier 3 features are longer-term enhancements.

## 11.1 Conversation Management (Tier 1)

The conversation system is the heart of the application. Each conversation is associated with a character card and a user persona, and maintains its own message history, context state, and generation parameters. Conversations can be created from scratch (starting with the character's first_mes), resumed from history, or forked from any point in an existing conversation. Message editing allows users to modify any previous user message, which automatically creates a new branch at that point while preserving the original branch. The entire branch tree is stored efficiently in SQLite using a parent-child message relationship.

## 11.2 Character Management (Tier 1)

The character management system provides full CRUD operations for character cards, plus import/export functionality. Characters are displayed in a searchable, tag-filterable GtkFlowBox grid. The card editor validates all fields in real-time, warns about potential prompt injection risks in user-provided cards, and provides a live preview of how the character's data will be assembled into the AI prompt. Avatar images are stored as base64 data within the card for portability, with a thumbnail cache for fast grid rendering.

## 11.3 User Persona System (Tier 1-2)

The User Persona System is integrated across Tiers 1 and 2. Tier 1 includes basic persona CRUD, the persona block in prompt assembly, {{user}} variable resolution, and the persona indicator in the chat header. Tier 2 adds the persona switcher popover, per-conversation persona assignment, the global default persona, the persona library view in the Characters view, and token budget awareness for the persona block.

## 11.4 Prompt Presets (Tier 1)

Prompt presets allow users to define and save complete generation parameter profiles. Each preset includes a name, system prompt template, temperature, top_p, max_tokens, frequency_penalty, presence_penalty, and stop sequences. Presets can be assigned at the character level or overridden at the conversation level. A built-in set of starter presets (Creative, Balanced, Precise) provides sensible defaults for new users.

## 11.5 Group Chat (Tier 2)

Group chat allows multiple characters to participate in a single conversation. In a group chat, each message is attributed to a specific character, and the AI generates responses from the appropriate character's perspective. The system prompt for group chats includes all participating characters' descriptions and a turn-order mechanism. Users can manually trigger a specific character to speak, or the system can auto-rotate based on conversation flow.

## 11.6 Lorebook Management (Tier 2)

The lorebook editor provides a structured interface for managing world-building entries within a character card. Entries are displayed in a sortable, filterable GtkColumnView with inline editing. The editor highlights keyword matches in real-time as the user types, showing which entries would activate given the current conversation context and active persona data. Recursive activation is visualized as a dependency tree.

## 11.7 Regex Scripting (Tier 2)

For advanced users, Revelry supports regex-based find-and-replace scripts that can modify AI responses before display. Scripts are defined per-character with a test panel that shows before/after results on sample text. Regex patterns are validated for ReDoS vulnerability before being applied to live conversations, protecting users from malicious scripts.

## 11.8 Conversation Search (Tier 2)

Full-text search across all conversations using SQLite FTS5, allowing users to find specific exchanges by keyword, character name, or date range. Search results are displayed in context with surrounding messages for orientation, and clicking a result navigates directly to that point in the conversation.

## 11.9 Vault Mode (Tier 3)

Vault mode is an optional security feature that encrypts the SQLite database and all character card files using AES-256-GCM encryption with a user-provided passphrase. When vault mode is enabled, the application requires the passphrase on startup to decrypt the database. The encryption key is derived from the passphrase using Argon2id, a memory-hard key derivation function that resists brute-force attacks.

## 11.10 Persona Evolution Tracking (Tier 3)

Persona Evolution Tracking allows users to create timestamped snapshots of their persona at different points in a campaign, with each snapshot linked to a specific conversation or message. Viewing a conversation's history automatically loads the persona snapshot that was active at each message, so the context is always historically accurate.

## 11.11 Persona Inheritance (Tier 3)

Users can create a new persona by inheriting from an existing one, then overriding specific fields. When a child persona is active, the prompt assembly resolves fields by checking the child first; if a field is empty, it falls back to the parent's value. This ensures that variant personas don't require duplicating large amounts of text.

## 11.12 Dual-Persona Mode (Tier 3)

In some roleplay scenarios, the user may want to portray two distinct identities simultaneously. Dual-Persona Mode allows the user to assign two personas to a single conversation and toggle between them using a keyboard shortcut. Each message sent while a persona is active is tagged with that persona's identity in the message metadata.
