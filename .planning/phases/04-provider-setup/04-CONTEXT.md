# Phase 4: Provider Setup - Context

**Gathered:** 2026-05-12
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can configure AI providers (name, type, API key, models, generation params) through the settings page, with API keys stored exclusively in the OS-native keychain — never written to disk. Providers support multiple models under a single API key. The app gracefully handles keyring unavailability on Linux.

In scope:
- KeyringStore: OS-native keychain integration via `keyring` 4.0.0 crate
- Provider domain model port: ProviderConfig with 1:N provider-models (JSON column)
- Schema migration: add `models` JSON column and `model_name` to providers/conversations
- Tauri commands for provider CRUD + keyring operations
- Provider editor dialog (add/edit with conditional fields)
- Settings page wiring: replace stub UI with real backend data
- Keyring availability check on startup with graceful degradation on Linux
- Conversation-provider-model linking (provider_id + model_name)

NOT in scope: AI streaming/inference (Phase 5), prompt assembly (Phase 6), token counting (Phase 6), card import/export (Phase 7), theme toggle (Phase 10).

</domain>

<decisions>
## Implementation Decisions

### Keyring Approach

- **D-01:** Use the `keyring` crate — same pattern as gtk_ref KeyringStore (service name "revelry", provider_id as username). Not a Tauri plugin.
- **D-02:** Pin `keyring` to version 4.0.0 (user preference over 3.6.3 from gtk_ref). Researcher/planner must verify 4.0.0 API stability and feature flag changes before implementing.
- **D-03:** KeyringStore as a standalone `src-tauri/src/secrets/` module — not routed through DbWorker. Clean separation; keyring ops are independent of DB.
- **D-04:** Dispatch keyring operations via `tokio::task::spawn_blocking()` inside Tauri async commands. Keyring ops are synchronous and blocking; spawn_blocking keeps Tauri commands async without a dedicated thread/channel infrastructure.

### Provider Editor UX

- **D-05:** Provider add/edit uses a Bits.ui Dialog — consistent with Phase 3 character/persona editor pattern. Opens over the settings page.
- **D-06:** Single `ProviderEditorDialog` component with `mode: 'create' | 'edit'` — on create, provider_type selector is active and base_url/model auto-fill from ProviderType defaults; on edit, provider_type is locked (read-only), base_url and model list are editable.
- **D-07:** API key field is inline in the provider dialog — part of the same save action. Key is saved to OS keychain when the dialog is saved. For Ollama (no key required), the field is optional/hidden.
- **D-08:** ProviderType selector uses grouped select with optgroups: Cloud (OpenAI, Anthropic, Gemini, Cohere, Mistral, DeepSeek, Groq, Azure OpenAI, HuggingFace, Perplexity, xAI), Local (Ollama), Gateway (OpenRouter, Together), Custom (custom_openai_compatible). Reduces 15-item list to scannable categories.

### Linux Keyring Fallback

- **D-09:** When OS keyring is unavailable (e.g., headless Linux, no Secret Service daemon): show a persistent warning banner on the settings page, disable the API key field in the provider editor, but allow provider creation (Ollama works without a key). The app remains functional — keyring failure is not fatal.
- **D-10:** Check keyring availability once on app startup — attempt to create and delete a test entry. Cache the result (`keyring_available: bool`). Show a persistent banner in settings if unavailable. Avoids checking on every provider operation.

### Provider-Model Design

- **D-11:** 1:N provider-models — one provider entry can have multiple models under the same API key. A user creates one "OpenAI" provider and adds GPT-4o and GPT-4o-mini as models within it.
- **D-12:** Store models as a JSON TEXT column on the `providers` table — `models TEXT NOT NULL DEFAULT '[]'`. Each model entry: `{ name: string, context_window_size: number | null, enabled: boolean }`. No separate table. Models are configuration, not independent entities.
- **D-13:** Generation parameters (temperature, top_p, max_tokens, etc.) are shared across all models under a provider — one set of `default_params` per provider. Per-model params are not supported. Users can override per-conversation in Phase 5.
- **D-14:** Conversations reference a specific model via `provider_id` (existing FK) + a new `model_name TEXT` column on the `conversations` table. A model is identified by the (provider_id, model_name) pair. No surrogate model_id.

### Schema Changes Required

- **D-15:** Add `models TEXT NOT NULL DEFAULT '[]'` column to the `providers` table (Phase 4 migration #3). This stores the JSON array of model entries. The existing `model_name` column on providers becomes the "default" or "primary" model for backward compat, OR is deprecated in favor of the models array. Planner decides the cleanest approach.
- **D-16:** Add `model_name TEXT` column to the `conversations` table (Phase 4 migration #3). Alongside existing `provider_id` FK, this identifies the specific model used for the conversation.

### Claude's Discretion

- Exact layout of the provider editor dialog fields (grouping, order, spacing) — I will pick based on ergonomics and existing Phase 3 dialog patterns.
- Whether `model_name` column on `providers` table should be kept (for primary/default model) or removed in favor of the `models` JSON array — I will evaluate during planning based on query simplicity vs. redundancy.
- Exact wording of the keyring-unavailable banner message.
- Whether to add a `has_api_key` boolean field returned by the backend (so the frontend can show "key set" / "no key" without ever receiving the actual key) — likely yes for UX, I will implement this.
- Exact keyring test entry naming convention for the startup check.

</decisions>

<canonical_refs>

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Planning

- `.planning/ROADMAP.md` — Phase 4 definition, plans list (04-01 through 04-04), success criteria
- `.planning/REQUIREMENTS.md` — PROV-01, PROV-02, PROV-03, PROV-04, PROV-05, PROV-06 requirements
- `.planning/PROJECT.md` — Project context, constraints, tech stack decisions, key decisions table

### Phase 2 Decisions (carry-forward)

- `.planning/phases/02-persistent-data/02-CONTEXT.md` — D-08 through D-19 (Tauri command design, type alignment, DB threading model)

### Archival Reference (MUST read — patterns to port)

- `gtk_ref/src/secrets/mod.rs` — KeyringStore implementation (save/load/delete pattern, SERVICE_NAME constant, synchronous ops)
- `gtk_ref/src/domain/provider.rs` — ProviderConfig domain model, ProviderType enum with requires_api_key(), default_base_url(), default_model_name() helpers
- `gtk_ref/src/db/providers.rs` — Provider CRUD operations (insert, load, update, delete), row mapper pattern
- `gtk_ref/src/db/models.rs` — row_to_provider() mapper function

### Active Backend (will be extended)

- `src-tauri/src/lib.rs` — Tauri command registration (will add provider + keyring commands)
- `src-tauri/src/domain/mod.rs` — Domain module registry (will add provider module)
- `src-tauri/src/db/migrations.rs` — Current 2-migration schema (will add migration #3 for models column + model_name on conversations)
- `src-tauri/Cargo.toml` — Dependencies (will add `keyring` 4.0.0, update `tokio` features for spawn_blocking)

### Active Frontend (will be modified)

- `src/lib/types/index.ts` — Provider and ProviderType TypeScript types (will update for models array + model_name on Conversation)
- `src/lib/stores/app.svelte.ts` — AppState (will add provider CRUD actions, keyring_available state)
- `src/lib/data/api.ts` — API layer (will add provider/keyring invoke wrappers)
- `src/routes/settings/+page.svelte` — Settings page (will wire provider editor dialog, keyring banner)
- `src/lib/components/ui/` — Existing UI primitives: Dialog, Button, Input, Tag, Select (will be used by ProviderEditorDialog)

### Architecture Documentation

- `.planning/codebase/ARCHITECTURE.md` — System overview, layers, data flow
- `.planning/codebase/INTEGRATIONS.md` — External integrations, keyring/auth patterns
- `.planning/codebase/STACK.md` — Technology stack details

</canonical_refs>

<code_context>

## Existing Code Insights

### Reusable Assets

- **KeyringStore from gtk_ref** — Complete, tested keyring implementation with save/load/delete. Port directly to `src-tauri/src/secrets/mod.rs`. Update from `keyring` 3.6.3 to 4.0.0 API.
- **ProviderConfig from gtk_ref** — Domain model with ProviderType enum, default_base_url(), default_model_name(), requires_api_key(). Port to `src-tauri/src/domain/provider.rs`. Extend with `models` JSON field.
- **Provider CRUD from gtk_ref** — insert_provider, load_providers, update_provider, delete_provider. Port to `src-tauri/src/db/providers.rs`. Extend for models JSON column.
- **Settings page scaffold** — Already renders provider cards with name, type tag, model, params, API key field with reveal toggle. "Edit" and "Add provider" buttons are stubs ready to wire.
- **Dialog component from Phase 3** — Bits.ui Dialog wrapper used by character/persona/lorebook editors. Reuse pattern for ProviderEditorDialog.
- **KebabMenu / DeleteConfirmationDialog** — Phase 3 patterns for card-level actions. Provider cards need kebab menu with Edit/Delete.
- **AppState CRUD pattern** — Established in Phase 3: optimistic local update + backend persist + revert on failure. Apply to provider CRUD.

### Established Patterns

- **Tauri IPC commands**: snake_case function names, async, return `Result<T, String>`. Registered in `lib.rs` via `invoke_handler`.
- **DB worker pattern**: Background-thread `DbWorker` with `mpsc` channels and `oneshot` results. Provider CRUD goes through DbWorker (like characters/personas). Keyring ops do NOT (standalone module).
- **TypeScript type alignment**: Rust serde output must match TS interfaces. Phase 2 D-08.
- **ISO 8601 timestamps**: `DateTime<Utc>` in Rust, `string` in TypeScript. Phase 2 D-09.
- **UUID text PKs**: All entities use `TEXT PRIMARY KEY`. Phase 2 D-10.
- **Provider type serialization**: JSON string in SQLite TEXT column (e.g., `"OpenAI"`, `"Anthropic"`). gtk_ref uses `serde_json::to_string(&provider_type)`.
- **Settings page nav**: Sidebar with section buttons, content area on right. Active section tracked via `$state`.

### Integration Points

- **`src-tauri/src/lib.rs`**: Register new commands: `get_providers`, `save_provider`, `delete_provider`, `save_api_key`, `load_api_key`, `delete_api_key`, `check_keyring_available`
- **`src-tauri/Cargo.toml`**: Add `keyring = "4.0.0"` with platform-specific features (`apple-native`, `linux-native`, `windows-native`). Update `tokio` features to include `rt-multi-thread` for `spawn_blocking`.
- **`src-tauri/src/db/migrations.rs`**: Add migration #3: ALTER TABLE providers ADD COLUMN models TEXT DEFAULT '[]'; ALTER TABLE conversations ADD COLUMN model_name TEXT;
- **`src-tauri/src/domain/mod.rs`**: Add `pub mod provider;` and `pub mod secrets;`
- **`src/lib/stores/app.svelte.ts`**: Add `keyring_available = $state<boolean>(true)`, provider CRUD actions (createProvider, updateProvider, deleteProvider)
- **`src/lib/data/api.ts`**: Add invoke wrappers for provider + keyring commands
- **`src/lib/types/index.ts`**: Add `models` array to Provider interface, `model_name` to Conversation interface, ProviderModelEntry type
- **`src/routes/settings/+page.svelte`**: Wire ProviderEditorDialog, add keyring-unavailable banner, connect Add/Edit/Delete buttons
- **`src/routes/+layout.svelte`**: Call `check_keyring_available` during hydration, set `app.keyring_available`

### Constraints

- API keys MUST NEVER appear in SQLite, config files, logs, or IPC payloads. Security invariant from PROV-05.
- The `providers` table already exists in the schema (migration 1). Phase 4 adds columns, not a new table.
- `keyring` 4.0.0 API must be verified — the gtk_ref used 3.6.3. Planner/researcher should check for breaking changes.
- Ollama providers do not require an API key — the editor must handle this (optional key field).
- The `conversations.provider_id` FK already exists. Phase 4 adds `model_name` alongside it.

</code_context>

<specifics>
## Specific Ideas

- Keyring startup check: try to create/delete an entry with a well-known test key (e.g., service="revelry", username="__keyring_test__"). If it fails, keyring is unavailable.
- Provider editor: when user selects a ProviderType in create mode, auto-fill base_url from ProviderType::default_base_url() and add the default model from ProviderType::default_model_name() to the models array.
- The backend should return a `has_api_key: boolean` field alongside provider data (not the actual key) so the frontend can display "Key set ✓" / "No key" status without ever receiving the key.
- Provider type categories for the grouped select:
  - Cloud: OpenAI, Anthropic, Gemini, Cohere, Mistral, DeepSeek, Groq, Azure OpenAI, HuggingFace, Perplexity, xAI
  - Local: Ollama
  - Gateway: OpenRouter, Together
  - Custom: custom_openai_compatible

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

_Phase: 04-Provider Setup_
_Context gathered: 2026-05-12_
