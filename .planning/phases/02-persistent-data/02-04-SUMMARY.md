# Plan 02-04 Summary: Frontend Persistent Data Integration

**Date:** 2026-05-11
**Status:** ✅ COMPLETE

## Objective

Replace all mock data with real Tauri `invoke()` calls, align TypeScript types with Rust serde output, add loading/error states, and delete `mock.ts`.

## Tasks Completed

### Task 1: Aligned TypeScript types with Rust serde output

**File:** `src/lib/types/index.ts`

| Change                                                               | Before                              | After                                         |
| -------------------------------------------------------------------- | ----------------------------------- | --------------------------------------------- |
| MessageRole                                                          | `'user' \| 'character' \| 'system'` | `'user' \| 'assistant' \| 'system'`           |
| Message.parent_id / branch_index                                     | Present (tree model)                | Removed                                       |
| Message.sort_order / timestamp                                       | Absent                              | Added                                         |
| Message.created_at                                                   | Present                             | Removed (→ timestamp)                         |
| Conversation.user_persona_id                                         | Present                             | Removed (→ creator_persona_id)                |
| Conversation.active_branch_tip / message_count                       | Present                             | Removed                                       |
| Conversation.provider_id / generation_preset_id / creator_persona_id | Absent                              | Added                                         |
| Conversation.last_message_preview                                    | Optional                            | Required                                      |
| CharacterCard.avatar_url                                             | Present                             | avatar_path                                   |
| CharacterCard.extensions                                             | CharacterExtensions type            | `Record<string, unknown>`                     |
| CharacterCard fields                                                 | Mostly optional                     | Required strings with serde defaults          |
| UserPersona.avatar_url                                               | Present                             | avatar_path (optional, UI-only)               |
| UserPersona.is_default                                               | Present                             | Removed                                       |
| PromptPreset                                                         | Interface                           | GenerationPreset (with backward-compat alias) |
| GenerationPreset.is_builtin / stop_sequences / timestamps            | Absent                              | Added                                         |

### Task 2: Created typed API layer

**File:** `src/lib/data/api.ts` (new)

- 16 typed `invoke()` wrappers for all Tauri commands
- Snake_case command names matching Rust function names
- camelCase arguments (Tauri v2 auto-converts)
- Proper TypeScript generics for return types

### Task 3: Updated AppState and +layout.svelte

**File:** `src/lib/stores/app.svelte.ts`

- `prompt_presets` → `presets` (GenerationPreset[])
- Added `is_loading` and `init_error` state
- `hydrate()` now async, calls API functions with `Promise.allSettled`
- `active_persona` getter uses `creator_persona_id` instead of `user_persona_id`
- Removed `default_persona` getter (is_default no longer exists)
- `switchPersona` is now async, uses `updateConversationPersona` API
- Optimistic local update + rollback on API failure

**File:** `src/routes/+layout.svelte`

- Removed all mock data imports
- Calls `app.hydrate()` (no arguments)
- Skeleton loading UI while `app.is_loading` is true
- DB error overlay with retry button when `app.init_error` is set
- App only renders after loading completes successfully

### Task 4: Updated all components for type changes

| Component                  | Change                                                                                                      |
| -------------------------- | ----------------------------------------------------------------------------------------------------------- |
| MessageBubble.svelte       | `avatar_url` → `avatar_path`                                                                                |
| ChatHeader.svelte          | `avatar_url` → `avatar_path`                                                                                |
| PersonaSwitcher.svelte     | `avatar_url` → `avatar_path`, removed `is_default` check                                                    |
| ConversationSidebar.svelte | `avatar_url` → `avatar_path`, `last_message_preview` optional chaining removed                              |
| CharacterCard.svelte       | `avatar_url` → `avatar_path`                                                                                |
| PersonaCard.svelte         | `avatar_url` → `avatar_path`, removed `is_default` block and `Star` import                                  |
| +page.svelte (home)        | `avatar_url` → `avatar_path`                                                                                |
| chat/[id]/+page.svelte     | Mock import → API import, `'character'` → `'assistant'`, message construction uses `sort_order`/`timestamp` |
| settings/+page.svelte      | `prompt_presets` → `presets`, added `is_builtin` and `stop_sequences` display                               |

### Task 5: Deleted mock.ts and verified end-to-end

- Deleted `src/lib/data/mock.ts`
- Verified no remaining `avatar_url`, `PromptPreset` (except alias), `'character'` role, `user_persona_id`, `parent_id`, `branch_index`, or `is_default` references
- `npm run check` → 0 errors
- `npm run build` → success
- `cargo build` → success (pre-existing warnings only)

## Verification Results

| Check                    | Result                                         |
| ------------------------ | ---------------------------------------------- |
| `npm run check`          | ✅ 0 errors (1 pre-existing warning)           |
| `npm run build`          | ✅ Success                                     |
| `cargo build`            | ✅ Success (4 pre-existing dead_code warnings) |
| No mock data references  | ✅ Verified via grep                           |
| No avatar_url references | ✅ All avatar_path                             |
| No 'character' role      | ✅ All 'assistant'                             |
| No user_persona_id       | ✅ All creator_persona_id                      |

## Files Modified

- `src/lib/types/index.ts` — Type alignment with Rust serde
- `src/lib/data/api.ts` — **NEW** Typed Tauri invoke() API layer
- `src/lib/data/mock.ts` — **DELETED**
- `src/lib/stores/app.svelte.ts` — Async hydrate, loading/error state, persona API
- `src/routes/+layout.svelte` — Loading skeleton, error overlay, async hydrate
- `src/routes/+page.svelte` — avatar_url → avatar_path
- `src/routes/chat/[id]/+page.svelte` — API import, role fix, message construction
- `src/routes/settings/+page.svelte` — presets field, is_builtin display
- `src/lib/components/chat/MessageBubble.svelte` — avatar_path
- `src/lib/components/chat/ChatHeader.svelte` — avatar_path
- `src/lib/components/chat/PersonaSwitcher.svelte` — avatar_path, remove is_default
- `src/lib/components/layout/ConversationSidebar.svelte` — avatar_path, required last_message_preview
- `src/lib/components/characters/CharacterCard.svelte` — avatar_path
- `src/lib/components/personas/PersonaCard.svelte` — avatar_path, remove is_default/Star

## Design Decisions

1. **UserPersona.avatar_path** — Kept as optional UI-only field. The Rust `Persona` struct doesn't have `avatar_path`, but the frontend components expect it. It will be `undefined` for backend-loaded personas until avatar support is added.

2. **PromptPreset alias** — `export type { GenerationPreset as PromptPreset }` for backward compat. New code should use `GenerationPreset`.

3. **Promise.allSettled for hydration** — Allows partial success: if presets fail, characters/conversations still load. Critical failures (first 3) set `init_error`.

4. **Optimistic persona switch** — Local state updates immediately, persists to backend in background, reverts on failure.
