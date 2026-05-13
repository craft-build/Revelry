# Phase 4: Provider Setup - Research

**Researched:** 2026-05-12
**Domain:** Rust keyring integration, Tauri IPC provider CRUD, Svelte 5 provider editor UI
**Confidence:** HIGH

## Summary

Phase 4 adds provider configuration with OS-native keychain storage for API keys, a 1:N provider-models relationship, and a full provider editor dialog in the settings page. The core technical challenge is the keyring crate migration from v3.6.3 (used in gtk_ref) to v4.0.x (pinned by CONTEXT.md D-02). This migration is a **major breaking change**: keyring 4.0 splits the library into `keyring-core` (Entry API) and separate platform-specific store crates, requires explicit store initialization before use, and requires cleanup on shutdown. The `Entry::new()`, `set_password()`, `get_password()`, and `delete_credential()` method signatures are compatible between v3 and v4, but the initialization and dependency model are completely different.

On the backend side, the provider CRUD follows the established DbWorker pattern with `DbCommand` variants and oneshot reply channels. The keyring module is standalone (D-03) — not routed through DbWorker — and uses `tokio::task::spawn_blocking` for its synchronous operations inside async Tauri commands. Schema migration #3 adds a `models` JSON TEXT column to `providers` and a `model_name` TEXT column to `conversations`, both safe `ALTER TABLE ADD COLUMN` operations in SQLite.

On the frontend side, the ProviderEditorDialog, ModelEntryRow, and KeyringUnavailableBanner are new domain components built from existing UI primitives (Dialog, Button, Input, Select, Tag, KebabMenu). The settings page is modified in-place to replace stub provider cards with real backend-wired cards. The AppState gains provider CRUD actions following the established optimistic-update pattern from Phase 3.

**Primary recommendation:** Use `keyring = "4.0.1"` + `keyring-core = "1.0.0"` together (not `keyring` alone). Call `keyring::use_native_store(true)` on app startup and `keyring::release_store()` on shutdown. Wrap all keyring operations in `tokio::task::spawn_blocking`.

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Use the `keyring` crate — same pattern as gtk_ref KeyringStore (service name "revelry", provider_id as username). Not a Tauri plugin.
- **D-02:** Pin `keyring` to version 4.0.0 (user preference over 3.6.3 from gtk_ref). Researcher/planner must verify 4.0.0 API stability and feature flag changes before implementing.
- **D-03:** KeyringStore as a standalone `src-tauri/src/secrets/` module — not routed through DbWorker. Clean separation; keyring ops are independent of DB.
- **D-04:** Dispatch keyring operations via `tokio::task::spawn_blocking()` inside Tauri async commands. Keyring ops are synchronous and blocking; spawn_blocking keeps Tauri commands async without a dedicated thread/channel infrastructure.
- **D-05:** Provider add/edit uses a Bits.ui Dialog — consistent with Phase 3 character/persona editor pattern. Opens over the settings page.
- **D-06:** Single `ProviderEditorDialog` component with `mode: 'create' | 'edit'` — on create, provider_type selector is active and base_url/model auto-fill from ProviderType defaults; on edit, provider_type is locked (read-only), base_url and model list are editable.
- **D-07:** API key field is inline in the provider dialog — part of the same save action. Key is saved to OS keychain when the dialog is saved. For Ollama (no key required), the field is optional/hidden.
- **D-08:** ProviderType selector uses grouped select with optgroups: Cloud, Local, Gateway, Custom.
- **D-09:** When OS keyring is unavailable: show a persistent warning banner on the settings page, disable the API key field in the provider editor, but allow provider creation (Ollama works without a key). The app remains functional — keyring failure is not fatal.
- **D-10:** Check keyring availability once on app startup — attempt to create and delete a test entry. Cache the result (`keyring_available: bool`). Show a persistent banner in settings if unavailable. Avoids checking on every provider operation.
- **D-11:** 1:N provider-models — one provider entry can have multiple models under the same API key.
- **D-12:** Store models as a JSON TEXT column on the `providers` table — `models TEXT NOT NULL DEFAULT '[]'`. Each model entry: `{ name: string, context_window_size: number | null, enabled: boolean }`. No separate table.
- **D-13:** Generation parameters (temperature, top_p, max_tokens, etc.) are shared across all models under a provider — one set of `default_params` per provider. Per-model params are not supported.
- **D-14:** Conversations reference a specific model via `provider_id` (existing FK) + a new `model_name TEXT` column on the `conversations` table. A model is identified by the (provider_id, model_name) pair. No surrogate model_id.
- **D-15:** Add `models TEXT NOT NULL DEFAULT '[]'` column to the `providers` table (Phase 4 migration #3). The existing `model_name` column on providers becomes the "default" or "primary" model for backward compat, OR is deprecated in favor of the models array. Planner decides the cleanest approach.
- **D-16:** Add `model_name TEXT` column to the `conversations` table (Phase 4 migration #3). Alongside existing `provider_id` FK, this identifies the specific model used for the conversation.

### Claude's Discretion

- Exact layout of the provider editor dialog fields (grouping, order, spacing)
- Whether `model_name` column on `providers` table should be kept (for primary/default model) or removed in favor of the `models` JSON array
- Exact wording of the keyring-unavailable banner message
- Whether to add a `has_api_key` boolean field returned by the backend (so the frontend can show "key set" / "no key" without ever receiving the actual key) — likely yes for UX
- Exact keyring test entry naming convention for the startup check

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>

## Phase Requirements

| ID      | Description                                                                                                    | Research Support                                                                                        |
| ------- | -------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| PROV-01 | User can add an AI provider with name, type, and API key                                                       | ProviderEditorDialog create mode + Tauri `save_provider` command + `save_api_key` keyring command       |
| PROV-02 | User can edit provider configuration                                                                           | ProviderEditorDialog edit mode + Tauri `save_provider` command                                          |
| PROV-03 | User can delete a provider                                                                                     | KebabMenu delete action + `delete_provider` DB command + `delete_api_key` keyring command (best-effort) |
| PROV-04 | API keys are stored in the OS-native keychain (never on disk)                                                  | KeyringStore module using keyring-core 1.0 / keyring 4.0.1 Entry API, `tokio::task::spawn_blocking`     |
| PROV-05 | User can select a provider and model for a conversation                                                        | `model_name` column on conversations + provider_id FK; model selection UI deferred to Phase 5           |
| PROV-06 | User can configure generation parameters (temperature, top_p, max_tokens, frequency_penalty, presence_penalty) | Generation params in ProviderEditorDialog + stored in providers table                                   |

</phase_requirements>

## Architectural Responsibility Map

| Capability                 | Primary Tier                        | Secondary Tier                     | Rationale                                                                                                             |
| -------------------------- | ----------------------------------- | ---------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| API key storage/retrieval  | OS Keychain (via Rust keyring-core) | —                                  | Security invariant: keys never touch disk or IPC. Only the Rust backend can access the keychain.                      |
| Provider CRUD persistence  | Backend (Rust/SQLite via DbWorker)  | —                                  | Database operations follow established DbWorker pattern. Provider data (not keys) stored in SQLite.                   |
| Keyring availability check | Backend (Rust keyring)              | Frontend (state display)           | Rust performs the actual keychain probe; frontend only stores/display the boolean result.                             |
| Provider type defaults     | Backend (Rust ProviderType enum)    | Frontend (type selector rendering) | `default_base_url()` and `default_model_name()` live in Rust; frontend receives a lookup map or computes client-side. |
| Provider editor UI         | Frontend (Svelte)                   | —                                  | Dialog, form validation, model list management are all client-side. Only persists on Save.                            |
| `has_api_key` indicator    | Backend (Rust, probes keyring)      | Frontend (display only)            | Rust checks if a key exists for a provider_id and returns the boolean; never sends the key to frontend.               |
| Schema migration           | Backend (Rust/SQLite)               | —                                  | `ALTER TABLE ADD COLUMN` in migration #3. No frontend involvement.                                                    |

## Standard Stack

### Core (Backend — Rust)

| Library            | Version                               | Purpose                                                                              | Why Standard                                                                                             |
| ------------------ | ------------------------------------- | ------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------- |
| keyring            | 4.0.1                                 | Store management: `use_native_store()`, `release_store()`                            | [VERIFIED: crates.io] Convenience wrapper for platform store selection. bundles all platform stores.     |
| keyring-core       | 1.0.0                                 | Entry API: `Entry::new()`, `set_password()`, `get_password()`, `delete_credential()` | [VERIFIED: crates.io + docs.rs] The actual credential management library. keyring 4.0 delegates to this. |
| tokio              | 1.x (features: sync, rt-multi-thread) | Async runtime + `spawn_blocking` for keyring ops                                     | [VERIFIED: Cargo.toml] Already in project; needs `rt-multi-thread` feature added for `spawn_blocking`.   |
| rusqlite           | 0.31 (features: bundled)              | SQLite provider CRUD via DbWorker                                                    | [VERIFIED: Cargo.toml] Already in project; provider CRUD follows existing pattern.                       |
| serde + serde_json | 1.x                                   | ProviderType serialization, models JSON column                                       | [VERIFIED: Cargo.toml] Already in project.                                                               |

### Core (Frontend — Svelte)

| Library           | Version | Purpose                                                          | Why Standard                                                |
| ----------------- | ------- | ---------------------------------------------------------------- | ----------------------------------------------------------- |
| bits-ui           | 2.x     | Dialog, Select headless primitives for ProviderEditorDialog      | [VERIFIED: AGENTS.md] Project's headless component library. |
| @lucide/svelte    | 1.x     | Icons: AlertTriangle, Check, Cpu, Plus, MoreVertical, Loader2, X | [VERIFIED: AGENTS.md] Project's icon library.               |
| tailwind-variants | 3.x     | Component variant definitions if needed                          | [VERIFIED: AGENTS.md] Project's variant system.             |
| @tauri-apps/api   | 2.x     | `invoke()` for Tauri IPC                                         | [VERIFIED: api.ts] Already in project.                      |

### Supporting

| Library                           | Version | Purpose                                | When to Use                                                                   |
| --------------------------------- | ------- | -------------------------------------- | ----------------------------------------------------------------------------- |
| apple-native-keyring-store        | 1.0.0   | macOS Keychain Services backend        | Pulled in automatically by keyring 4.0.1 — no direct dependency needed        |
| linux-keyutils-keyring-store      | 1.0.0   | Linux kernel keyutils backend          | Pulled in automatically by keyring 4.0.1                                      |
| dbus-secret-service-keyring-store | 1.0.0   | Linux Secret Service (libdbus) backend | Pulled in automatically by keyring 4.0.1 — used when `use_native_store(true)` |
| zbus-secret-service-keyring-store | 1.0.0   | Linux Secret Service (zbus) backend    | Pulled in automatically by keyring 4.0.1                                      |
| windows-native-keyring-store      | 1.0.0   | Windows Credential Manager backend     | Pulled in automatically by keyring 4.0.1                                      |

### Alternatives Considered

| Instead of                         | Could Use                       | Tradeoff                                                                                                                                                            |
| ---------------------------------- | ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| keyring 4.0.1 + keyring-core 1.0.0 | keyring 3.6.3 (gtk_ref version) | 3.6.3 has simpler API (no store init needed) but is the "final release of v3" and won't receive updates. D-02 explicitly pins to 4.0.0.                             |
| keyring 4.0.1 + keyring-core 1.0.0 | tauri-plugin-keyring            | Would add a Tauri plugin dependency. D-01 explicitly says "Not a Tauri plugin."                                                                                     |
| keyring 4.0.1 + keyring-core 1.0.0 | keyring-core 1.0.0 alone        | keyring-core doesn't include platform store crates or `use_native_store()`. Would need manual store selection per platform. Using keyring 4.0.1 as well is simpler. |

**Installation:**

```toml
# In src-tauri/Cargo.toml, add:
keyring = "4.0"
keyring-core = "1.0"

# Update tokio features:
tokio = { version = "1", features = ["sync", "rt-multi-thread"] }
```

**Version verification:**

- keyring 4.0.1 [VERIFIED: crates.io — published 2026-05-12]
- keyring-core 1.0.0 [VERIFIED: crates.io + docs.rs]
- tokio 1.x [VERIFIED: Cargo.toml already has `1` with `sync` feature]

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    Frontend (Svelte 5)                       │
│                                                              │
│  ┌──────────────────┐    ┌────────────────────────────────┐ │
│  │ Settings Page     │    │ ProviderEditorDialog           │ │
│  │ - Provider cards   │───▶│ - Name, Type, Base URL        │ │
│  │ - Keyring banner  │    │ - API Key input                │ │
│  │ - Add provider    │    │ - Models list (ModelEntryRow)   │ │
│  │ - KebabMenu       │    │ - Generation params grid       │ │
│  └────────┬─────────┘    └──────────────┬─────────────────┘ │
│           │                             │                     │
│  ┌────────▼─────────┐                  │                     │
│  │ AppState          │                  │ onsave()            │
│  │ - providers[]     │                  │                     │
│  │ - keyring_available│                 │                     │
│  │ - createProvider() │                │                     │
│  │ - updateProvider() │                │                     │
│  │ - deleteProvider() │                │                     │
│  └────────┬─────────┘                  │                     │
│           │                             │                     │
│  ┌────────▼─────────┐                  │                     │
│  │ api.ts            │◀─────────────────┘                     │
│  │ invoke wrappers   │                                        │
│  └────────┬─────────┘                                        │
└───────────┼──────────────────────────────────────────────────┘
            │ Tauri IPC (invoke)
            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Backend (Rust/Tauri)                       │
│                                                              │
│  ┌──────────────────┐    ┌────────────────────────────────┐ │
│  │ Tauri Commands    │    │ secrets/ module (standalone)   │ │
│  │ - get_providers   │    │ - KeyringStore::save_key()     │ │
│  │ - save_provider   │───▶│ - KeyringStore::load_key()     │ │
│  │ - delete_provider │    │ - KeyringStore::delete_key()   │ │
│  │ - save_api_key    │    │ - check_keyring_available()   │ │
│  │ - load_api_key    │    │                                │ │
│  │ - delete_api_key  │    │ Uses: keyring::use_native_store│ │
│  │ - check_keyring   │    │        keyring_core::Entry     │ │
│  └────────┬─────────┘    │        spawn_blocking()        │ │
│           │              └────────────────────────────────┘ │
│           │                                                  │
│  ┌────────▼─────────┐                                       │
│  │ DbWorker (mpsc)  │                                       │
│  │ - LoadProviders  │                                       │
│  │ - SaveProvider   │                                       │
│  │ - DeleteProvider │                                       │
│  └────────┬─────────┘                                       │
│           │                                                  │
│  ┌────────▼─────────┐    ┌────────────────────────────────┐ │
│  │ SQLite            │    │ OS Keychain                     │ │
│  │ - providers table │    │ - macOS Keychain Services       │ │
│  │ - conversations   │    │ - Linux Secret Service/Keyutils│ │
│  │   (model_name)   │    │ - Windows Credential Manager    │ │
│  └──────────────────┘    └────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure

```
src-tauri/src/
├── commands/
│   ├── providers.rs          # NEW: provider + keyring Tauri commands
│   └── mod.rs                # UPDATE: add `pub mod providers;`
├── db/
│   ├── providers.rs          # NEW: provider CRUD SQL operations
│   ├── migrations.rs         # UPDATE: add migration #3
│   ├── models.rs             # UPDATE: add row_to_provider() mapper
│   └── mod.rs                # UPDATE: add DbCommand variants, pub mod providers
├── domain/
│   ├── provider.rs           # NEW: ProviderConfig, ProviderType, ProviderModelEntry
│   └── mod.rs                # UPDATE: add `pub mod provider;`
├── secrets/
│   └── mod.rs                # NEW: KeyringStore (save/load/delete/check)
└── lib.rs                    # UPDATE: register new commands, init keyring

src/lib/
├── components/
│   ├── providers/             # NEW directory
│   │   ├── ProviderEditorDialog.svelte
│   │   ├── ModelEntryRow.svelte
│   │   └── KeyringUnavailableBanner.svelte
│   └── ui/
│       └── index.ts          # UPDATE: barrel exports if needed
├── stores/
│   └── app.svelte.ts         # UPDATE: add provider CRUD, keyring_available
├── types/
│   └── index.ts              # UPDATE: add ProviderModelEntry, update Provider/Conversation
└── data/
    └── api.ts                # UPDATE: add provider/keyring invoke wrappers

src/routes/
├── +layout.svelte            # UPDATE: call checkKeyringAvailable during hydration
└── settings/
    └── +page.svelte          # UPDATE: wire provider cards, editor dialog, banner
```

### Pattern 1: Keyring v4 Store Initialization (NEW — required by v4 API)

**What:** keyring 4.0 requires explicit store initialization before `Entry::new()` can be called. The v3 API did this implicitly.

**When to use:** Once at app startup, before any keyring operations.

**Example:**

```rust
// Source: [CITED: docs.rs/keyring/4.0.1] + [CITED: github.com/open-source-cooperative/keyring-rs/wiki/Keyring-Core]
use keyring;      // for use_native_store(), release_store()
use keyring_core; // for Entry

/// Called once during Tauri app initialization.
/// `not_keyutils = true` on Linux means use Secret Service (GNOME Keyring)
/// instead of kernel keyutils. Secret Service is more widely available
/// on desktop Linux environments and matches user expectations.
pub fn init_keyring() -> Result<(), String> {
    keyring::use_native_store(true)
        .map_err(|e| format!("Failed to initialize keyring: {}", e))
}

/// Called once during Tauri app shutdown.
pub fn cleanup_keyring() {
    keyring::release_store();
}
```

### Pattern 2: KeyringStore with spawn_blocking (adapted from gtk_ref)

**What:** Port the gtk_ref KeyringStore to v4 API, wrapping synchronous calls in `spawn_blocking`.

**When to use:** Every keyring operation inside async Tauri commands.

**Example:**

```rust
// Source: adapted from gtk_ref/src/secrets/mod.rs with v4 API
use anyhow::Result;
use keyring_core::Entry;

pub struct KeyringStore;

impl KeyringStore {
    const SERVICE_NAME: &'static str = "revelry";

    pub fn save_key(provider_id: &str, api_key: &str) -> Result<()> {
        let entry = Entry::new(Self::SERVICE_NAME, provider_id)?;
        entry.set_password(api_key)?;
        Ok(())
    }

    pub fn load_key(provider_id: &str) -> Result<String> {
        let entry = Entry::new(Self::SERVICE_NAME, provider_id)?;
        let password = entry.get_password()?;
        Ok(password)
    }

    pub fn delete_key(provider_id: &str) -> Result<()> {
        let entry = Entry::new(Self::SERVICE_NAME, provider_id)?;
        entry.delete_credential()?;
        Ok(())
    }

    /// Check if keyring is available by creating and deleting a test entry.
    pub fn is_available() -> bool {
        let test_user = "__keyring_test__";
        if let Ok(entry) = Entry::new(Self::SERVICE_NAME, test_user) {
            // Try to set and delete a test password
            if entry.set_password("test").is_ok() {
                let _ = entry.delete_credential();
                return true;
            }
        }
        false
    }
}
```

### Pattern 3: Tauri Command with spawn_blocking for Keyring

**What:** Async Tauri command that delegates synchronous keyring work to a blocking thread pool.

**When to use:** All keyring Tauri commands.

**Example:**

```rust
// Source: [CITED: docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html]
#[tauri::command]
pub async fn save_api_key(provider_id: String, api_key: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        crate::secrets::KeyringStore::save_key(&provider_id, &api_key)
    })
    .await
    .map_err(|e| format!("Keyring task failed: {}", e))?
    .map_err(|e| format!("Failed to save API key: {}", e))
}

#[tauri::command]
pub async fn check_keyring_available() -> Result<bool, String> {
    tokio::task::spawn_blocking(|| {
        Ok(crate::secrets::KeyringStore::is_available())
    })
    .await
    .map_err(|e| format!("Keyring check failed: {}", e))
}
```

### Pattern 4: Provider CRUD via DbWorker (follows existing pattern)

**What:** Provider CRUD goes through the existing DbWorker background-thread pattern.

**When to use:** All provider database operations.

**Example:**

```rust
// In DbCommand enum (db/mod.rs):
LoadProviders { reply: oneshot::Sender<anyhow::Result<Vec<ProviderConfig>>> },
SaveProvider { provider: ProviderConfig, reply: oneshot::Sender<anyhow::Result<ProviderConfig>> },
DeleteProvider { id: String, reply: oneshot::Sender<anyhow::Result<()>> },

// In Tauri commands (commands/providers.rs):
#[tauri::command]
pub async fn get_providers(db: State<'_, DbWorker>) -> Result<Vec<ProviderConfig>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadProviders { reply: reply_tx })
        .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}
```

### Pattern 5: has_api_key Backend Endpoint

**What:** Backend returns a boolean indicating whether a key exists for a provider, without ever sending the key to the frontend.

**When to use:** Every provider list load; individual provider load.

**Example:**

```rust
// In Tauri commands:
#[tauri::command]
pub async fn has_api_key(provider_id: String) -> Result<bool, String> {
    tokio::task::spawn_blocking(move || {
        match crate::secrets::KeyringStore::load_key(&provider_id) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),  // key doesn't exist or keyring unavailable
        }
    })
    .await
    .map_err(|e| format!("Keyring check failed: {}", e))
}
```

### Anti-Patterns to Avoid

- **Never send API keys through Tauri IPC.** The frontend should never receive the actual key value. Only `has_api_key: boolean` is sent. [SECURITY: PROV-04]
- **Never store API keys in SQLite.** The `providers` table must NOT have an `api_key` column. [SECURITY: existing migration test verifies this]
- **Never call keyring Entry methods without store initialization.** keyring v4 requires `use_native_store()` before `Entry::new()`. The v3 implicit initialization no longer works.
- **Never call keyring operations directly on the async runtime thread.** They are synchronous and blocking; use `spawn_blocking`.
- **Never forget `release_store()` on shutdown.** keyring v4 requires explicit cleanup. Without it, some backends (especially Linux Secret Service) may leak resources.

## Don't Hand-Roll

| Problem                     | Don't Build                                                                      | Use Instead                                                           | Why                                                                                                                                                             |
| --------------------------- | -------------------------------------------------------------------------------- | --------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| OS keychain access          | Custom FFI to macOS Keychain / Linux Secret Service / Windows Credential Manager | keyring 4.0.1 + keyring-core 1.0.0                                    | Platform-specific keychain APIs are complex, error-prone, and security-sensitive. keyring handles all the edge cases.                                           |
| Keyring availability check  | Try/catch around Entry::new() without cleanup                                    | KeyringStore::is_available() with test entry create+delete            | Must be a real round-trip: create → verify → delete. Otherwise, a "no keyring daemon" scenario may silently succeed at Entry::new() but fail at set_password(). |
| Provider type defaults      | Hardcoded lookup table in TypeScript                                             | ProviderType Rust enum with default_base_url() / default_model_name() | Single source of truth in Rust. Frontend can receive a defaults map via a Tauri command.                                                                        |
| JSON column storage         | Custom serde for models TEXT column                                              | `serde_json::to_string()` / `serde_json::from_str()`                  | Same pattern as tags, alternate_greetings, extensions columns already in the codebase.                                                                          |
| spawn_blocking for sync ops | Custom thread pool or channel for keyring                                        | `tokio::task::spawn_blocking()`                                       | tokio's built-in blocking thread pool. No need to reinvent.                                                                                                     |

**Key insight:** The keyring v4 architecture split into `keyring-core` (library) + `keyring` (store management + convenience) is the canonical way to use v4. Don't try to use just `keyring` or just `keyring-core` alone.

## Common Pitfalls

### Pitfall 1: Using keyring v4 without store initialization

**What goes wrong:** `Entry::new()` returns `NoDefaultStore` error. In v3, the default store was implicit.
**Why it happens:** keyring v4 requires `use_native_store()` or another `use_*_store()` call before any `Entry::new()`.
**How to avoid:** Call `keyring::use_native_store(true)` during Tauri app setup, before any keyring commands can be invoked.
**Warning signs:** `NoDefaultStore` error from keyring-core.

### Pitfall 2: Forgetting release_store() on shutdown

**What goes wrong:** Linux Secret Service connections may not be properly cleaned up, causing resource leaks.
**Why it happens:** keyring v4 requires explicit cleanup. v3 didn't need this.
**How to avoid:** Call `keyring::release_store()` during Tauri app shutdown.
**Warning signs:** D-Bus connection warnings on Linux; no visible issue on macOS/Windows (but still required for correctness).

### Pitfall 3: Using keyring 4.0.0 instead of 4.0.1

**What goes wrong:** iOS sample code bug (doesn't affect desktop, but 4.0.1 is the latest stable).
**Why it happens:** CONTEXT.md D-02 says "4.0.0" but 4.0.1 is the actual latest with bug fixes.
**How to avoid:** Pin to `keyring = "4.0"` (SemVer range allows 4.0.1) or explicitly `"4.0.1"`.
**Warning signs:** None for desktop — but using the latest patch is best practice.

### Pitfall 4: Not adding rt-multi-thread tokio feature

**What goes wrong:** `spawn_blocking` requires the `rt-multi-thread` or `rt` feature. Current Cargo.toml only has `sync`.
**Why it happens:** `tokio::task::spawn_blocking` needs a multi-threaded runtime. The `sync` feature alone doesn't include the runtime.
**How to avoid:** Update tokio dependency: `tokio = { version = "1", features = ["sync", "rt-multi-thread"] }`. Tauri 2 itself may already pull this in, but explicit is better.
**Warning signs:** Compile error: `spawn_blocking` is unavailable or runtime panic about missing runtime.

### Pitfall 5: Provider delete doesn't clean up keyring entry

**What goes wrong:** After deleting a provider from SQLite, the API key remains in the OS keychain forever.
**Why it happens:** The keyring operation is independent of the DB operation and must be called separately.
**How to avoid:** In the `delete_provider` Tauri command, after successful DB deletion, call `KeyringStore::delete_key()` as best-effort (ignore errors if key doesn't exist or keyring is unavailable).
**Warning signs:** Orphaned keychain entries accumulating after provider deletions.

### Pitfall 6: keyring Entry::new() service name mismatch

**What goes wrong:** If the service name changes between saves and loads, keys can't be found.
**Why it happens:** Service name is part of the keyring entry identity (service + username pair).
**How to avoid:** Use a single constant `SERVICE_NAME = "revelry"` in KeyringStore. Never change it.
**Warning signs:** `NoEntry` errors when loading keys that were definitely saved.

### Pitfall 7: ALTER TABLE ADD COLUMN with DEFAULT value

**What goes wrong:** SQLite's ALTER TABLE is limited — you can't add a column with a non-constant DEFAULT in older versions, but SQLite 3.25+ supports it.
**Why it happens:** rusqlite's bundled SQLite is always a recent version (3.39+), so `ALTER TABLE ... ADD COLUMN ... DEFAULT '[]'` is supported.
**How to avoid:** Use the bundled SQLite feature (already enabled). Test migration with `Connection::open_in_memory()`.
**Warning signs:** Migration failure on systems with very old system SQLite (but bundled feature avoids this).

### Pitfall 8: ProviderType serde mismatch between Rust and TypeScript

**What goes wrong:** Rust serializes `ProviderType::OpenAI` as `"OpenAI"` (PascalCase) but TypeScript expects `"openai"` (snake_case).
**Why it happens:** gtk_ref's `serde_json::to_string(&ProviderType::OpenAI)` produces `"OpenAI"` by default. The frontend `ProviderType` uses lowercase snake_case values.
**How to avoid:** Use `#[serde(rename_all = "snake_case")]` on the Rust `ProviderType` enum, or implement custom serialization. Verify round-trip matches the TypeScript union type.
**Warning signs:** Frontend receives `"OpenAI"` but the `ProviderType` union only matches `"openai"`.

## Code Examples

### KeyringStore Module (src-tauri/src/secrets/mod.rs)

```rust
// Source: adapted from gtk_ref/src/secrets/mod.rs for keyring v4 API
use anyhow::Result;
use keyring_core::Entry;

/// Manages API keys in the OS-native credential store.
///
/// API keys are stored exclusively in the OS keychain and NEVER in the database.
/// This is a core security invariant (PROV-05).
///
/// All keyring operations are synchronous and MUST run via
/// `tokio::task::spawn_blocking()` to avoid blocking the Tauri async runtime.
pub struct KeyringStore;

impl KeyringStore {
    const SERVICE_NAME: &'static str = "revelry";

    pub fn save_key(provider_id: &str, api_key: &str) -> Result<()> {
        let entry = Entry::new(Self::SERVICE_NAME, provider_id)?;
        entry.set_password(api_key)?;
        Ok(())
    }

    pub fn load_key(provider_id: &str) -> Result<String> {
        let entry = Entry::new(Self::SERVICE_NAME, provider_id)?;
        Ok(entry.get_password()?)
    }

    pub fn delete_key(provider_id: &str) -> Result<()> {
        let entry = Entry::new(Self::SERVICE_NAME, provider_id)?;
        entry.delete_credential()?;
        Ok(())
    }

    /// Check if the OS keyring is available by attempting
    /// a round-trip: create → write → read → delete a test entry.
    pub fn is_available() -> bool {
        let test_user = "__keyring_test__";
        if let Ok(entry) = Entry::new(Self::SERVICE_NAME, test_user) {
            if entry.set_password("availability-check").is_ok() {
                // Verify we can read it back
                if entry.get_password().is_ok() {
                    // Clean up the test entry
                    let _ = entry.delete_credential();
                    return true;
                }
                let _ = entry.delete_credential();
            }
        }
        false
    }
}
```

### Provider Domain Model (src-tauri/src/domain/provider.rs)

```rust
// Source: adapted from gtk_ref/src/domain/provider.rs with 1:N models support
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]  // CRITICAL: match TypeScript ProviderType values
pub enum ProviderType {
    OpenAI,
    Anthropic,
    Ollama,
    Gemini,
    Cohere,
    Mistral,
    DeepSeek,
    Groq,
    AzureOpenAI,
    OpenRouter,
    Xai,
    Perplexity,
    Together,
    HuggingFace,
    CustomOpenAICompatible,
}

/// A single model entry within a provider's models list.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderModelEntry {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window_size: Option<i64>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool { true }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    #[serde(rename = "provider_type")]
    pub provider_type: ProviderType,
    pub base_url: Option<String>,
    /// The primary/default model name (kept for backward compat and quick reference).
    pub model_name: String,
    /// JSON array of model entries stored in the `models` TEXT column.
    #[serde(default = "Vec::new")]
    pub models: Vec<ProviderModelEntry>,
    /// User override for context window size.
    #[serde(default)]
    pub context_window_size: Option<i64>,
    pub enabled: bool,
    /// Default generation parameters stored as JSON.
    #[serde(default)]
    pub default_params: Option<GenerationParams>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerationParams {
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub max_tokens: Option<i64>,
    pub frequency_penalty: Option<f64>,
    pub presence_penalty: Option<f64>,
}
```

### Schema Migration #3 (addition to migrations.rs)

```sql
-- Source: [VERIFIED: SQLite ALTER TABLE ADD COLUMN compatibility]
-- Migration #3: Add models JSON column to providers, model_name to conversations

ALTER TABLE providers ADD COLUMN models TEXT NOT NULL DEFAULT '[]';
ALTER TABLE providers ADD COLUMN default_params TEXT;
ALTER TABLE conversations ADD COLUMN model_name TEXT;
```

### TypeScript Type Updates (src/lib/types/index.ts)

```typescript
// Source: aligned with Rust ProviderConfig serde output
export interface ProviderModelEntry {
  name: string;
  context_window_size?: number | null;
  enabled: boolean;
}

export interface Provider {
  id: string;
  name: string;
  provider_type: ProviderType;
  base_url?: string;
  model_name: string; // primary/default model
  models: ProviderModelEntry[]; // all models
  default_params?: GenerationParams;
  has_api_key: boolean; // from backend, never the actual key
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface Conversation {
  id: string;
  character_id: string;
  provider_id?: string;
  model_name?: string; // NEW: specific model within provider
  creator_persona_id?: string;
  generation_preset_id?: string;
  title: string;
  last_message_preview: string;
  created_at: string;
  updated_at: string;
}
```

## State of the Art

| Old Approach                                                                | Current Approach                                                                                                    | When Changed                        | Impact                                                                                                         |
| --------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- | ----------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| keyring 3.6.3 (single crate with feature flags)                             | keyring 4.0 + keyring-core 1.0 (split architecture)                                                                 | 2026-04-26 (keyring v4.0.0 release) | Must use both crates; must call `use_native_store()` before Entry ops; must call `release_store()` on shutdown |
| keyring v3 `apple-native` / `linux-native` / `windows-native` feature flags | Separate store crates (apple-native-keyring-store, linux-keyutils-keyring-store, etc.) — all bundled by keyring 4.0 | 2026-04-26                          | No feature flags needed on `keyring` crate itself. All platform stores are unconditional dependencies.         |
| keyring v3 implicit store initialization                                    | keyring v4 explicit `use_native_store()` call                                                                       | 2026-04-26                          | Must add init call at app startup                                                                              |
| ProviderType PascalCase serde                                               | `#[serde(rename_all = "snake_case")]`                                                                               | Phase 4 design                      | Must match TypeScript `ProviderType` union values                                                              |

**Deprecated/outdated:**

- keyring 3.x series: "Likely final release of v3" per maintainer. No future updates. [CITED: github.com/open-source-cooperative/keyring-rs/releases/tag/v3.6.3-republish]
- `keyring::Entry` (v3): Moved to `keyring_core::Entry` in v4. Import path changed.
- keyring v3 feature flags (`apple-native`, `linux-native`, `windows-native`): No longer exist in v4. All platform stores are pulled in automatically.

## Assumptions Log

| #   | Claim                                                                                                                   | Section               | Risk if Wrong                                                                                                             |
| --- | ----------------------------------------------------------------------------------------------------------------------- | --------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| A1  | `keyring::use_native_store(true)` is the correct initialization call for all platforms (macOS, Linux, Windows)          | Architecture Patterns | If wrong: keyring ops fail at runtime. [VERIFIED: docs.rs/keyring/4.0.1 confirms `use_native_store` selects per-platform] |
| A2  | `tokio::task::spawn_blocking` requires `rt-multi-thread` feature (not just `rt`)                                        | Pitfall 4             | If wrong: compile error or runtime panic. [VERIFIED: Context7 tokio docs confirm spawn_blocking needs runtime]            |
| A3  | SQLite `ALTER TABLE ADD COLUMN ... DEFAULT '[]'` works with rusqlite bundled SQLite                                     | Pitfall 7             | If wrong: migration fails. [VERIFIED: SQLite 3.25+ supports this; bundled SQLite is 3.39+]                                |
| A4  | `ProviderType` Rust serde with `rename_all = "snake_case"` will produce `"openai"` (not `"OpenAI"`) matching TypeScript | Pitfall 8             | If wrong: frontend can't match provider type strings. [ASSUMED — must verify in implementation]                           |
| A5  | `keyring 4.0.1` is a suitable replacement for `4.0.0` (pinned in D-02)                                                  | Pitfall 3             | If wrong: no real risk — 4.0.1 is just an iOS sample code bug fix. [VERIFIED: GitHub release notes]                       |
| A6  | Tauri 2 already depends on tokio with rt-multi-thread feature                                                           | Pitfall 4             | If wrong: need to explicitly add the feature. [ASSUMED — needs Cargo.toml verification]                                   |
| A7  | `keyring::release_store()` can safely be called even if `use_native_store()` was never called                           | Architecture Patterns | If wrong: shutdown may panic. [ASSUMED — needs verification]                                                              |

## Open Questions

1. **Should D-02 pin be `keyring = "4.0"` or `keyring = "4.0.1"`?**
   - What we know: 4.0.1 is the latest with an iOS bug fix; 4.0.0 works fine for desktop.
   - What's unclear: Whether D-02's explicit "4.0.0" means exact pin or SemVer range.
   - Recommendation: Use `keyring = "4.0"` (allows 4.0.1) or `keyring = "4.0.1"` explicitly. Either way, the `keyring-core = "1.0"` dependency is also required.

2. **Does Tauri 2 already provide tokio with rt-multi-thread?**
   - What we know: The current Cargo.toml has `tokio = { version = "1", features = ["sync"] }`.
   - What's unclear: Whether Tauri's own dependency on tokio brings in rt-multi-thread.
   - Recommendation: Add `rt-multi-thread` to the explicit tokio features list to be safe.

3. **How to deliver ProviderType defaults to the frontend?**
   - What we know: The Rust enum has `default_base_url()` and `default_model_name()`. The frontend needs these for auto-fill when the user selects a provider type in the editor.
   - What's unclear: Whether to add a `get_provider_defaults` Tauri command that returns a JSON map, or hardcode the map in TypeScript.
   - Recommendation: Add a `get_provider_defaults` command — single source of truth in Rust, frontend stays thin.

4. **Should the `model_name` column on `providers` be kept or removed?**
   - What we know: CONTEXT.md D-15 says "planner decides." The `models` JSON array is the canonical source.
   - What's unclear: Whether having both `model_name` (scalar) and `models` (array) is confusing.
   - Recommendation: Keep `model_name` as the "primary" or "default" model for quick reference. The `models[0].name` should equal `model_name` for consistency. This avoids breaking any future code that queries `model_name` directly.

## Environment Availability

| Dependency       | Required By    | Available | Version         | Fallback |
| ---------------- | -------------- | --------- | --------------- | -------- |
| Rust/Cargo       | Backend build  | ✓         | 1.95.0          | —        |
| Bun              | Frontend build | ✓         | 1.3.13          | —        |
| Node.js          | Tauri tooling  | ✓         | 25.9.0          | —        |
| Tauri CLI        | Build/deploy   | ✓         | 2.11.1          | —        |
| macOS Keychain   | KeyringStore   | ✓         | — (native)      | —        |
| SQLite (bundled) | DB operations  | ✓         | 3.39+ (bundled) | —        |

**Missing dependencies with no fallback:** None — all required tools are available.

**Missing dependencies with fallback:** None.

## Validation Architecture

### Test Framework

| Property           | Value                                                            |
| ------------------ | ---------------------------------------------------------------- | --------- |
| Framework          | Rust: cargo test (built-in); Svelte: Vitest (not yet configured) |
| Config file        | Cargo.toml (Rust); none for Svelte yet                           |
| Quick run command  | `cargo test -p revelry --lib 2>&1                                | head -30` |
| Full suite command | `cargo test -p revelry`                                          |

### Phase Requirements → Test Map

| Req ID  | Behavior                                 | Test Type                       | Automated Command                                          | File Exists?                 |
| ------- | ---------------------------------------- | ------------------------------- | ---------------------------------------------------------- | ---------------------------- |
| PROV-01 | Save provider + API key to DB + keychain | unit (Rust) + integration (IPC) | `cargo test -p revelry --lib test_insert_provider`         | ❌ Wave 0                    |
| PROV-02 | Update provider in DB                    | unit (Rust)                     | `cargo test -p revelry --lib test_update_provider`         | ❌ Wave 0                    |
| PROV-03 | Delete provider from DB + keyring        | unit (Rust)                     | `cargo test -p revelry --lib test_delete_provider`         | ❌ Wave 0                    |
| PROV-04 | API key stored only in OS keychain       | unit (Rust)                     | `cargo test -p revelry --lib test_no_api_key_column`       | ✅ Existing (migration test) |
| PROV-05 | Conversation references provider + model | unit (Rust)                     | `cargo test -p revelry --lib test_conversation_model_name` | ❌ Wave 0                    |
| PROV-06 | Generation params stored/retrieved       | unit (Rust)                     | `cargo test -p revelry --lib test_generation_params`       | ❌ Wave 0                    |

### Sampling Rate

- **Per task commit:** `cargo test -p revelry --lib`
- **Per wave merge:** `cargo test -p revelry`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `src-tauri/src/db/providers.rs` — test provider CRUD with models JSON column
- [ ] `src-tauri/src/domain/provider.rs` — test ProviderType serde roundtrip with snake_case
- [ ] `src-tauri/src/secrets/mod.rs` — test KeyringStore save/load/delete (requires OS keychain)
- [ ] Migration #3 test — verify `models` and `model_name` columns exist after migration
- [ ] `src-tauri/src/db/migrations.rs` — add test for migration #3 columns on providers and conversations

## Security Domain

### Applicable ASVS Categories

| ASVS Category         | Applies | Standard Control                                                 |
| --------------------- | ------- | ---------------------------------------------------------------- |
| V2 Authentication     | no      | No user auth — single-user desktop app                           |
| V3 Session Management | no      | No sessions — desktop app                                        |
| V4 Access Control     | no      | No multi-user access                                             |
| V5 Input Validation   | yes     | zod/TypeScript strict for frontend; Rust type system for backend |
| V6 Cryptography       | yes     | keyring crate for OS-native keychain (never hand-roll crypto)    |

### Known Threat Patterns for Provider Setup

| Pattern                                  | STRIDE                 | Standard Mitigation                                                              |
| ---------------------------------------- | ---------------------- | -------------------------------------------------------------------------------- |
| API key leakage via IPC payloads         | Information Disclosure | Never return API key in Tauri command responses; only `has_api_key: bool`        |
| API key leakage via SQLite               | Information Disclosure | No `api_key` column in providers table (verified by existing migration test)     |
| API key leakage via logs                 | Information Disclosure | Never log API key values; only log success/failure of keyring operations         |
| Keyring unavailability on headless Linux | Denial of Service      | Graceful degradation: banner warning, disabled API key field, Ollama still works |
| SQL injection in provider CRUD           | Tampering              | Parameterized queries via rusqlite `params![]` macro                             |

## Sources

### Primary (HIGH confidence)

- [CITED: docs.rs/keyring/4.0.1] — keyring 4.0.1 API: `use_native_store()`, `release_store()`, store functions
- [CITED: docs.rs/keyring-core/1.0.0] — Entry struct API: `new()`, `set_password()`, `get_password()`, `delete_credential()`
- [CITED: github.com/open-source-cooperative/keyring-rs/releases] — v4.0.0 release notes, v4.0.1 release notes, v3.6.3 "final release" notes
- [VERIFIED: crates.io] — keyring 4.0.1, keyring-core 1.0.0 verified on registry
- [VERIFIED: codebase] — gtk_ref/src/secrets/mod.rs, gtk_ref/src/domain/provider.rs, gtk_ref/src/db/providers.rs

### Secondary (MEDIUM confidence)

- [CITED: docs.rs/tokio/latest] — spawn_blocking documentation and examples
- [CITED: SQLite documentation] — ALTER TABLE ADD COLUMN compatibility

### Tertiary (LOW confidence)

- [ASSUMED] ProviderType `#[serde(rename_all = "snake_case")]` will produce lowercase values matching TypeScript — needs compile-time verification
- [ASSUMED] Tauri 2's tokio dependency includes rt-multi-thread — needs Cargo.toml verification

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — keyring 4.0.1 + keyring-core 1.0.0 verified on crates.io and docs.rs; all other deps already in project
- Architecture: HIGH — follows established DbWorker pattern; spawn_blocking pattern verified in tokio docs; KeyringStore port is straightforward
- Pitfalls: HIGH — keyring v4 breaking changes identified and documented with official sources; ProviderType serde mismatch identified

**Research date:** 2026-05-12
**Valid until:** 2026-06-12 (30 days — stable libraries, no fast-moving deps)
