---
phase: 04-personas-template-resolution
plan: 03
subsystem: ui
tags: [gtk4, relm4, persona, prompt-engine, template-variables, chat-header, popover, menu-button]

# Dependency graph
requires:
  - phase: 04-personas-template-resolution/01
    provides: Persona domain model, DB CRUD, resolve_persona_for_conversation, PromptEngine::assemble/assemble_initial, build_persona_block, resolve_variables
  - phase: 04-personas-template-resolution/02
    provides: PersonaEditor UI, CharactersPanel personas tab, SettingsPanel default persona dropdown, character_editor default_persona_id
provides:
  - ChatPanel header persona indicator with per-conversation selector
  - AppModel persona routing and assignment hierarchy resolution
  - Persona block injection into AI streaming prompt pipeline
  - Per-conversation persona update via DB
  - Persona name display on user message rows
affects: [05-context-window, 06-group-chats]

# Tech tracking
tech-stack:
  added: []
  patterns: [MenuButton+Popover for persona selection, assignment hierarchy (char->global->none), persona-aware prompt assembly]

key-files:
  created: []
  modified:
    - src/ui/chat/mod.rs
    - src/ui/chat/message_row.rs
    - src/app.rs
    - src/db/mod.rs
    - src/db/conversations.rs

key-decisions:
  - "Used MenuButton+Popover for persona selection (not DropDown) for flexible row layout"
  - "Personas forwarded to ChatPanel on PersonasLoaded for popover population"
  - "Assignment hierarchy: character default -> global default -> none"
  - "Global default persona ID queried from app_settings on startup"
  - "Persona block injected via existing PromptEngine::assemble, no AiService changes needed"
  - "Template variables resolved in combined preamble (character fields + persona block)"

requirements-completed: [PERS-02, PERS-03, PERS-06, CHAR-08]

# Metrics
duration: 6min
completed: 2026-05-07
---

# Phase 4 Plan 3: Persona Chat Integration Summary

**Persona display in chat header with per-conversation selector, assignment hierarchy resolution, and persona-injected AI prompt assembly**

## Performance

- **Duration:** 6 min
- **Started:** 2026-05-07T04:45:47Z
- **Completed:** 2026-05-07T04:52:15Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments
- Chat header shows active persona name ("as Alice" or "as User") with MenuButton popover for changing persona per-conversation
- AppModel routes all persona DbOutput variants and forwards personas to ChatPanel for selector
- Assignment hierarchy resolves persona at conversation start (character default -> global default -> none)
- Per-conversation persona update persisted to DB via UpdateConversationPersona
- PromptEngine::assemble receives active persona, injecting persona block and resolving {{user}}, {{char}}, {{persona.*}} template variables
- User message rows display persona name instead of "You" when a persona is active

## Task Commits

Each task was committed atomically:

1. **Task 1: ChatPanel header persona indicator + per-conversation selection popover** - `fcb911e` (feat)
2. **Task 2: AppModel persona routing, assignment hierarchy, and conversation start** - `feaa84b` (feat)
3. **Task 3: Persona block injection into AI prompt stream** - `7548c2b` (feat)

## Files Created/Modified
- `src/ui/chat/mod.rs` - ChatPanel with persona state, header MenuButton+Popover, persona message handling, PromptEngine integration
- `src/ui/chat/message_row.rs` - MessageRowInit with user_persona_name, persona name display on user messages
- `src/app.rs` - AppModel with global_default_persona_id, persona routing, hierarchy resolution, SetActivePersona forwarding
- `src/db/mod.rs` - DbInput::StartConversation with creator_persona_id, UpdateConversationPersona, ConversationPersonaUpdated
- `src/db/conversations.rs` - start_conversation with creator_persona_id, update_conversation_persona function

## Decisions Made
- Used MenuButton+Popover instead of DropDown for persona selection — more flexible for future enhancements (persona details, icons)
- Kept AiService interface unchanged (takes system_prompt: &str) — PromptEngine::assemble produces the persona-injected preamble, which AiService consumes
- Global default persona ID queried from app_settings on startup via LoadAppSetting — accepted potential race on first conversation (unlikely in practice)
- Stored persona_button and persona_listbox as model fields for dynamic popover management in update()
- Replaced fragile widget tree traversal in LoadConversation with #[watch] for character name label

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- Initial implementation used `ListBoxRow::connect_activated()` which is only available on `ActionRow` — fixed by using `ListBox::connect_row_activated()` with index-based persona ID lookup instead.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- All persona chat integration complete — personas flow from DB through AppModel to ChatPanel to AI prompt
- Ready for Phase 05 (context window management) which will add token budgeting and persona compression
- Ready for persona-aware lorebook activation in a future phase

---
*Phase: 04-personas-template-resolution*
*Completed: 2026-05-07*

## Self-Check: PASSED

All 5 modified files exist on disk. All 3 task commits present in git log (fcb911e, feaa84b, 7548c2b).
