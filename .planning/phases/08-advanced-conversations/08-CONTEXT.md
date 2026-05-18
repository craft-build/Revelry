# Phase 8: Advanced Conversations - Context

**Gathered:** 2026-05-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can branch conversations from any message to create parallel paths, navigate between branches, and search across all conversations and messages with full-text search. This phase delivers: message tree schema extension (branch-grouped model), conversation forking and branch navigation, FTS5 full-text search across conversations and messages, and UI for branch indicators, branch switching popover, and enhanced sidebar search with message preview.

Covers requirements: CONV-06 (fork conversation), CONV-07 (navigate branches), CONV-08 (full-text search across conversations and messages).

NOT in scope: lorebook injection (Phase 9), regex scripts (Phase 9), theme toggle (Phase 10), editing branches (beyond delete), merging branches.

</domain>

<decisions>
## Implementation Decisions

### Branching Data Model

- **D-01:** Branch-grouped model — add `message_branches` table (`id, conversation_id, parent_message_id, label, created_at`) and add `branch_id` column (nullable, FK to `message_branches`) to the `messages` table. Matches gtk_ref's proven schema. Root messages have `branch_id = NULL`.
- **D-02:** New DB migration (not collapsing into Phase 2 migration) — adds `message_branches` table, `branch_id` column on `messages`, and `messages_fts` virtual table. Follows incremental migration pattern.
- **D-03:** Port `MessageBranch` domain model from `gtk_ref/src/domain/conversation.rs` to `src-tauri/src/domain/conversation.rs`, adapting `NaiveDateTime` → `DateTime<Utc>` / RFC 3339 (Phase 2 convention).
- **D-04:** Add `branch_id` field to Rust `Message` struct (currently absent — Phase 2 flattened it). Nullable `Option<String>`. Update `row_to_message` mapper accordingly.

### Forking Behavior

- **D-05:** Forking allowed from any message (user or AI) — the fork trigger is a "Branch from here" action on any message in the chat view.
- **D-06:** Shared history preserved — when forking from message M, all messages up to and including M remain shared (they stay with `branch_id = NULL` or their existing branch_id). The new branch starts at sort_order M+1. New messages created after the fork point get the new branch_id. No duplication of shared history.
- **D-07:** Auto-labeled branches — new branches are labeled "Branch 1", "Branch 2", etc. (incrementing). The root path is labeled "Main". Users can rename branches from the branch popover (future polish, not blocking).
- **D-08:** Forking from an AI message auto-generates a new AI response on the new branch (like regenerate, but creating a branch instead of replacing the original). Forking from a user message creates the empty branch and waits for user input. This matches the "branching regenerate" pattern from SillyTavern.

### Branch Navigation UI

- **D-09:** Inline badge on forked messages — messages that have sibling branches show a small badge (e.g., "1/3") indicating branch position. Clicking the badge opens a popover listing all branches at that fork point with a content preview snippet.
- **D-10:** Branch popover shows: branch label, first few words of the first message in that branch, and which is active. Selecting a branch switches the chat view to show messages from that branch from the fork point onward. Shared history (before the fork) is always visible regardless of active branch.
- **D-11:** Flat timeline visualization — messages display in a single vertical timeline. At fork points, the badge indicates alternatives exist. No split view, no tree/graph view. Switching branches swaps out messages from the fork point onward while keeping shared history visible.

### Branch Management

- **D-12:** Users can delete an entire branch — removes all messages with that `branch_id` and the `message_branches` record. Other branches (including root/Main) are unaffected. Delete action accessible from the branch popover.
- **D-13:** Branch state tracked in AppState — add `active_branch_id: string | null` to track which branch the user is viewing for the active conversation. When loading a conversation, default to the last active branch or root (NULL).

### Full-Text Search

- **D-14:** FTS5 index covers both conversations (title) and messages (content). Use an `messages_fts` virtual table (following the existing `characters_fts` pattern with sync triggers on INSERT/UPDATE/DELETE of messages). Conversations title search can be handled by joining or by a separate FTS index.
- **D-15:** Enhanced sidebar search — the existing sidebar search input expands to also search message content via the backend FTS5. Results appear in the sidebar list grouped by conversation. When a message matches, show the conversation entry with a preview snippet of the matching message content (highlighted or trimmed).
- **D-16:** Clicking a search result navigates to the conversation and scrolls to the matching message. If the matching message is on a specific branch, auto-switch to that branch.
- **D-17:** New Tauri command `search_all` (or `search_messages`) — takes a query string, returns ranked results with conversation info + matching message snippet + branch_id if applicable. Replaces the client-side LIKE filtering in the sidebar.

### Agent's Discretion

- Exact FTS5 schema design (single unified table vs separate `messages_fts` and `conversations_fts`) — follow the `characters_fts` pattern as a guide.
- How to handle snippet/highlight extraction from FTS5 results (SQLite `snippet()` or `highlight()` functions).
- Branch popover component structure (new component vs extending MessageBubble).
- How `load_messages` changes to support branch-aware loading (branch_id filter parameter).
- Whether branch creation should be a single Tauri command or a multi-step process.
- Migration number and structure.
- How to track "last active branch" per conversation (in-memory state only, or persisted).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Reference Implementation (MUST read — patterns to port)

- `gtk_ref/src/domain/conversation.rs` — MessageBranch domain model with id, conversation_id, parent_message_id, label, created_at. Message struct with branch_id field.
- `gtk_ref/src/db/conversations.rs` — load_messages with branch_id filter, load_branches_for_message, create_branch DB functions. Forking SQL patterns.
- `gtk_ref/src/db/migrations.rs` (Migration 5) — message_branches table schema, messages table with branch_id, idx_messages_conv_branch index.
- `gtk_ref/src/ui/chat/branch_nav.rs` — BranchNav GTK component with popover, branch list, active branch tracking, switch_branch action. UI pattern reference.
- `gtk_ref/src/ui/chat/message_row.rs` — Branch indicator on message rows (colored left border, click to navigate).

### Prior Phase Decisions (carry-forward)

- `.planning/phases/02-persistent-data/02-CONTEXT.md` — D-03 (message model flattened, no branch_id), D-04 (FTS5 virtual tables kept), D-05/D-06 (DbWorker background-thread pattern), D-08 (type alignment Rust ↔ TS), D-15 (one Tauri command per CRUD op), D-16 (async Result<T, String>).
- `.planning/phases/05-live-conversations/05-CONTEXT.md` — D-04 (message persistence timing), D-08 (edit/resend truncates from sort_order), D-11/D-12/D-13 (regenerate replaces last AI message). Branching must coexist with these existing flows.

### Active Backend (will be extended)

- `src-tauri/src/db/migrations.rs` — Current migration 1 schema. Phase 8 adds migration 2 for branch tables + FTS.
- `src-tauri/src/db/conversations.rs` — load_messages, save_message, delete_messages_after_sort_order. Branch-aware loading adds branch_id parameter.
- `src-tauri/src/domain/conversation.rs` — Message struct (add branch_id), new MessageBranch struct.
- `src-tauri/src/db/mod.rs` — DbCommand enum (add branch-related commands + search_messages).
- `src-tauri/src/db/characters.rs` — sanitize_fts_query function and FTS5 search pattern. Reusable for messages FTS.
- `src-tauri/src/commands/` — Add branch Tauri commands and search_messages command.

### Active Frontend (will be modified)

- `src/lib/types/index.ts` — Add branch_id to Message, add MessageBranch type.
- `src/lib/stores/app.svelte.ts` — Add active_branch_id state, branch switching actions, enhanced search.
- `src/lib/components/layout/ConversationSidebar.svelte` — Enhance search to use backend FTS5 with message previews.
- `src/lib/components/chat/MessageBubble.svelte` — Add branch indicator badge and "Branch from here" action.
- `src/routes/chat/[id]/+page.svelte` — Branch-aware message loading, branch switching.

### Project Planning

- `.planning/ROADMAP.md` — Phase 8 definition, success criteria, plan list (08-01 through 08-04)
- `.planning/REQUIREMENTS.md` — CONV-06, CONV-07, CONV-08 requirements
- `.planning/PROJECT.md` — Key decisions including "Preserve full gtk_ref feature set"

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **gtk_ref MessageBranch + branching DB functions**: Complete implementation in `gtk_ref/src/db/conversations.rs` — load_branches_for_message, create_branch, load_messages with branch_id filter. Port directly.
- **gtk_ref branch_nav.rs UI**: Branch navigator component with popover, branch list, indicator. Pattern to adapt for Svelte (popover via bits-ui, badge on MessageBubble).
- **characters_fts pattern**: FTS5 virtual table with sync triggers already exists for characters in `src-tauri/src/db/migrations.rs`. Same pattern applies for `messages_fts`.
- **sanitize_fts_query**: Existing FTS query sanitizer in `src-tauri/src/db/characters.rs`. Reuse for message search.
- **DbWorker + DbCommand pattern**: All DB ops go through the mpsc worker. New branch commands follow the same pattern.

### Established Patterns

- **FTS5 with sync triggers**: characters_fts uses `content=characters, content_rowid=rowid` with INSERT/UPDATE/DELETE triggers. messages_fts follows the same pattern.
- **Migration as incremental M::up()**: Phase 2 used a single collapsed migration. Phase 8 adds a second migration. Use `rusqlite_migration` crate's incremental system.
- **Tauri IPC via invoke()**: snake_case commands, async, Result<T, String>. New commands: fork_conversation, switch_branch, delete_branch, search_messages, get_branches.
- **AppState reactive state**: $state fields, getter properties, action methods. New: active_branch_id, branch-aware message loading.
- **Sidebar search pattern**: Currently client-side LIKE filter on conversations. Backend FTS replaces this with invoke('search_messages') call.

### Integration Points

- **`src-tauri/src/db/migrations.rs`**: Add migration 2 — message_branches table, branch_id column on messages, messages_fts virtual table + triggers.
- **`src-tauri/src/db/conversations.rs`**: Add load_branches_for_message, create_branch, delete_branch_by_id. Modify load_messages to accept branch_id filter.
- **`src-tauri/src/domain/conversation.rs`**: Add MessageBranch struct. Add branch_id field to Message.
- **`src-tauri/src/db/mod.rs`**: Add DbCommand variants: CreateBranch, LoadBranchesForMessage, DeleteBranch, SearchMessages.
- **`src-tauri/src/commands/`**: Add Tauri command handlers for branch ops and search.
- **`src/lib/types/index.ts`**: Add MessageBranch type, add branch_id to Message interface.
- **`src/lib/stores/app.svelte.ts`**: Add active_branch_id, branch switching, enhanced search action.
- **`src/lib/components/chat/MessageBubble.svelte`**: Add fork action button + branch indicator badge.
- **`src/lib/components/layout/ConversationSidebar.svelte`**: Replace client-side search with backend FTS5 search, add message preview snippets to results.
- **`src/routes/chat/[id]/+page.svelte`**: Load branch-aware messages, handle branch switching.

### Constraints

- **Edit/resend coexistence**: Phase 5's edit/resend truncates messages after sort_order N. When branches exist, truncation must respect branch boundaries — only truncate messages on the active branch, not all branches.
- **Streaming + branching**: Phase 5's start_chat creates messages. When generating on a branch, the new message must carry the correct branch_id.
- **Token budget + branching**: Phase 6's PromptEngine assembles history. When on a branch, history should include only messages visible on the active branch path (shared root + branch messages).
- **FTS5 + branches**: Search results that match messages on non-active branches must indicate which branch the match is on.

</code_context>

<specifics>
## Specific Ideas

- Branch indicator badge styling: small pill-shaped badge (e.g., "2/3") positioned near the fork point message, using the accent color. Consistent with the existing hover action pattern on MessageBubble.
- Branch popover: uses bits-ui Popover (already in the project). Shows list of branches with label + first 50 chars of first branch message. Active branch highlighted with accent.
- "Branch from here" action: add to the hover actions row on MessageBubble (alongside Edit, Copy, Regenerate). Pencil for edit, refresh for regenerate, git-branch icon for branching.
- FTS5 snippet: use SQLite's `snippet(messages_fts, 0, '<mark>', '</mark>', '...', 32)` to extract highlighted context around matches.
- Search result in sidebar: when a message matches, show the conversation entry with a second line containing the message snippet in muted text, similar to how email clients show search results.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 08-advanced-conversations*
*Context gathered: 2026-05-18*
