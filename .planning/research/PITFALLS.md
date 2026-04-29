# Pitfalls Research

**Domain:** AI roleplay desktop application (Rust/GTK4/rig/SQLite)
**Researched:** 2026-04-28
**Confidence:** MEDIUM-HIGH

Research draws from: rig official docs (Context7, HIGH), gtk4-rs book (Context7, HIGH), Relm4 book (Context7, HIGH), rusqlite docs.rs (Context7, HIGH), tiktoken-rs docs (Context7, HIGH), SillyTavern documentation (official docs, MEDIUM), rig GitHub issues (direct, MEDIUM), AI roleplay ecosystem patterns (community knowledge, MEDIUM).

## Critical Pitfalls

### Pitfall 1: Blocking the GTK Main Loop with AI Streaming

**What goes wrong:**
AI response tokens arrive via the tokio runtime and are forwarded to the GTK main loop for UI updates. If the bridge between tokio and GLib is implemented incorrectly — e.g., processing multiple tokens in a single GLib idle callback, or doing string rendering work on the GTK thread — the UI freezes during streaming. Users see choppy typewriter rendering or a completely frozen window until the full response completes.

**Why it happens:**
The glib::MainContext channel bridge is the canonical pattern (documented in gtk4-rs book), but the temptation is to batch tokens for "efficiency" or to do markdown→Pango rendering inside the channel receiver callback. Both block the main loop. The gtk4-rs book's own `compile_fail` example shows developers trying to touch GTK widgets from `gio::spawn_blocking` — the compiler saves them, but the channel bridge version of the same mistake compiles and silently degrades performance.

**How to avoid:**
- Use `glib::MainContext::channel(PRIORITY_DEFAULT)` as the tokio→GTK bridge. Each token chunk should be a single `String` sent through the sender.
- The receiver callback (`receiver.attach()`) must do **minimal work**: append the token string to a buffer and mark the GTK Label/TextView as needing redraw. Do NOT re-render the full markdown→Pango pipeline on every token.
- Batch markdown re-rendering: accumulate tokens into a buffer, re-render markdown to Pango markup on a debounce timer (e.g., every 100ms or every N tokens), not per-token.
- Keep AI response parsing (rig streaming, `MultiTurnStreamItem` matching) entirely in the tokio task. Only the final text string crosses the channel boundary.

**Warning signs:**
- UI stutters when AI is streaming, especially with fast providers (Groq, local Ollama)
- GTK frame drops visible as "typewriter" not actually being smooth
- `gtk::Label` or `gtk::TextView` updates causing measurable main-loop blocking (profile with `GTK_DEBUG=interactive`)

**Phase to address:**
Phase implementing AI streaming (core chat loop). Must be architected correctly from day one — retrofitting the channel bridge after building a synchronous prototype is extremely painful.

---

### Pitfall 2: Token Counting Mismatch Across Providers

**What goes wrong:**
The app uses tiktoken-rs with `cl100k_base` encoding to count tokens for context budget tracking. But different AI providers use different tokenizers: Anthropic uses its own tokenizer, Gemini uses a different one, Llama/Mistral models use SentencePiece. The token counts the app reports to the user (and uses for context truncation) diverge from what the provider actually counts. This causes: (1) premature or late truncation, (2) context overflow errors from providers, (3) broken persona compression that thinks it has budget it doesn't.

**Why it happens:**
tiktoken-rs only provides OpenAI tokenizer implementations (`o200k_base`, `cl100k_base`, `p50k_base`, `r50k_base`). There is no Anthropic tokenizer, no Llama tokenizer, no Gemini tokenizer available in tiktoken-rs. Developers pick `cl100k_base` as a "reasonable approximation" and ship it. The divergence between encodings is 5-15% for English text and can exceed 20% for CJK or special-character-heavy roleplay text (character cards with markdown, template variables).

**How to avoid:**
- **Accept the approximation explicitly.** Design the token budget system with a safety margin (e.g., reserve 10-15% of context window as buffer).
- **Use provider-specific token counts when available.** After streaming completes, rig's `MultiTurnStreamItem::FinalResponse` may include `usage` data. Use this to recalibrate budget tracking.
- **Implement per-provider encoding selection:** OpenAI models → `o200k_base` or `cl100k_base` (tiktoken-rs supports both), Anthropic → `cl100k_base` with acknowledgment of ~10% error, local models → `cl100k_base` as rough estimate with larger safety margin.
- **Never count tokens at display precision.** Budget bars should show ranges ("~4,200 of ~8,000 tokens") not exact numbers.
- **Design persona compression tiers with provider-aware overcounting:** When compressing the persona block, count tokens with the largest likely encoding for that provider to ensure truncation is conservative.

**Warning signs:**
- Context overflow errors from providers despite budget tracker showing available space
- Persona block getting truncated when it shouldn't be
- Users reporting "the AI forgot my character's description" (late truncation)

**Phase to address:**
Phase implementing context window management and token budgeting. The token counting abstraction must be designed before persona compression and lorebook activation are built on top of it.

---

### Pitfall 3: SQLite on the Main Thread (UI Jank Under Load)

**What goes wrong:**
rusqlite is a synchronous, blocking API. If database operations (loading conversations, searching with FTS5, importing character cards) run on the GTK main thread, the UI freezes. This is especially catastrophic during batch operations (importing 50 character cards, FTS5 search across thousands of messages) where individual queries are fast but the aggregate blocks for hundreds of milliseconds.

**Why it happens:**
rusqlite's `Connection` is `!Send` by default (unless the `serde_json` or other features are used with care). Developers wrap DB calls in "quick" synchronous functions that seem instant during development with tiny test data, then ship with real data sizes and the UI janks. The temptation to call `conn.query()` directly from a Relm4 `update()` function is strong because it "just works" in early prototypes.

**How to avoid:**
- **Database operations MUST run on background threads.** Use Relm4's `spawn_oneshot_command()` or a dedicated `Worker` for all DB access.
- Consider using a dedicated DB thread with a message-passing interface: send DB commands through an `mpsc` channel, receive results via `ComponentSender::output()` or `CommandOutput`.
- If using `rusqlite::Connection` across threads, use `Connection::open()` in the background thread and keep it there. Do NOT share connections across threads — SQLite's threading mode 1 (serialized) allows it but it's slower than dedicated-thread access.
- Enable WAL mode (`PRAGMA journal_mode=WAL`) for concurrent reads while writing. This is critical for FTS5 search responsiveness.
- Set `busy_timeout` to a reasonable value (5 seconds) to handle lock contention gracefully instead of failing immediately.
- Use `prepare_cached()` for frequently executed queries to avoid re-preparing SQL statements.

**Warning signs:**
- UI stutters when opening large conversations
- Search feels sluggish with >100 messages
- Import operations freeze the entire window
- `SQLITE_BUSY` errors appearing intermittently

**Phase to address:**
Phase implementing data layer / SQLite schema. The DB access pattern (background thread + channel) must be established before any feature builds on top of it. Retrofitting async DB access after features are built on synchronous access requires rewriting every DB call site.

---

### Pitfall 4: Context Window Mismanagement — "Just Send Everything"

**What goes wrong:**
The roleplay context is assembled from multiple layers: system prompt, character card, persona block, activated lorebooks, example messages, and conversation history. The naive approach is to concatenate all of these and send them as the full prompt. This works for short conversations but breaks when: (1) the conversation grows beyond the context window, (2) multiple lorebooks activate simultaneously, (3) a large persona block pushes the character card out. The AI "forgets" the character, speaks out of character, or produces garbled output because critical context was truncated randomly.

**Why it happens:**
SillyTavern's own evolution documents this exact journey: early versions just concatenated everything, and users had to manually manage context. The "sticky/semi-sticky/volatile" priority system was added after years of user complaints about characters "going OOC" (out of character) in long chats. Without explicit priority layers, the truncation is FIFO — the oldest messages go first, which often removes the character card or system prompt that was sent first.

**How to avoid:**
- **Implement a layered context assembly with explicit priority tiers** from the start:
  - **Sticky (never truncated):** System prompt + character card (name, description, personality)
  - **Semi-sticky (compressible):** Persona block (full → trimmed → minimal → essential tiers), scenario
  - **Activatable (inserted dynamically):** Lorebook entries (respect depth/position settings)
  - **Volatile (truncated first):** Conversation history (oldest messages truncated)
- **Token budget must be computed BEFORE assembly**, not after. Calculate: `total_budget - sticky_tokens - semi_sticky_tokens - activatable_tokens = available_for_history`.
- **Persona compression tiers are not optional.** A full persona block might be 800 tokens but the essential tier might be 50 tokens. Budget tracking must know all tier sizes.
- **Lorebook insertion position matters.** SillyTavern's spec supports "before char defs" and "after char defs" positions. Inserting lorebook entries at the wrong position breaks the narrative structure the AI expects.

**Warning signs:**
- Characters "forget" their personality in long conversations
- Context overflow errors appearing at conversation lengths well below the model's advertised context window (because lorebooks bloated the non-history portion)
- Users can't use both a detailed persona AND a lorebook — they have to choose

**Phase to address:**
Phase implementing context window management. This is the architectural core of a roleplay app. Must be designed before conversation history persistence, persona system, or lorebook activation.

---

### Pitfall 5: Lorebook Recursive Activation Bomb

**What goes wrong:**
Lorebook entries can activate based on keywords in the conversation. If an activated entry's text contains keywords that trigger other entries, and those entries contain keywords that trigger more entries, you get cascading activation. Without a depth limit, this can: (1) consume the entire context budget with lorebook text, (2) cause infinite loops in activation logic, (3) insert contradictory information that confuses the AI.

**Why it happens:**
Recursive lorebook activation is a feature, not a bug — SillyTavern supports it with configurable depth limits. The pitfall is implementing the activation loop without depth tracking, or setting the depth limit too high (or making it configurable without a hard maximum). A character card with 50 lorebook entries where Entry A triggers B, B triggers C and D, C triggers E and F... can easily consume 10,000+ tokens in recursive activations.

**How to avoid:**
- **Hard cap on recursion depth:** Maximum 3-5 levels of recursive activation, regardless of user configuration. SillyTavern's default is typically 2.
- **Hard cap on total activated lorebook tokens:** Even with depth limits, cap the total token count of all activated entries (e.g., 30% of context budget maximum).
- **Track already-activated entries:** Prevent the same entry from activating twice in a single prompt assembly cycle.
- **Activation must be computed BEFORE token counting** for the budget. Don't activate lorebooks, count remaining budget, then realize you've overflowed — compute activation, count tokens, if over budget, deactivate lowest-priority entries.
- **Implement a two-pass assembly:** First pass identifies all activated entries, second pass selects which fit in budget by priority.

**Warning signs:**
- Context budget showing 90%+ consumed by lorebooks
- AI responses becoming incoherent with many lorebook entries active
- Activation computation taking noticeable time (indicating deep recursion)
- Same lorebook text appearing multiple times in the assembled prompt

**Phase to address:**
Phase implementing lorebook/WorldInfo system. Must have depth limits and budget caps from the initial implementation.

---

### Pitfall 6: rig Streaming Cancellation Without State Cleanup

**What goes wrong:**
Users click "stop generating" mid-stream. The tokio task running the rig stream is cancelled (dropped). But if the cancellation doesn't properly: (1) save the partial response to the database, (2) update the conversation history, (3) clean up the streaming state in the UI model, (4) release the per-conversation lock — the app enters an inconsistent state. Partial messages are lost, the UI shows stale state, or the next prompt includes the unsaved partial response in context.

**Why it happens:**
rig's `StreamingPrompt` returns a stream that, when dropped, simply stops yielding items. There's no built-in "on cancellation" hook. The tokio task gets cancelled via `JoinHandle::abort()`, which drops the future without running destructors on borrowed state. The partial response text might exist only in the streaming buffer and never makes it to the database or conversation history.

**How to avoid:**
- **Use `tokio::select!` with a cancellation token** rather than `abort()`. Give the streaming task a `CancellationToken` that it checks between stream chunks. On cancellation, the task can: save partial response, update UI state, release locks.
- **Persist every token chunk** to a "draft" row in the database as it arrives. On cancellation, the draft becomes the final (partial) response. On completion, the draft is promoted. This makes cancellation safe at any point.
- **Track streaming state in the component model:** A `StreamingState` enum (Idle, Streaming { task: JoinHandle }, Cancelling) prevents double-cancel or sending a new prompt while still streaming.
- **Always process `FinalResponse` from rig's stream** — it contains token usage data needed for budget tracking. If cancelled before `FinalResponse`, estimate usage from the partial text length.
- **Test cancellation at every point:** after 1 token, after 100 tokens, after streaming completes, during DB save, during markdown rendering. Each is a different race condition.

**Warning signs:**
- Lost messages after clicking "stop"
- Conversation history showing incomplete messages
- Token budget drifting (unsaved usage not counted)
- Two streaming tasks running simultaneously (no lock preventing double-send)

**Phase to address:**
Phase implementing AI streaming. Cancellation must be a first-class concern from the start, not a "we'll add a stop button later" feature.

---

### Pitfall 7: CharacterCardV2 Spec Drift — Incompatible Import

**What goes wrong:**
The app claims CharacterCardV2 compatibility but implements a slightly different schema. Community character cards (built for SillyTavern) fail to import correctly: fields are lost, lorebooks aren't loaded, extensions are dropped, PNG-embedded metadata isn't read. The "ecosystem" value proposition — interoperability with the massive library of existing SillyTavern character cards — is broken.

**Why it happens:**
The CharacterCardV2 spec is a community-driven JSON schema that lives in the SillyTavern codebase, not a formal standard document. It has evolved over time with additions (extensions system, alternate greetings, lorebook format changes). Implementing from memory or a blog post instead of the actual spec source leads to field mismatches. The PNG embedding format (tEXt chunks with specific keys) has its own quirks.

**How to avoid:**
- **Read the actual spec implementation, not blog posts.** The authoritative source is the SillyTavern codebase: `public/scripts/common-char-data.js` and the character import parsing code.
- **Implement the full `data` object structure** including `extensions` (which is a catch-all for custom fields), `alternate_greetings`, `character_book` with its full schema (entries with `keys`, `secondary_keys`, `content`, `extensions`, `position`, `depth`, etc.).
- **PNG import must handle tEXt chunks** with key `chara` containing base64-encoded JSON. Validate chunk size limits to prevent memory bombs from malformed PNGs.
- **Write import tests against real SillyTavern character cards** from community repositories. The spec has de facto edge cases that only real data reveals.
- **Validate aggressively on import:** schema validation, ReDoS detection for regex scripts, PNG chunk size limits. Don't trust community cards blindly.

**Warning signs:**
- Imported characters missing lorebook entries
- "Extensions" data lost on import/export round-trip
- PNG cards failing to import that work in SillyTavern
- JSON cards importing but with wrong field mappings

**Phase to address:**
Phase implementing character card CRUD and import/export. Must validate against the actual SillyTavern spec implementation, not assumptions.

---

### Pitfall 8: macOS GTK4 Rendering and Input Quirks

**What goes wrong:**
The app looks and works great on Linux but has subtle (and not-so-subtle) issues on macOS: text rendering looks different, keyboard shortcuts don't work (or conflict with macOS system shortcuts), window management behaves unexpectedly, and the libadwaita styling looks "off" compared to native macOS apps. Users on macOS (the developer's own platform!) experience a second-class feel.

**Why it happens:**
GTK4 on macOS uses Quartz backend instead of Wayland/X11. While the gtk4-rs bindings are cross-platform, the rendering pipeline, font handling, and input system differ. libadwaita widgets may render differently due to macOS's different compositor behavior. Keyboard shortcuts that use Ctrl on Linux need Cmd on macOS, and GTK4's default accelerator handling doesn't automatically remap.

**How to avoid:**
- **Test on macOS from the very first UI phase.** The developer is on macOS — use this advantage.
- **Implement platform-aware keyboard shortcuts:** Use `gtk::Shortcut` with platform-specific modifiers (Ctrl on Linux/Windows, Cmd on macOS). Check `gtk::functions::is_major_platform()` or use conditional compilation.
- **Set `gtk::Application::default_flags()`** which includes `APPLICATION_FLAGS_NON_UNIQUE` on macOS to handle single-instance correctly.
- **Test font rendering** — macOS may need explicit `gtk::Settings::gtk_xft_antialias` or `gtk::Settings::gtk_font_name` configuration to match expected rendering.
- **Use `AdwApplication` not plain `gtk::Application`** — libadwaita's `AdwApplication` handles some platform-specific initialization.
- **Expect scrollbar behavior differences** — macOS uses overlay scrollbars by default, Linux doesn't. Test with both.

**Warning signs:**
- Keyboard shortcuts using Ctrl instead of Cmd on macOS
- Text rendering looking blurry or differently sized
- Window not appearing in center or with wrong size on macOS
- Right-click context menus not appearing (different mouse button handling)

**Phase to address:**
Phase implementing the initial GTK4 shell and UI framework. Platform-specific handling must be in the app scaffold, not bolted on later.

---

### Pitfall 9: Template Variable Resolution Edge Cases

**What goes wrong:**
Character cards use template variables like `{{user}}`, `{{char}}`, `{{persona.*}}` in their descriptions, first messages, and lorebook entries. The resolution logic has edge cases: `{{user}}` inside a lorebook entry that's activated by a keyword containing `{{user}}`'s resolved value; `{{persona.appearance}}` when the persona has no appearance field set; nested variable references like `{{char}}` inside a `{{persona.*}}` block; or the same variable appearing in both the character card and the system prompt with different expected resolutions.

**Why it happens:**
Template variable resolution seems simple — string replacement — until you encounter: (1) variables that reference other variables, (2) variables that resolve to empty strings (unset persona fields), (3) variable names that are substrings of other variable names (`{{user}}` vs `{{user_name}}`), (4) variables inside quotes or code blocks that should NOT be resolved, (5) the `{{original}}` variable in branch messages. SillyTavern's template system evolved over years to handle these edge cases; a new implementation will hit all of them.

**How to avoid:**
- **Implement a spec for variable resolution order and rules** before writing code. Define: what gets resolved first, what happens for missing variables, how nested references work.
- **Resolve in a specific order:** `{{user}}` and `{{char}}` first (simple, always available), then `{{persona.*}}` (depends on active persona), then lorebook content (may contain all of the above).
- **Unresolved variables should resolve to empty string, NOT remain as `{{var}}`.** Leaving the raw template syntax in the prompt confuses the AI.
- **Use a proper parser, not regex replacement.** `{{user}}` and `{{user_name}}` require token-boundary awareness. A simple `str::replace("{{user}}", value)` will mangle `{{user_name}}`.
- **Resolve BEFORE token counting.** The resolved text is what goes to the AI and what counts toward the budget.
- **Test with adversarial cards:** Cards that use `{{char}}` in their name, cards with `{{persona.field_that_doesnt_exist}}`, cards with recursive variable references.

**Warning signs:**
- Raw `{{user}}` syntax appearing in AI context
- Character names appearing wrong (e.g., `{{char}}` resolved to the wrong value)
- Lorebook entries not activating because keywords contain unresolved variables
- Branch messages showing wrong content after variable resolution

**Phase to address:**
Phase implementing template variable resolution and prompt assembly. Must have a spec and test suite before implementation.

---

### Pitfall 10: Conversation Branching Without Tree Structure

**What goes wrong:**
The app supports "branch conversations by editing or regenerating any message." The naive approach stores messages in a flat list with a `parent_id` and `branch_id`. This works for single-level branching but breaks for: (1) branching from a branch (recursive branching), (2) navigating between branches visually, (3) pruning entire branches, (4) counting tokens per branch for context budget, (5) displaying "this message has 3 alternate branches" indicators.

**Why it happens:**
A flat `parent_id` pointer seems sufficient for a tree. But the query patterns for a conversation tree are not the same as for a flat list: "give me the full conversation from root to leaf along this branch" requires a recursive CTE or iterative traversal. Developers prototype with a flat list, it works for linear conversations, and branching becomes a bolt-on that fights the data model.

**How to avoid:**
- **Design the conversation tree data model FIRST**, before building any UI. Each message has: `id`, `parent_id`, `conversation_id`, `branch_index`, `content`, `role`, `created_at`.
- **Store the "active branch" per conversation** — which leaf path is currently displayed. This is a list of message IDs from root to current leaf.
- **Implement branch navigation as "switch to sibling"** — given a message with `parent_id`, its siblings are all messages with the same `parent_id`. The `branch_index` orders them.
- **Token counting must be per-branch** — the same message may be in multiple branches' contexts. Budget tracking follows the active branch path.
- **Use SQLite recursive CTEs** for branch traversal queries: `WITH RECURSIVE branch AS (SELECT ... UNION ALL SELECT ...)` walks the tree efficiently.
- **Pruning deletes the subtree** — a `DELETE` with a recursive CTE finds all descendants of a message.

**Warning signs:**
- Regenerating a message creates a "branch" but the UI shows both messages sequentially
- Can't navigate back to the original after editing a message
- Branch indicators ("3 alternatives") don't appear or show wrong counts
- Context budget includes messages from all branches, not just the active one

**Phase to address:**
Phase implementing conversation persistence and branching. The tree data model is foundational and can't be retrofitted onto a flat list without a migration nightmare.

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Synchronous DB calls from UI code | Faster to prototype, no channel setup | UI jank under load, hard to retrofit to async | Never — establish the background-thread pattern from day one |
| Using cl100k_base for all providers | One tokenizer implementation, simpler code | Token count errors for non-OpenAI providers, broken budget tracking | Acceptable for MVP if documented as approximate; must add per-provider selection |
| Flat message list instead of tree | Simpler schema, simpler queries | Can't support branching without migration | Never — conversation tree is core feature, not optional |
| Skipping persona compression tiers | Persona always sent at full size | Large personas consume too much budget, force aggressive history truncation | Acceptable for MVP with small personas only; must add before power users adopt |
| Storing API keys in SQLite | No OS keychain dependency, simpler setup | Security vulnerability, keys in backups, no OS-level protection | Never — PROJECT.md mandates OS keychain |
| Implementing lorebook without depth/budget caps | Faster to implement, works for simple cards | Recursive activation bombs, context overflow with complex cards | Never — caps are safety-critical, not optional |
| One-shot rendering of markdown on every token | Simpler rendering code | UI freezes during streaming, CPU waste | Acceptable only for non-streaming responses; must debounce for streaming |
| Skipping CharacterCardV2 extensions field | Smaller import schema, less validation | Community cards lose data on import/export, broken ecosystem compatibility | Never — extensions are where critical card data lives |

## Integration Gotchas

Common mistakes when connecting to external services.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| rig OpenAI provider | Assuming `cl100k_base` tokenizer matches OpenAI's server-side count | Use `o200k_base` for GPT-4o+, `cl100k_base` for GPT-3.5/4, apply safety margin |
| rig Anthropic provider | Sending system prompt as a regular message in history | Anthropic requires system prompt as a separate parameter. rig's `preamble()` handles this — use it |
| rig Ollama provider | Using OpenAI token counts for local models | Local models use SentencePiece — counts diverge significantly. Use larger safety margin (20%) |
| rig streaming | Not handling `PromptError::CompletionError` separately from other errors | Match on `PromptError::CompletionError(e)` for model-specific failures vs. network errors. Different retry strategies |
| rig streaming | Dropping the stream without processing `FinalResponse` | `FinalResponse` contains `usage` data needed for budget tracking. Process before dropping |
| OS keychain (keyring crate) | Assuming keychain is always available | Keychain may be unavailable (headless Linux, locked keychain). Fall back to prompting user, never store in DB |
| PNG character card import | Trusting arbitrary data from PNG tEXt chunks | Validate: chunk size limits, valid UTF-8 after base64 decode, valid JSON schema, ReDoS in regex scripts |
| SillyTavern format import | Only reading top-level CharacterCardV2 fields | Must read `data.character_book.entries[].extensions`, `data.alternate_greetings`, `data.extensions` — these contain critical custom data |
| GTK4 on macOS | Using Ctrl for all keyboard shortcuts | macOS users expect Cmd. Use platform-aware accelerator resolution |

## Performance Traps

Patterns that work at small scale but fail as usage grows.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| SQLite without WAL mode | `SQLITE_BUSY` errors during concurrent reads/writes, slow writes blocking reads | `PRAGMA journal_mode=WAL` on first connection | 10+ concurrent read/write operations |
| Un-indexed FTS5 search | Search takes seconds across thousands of messages | Create FTS5 virtual table, use `MATCH` not `LIKE`, index content columns | >100 messages with FTS5 |
| Re-preparing SQL statements | Slow DB operations despite simple queries | Use `prepare_cached()` for frequently executed queries, set cache capacity | Frequent DB access patterns (message loading, card queries) |
| Token counting on every keystroke | UI lag when typing in character card editor | Debounce token counting (300ms after last keystroke), cache counts for unchanged text | Character descriptions >1000 tokens |
| Lorebook keyword scanning on every message | Noticeable delay before AI response after user sends message | Cache activated entries per conversation, only rescan when keywords change or new messages arrive | 20+ lorebook entries per character |
| Rendering full markdown on every streaming token | CPU spikes during streaming, frame drops | Debounce rendering (every 100ms or N tokens), render partial text as plain during streaming | Streaming responses >500 tokens |
| Loading full conversation tree into memory | Memory usage grows linearly with conversation length, slow conversation switching | Lazy-load branches, only load active branch path + metadata for siblings | Conversations >500 messages |
| No prepared statement cache limit | Memory leak from cached prepared statements growing unbounded | Call `set_prepared_statement_cache_capacity(N)` with a reasonable limit (e.g., 50) | Long-running sessions with diverse queries |

## Security Mistakes

Domain-specific security issues beyond general web security.

| Mistake | Risk | Prevention |
|---------|------|------------|
| Storing API keys in SQLite | Keys in database file, included in backups, readable by any process with file access | Use `keyring` crate for OS-native credential store. Keys never touch the database |
| Logging API keys or prompt content | Keys/conversations in log files, crash reports, or debugging output | Never log raw API requests. Redact keys in error messages. Log only metadata (provider, model, token count) |
| No ReDoS validation on imported regex scripts | Malicious character cards with pathological regex can freeze the app | Validate regex complexity on import: limit repetition operators, check for nested quantifiers, set execution timeout |
| No PNG chunk size limits on import | Malicious PNG with multi-GB tEXt chunk causes OOM | Enforce maximum chunk size (e.g., 10MB) before attempting base64 decode |
| AES-256-GCM vault with weak KDF parameters | Fast Argon2id parameters make brute-force feasible if attacker gets the database | Use OWASP-recommended Argon2id parameters: memory ≥64MB, iterations ≥3, parallelism ≥4 |
| Template variable injection | `{{system}}` or similar in character card description could inject into system prompt | Whitelist allowed template variables, don't resolve arbitrary `{{.*}}` patterns |

## UX Pitfalls

Common user experience mistakes in this domain.

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| No streaming typewriter effect | Users stare at empty screen for 5-30 seconds waiting for complete response | Stream token-by-token with smooth rendering. This is table-stakes for AI chat apps |
| Token budget not visible | Users don't know why AI "forgot" things; feel the app is broken | Show live token budget bar with layer breakdown (system, character, persona, lorebook, history) |
| No cancel/stop button during generation | Users trapped waiting for long responses, or force-quit the app | Prominent stop button, instant cancellation with partial message saved |
| Context overflow with no explanation | API error shown as raw error code, user has no idea what happened | Show user-friendly "context window full" message with suggestion to reduce persona detail or clear history |
| Lorebook entries activating invisibly | Users don't understand why AI is behaving differently; no transparency | Show activated lorebook count and token impact in context budget display |
| Character import fails silently | Card appears to import but lorebook/extensions are missing | Show import preview with detected fields, warn on missing/unsupported fields |
| No branch navigation affordance | Users don't know branches exist after editing/regenerating | Visual branch indicators (dots, tree lines), swipe/click navigation between branches |
| Platform-inconsistent keyboard shortcuts | macOS users frustrated by Ctrl-key shortcuts instead of Cmd | Platform-aware shortcut system from day one |

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **AI Streaming:** Often missing cancellation handling — verify that partial messages are saved on cancel and token usage is tracked
- [ ] **Token Counting:** Often missing per-provider encoding selection — verify that Anthropic/Ollama/Gemini models use appropriate approximations with safety margins
- [ ] **Context Window Management:** Often missing persona compression tiers — verify that persona block compresses at budget thresholds (full → trimmed → minimal → essential)
- [ ] **Conversation Branching:** Often missing branch navigation UI — verify that users can see alternate branches and switch between them
- [ ] **Lorebook Activation:** Often missing recursion depth cap — verify that activation is limited to 3-5 levels max and total token budget is capped
- [ ] **Character Card Import:** Often missing `extensions` field and `alternate_greetings` — verify full CharacterCardV2 spec coverage with real SillyTavern cards
- [ ] **SQLite Setup:** Often missing WAL mode and busy_timeout — verify `PRAGMA journal_mode=WAL` is set on initialization
- [ ] **API Key Storage:** Often missing OS keychain integration — verify keys are stored via `keyring` crate, never in config files or database
- [ ] **Template Variables:** Often missing edge cases for unresolved variables — verify `{{persona.nonexistent_field}}` resolves to empty string, not raw syntax
- [ ] **Cross-Platform:** Often missing macOS keyboard shortcut adaptation — verify Cmd vs Ctrl handling on macOS
- [ ] **Markdown Rendering:** Often missing safe Pango markup — verify that pulldown-cmark output is sanitized before inserting into Pango markup (prevents Pango parse errors from malformed AI output)
- [ ] **Group Chat:** Often missing per-character context budget allocation — verify that multi-character conversations divide budget fairly, not first-character-wins

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| GTK main loop blocking | MEDIUM | Refactor DB calls to background thread/channel. Each call site needs individual migration. No schema changes needed |
| Token counting mismatch | MEDIUM | Add per-provider encoding selection layer. Budget system needs recalibration pass. No data migration |
| Synchronous DB access | HIGH | Migrate every DB call site to async pattern. Requires rewriting all data access layers. No schema changes |
| Flat message list → tree | HIGH | Schema migration: add `parent_id`, `branch_index` columns. Backfill: all existing messages become linear tree. UI needs complete rewrite for branch display |
| No persona compression | LOW | Add compression tiers as a new layer. No schema changes, no migration. Budget calculation needs update |
| Lorebook recursion bomb | LOW | Add depth and budget caps to activation logic. No schema changes. Test with adversarial cards |
| Cancellation state corruption | MEDIUM | Add draft-message persistence layer. Migrate streaming state machine to include `Cancelling` state. May need to recover partial messages from users' corrupted conversations |
| CharacterCardV2 incompatibility | MEDIUM | Fix import parser to match spec. May need to re-import affected cards. Exported cards may need re-export. No data loss if source cards are available |
| macOS keyboard shortcuts | LOW | Add platform-aware accelerator resolution. No data changes. Users may need to re-learn shortcuts |

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Blocking GTK main loop | Phase 1 (App Shell / UI Framework) | Profile UI responsiveness during streaming with `GTK_DEBUG=interactive`. No main-loop blocking >16ms |
| Token counting mismatch | Phase implementing context budget | Test: send known-token-count text to each provider, compare app's count vs. provider's reported usage. Verify <15% divergence |
| SQLite on main thread | Phase 1 (Data Layer) | Audit: grep for `conn.query` / `conn.execute` calls outside of background thread handlers. Zero hits |
| Context window mismanagement | Phase implementing context assembly | Test: create conversation exceeding context window, verify character card remains in context and history is truncated |
| Lorebook recursive activation | Phase implementing lorebooks | Test: create card with circular keyword references, verify activation depth stops at configured limit and total tokens are capped |
| Streaming cancellation | Phase implementing AI streaming | Test: cancel at 1%, 50%, 99% of response. Verify partial message is saved, budget is updated, next prompt works correctly |
| CharacterCardV2 spec drift | Phase implementing character card import | Test: import 10 real SillyTavern community cards (various complexity). Verify all fields present after import/export round-trip |
| macOS GTK4 quirks | Phase 1 (App Shell) | Test: run on macOS. Verify Cmd shortcuts work, text renders correctly, window management is native-feeling |
| Template variable edge cases | Phase implementing prompt assembly | Test: adversarial cards with missing fields, nested variables, variable-name substrings. Verify clean resolution |
| Conversation branching tree | Phase implementing conversation persistence | Test: branch from branch, navigate between 3+ branches, delete a branch. Verify tree integrity and correct context assembly per branch |

## Sources

- gtk4-rs book: Main event loop and threading patterns — https://github.com/gtk-rs/gtk4-rs/blob/main/book/src/main_event_loop.md (HIGH confidence, official docs)
- Relm4 book: Worker and Command threading patterns — https://relm4.org/book/stable/threads_and_async/ (HIGH confidence, official docs, Context7)
- rig framework: Streaming, error handling, multi-turn patterns — https://github.com/0xPlaygrounds/rig (HIGH confidence, Context7 + official repo)
- rig GitHub issues: #1599 (system_message support), #1556 (Chat trait mutability) — https://github.com/0xPlaygrounds/rig/issues (MEDIUM confidence, current open issues)
- rusqlite docs.rs: Connection, prepared statements, busy handling, WAL — https://docs.rs/rusqlite/0.39.0/rusqlite/ (HIGH confidence, official docs, Context7)
- tiktoken-rs: Encoding selection per model — https://github.com/zurawiki/tiktoken-rs (HIGH confidence, official docs, Context7)
- SillyTavern documentation: Context template, WorldInfo/lorebook, character cards — https://docs.sillytavern.app/ (MEDIUM confidence, official docs but community-driven spec)
- SillyTavern GitHub: CharacterCardV2 implementation reference — https://github.com/SillyTavern/SillyTavern (MEDIUM confidence, community codebase)
- PROJECT.md: Project constraints and requirements — local, verified source (HIGH confidence)

---
*Pitfalls research for: Revelry — AI roleplay desktop application (Rust/GTK4/rig)*
*Researched: 2026-04-28*
