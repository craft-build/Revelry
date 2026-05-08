---
phase: 06-generation-controls-regex-data
plan: 02
subsystem: ui, ai

# Dependency graph
requires:
  - phase: 06-generation-controls-regex-data
    provides: GenerationPreset domain model, DB CRUD, built-in preset seeding (Plan 01)
provides:
  - PresetEditor sub-dialog component for create/edit/duplicate
  - Settings preset page with default dropdown and preset list
  - Character editor preset dropdown on Identity page
  - Chat header preset selector popover
  - GenerationParams struct with from_preset() factory
  - AiService generation parameter injection via AgentBuilder
  - Preset resolution hierarchy (conversation→character→global→none)
  - Provider-specific parameter filtering (Anthropic/Gemini/Cohere skip freq/pres penalty)
affects: [06-03, any future plan using AiService::stream_chat]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "GenerationParams pattern: Option fields for each param, from_preset() factory, all-None default = provider defaults"
    - "provider_supports_param() allowlist for per-provider parameter filtering"
    - "pending_app_setting field pattern for routing AppSettingLoaded to correct model field"
    - "PresetEditor imperative sub-dialog (same as PersonaEditor/LorebookEditor)"

key-files:
  created:
    - src/ui/preset_editor.rs
  modified:
    - src/service/ai.rs
    - src/ui/settings/mod.rs
    - src/ui/chat/mod.rs
    - src/ui/character_editor.rs
    - src/ui/mod.rs
    - src/app.rs
    - src/db/mod.rs
    - src/db/conversations.rs
    - src/db/characters.rs
    - src/db/models.rs
    - src/domain/character.rs
    - src/domain/conversation.rs

key-decisions:
  - "GenerationParams uses Option fields — None means no override, not zero"
  - "temperature/max_tokens applied natively via AgentBuilder; top_p/freq_penalty/pres_penalty/stop via additional_params JSON"
  - "Provider-specific param filtering: Anthropic/Gemini/Cohere skip frequency_penalty and presence_penalty"
  - "pending_app_setting tracks which key was requested via LoadAppSetting to route value correctly"
  - "EntryRow set_subtitle not supported — removed from preset editor EntryRows (Adwaita limitation)"

patterns-established:
  - "GenerationParams: struct with all-Option fields + from_preset() factory + Default::default() = no overrides"
  - "provider_supports_param(): allowlist function for per-provider parameter support matrix"
  - "Preset resolution: conversation→character→global→none, implemented in ChatPanel.resolve_generation_params()"

requirements-completed: [GENR-01, GENR-02, GENR-03, GENR-04]

# Metrics
duration: 5min
completed: 2026-05-08
---

# Phase 06 Plan 02: Generation Controls UI & AI Integration Summary

**Preset editor dialog, settings/character/chat preset assignment UI, and AiService generation parameter injection via rig AgentBuilder**

## Performance

- **Duration:** 5 min
- **Started:** 2026-05-08T05:49:29Z
- **Completed:** 2026-05-08T05:55:22Z
- **Tasks:** 2
- **Files modified:** 13

## Accomplishments
- Full preset CRUD UI: create, edit, duplicate built-in, delete custom presets
- Three-level assignment hierarchy: global default (Settings), per-character (Character Editor), per-conversation (Chat Header popover)
- GenerationParams struct with from_preset() factory and all-Option fields for clean "no override = provider defaults" semantics
- AiService::stream_chat() injects generation parameters via rig AgentBuilder temperature/max_tokens natively, top_p/frequency_penalty/presence_penalty/stop_sequences via additional_params JSON
- Provider-specific parameter filtering: Anthropic, Gemini, Cohere silently skip frequency_penalty/presence_penalty per D-02
- 8 new unit tests covering GenerationParams, from_preset(), provider_supports_param(), and build_additional_params()

## Task Commits

Each task was committed atomically:

1. **Task 1: Preset editor, Settings preset page, and assignment hierarchy UI** - `bc6dfc8` (feat)
2. **Task 2: AiService generation parameter injection** - `958b684` (feat)

## Files Created/Modified
- `src/ui/preset_editor.rs` - PresetEditor AdwPreferencesWindow sub-dialog with all parameter rows
- `src/service/ai.rs` - GenerationParams struct, from_preset(), provider_supports_param(), build_additional_params(), modified stream_chat/run_stream
- `src/ui/settings/mod.rs` - Settings preset page with default dropdown, preset list with built-in/custom badges
- `src/ui/chat/mod.rs` - Chat header preset popover, SetGlobalDefaultPreset msg, resolve_generation_params()
- `src/ui/character_editor.rs` - Character editor preset dropdown on Identity page
- `src/ui/mod.rs` - PresetEditor module exports
- `src/app.rs` - AppModel preset routing (PresetEditorOutput, SetGlobalDefaultPreset, SetConversationPreset, AppSettingLoaded fix)
- `src/db/mod.rs` - UpdateConversationPreset DbInput variant, ConversationPresetUpdated DbOutput
- `src/db/conversations.rs` - update_conversation_preset(), updated queries with generation_preset_id
- `src/db/characters.rs` - Updated queries with default_preset_id
- `src/db/models.rs` - Updated row_to_conversation/row_to_character mappers
- `src/domain/conversation.rs` - generation_preset_id field
- `src/domain/character.rs` - default_preset_id field

## Decisions Made
- GenerationParams uses `Option` fields throughout — `None` means "no override" (not zero), so `Default::default()` correctly produces "use provider defaults" per D-13
- `temperature` and `max_tokens` applied natively via AgentBuilder (rig supports these directly); `top_p`, `frequency_penalty`, `presence_penalty`, `stop_sequences` go through `additional_params(serde_json::Value::Object(...))`
- `provider_supports_param()` is a simple allowlist — returns `false` for frequency_penalty/presence_penalty on Anthropic, Gemini, Cohere per the research matrix
- `pending_app_setting` field on AppModel tracks which key was requested via `LoadAppSetting` so `AppSettingLoaded` can route the value to the correct model field (global_default_persona_id vs global_default_preset_id)
- `adw::EntryRow` does not support `set_subtitle` (only `ActionRow` does) — removed subtitle from EntryRows in PresetEditor

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed AppSettingLoaded routing both persona and preset IDs to global_default_persona_id**
- **Found during:** Task 1 (AppModel routing)
- **Issue:** AppSettingLoaded handler had no way to distinguish whether the loaded value was for default_persona_id or default_preset_id — both went to global_default_persona_id
- **Fix:** Added `pending_app_setting: Option<String>` field to AppModel, set it before each `LoadAppSetting` dispatch, and match on it in `AppSettingLoaded` handler
- **Files modified:** src/app.rs
- **Verification:** Build compiles, setting loading routes correctly
- **Committed in:** bc6dfc8 (Task 1 commit)

**2. [Rule 1 - Bug] Removed unsupported set_subtitle from EntryRow in PresetEditor**
- **Found during:** Task 1 (build attempt)
- **Issue:** `adw::EntryRow` doesn't implement `IsA<ActionRow>` — `set_subtitle` is only available on ActionRow, causing E0599 build errors
- **Fix:** Removed `set_subtitle` from both EntryRows in PresetEditor (name_row and stop_sequences_row)
- **Files modified:** src/ui/preset_editor.rs
- **Verification:** Build compiles without errors
- **Committed in:** bc6dfc8 (Task 1 commit)

**3. [Rule 2 - Missing Critical] Added missing AppModel handlers for SettingsPanelOutput and ChatPanelOutput preset variants**
- **Found during:** Task 1 (build attempt)
- **Issue:** Non-exhaustive match errors — SettingsPanelOutput::SetGlobalDefaultPreset, SavePreset, DeletePreset and ChatPanelOutput::SetConversationPreset not handled in AppModel
- **Fix:** Added match arms for all three SettingsPanelOutput variants and the SetConversationPreset ChatPanelOutput variant
- **Files modified:** src/app.rs
- **Verification:** Build compiles without errors
- **Committed in:** bc6dfc8 (Task 1 commit)

**4. [Rule 2 - Missing Critical] Added ConversationPresetUpdated DbOutput handler**
- **Found during:** Task 1 (build attempt)
- **Issue:** Non-exhaustive match error on DbOutput — ConversationPresetUpdated variant not covered
- **Fix:** Added no-op handler for DbOutput::ConversationPresetUpdated (same pattern as ConversationPersonaUpdated)
- **Files modified:** src/app.rs
- **Verification:** Build compiles
- **Committed in:** bc6dfc8 (Task 1 commit)

---

**Total deviations:** 4 auto-fixed (2 bugs, 2 missing critical handlers)
**Impact on plan:** All auto-fixes necessary for correctness and compilation. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Preset UI and AI integration complete, ready for Plan 03 (regex scripting UI)
- AiService::stream_chat() now requires GenerationParams — all callers updated
- Preset resolution hierarchy fully wired through Settings, Character Editor, Chat Header, and AppModel

---
*Phase: 06-generation-controls-regex-data*
*Completed: 2026-05-08*

## Self-Check: PASSED
