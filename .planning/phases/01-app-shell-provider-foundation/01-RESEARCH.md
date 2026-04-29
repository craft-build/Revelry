# Phase 1 Research: App Shell & Provider Foundation

**Researched:** 2026-04-29
**Confidence:** HIGH

## Standard Stack

| Category | Technology | Version | Role |
|----------|-----------|---------|------|
| UI Framework | gtk4 | 0.11.3 | GTK4 Rust bindings (v4_14 feature) |
| Adwaita | libadwaita | 0.9.1 | Adwaita widget bindings (v1_5 feature for breakpoints) |
| Declarative UI | relm4 | 0.11.0 | Elm Architecture framework (libadwaita + macros features) |
| Icons | relm4-icons | 0.11.0 | Bundled symbolic icons |
| State tracking | tracker | 0.2.2 | Fine-grained UI update tracking |
| AI Abstraction | rig-core | 0.36.0 | Provider-agnostic AI inference (derive feature) |
| Async Runtime | tokio | 1.52.1 | Async runtime (macros + rt-multi-thread features) |
| Stream utils | futures | 0.3.32 | StreamExt for async stream processing |
| Database | rusqlite | 0.39.0 | SQLite bindings (bundled + window + functions features) |
| Migrations | rusqlite_migration | 2.5.0 | Schema migration via user_version |
| Secrets | keyring | 3.6.3 | OS-native credential storage (platform-specific features) |
| Error handling | anyhow | 1.0.102 | Application error propagation |
| Error types | thiserror | 2.0.18 | Domain error type derivation |
| Logging | tracing | 0.1.44 | Structured logging |
| Log config | tracing-subscriber | 0.3.23 | Log output (env-filter feature) |
| CLI | clap | 4.6.1 | Command-line argument parsing (derive feature) |
| Paths | dirs | 6.0.0 | Platform directory paths |
| IDs | uuid | 1.23.1 | Unique identifiers (v4 feature) |
| Timestamps | chrono | 0.4.44 | Timestamp handling (serde feature) |
| Serialization | serde | 1.0.228 | Serialization framework (derive feature) |
| JSON | serde_json | 1.0.149 | JSON serialization |
| GLib core | glib | 0.22.6 | MainContext channel, spawn_future_local |
| GIO | gio | 0.22.6 | File monitoring, async I/O |
| Build tools | glib-build-tools | 0.22.0 | Build-time GResource compilation |

## dont_hand_roll

- **AI provider clients**: Use rig-core's built-in provider clients (OpenAI, Anthropic, Ollama, Gemini, etc.). Do NOT write custom reqwest HTTP clients per provider.
- **Streaming bridge**: Use Relm4's `sender.command()` with `Sender<CommandOutput>` for multi-message streaming. Do NOT manually set up `glib::MainContext::channel()`.
- **DB access**: Use Relm4's `Worker` trait with `detach_worker()` for background DB thread. Do NOT wrap `rusqlite::Connection` in `Arc<Mutex<>>`.
- **Schema migration**: Use `rusqlite_migration` crate with `user_version`. Do NOT hand-write migration tracking.
- **API key storage**: Use `keyring` crate for OS-native credential storage. Do NOT store keys in SQLite, config files, or env vars in production.
- **Icon resources**: Use `relm4-icons` bundled symbolic icons. Do NOT bundle custom SVGs for standard actions.
- **State tracking**: Use `tracker::track` macro on model structs. Do NOT manually implement change detection.

## common_pitfalls

1. **Blocking GTK main loop** — Never call rusqlite, keyring, or blocking I/O from `update()`. Always use `command()`, `spawn_oneshot_command()`, or `Worker`.
2. **Using oneshot_command for streaming** — Use `sender.command()` which provides `Sender<CommandOutput>` for multiple messages. `oneshot_command()` returns a single result.
3. **Sharing rusqlite::Connection across threads** — Connection is `!Send`. Use a `Worker` that owns the connection on its own thread.
4. **Storing API keys in SQLite** — Security vulnerability. Use `keyring` crate exclusively for API key storage.
5. **Giant monolithic AppModel** — Decompose into child components. Root `AppModel` only holds routing state and shared references.
6. **Direct GTK mutation outside view!** — Always mutate model state in `update()`. Use `#[watch]` in view! for declarative rendering.
7. **macOS Ctrl vs Cmd** — Use platform-aware keyboard shortcuts. macOS users expect Cmd, not Ctrl.
8. **keyring 4.0.0** — Released 2 days ago with empty feature flags, likely breaking. Pin to 3.6.3.

## architecture_patterns

### Pattern 1: Relm4 Component (Elm Architecture)

Every UI panel is a Relm4 `Component` with typed `Input`, `Output`, and `CommandOutput` message enums. The `update()` function is the only place that mutates state. The `view!` macro declaratively renders state.

```rust
#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = AppMsg;
    type Output = ();
    type CommandOutput = CommandMsg;
    // ...
}
```

### Pattern 2: Worker for DB Background Thread

The `Worker` trait provides a headless component running on a separate thread. The worker owns the `rusqlite::Connection` and processes DB messages one-at-a-time.

```rust
impl Worker for DbWorker {
    type Init = PathBuf;
    type Input = DbInput;
    type Output = DbOutput;

    fn init(db_path: Self::Init, sender: ComponentSender<Self>) -> Self {
        let conn = Connection::open(&db_path).expect("Failed to open database");
        run_migrations(&conn);
        DbWorker { conn }
    }

    fn update(&mut self, msg: DbInput, sender: ComponentSender<Self>) {
        let result = match msg {
            DbInput::LoadProviders => { /* ... */ }
            DbInput::SaveProvider(p) => { /* ... */ }
        };
        sender.output(result).unwrap();
    }
}
```

Launch with `detach_worker()`:
```rust
let db_worker = DbWorker::builder()
    .detach_worker(db_path)
    .forward(sender.input_sender(), AppMsg::DbOutput);
```

### Pattern 3: Command for Async/Background Operations

Use `sender.command()` for async work that returns multiple messages (streaming), and `sender.spawn_oneshot_command()` for one-shot synchronous work (keyring access, file I/O).

```rust
// One-shot synchronous command (keyring access)
sender.spawn_oneshot_command(|| {
    let entry = keyring::Entry::new("revelry", &provider_id);
    match entry.get_password() {
        Ok(key) => CommandMsg::KeyLoaded(provider_id, key),
        Err(e) => CommandMsg::KeyringError(e.to_string()),
    }
});
```

### Pattern 4: rig Provider Client Construction

rig clients are constructed per-request from stored config. For Phase 1, we only need health-check / model-list (not streaming chat — that's Phase 3).

```rust
// OpenAI with explicit API key (not from_env)
let client = openai::Client::new("sk-...");

// Ollama with custom base URL
let client = ollama::Client::from_url("http://localhost:11434");

// Anthropic with explicit API key
let client = anthropic::Client::new("sk-ant-...");
```

### Pattern 5: AdwOverlaySplitView with Breakpoint

The adaptive sidebar/content layout uses `AdwOverlaySplitView` with an `AdwBreakpoint` at 720px.

```rust
adw::OverlaySplitView {
    set_sidebar_width_fraction: 0.30,
    set_min_sidebar_width: 280,
    set_max_sidebar_width: 360,
    set_collapsed: false,

    #[wrap(Some)]
    set_breakpoints: adw::Breakpoint::new(adw::BreakpointCondition::new_length(
        adw::BreakpointConditionLengthType::MaxWidth,
        720.0,
        adw::LengthUnit::Pt,
    )) {
        add_setter: (&split_view, "collapsed", &true.into()),
    },
}
```

### Pattern 6: tracker for Efficient UI Updates

Use `#[tracker::track]` on model structs and `#[track]` in view! to only re-render changed widgets.

```rust
#[tracker::track]
struct AppModel {
    active_view: ActiveView,
    sidebar_visible: bool,
    // ...
}

// In view!:
#[track = "model.changed(AppModel::active_view())"]
set_visible: model.active_view == ActiveView::Chat,
```

## Integration Points

### tokio ↔ GLib Bridge
- Relm4 internally manages tokio runtime for commands
- `sender.command()` automatically marshals `CommandOutput` to GLib main loop
- `sender.spawn_oneshot_command()` runs sync work on thread pool, delivers result to GLib
- No manual `glib::MainContext::channel()` setup needed for Phase 1 (no streaming)

### rig Provider Health Check
- Build rig client from stored provider config (type, base_url, api_key)
- Call a lightweight endpoint (list models or verify endpoint) to test connectivity
- For Phase 1: use `client.agent(model_name).build()` then `agent.prompt("test").await` — or better, list available models via provider-specific API

### keyring for API Key Storage
- `keyring::Entry::new("revelry", &provider_id)` creates/opens a keychain entry
- `entry.set_password(&api_key)` stores the key
- `entry.get_password()` retrieves the key
- `entry.delete_credential()` removes the key
- Must handle `keyring::Error` (keychain unavailable, entry not found)
- Run on background thread via `spawn_oneshot_command()` (synchronous API)

### SQLite Bootstrap
- `Connection::open(path)` with `bundled` feature (zero external dependency)
- `rusqlite_migration::Migrations::new()` with migration files
- `PRAGMA journal_mode=WAL` for concurrent read performance
- `PRAGMA busy_timeout=5000` for lock contention
- Schema: `providers` table + `app_settings` table (Phase 1 minimal)

## Validation Architecture

### Testable Behaviors (Phase 1)

| Behavior | How to Verify |
|----------|--------------|
| App launches and shows main window | `cargo run` produces visible window with header bar |
| View switcher shows Chat/Characters/Settings | Header bar displays 3-tab view switcher |
| Sidebar collapses at narrow width | Resize window < 720px → sidebar goes to overlay |
| F9 toggles sidebar | Keyboard shortcut shows/hides sidebar |
| Provider can be added | Settings → Add Provider → fill fields → save → appears in list |
| API key stored in OS keychain | After saving provider, `keyring::Entry::new("revelry", &id).get_password()` returns the key |
| API key NOT in database | `SELECT * FROM providers` shows no key column |
| Provider test shows success/failure | Test Connection button → spinner → result indicator |
| Provider can be deleted | Delete button → confirmation → removed from list + keychain |
| Empty states display correctly | No providers → "Welcome to Revelry" in chat view |
| Keyboard navigation works | Ctrl+1/2/3 switch views, Ctrl+Comma opens settings, F9 toggles sidebar |

### Evaluation Dimensions

1. **Functional completeness** — All 5 success criteria from ROADMAP.md pass
2. **Security** — API keys never in DB/config/logs; keyring errors handled gracefully
3. **UX** — GNOME HIG compliance: content-first, progressive disclosure, Adwaita CSS classes
4. **Cross-platform** — macOS renders correctly, Cmd shortcuts work on macOS
5. **Code quality** — Worker pattern for DB, command pattern for keyring, no main-loop blocking
6. **Architecture** — Clean separation: domain types → DB worker → service layer → UI components
7. **Performance** — DB operations < 50ms, keyring < 100ms, UI responsive during all async ops
8. **Validation coverage** — All testable behaviors above have automated or manual test paths

---
*Phase 1 research for: Revelry — App Shell & Provider Foundation*
*Researched: 2026-04-29*
