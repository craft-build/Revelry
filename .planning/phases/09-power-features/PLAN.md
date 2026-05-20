# Phase 9: Power Features — Implementation Plan

```yaml
wave: 1
depends_on: []
files_modified:
  - src-tauri/src/domain/lorebook.rs
  - src-tauri/src/db/migrations.rs
  - src-tauri/src/db/lorebook.rs
  - src-tauri/src/db/models.rs
  - src-tauri/src/db/mod.rs
  - src-tauri/src/service/lorebook.rs
  - src-tauri/src/service/mod.rs
  - src/lib/types/index.ts
autonomous: true
```

## Plan 09-01: Port LorebookService — Domain, Migration, DB, and Service

**Requirements:** LORB-01, LORB-05, PRMP-04, PRMP-05

### Task 1: Extend LorebookEntry domain model with token_budget, sort_order, enabled

<read_first>

- src-tauri/src/domain/lorebook.rs (current domain model being extended)
- gtk_ref/src/domain/lorebook.rs (reference for field names: keywords, secondary_terms, token_budget, sort_order, enabled)
  </read_first>

<action>
Add three new fields to the `LorebookEntry` struct in `src-tauri/src/domain/lorebook.rs`:

1. `token_budget: Option<i64>` — inserted after `extensions`, before `created_at`. Uses `#[serde(default)]` and `#[serde(skip_serializing_if = "Option::is_none")]`. Default is `None`.
2. `sort_order: i32` — inserted after `token_budget`, before `created_at`. Uses `#[serde(default)]` annotation. Default is `0` (via `fn default_sort_order() -> i32 { 0 }`).
3. `enabled: bool` — inserted after `sort_order`, before `created_at`. Uses `#[serde(default = "default_enabled")]` annotation. Default is `true` (via `fn default_enabled() -> bool { true }`).

Update `LorebookEntry::new()` to initialize:

- `token_budget: None`
- `sort_order: 0`
- `enabled: true`

Update the existing `test_lorebook_entry_serde_roundtrip` test to set and verify all three new fields:

- `entry.token_budget = Some(100);`
- `entry.sort_order = 2;`
- `entry.enabled = false;`

Add new tests:

- `test_lorebook_entry_defaults` — create entry via `new()`, assert `token_budget.is_none()`, `sort_order == 0`, `enabled == true`
- `test_lorebook_entry_serde_with_token_budget` — set `token_budget = Some(50)`, serialize/deserialize, verify `token_budget == Some(50)`
- `test_lorebook_entry_serde_without_token_budget` — leave `token_budget = None`, serialize/deserialize, verify `token_budget == None`
  </action>

<acceptance_criteria>

- `src-tauri/src/domain/lorebook.rs` contains `pub token_budget: Option<i64>` field on `LorebookEntry`
- `src-tauri/src/domain/lorebook.rs` contains `pub sort_order: i32` field on `LorebookEntry`
- `src-tauri/src/domain/lorebook.rs` contains `pub enabled: bool` field on `LorebookEntry`
- `LorebookEntry::new()` initializes `token_budget: None, sort_order: 0, enabled: true`
- `cargo test -p revelry-app -- domain::lorebook::tests` exits 0
- Serialized JSON for a new entry does NOT include `token_budget` when it is `None`
- Serialized JSON for an entry with `token_budget: Some(100)` includes `"token_budget": 100`
  </acceptance_criteria>

### Task 2: Add migration — conversation_lorebooks table + lorebook_entries columns

<read_first>

- src-tauri/src/db/migrations.rs (existing migrations, pattern for new M::up blocks)
- src-tauri/src/db/lorebook.rs (existing lorebook_entries table columns from Phase 3 migration)
  </read_first>

<action>
Add a new `M::up` / `M::down` block at the end of the migrations vector in `src-tauri/src/db/migrations.rs`. The migration must:

**Up:**

```sql
-- Add token_budget column to lorebook_entries (nullable INTEGER)
ALTER TABLE lorebook_entries ADD COLUMN token_budget INTEGER;

-- Add sort_order column to lorebook_entries (default 0)
ALTER TABLE lorebook_entries ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;

-- Add enabled column to lorebook_entries (default 1 = true)
ALTER TABLE lorebook_entries ADD COLUMN enabled INTEGER NOT NULL DEFAULT 1;

-- Create conversation_lorebooks junction table (mirrors character_lorebooks structure)
CREATE TABLE conversation_lorebooks (
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    lorebook_id TEXT NOT NULL REFERENCES lorebooks(id) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (conversation_id, lorebook_id)
);
CREATE INDEX idx_conversation_lorebooks_conv ON conversation_lorebooks(conversation_id, sort_order);
```

**Down:**

```sql
DROP INDEX IF EXISTS idx_conversation_lorebooks_conv;
DROP TABLE IF EXISTS conversation_lorebooks;
ALTER TABLE lorebook_entries DROP COLUMN enabled;
ALTER TABLE lorebook_entries DROP COLUMN sort_order;
ALTER TABLE lorebook_entries DROP COLUMN token_budget;
```

Add tests in the migrations test module:

- `test_lorebook_entries_has_token_budget` — assert `lorebook_entries` table has `token_budget` column
- `test_lorebook_entries_has_sort_order` — assert `lorebook_entries` table has `sort_order` column
- `test_lorebook_entries_has_enabled` — assert `lorebook_entries` table has `enabled` column
- `test_conversation_lorebooks_table_exists` — assert `conversation_lorebooks` table exists
- `test_conversation_lorebooks_columns` — assert `conversation_lorebooks` has columns: `conversation_id, lorebook_id, sort_order`
  </action>

<acceptance_criteria>

- `src-tauri/src/db/migrations.rs` contains `ALTER TABLE lorebook_entries ADD COLUMN token_budget INTEGER`
- `src-tauri/src/db/migrations.rs` contains `ALTER TABLE lorebook_entries ADD COLUMN sort_order`
- `src-tauri/src/db/migrations.rs` contains `ALTER TABLE lorebook_entries ADD COLUMN enabled`
- `src-tauri/src/db/migrations.rs` contains `CREATE TABLE conversation_lorebooks`
- `src-tauri/src/db/migrations.rs` contains `CREATE INDEX idx_conversation_lorebooks_conv`
- `cargo test -p revelry-app -- db::migrations::tests` exits 0
- Running migrations on a fresh in-memory DB produces a `lorebook_entries` table with `token_budget`, `sort_order`, `enabled` columns
- Running migrations on a fresh in-memory DB produces a `conversation_lorebooks` table with `conversation_id`, `lorebook_id`, `sort_order` columns
  </acceptance_criteria>

### Task 3: Update DB layer — lorebook_entry queries and conversation_lorebooks CRUD

<read_first>

- src-tauri/src/db/lorebook.rs (existing CRUD functions, pattern for junction table operations)
- src-tauri/src/db/models.rs (row_to_lorebook_entry mapper — needs new fields)
  </read_first>

<action>
**Update `row_to_lorebook_entry` in `src-tauri/src/db/models.rs`:**

The SELECT queries must include the new columns. Update the function to read:

- `token_budget: row.get::<_, Option<i64>>("token_budget")?` — nullable, read as `Option<i64>`
- `sort_order: row.get::<_, i32>("sort_order")?`
- `enabled: row.get::<_, i32>("enabled")? != 0` — stored as INTEGER, convert to bool

Add these fields to the returned `LorebookEntry` struct construction.

**Update all SQL queries in `src-tauri/src/db/lorebook.rs` that read/write lorebook_entries:**

1. `load_lorebook_entries()` — update SELECT to include `token_budget, sort_order, enabled` columns
2. `insert_lorebook_entry()` — add `token_budget, sort_order, enabled` to INSERT columns and VALUES; `token_budget` uses `entry.token_budget`, `sort_order` uses `entry.sort_order`, `enabled` uses `entry.enabled as i32`
3. `update_lorebook_entry()` — add `token_budget = ?X, sort_order = ?Y, enabled = ?Z` to SET clause; add corresponding params

**Add conversation_lorebooks junction CRUD functions in `src-tauri/src/db/lorebook.rs`:**

4. `assign_lorebook_to_conversation(conn, conversation_id, lorebook_id, sort_order)` — `INSERT OR IGNORE INTO conversation_lorebooks (conversation_id, lorebook_id, sort_order) VALUES (?1, ?2, ?3)` (mirrors character_lorebooks pattern)
5. `remove_lorebook_from_conversation(conn, conversation_id, lorebook_id)` — `DELETE FROM conversation_lorebooks WHERE conversation_id = ?1 AND lorebook_id = ?2`
6. `load_lorebooks_for_conversation(conn, conversation_id)` — `SELECT l.id, l.name, l.description, l.created_at, l.updated_at FROM lorebooks l INNER JOIN conversation_lorebooks cl ON cl.lorebook_id = l.id WHERE cl.conversation_id = ?1 ORDER BY cl.sort_order ASC`

**Add a bulk entry loader for multiple lorebooks (used by ChatContext):**

7. `load_entries_for_lorebooks(conn, lorebook_ids: &[String])` — returns `Vec<LorebookEntry>` for all entries belonging to any of the given lorebook_ids. Uses `WHERE lorebook_id IN (` + dynamic placeholders + `)` pattern.

**Add tests:**

- `test_insert_and_load_entry_with_new_fields` — create entry with `token_budget = Some(50)`, `sort_order = 3`, `enabled = false`, verify loaded values match
- `test_entry_defaults_when_new_fields_absent` — insert via raw SQL without new columns (simulating pre-migration data), verify loaded defaults: `token_budget = None`, `sort_order = 0`, `enabled = true`
- `test_conversation_lorebooks_assign_and_load` — assign a lorebook to a conversation, verify `load_lorebooks_for_conversation` returns it
- `test_conversation_lorebooks_remove` — assign then remove, verify empty
- `test_load_entries_for_lorebooks` — create 2 lorebooks with entries, call `load_entries_for_lorebooks(&[lb1.id, lb2.id])`, verify all entries returned
  </action>

<acceptance_criteria>

- `src-tauri/src/db/models.rs` function `row_to_lorebook_entry` reads `token_budget`, `sort_order`, `enabled` columns
- `src-tauri/src/db/lorebook.rs` `insert_lorebook_entry` includes `token_budget, sort_order, enabled` in INSERT
- `src-tauri/src/db/lorebook.rs` `update_lorebook_entry` includes `token_budget, sort_order, enabled` in UPDATE SET
- `src-tauri/src/db/lorebook.rs` contains `pub fn assign_lorebook_to_conversation` with signature `(conn: &Connection, conversation_id: &str, lorebook_id: &str, sort_order: i32) -> Result<()>`
- `src-tauri/src/db/lorebook.rs` contains `pub fn remove_lorebook_from_conversation` with signature `(conn: &Connection, conversation_id: &str, lorebook_id: &str) -> Result<()>`
- `src-tauri/src/db/lorebook.rs` contains `pub fn load_lorebooks_for_conversation` with signature `(conn: &Connection, conversation_id: &str) -> Result<Vec<Lorebook>>`
- `src-tauri/src/db/lorebook.rs` contains `pub fn load_entries_for_lorebooks` with signature `(conn: &Connection, lorebook_ids: &[String]) -> Result<Vec<LorebookEntry>>`
- `cargo test -p revelry-app -- db::lorebook::tests` exits 0
  </acceptance_criteria>

### Task 4: Add DbCommand variants for conversation lorebooks and bulk entry loading

<read_first>

- src-tauri/src/db/mod.rs (existing DbCommand enum and handle_command match arms)
  </read_first>

<action>
Add these new `DbCommand` variants to the enum in `src-tauri/src/db/mod.rs`:

```rust
// Conversation-lorebook junction
AssignLorebookToConversation {
    conversation_id: String,
    lorebook_id: String,
    sort_order: i32,
    reply: oneshot::Sender<anyhow::Result<()>>,
},
RemoveLorebookFromConversation {
    conversation_id: String,
    lorebook_id: String,
    reply: oneshot::Sender<anyhow::Result<()>>,
},
LoadConversationLorebooks {
    conversation_id: String,
    reply: oneshot::Sender<anyhow::Result<Vec<Lorebook>>>,
},
// Bulk entry loading for multiple lorebooks (used by ChatContext)
LoadLorebookEntriesForLorebooks {
    lorebook_ids: Vec<String>,
    reply: oneshot::Sender<anyhow::Result<Vec<LorebookEntry>>>,
},
```

Add corresponding match arms in `DbWorker::handle_command`:

- `AssignLorebookToConversation` → call `lorebook::assign_lorebook_to_conversation(conn, &conversation_id, &lorebook_id, sort_order)`
- `RemoveLorebookFromConversation` → call `lorebook::remove_lorebook_from_conversation(conn, &conversation_id, &lorebook_id)`
- `LoadConversationLorebooks` → call `lorebook::load_lorebooks_for_conversation(conn, &conversation_id)`
- `LoadLorebookEntriesForLorebooks` → call `lorebook::load_entries_for_lorebooks(conn, &lorebook_ids)`

Also update the `LoadLorebookEntries` match arm: since `insert_lorebook_entry` now checks entry existence using `load_lorebook_entries`, and the new columns are returned, the save logic should still work. No changes needed to existing match arms beyond verifying compilation.
</action>

<acceptance_criteria>

- `src-tauri/src/db/mod.rs` contains `AssignLorebookToConversation` variant
- `src-tauri/src/db/mod.rs` contains `RemoveLorebookFromConversation` variant
- `src-tauri/src/db/mod.rs` contains `LoadConversationLorebooks` variant
- `src-tauri/src/db/mod.rs` contains `LoadLorebookEntriesForLorebooks` variant
- `src-tauri/src/db/mod.rs` has match arms for all four new variants that call the corresponding `lorebook::` functions
- `cargo check -p revelry-app` exits 0
  </acceptance_criteria>

### Task 5: Create service/lorebook.rs — port LorebookService from gtk_ref

<read_first>

- gtk_ref/src/service/lorebook.rs (reference implementation: activate_lorebook_entries, inject_lorebook_entries, keyword_matches, truncate_to_token_budget, ActivatedEntry)
- src-tauri/src/domain/lorebook.rs (tauri domain model — keys, secondary_keys, LorebookPosition, LorebookActivation, enabled, token_budget, sort_order)
- src-tauri/src/service/token_count.rs (TokenCountService for budget enforcement)
- src-tauri/src/service/prompt.rs (prompt layer system where lorebook injection occurs)
- src-tauri/src/domain/conversation.rs (Message, MessageRole structs used in scan)
  </read_first>

<action>
Create `src-tauri/src/service/lorebook.rs`. Port `activate_lorebook_entries`, `inject_lorebook_entries`, `keyword_matches`, `truncate_to_token_budget` from gtk_ref, adapting to the tauri domain model.

**Struct `ActivatedEntry`:**

```rust
pub struct ActivatedEntry {
    pub entry: LorebookEntry,
    pub injected_content: String,
    pub effective_depth: i32,
}
```

**Function `activate_lorebook_entries`:**

Signature:

```rust
pub fn activate_lorebook_entries(
    all_entries: &[LorebookEntry],
    recent_messages: &[Message],
    scan_depth: i32,
    max_recursive_depth: i32,
    lorebook_token_budget: i64,
    token_service: &TokenCountService,
) -> Vec<ActivatedEntry>
```

Algorithm (ported from gtk_ref with tauri adaptations):

1. Filter to `enabled == true` entries only
2. Take last `scan_depth` messages (reversed, newest first) as scan scope
3. **Adaptation for LorebookActivation**: Build separate scan texts per activation type:
   - `scan_text_all`: content of all scan messages (lowercased, concatenated)
   - `scan_text_character_only`: content of only `MessageRole::Assistant` messages (lowercased)
   - `scan_text_user_only`: content of only `MessageRole::User` messages (lowercased)
4. Initial keyword scan: for each enabled entry, select scan text based on `activation_state`:
   - `LorebookActivation::Always` → scan against `scan_text_all`
   - `LorebookActivation::CharacterOnly` → scan against `scan_text_character_only`
   - `LorebookActivation::UserOnly` → scan against `scan_text_user_only`
   - Check `keys` (maps from gtk_ref `keywords`) OR `secondary_keys` (maps from gtk_ref `secondary_terms`) against the selected scan text using `keyword_matches()`. OR logic: ANY key or secondary_key triggers activation.
5. Recursive activation: iterate up to `max_recursive_depth` times. Each iteration:
   - Concatenate content of all currently activated entries (lowercased) into `activated_content`
   - Check each not-yet-activated enabled entry against `activated_content` using the same activation_state-based scan text selection (but for recursive, always use the activated content, not the original messages)
   - Stop early if no new activations
6. Sort by `sort_order` (ascending)
7. Token budget enforcement:
   - Track `remaining_budget = lorebook_token_budget`
   - For entries with explicit `token_budget`: truncate content via `truncate_to_token_budget` if exceeds budget; deduct `min(budget, content_tokens)` from remaining
   - For entries without explicit budget: if `remaining_budget <= 0`, skip; otherwise if content fits, deduct tokens; if not, truncate to fit remaining and set remaining to 0

**Function `inject_lorebook_entries`:**

Signature:

```rust
pub fn inject_lorebook_entries(
    preamble_before_char: &str,
    preamble_after_char: &str,
    entries: &[ActivatedEntry],
    messages: &mut Vec<rig_core::completion::Message>,
) -> (String, String)
```

Returns `(preamble_before_char_with_injections, preamble_after_char_with_injections)`. Separates entries by `position`:

- `LorebookPosition::BeforeChar` → append to `preamble_before_char`
- `LorebookPosition::AfterChar` → append to `preamble_after_char`
- `LorebookPosition::AtDepth` where `depth == 0` → append to `preamble_after_char`
- `LorebookPosition::AtDepth` where `depth > 0` → inject as system message into `messages` at N messages from end (processed from highest depth to lowest to preserve indices)

**Function `keyword_matches`:**

```rust
fn keyword_matches(keywords: &[String], text: &str) -> bool
```

Same as gtk_ref: OR logic, case-insensitive, empty keywords don't match.

**Function `truncate_to_token_budget`:**

```rust
fn truncate_to_token_budget(content: &str, budget: usize, token_service: &TokenCountService) -> String
```

Same binary search approach as gtk_ref.

**Register module in `src-tauri/src/service/mod.rs`:**
Add `pub mod lorebook;` and `pub use lorebook::{ActivatedEntry, LorebookService};`

Wait — use a struct-based approach or free functions? Following the existing codebase pattern (PromptEngine uses free functions in a struct impl), use the same pattern. But the gtk_ref uses free functions, which is simpler. Keep it as free functions for consistency with the port.

Add to `src-tauri/src/service/mod.rs`: `pub mod lorebook;`

**Tests:** Port the test suite from gtk_ref, adapting to tauri domain types:

- `test_keyword_in_recent_messages_activates_entry`
- `test_keyword_not_present_no_activation`
- `test_or_logic_any_keyword_activates`
- `test_disabled_entry_not_activated`
- `test_scan_depth_limits_scope`
- `test_recursive_activation`
- `test_recursive_max_depth_prevents_infinite_loop`
- `test_per_entry_token_budget_truncates`
- `test_entries_without_budget_share_remaining`
- `test_depth_from_bottom_insertion`
- `test_secondary_keys_trigger_activation`
- `test_sort_order_respected`
- `test_activation_state_character_only` — NEW: entry with `CharacterOnly` only activates on assistant messages
- `test_activation_state_user_only` — NEW: entry with `UserOnly` only activates on user messages
- `test_activation_state_always_activates_on_both` — NEW: `Always` activates on any message
- `test_inject_before_char` — NEW: `LorebookPosition::BeforeChar` entries go to preamble_before_char
- `test_inject_after_char` — NEW: `LorebookPosition::AfterChar` entries go to preamble_after_char
- `test_inject_at_depth` — NEW: `AtDepth` with depth > 0 injects system messages
- `test_keyword_matches_or_logic`
- `test_keyword_matches_case_insensitive`
- `test_keyword_matches_empty_keyword`

Use `crate::domain::conversation::Message` with `DateTime<Utc>` timestamps (not NaiveDateTime like gtk_ref).
</action>

<acceptance_criteria>

- `src-tauri/src/service/lorebook.rs` exists and contains `pub fn activate_lorebook_entries`
- `src-tauri/src/service/lorebook.rs` contains `pub fn inject_lorebook_entries`
- `src-tauri/src/service/lorebook.rs` contains `fn keyword_matches`
- `src-tauri/src/service/lorebook.rs` contains `fn truncate_to_token_budget`
- `src-tauri/src/service/lorebook.rs` contains `pub struct ActivatedEntry` with fields `entry`, `injected_content`, `effective_depth`
- `src-tauri/src/service/mod.rs` contains `pub mod lorebook;`
- `activate_lorebook_entries` signature includes `scan_depth: i32, max_recursive_depth: i32, lorebook_token_budget: i64, token_service: &TokenCountService`
- `inject_lorebook_entries` handles all three `LorebookPosition` variants: `BeforeChar`, `AfterChar`, `AtDepth`
- Activation logic respects `LorebookActivation::CharacterOnly` (only scans assistant messages) and `UserOnly` (only scans user messages)
- `cargo test -p revelry-app -- service::lorebook::tests` exits 0
- At least 15 test functions exist in the test module
  </acceptance_criteria>

### Task 6: Update frontend TypeScript types for LorebookEntry new fields

<read_first>

- src/lib/types/index.ts (existing LorebookEntry type)
- src-tauri/src/domain/lorebook.rs (Rust domain model — source of truth for field names)
  </read_first>

<action>
Update the `LorebookEntry` interface in `src/lib/types/index.ts` to add three new fields:

```typescript
export interface LorebookEntry {
  id: string;
  lorebook_id: string;
  keys: string[];
  secondary_keys?: string[];
  content: string;
  position: LorebookPosition;
  depth: number;
  activation_state: LorebookActivation;
  token_budget?: number | null; // NEW — per-entry soft token limit (nullable)
  sort_order: number; // NEW — display/activation ordering
  enabled: boolean; // NEW — activation toggle
  extensions?: Record<string, unknown> | null;
  created_at: string;
  updated_at: string;
}
```

Also add the `LorebookActivationDetail` type for the context bar:

```typescript
export interface LorebookActivationDetail {
  lorebook_name: string;
  entry_name: string;
  content_snippet: string;
  token_cost: number;
  matched_keys?: string[];
}
```

Also extend the `StreamChunk` complete event data and `ContextBudget`:

```typescript
// In StreamChunk complete data, add optional lorebook_details:
// The complete event data gains: lorebook_details?: LorebookActivationDetail[]

// Extend ContextBudget:
export interface ContextBudget {
  tokensUsed: number;
  tokensLimit: number;
  truncatedCount: number;
  lorebookDetails?: LorebookActivationDetail[]; // NEW
}
```

Add `RegexScript` type (used later but define here for completeness):

```typescript
export interface RegexScript {
  id: string;
  character_id: string;
  name: string;
  find_pattern: string;
  replace_pattern: string;
  flags: string;
  sort_order: number;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface RegexTestResult {
  output: string;
  match_count: number;
}
```

</action>

<acceptance_criteria>

- `src/lib/types/index.ts` `LorebookEntry` interface contains `token_budget?: number | null`
- `src/lib/types/index.ts` `LorebookEntry` interface contains `sort_order: number`
- `src/lib/types/index.ts` `LorebookEntry` interface contains `enabled: boolean`
- `src/lib/types/index.ts` contains `export interface LorebookActivationDetail`
- `src/lib/types/index.ts` contains `export interface RegexScript`
- `src/lib/types/index.ts` contains `export interface RegexTestResult`
- `src/lib/types/index.ts` `ContextBudget` interface contains `lorebookDetails?: LorebookActivationDetail[]`
- `bun run check` exits 0 (or `svelte-check` passes)
  </acceptance_criteria>

---

## Verification Criteria (Plan 09-01)

1. All new domain model fields serialize/deserialize correctly between Rust and TypeScript
2. Migration runs cleanly on fresh DB and on existing DB with data
3. LorebookService activation algorithm matches gtk_ref behavior plus tauri `LorebookActivation` enhancements
4. All unit tests pass: `cargo test -p revelry-app` and `bun run check`

## must_haves

- `LorebookEntry` domain model has `token_budget`, `sort_order`, `enabled` fields
- Migration creates `conversation_lorebooks` table and adds new columns to `lorebook_entries`
- `activate_lorebook_entries()` function exists with keyword matching, recursive activation, budget enforcement, and `LorebookActivation` role filtering
- `inject_lorebook_entries()` function exists with `BeforeChar`/`AfterChar`/`AtDepth` position handling
- Frontend `LorebookEntry` TypeScript type matches Rust serde output
- `RegexScript` and `LorebookActivationDetail` TypeScript types defined

---

```yaml
wave: 2
depends_on: ["09-01"]
files_modified:
  - src-tauri/src/service/prompt.rs
  - src-tauri/src/commands/chat.rs
  - src-tauri/src/commands/lorebooks.rs
autonomous: true
```

## Plan 09-02: Integrate Lorebook into PromptEngine and ChatContext

**Requirements:** PRMP-04, PRMP-05, LORB-04

### Task 1: Extend ChatContext to load lorebook entries and conversation lorebooks

<read_first>

- src-tauri/src/commands/chat.rs (existing ChatContext struct and load_chat_context function)
- src-tauri/src/db/mod.rs (DbCommand variants for lorebook loading)
- src-tauri/src/domain/lorebook.rs (LorebookEntry, Lorebook structs)
  </read_first>

<action>
Extend the `ChatContext` struct in `src-tauri/src/commands/chat.rs`:

Add two new fields:

```rust
lorebook_entries: Vec<LorebookEntry>,
character_lorebook_ids: Vec<String>,
conversation_lorebook_ids: Vec<String>,
```

In `load_chat_context()`, after step 5 (provider loading), add steps 6 and 7:

**Step 6: Load character lorebooks and their entries**

```rust
// 6. Load character-assigned lorebooks
let character_lorebooks: Vec<Lorebook> = if let Some(ref ch) = character {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadCharacterLorebooks {
        character_id: ch.id.clone(),
        reply: reply_tx,
    }).map_err(|e| e.to_string())?;
    reply_rx.await.map_err(|e| format!("DB worker dropped: {}", e))?.map_err(|e| e.to_string())?
} else {
    Vec::new()
};
let character_lorebook_ids: Vec<String> = character_lorebooks.iter().map(|lb| lb.id.clone()).collect();
```

**Step 7: Load conversation-assigned lorebooks and merge entries**

```rust
// 7. Load conversation-assigned lorebooks
let conversation_lorebooks: Vec<Lorebook> = {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadConversationLorebooks {
        conversation_id: conversation_id.to_string(),
        reply: reply_tx,
    }).map_err(|e| e.to_string())?;
    reply_rx.await.map_err(|e| format!("DB worker dropped: {}", e))?.map_err(|e| e.to_string())?
};
let conversation_lorebook_ids: Vec<String> = conversation_lorebooks.iter().map(|lb| lb.id.clone()).collect();

// Load entries for all assigned lorebooks (character + conversation)
let all_lorebook_ids: Vec<String> = character_lorebook_ids.iter()
    .chain(conversation_lorebook_ids.iter())
    .cloned()
    .collect();
let lorebook_entries: Vec<LorebookEntry> = if !all_lorebook_ids.is_empty() {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadLorebookEntriesForLorebooks {
        lorebook_ids: all_lorebook_ids,
        reply: reply_tx,
    }).map_err(|e| e.to_string())?;
    reply_rx.await.map_err(|e| format!("DB worker dropped: {}", e))?.map_err(|e| e.to_string())?
} else {
    Vec::new()
};
```

Add these fields to the `ChatContext` construction at the end of the function.

Import `LorebookEntry` and `Lorebook` from `crate::domain::lorebook`.
</action>

<acceptance_criteria>

- `ChatContext` struct in `src-tauri/src/commands/chat.rs` has `lorebook_entries: Vec<LorebookEntry>` field
- `ChatContext` struct has `character_lorebook_ids: Vec<String>` field
- `ChatContext` struct has `conversation_lorebook_ids: Vec<String>` field
- `load_chat_context()` sends `DbCommand::LoadCharacterLorebooks` when character exists
- `load_chat_context()` sends `DbCommand::LoadConversationLorebooks`
- `load_chat_context()` sends `DbCommand::LoadLorebookEntriesForLorebooks` with merged lorebook IDs
- `cargo check -p revelry-app` exits 0
  </acceptance_criteria>

### Task 2: Replace Layer 2 stub in PromptEngine with actual lorebook activation

<read_first>

- src-tauri/src/service/prompt.rs (existing assemble_with_budget with Layer 2 stub)
- src-tauri/src/service/lorebook.rs (new lorebook service with activate/inject)
- src-tauri/src/domain/character.rs (CharacterCardV2 with lorebook_scan_depth and lorebook_recursive_depth fields)
  </read_first>

<action>
Replace the Layer 2 stub in `PromptEngine::assemble_with_budget()` in `src-tauri/src/service/prompt.rs`.

**Replace:**

```rust
// ── Layer 2: Lorebook layer (Phase 9 — skipped for now) ──
// TODO: Phase 9 — activate lorebook entries with lorebook::activate_lorebook_entries()
let active_lorebook_count = 0;
let _ = lorebook_entries; // Suppress unused warning
```

**With:**

```rust
// ── Layer 2: Lorebook layer ──
let scan_depth = character.lorebook_scan_depth;
let max_recursive_depth = character.lorebook_recursive_depth;
let lorebook_budget_fraction = 0.25; // 25% of context window for lorebook
let lorebook_token_budget = (budget as f64 * lorebook_budget_fraction) as i64;

let activated_entries = crate::service::lorebook::activate_lorebook_entries(
    lorebook_entries,
    messages,
    scan_depth,
    max_recursive_depth,
    lorebook_token_budget,
    token_service,
);
let active_lorebook_count = activated_entries.len();
```

**Use `inject_lorebook_entries` service function for position-based injection:**

Instead of duplicating the position-handling logic inline, call the existing `inject_lorebook_entries()` service function from Plan 09-01 Task 5. This avoids duplication and ensures a single source of truth for injection logic.

```rust
// ── Layer 2: Lorebook layer ──
let scan_depth = character.lorebook_scan_depth;
let max_recursive_depth = character.lorebook_recursive_depth;
let lorebook_budget_fraction = 0.25; // 25% of context window for lorebook
let lorebook_token_budget = (budget as f64 * lorebook_budget_fraction) as i64;

let activated_entries = crate::service::lorebook::activate_lorebook_entries(
    lorebook_entries,
    messages,
    scan_depth,
    max_recursive_depth,
    lorebook_token_budget,
    token_service,
);
let active_lorebook_count = activated_entries.len();

// Count lorebook tokens
let lorebook_tokens: usize = activated_entries.iter()
    .map(|e| token_service.count_tokens(&e.injected_content))
    .sum();
tokens_used += lorebook_tokens;

// Apply lorebook injection via service function
// First, truncate history BEFORE injecting depth-based entries.
// This prevents depth-injected system messages at old positions from
// being silently removed by budget truncation.
let truncated_history = crate::service::prompt::truncate_history_oldest_first(
    history_messages,
    budget - tokens_used,
    token_service,
);

// Now inject lorebook entries into the already-truncated history
let mut history_messages_for_inject = truncated_history;
let (preamble_before, preamble_after) = crate::service::lorebook::inject_lorebook_entries(
    &sticky_prompt,      // preamble_before_char (BeforeChar content appended before this)
    &String::new(),      // preamble_after_char (AfterChar content appended after this; persona added separately)
    &activated_entries,
    &mut history_messages_for_inject, // depth entries injected into truncated history
);
```

Then in the final preamble assembly:

```rust
let mut preamble_parts = Vec::new();
if !preamble_before.is_empty() {
    preamble_parts.push(preamble_before);
}
if !preamble_after.is_empty() {
    preamble_parts.push(preamble_after);
}
if !persona_block.is_empty() {
    preamble_parts.push(persona_block);
}
if let Some(post) = post_history {
    preamble_parts.push(post);
}
```

**Key ordering fix:** Depth-based lorebook entries are injected AFTER truncation, not before. This prevents the oldest-first truncation from silently removing depth-injected system messages at old positions. The tradeoff is that depth offsets are relative to the truncated history length, which is the standard behavior in SillyTavern/gtk_ref.

**Build `AssembledPrompt` with lorebook details for the Complete chunk:**

After activation, build the `lorebook_details` payload that will be sent with `StreamChunk::Complete`:

```rust
// Build lorebook activation details for the Complete chunk
// lorebook_names: HashMap<String, String> built earlier from load_lorebooks_for_character + load_lorebooks_for_conversation
let lorebook_details: Vec<LorebookActivationDetail> = activated_entries.iter().map(|e| {
    LorebookActivationDetail {
        lorebook_name: lorebook_names.get(&e.entry.lorebook_id).cloned().unwrap_or_else(|| "unknown".to_string()),
        entry_name: e.entry.keys.first().cloned().unwrap_or_default(),
        content_snippet: e.injected_content.chars().take(30).collect(),
        token_cost: token_service.count_tokens(&e.injected_content),
        matched_keys: Some(e.matched_keys.clone()),
    }
}).collect();
```

Add `lorebook_details: Vec<LorebookActivationDetail>` to the `AssembledPrompt` struct (or the return value of `assemble_with_budget`). This data is then passed through the chat command to `AiService::stream_chat` so it can be included in the `StreamChunk::Complete` payload.

**Add `LorebookActivationDetail` to the Rust domain model** (in `src-tauri/src/domain/lorebook.rs`):

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LorebookActivationDetail {
    pub lorebook_name: String,
    pub entry_name: String,
    pub content_snippet: String,
    pub token_cost: usize,
    pub matched_keys: Option<Vec<String>>,
}
```

Add `use crate::domain::lorebook::LorebookActivationDetail;` to imports in prompt.rs.

**Update tests:** Modify the existing budget test to pass `&[]` for lorebook entries (already the case). Add a new test:

- `test_assemble_with_budget_lorebook_entries_activated` — create a character with `lorebook_scan_depth=2`, create lorebook entries with a keyword, pass them to `assemble_with_budget`, assert `active_lorebook_count > 0` and the keyword content appears in the preamble.
  </action>

<acceptance_criteria>

- `src-tauri/src/service/prompt.rs` does NOT contain `// TODO: Phase 9` or the lorebook stub comment
- `src-tauri/src/service/prompt.rs` calls `crate::service::lorebook::activate_lorebook_entries`
- `src-tauri/src/service/prompt.rs` calls `crate::service::lorebook::inject_lorebook_entries` (no inline position-handling duplication)
- `src-tauri/src/service/prompt.rs` injects depth-based entries AFTER truncation (not before)
- Lorebook token usage is added to `tokens_used`
- `active_lorebook_count` reflects the number of activated entries (not hardcoded 0)
- `AssembledPrompt` struct includes `lorebook_details: Vec<LorebookActivationDetail>`
- `src-tauri/src/domain/lorebook.rs` contains `LorebookActivationDetail` struct
- `cargo test -p revelry-app -- service::prompt::tests` exits 0
  </acceptance_criteria>

### Task 3: Add conversation lorebook Tauri commands

<read_first>

- src-tauri/src/commands/lorebooks.rs (existing lorebook commands pattern)
- src-tauri/src/db/mod.rs (new DbCommand variants for conversation lorebooks)
- src-tauri/src/lib.rs (invoke_handler registration pattern)
  </read_first>

<action>
Add three new Tauri commands in `src-tauri/src/commands/lorebooks.rs`:

```rust
#[tauri::command]
pub async fn assign_lorebook_to_conversation(
    db: State<'_, DbWorker>,
    conversation_id: String,
    lorebook_id: String,
    sort_order: i32,
) -> Result<(), String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::AssignLorebookToConversation {
        conversation_id,
        lorebook_id,
        sort_order,
        reply: reply_tx,
    }).map_err(|e| e.to_string())?;
    reply_rx.await.map_err(|e| format!("DB worker dropped: {}", e))?.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_lorebook_from_conversation(
    db: State<'_, DbWorker>,
    conversation_id: String,
    lorebook_id: String,
) -> Result<(), String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::RemoveLorebookFromConversation {
        conversation_id,
        lorebook_id,
        reply: reply_tx,
    }).map_err(|e| e.to_string())?;
    reply_rx.await.map_err(|e| format!("DB worker dropped: {}", e))?.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_conversation_lorebooks(
    db: State<'_, DbWorker>,
    conversation_id: String,
) -> Result<Vec<Lorebook>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadConversationLorebooks {
        conversation_id,
        reply: reply_tx,
    }).map_err(|e| e.to_string())?;
    reply_rx.await.map_err(|e| format!("DB worker dropped: {}", e))?.map_err(|e| e.to_string())
}
```

Add `use crate::domain::lorebook::Lorebook;` import if not already present.

Register in `src-tauri/src/lib.rs` invoke_handler:

```rust
commands::lorebooks::assign_lorebook_to_conversation,
commands::lorebooks::remove_lorebook_from_conversation,
commands::lorebooks::get_conversation_lorebooks,
```

</action>

<acceptance_criteria>

- `src-tauri/src/commands/lorebooks.rs` contains `pub async fn assign_lorebook_to_conversation` with `conversation_id: String, lorebook_id: String, sort_order: i32` params
- `src-tauri/src/commands/lorebooks.rs` contains `pub async fn remove_lorebook_from_conversation` with `conversation_id: String, lorebook_id: String` params
- `src-tauri/src/commands/lorebooks.rs` contains `pub async fn get_conversation_lorebooks` with `conversation_id: String` param returning `Result<Vec<Lorebook>, String>`
- `src-tauri/src/lib.rs` `invoke_handler` contains all three new command registrations
- `cargo check -p revelry-app` exits 0
  </acceptance_criteria>

---

## Verification Criteria (Plan 09-02)

1. ChatContext loads both character and conversation lorebook entries
2. PromptEngine Layer 2 activates lorebook entries and injects them at correct positions
3. Lorebook tokens are counted in the budget
4. Conversation lorebook CRUD Tauri commands are callable from frontend
5. `cargo test -p revelry-app` exits 0

## must_haves

- `load_chat_context()` loads lorebook entries from character + conversation lorebooks
- `assemble_with_budget()` calls `activate_lorebook_entries()` and `inject_lorebook_entries()` (no inline duplication)
- Depth-based lorebook entries injected AFTER truncation
- `AssembledPrompt` includes `lorebook_details: Vec<LorebookActivationDetail>` for Complete chunk
- `active_lorebook_count` is no longer hardcoded to 0
- Lorebook tokens are included in `tokens_used`
- Three new Tauri commands for conversation-lorebook junction operations

---

```yaml
wave: 1
depends_on: []
files_modified:
  - src-tauri/src/domain/regex_script.rs
  - src-tauri/src/db/regex_scripts.rs
  - src-tauri/src/db/mod.rs
  - src-tauri/src/service/regex.rs
  - src-tauri/src/service/mod.rs
  - src-tauri/src/commands/regex.rs
  - src-tauri/src/commands/mod.rs
  - src-tauri/src/lib.rs
autonomous: true
```

## Plan 09-03: Port RegexService with Validation — Domain, DB, Service, and Commands

**Requirements:** REGX-01, REGX-02, REGX-03, REGX-05

### Task 1: Create domain/regex_script.rs — RegexScript model

<read_first>

- gtk_ref/src/domain/regex_script.rs (reference: RegexScript struct, from_extensions_json/to_extensions_json for SillyTavern card compat)
- src-tauri/src/db/migrations.rs (existing regex_scripts table schema)
  </read_first>

<action>
Create `src-tauri/src/domain/regex_script.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RegexScript {
    pub id: String,
    pub character_id: String,
    pub name: String,
    pub find_pattern: String,
    pub replace_pattern: String,
    pub flags: String,
    pub sort_order: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RegexScript {
    pub fn new(character_id: String, name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            character_id,
            name,
            find_pattern: String::new(),
            replace_pattern: String::new(),
            flags: "gi".to_string(),
            sort_order: 0,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Deserialize from SillyTavern extensions JSON (card import compat).
    /// Mirrors gtk_ref `from_extensions_json` — extracts regex entries from
    /// the `data.extensions.regex_scripts` array in a character card.
    pub fn from_extensions_json(value: &serde_json::Value, character_id: &str) -> Vec<Self> {
        value.get("regex_scripts")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().filter_map(|entry| {
                    Some(Self {
                        id: entry.get("id").and_then(|v| v.as_str()).unwrap_or(&Uuid::new_v4().to_string()).to_string(),
                        character_id: character_id.to_string(),
                        name: entry.get("scriptName").and_then(|v| v.as_str()).unwrap_or("Unnamed").to_string(),
                        find_pattern: entry.get("findRegex").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        replace_pattern: entry.get("replaceString").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        flags: entry.get("flags").and_then(|v| v.as_str()).unwrap_or("gi").to_string(),
                        sort_order: entry.get("sort_order").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                        enabled: entry.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    })
                }).collect())
            .unwrap_or_default()
    }

    /// Serialize to SillyTavern extensions JSON format (card export compat).
    /// Mirrors gtk_ref `to_extensions_json` — produces the `regex_scripts` array
    /// that goes inside `data.extensions`.
    pub fn to_extensions_json(scripts: &[Self]) -> serde_json::Value {
        serde_json::Value::Array(
            scripts.iter().map(|s| {
                serde_json::json!({
                    "id": s.id,
                    "scriptName": s.name,
                    "findRegex": s.find_pattern,
                    "replaceString": s.replace_pattern,
                    "flags": s.flags,
                    "sort_order": s.sort_order,
                    "enabled": s.enabled,
                })
            }).collect()
        )
    }
}
```

Add `pub mod regex_script;` to `src-tauri/src/domain/mod.rs` (or ensure domain module exports it).

Add tests:

- `test_regex_script_new` — verify defaults: `flags == "gi"`, `sort_order == 0`, `enabled == true`, `find_pattern.is_empty()`
- `test_regex_script_serde_roundtrip` — serialize/deserialize, verify equality
- `test_from_extensions_json` — parse a JSON value with `regex_scripts` array, verify all fields mapped correctly (SillyTavern `scriptName` → `name`, `findRegex` → `find_pattern`, etc.)
- `test_from_extensions_json_empty` — parse JSON without `regex_scripts` key, verify empty Vec returned
- `test_to_extensions_json` — serialize a Vec of scripts, verify output contains `scriptName`, `findRegex`, `replaceString` keys (not Rust field names)
- `test_extensions_json_roundtrip` — `from_extensions_json` → `to_extensions_json` → `from_extensions_json`, verify data preserved (IDs and timestamps differ, but logic fields match)
  </action>

<acceptance_criteria>

- `src-tauri/src/domain/regex_script.rs` exists
- Contains `pub struct RegexScript` with fields: `id, character_id, name, find_pattern, replace_pattern, flags, sort_order, enabled, created_at, updated_at`
- `RegexScript::new()` defaults: `flags = "gi"`, `sort_order = 0`, `enabled = true`
- `src-tauri/src/domain/regex_script.rs` contains `pub fn from_extensions_json(value: &serde_json::Value, character_id: &str) -> Vec<Self>`
- `src-tauri/src/domain/regex_script.rs` contains `pub fn to_extensions_json(scripts: &[Self]) -> serde_json::Value`
- `cargo test -p revelry-app -- domain::regex_script::tests` exits 0
  </acceptance_criteria>

### Task 2: Implement regex_scripts DB CRUD

<read_first>

- src-tauri/src/db/regex_scripts.rs (currently empty stub)
- src-tauri/src/db/lorebook.rs (pattern for CRUD functions)
- src-tauri/src/db/models.rs (pattern for row mappers)
  </read_first>

<action>
Replace the stub in `src-tauri/src/db/regex_scripts.rs` with full CRUD operations.

**Add row mapper in `src-tauri/src/db/models.rs`:**

```rust
pub fn row_to_regex_script(row: &Row) -> rusqlite::Result<RegexScript> {
    let created_at_str: String = row.get("created_at")?;
    let updated_at_str: String = row.get("updated_at")?;
    let created_at = parse_timestamp(&created_at_str)?;
    let updated_at = parse_timestamp(&updated_at_str)?;

    Ok(RegexScript {
        id: row.get("id")?,
        character_id: row.get("character_id")?,
        name: row.get("name")?,
        find_pattern: row.get("find_pattern")?,
        replace_pattern: row.get("replace_pattern")?,
        flags: row.get("flags")?,
        sort_order: row.get("sort_order")?,
        enabled: row.get::<_, i32>("enabled")? != 0,
        created_at,
        updated_at,
    })
}
```

**Implement CRUD in `src-tauri/src/db/regex_scripts.rs`:**

1. `load_regex_scripts(conn: &Connection, character_id: &str) -> Result<Vec<RegexScript>>` — `SELECT id, character_id, name, find_pattern, replace_pattern, flags, sort_order, enabled, created_at, updated_at FROM regex_scripts WHERE character_id = ?1 ORDER BY sort_order ASC`
2. `load_all_regex_scripts(conn: &Connection) -> Result<Vec<RegexScript>>` — load all scripts (for testing)
3. `insert_regex_script(conn: &Connection, script: &RegexScript) -> Result<()>` — INSERT with all fields, `enabled` stored as `i32` (1/0)
4. `update_regex_script(conn: &Connection, script: &RegexScript) -> Result<()>` — UPDATE SET all mutable fields WHERE id
5. `delete_regex_script(conn: &Connection, id: &str) -> Result<()>` — DELETE WHERE id

**Add tests:**

- `test_insert_and_load_regex_scripts` — insert 2 scripts for same character, verify both loaded
- `test_load_regex_scripts_filters_by_character` — insert scripts for 2 different characters, verify isolation
- `test_update_regex_script` — insert, update name and pattern, verify updated
- `test_delete_regex_script` — insert, delete, verify empty
- `test_regex_script_enabled_stored_as_integer` — insert with `enabled: false`, verify loaded as `false`
  </action>

<acceptance_criteria>

- `src-tauri/src/db/regex_scripts.rs` contains `pub fn load_regex_scripts` with `(conn: &Connection, character_id: &str) -> Result<Vec<RegexScript>>`
- `src-tauri/src/db/regex_scripts.rs` contains `pub fn insert_regex_script`
- `src-tauri/src/db/regex_scripts.rs` contains `pub fn update_regex_script`
- `src-tauri/src/db/regex_scripts.rs` contains `pub fn delete_regex_script`
- `src-tauri/src/db/models.rs` contains `pub fn row_to_regex_script`
- `cargo test -p revelry-app -- db::regex_scripts::tests` exits 0
  </acceptance_criteria>

### Task 3: Add DbCommand variants for regex scripts

<read_first>

- src-tauri/src/db/mod.rs (existing DbCommand enum and handle_command)
  </read_first>

<action>
Add three new `DbCommand` variants:

```rust
LoadRegexScripts {
    character_id: String,
    reply: oneshot::Sender<anyhow::Result<Vec<RegexScript>>>,
},
SaveRegexScript {
    script: RegexScript,
    reply: oneshot::Sender<anyhow::Result<RegexScript>>,
},
DeleteRegexScript {
    id: String,
    reply: oneshot::Sender<anyhow::Result<()>>,
},
```

Add match arms in `DbWorker::handle_command`:

- `LoadRegexScripts` → `regex_scripts::load_regex_scripts(conn, &character_id)`
- `SaveRegexScript` → check existence, then `insert_regex_script` or `update_regex_script`
- `DeleteRegexScript` → `regex_scripts::delete_regex_script(conn, &id)`

Add `use crate::domain::regex_script::RegexScript;` import.
</action>

<acceptance_criteria>

- `src-tauri/src/db/mod.rs` contains `LoadRegexScripts` variant with `character_id: String`
- `src-tauri/src/db/mod.rs` contains `SaveRegexScript` variant with `script: RegexScript`
- `src-tauri/src/db/mod.rs` contains `DeleteRegexScript` variant with `id: String`
- All three have match arms in `handle_command`
- `cargo check -p revelry-app` exits 0
  </acceptance_criteria>

### Task 4: Create service/regex.rs — port RegexService from gtk_ref

<read_first>

- gtk_ref/src/service/regex.rs (reference: validate_regex_pattern, validate_regex_flags, apply_regex_scripts, apply_single_regex)
- src-tauri/Cargo.toml (verify `regex` crate is a dependency)
  </read_first>

<action>
Create `src-tauri/src/service/regex.rs`. Port all four functions from gtk_ref.

```rust
use regex::RegexBuilder;
use crate::domain::regex_script::RegexScript;

/// Validates a regex pattern for ReDoS vulnerability.
/// Uses regex::RegexBuilder::size_limit(1_000_000) to cap backtracking.
/// Returns compiled Regex if safe, or error message if dangerous/invalid.
pub fn validate_regex_pattern(pattern: &str, flags: &str) -> Result<regex::Regex, String> {
    let mut builder = RegexBuilder::new(pattern);
    builder.size_limit(1_000_000);
    if flags.contains('i') { builder.case_insensitive(true); }
    if flags.contains('m') { builder.multi_line(true); }
    if flags.contains('s') { builder.dot_matches_new_line(true); }
    builder.build().map_err(|e| e.to_string())
}

/// Validates that flags contain only valid characters (i, g, m, s).
pub fn validate_regex_flags(flags: &str) -> Result<(), String> {
    for c in flags.chars() {
        if !matches!(c, 'i' | 'g' | 'm' | 's') {
            return Err(format!("Invalid flag '{}'. Only i, g, m, s are allowed.", c));
        }
    }
    Ok(())
}

/// Applies a list of regex scripts to input text.
/// Scripts run in sort_order sequence (pipe chain).
/// Disabled scripts and invalid patterns are skipped.
pub fn apply_regex_scripts(text: &str, scripts: &[RegexScript]) -> String {
    let mut sorted: Vec<&RegexScript> = scripts.iter()
        .filter(|s| s.enabled)
        .collect();
    sorted.sort_by_key(|s| s.sort_order);

    let mut result = text.to_string();
    for script in sorted {
        match validate_regex_pattern(&script.find_pattern, &script.flags) {
            Ok(re) => {
                let is_global = script.flags.contains('g');
                if is_global {
                    result = re.replace_all(&result, script.replace_pattern.as_str()).to_string();
                } else {
                    result = re.replace(&result, script.replace_pattern.as_str()).to_string();
                }
            }
            Err(_) => {
                tracing::warn!("Skipping regex script '{}' — invalid pattern", script.name);
            }
        }
    }
    result
}

/// Applies a single regex find/replace (for test panel preview).
pub fn apply_single_regex(text: &str, pattern: &str, replace: &str, flags: &str) -> Result<String, String> {
    let re = validate_regex_pattern(pattern, flags)?;
    let is_global = flags.contains('g');
    if is_global {
        Ok(re.replace_all(text, replace).to_string())
    } else {
        Ok(re.replace(text, replace).to_string())
    }
}
```

Add `pub mod regex;` to `src-tauri/src/service/mod.rs`.

Port the test suite from gtk_ref, adapting to use `crate::domain::regex_script::RegexScript`:

- `test_validate_safe_pattern`
- `test_validate_redos_pattern` — deeply nested pattern exceeding size_limit
- `test_validate_invalid_syntax`
- `test_apply_two_enabled_scripts_in_order`
- `test_apply_skips_disabled_scripts`
- `test_apply_global_and_case_insensitive`
- `test_apply_capture_group_replacement`
- `test_validate_flags_valid`
- `test_validate_flags_invalid`
- `test_apply_single_regex`
- `test_apply_single_regex_invalid_pattern`
- `test_apply_non_global_replaces_once`
- `test_apply_empty_scripts_list`
- `test_pipe_chain_output_becomes_input`

Use `chrono::Utc::now()` for timestamps instead of `Local::now().naive_local()`.
</action>

<acceptance_criteria>

- `src-tauri/src/service/regex.rs` exists
- Contains `pub fn validate_regex_pattern(pattern: &str, flags: &str) -> Result<regex::Regex, String>` with `size_limit(1_000_000)`
- Contains `pub fn validate_regex_flags(flags: &str) -> Result<(), String>`
- Contains `pub fn apply_regex_scripts(text: &str, scripts: &[RegexScript]) -> String`
- Contains `pub fn apply_single_regex(text: &str, pattern: &str, replace: &str, flags: &str) -> Result<String, String>`
- `src-tauri/src/service/mod.rs` contains `pub mod regex;`
- `cargo test -p revelry-app -- service::regex::tests` exits 0
- At least 14 test functions in the regex test module
  </acceptance_criteria>

### Task 5: Create Tauri commands for regex scripts

<read_first>

- src-tauri/src/commands/lorebooks.rs (pattern for CRUD commands)
- src-tauri/src/commands/keywords.rs (pattern for test commands)
- src-tauri/src/lib.rs (invoke_handler registration)
  </read_first>

<action>
Create `src-tauri/src/commands/regex.rs` with these commands:

```rust
#[tauri::command]
pub async fn get_regex_scripts(
    db: State<'_, DbWorker>,
    character_id: String,
) -> Result<Vec<RegexScript>, String> { ... }

#[tauri::command]
pub async fn save_regex_script(
    db: State<'_, DbWorker>,
    script: RegexScript,
) -> Result<RegexScript, String> {
    // Validate pattern before saving
    crate::service::regex::validate_regex_flags(&script.flags)
        .map_err(|e| e.to_string())?;
    crate::service::regex::validate_regex_pattern(&script.find_pattern, &script.flags)
        .map_err(|e| e.to_string())?;
    // Then save via DbCommand
    ...
}

#[tauri::command]
pub async fn delete_regex_script(
    db: State<'_, DbWorker>,
    id: String,
) -> Result<(), String> { ... }

#[tauri::command]
pub async fn apply_single_regex(
    pattern: String,
    replace_pattern: String,
    flags: String,
    sample_text: String,
) -> Result<serde_json::Value, String> {
    // Validate pattern
    crate::service::regex::validate_regex_flags(&flags)
        .map_err(|e| e.to_string())?;
    let re = crate::service::regex::validate_regex_pattern(&pattern, &flags)
        .map_err(|e| e.to_string())?;
    // Apply and count matches
    let is_global = flags.contains('g');
    if is_global {
        let result = re.replace_all(&sample_text, replace_pattern.as_str()).to_string();
        let match_count = re.find_iter(&sample_text).count();
        Ok(serde_json::json!({ "output": result, "match_count": match_count }))
    } else {
        let result = re.replace(&sample_text, replace_pattern.as_str()).to_string();
        let match_count = if re.is_match(&sample_text) { 1 } else { 0 };
        Ok(serde_json::json!({ "output": result, "match_count": match_count }))
    }
}

#[tauri::command]
pub async fn validate_regex_pattern(
    pattern: String,
    flags: String,
) -> Result<(), String> {
    crate::service::regex::validate_regex_flags(&flags)
        .map_err(|e| e.to_string())?;
    crate::service::regex::validate_regex_pattern(&pattern, &flags)
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

Add `pub mod regex;` to `src-tauri/src/commands/mod.rs`.

Register all five commands in `src-tauri/src/lib.rs` invoke_handler:

```rust
commands::regex::get_regex_scripts,
commands::regex::save_regex_script,
commands::regex::delete_regex_script,
commands::regex::apply_single_regex,
commands::regex::validate_regex_pattern,
```

</action>

<acceptance_criteria>

- `src-tauri/src/commands/regex.rs` exists
- Contains `pub async fn get_regex_scripts` returning `Result<Vec<RegexScript>, String>`
- Contains `pub async fn save_regex_script` that validates pattern and flags before saving
- Contains `pub async fn delete_regex_script`
- Contains `pub async fn apply_single_regex` with params `pattern, replace_pattern, flags, sample_text` returning JSON with `output` and `match_count`
- Contains `pub async fn validate_regex_pattern` returning `Result<(), String>`
- `src-tauri/src/commands/mod.rs` contains `pub mod regex;`
- `src-tauri/src/lib.rs` invoke_handler contains all 5 regex command registrations
- `cargo check -p revelry-app` exits 0
  </acceptance_criteria>

---

## Verification Criteria (Plan 09-03)

1. RegexScript domain model matches the existing DB schema
2. DB CRUD operations work for all script fields including `enabled` (stored as INTEGER)
3. `validate_regex_pattern` uses `size_limit(1_000_000)` for ReDoS protection
4. `apply_regex_scripts` applies pipe chain in sort_order, skipping disabled/invalid
5. `save_regex_script` validates before persisting (REGX-05)
6. All Tauri commands callable from frontend

## must_haves

- `RegexScript` domain model with all fields from the existing DB schema
- `RegexScript` includes `from_extensions_json()` and `to_extensions_json()` for SillyTavern card import/export compat
- DB CRUD for regex scripts (load, insert, update, delete)
- `service/regex.rs` with `validate_regex_pattern`, `validate_regex_flags`, `apply_regex_scripts`, `apply_single_regex`
- Tauri commands: `get_regex_scripts`, `save_regex_script`, `delete_regex_script`, `apply_single_regex`, `validate_regex_pattern`
- `save_regex_script` validates pattern before persisting
- All regex tests pass

---

```yaml
wave: 2
depends_on: ["09-03"]
files_modified:
  - src-tauri/src/commands/chat.rs
  - src-tauri/src/ai/mod.rs
autonomous: true
```

## Plan 09-04: Integrate Regex Scripts into AI Response Pipeline

**Requirements:** REGX-04

### Task 1: Add regex scripts to ChatContext and apply on stream completion

<read_first>

- src-tauri/src/commands/chat.rs (existing ChatContext, assemble_prompt, and stream flow)
- src-tauri/src/service/regex.rs (apply_regex_scripts function)
- src-tauri/src/ai/mod.rs (AiService::stream_chat — where StreamChunk::Complete is sent)
  </read_first>

<action>
**Step 1: Add regex scripts to ChatContext**

Add field to `ChatContext`:

```rust
regex_scripts: Vec<RegexScript>,
```

In `load_chat_context()`, add after step 7 (lorebook loading):

```rust
// 8. Load regex scripts for the character
let regex_scripts: Vec<RegexScript> = if let Some(ref ch) = character {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadRegexScripts {
        character_id: ch.id.clone(),
        reply: reply_tx,
    }).map_err(|e| e.to_string())?;
    reply_rx.await.map_err(|e| format!("DB worker dropped: {}", e))?.map_err(|e| e.to_string())?
} else {
    Vec::new()
};
```

Add `regex_scripts` to `ChatContext` construction.

**Step 2: Apply regex transforms before sending StreamChunk::Complete**

The regex transforms must be applied in `AiService::stream_chat()` when the stream completes. Read `src-tauri/src/ai/mod.rs` to find where `StreamChunk::Complete` is built and sent.

If `stream_chat` is defined in `src-tauri/src/ai/mod.rs`, the completion path builds a `StreamChunk::Complete` with the full content. Before sending this chunk, apply regex:

The cleanest approach: pass `regex_scripts` into `stream_chat` as a parameter. Before constructing the `Complete` chunk, apply `service::regex::apply_regex_scripts(&full_content, &regex_scripts)`.

Update `AiService::stream_chat` signature to accept `regex_scripts: Vec<RegexScript>`.

Update all call sites in `src-tauri/src/commands/chat.rs`:

- `start_chat`: pass `ctx.regex_scripts.clone()` to `stream_chat`
- `regenerate`: pass `ctx.regex_scripts.clone()`
- `edit_and_resend`: pass `ctx.regex_scripts.clone()`

In the `stream_chat` function, before the `Complete` chunk:

```rust
// Apply regex transforms to the completed content
let transformed_content = if !regex_scripts.is_empty() {
    crate::service::regex::apply_regex_scripts(&accumulated_content, &regex_scripts)
} else {
    accumulated_content
};
```

Then use `transformed_content` in the `StreamChunk::Complete` data and when saving the message to DB.

**Step 3: Include lorebook_details in StreamChunk::Complete payload**

The `AssembledPrompt` struct (from Plan 09-02 Task 2) now carries `lorebook_details: Vec<LorebookActivationDetail>`. When `assemble_with_budget` is called in `start_chat`/`regenerate`/`edit_and_resend`, the returned `lorebook_details` must be passed through to `stream_chat` and included in the `StreamChunk::Complete` chunk.

Update `AiService::stream_chat` signature to also accept `lorebook_details: Vec<LorebookActivationDetail>`.

In the `StreamChunk::Complete` construction, include:

```rust
let complete_data = serde_json::json!({
    "content": transformed_content,
    "lorebookDetails": lorebook_details,
});
```

This makes lorebook activation details available to the frontend context bar (Plan 09-05 Task 2).

**Import:** Add `use crate::domain::lorebook::LorebookActivationDetail;` in chat.rs and ai/mod.rs as needed.

**Import:** Add `use crate::domain::regex_script::RegexScript;` in chat.rs.
</action>

<acceptance_criteria>

- `ChatContext` struct has `regex_scripts: Vec<RegexScript>` field
- `load_chat_context()` sends `DbCommand::LoadRegexScripts` when character exists
- `AiService::stream_chat` accepts `regex_scripts: Vec<RegexScript>` parameter
- All three call sites (`start_chat`, `regenerate`, `edit_and_resend`) pass `ctx.regex_scripts.clone()` to `stream_chat`
- Before `StreamChunk::Complete` is sent, `apply_regex_scripts` is called on the full content when `!regex_scripts.is_empty()`
- The `Complete` chunk `content` field contains the regex-transformed text
- The saved message content contains the regex-transformed text
- `AiService::stream_chat` accepts `lorebook_details: Vec<LorebookActivationDetail>` parameter
- All three call sites pass `ctx.assembled_prompt.lorebook_details.clone()` (or equivalent) to `stream_chat`
- `StreamChunk::Complete` payload includes `lorebookDetails` field
- `cargo check -p revelry-app` exits 0
  </acceptance_criteria>

1. Regex scripts are loaded as part of ChatContext
2. AI responses are regex-transformed before being sent to the frontend and saved to DB
3. During streaming, raw content displays (no regex applied mid-stream)
4. On completion, the final content includes all regex transformations in sort_order

## must_haves

- `ChatContext.regex_scripts` loaded for the character
- `stream_chat` applies `apply_regex_scripts` to the complete content before `StreamChunk::Complete`
- `stream_chat` includes `lorebook_details` in the `StreamChunk::Complete` payload
- Transformed content is what gets saved to the database and sent to the frontend
- All three chat command call sites pass regex scripts and lorebook_details

---

```yaml
wave: 3
depends_on: ["09-01", "09-02", "09-03", "09-04"]
files_modified:
  - src/lib/data/api.ts
  - src/lib/stores/app.svelte.ts
  - src/lib/components/chat/ChatHeader.svelte
  - src/lib/components/chat/MessageInput.svelte
  - src/lib/components/chat/ConversationLorebookPanel.svelte
  - src/lib/components/chat/LorebookDetailPanel.svelte
  - src/lib/components/characters/RegexScriptEditor.svelte
  - src/lib/components/characters/CharacterEditorDialog.svelte
autonomous: false
```

## Plan 09-05: Wire Lorebook and Regex to Editor UIs and Conversation Flow

**Requirements:** LORB-01, LORB-02, LORB-03, LORB-04, LORB-05, REGX-01, REGX-02, REGX-03, REGX-04, REGX-05

### Task 1: Add API wrappers for regex and conversation lorebook commands

<read_first>

- src/lib/data/api.ts (existing invoke wrappers, pattern for new commands)
- src/lib/types/index.ts (RegexScript, RegexTestResult, LorebookActivationDetail types)
  </read_first>

<action>
Add the following API wrapper functions to `src/lib/data/api.ts`:

```typescript
// ── Regex scripts ──────────────────────────────────────────────────────────

export async function getRegexScripts(
  characterId: string,
): Promise<RegexScript[]> {
  return invoke<RegexScript[]>("get_regex_scripts", { characterId });
}

export async function saveRegexScript(
  script: RegexScript,
): Promise<RegexScript> {
  return invoke<RegexScript>("save_regex_script", { script });
}

export async function deleteRegexScript(id: string): Promise<void> {
  return invoke<void>("delete_regex_script", { id });
}

export async function applySingleRegex(
  pattern: string,
  replacePattern: string,
  flags: string,
  sampleText: string,
): Promise<RegexTestResult> {
  return invoke<RegexTestResult>("apply_single_regex", {
    pattern,
    replacePattern,
    flags,
    sampleText,
  });
}

export async function validateRegexPattern(
  pattern: string,
  flags: string,
): Promise<void> {
  return invoke<void>("validate_regex_pattern", { pattern, flags });
}

// ── Conversation-lorebook junction ──────────────────────────────────────

export async function getConversationLorebooks(
  conversationId: string,
): Promise<Lorebook[]> {
  return invoke<Lorebook[]>("get_conversation_lorebooks", {
    conversationId,
  });
}

export async function assignLorebookToConversation(
  conversationId: string,
  lorebookId: string,
  sortOrder: number,
): Promise<void> {
  return invoke<void>("assign_lorebook_to_conversation", {
    conversationId,
    lorebookId,
    sortOrder,
  });
}

export async function removeLorebookFromConversation(
  conversationId: string,
  lorebookId: string,
): Promise<void> {
  return invoke<void>("remove_lorebook_from_conversation", {
    conversationId,
    lorebookId,
  });
}

export async function getCharacterLorebooks(
  characterId: string,
): Promise<Lorebook[]> {
  return invoke<Lorebook[]>("get_character_lorebooks", { characterId });
}
```

Also add the import for `RegexScript` and `RegexTestResult` at the top of the file.
</action>

<acceptance_criteria>

- `src/lib/data/api.ts` contains `export async function getRegexScripts(characterId: string): Promise<RegexScript[]>`
- `src/lib/data/api.ts` contains `export async function saveRegexScript(script: RegexScript): Promise<RegexScript>`
- `src/lib/data/api.ts` contains `export async function deleteRegexScript(id: string): Promise<void>`
- `src/lib/data/api.ts` contains `export async function applySingleRegex(pattern, replacePattern, flags, sampleText): Promise<RegexTestResult>`
- `src/lib/data/api.ts` contains `export async function validateRegexPattern(pattern, flags): Promise<void>`
- `src/lib/data/api.ts` contains `export async function getConversationLorebooks(conversationId): Promise<Lorebook[]>`
- `src/lib/data/api.ts` contains `export async function assignLorebookToConversation(conversationId, lorebookId, sortOrder): Promise<void>`
- `src/lib/data/api.ts` contains `export async function removeLorebookFromConversation(conversationId, lorebookId): Promise<void>`
- `src/lib/data/api.ts` contains `export async function getCharacterLorebooks(characterId): Promise<Lorebook[]>`
- `bun run check` exits 0
  </acceptance_criteria>

### Task 2: Extend AppState with regex scripts and conversation lorebook state/actions

<read_first>

- src/lib/stores/app.svelte.ts (existing AppState class)
- src/lib/data/api.ts (new API wrappers to import)
  </read_first>

<action>
Add the following to `AppState` in `src/lib/stores/app.svelte.ts`:

**New state fields:**

```typescript
regex_scripts = $state<Map<string, RegexScript[]>>(new Map());
conversation_lorebooks = $state<Lorebook[]>([]);
character_lorebooks = $state<Lorebook[]>([]);
lorebook_detail_open = $state<boolean>(false);
lorebook_details = $state<LorebookActivationDetail[]>([]);
```

**New actions:**

```typescript
async loadRegexScripts(characterId: string): Promise<void> {
  if (this.regex_scripts.has(characterId)) return; // cached
  try {
    const scripts = await apiGetRegexScripts(characterId);
    this.regex_scripts.set(characterId, scripts);
    // Trigger reactivity by creating new Map
    this.regex_scripts = new Map(this.regex_scripts);
  } catch (err) {
    console.error("Failed to load regex scripts:", err);
  }
}

async saveRegexScript(script: RegexScript): Promise<RegexScript> {
  try {
    const saved = await apiSaveRegexScript(script);
    const charScripts = this.regex_scripts.get(saved.character_id) ?? [];
    const updated = charScripts.map(s => s.id === saved.id ? saved : s);
    if (!charScripts.find(s => s.id === saved.id)) {
      updated.push(saved);
    }
    this.regex_scripts.set(saved.character_id, updated);
    this.regex_scripts = new Map(this.regex_scripts);
    return saved;
  } catch (err) {
    console.error("Failed to save regex script:", err);
    throw err;
  }
}

async deleteRegexScript(id: string, characterId: string): Promise<void> {
  try {
    await apiDeleteRegexScript(id);
    const charScripts = this.regex_scripts.get(characterId) ?? [];
    this.regex_scripts.set(characterId, charScripts.filter(s => s.id !== id));
    this.regex_scripts = new Map(this.regex_scripts);
  } catch (err) {
    console.error("Failed to delete regex script:", err);
    throw err;
  }
}

async testRegexPattern(pattern: string, replacePattern: string, flags: string, sampleText: string): Promise<RegexTestResult> {
  return await apiApplySingleRegex(pattern, replacePattern, flags, sampleText);
}

async loadConversationLorebooks(conversationId: string): Promise<void> {
  try {
    this.conversation_lorebooks = await apiGetConversationLorebooks(conversationId);
  } catch (err) {
    console.error("Failed to load conversation lorebooks:", err);
    this.conversation_lorebooks = [];
  }
}

async loadCharacterLorebooks(characterId: string): Promise<void> {
  try {
    this.character_lorebooks = await apiGetCharacterLorebooks(characterId);
  } catch (err) {
    console.error("Failed to load character lorebooks:", err);
    this.character_lorebooks = [];
  }
}

async toggleConversationLorebook(conversationId: string, lorebookId: string): Promise<void> {
  const isAssigned = this.conversation_lorebooks.some(l => l.id === lorebookId);
  try {
    if (isAssigned) {
      await apiRemoveLorebookFromConversation(conversationId, lorebookId);
      this.conversation_lorebooks = this.conversation_lorebooks.filter(l => l.id !== lorebookId);
    } else {
      const sortOrder = this.conversation_lorebooks.length;
      await apiAssignLorebookToConversation(conversationId, lorebookId, sortOrder);
      const lorebook = this.lorebooks.find(l => l.id === lorebookId);
      if (lorebook) {
        this.conversation_lorebooks = [...this.conversation_lorebooks, lorebook];
      }
    }
  } catch (err) {
    console.error("Failed to toggle conversation lorebook:", err);
  }
}
```

Update `openConversation` to also load conversation lorebooks and regex scripts:

```typescript
async openConversation(id: string, messages: Message[], branchId?: string | null) {
  this.active_conversation_id = id;
  this.messages = messages;
  this.active_branch_id = branchId ?? null;
  this.loadBranchesForConversation(id);
  this.loadConversationLorebooks(id);
  // Load character lorebooks and regex scripts when opening conversation
  if (this.active_character) {
    this.loadCharacterLorebooks(this.active_character.id);
    this.loadRegexScripts(this.active_character.id);
  }
}
```

Update `closeConversation` to clear lorebook state:

```typescript
closeConversation() {
  this.active_conversation_id = null;
  this.messages = [];
  this.context_budget = null;
  this.active_branch_id = null;
  this.branches = [];
  this.conversation_lorebooks = [];
  this.character_lorebooks = [];
  this.lorebook_details = [];
  this.lorebook_detail_open = false;
}
```

Update the `startChat` complete handler to extract lorebook_details from the StreamChunk complete event:

```typescript
case "complete": {
  // ... existing code ...
  // Update lorebook details from complete event
  if (chunk.data.lorebookDetails) {
    this.lorebook_details = chunk.data.lorebookDetails;
  }
}
```

Import the new API functions at the top of the file.
</action>

<acceptance_criteria>

- `AppState` has `regex_scripts = $state<Map<string, RegexScript[]>>(new Map())` field
- `AppState` has `conversation_lorebooks = $state<Lorebook[]>([])` field
- `AppState` has `character_lorebooks = $state<Lorebook[]>([])` field
- `AppState` has `lorebook_detail_open = $state<boolean>(false)` field
- `AppState` has `lorebook_details = $state<LorebookActivationDetail[]>([])` field
- `AppState` has `async loadRegexScripts(characterId: string): Promise<void>` method
- `AppState` has `async saveRegexScript(script: RegexScript): Promise<RegexScript>` method
- `AppState` has `async deleteRegexScript(id: string, characterId: string): Promise<void>` method
- `AppState` has `async testRegexPattern(pattern, replacePattern, flags, sampleText): Promise<RegexTestResult>` method
- `AppState` has `async loadCharacterLorebooks(characterId: string): Promise<void>` method
- `AppState` has `async loadConversationLorebooks(conversationId: string): Promise<void>` method
- `AppState` has `async toggleConversationLorebook(conversationId: string, lorebookId: string): Promise<void>` method
- `openConversation` calls `loadConversationLorebooks`, `loadCharacterLorebooks`, and `loadRegexScripts`
- `closeConversation` clears `conversation_lorebooks`, `character_lorebooks`, `lorebook_details`, `lorebook_detail_open`
- `bun run check` exits 0
  </acceptance_criteria>

### Task 3: Create ConversationLorebookPanel component

<read_first>

- src/lib/components/chat/ChatHeader.svelte (where the panel trigger button goes)
- src/lib/components/chat/PersonaSwitcher.svelte (Popover pattern reference)
- 09-UI-SPEC.md (Interaction Contract section 1: Chat Header Lorebook Panel)
  </read_first>

<action>
Create `src/lib/components/chat/ConversationLorebookPanel.svelte`:

A Popover component that shows all available lorebooks with checkboxes. Character-assigned lorebooks are shown as read-only (pre-checked with "Character" badge). Conversation-level lorebooks can be toggled.

**Props:** None — reads from `app` store directly.

**Structure (per UI-SPEC section 1):**

Trigger button (rendered in ChatHeader, not in this component — this component only renders the Popover content and provides the trigger slot):

Actually, following the PersonaSwitcher pattern, the component should include both trigger and content:

```svelte
<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { BookOpen } from '@lucide/svelte';
  import { Popover } from 'bits-ui';
  import { cn } from '$lib/utils';
  import type { Lorebook } from '$lib/types';

  let open = $state(false);

  // Character-level lorebooks (read-only, always active)
  let characterLorebooks = $derived(app.character_lorebooks);

  // Available lorebooks that are NOT character-assigned (for conversation assignment)
  let availableForConversation = $derived(
    app.lorebooks.filter(lb =>
      !characterLorebooks.some(cl => cl.id === lb.id)
    )
  );

  let assignedCount = $derived(app.conversation_lorebooks.length);
</script>
```

Implement per UI-SPEC section 1 specs:

- Trigger: BookOpen icon + "Lorebooks" label + optional count badge "(N)"
- Active state styling when lorebooks assigned: `bg-(--color-accent)/10 text-(--color-accent) border border-(--color-accent)/30`
- Inactive state: `text-(--color-muted) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)`
- Popover: `w-80`, `bg-(--color-surface-elevated)`, `shadow-xl`
- Header: "CONVERSATION LOREBOOKS" uppercase section header
- Subtext: "Even budget split across selected lorebooks"
- Character-assigned lorebooks: disabled checkboxes with "Character" badge
- Conversation lorebooks: toggleable checkboxes
- Budget display at bottom: "Budget: N tokens / M max"
- Max height: `max-h-80 overflow-y-auto`
- Empty state: "No lorebooks available. Create one in the Characters tab."

Clicking a conversation lorebook checkbox calls `app.toggleConversationLorebook(conversation_id, lorebook_id)`.

The component needs access to character-level lorebooks. Since `app.getCharacterLorebooks(characterId)` is an async call, load them when the conversation opens and store in a local state variable. Use `$effect` to load when `active_conversation_id` changes.
</action>

<acceptance_criteria>

- `src/lib/components/chat/ConversationLorebookPanel.svelte` exists
- Contains a `Popover.Root` with trigger and content
- Trigger shows BookOpen icon + "Lorebooks" text
- Popover content shows lorebooks with checkboxes
- Character-level lorebooks are shown as disabled/pre-checked with a "Character" badge
- Clicking a conversation-level lorebook checkbox calls `app.toggleConversationLorebook`
- Empty state shows "No lorebooks available. Create one in the Characters tab."
- Component uses `cn()` for class merging
- `bun run check` exits 0
  </acceptance_criteria>

### Task 4: Add ConversationLorebookPanel to ChatHeader

<read_first>

- src/lib/components/chat/ChatHeader.svelte (existing header with persona switcher and provider selector)
  </read_first>

<action>
Modify `src/lib/components/chat/ChatHeader.svelte`:

1. Import `ConversationLorebookPanel` from `./ConversationLorebookPanel.svelte`
2. Add it after the `<ProviderModelSelector />` component, separated by a vertical divider:

```svelte
<ProviderModelSelector />

<div class="h-5 w-px bg-(--color-border)" aria-hidden="true"></div>

<ConversationLorebookPanel />
```

This matches the existing pattern of dividers between header elements. The ConversationLorebookPanel component internally handles the Popover trigger button, so no additional button markup is needed here.
</action>

<acceptance_criteria>

- `src/lib/components/chat/ChatHeader.svelte` imports `ConversationLorebookPanel`
- ChatHeader contains `<ConversationLorebookPanel />` after `ProviderModelSelector` with a divider
- Header renders correctly with the new lorebook panel button visible when a conversation is active
- `bun run check` exits 0
  </acceptance_criteria>

### Task 5: Create LorebookDetailPanel component for context bar

<read_first>

- src/lib/components/chat/MessageInput.svelte (existing context bar where the detail goes below)
- 09-UI-SPEC.md (Interaction Contract section 3: Context Bar Lorebook Detail)
  </read_first>

<action>
Create `src/lib/components/chat/LorebookDetailPanel.svelte`:

Per UI-SPEC section 3, this is an expandable section below the token bar:

```svelte
<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { ChevronRight, ChevronDown, Check } from '@lucide/svelte';
  import { cn } from '$lib/utils';

  let open = $derived(app.lorebook_detail_open);
  let detailCount = $derived(app.lorebook_details.length);

  function toggle() {
    app.lorebook_detail_open = !app.lorebook_detail_open;
  }
</script>
```

**Collapsed state:**

- Row: `flex items-center justify-between px-1`
- Left: ChevronRight + "Lorebook: N active" (or "Lorebook: none active" when 0)
- When N > 0: text is `text-(--color-accent)` to draw attention
- When N == 0: text is `text-(--color-subtle)`
- Click expands

**Expanded state:**

- Same header row but ChevronDown
- Below: list of `LorebookActivationDetail` entries in `space-y-1` container with `px-1 pb-2`
- Each entry row: `flex items-center gap-2 rounded-(--radius-sm) px-2 py-1.5`
  - Entry name: `text-sm text-(--color-foreground)`
  - Content snippet: `text-xs text-(--color-muted)` — first ~30 chars
  - Token cost: `text-[10px] font-mono text-(--color-subtle) bg-(--color-surface-elevated) rounded-(--radius-sm) px-1.5 py-0.5` (e.g., "42 tok")
  - Match indicator: Check icon in `text-(--color-success)` for keyword-matched entries, or "constant" label for always-active entries
- Max height: `max-h-48 overflow-y-auto`
- Budget warning if total exceeds allocation
  </action>

<acceptance_criteria>

- `src/lib/components/chat/LorebookDetailPanel.svelte` exists
- Collapsed state shows "Lorebook: N active" (or "Lorebook: none active")
- When `app.lorebook_details.length > 0`, collapsed text uses `text-(--color-accent)`
- Expanded state renders each `LorebookActivationDetail` with name, snippet, and token cost
- Click toggles `app.lorebook_detail_open`
- Uses `cn()` for class merging
- `bun run check` exits 0
  </acceptance_criteria>

### Task 6: Add LorebookDetailPanel to MessageInput

<read_first>

- src/lib/components/chat/MessageInput.svelte (existing context bar)
  </read_first>

<action>
Modify `src/lib/components/chat/MessageInput.svelte`:

1. Import `LorebookDetailPanel` from `./LorebookDetailPanel.svelte`
2. Add the component below the existing context bar row (the `div` with `mb-2 flex items-center justify-between gap-3 px-1`):

Add after the closing `</div>` of the context bar row:

```svelte
<!-- Lorebook detail panel (expandable) -->
<LorebookDetailPanel />
```

This places the expandable lorebook detail between the token bar and the input frame.
</action>

<acceptance_criteria>

- `src/lib/components/chat/MessageInput.svelte` imports `LorebookDetailPanel`
- `<LorebookDetailPanel />` is rendered below the token bar row
- `bun run check` exits 0
  </acceptance_criteria>

### Task 6a: Update LorebookEditor with enabled, sort_order, and token_budget controls

<read_first>

- src/lib/components/lorebooks/LorebookEditor.svelte (existing editor — needs new entry-level controls)
- src/lib/types/index.ts (LorebookEntry with token_budget, sort_order, enabled fields from Plan 09-01 Task 6)
  </read_first>

<action>
Update `src/lib/components/lorebooks/LorebookEditor.svelte` to expose the three new LorebookEntry fields to the user in the entry detail form (right column).

**Add an `enabled` toggle at the top of the entry form:**

When editing an entry, show an enabled/disabled toggle switch above the entry name field:

```svelte
<div class="flex items-center justify-between mb-3">
  <label class="text-xs text-(--color-muted) uppercase tracking-wider">Enabled</label>
  <button
    type="button"
    class={cn(
      'relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors',
      entry.enabled ? 'bg-(--color-accent)' : 'bg-(--color-surface-hover)'
    )}
    onclick={() => (entry.enabled = !entry.enabled)}
    aria-label="Toggle entry enabled"
  >
    <span class={cn(
      'pointer-events-none inline-block size-4 rounded-full bg-white shadow ring-0 transition-transform',
      entry.enabled ? 'translate-x-4' : 'translate-x-0'
    )} />
  </button>
</div>
```

When `enabled` is `false`, dim the entire entry form with `opacity-50 pointer-events-none` to visually indicate the entry is disabled.

**Add `sort_order` number input:**

Below the enabled toggle, add a small number input for sort order:

```svelte
<div class="mb-3">
  <label class="text-xs text-(--color-muted) uppercase tracking-wider">Sort Order</label>
  <input
    type="number"
    bind:value={entry.sort_order}
    class="w-20 rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-input) px-2 py-1 text-sm"
    min="0"
  />
</div>
```

Lower sort_order values appear first in the activation list. Default is 0.

**Add `token_budget` optional number input:**

After the sort_order input, add an optional token budget field:

```svelte
<div class="mb-3">
  <label class="text-xs text-(--color-muted) uppercase tracking-wider">Token Budget</label>
  <div class="flex items-center gap-2">
    <input
      type="number"
      bind:value={entry.token_budget}
      class="w-24 rounded-(--radius-md) border border-(--color-border) bg-(--color-surface-input) px-2 py-1 text-sm"
      min="0"
      placeholder="∞"
    />
    <span class="text-xs text-(--color-subtle)">Leave empty for shared budget</span>
  </div>
</div>
```

When `token_budget` is `null`/`undefined`, the entry shares the remaining lorebook budget. When set to a positive number, that entry is capped at that many tokens (content truncated if exceeded). The `∞` placeholder indicates the default (unlimited / shared).

**Update the entry list (left column) to show enabled state:**

In each entry row in the list, add a visual indicator for disabled entries:

- Disabled entries: `opacity-50` on the row, plus a small `text-(--color-subtle)` "Disabled" label
- The entry save function must include `enabled`, `sort_order`, and `token_budget` in the data sent to the backend
  </action>

<acceptance_criteria>

- `src/lib/components/lorebooks/LorebookEditor.svelte` contains an `enabled` toggle switch per entry
- `src/lib/components/lorebooks/LorebookEditor.svelte` contains a `sort_order` number input per entry
- `src/lib/components/lorebooks/LorebookEditor.svelte` contains a `token_budget` number input per entry with placeholder "∞"
- Disabled entries show `opacity-50` in the entry list with a "Disabled" label
- Entry save payload includes `enabled`, `sort_order`, and `token_budget` fields
- `bun run check` exits 0
  </acceptance_criteria>

### Task 7: Create RegexScriptEditor component with test panel

<read_first>

- src/lib/components/lorebooks/LorebookEditor.svelte (two-column editor + test panel pattern to mirror)
- src/lib/components/characters/CharacterEditorDialog.svelte (wizard where regex step is added)
- 09-UI-SPEC.md (Interaction Contract section 2: Regex Script Editor)
  </read_first>

<action>
Create `src/lib/components/characters/RegexScriptEditor.svelte`:

Per UI-SPEC section 2, this is a two-column layout matching the LorebookEditor pattern.

**Props:**

```typescript
interface Props {
  characterId: string;
  createMode?: boolean;
}
```

**Left column — Script list (w-2/5):**

- Header: "Scripts" + Plus button (aria-label="Add regex script")
- Each script row:
  - `rounded-(--radius-md) border px-3 py-2` container
  - Script name: `text-sm text-(--color-foreground)` (truncated)
  - Find pattern preview: `text-xs text-(--color-muted) font-mono` (first 40 chars)
  - Enabled indicator: green dot `bg-(--color-success) size-2 rounded-full` when enabled, grey when disabled
  - Selected state: `border-(--color-accent) bg-(--color-accent)/5`
  - Hover delete button: `opacity-0 group-hover:opacity-100`, Trash2 icon, aria-label="Delete script"
  - Delete: immediate delete with undo toast (per UI-SPEC)
- Empty state: "No regex scripts. Add one to transform AI output."
- Create mode lockout: "Regex scripts can be added after the character is created."
- `max-h-[280px] overflow-y-auto`

**Right column — Script form (w-3/5, border-l pl-4):**

- Header: "Script details"
- Fields:
  1. Name — Input, placeholder "Script name"
  2. Find pattern — Input with `font-mono text-sm`, placeholder `Regex pattern (e.g., \\_([^_]+)\\*)`
     - Error state: `border-(--color-destructive)` when backend rejects
     - Error message: `text-xs text-(--color-destructive)` below input
  3. Replace pattern — Input with `font-mono text-sm`, placeholder `Replacement string (e.g. **$1**)`
  4. Flags — Input with `font-mono text-sm`, placeholder "Flags (e.g., gi)", width `w-32`
  5. Enabled — native checkbox, label "Enabled"
- Save button: "Save script" with Check icon, `variant="primary" size="sm" class="w-full"`
- No script selected state: "Select a script to edit"

**Test panel (collapsible, below two-column layout):**

- Collapsible section: `border-t border-(--color-border) pt-3`
- Toggle: ChevronRight/ChevronDown + "Test pattern" text
- When expanded:
  - "Sample AI text" label + Textarea (rows=3)
  - "Run test" button: `variant="secondary" size="sm"`, Play icon
  - Calls `app.testRegexPattern(findPattern, replacePattern, flags, sampleText)`
  - Output section (when results exist):
    - "Result" header
    - Transformed output in `rounded-(--radius-md) border p-3 bg-(--color-surface)` container
    - Match count: "N matches found" or "No matches"
  - Error state: error text in `text-xs text-(--color-destructive)`

**Save logic:**

- Before saving, call `app.saveRegexScript(script)` which validates via backend
- If validation fails, display error on the find pattern field
- On success, update the local selected script

**State:**

- `selectedScriptId: string | null` — currently selected script
- `scriptForm: { name, find_pattern, replace_pattern, flags, enabled }` — form fields
- `testInput: string` — sample text for test panel
- `testResult: RegexTestResult | null` — test output
- `testError: string | null` — test/validation error
- `patternError: string | null` — validation error on save

Load scripts on mount via `app.loadRegexScripts(characterId)`.
</action>

<acceptance_criteria>

- `src/lib/components/characters/RegexScriptEditor.svelte` exists
- Contains two-column layout: script list (left) + script form (right)
- Script list shows each script with name, pattern preview, enabled indicator
- Script form has fields: Name, Find pattern, Replace pattern, Flags, Enabled
- Save button calls `app.saveRegexScript()` which validates via backend
- Invalid pattern shows `border-(--color-destructive)` on find pattern input with error message
- Collapsible test panel with textarea input and "Run test" button
- Test panel calls `app.testRegexPattern()`
- Test output shows transformed text and match count
- Create mode shows "Regex scripts can be added after the character is created."
- Delete uses immediate delete with undo toast pattern
- `bun run check` exits 0
  </acceptance_criteria>

### Task 8: Add "Regex" step to CharacterEditorDialog

<read_first>

- src/lib/components/characters/CharacterEditorDialog.svelte (existing 3-step wizard: Identity, Details, Lorebooks)
  </read_first>

<action>
Modify `src/lib/components/characters/CharacterEditorDialog.svelte`:

1. Import `RegexScriptEditor` from `./RegexScriptEditor.svelte`
2. Add a 4th step to the stepper: "Regex" after "Lorebooks"

**Steps array update:**
The existing steps are approximately: `['Identity', 'Details', 'Lorebooks']`. Add `'Regex'` as the 4th step:

```typescript
const steps = ["Identity", "Details", "Lorebooks", "Regex"];
```

**Step content:**
When `current_step === 3` (0-indexed), render:

```svelte
{#if current_step === 3}
  <RegexScriptEditor
    characterId={pending_id ?? character?.id ?? ''}
    createMode={mode === 'create'}
  />
{/if}
```

**In create mode:** When the character hasn't been saved yet, `pending_id` is null and the regex step should show the lockout message (handled by `createMode` prop on RegexScriptEditor).

**Stepper header update:**
The `StepperHeader` component already renders steps from the array, so adding the 4th element is sufficient.

**Navigation:** The existing previous/next buttons already use `current_step` bounds. Update the max step from 2 to 3 in any bounds-checking logic.
</action>

<acceptance_criteria>

- `CharacterEditorDialog.svelte` steps array has 4 elements: `['Identity', 'Details', 'Lorebooks', 'Regex']`
- When `current_step === 3`, `<RegexScriptEditor>` is rendered
- RegexScriptEditor receives `characterId` and `createMode` props
- In create mode, the regex step shows the lockout message
- In edit mode, existing scripts for the character are displayed
- Navigation (prev/next) works across all 4 steps
- `bun run check` exits 0
  </acceptance_criteria>

---

## Verification Criteria (Plan 09-05)

1. Conversation lorebook assignments can be toggled from chat header
2. Character-level lorebooks appear as read-only in the lorebook panel
3. Regex scripts can be created, edited, deleted in the character editor
4. Invalid regex patterns show clear error messages (REGX-05)
5. Regex test panel shows live preview of transformations
6. Context bar shows expandable lorebook activation detail
7. All components use existing design tokens and patterns

## must_haves

- API wrappers for all new Tauri commands (regex CRUD, conversation lorebook junction, character lorebooks)
- AppState extended with regex_scripts, conversation_lorebooks, character_lorebooks, lorebook_details state and actions
- ConversationLorebookPanel in chat header with character/conversation lorebook management
- LorebookDetailPanel in context bar with expandable activation details
- LorebookEditor updated with enabled toggle, sort_order input, and token_budget input per entry
- RegexScriptEditor in character editor with inline test panel
- CharacterEditorDialog has 4th step "Regex"
- Invalid regex patterns rejected on save with error display (REGX-05)
- All TypeScript types match Rust serde output

---

## Phase-Wide Cross-Cutting Constraints

1. **Rust-TS type alignment:** `LorebookEntry` serde output must match the TypeScript `LorebookEntry` interface — `token_budget` as `number | null`, `sort_order` as `number`, `enabled` as `boolean`. `RegexScript` serde must match TypeScript `RegexScript` interface.

2. **Migration safety:** The Phase 9 migration adds columns with defaults (`sort_order DEFAULT 0`, `enabled DEFAULT 1`) and nullable columns (`token_budget`). Existing rows will have sensible defaults. Test migration on a DB with existing data.

3. **Lorebook budget fraction:** 25% of context window allocated to lorebook by default. This is a constant in the PromptEngine, not yet a user-configurable setting. The budget is split evenly across all assigned lorebooks (character + conversation), per D-04.

4. **Regex application timing:** Regex transforms are applied server-side on stream completion, BEFORE the `Complete` chunk is sent and BEFORE saving to DB. During streaming, raw text displays. This is consistent with Phase 7's conditional rendering pattern.

5. **LorebookActivation role filtering:** The tauri `LorebookActivation` enum (Always/CharacterOnly/UserOnly) is an enhancement over gtk_ref's simple `enabled` bool. The activation logic must scan only messages of the matching role for CharacterOnly/UserOnly entries.

6. **DbCommand registration:** Every new `DbCommand` variant must have a corresponding match arm in `DbWorker::handle_command`. Missing match arms cause compile errors.

7. **Tauri command registration:** Every new `#[tauri::command]` must be registered in `lib.rs` `invoke_handler`. Missing registrations cause "command not found" runtime errors.

8. **Regex validation before save:** `save_regex_script` Tauri command MUST validate the pattern (via `validate_regex_pattern` + `validate_regex_flags`) BEFORE persisting. This ensures REGX-05: "Invalid regex patterns are rejected on save with a clear error message."

9. **Frontend streaming unchanged:** The frontend's `startChat`, `regenerate`, and `editResend` methods in AppState do NOT need changes for regex — the backend applies transforms before sending the `Complete` chunk. The frontend receives already-transformed content.

10. **Regex application is server-side (UI-SPEC reconciliation):** If the UI-SPEC document states that regex should be applied on the frontend, that is superseded by this plan's decision. Regex runs server-side before the `Complete` chunk is emitted, which is necessary because (a) the regex-transformed content is what gets saved to the DB, and (b) applying regex on partially-streamed chunks would produce inconsistent intermediate output. The UI-SPEC should be updated to reflect server-side application on next revision.

11. **Depth-based lorebook entries injected AFTER truncation:** Depth-based lorebook system messages are inserted into history AFTER the oldest-first truncation step, not before. This prevents injected messages at old positions from being silently removed by budget truncation. Depth offsets are therefore relative to the truncated history length, matching SillyTavern/gtk_ref behavior.

12. **`inject_lorebook_entries` is the single source of truth for position handling:** The `inject_lorebook_entries` service function in `service/lorebook.rs` handles all position-based injection logic (`BeforeChar`, `AfterChar`, `AtDepth`). `assemble_with_budget` must call this function rather than duplicating the position-handling logic inline. This avoids drift between the service function and its callers.
