<!-- Revelry Design Specification v2.0 — Section 12 -->
<!-- Last updated: 2026-05-20 -->

# Data Model and Storage

## 12.1 Core Schema

Revelry's data model is designed for efficient local storage and fast query performance using SQLite via rusqlite. The schema supports the full feature set including branching conversations, lorebooks, user personas, and provider configurations. All tables use TEXT primary keys (UUIDs) for simplicity and portability across SQLite versions.

| **Table** | **Purpose** | **Key Columns** |
|---|---|---|
| characters | AI character card storage | id, name, description, personality, scenario, first_mes, mes_example, system_prompt, post_history_instructions, alternate_greetings, tags, extensions, created_at, updated_at |
| conversations | Conversation metadata | id, character_id, user_persona_id, title, created_at, updated_at, active_branch_tip |
| messages | Conversation messages with branching | id, conversation_id, parent_id, branch_index, role, content, token_count, metadata, created_at |
| lorebook_entries | World-building entries per character | id, character_id, keys, secondary_keys, content, position, depth, activation_state, extensions |
| user_personas | User roleplay identities | id, name, avatar_url, appearance, personality, background, role_description, relationships, speech_style, notes, tags, is_default, extensions, created_at, updated_at |
| persona_snapshots | Persona evolution tracking (Tier 3) | id, persona_id, conversation_id, message_id, snapshot_data, label, created_at |
| providers | AI provider configurations | id, name, provider_type, base_url, model, default_params, extensions |
| prompt_presets | Generation parameter profiles | id, name, system_prompt, temperature, top_p, max_tokens, frequency_penalty, presence_penalty, stop_sequences |
| regex_scripts | Per-character regex post-processing | id, character_id, find_pattern, replace_pattern, flags, order |

## 12.2 Branch Storage Model

Conversation branching uses an adjacency list model where each message references its parent message via a parent_id column. When a user edits a message or regenerates a response, the new message is created with the same parent_id as the original, creating a sibling branch. The branch_index column orders siblings chronologically. The active branch of a conversation is tracked by storing a pointer to the last message in each conversation's current branch. This approach avoids duplicating message data across branches and ensures that storage grows linearly with the number of messages, not exponentially with branching depth.

## 12.3 Backup and Portability

The entire application data is stored in a single directory containing the SQLite database file, a thumbnails cache directory, and a configuration file. Backup is trivial: copying the directory creates a complete backup. The application includes a built-in backup/restore feature that exports the database as a portable SQL dump or a direct file copy. Export to a ZIP archive bundles the database, configuration, and all avatar images into a single file that can be transferred between machines.
