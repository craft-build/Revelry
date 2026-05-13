# Phase 4: Provider Setup - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-12
**Phase:** 04-Provider Setup
**Areas discussed:** Keyring approach, Provider editor UX, Linux keyring fallback, Provider-model design

---

## Keyring Approach

| Option | Description | Selected |
|--------|-------------|----------|
| keyring crate | Port from gtk_ref pattern, service='revelry', provider_id as username | ✓ |
| tauri-plugin-keyring | Community/official Tauri plugin | |
| tauri-plugin-store (encrypted) | Simpler but stores on disk (encrypted) | |

**User's choice:** keyring crate
**Notes:** Proven pattern from gtk_ref. Direct Tauri command integration.

| Option | Description | Selected |
|--------|-------------|----------|
| keyring 3.6.3 | Same as gtk_ref, well-tested | |
| keyring 4.0.0 | Latest release, need to verify stability | ✓ |

**User's choice:** keyring 4.0.0
**Notes:** User prefers latest version despite gtk_ref pinning to 3.6.3. Researcher/planner must verify 4.0.0 API.

| Option | Description | Selected |
|--------|-------------|----------|
| Standalone module | New src-tauri/src/secrets/ module, clean separation | ✓ |
| Inside DbWorker | Route through existing DbWorker channel | |

**User's choice:** Standalone module
**Notes:** Keyring ops are independent of DB. No need to add them to DbInput/DbOutput.

| Option | Description | Selected |
|--------|-------------|----------|
| spawn_blocking | tokio::task::spawn_blocking() in Tauri async commands | ✓ |
| Dedicated thread + channel | Like DbWorker pattern | |

**User's choice:** spawn_blocking
**Notes:** Keyring ops are infrequent (save/load/delete). spawn_blocking is sufficient without extra infrastructure.

---

## Provider Editor UX

| Option | Description | Selected |
|--------|-------------|----------|
| Dialog | Bits.ui Dialog like Phase 3 editors | ✓ |
| Inline expandable card | In-place editing on settings page | |
| Sub-route page | Navigate to /settings/providers/:id | |

**User's choice:** Dialog
**Notes:** Consistent with Phase 3 character/persona editor pattern.

| Option | Description | Selected |
|--------|-------------|----------|
| Same dialog, conditional fields | provider_type locked on edit, defaults on create | ✓ |
| Split: quick-add + full-edit | Two different dialog components | |

**User's choice:** Same dialog, conditional fields
**Notes:** Single component with mode prop. Auto-fill on create, lock type on edit.

| Option | Description | Selected |
|--------|-------------|----------|
| Inline in form | Part of dialog, saved to keychain on save | ✓ |
| Post-create key entry | Create provider first, enter key separately | |

**User's choice:** Inline in form
**Notes:** Single-step flow. Key saved when dialog is saved. Ollama key field is optional.

| Option | Description | Selected |
|--------|-------------|----------|
| Grouped select | Categories: Cloud, Local, Gateway, Custom | ✓ |
| Flat select | All 15 types in a flat dropdown | |
| Icon grid | Visual grid of provider logos | |

**User's choice:** Grouped select
**Notes:** Reduces 15-item list to scannable categories. Requires optgroup support in Select component.

---

## Linux Keyring Fallback

| Option | Description | Selected |
|--------|-------------|----------|
| Warn + continue | Banner in settings, disable key field, allow Ollama | ✓ |
| Block provider creation | No providers needing keys allowed | |
| Encrypted file fallback | Encrypt keys with user password | |

**User's choice:** Warn + continue
**Notes:** App remains functional. Ollama (no key) works. Key field disabled when keyring unavailable.

| Option | Description | Selected |
|--------|-------------|----------|
| On app startup | Test entry read/write, cache result, persistent banner | ✓ |
| On first provider creation | Deferred check | |
| On every save/load | Per-call error handling | |

**User's choice:** On app startup
**Notes:** One-time check, cached result. Persistent banner on settings page.

---

## Provider-Model Design

| Option | Description | Selected |
|--------|-------------|----------|
| 1:1 provider-model | One provider = one model (existing schema) | |
| 1:N provider-models | One provider, multiple models (new) | ✓ |

**User's choice:** 1:N provider-models
**Notes:** Users want multiple models under one API key (e.g., GPT-4o + GPT-4o-mini under OpenAI).

| Option | Description | Selected |
|--------|-------------|----------|
| JSON column on providers | models TEXT DEFAULT '[]' | ✓ |
| Separate provider_models table | Normalized, new table with FKs | |

**User's choice:** JSON column on providers
**Notes:** Simpler migration. Models are configuration, not independent entities.

| Option | Description | Selected |
|--------|-------------|----------|
| Share provider defaults | One set of default_params per provider | ✓ |
| Per-model params | Each model has its own generation params | |

**User's choice:** Share provider defaults
**Notes:** Simpler. Per-conversation overrides in Phase 5.

| Option | Description | Selected |
|--------|-------------|----------|
| provider_id + model_name | FK + text column on conversations | ✓ |
| model_id (surrogate key) | UUID per model entry | |

**User's choice:** provider_id + model_name
**Notes:** Simple, no new FK, model_name is human-readable.

---

## Claude's Discretion

- Exact layout of provider editor dialog fields
- Whether to keep or remove `model_name` column on providers (redundant with models array)
- Keyring-unavailable banner wording
- Whether to add `has_api_key` boolean field in provider response (likely yes)
- Keyring test entry naming convention

## Deferred Ideas

None — discussion stayed within phase scope.
