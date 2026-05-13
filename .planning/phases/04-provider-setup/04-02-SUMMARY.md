---
phase: 04-provider-setup
plan: 02
subsystem: ui
tags:
  [
    provider-editor,
    api-key,
    keyring-banner,
    model-entry,
    kebab-menu,
    settings,
    svelte-5,
  ]

# Dependency graph
requires:
  - phase: 02-persistent-data
    provides: Tauri IPC infrastructure, domain models, SQLite schema
  - phase: 03-editor-features
    provides: Editor dialog pattern, KebabMenu, DeleteConfirmationDialog
  - phase: 04-provider-setup
    provides: Plan 01 backend — KeyringStore, 8 Tauri commands, provider CRUD
provides:
  - ProviderEditorDialog with create/edit modes, grouped type selector, API key section, model list, generation params
  - ModelEntryRow for per-model name/context-window/enabled management
  - KeyringUnavailableBanner for OS keychain unavailability warning
  - Settings page wired with provider cards, kebab menus, key status, delete confirmation
  - AppState provider CRUD actions (createProvider, updateProvider, deleteProvider, checkKeyring)
  - TypeScript types updated (ProviderModelEntry, Provider with models/has_api_key, Conversation with model_name)
  - API wrappers for all 7 provider/keyring invoke calls
affects: [05-inference-streaming]

# Tech tracking
tech-stack:
  added: []
  patterns:
    [
      grouped-select-optgroups,
      api-key-never-in-frontend-state,
      provider-type-defaults-map,
      optimistic-provider-update,
    ]

key-files:
  created:
    [
      src/lib/components/providers/ProviderEditorDialog.svelte,
      src/lib/components/providers/ModelEntryRow.svelte,
      src/lib/components/providers/KeyringUnavailableBanner.svelte,
      src/lib/components/providers/index.ts,
    ]
  modified:
    [
      src/lib/types/index.ts,
      src/lib/data/api.ts,
      src/lib/stores/app.svelte.ts,
      src/routes/+layout.svelte,
      src/routes/settings/+page.svelte,
    ]

key-decisions:
  - "API key never stored in frontend state — only has_api_key boolean from backend"
  - "Provider type selector uses native <select> with <optgroup> elements for Cloud/Local/Gateway/Custom grouping"
  - "ModelEntryRow uses callback props instead of bindable for simpler parent control"
  - "Provider type locked (disabled) in edit mode — immutable after creation"
  - "Generation params (Temperature, Top P, Max tokens) are optional — blank means use default"

patterns-established:
  - "Grouped select with optgroups: Cloud, Local, Gateway, Custom provider type categories"
  - "API key reveal toggle: password input with Eye/EyeOff icon toggle"
  - "Keyring unavailability graceful degradation: disabled API key field + persistent warning banner"
  - "Provider type defaults map: auto-fill base_url and default model on type selection"

requirements-completed: [PROV-01, PROV-02, PROV-03, PROV-04, PROV-05, PROV-06]

# Metrics
duration: 6min
completed: 2026-05-13
---

# Phase 4 Plan 02: Provider UI Summary

**Provider management UI with ProviderEditorDialog (create/edit), model list, API key status, kebab menus, and keyring unavailability banner**

## Performance

- **Duration:** 6 min
- **Started:** 2026-05-13T02:57:24Z
- **Completed:** 2026-05-13T03:04:21Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- Full provider management UI: create, edit, delete providers with API keys stored in OS keychain
- ProviderEditorDialog with grouped type selector, auto-fill on type selection, API key reveal toggle, model list management, generation params grid
- Settings page rewired with provider cards showing model list, key status indicator, kebab menu actions
- KeyringUnavailableBanner shows persistent warning when OS keychain unavailable
- API key never appears in frontend state — only has_api_key boolean
- Empty state with CTA when no providers exist
- TypeScript types updated with ProviderModelEntry, extended Provider, Conversation.model_name
- 7 API invoke wrappers for provider/keyring backend commands
- AppState CRUD actions with optimistic updates and revert on failure

## Task Commits

Each task was committed atomically:

1. **Task 1: Update TypeScript types, API wrappers, AppState provider CRUD, and keyring check** - `8f62631` (feat)
2. **Task 2: Build ProviderEditorDialog, ModelEntryRow, KeyringUnavailableBanner, and wire settings page** - `7d31c00` (feat)

## Files Created/Modified

- `src/lib/types/index.ts` - Added ProviderModelEntry, updated Provider (model_name, models, has_api_key, enabled, timestamps), added Conversation.model_name
- `src/lib/data/api.ts` - Added 7 provider/keyring invoke wrappers (getProviders, saveProvider, deleteProvider, saveApiKey, hasApiKey, deleteApiKey, checkKeyringAvailable)
- `src/lib/stores/app.svelte.ts` - Added keyring_available state, provider CRUD actions, checkKeyring, apiGetProviders in hydrate
- `src/routes/+layout.svelte` - Added app.checkKeyring() call on startup
- `src/lib/components/providers/ProviderEditorDialog.svelte` - Full dialog with create/edit modes, type selector, API key, models, generation params
- `src/lib/components/providers/ModelEntryRow.svelte` - Model name + context window + enabled toggle + remove button
- `src/lib/components/providers/KeyringUnavailableBanner.svelte` - Warning banner for OS keychain unavailability
- `src/lib/components/providers/index.ts` - Barrel exports
- `src/routes/settings/+page.svelte` - Provider cards with model list, key status, kebab menus, add/edit/delete flows, empty state

## Decisions Made

- API key never stored in frontend state — only has_api_key boolean flows from backend, key only exists in form during save
- Provider type selector uses native `<select>` with `<optgroup>` for grouped categories (Cloud, Local, Gateway, Custom)
- ModelEntryRow uses callback props (onupdatename, etc.) instead of bindable for simpler parent state management
- Provider type is immutable after creation (disabled select in edit mode)
- Generation params are optional — leaving blank means "use provider/model default"
- context_window_size converted to string for Input component compatibility

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed settings page provider.model → provider.model_name reference**

- **Found during:** Task 1 (type check)
- **Issue:** The Provider interface was renamed from `model` to `model_name`, but settings/+page.svelte still referenced `provider.model`
- **Fix:** Changed to `provider.model_name` in settings page
- **Files modified:** src/routes/settings/+page.svelte
- **Verification:** bun run check passes with 0 errors
- **Committed in:** 8f62631 (Task 1 commit)

**2. [Rule 1 - Bug] Fixed context_window_size type errors**

- **Found during:** Task 2 (svelte-check)
- **Issue:** `m.context_window_size` is possibly undefined (TypeScript strict mode), and `contextWindowSize ?? ''` produces `string | number` not assignable to `string | undefined`
- **Fix:** Used `!= null` check instead of `!== null`, and `String(contextWindowSize)` for Input value
- **Files modified:** ProviderEditorDialog.svelte, ModelEntryRow.svelte
- **Verification:** bun run check passes with 0 errors
- **Committed in:** 7d31c00 (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2 bugs)
**Impact on plan:** Both auto-fixes necessary for type safety. No scope creep.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Provider management fully functional end-to-end: UI → AppState → Tauri IPC → SQLite + Keyring
- Ready for Phase 5 (inference-streaming) — provider selection in conversations, model dropdown
- All 6 provider requirements (PROV-01 through PROV-06) satisfied

## Self-Check: PASSED

---

_Phase: 04-provider-setup_
_Completed: 2026-05-13_
