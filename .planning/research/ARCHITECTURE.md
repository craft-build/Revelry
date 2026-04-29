# Architecture Research

**Domain:** AI roleplay desktop application (Rust/GTK4)
**Researched:** 2026-04-28
**Confidence:** HIGH

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         UI Layer (GTK4/libadwaita)                       │
├──────────────────┬──────────────────┬──────────────────┬────────────────┤
│  AppWindow       │  ChatPanel       │  LibraryPanel    │  SettingsPanel│
│  (Root Component)│  (Conversation)  │  (Character Grid)│  (Preferences)│
│  ├─HeaderBar     │  ├─MessageList   │  ├─SearchBar     │  ├─Providers  │
│  ├─SplitView     │  ├─InputBar      │  ├─CardGrid      │  ├─Presets    │
│  └─StatusBar     │  └─BranchNav     │  └─TagFilter     │  └─Vault      │
├──────────────────┴──────────────────┴──────────────────┴────────────────┤
│                    Relm4 Elm Architecture (Message Bus)                   │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐  │
│  │ Input Msgs  │→ │  Update Fn   │→ │ Model State  │→ │ View Render │  │
│  │ (User/Cmd)  │  │ (Pure Logic) │  │ (Struct)     │  │ (Declarative│  │
│  └─────────────┘  └──────┬───────┘  └──────────────┘  └─────────────┘  │
│                          │ Commands                                      │
├──────────────────────────┼──────────────────────────────────────────────┤
│                    Service Layer (tokio runtime)                        │
│  ┌──────────────┐  ┌────▼─────────┐  ┌──────────────┐                 │
│  │ PromptEngine │  │ AiService    │  │ ImportService │                 │
│  │ (Assemble,   │  │ (rig agents, │  │ (PNG parse,  │                 │
│  │  Tokenize,   │  │  stream,     │  │  JSON parse, │                 │
│  │  Lorebook)   │  │  cancel)     │  │  Validate)   │                 │
│  └──────┬───────┘  └──────┬───────┘  └──────────────┘                 │
│         │                 │                                             │
├─────────┴─────────────────┴─────────────────────────────────────────────┤
│                    Data Layer                                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                  │
│  │ DbService    │  │ KeyringStore │  │ FileStore    │                  │
│  │ (SQLite via  │  │ (OS keychain │  │ (PNG export, │                  │
│  │  rusqlite,   │  │  via keyring) │  │  backup)     │                  │
│  │  bg thread)  │  │              │  │              │                  │
│  └──────────────┘  └──────────────┘  └──────────────┘                  │
└─────────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Implementation |
|-----------|----------------|----------------|
| **AppWindow** | Root component, owns split view layout, routes messages between panels | Relm4 `Component` with `AdwOverlaySplitView` |
| **ChatPanel** | Conversation display, message input, streaming token rendering | Relm4 `Component` with `command()` for streaming |
| **LibraryPanel** | Character card grid, search, tag filtering, import/export | Relm4 `Component` + `FactoryVecDeque` for card grid |
| **SettingsPanel** | Provider config, prompt presets, vault management | Relm4 `Component` with `AdwPreferencesWindow` |
| **AiService** | Provider-agnostic AI inference, streaming, cancellation | rig agents built per-request; runs on tokio via `command()` |
| **PromptEngine** | Assembles prompts: system + character + persona + lorebook + history | Pure functions; tokenized with tiktoken-rs |
| **DbService** | All SQLite CRUD: conversations, characters, personas, lorebooks | rusqlite on dedicated background thread via `Worker` |
| **KeyringStore** | API key storage/retrieval from OS keychain | keyring crate; sync ops on background thread |
| **ImportService** | Parse CharacterCardV2 PNG/JSON, validate, batch import | Background `command()`, schema validation |
| **FileStore** | PNG export (with embedded metadata), backup/restore | File I/O on background thread |

## Recommended Project Structure

```
src/
├── main.rs                # App bootstrap, RELM_THREADS config, RelmApp launch
├── app.rs                 # Root AppModel component (split view, routing)
│
├── ui/                    # Relm4 components (view + update logic)
│   ├── mod.rs
│   ├── chat/              # Conversation panel
│   │   ├── mod.rs         # ChatPanel component
│   │   ├── message_row.rs # Single message row (FactoryComponent)
│   │   ├── input_bar.rs   # Text input + send + persona switcher
│   │   └── branch_nav.rs  # Branch navigation indicator
│   ├── library/           # Character library panel
│   │   ├── mod.rs         # LibraryPanel component
│   │   ├── card.rs        # CharacterCard widget (FactoryComponent)
│   │   ├── card_editor.rs # Character editor (AdwPreferencesWindow wizard)
│   │   └── import_dialog.rs
│   ├── settings/          # Settings/preferences
│   │   ├── mod.rs         # SettingsPanel component
│   │   ├── provider_page.rs   # Provider config (AdwPreferencesPage)
│   │   ├── preset_page.rs    # Prompt presets
│   │   ├── persona_page.rs   # Persona management
│   │   └── vault_page.rs     # Encryption/vault settings
│   ├── components/        # Shared reusable Relm4 components
│   │   ├── mod.rs
│   │   ├── markdown_view.rs  # pulldown-cmark → Pango renderer
│   │   ├── token_gauge.rs    # Context window usage bar
│   │   └── search_bar.rs     # Reusable search entry
│   └── dialogs/           # Modal dialogs
│       ├── mod.rs
│       ├── new_chat.rs
│       └── confirmation.rs
│
├── service/               # Business logic (no UI dependency)
│   ├── mod.rs
│   ├── ai.rs              # AiService: rig agent construction, streaming
│   ├── prompt.rs          # PromptEngine: assembly, lorebook, templates
│   ├── token.rs           # Token counting (tiktoken-rs)
│   ├── import.rs          # Character card import/export/validation
│   └── backup.rs          # Backup/restore operations
│
├── db/                    # Database layer
│   ├── mod.rs             # DbService Worker + migration setup
│   ├── models.rs          # Data structs (Character, Persona, Conversation, etc.)
│   ├── characters.rs      # Character CRUD queries
│   ├── conversations.rs   # Conversation + message CRUD
│   ├── personas.rs        # Persona CRUD queries
│   ├── lorebooks.rs       # Lorebook entry CRUD queries
│   ├── presets.rs         # Prompt preset CRUD queries
│   ├── providers.rs       # Provider config CRUD queries
│   └── fts.rs             # FTS5 full-text search setup & queries
│
├── secrets/               # Credential management
│   ├── mod.rs
│   └── keyring.rs         # OS keychain operations (keyring crate)
│
├── domain/                # Pure domain types & logic (no I/O)
│   ├── mod.rs
│   ├── character_card.rs  # CharacterCardV2 spec types, PNG metadata
│   ├── persona.rs          # Persona types, inheritance, compression levels
│   ├── lorebook.rs        # Lorebook entry types, activation logic
│   ├── template.rs        # {{user}}, {{char}}, {{persona.*}} resolution
│   └── context_window.rs  # Context budget calculation, truncation strategy
│
└── util/                  # Utilities
    ├── mod.rs
    └── regex_safety.rs    # ReDoS validation for user regex scripts
```

### Structure Rationale

- **ui/**: Each subfolder is one Relm4 `Component` or `FactoryComponent`. The `view!` macro and `update` function live together because Relm4 couples them via the `#[relm4::component]` macro. Separating view from update across files is not idiomatic.
- **service/**: Pure async business logic that never imports GTK/Relm4 types. Services are called from `command()` closures, making them testable without a GUI.
- **db/**: The `DbService` is a Relm4 `Worker` running on a dedicated thread (rusqlite requires `!Send` connection). Each submodule organizes queries by entity.
- **domain/**: Zero-dependency types and pure functions. No I/O, no GTK, no async. These types flow between all layers.
- **secrets/**: Isolated because keyring operations are synchronous and platform-specific; they must run on background threads.

## Architectural Patterns

### Pattern 1: Elm Architecture via Relm4 Component

**What:** Every UI module is a Relm4 `Component` with typed `Input`, `Output`, and `CommandOutput` message enums. State lives in a model struct. The `update()` function is the only place that mutates state. The `view!` macro declaratively renders state.

**When to use:** For every UI panel, dialog, and reusable widget. This is the only pattern for Relm4.

**Trade-offs:**
- (+) Deterministic state transitions — easy to reason about, test, and debug
- (+) Type-safe message passing between components eliminates runtime errors
- (+) `#[watch]` annotations auto-update views when model changes
- (-) Verbose for simple widgets (3 enums + model + init + update + view for a button)
- (-) Message enum variants can grow large in complex components

**Example:**
```rust
#[derive(Debug)]
enum ChatMsg {
    SendMessage(String),
    CancelStream,
    SwitchPersona(PersonaId),
}

#[derive(Debug)]
enum ChatCmdOut {
    TokenReceived(String),      // Each streaming token
    StreamComplete(Message),     // Final assembled message
    StreamError(AiError),        // Provider error
    TokenCountUpdated(usize),    // Context budget update
}

#[relm4::component]
impl Component for ChatPanel {
    type CommandOutput = ChatCmdOut;
    type Input = ChatMsg;
    // ...
}
```

### Pattern 2: Streaming Bridge (sender.command + rig stream)

**What:** The critical pattern for token-by-token AI streaming. Use `sender.command()` (NOT `oneshot_command()`) which provides a `Sender<CommandOutput>` that can emit multiple messages. Inside the command, iterate over the rig streaming response, sending each token chunk as a `CommandOutput` message back to the GLib main loop.

**When to use:** For ALL AI inference calls — both streaming and non-streaming. Even non-streaming calls benefit from `command()` because it provides `ShutdownReceiver` for cancellation.

**Trade-offs:**
- (+) Each token chunk dispatches to the GLib main loop, keeping UI responsive
- (+) `ShutdownReceiver` enables cancellation (user clicks stop → component drops → future cancelled)
- (+) No manual `glib::MainContext::channel()` setup needed — Relm4 handles the bridge
- (-) Many small messages can cause rendering overhead; batch tokens if needed (e.g., every 2-3 tokens)
- (-) Must be careful not to hold `Sender` after component drops

**Example:**
```rust
fn update(&mut self, msg: ChatMsg, sender: ComponentSender<Self>, _: &Self::Root) {
    match msg {
        ChatMsg::SendMessage(text) => {
            self.streaming = true;
            let agent = self.agent.clone();
            let history = self.prompt_engine.assemble(text);

            sender.command(|out, shutdown| {
                shutdown
                    .register(async move {
                        let mut stream = agent
                            .stream_chat(&text, &history)
                            .await
                            .unwrap_or_else(|e| {
                                out.send(ChatCmdOut::StreamError(e.into())).unwrap();
                                return; // (simplified)
                            });

                        while let Some(item) = stream.next().await {
                            match item {
                                Ok(StreamedAssistantContent::Text(token)) => {
                                    out.send(ChatCmdOut::TokenReceived(token.text)).unwrap();
                                }
                                Err(e) => {
                                    out.send(ChatCmdOut::StreamError(e.into())).unwrap();
                                    break;
                                }
                                _ => {}
                            }
                        }
                        out.send(ChatCmdOut::StreamComplete).unwrap();
                    })
                    .drop_on_shutdown()
                    .boxed()
            });
        }
        ChatMsg::CancelStream => {
            // Dropping the component's shutdown receiver cancels the future
            self.streaming = false;
        }
    }
}

fn update_cmd(&mut self, msg: ChatCmdOut, _sender: ComponentSender<Self>, _: &Self::Root) {
    match msg {
        ChatCmdOut::TokenReceived(token) => {
            self.partial_response.push_str(&token);
            // view! #[watch] picks this up automatically
        }
        ChatCmdOut::StreamComplete => {
            let message = Message::from_assistant(self.partial_response.clone());
            self.messages.push(message);
            self.partial_response.clear();
            self.streaming = false;
        }
        ChatCmdOut::StreamError(e) => {
            self.last_error = Some(e.to_string());
            self.streaming = false;
        }
        ChatCmdOut::TokenCountUpdated(count) => {
            self.token_count = count;
        }
    }
}
```

### Pattern 3: Database Worker (rusqlite on dedicated thread)

**What:** SQLite via rusqlite requires a single-threaded `Connection` (it's `!Send`). Run all database operations on a Relm4 `Worker` that owns the connection. The worker processes DB messages one-at-a-time on its own thread, returning results as `Output` messages.

**When to use:** For ALL database operations. Every CRUD call goes through the worker.

**Trade-offs:**
- (+) Simple model: one connection, one thread, no locking needed
- (+) rusqlite operations are fast (sub-ms for most queries) so serializing is fine
- (-) Long-running queries (FTS5, batch imports) can queue behind simple CRUD
- (-) Cannot do parallel DB reads

**Example:**
```rust
#[derive(Debug)]
enum DbInput {
    SaveCharacter(Character),
    LoadCharacters,
    SaveMessage(ConversationId, Message),
    SearchConversations(String),
}

#[derive(Debug)]
enum DbOutput {
    CharactersLoaded(Vec<Character>),
    CharacterSaved(CharacterId),
    MessageSaved(MessageId),
    SearchResults(Vec<SearchResult>),
    Error(String),
}

struct DbWorker {
    conn: Connection,  // !Send — stays on worker thread
}

impl Worker for DbWorker {
    type Init = PathBuf;  // Database file path
    type Input = DbInput;
    type Output = DbOutput;

    fn init(db_path: Self::Init, sender: ComponentSender<Self>) -> Self {
        let conn = Connection::open(&db_path).expect("Failed to open database");
        run_migrations(&conn);  // rusqlite_migration
        DbWorker { conn }
    }

    fn update(&mut self, msg: DbInput, sender: ComponentSender<Self>) {
        let result = match msg {
            DbInput::SaveCharacter(ch) => {
                self.conn.execute("INSERT INTO characters ...", params![...])
                    .map(|_| DbOutput::CharacterSaved(ch.id))
            }
            DbInput::LoadCharacters => {
                let chars = self.conn.query_map("SELECT * FROM characters", [], |row| { ... })
                    .and_then(|rows| rows.collect::<Result<Vec<_>, _>>());
                chars.map(DbOutput::CharactersLoaded)
            }
            // ...
        };
        // Send result back to main thread
        match result {
            Ok(output) => sender.output(output).unwrap(),
            Err(e) => sender.output(DbOutput::Error(e.to_string())).unwrap(),
        }
    }
}
```

### Pattern 4: Prompt Assembly Pipeline (pure function chain)

**What:** Building the AI prompt is a pure, synchronous pipeline: system prompt → character card → persona block → activated lorebooks → truncated history. Each stage is a pure function that takes the accumulated token budget and returns the assembled section plus remaining budget. No I/O, no async.

**When to use:** Every time an AI request is made. The pipeline runs inside the `command()` closure BEFORE calling rig, because the assembled prompt determines the agent configuration.

**Trade-offs:**
- (+) Extremely testable — pure functions with deterministic inputs/outputs
- (+) Token budget can be precisely managed at each stage
- (-) Tokenization (tiktoken-rs) is CPU-bound and must run on background thread
- (-) Lorebook activation can be recursive, adding complexity

**Example:**
```rust
/// Pure function: assemble the full prompt given domain data.
/// Returns the assembled chat history + system preamble for rig.
fn assemble_prompt(
    character: &CharacterCard,
    persona: &Persona,
    lorebook: &[LorebookEntry],
    history: &[Message],
    budget: TokenBudget,
) -> PromptResult {
    let mut remaining = budget.total;

    // 1. System prompt + character card (sticky — always included)
    let system = format_system_block(character);
    remaining -= count_tokens(&system);

    // 2. Persona block (semi-sticky — compressible)
    let persona_block = persona.to_block(remaining.persona_budget());
    remaining -= count_tokens(&persona_block);

    // 3. Activated lorebooks
    let activated = activate_lorebook(lorebook, &history.last_user_message(), persona);
    let lorebook_block = format_lorebook(&activated, remaining.lorebook_budget());
    remaining -= count_tokens(&lorebook_block);

    // 4. Conversation history (truncated from oldest)
    let history_block = truncate_history(history, remaining);

    PromptResult {
        preamble: format!("{}\n{}\n{}", system, persona_block, lorebook_block),
        history: history_block,
        token_usage: budget.total - remaining,
    }
}
```

### Pattern 5: Factory Components for Dynamic Lists

**What:** Use `FactoryVecDeque` or `FactoryHashMap` for any dynamic list of widgets — message rows, character cards, lorebook entries, etc. Each item is a `FactoryComponent` with its own model, messages, and update loop.

**When to use:** Character card grids, message lists, lorebook entry editors, persona lists.

**Trade-offs:**
- (+) Each item manages its own state independently
- (+) Items can send output messages to parent (forward pattern)
- (+) Efficient diffing — only changed items re-render
- (-) More boilerplate than a simple loop for trivial lists
- (-) Must be careful with index-based operations (items can shift)

## Data Flow

### AI Streaming Request Flow

```
User types message + presses Enter
    │
    ▼
ChatPanel::update(ChatMsg::SendMessage)
    │
    ├─► DbWorker: SaveMessage(user_message)
    │       │
    │       ▼
    │   DbOutput::MessageSaved → ChatPanel::update
    │
    ├─► PromptEngine::assemble_prompt(character, persona, lorebook, history, budget)
    │       │  (runs inside command() closure on tokio)
    │       ▼
    │   Assembled { preamble, history, token_usage }
    │
    ├─► AiService: Build rig agent from provider config + preamble
    │       │
    │       ▼
    │   agent.stream_chat(text, history)
    │       │
    │       ▼ (for each token chunk)
    │   ChatCmdOut::TokenReceived(token) → ChatPanel::update_cmd
    │       │  (each token → GLib main loop → model update → view refresh)
    │       ▼
    │   ChatCmdOut::StreamComplete → ChatPanel::update_cmd
    │       │
    │       ├─► DbWorker: SaveMessage(ai_message)
    │       └─► ChatPanel: append final message, clear partial
    │
    └─► [On Cancel] Component drop → ShutdownReceiver → future dropped
```

### Character Import Flow

```
User selects PNG/JSON file
    │
    ▼
LibraryPanel::update(LibMsg::ImportFile(path))
    │
    ▼
sender.command() → ImportService::import_card(path)
    │
    ├─► Parse PNG chunks / Parse JSON
    ├─► Validate CharacterCardV2 schema
    ├─► Check PNG chunk size limits
    ├─► Validate regex scripts for ReDoS
    │
    ▼
LibCmdOut::ImportResult(ValidatedCard) → LibraryPanel::update_cmd
    │
    ├─► DbWorker: SaveCharacter(card)
    │       │
    │       ▼
    │   DbOutput::CharacterSaved(id) → LibraryPanel::update
    │       │
    │       ┼─► FactoryVecDeque: push new CardWidget
    │
    └─► AdwToast: "Character imported successfully"
```

### State Management

```
┌──────────────────────────────────────────────────────┐
│ Relm4 Message Channels (flume)                        │
│                                                       │
│  User Action ──→ Input ──→ update() ──→ Model Mutate │
│                                        │              │
│  Command Output ──→ update_cmd() ───────┘              │
│                                        │              │
│  Child Output ──→ forward() ──────────┘              │
│                                        │              │
│                                        ▼              │
│                                   view! macro         │
│                                   (#[watch] bindings) │
│                                        │              │
│                                        ▼              │
│                                   GTK Widget Update   │
│                                   (batched by GLib)   │
└──────────────────────────────────────────────────────┘

Key: All state mutation happens in update/update_cmd.
    View is a pure function of model state.
    Commands are the only way to touch tokio runtime from UI.
```

### Key Data Flows

1. **Streaming AI tokens:** `user input → command() → rig stream → Sender<CmdOut>::send(TokenReceived) per chunk → update_cmd() → model.partial_response.push_str() → #[watch] view update`
2. **Database CRUD:** `UI Input msg → Worker::input_sender() → DbWorker::update() on bg thread → Worker::output() → UI update()`
3. **Inter-panel routing:** `Child::Output → parent.forward(input_sender(), transform) → Parent::Input → Parent::update()`
4. **Persona switch mid-chat:** `ChatPanel::SwitchPersona → DbWorker::LoadPersona → ChatPanel::update → PromptEngine::assemble (next request uses new persona)`
5. **Context window tracking:** `PromptEngine::assemble returns TokenUsage → ChatCmdOut::TokenCountUpdated → ChatPanel model.token_count → token_gauge widget #[watch]`

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| Single user, <100 characters | Current architecture is ideal. One DbWorker, one tokio runtime, serial DB ops. |
| Heavy user, 1000+ characters, long conversations | Add pagination to character grid (FactoryVecDeque supports this). Add conversation list virtualization. |
| Power user, 10K+ messages per conversation | Tokenization and prompt assembly may need caching (cache assembled prompt, invalidate on persona/lorebook change). Consider moving token counting to a separate Worker if it becomes a bottleneck. |
| Many simultaneous AI requests (group chat) | AiService needs a request queue or per-provider Workers to avoid overwhelming a single tokio task. Group chat is the primary scaling concern. |

### Scaling Priorities

1. **First bottleneck:** Streaming token rendering — many small `update_cmd()` calls can cause GTK to repaint excessively. **Mitigation:** Batch tokens (accumulate 2-3 tokens before sending `TokenReceived`), or use `gtk::TextView` incremental updates instead of full `#[watch]` label replacement.
2. **Second bottleneck:** Database queries during prompt assembly — loading character + persona + lorebook + full history for each AI request. **Mitigation:** Cache domain objects in memory, invalidate on edit. The DbWorker can maintain an in-memory cache alongside the SQLite connection.
3. **Third bottleneck:** Group chat with multiple AI characters — N concurrent streaming requests. **Mitigation:** Serialize AI requests per-conversation (one AI responds at a time) or use tokio task spawning with a semaphore.

## Anti-Patterns

### Anti-Pattern 1: Blocking the GLib Main Loop

**What people do:** Call `rusqlite::Connection::query()` or `keyring::Entry::get_password()` directly in `update()`.
**Why it's wrong:** These are synchronous blocking calls. The GLib main loop freezes until they return. The UI becomes unresponsive.
**Do this instead:** Use `sender.command()` for async work, `sender.spawn_oneshot_command()` for sync work, or `Worker` for persistent background services. All three run off the GLib main loop.

### Anti-Pattern 2: oneshot_command for Streaming

**What people do:** Use `oneshot_command()` for AI inference, awaiting the full response before sending a single `CommandOutput`.
**Why it's wrong:** User sees nothing until the entire response is generated. For a 500-token response, this could be 10-30 seconds of blank screen.
**Do this instead:** Use `sender.command()` which provides a `Sender<CommandOutput>` for multiple messages. Send `TokenReceived` for each chunk, `StreamComplete` at the end.

### Anti-Pattern 3: Sharing rusqlite::Connection Across Threads

**What people do:** Wrap `Connection` in `Arc<Mutex<Connection>>` and share it.
**Why it's wrong:** rusqlite's `Connection` is `!Send` by design. SQLite itself requires serialized access unless using WAL mode with careful configuration. The mutex creates contention and deadlocks in async contexts.
**Do this instead:** Use a `Worker` that owns the connection on a single thread. Send messages to it. The worker processes them serially, which is exactly what SQLite wants.

### Anti-Pattern 4: Storing API Keys in SQLite

**What people do:** Store API keys in a `providers` table in the SQLite database.
**Why it's wrong:** Database files are unencrypted by default. API keys in plaintext on disk is a security vulnerability. Even with vault mode, keys shouldn't be in the same trust boundary as application data.
**Do this instead:** Store API keys exclusively in the OS keychain via the `keyring` crate. Store only a key ID/reference in the database, never the key itself.

### Anti-Pattern 5: Giant Monolithic AppModel

**What people do:** Put all state (characters, conversations, personas, settings, AI state) in one `AppModel` struct with one giant `AppMsg` enum.
**Why it's wrong:** The `view!` macro re-evaluates all `#[watch]` bindings on every update. A monolithic model means every message triggers every binding check. The enum becomes unwieldy (50+ variants). Update function becomes a massive match statement.
**Do this instead:** Decompose into child components. `ChatPanel` owns conversation state. `LibraryPanel` owns character state. `SettingsPanel` owns config state. The root `AppWindow` only holds routing state and shared references. Use the `forward()` pattern for inter-component communication.

### Anti-Pattern 6: Direct GTK Mutation Outside view!

**What people do:** Imperatively modify widgets (e.g., `widgets.label.set_text()`) in `update()` instead of using `#[watch]`.
**Why it's wrong:** Bypasses Relm4's declarative model. Can lead to stale state, double-updates, and hard-to-debug inconsistencies between model and view.
**Do this instead:** Always mutate model state in `update()`. Use `#[watch]` bindings in `view!` to declaratively render model state. Only use imperative widget access for operations that can't be expressed declaratively (e.g., scroll position, text buffer manipulation).

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| OpenAI / Anthropic / Gemini / etc. | rig provider clients via `sender.command()` | rig handles HTTP, SSE parsing, retries. No raw reqwest needed. |
| Ollama / LM Studio / vLLM | rig `openai` provider with custom base URL | Local endpoints use OpenAI-compatible API. rig supports custom `base_url`. |
| GNOME Keyring / macOS Keychain / Windows Credential Manager | `keyring` crate via `spawn_oneshot_command()` | Sync API; must run on background thread. Platform-specific backend selection is automatic. |
| SQLite | `rusqlite` via `Worker` | Bundled SQLite (feature flag). No external dependency. |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| UI ↔ Service | `sender.command()` / `oneshot_command()` | Commands run on tokio. Service functions take & return domain types. |
| UI ↔ Database | `Worker` input/output messages | Worker owns `Connection`. Serial execution on dedicated thread. |
| UI Panel ↔ UI Panel | `Controller::forward()` + parent `Input` messages | Child sends `Output` → parent transforms to `Input` → parent `update()`. Never direct widget access. |
| Service ↔ Database | Service calls DbWorker via `WorkerController::sender()` | AiService and PromptEngine do NOT talk to DB directly. UI orchestrates: loads data from DB, passes to service. |
| Service ↔ rig | Direct async function calls inside `command()` | rig agents are constructed per-request from current config. No persistent agent state. |
| PromptEngine ↔ AiService | PromptEngine returns `PromptResult`, AiService uses it to build rig agent | PromptEngine is pure. AiService is async. Clean boundary. |

## Build Order

The dependency graph between components dictates the order they must be built:

```
Phase 1: Domain Types + Database Schema
    domain/  →  db/models.rs  →  db/migrations
    (pure types, no deps)    (depends on domain types)

Phase 2: Database Worker
    db/ (full CRUD)  →  secrets/keyring.rs
    (depends on domain types + schema)

Phase 3: Service Layer
    service/token.rs  →  service/prompt.rs  →  service/ai.rs  →  service/import.rs
    (depends on domain types; token is standalone; prompt depends on token; ai depends on prompt)

Phase 4: UI Shared Components
    ui/components/ (markdown_view, token_gauge, search_bar)
    (depends on domain types; no service dependency)

Phase 5: UI Panels
    ui/chat/  →  ui/library/  →  ui/settings/
    (each depends on services + db worker + shared components)

Phase 6: Root App
    app.rs (wires all panels together)
    (depends on all panels)
```

### Build Order Rationale

1. **Domain types first** — everything depends on them, they depend on nothing. Pure Rust structs and enums.
2. **Database second** — models and CRUD need domain types. The Worker interface must be defined early because services need to know how to request data.
3. **Services third** — they transform domain types into rig API calls and prompt assemblies. They need domain types but don't touch the DB directly (UI orchestrates).
4. **Shared UI components fourth** — reusable widgets that don't know about specific panels. They can be tested independently.
5. **Panels fifth** — each panel is a self-contained Relm4 component that wires services and DB worker together. Can be built in parallel once the service layer is stable.
6. **Root app last** — minimal glue that assembles panels into the split view layout.

## Sources

- Relm4 Book — Component model, commands, workers, async components: https://relm4.org/book/stable/
- Relm4 ComponentSender API — `command()` method with `Sender` + `ShutdownReceiver`: https://docs.rs/relm4/0.10.0/relm4/struct.ComponentSender.html
- rig framework — Provider abstraction, streaming, agent builder: https://github.com/0xplaygrounds/rig
- gtk-rs-core — GLib MainContext, async integration: https://docs.rs/glib/
- rusqlite — Connection model, prepared statements: https://docs.rs/rusqlite/
- keyring crate — Cross-platform OS keychain: https://docs.rs/keyring/
- tiktoken-rs — BPE tokenization: https://docs.rs/tiktoken-rs/
- CharacterCardV2 spec — SillyTavern ecosystem character format

---
*Architecture research for: AI roleplay desktop application (Rust/GTK4/rig)*
*Researched: 2026-04-28*
