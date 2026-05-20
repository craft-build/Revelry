# Plan 09-05 Summary: Wire Lorebook and Regex to Editor UIs and Conversation Flow

**Phase:** 09-power-features
**Plan:** 09-05
**Status:** Complete
**Commits:** 8

## Tasks Executed

### Task 1: Add API wrappers for regex and conversation lorebook commands
- **File:** `src/lib/data/api.ts`
- Added 9 new invoke wrappers: `getRegexScripts`, `saveRegexScript`, `deleteRegexScript`, `applySingleRegex`, `validateRegexPattern`, `getConversationLorebooks`, `assignLorebookToConversation`, `removeLorebookFromConversation`, `getCharacterLorebooks`
- Added `RegexScript` and `RegexTestResult` type imports

### Task 2: Extend AppState with regex scripts and conversation lorebook state/actions
- **File:** `src/lib/stores/app.svelte.ts`
- New state fields: `regex_scripts` (Map), `conversation_lorebooks`, `character_lorebooks`, `lorebook_detail_open`, `lorebook_details`
- New actions: `loadRegexScripts`, `saveRegexScript`, `deleteRegexScript`, `testRegexPattern`, `loadConversationLorebooks`, `loadCharacterLorebooks`, `toggleConversationLorebook`
- Updated `openConversation` to load conversation lorebooks, character lorebooks, and regex scripts
- Updated `closeConversation` to clear all lorebook/regex state
- Updated all 3 streaming complete handlers (startChat, regenerate, editResend) to extract `lorebookDetails` from StreamChunk
- **File:** `src/lib/types/index.ts` — Added `lorebookDetails?: LorebookActivationDetail[]` to StreamChunk complete data

### Task 3: Create ConversationLorebookPanel component
- **File:** `src/lib/components/chat/ConversationLorebookPanel.svelte`
- Popover with BookOpen icon trigger + lorebook count badge
- Character-level lorebooks shown as read-only with "Character" badge
- Conversation-level lorebooks toggleable via checkboxes
- Empty state message for no lorebooks

### Task 4: Add ConversationLorebookPanel to ChatHeader
- **File:** `src/lib/components/chat/ChatHeader.svelte`
- Added ConversationLorebookPanel after ProviderModelSelector with vertical divider

### Task 5: Create LorebookDetailPanel component for context bar
- **File:** `src/lib/components/chat/LorebookDetailPanel.svelte`
- Expandable section showing "Lorebook: N active" collapsed state
- Expanded state shows each `LorebookActivationDetail` with name, content snippet, token cost, and match/constant indicator
- Accent color when entries active, subtle when none

### Task 6: Add LorebookDetailPanel to MessageInput
- **File:** `src/lib/components/chat/MessageInput.svelte`
- Added LorebookDetailPanel between context bar and input frame

### Task 6a: Update LorebookEditor with enabled, sort_order, and token_budget controls
- **File:** `src/lib/components/lorebooks/LorebookEditor.svelte`
- Added enabled toggle switch (dims form when disabled)
- Added sort_order number input
- Added token_budget number input with "Leave empty for shared budget" hint
- Entry list shows "Disabled" label and opacity-50 for disabled entries
- Entry save payload includes `enabled`, `sort_order`, `token_budget`
- Added `cn` import from `$lib/utils`

### Task 7: Create RegexScriptEditor component with test panel
- **File:** `src/lib/components/characters/RegexScriptEditor.svelte`
- Two-column layout: script list (left) + script form (right)
- Fields: Name, Find pattern, Replace pattern, Flags, Enabled
- Collapsible test panel with sample text input and live result preview
- Delete with undo toast pattern
- Create mode shows lockout message
- Error display for invalid regex patterns

### Task 8: Add "Regex" step to CharacterEditorDialog
- **File:** `src/lib/components/characters/CharacterEditorDialog.svelte`
- Steps array extended to 4: Profile, Prompting, Lorebooks, Regex
- Step 4 renders `<RegexScriptEditor>` with characterId and createMode props

## Verification

- `bun run check` exits with **0 errors** (48 pre-existing a11y warnings only)
- All TypeScript types align with Rust serde output
- All components use `cn()` for class merging, Svelte 5 runes, and design tokens

## Requirements Coverage

| Requirement | Status |
|---|---|
| LORB-01 | Conversation lorebook assignments toggleable from chat header |
| LORB-02 | Character-level lorebooks shown as read-only in lorebook panel |
| LORB-03 | Context bar shows expandable lorebook activation detail |
| LORB-04 | LorebookEditor exposes enabled, sort_order, token_budget per entry |
| LORB-05 | Budget display in conversation lorebook panel |
| REGX-01 | Regex scripts CRUD in character editor |
| REGX-02 | Invalid regex patterns rejected on save with error display |
| REGX-03 | Regex test panel shows live preview of transformations |
| REGX-04 | Regex step added to character editor wizard |
| REGX-05 | Invalid regex patterns rejected with clear error messages |
