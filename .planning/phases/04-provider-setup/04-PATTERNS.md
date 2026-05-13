# Phase 4: Provider Setup - Pattern Map

**Mapped:** 2026-05-12
**Files analyzed:** 19
**Analogs found:** 19 / 19

## File Classification

| New/Modified File                                              | Role       | Data Flow        | Closest Analog                                                  | Match Quality                      |
| -------------------------------------------------------------- | ---------- | ---------------- | --------------------------------------------------------------- | ---------------------------------- |
| `src-tauri/src/secrets/mod.rs`                                 | service    | request-response | `gtk_ref/src/secrets/mod.rs`                                    | exact (port with v4 API updates)   |
| `src-tauri/src/domain/provider.rs`                             | model      | CRUD             | `gtk_ref/src/domain/provider.rs`                                | exact (extend with models/params)  |
| `src-tauri/src/db/providers.rs`                                | service    | CRUD             | `src-tauri/src/db/characters.rs`                                | role-match (active CRUD pattern)   |
| `src-tauri/src/commands/providers.rs`                          | controller | request-response | `src-tauri/src/commands/characters.rs`                          | role-match (Tauri command pattern) |
| `src-tauri/Cargo.toml`                                         | config     | —                | `src-tauri/Cargo.toml`                                          | exact (in-place modification)      |
| `src-tauri/src/db/migrations.rs`                               | migration  | batch            | `src-tauri/src/db/migrations.rs` lines 193-243                  | exact (Phase 3 migration pattern)  |
| `src-tauri/src/db/models.rs`                                   | utility    | transform        | `src-tauri/src/db/models.rs` row_to_character                   | exact (in-place addition)          |
| `src-tauri/src/db/mod.rs`                                      | config     | event-driven     | `src-tauri/src/db/mod.rs`                                       | exact (in-place modification)      |
| `src-tauri/src/domain/mod.rs`                                  | config     | —                | `src-tauri/src/domain/mod.rs`                                   | exact (in-place modification)      |
| `src-tauri/src/commands/mod.rs`                                | config     | —                | `src-tauri/src/commands/mod.rs`                                 | exact (in-place modification)      |
| `src-tauri/src/lib.rs`                                         | config     | —                | `src-tauri/src/lib.rs`                                          | exact (in-place modification)      |
| `src/lib/types/index.ts`                                       | model      | —                | `src/lib/types/index.ts`                                        | exact (in-place modification)      |
| `src/lib/stores/app.svelte.ts`                                 | service    | CRUD             | `src/lib/stores/app.svelte.ts`                                  | exact (in-place modification)      |
| `src/lib/data/api.ts`                                          | utility    | request-response | `src/lib/data/api.ts`                                           | exact (in-place modification)      |
| `src/routes/settings/+page.svelte`                             | component  | request-response | `src/routes/settings/+page.svelte`                              | exact (in-place modification)      |
| `src/routes/+layout.svelte`                                    | component  | request-response | `src/routes/+layout.svelte`                                     | exact (in-place modification)      |
| `src/lib/components/providers/ProviderEditorDialog.svelte`     | component  | request-response | `src/lib/components/personas/PersonaEditorDialog.svelte`        | role-match                         |
| `src/lib/components/providers/ModelEntryRow.svelte`            | component  | CRUD             | `src/lib/components/lorebooks/LorebookEditor.svelte` entry rows | role-match                         |
| `src/lib/components/providers/KeyringUnavailableBanner.svelte` | component  | —                | PersonaEditorDialog error banner pattern                        | partial                            |

## Pattern Assignments

### `src-tauri/src/secrets/mod.rs` (service, request-response)

**Analog:** `gtk_ref/src/secrets/mod.rs`

**Imports pattern** (lines 1-2):

```rust
use anyhow::Result;
use keyring_core::Entry;  // CHANGED: v4 uses keyring_core::Entry, not keyring::Entry
```

**Core pattern — KeyringStore struct with SERVICE_NAME constant** (lines 14-17):

```rust
pub struct KeyringStore;

impl KeyringStore {
    const SERVICE_NAME: &'static str = "revelry";
```

**Core CRUD — save_key / load_key / delete_key** (lines 22-46):

```rust
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
```

**NEW — is_available() method** (from RESEARCH.md Pattern 2):

```rust
    /// Check if the OS keyring is available by attempting
    /// a round-trip: create → write → read → delete a test entry.
    pub fn is_available() -> bool {
        let test_user = "__keyring_test__";
        if let Ok(entry) = Entry::new(Self::SERVICE_NAME, test_user) {
            if entry.set_password("availability-check").is_ok() {
                if entry.get_password().is_ok() {
                    let _ = entry.delete_credential();
                    return true;
                }
                let _ = entry.delete_credential();
            }
        }
        false
    }
```

**Test pattern** (lines 49-103):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn test_provider_id() -> String {
        format!("test-provider-{}", Uuid::new_v4())
    }

    #[test]
    fn test_save_and_load() {
        let provider_id = test_provider_id();
        let api_key = "sk-test-key-12345";
        KeyringStore::save_key(&provider_id, api_key).expect("save_key should succeed");
        let loaded = KeyringStore::load_key(&provider_id).expect("load_key should succeed");
        assert_eq!(loaded, api_key);
        let _ = KeyringStore::delete_key(&provider_id);
    }
    // ... test_delete_removes_key, test_load_nonexistent ...
}
```

**Key differences from gtk_ref:**

- Import `keyring_core::Entry` instead of `keyring::Entry` (v4 API split)
- Add `is_available()` method for startup keyring check
- No changes to save/load/delete signatures — they're identical between v3 and v4

---

### `src-tauri/src/domain/provider.rs` (model, CRUD)

**Analog:** `gtk_ref/src/domain/provider.rs`

**Imports pattern** (adapted from gtk_ref + active codebase style):

```rust
use chrono::{DateTime, Utc};  // CHANGED: active codebase uses DateTime<Utc>, gtk_ref uses NaiveDateTime
use serde::{Deserialize, Serialize};
use uuid::Uuid;
```

**ProviderType enum — add snake_case serde** (from gtk_ref lines 7-24, MODIFIED):

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]  // NEW: match TypeScript ProviderType values
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
    CustomOpenAICompatible,  // RENAMED from Custom
}
```

**ProviderType helper methods** (from gtk_ref lines 26-77, keep as-is):

```rust
impl ProviderType {
    pub fn requires_api_key(&self) -> bool {
        matches!(self, ProviderType::Ollama) == false  // or explicit match
    }

    pub fn default_base_url(&self) -> Option<&'static str> {
        match self {
            ProviderType::OpenAI => Some("https://api.openai.com/v1"),
            // ... same as gtk_ref ...
            ProviderType::AzureOpenAI => None,
            ProviderType::CustomOpenAICompatible => None,
        }
    }

    pub fn default_model_name(&self) -> &'static str {
        match self {
            ProviderType::OpenAI => "gpt-4o-mini",
            // ... same as gtk_ref ...
        }
    }
}
```

**NEW — ProviderModelEntry struct:**

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderModelEntry {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window_size: Option<i64>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool { true }
```

**NEW — GenerationParams struct:**

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerationParams {
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub max_tokens: Option<i64>,
    pub frequency_penalty: Option<f64>,
    pub presence_penalty: Option<f64>,
}
```

**ProviderConfig struct** (from gtk_ref lines 81-98, EXTENDED):

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    #[serde(rename = "provider_type")]
    pub provider_type: ProviderType,
    pub base_url: Option<String>,
    pub model_name: String,
    #[serde(default = "Vec::new")]
    pub models: Vec<ProviderModelEntry>,       // NEW: 1:N models
    #[serde(default)]
    pub context_window_size: Option<i64>,
    pub enabled: bool,
    #[serde(default)]
    pub default_params: Option<GenerationParams>,  // NEW: generation params
    pub created_at: DateTime<Utc>,            // CHANGED: active codebase uses DateTime<Utc>
    pub updated_at: DateTime<Utc>,
}
```

**ProviderConfig::new()** (from gtk_ref lines 104-117, adapted for DateTime<Utc>):

```rust
impl ProviderConfig {
    pub fn new(name: String, provider_type: ProviderType) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            base_url: provider_type.default_base_url().map(|s| s.to_string()),
            model_name: provider_type.default_model_name().to_string(),
            models: vec![ProviderModelEntry {
                name: provider_type.default_model_name().to_string(),
                context_window_size: None,
                enabled: true,
            }],
            context_window_size: None,
            provider_type,
            enabled: true,
            default_params: None,
            created_at: now,
            updated_at: now,
        }
    }
}
```

---

### `src-tauri/src/db/providers.rs` (service, CRUD)

**Analog:** `src-tauri/src/db/characters.rs` (active codebase CRUD pattern)

**Imports pattern** (from characters.rs lines 1-4):

```rust
use crate::db::models::row_to_provider;
use crate::domain::provider::ProviderConfig;
use anyhow::Result;
use rusqlite::{params, Connection};
```

**Insert pattern** (from characters.rs lines 9-50, adapted):

```rust
pub fn insert_provider(conn: &Connection, provider: &ProviderConfig) -> Result<()> {
    let provider_type_str = serde_json::to_string(&provider.provider_type)?;
    let models_str = serde_json::to_string(&provider.models)?;
    let default_params_str = match &provider.default_params {
        Some(p) => serde_json::to_string(p)?,
        None => "null".to_string(),
    };

    conn.prepare_cached(
        "INSERT INTO providers (id, name, provider_type, base_url, model_name, models,
         context_window_size, default_params, enabled, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
    )?
    .execute(params![
        provider.id,
        provider.name,
        provider_type_str,
        provider.base_url,
        provider.model_name,
        models_str,
        provider.context_window_size,
        default_params_str,
        provider.enabled as i32,
        provider.created_at.to_rfc3339(),
        provider.updated_at.to_rfc3339(),
    ])?;
    Ok(())
}
```

**Load pattern** (from characters.rs lines 53-68):

```rust
pub fn load_providers(conn: &Connection) -> Result<Vec<ProviderConfig>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, name, provider_type, base_url, model_name, models,
         context_window_size, default_params, enabled, created_at, updated_at
         FROM providers ORDER BY updated_at DESC",
    )?;
    let providers = stmt.query_map([], |row| row_to_provider(row))?;
    let mut result = Vec::new();
    for provider in providers {
        result.push(provider?);
    }
    Ok(result)
}
```

**Load single / Update / Delete** — follow same pattern as characters.rs lines 71-135.

**Key conventions from active codebase:**

- Use `to_rfc3339()` for timestamp serialization (not `.to_string()`)
- Use `parse_timestamp()` helper from models.rs for deserialization
- JSON columns use `serde_json::to_string()` for serialization
- `enabled` stored as `i32`, converted with `as i32` on write and `!= 0` on read

---

### `src-tauri/src/commands/providers.rs` (controller, request-response)

**Analog:** `src-tauri/src/commands/characters.rs`

**Imports pattern** (from characters.rs lines 1-4, extended):

```rust
use tauri::State;

use crate::db::{DbCommand, DbWorker};
use crate::domain::provider::ProviderConfig;
```

**DB-backed command pattern** (from characters.rs lines 10-19):

```rust
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

**NEW — Keyring commands with spawn_blocking** (from RESEARCH.md Pattern 3):

```rust
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
pub async fn has_api_key(provider_id: String) -> Result<bool, String> {
    tokio::task::spawn_blocking(move || {
        match crate::secrets::KeyringStore::load_key(&provider_id) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    })
    .await
    .map_err(|e| format!("Keyring check failed: {}", e))
}

#[tauri::command]
pub async fn delete_api_key(provider_id: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        crate::secrets::KeyringStore::delete_key(&provider_id)
    })
    .await
    .map_err(|e| format!("Keyring task failed: {}", e))?
    .map_err(|e| format!("Failed to delete API key: {}", e))
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

**Save provider command** (follows characters.rs save_character pattern):

```rust
#[tauri::command]
pub async fn save_provider(
    db: State<'_, DbWorker>,
    provider: ProviderConfig,
) -> Result<ProviderConfig, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::SaveProvider { provider, reply: reply_tx })
        .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}
```

**Delete provider command** — also deletes keyring entry (best-effort):

```rust
#[tauri::command]
pub async fn delete_provider(db: State<'_, DbWorker>, id: String) -> Result<(), String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::DeleteProvider { id: id.clone(), reply: reply_tx })
        .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())?;

    // Best-effort keyring cleanup
    let keyring_id = id;
    let _ = tokio::task::spawn_blocking(move || {
        crate::secrets::KeyringStore::delete_key(&keyring_id)
    }).await;

    Ok(())
}
```

---

### `src-tauri/Cargo.toml` (config, —)

**Analog:** `src-tauri/Cargo.toml` (in-place modification)

**Current tokio line** (line 31):

```toml
tokio = { version = "1", features = ["sync"] }
```

**Updated tokio + new keyring deps:**

```toml
tokio = { version = "1", features = ["sync", "rt-multi-thread"] }
keyring = "4.0"
keyring-core = "1.0"
```

---

### `src-tauri/src/db/migrations.rs` (migration, batch)

**Analog:** `src-tauri/src/db/migrations.rs` Phase 3 migration (lines 193-243)

**Phase 3 migration pattern** (lines 193-243):

```rust
        // Phase 3: standalone lorebooks + avatar_path on personas
        M::up(
            "
            -- Add avatar_path column to personas
            ALTER TABLE personas ADD COLUMN avatar_path TEXT;

            -- Drop old lorebook_entries table
            -- ... etc
            "
        )
        .down(
            "DROP TABLE IF EXISTS character_lorebooks;
             -- ... etc"
        ),
```

**Phase 4 migration #3 — add columns to providers and conversations:**

```rust
        // Phase 4: provider models + conversation model_name
        M::up(
            "
            -- Add models JSON column to providers (1:N provider-models)
            ALTER TABLE providers ADD COLUMN models TEXT NOT NULL DEFAULT '[]';

            -- Add default_params JSON column to providers (generation params)
            ALTER TABLE providers ADD COLUMN default_params TEXT;

            -- Add model_name to conversations (provider_id + model_name pair)
            ALTER TABLE conversations ADD COLUMN model_name TEXT;
            "
        )
        .down(
            "
            ALTER TABLE conversations DROP COLUMN model_name;
            ALTER TABLE providers DROP COLUMN default_params;
            ALTER TABLE providers DROP COLUMN models;
            "
        ),
```

**Test pattern — verify new columns exist** (follow existing migration tests, e.g. `test_personas_has_avatar_path` lines 672-685):

```rust
    #[test]
    fn test_providers_has_models_column() {
        let conn = setup_test_db();
        let columns: Vec<String> = conn
            .prepare("SELECT * FROM providers")
            .expect("Failed to prepare")
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(
            columns.iter().any(|c| c == "models"),
            "providers table should have 'models' column after Phase 4 migration"
        );
    }

    #[test]
    fn test_conversations_has_model_name() {
        let conn = setup_test_db();
        let columns: Vec<String> = conn
            .prepare("SELECT * FROM conversations")
            .expect("Failed to prepare")
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(
            columns.iter().any(|c| c == "model_name"),
            "conversations table should have 'model_name' column after Phase 4 migration"
        );
    }
```

---

### `src-tauri/src/db/models.rs` (utility, transform)

**Analog:** `src-tauri/src/db/models.rs` row_to_character/row_to_persona + `gtk_ref/src/db/models.rs` row_to_provider

**Existing timestamp parsing helper** (lines 11-19):

```rust
fn parse_timestamp(s: &str) -> rusqlite::Result<DateTime<Utc>> {
    s.parse::<DateTime<Utc>>().map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        )
    })
}
```

**NEW — row_to_provider mapper** (adapted from gtk_ref row_to_provider for active codebase conventions):

```rust
use crate::domain::provider::{ProviderConfig, ProviderType, ProviderModelEntry, GenerationParams};

pub fn row_to_provider(row: &Row) -> rusqlite::Result<ProviderConfig> {
    let created_at_str: String = row.get("created_at")?;
    let updated_at_str: String = row.get("updated_at")?;
    let created_at = parse_timestamp(&created_at_str)?;
    let updated_at = parse_timestamp(&updated_at_str)?;

    // ProviderType from JSON TEXT column
    let provider_type_str: String = row.get("provider_type")?;
    let provider_type: ProviderType = serde_json::from_str(&provider_type_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    // models JSON TEXT column
    let models_str: String = row.get("models")?;
    let models: Vec<ProviderModelEntry> = serde_json::from_str(&models_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    // default_params JSON TEXT column (nullable)
    let default_params: Option<GenerationParams> = {
        let default_params_str: Option<String> = row.get("default_params")?;
        match default_params_str {
            Some(s) if s != "null" => serde_json::from_str(&s).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
            })?,
            _ => None,
        }
    };

    Ok(ProviderConfig {
        id: row.get("id")?,
        name: row.get("name")?,
        provider_type,
        base_url: row.get("base_url")?,
        model_name: row.get("model_name")?,
        models,
        context_window_size: row.get("context_window_size")?,
        enabled: row.get::<_, i32>("enabled")? != 0,
        default_params,
        created_at,
        updated_at,
    })
}
```

**Also update row_to_conversation** to include `model_name`:

```rust
pub fn row_to_conversation(row: &Row) -> rusqlite::Result<Conversation> {
    // ... existing fields ...
    Ok(Conversation {
        id: row.get("id")?,
        character_id: row.get("character_id")?,
        provider_id: row.get("provider_id")?,
        model_name: row.get("model_name")?,  // NEW
        creator_persona_id: row.get("creator_persona_id")?,
        generation_preset_id: row.get("generation_preset_id")?,
        title: row.get("title")?,
        last_message_preview: row.get("last_message_preview")?,
        created_at,
        updated_at,
    })
}
```

---

### `src-tauri/src/db/mod.rs` (config, event-driven)

**Analog:** `src-tauri/src/db/mod.rs`

**Add pub mod** (line 1 area):

```rust
pub mod providers;  // NEW
```

**Add import** (line 10 area):

```rust
use crate::domain::provider::ProviderConfig;  // NEW
```

**Add DbCommand variants** (after existing lorebook variants, around line 137):

```rust
    // Provider operations
    LoadProviders {
        reply: oneshot::Sender<anyhow::Result<Vec<ProviderConfig>>>,
    },
    SaveProvider {
        provider: ProviderConfig,
        reply: oneshot::Sender<anyhow::Result<ProviderConfig>>,
    },
    DeleteProvider {
        id: String,
        reply: oneshot::Sender<anyhow::Result<()>>,
    },
```

**Add handle_command match arms** (follow existing SaveCharacter pattern, lines 215-224):

```rust
            DbCommand::LoadProviders { reply } => {
                let result = providers::load_providers(conn);
                let _ = reply.send(result);
            }
            DbCommand::SaveProvider { provider, reply } => {
                let result = if providers::load_provider(conn, &provider.id)
                    .unwrap_or(None)
                    .is_some()
                {
                    providers::update_provider(conn, &provider).map(|_| provider.clone())
                } else {
                    providers::insert_provider(conn, &provider).map(|_| provider.clone())
                };
                let _ = reply.send(result);
            }
            DbCommand::DeleteProvider { id, reply } => {
                let result = providers::delete_provider(conn, &id);
                let _ = reply.send(result);
            }
```

---

### `src-tauri/src/domain/mod.rs` (config, —)

**Analog:** `src-tauri/src/domain/mod.rs`

**Add provider module** (after line 4):

```rust
pub mod provider;  // NEW
```

**Add re-export** (after line 13):

```rust
pub use provider::{ProviderConfig, ProviderType, ProviderModelEntry, GenerationParams};  // NEW
```

---

### `src-tauri/src/commands/mod.rs` (config, —)

**Analog:** `src-tauri/src/commands/mod.rs`

**Add providers module** (after line 5):

```rust
pub mod providers;  // NEW
```

---

### `src-tauri/src/lib.rs` (config, —)

**Analog:** `src-tauri/src/lib.rs`

**Add secrets module** (after line 3):

```rust
mod secrets;  // NEW
```

**Add keyring initialization** (before tauri::Builder, after DbWorker::start):

```rust
    // Initialize keyring store (v4 requires explicit init before Entry::new())
    if let Err(e) = keyring::use_native_store(true) {
        tracing::warn!("Failed to initialize keyring: {}. Keyring features will be disabled.", e);
    }
```

**Register new commands** (extend invoke_handler, lines 29-58):

```rust
        .invoke_handler(tauri::generate_handler![
            // ... existing commands ...
            commands::providers::get_providers,
            commands::providers::save_provider,
            commands::providers::delete_provider,
            commands::providers::save_api_key,
            commands::providers::has_api_key,
            commands::providers::delete_api_key,
            commands::providers::check_keyring_available,
        ])
```

---

### `src/lib/types/index.ts` (model, —)

**Analog:** `src/lib/types/index.ts`

**Add ProviderModelEntry interface** (after ProviderType, around line 157):

```typescript
export interface ProviderModelEntry {
  name: string;
  context_window_size?: number | null;
  enabled: boolean;
}
```

**Update Provider interface** (replace lines 158-166):

```typescript
export interface Provider {
  id: string;
  name: string;
  provider_type: ProviderType;
  base_url?: string;
  model_name: string; // RENAMED from model, primary/default model
  models: ProviderModelEntry[]; // NEW: all models under this provider
  context_window_size?: number; // NEW
  default_params?: GenerationParams;
  has_api_key: boolean; // NEW: from backend, never the actual key
  enabled: boolean; // NEW
  created_at: string; // NEW
  updated_at: string; // NEW
}
```

**Update Conversation interface** (add model_name, around line 121-135):

```typescript
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

**Update GenerationParams** (add context_length → context_window_size alignment):

```typescript
export interface GenerationParams {
  temperature?: number;
  top_p?: number;
  max_tokens?: number;
  frequency_penalty?: number;
  presence_penalty?: number;
  stop_sequences?: string[]; // keep for frontend
  context_length?: number; // keep for frontend compat
}
```

---

### `src/lib/stores/app.svelte.ts` (service, CRUD)

**Analog:** `src/lib/stores/app.svelte.ts`

**Add keyring_available state** (after line 56):

```typescript
keyring_available = $state<boolean>(true); // NEW
```

**Add provider API imports** (extend lines 10-27):

```typescript
import {
  // ... existing imports ...
  getProviders as apiGetProviders,
  saveProvider as apiSaveProvider,
  deleteProvider as apiDeleteProvider,
  checkKeyringAvailable as apiCheckKeyringAvailable,
  saveApiKey as apiSaveApiKey,
  deleteApiKey as apiDeleteApiKey,
} from "$lib/data/api";
```

**Add getProviders to hydrate** (extend Promise.allSettled around line 114):

```typescript
const results = await Promise.allSettled([
  apiGetCharacters(),
  apiGetPersonas(),
  apiGetLorebooks(),
  apiGetConversations(),
  apiGetPresets(),
  apiGetProviders(), // NEW
]);
```

**Add provider hydration handler** (after presets handler, around line 153):

```typescript
if (providers.status === "fulfilled") {
  this.providers = providers.value;
} else {
  console.error("Failed to load providers:", providers.reason);
}
```

**Add provider CRUD actions** (follow character CRUD pattern, lines 247-331):

```typescript
  // ── Provider CRUD ──────────────────────────────────────────────────────

  async createProvider(
    data: Omit<Provider, "id" | "created_at" | "updated_at" | "has_api_key"> & { id?: string; api_key?: string },
  ): Promise<Provider> {
    const provider: Provider = {
      id: data.id ?? crypto.randomUUID(),
      name: data.name,
      provider_type: data.provider_type,
      base_url: data.base_url,
      model_name: data.model_name,
      models: data.models ?? [],
      context_window_size: data.context_window_size,
      default_params: data.default_params,
      has_api_key: false,
      enabled: data.enabled ?? true,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    try {
      // Save provider to DB
      const saved = await apiSaveProvider(provider);
      // Save API key to keyring if provided
      if (data.api_key) {
        try {
          await apiSaveApiKey(saved.id, data.api_key);
          saved.has_api_key = true;
        } catch (err) {
          console.error("Failed to save API key to keyring:", err);
        }
      }
      this.providers = [saved, ...this.providers];
      return saved;
    } catch (err) {
      console.error("Failed to create provider:", err);
      throw err;
    }
  }

  async updateProvider(provider: Provider, apiKey?: string): Promise<Provider> {
    const prev = this.providers.find((p) => p.id === provider.id);
    try {
      const saved = await apiSaveProvider({
        ...provider,
        updated_at: new Date().toISOString(),
      });
      // Update API key if provided
      if (apiKey !== undefined) {
        try {
          await apiSaveApiKey(saved.id, apiKey);
          saved.has_api_key = true;
        } catch (err) {
          console.error("Failed to update API key:", err);
        }
      }
      this.providers = this.providers.map((p) =>
        p.id === saved.id ? saved : p,
      );
      return saved;
    } catch (err) {
      console.error("Failed to update provider:", err);
      if (prev) {
        this.providers = this.providers.map((p) =>
          p.id === prev.id ? prev : p,
        );
      }
      throw err;
    }
  }

  async deleteProvider(id: string): Promise<void> {
    try {
      await apiDeleteProvider(id);
      this.providers = this.providers.filter((p) => p.id !== id);
    } catch (err) {
      console.error("Failed to delete provider:", err);
      throw err;
    }
  }

  async checkKeyring(): Promise<void> {
    try {
      this.keyring_available = await apiCheckKeyringAvailable();
    } catch (err) {
      this.keyring_available = false;
      console.error("Failed to check keyring availability:", err);
    }
  }
```

---

### `src/lib/data/api.ts` (utility, request-response)

**Analog:** `src/lib/data/api.ts`

**Add provider invoke wrappers** (after Lorebook entries section, around line 169):

```typescript
// ── Providers ───────────────────────────────────────────────────────────────

export async function getProviders(): Promise<Provider[]> {
  return invoke<Provider[]>("get_providers");
}

export async function saveProvider(provider: Provider): Promise<Provider> {
  return invoke<Provider>("save_provider", { provider });
}

export async function deleteProvider(id: string): Promise<void> {
  return invoke<void>("delete_provider", { id });
}

// ── Keyring ──────────────────────────────────────────────────────────────────

export async function saveApiKey(
  providerId: string,
  apiKey: string,
): Promise<void> {
  return invoke<void>("save_api_key", { providerId, apiKey });
}

export async function hasApiKey(providerId: string): Promise<boolean> {
  return invoke<boolean>("has_api_key", { providerId });
}

export async function deleteApiKey(providerId: string): Promise<void> {
  return invoke<void>("delete_api_key", { providerId });
}

export async function checkKeyringAvailable(): Promise<boolean> {
  return invoke<boolean>("check_keyring_available");
}
```

**Update type imports** (line 9-19):

```typescript
import type {
  // ... existing types ...
  Provider,
  ProviderModelEntry, // NEW
} from "$lib/types";
```

---

### `src/routes/settings/+page.svelte` (component, request-response)

**Analog:** `src/routes/settings/+page.svelte`

**Key changes:**

1. Import `ProviderEditorDialog` and `KeyringUnavailableBanner`
2. Replace stub provider cards with real data from `app.providers`
3. Wire Add/Edit/Delete buttons
4. Add keyring banner when `!app.keyring_available`
5. Replace mock API key display with `has_api_key` boolean display

**Updated provider section imports** (extend lines 1-14):

```svelte
<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import { Button, Input, Tag, KebabMenu, KebabMenuItem, KebabMenuSeparator } from '$lib/components/ui';
  import { ProviderEditorDialog } from '$lib/components/providers';
  import { KeyringUnavailableBanner } from '$lib/components/providers';
  import { DeleteConfirmationDialog } from '$lib/components/characters';
  import { Cpu, SlidersHorizontal, Palette, KeyRound, Database, Info, Plus, Eye, EyeOff, Check, X } from '@lucide/svelte';

  // ... sections array stays the same ...

  let active = $state<string>('providers');
  let editor_open = $state(false);
  let editor_mode = $state<'create' | 'edit'>('create');
  let editor_provider = $state<Provider | undefined>(undefined);
  let delete_open = $state(false);
  let delete_target = $state<string | null>(null);
</script>
```

**Updated provider cards section** (replace lines 57-135):

```svelte
  {#if active === 'providers'}
    <header class="mb-6">
      <h1 class="font-display text-2xl font-semibold tracking-tight">AI Providers</h1>
      <p class="mt-1 text-sm text-(--color-muted)">
        Configure AI providers and API keys. Keys are stored in your OS keychain.
      </p>
    </header>

    {#if !app.keyring_available}
      <KeyringUnavailableBanner class="mb-4" />
    {/if}

    <div class="flex flex-col gap-3">
      {#each app.providers as provider (provider.id)}
        <div class="rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface)">
          <div class="flex items-center justify-between gap-3 border-b border-(--color-border) px-4 py-3">
            <div>
              <h3 class="font-display text-base font-semibold tracking-tight">
                {provider.name}
              </h3>
              <div class="mt-0.5 flex items-center gap-2 text-[11px] text-(--color-subtle)">
                <Tag>{provider.provider_type}</Tag>
                <span class="font-mono">{provider.model_name}</span>
                {#if provider.models.length > 1}
                  <span>+{provider.models.length - 1} more</span>
                {/if}
              </div>
            </div>
            <div class="flex items-center gap-2">
              {#if provider.has_api_key}
                <span class="inline-flex items-center gap-1 text-xs text-(--color-success)">
                  <Check class="size-3" /> Key set
                </span>
              {:else}
                <span class="inline-flex items-center gap-1 text-xs text-(--color-subtle)">
                  <X class="size-3" /> No key
                </span>
              {/if}
              <KebabMenu>
                <KebabMenuItem onclick={() => { editor_mode = 'edit'; editor_provider = provider; editor_open = true; }}>
                  Edit
                </KebabMenuItem>
                <KebabMenuSeparator />
                <KebabMenuItem onclick={() => { delete_target = provider.id; delete_open = true; }}>
                  Delete
                </KebabMenuItem>
              </KebabMenu>
            </div>
          </div>
          <!-- generation params display stays similar -->
        </div>
      {/each}

      <button type="button" onclick={() => { editor_mode = 'create'; editor_provider = undefined; editor_open = true; }}
        class="inline-flex items-center justify-center gap-2 rounded-(--radius-lg) border border-dashed border-(--color-border) px-4 py-4 text-sm text-(--color-muted) hover:border-(--color-border-strong) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)">
        <Plus class="size-4" />
        Add provider
      </button>
    </div>

    <ProviderEditorDialog bind:open={editor_open} mode={editor_mode} provider={editor_provider} />
    <DeleteConfirmationDialog bind:open={delete_open} title="Delete provider?" description="This will remove the provider and its API key from the keychain." onconfirm={() => { if (delete_target) app.deleteProvider(delete_target); delete_target = null; }} />
  {/if}
```

---

### `src/routes/+layout.svelte` (component, request-response)

**Analog:** `src/routes/+layout.svelte`

**Add keyring check to onMount** (extend lines 12-24):

```svelte
  onMount(() => {
    app.hydrate();
    app.checkKeyring();  // NEW: check keyring availability on startup

    // F9 toggles sidebar
    const onkey = (e: KeyboardEvent) => {
      if (e.key === 'F9') {
        e.preventDefault();
        app.toggleSidebar();
      }
    };
    window.addEventListener('keydown', onkey);
    return () => window.removeEventListener('keydown', onkey);
  });
```

---

### `src/lib/components/providers/ProviderEditorDialog.svelte` (component, request-response)

**Analog:** `src/lib/components/personas/PersonaEditorDialog.svelte`

**Key pattern elements to copy from PersonaEditorDialog:**

- Props: `open`, `onopenchange`, `mode: 'create' | 'edit'`, `provider?: Provider`
- Form state with `$state()` for each field
- `$effect()` to initialize form when dialog opens (was_open pattern, lines 68-104)
- Dirty tracking via snapshot comparison (lines 47-63)
- Discard confirmation on close with dirty data (lines 106-119)
- Backend error banner pattern (lines 199-205)
- Save handler: `is_saving` guard, validate, call `app.createProvider` / `app.updateProvider` (lines 137-179)
- Dialog component with title/description, footer snippet (lines 191-317)

**Key differences from PersonaEditorDialog:**

- No avatar upload
- ProviderType selector (grouped Select with optgroups)
- Base URL field (auto-filled from ProviderType default)
- API key input (password type, optional for Ollama)
- Model list section (ModelEntryRow components with add/remove)
- Generation params grid (temperature, top_p, max_tokens, frequency_penalty, presence_penalty)
- On provider_type change in create mode: auto-fill base_url and add default model

**Imports pattern:**

```svelte
<script lang="ts">
  import type { Provider, ProviderType, ProviderModelEntry } from '$lib/types';
  import { app } from '$lib/stores/app.svelte';
  import { Dialog, Button, Input, Textarea, Select } from '$lib/components/ui';
  import { ModelEntryRow } from '$lib/components/providers';
  import { DeleteConfirmationDialog } from '$lib/components/characters';
  import { AlertCircle, Plus, Eye, EyeOff } from '@lucide/svelte';
```

---

### `src/lib/components/providers/ModelEntryRow.svelte` (component, CRUD)

**Analog:** LorebookEditor.svelte entry list rows (lines 447-483)

**Pattern from LorebookEditor entry items:**

- Reusable row component with model name, context window size, enabled toggle, and delete button
- Trash2 icon button for delete (lines 474-481)
- Click handler for selection (line 453)

**Props pattern:**

```svelte
<script lang="ts">
  import type { ProviderModelEntry } from '$lib/types';
  import { Input } from '$lib/components/ui';
  import { Trash2, GripVertical } from '@lucide/svelte';

  interface Props {
    entry: ProviderModelEntry;
    disabled?: boolean;
    onupdate?: (entry: ProviderModelEntry) => void;
    ondelete?: () => void;
  }

  let { entry, disabled = false, onupdate, ondelete }: Props = $props();
```

**Layout:** Single row with: drag handle (future) | model name input | context window input | enabled checkbox | delete button

---

### `src/lib/components/providers/KeyringUnavailableBanner.svelte` (component, —)

**Analog:** PersonaEditorDialog error banner pattern (lines 199-205) + layout.svelte AlertTriangle

**Error banner pattern from PersonaEditorDialog:**

```svelte
<div class="flex items-center gap-2 rounded-(--radius-md) border border-(--color-destructive)/30 bg-(--color-destructive)/10 px-3 py-2 text-sm text-(--color-destructive)">
  <AlertCircle class="size-4 shrink-0" />
  {backend_error}
</div>
```

**KeyringUnavailableBanner implementation:**

```svelte
<script lang="ts">
  import { cn } from '$lib/utils';
  import { AlertTriangle } from '@lucide/svelte';

  interface Props {
    class?: string;
  }

  let { class: className }: Props = $props();
</script>

<div class={cn(
  'flex items-center gap-2 rounded-(--radius-md) border border-(--color-warning)/30 bg-(--color-warning)/10 px-4 py-3 text-sm text-(--color-warning)',
  className
)}>
  <AlertTriangle class="size-4 shrink-0" />
  <span>OS keychain is unavailable. API keys cannot be stored. You can still use local providers like Ollama.</span>
</div>
```

**Note:** Uses `--color-warning` semantic token (exists in `src/app.css` theme) instead of `--color-destructive`.

---

## Shared Patterns

### Tauri Command Pattern

**Source:** `src-tauri/src/commands/characters.rs`
**Apply to:** `src-tauri/src/commands/providers.rs` (all commands)

```rust
#[tauri::command]
pub async fn COMMAND_NAME(db: State<'_, DbWorker>, params) -> Result<ReturnType, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::VariantName { reply: reply_tx })
        .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}
```

### DB CRUD Pattern

**Source:** `src-tauri/src/db/characters.rs`
**Apply to:** `src-tauri/src/db/providers.rs` (all CRUD functions)

```rust
pub fn insert_ENTITY(conn: &Connection, entity: &Entity) -> Result<()> {
    let json_field_str = serde_json::to_string(&entity.json_field)?;
    conn.prepare_cached("INSERT INTO ... VALUES (?1, ?2, ...)")?
        .execute(params![entity.field1, json_field_str, ...])?;
    Ok(())
}

pub fn load_ENTITIES(conn: &Connection) -> Result<Vec<Entity>> {
    let mut stmt = conn.prepare_cached("SELECT ... FROM entities ORDER BY updated_at DESC")?;
    let entities = stmt.query_map([], |row| row_to_entity(row))?;
    let mut result = Vec::new();
    for entity in entities { result.push(entity?); }
    Ok(result)
}
```

### Row Mapper Pattern

**Source:** `src-tauri/src/db/models.rs`
**Apply to:** `row_to_provider` in `src-tauri/src/db/models.rs`

```rust
pub fn row_to_ENTITY(row: &Row) -> rusqlite::Result<Entity> {
    let created_at = parse_timestamp(&row.get::<_, String>("created_at")?)?;
    let updated_at = parse_timestamp(&row.get::<_, String>("updated_at")?)?;
    // Deserialize JSON columns
    let json_str: String = row.get("json_column")?;
    let json_field: Vec<T> = serde_json::from_str(&json_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;
    Ok(Entity { id: row.get("id")?, /* ... */ created_at, updated_at })
}
```

### DbCommand Variant Pattern

**Source:** `src-tauri/src/db/mod.rs`
**Apply to:** New LoadProviders/SaveProvider/DeleteProvider variants

```rust
// In DbCommand enum:
VariantName {
    data_field: DataType,
    reply: oneshot::Sender<anyhow::Result<ReturnType>>,
},

// In handle_command:
DbCommand::VariantName { data_field, reply } => {
    let result = module::operation(conn, &data_field);
    let _ = reply.send(result);
}
```

### Save (Upsert) Pattern

**Source:** `src-tauri/src/db/mod.rs` lines 215-224
**Apply to:** `DbCommand::SaveProvider` handler

```rust
DbCommand::SaveEntity { entity, reply } => {
    let result = if module::load_entity(conn, &entity.id)
        .unwrap_or(None)
        .is_some()
    {
        module::update_entity(conn, &entity).map(|_| entity.clone())
    } else {
        module::insert_entity(conn, &entity).map(|_| entity.clone())
    };
    let _ = reply.send(result);
}
```

### Optimistic Update Pattern (Frontend)

**Source:** `src/lib/stores/app.svelte.ts` lines 289-310
**Apply to:** `updateProvider` in app.svelte.ts

```typescript
async updateENTITY(entity: Entity): Promise<Entity> {
    const prev = this.entities.find((e) => e.id === entity.id);
    try {
        const saved = await apiSaveEntity({ ...entity, updated_at: new Date().toISOString() });
        this.entities = this.entities.map((e) => (e.id === saved.id ? saved : e));
        return saved;
    } catch (err) {
        console.error("Failed to update entity:", err);
        if (prev) {
            this.entities = this.entities.map((e) => (e.id === prev.id ? prev : e));
        }
        throw err;
    }
}
```

### Editor Dialog Pattern (Frontend)

**Source:** `src/lib/components/personas/PersonaEditorDialog.svelte`
**Apply to:** `ProviderEditorDialog.svelte`

```svelte
<script lang="ts">
    import type { Entity } from '$lib/types';
    import { app } from '$lib/stores/app.svelte';
    import { Dialog, Button, Input } from '$lib/components/ui';
    import { DeleteConfirmationDialog } from '$lib/components/characters';
    import { AlertCircle } from '@lucide/svelte';

    interface Props {
        open?: boolean;
        onopenchange?: (open: boolean) => void;
        mode?: 'create' | 'edit';
        entity?: Entity;
    }

    let { open = $bindable(false), onopenchange, mode = 'create', entity }: Props = $props();

    // Form state with $state()
    let form_name = $state('');
    let is_saving = $state(false);
    let backend_error = $state('');
    let discard_open = $state(false);

    // Dirty tracking via snapshot
    let initial_snapshot = $state('');
    function snapshot(): string { return JSON.stringify({ form_name /* ... */ }); }
    let is_dirty = $derived(snapshot() !== initial_snapshot);

    // Initialize form on open
    let was_open = $state(false);
    $effect(() => {
        const just_opened = open && !was_open;
        if (just_opened) {
            if (mode === 'edit' && entity) { /* populate from entity */ }
            else { /* reset form */ }
            initial_snapshot = snapshot();
        }
        was_open = open;
    });

    // Save handler
    async function handleSave() { /* validate → save → close */ }
</script>

<Dialog {open} onopenchange={handleOpenChange} title="..." description="...">
    <!-- Error banner -->
    <!-- Form fields -->
    {#snippet footer()}
        <Button variant="secondary" onclick={cancel}>Cancel</Button>
        <Button variant="primary" onclick={handleSave} disabled={is_saving}>Save</Button>
    {/snippet}
</Dialog>
<DeleteConfirmationDialog bind:open={discard_open} ... />
```

### API Wrapper Pattern (Frontend)

**Source:** `src/lib/data/api.ts`
**Apply to:** New provider and keyring invoke wrappers

```typescript
export async function ENTITY_COMMAND(params: Types): Promise<ReturnType> {
  return invoke<ReturnType>("entity_command", { params });
}
```

### Timestamp Convention

**Source:** `src-tauri/src/db/characters.rs` + `src-tauri/src/db/models.rs`
**Apply to:** All provider CRUD and row mapping

- **Rust serialization:** `entity.created_at.to_rfc3339()` (RFC 3339 TEXT in SQLite)
- **Rust deserialization:** `parse_timestamp(&str)` helper in models.rs → `DateTime<Utc>`
- **TypeScript:** `string` type (ISO 8601 / RFC 3339)

### JSON Column Convention

**Source:** `src-tauri/src/db/characters.rs` (alternate_greetings, tags, extensions, character_book)
**Apply to:** `models` column and `default_params` column on providers table

- **Rust serialization:** `serde_json::to_string(&field)?`
- **Rust deserialization:** `serde_json::from_str(&str).map_err(|e| FromSqlConversionFailure(...))`
- **SQLite:** `TEXT NOT NULL DEFAULT '[]'` (for arrays) or `TEXT` (for nullable objects)

## No Analog Found

| File                                             | Role      | Data Flow        | Reason                                                                                                                                                                |
| ------------------------------------------------ | --------- | ---------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src-tauri/src/secrets/mod.rs` + keyring v4 init | service   | request-response | No existing `secrets/` module. Keyring v4 `use_native_store()` / `release_store()` init pattern is new. Use RESEARCH.md Patterns 1-2 and gtk_ref analog.              |
| KeyringUnavailableBanner using `--color-warning` | component | —                | No existing warning-colored banner (only error banners with `--color-destructive`). Use `--color-warning` theme token + error banner layout from PersonaEditorDialog. |

## Metadata

**Analog search scope:** `src-tauri/src/`, `src/lib/`, `gtk_ref/src/`
**Files scanned:** 30+
**Pattern extraction date:** 2026-05-12
