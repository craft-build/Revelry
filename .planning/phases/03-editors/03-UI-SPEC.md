---
phase: 3
slug: editors
status: approved
reviewed_at: 2026-05-11
shadcn_initialized: false
preset: none
created: 2026-05-11
---

# Phase 3 — UI Design Contract

> Visual and interaction contract for the Editors phase. This phase introduces editor UIs for creating and modifying characters (4-step wizard), personas (single-form dialog), and lorebooks (standalone editor with keyword testing). It also adds kebab menus to library cards, avatar upload via native file picker, and delete confirmation dialogs. All design tokens, typography, color, spacing, and radii are inherited from Phase 1's locked design system.

---

## Design System

**Inherited from Phase 1.** No changes to tooling, tokens, or component library.

| Property | Value |
|----------|-------|
| Tool | none (no shadcn — project uses Bits.ui + Tailwind v4 directly) |
| Component library | Bits.ui ^2.18 (headless Svelte primitives) |
| Icon library | @lucide/svelte ^1.14 |
| Font | Geist (sans), Fraunces (serif/display), Geist Mono (mono) |
| Styling | Tailwind CSS v4 with custom OKLCH theme tokens in `src/app.css` |
| Variant system | tailwind-variants ^3.2 (`tv()`) |

**Reference:** `.planning/phases/01-native-desktop-shell/01-UI-SPEC.md` § Design System — all unchanged.

---

## Spacing Scale

**Inherited from Phase 1.** No changes.

See Phase 1 UI-SPEC § Spacing Scale for the full 4px-multiple scale (4, 8, 12, 16, 20, 24, 32, 40, 48, 64).

**Phase 3 additions:**

| Token | Value | Usage |
|-------|-------|-------|
| `dialog-gutter` | 24px (px-6) | Horizontal padding inside modal dialogs |
| `dialog-section` | 24px (gap-6) | Vertical spacing between wizard step content blocks |
| `form-field-gap` | 12px (gap-3) | Spacing between label, input, and helper text within a form field |
| `stepper-gap` | 32px | Horizontal space between stepper nodes in the wizard header |

All new spacing uses existing Tailwind tokens. No custom spacing values introduced.

---

## Typography

**Inherited from Phase 1.** Consolidated to 4 sizes and 2 weights per design system rules.

**Type scale (4 sizes, 2 weights):**

| Role | Size | Weight | Line Height | Font Family |
|------|------|--------|-------------|-------------|
| Body (chat bubbles, input) | 15px | 400 (regular) | 1.625 (relaxed) | Geist (sans) |
| Heading (speaker names, card titles, dialog title) | 14px–18px* | 600 (semibold) | 1.2 | Fraunces (serif/display) / Geist (sans) |
| Label / small (sidebar items, nav, metadata, stepper, field labels, errors, micro text) | 12px (xs) | 400 (regular) | 1.25 | Geist (sans) |
| Textarea body | 14px (sm) | 400 (regular) | 1.625 | Geist (sans) |

*

- Dialog title: 18px Geist (sans), weight 600 — exception to the Fraunces rule, as dialog titles are UI chrome, not character names.
- Card titles / speaker names: 14px Fraunces (serif/display), weight 600.

**Font families (from `src/app.css` @theme):**
- `--font-sans`: Geist, ui-sans-serif, system-ui, sans-serif — body text, UI chrome
- `--font-serif`: Fraunces, ui-serif, Georgia, serif — character names, app identity, display headings (via `.font-display` utility)
- `--font-mono`: Geist Mono, ui-monospace, SF Mono, Menlo, monospace — code blocks, token counts

**Font feature settings:**
- Sans: `'cv11', 'ss01', 'ss03'` (Geist alternates)
- Serif (`.font-display`): `'ss01', 'ss02', 'liga'` with `font-optical-sizing: auto`

**Phase 3 role breakdown by component:**

| Element | Size | Weight | Notes |
|---------|------|--------|-------|
| Dialog title | 18px | 600 | Geist sans, dialog header |
| Stepper label | 12px | 400 | All step labels |
| Stepper number | 12px | 600 | Circle number inside step node |
| Field label | 12px | 400 | Form field labels above inputs |
| Field error | 12px | 400 | `--color-destructive`, below invalid inputs |
| Textarea body | 14px | 400 | Form textareas (description, personality, etc.) |

**No 500 (medium) weight anywhere.** All labels, nav items, and metadata use 400. All headings, dialog titles, and emphasis use 600.

---

## Color

All colors defined as OKLCH values in `src/app.css` @theme block. Dark theme only — light theme deferred to Phase 10.

| Role | OKLCH Value | Tailwind Token | Usage |
|------|-------------|----------------|-------|
| **Dominant (60%)** | `oklch(0.16 0.005 270)` | `--color-background` | App background, page-level surface |
| **Secondary (30%)** | `oklch(0.19 0.006 270)` | `--color-surface` | Sidebar, header, input area backgrounds |
| | `oklch(0.22 0.008 270)` | `--color-surface-elevated` | Cards, elevated panels, active nav tab |
| | `oklch(0.24 0.009 270)` | `--color-surface-hover` | Hover state backgrounds |
| | `oklch(0.21 0.007 270)` | `--color-surface-input` | Text input backgrounds |
| **Accent (10%)** | `oklch(0.78 0.13 195)` | `--color-accent` | See reserved-for list below |
| | `oklch(0.18 0.02 200)` | `--color-accent-fg` | Text on accent backgrounds |
| | `oklch(0.32 0.06 195)` | `--color-accent-muted` | Selection highlight, progress bars, subtle accent fills |
| **Foreground** | `oklch(0.96 0.005 270)` | `--color-foreground` | Primary text color |
| **Muted** | `oklch(0.65 0.012 270)` | `--color-muted` | Secondary text, timestamps, previews |
| **Subtle** | `oklch(0.45 0.01 270)` | `--color-subtle` | Placeholder text, disabled icons, metadata |
| **Border** | `oklch(0.28 0.008 270)` | `--color-border` | Default borders (set on `*` via base layer) |
| **Border strong** | `oklch(0.36 0.01 270)` | `--color-border-strong` | Emphasized borders (outline buttons, scrollbars on hover) |
| **Destructive** | `oklch(0.66 0.18 25)` | `--color-destructive` | Stop-generation button, delete actions, validation errors |
| **Warning** | `oklch(0.78 0.14 80)` | `--color-warning` | Reserved for future |
| **Success** | `oklch(0.72 0.15 155)` | `--color-success` | Reserved for future |
| **Bubble user** | `oklch(0.26 0.02 220)` | `--color-bubble-user` | User message bubble background |
| **Bubble character** | `oklch(0.21 0.007 270)` | `--color-bubble-character` | Character message bubble background |

**Accent reserved for (explicit list):**
- Send button background
- New chat button (primary variant)
- Focus ring (`outline: 2px solid var(--color-accent)`)
- Context utilization progress bar fill
- Input border on focus (`focus-within:border-accent`)
- App identity gradient (accent → accent-muted)
- Streaming caret color (`.streaming-caret::after`)
- Selection highlight (via accent-muted)
- **Phase 3 additions:**
  - Step active fill (active step node background in stepper)
  - Step connector completed (completed portion of connector line)
  - Saving spinner (`Loader2` icon on disabled save button)
  - Avatar upload accent states (hover overlay border, upload icon tint)

**Phase 3 semantic color usages (no new tokens):**

| State | Token Used | Usage |
|-------|------------|-------|
| Field error text | `--color-destructive` | Inline validation message below invalid inputs |
| Field error border | `--color-destructive` | Input border when validation fails |
| Step completed fill | `--color-accent-muted` | Completed step node border/background |
| Step future muted | `--color-subtle` | Future step node border and label |
| Step connector line | `--color-border` | Horizontal line connecting step nodes |
| Avatar upload overlay | `--color-background` / `--color-foreground` | Dark overlay on avatar hover with icon |
| Kebab menu hover | `--color-surface-hover` | Menu item hover background |
| Destructive menu item | `--color-destructive` | "Delete" option in kebab menu |

No new color tokens are added.

---

## Radii

**Inherited from Phase 1.** No changes.

| Token | Value | Usage |
|-------|-------|-------|
| `--radius-sm` | 0.375rem (6px) | Small buttons, icon buttons, action chips |
| `--radius-md` | 0.5rem (8px) | Default buttons, inputs, stepper nodes |
| `--radius-lg` | 0.75rem (12px) | Chat bubbles, input frames, card content |
| `--radius-xl` | 1rem (16px) | Dialog panels, modal containers |

**Phase 3 note:** Dialog wrapper uses `--radius-xl` for the outer panel. Avatar upload circle uses `--radius-md` for the clickable avatar itself (since avatars are circular via overflow-hidden on a rounded container, the actual radius of the wrapper is irrelevant — it's a circle).

---

## Component Inventory

### New UI Primitives (`src/lib/components/ui/`)

| Component | Source | Variants | Notes |
|-----------|--------|----------|-------|
| **Dialog** | Custom wrapper around Bits.ui Dialog | — | Generic dialog shell: title, close button, scrollable body, action footer. Used by all editor dialogs. Backdrop blur + dark overlay. |
| **Textarea** | Custom (Bits.ui-free) | `variant`: default; `size`: default | Multi-line text input with auto-grow. Same border/focus styling as Input. Used for description, personality, scenario, system_prompt, mes_example, content. |
| **Select** | Custom wrapper around native `<select>` | — | Styled dropdown with surface-input bg, focus ring on accent. Used for position, activation_state, and model selection. |
| **KebabMenu** | Custom wrapper around Bits.ui DropdownMenu | — | Three-dot trigger button. **Accessibility:** trigger has `aria-label="More options"` + Tooltip. Menu items: Edit, Duplicate (optional), Delete (destructive). Used on CharacterCard and PersonaCard. |
| **StepperHeader** | Custom | — | Horizontal step indicator with numbered circles + labels. Steps: Identity, Personality, Prompt, Greeting. |
| **AvatarUpload** | Custom (wraps Avatar + file picker) | — | Clickable avatar area with upload/change overlay. Shows initials fallback when no image. Triggers Tauri file dialog on click. |

### New Domain Components (`src/lib/components/`)

| Component | Domain | Notes |
|-----------|--------|-------|
| **CharacterEditorDialog** | characters | 4-step wizard inside a Bits.ui Dialog. Mode: 'create' (linear) or 'edit' (free-jump). Max-w-2xl (~640px). |
| **PersonaEditorDialog** | personas | Single-form dialog inside a Bits.ui Dialog. All fields in one scrollable view. Max-w-2xl (~640px). |
| **LorebookEditor** | lorebooks | Dialog-based editor for a single lorebook. Wider: max-w-3xl (~768px). Shows entry list + entry form + keyword test panel. |
| **DeleteConfirmationDialog** | shared | Reusable confirmation dialog. Destructive confirm button, neutral cancel. |

### Modified Existing Components

| Component | Change | What Changes |
|-----------|--------|--------------|
| `CharacterCard` | Add kebab menu | Replace explicit "Edit" button with `MoreVertical` kebab trigger. Add `ondelete` prop. Menu items: Edit, Delete (destructive). |
| `PersonaCard` | Add kebab menu | Replace explicit "Edit" button with kebab trigger. Add `ondelete` prop. Menu items: Edit, Delete (destructive). |
| `characters/+page.svelte` | Wire "New" buttons | `onclick` on "New character" and "New persona" buttons opens respective editor dialogs in `mode: 'create'`. |
| `characters/+page.svelte` | Add Lorebooks tab | New third tab "Lorebooks" in the library Tabs.List. Shows lorebook list. "New lorebook" button in header. |
| `AppState` | New actions | `createCharacter`, `updateCharacter`, `deleteCharacter`, `createPersona`, `updatePersona`, `deletePersona`, `createLorebook`, `updateLorebook`, `deleteLorebook`, `setEditorOpen`, etc. |

---

## Loading States

### Saving State Design Contract

When the user clicks "Save" or "Next" in a wizard, the form enters a **saving state**. The UI must prevent duplicate submissions and communicate progress.

| Element | State | Visual |
|---------|-------|--------|
| Save / Next button | Disabled | `disabled: true`, opacity 50%, cursor not-allowed. Label changes to "Saving…" with `Loader2` icon (animate-spin). |
| Back button | Disabled | `disabled: true` during save. Prevents navigation mid-submit. |
| Stepper header | Disabled | Clicking step headers disabled during save (in edit mode). |
| Form fields | Read-only | `disabled` or `readonly` on all inputs during save. Visual: subtle opacity reduction (not full disabled styling, just muted). |
| Avatar upload | Hidden | Overlay / upload controls hidden during save. |

**Spinner icon:** `Loader2` from `@lucide/svelte` with `class="animate-spin"` at `size-4`.

**Duration target:** Local SQLite saves should complete in < 500ms. The saving state is primarily a safety measure against double-submit, not a long-duration loading indicator.

### Avatar Upload Loading

When the user selects a file via the native file picker, the avatar area shows a brief intermediate state:

| State | Visual |
|-------|--------|
| File selected, uploading | Avatar area shows `Loader2` spinner centered. Label "Uploading…". |
| Upload complete | Avatar area transitions to the uploaded image preview. |
| Upload failed | Temporary error tooltip: "Couldn't upload image." Reverts to previous avatar or initials. |

---

## Empty States

Phase 3 adds new empty states for the Lorebooks tab. Existing empty states from Phase 2 (characters, personas, conversations) are unchanged.

### Empty State Design Pattern

Same pattern as Phase 2: centered flex column with icon, heading, body, and optional CTA.

### Empty State Specifications

| View | Condition | Icon | Heading | Body | CTA |
|------|-----------|------|---------|------|-----|
| Lorebooks tab | No lorebooks in DB | `BookOpen` | "No lorebooks yet." | "Create a lorebook to add world-building context that activates during conversations." | "New lorebook" (primary) |
| Lorebooks tab | Search with no matches | `Search` | "No lorebooks match." | "Try a different search term." | — |
| Lorebook editor | No entries in a lorebook | `FileText` | "No entries yet." | "Add entries with keywords that trigger during conversations." | "Add entry" (primary, within the editor) |

---

## Error States

Phase 3 introduces **validation errors** (form-level, client-side) and **backend failures** during save operations.

### Validation Errors (Client-Side)

**Pattern:** Inline per-field error text below each invalid input, shown in `--color-destructive` with a small `AlertTriangle` icon.

**Character wizard validation:**

| Step | Fields Validated | Rule | Error Message |
|------|------------------|------|---------------|
| Step 1 (Identity) | `name` | Non-empty after trim | "Name is required." |
| Step 1 (Identity) | `description` | Non-empty after trim | "Description is required." |
| Step 2 (Personality) | `personality` | Non-empty after trim | "Personality is required." |
| Step 4 (Greeting) | `first_mes` | Non-empty after trim | "Greeting message is required." |

**Persona dialog validation:**

| Field | Rule | Error Message |
|-------|------|---------------|
| `name` | Non-empty after trim | "Name is required." |

**Lorebook entry validation:**

| Field | Rule | Error Message |
|-------|------|---------------|
| `keys` | At least one non-empty key | "At least one keyword is required." |
| `content` | Non-empty after trim | "Content is required." |

**Validation display:**
```
[Label — 12px, weight 400]
[Input — border turns --color-destructive]
⚠ [Error text — 12px, weight 400, --color-destructive]
```

- Error text: `text-xs text-(--color-destructive)`
- Error icon: `AlertTriangle class="size-3"`
- Input border on error: `border-(--color-destructive)` (replaces default border)
- Input focus on error: `focus-visible:border-(--color-destructive) focus-visible:ring-(--color-destructive)`

**Step-level validation (wizard):**
- When user clicks "Next", validate all required fields on the current step.
- If any field is invalid, scroll the first invalid field into view and show its error.
- "Next" button is blocked until the step passes validation.
- "Save" button (on Step 4) validates all required fields across all steps before submitting.

### Backend Failure Errors

**Pattern:** If a `save_character`, `save_persona`, `save_lorebook`, or `delete_*` command fails, show a non-blocking error notification. Since the app does not have a Toast system yet (deferred), use an inline error banner at the top of the dialog.

**Dialog error banner:**
```
┌─────────────────────────────────────────────┐
│ ⚠ Couldn't save character.                  │
│   Try again. If this keeps happening,       │
│   restart the app.                          │
│                                   [Retry]   │
└─────────────────────────────────────────────┘
```

- Background: `bg-(--color-destructive)/10`
- Border: `border border-(--color-destructive)/20`
- Text: `text-sm text-(--color-foreground)`
- Icon: `AlertTriangle class="size-4 text-(--color-destructive)"`
- Retry button: `variant="secondary" size="sm"`
- Dismiss: Click X or automatic on user interaction (hidden when user changes a field).

**Backend error specifications:**

| Action | Error Banner Heading | Body |
|--------|---------------------|------|
| Save character | "Couldn't save character." | "Something went wrong saving to the database. Try again." |
| Save persona | "Couldn't save persona." | "Something went wrong saving to the database. Try again." |
| Save lorebook | "Couldn't save lorebook." | "Something went wrong saving to the database. Try again." |
| Delete character | "Couldn't delete character." | "Something went wrong deleting from the database. Try again." |
| Delete persona | "Couldn't delete persona." | "Something went wrong deleting from the database. Try again." |
| Avatar upload | "Couldn't upload image." | "Make sure the file is a valid image (PNG, JPG, WEBP)." |

---

## Interaction Patterns

### New Patterns in Phase 3

| Pattern | Trigger | Behavior | Notes |
|---------|---------|----------|-------|
| **Open character editor (create)** | Click "New character" on Library page | CharacterEditorDialog opens with `mode: 'create'`, Step 1 active, all fields empty | Dialog title: "New character". Linear stepper — user must progress through steps. |
| **Open character editor (edit)** | Click "Edit" in kebab menu on CharacterCard | CharacterEditorDialog opens with `mode: 'edit'`, pre-filled data, free-jump stepper | Dialog title: "Edit {name}". Free-jump — clicking any step header jumps to it. |
| **Wizard Next** | Click "Next" button | Validates current step → advances to next step | Invalid fields show errors, scroll into view. |
| **Wizard Back** | Click "Back" button | Returns to previous step, preserves form data | No validation on Back (user can go back to fix). |
| **Wizard Save** | Click "Save" (Step 4) | Validates ALL steps → calls `saveCharacter` → closes dialog on success | On failure: error banner in dialog, form stays open. |
| **Wizard step jump (edit mode)** | Click step header | Jumps to that step, preserves data in all steps | Only enabled in `mode: 'edit'`. Disabled during save. |
| **Open persona editor (create)** | Click "New persona" on Library page | PersonaEditorDialog opens, all fields empty | Dialog title: "New persona". |
| **Open persona editor (edit)** | Click "Edit" in kebab menu on PersonaCard | PersonaEditorDialog opens, pre-filled data | Dialog title: "Edit {name}". |
| **Persona Save** | Click "Save" in persona dialog | Validates name → calls `savePersona` → closes on success | Single-form, no stepper. |
| **Kebab menu open** | Click `MoreVertical` icon on card | Dropdown menu appears below the button | Trigger has `aria-label="More options"`. Uses Bits.ui DropdownMenu primitive. |
| **Kebab → Edit** | Click "Edit" in menu | Opens respective editor dialog | Menu auto-closes on selection. |
| **Kebab → Delete** | Click "Delete" in menu | Opens DeleteConfirmationDialog | Menu auto-closes on selection. |
| **Delete confirm** | Click "Delete" in confirmation dialog | Calls `delete_*` → removes from local state → closes dialog | Delete button is `variant="destructive"`. |
| **Delete cancel** | Click "Cancel" in confirmation dialog | Closes dialog, no action | |
| **Avatar upload click** | Click avatar area in editor | Tauri native file picker opens (image files only) | Accepts: PNG, JPG, JPEG, WEBP. |
| **Avatar replace** | Upload new avatar when one exists | Backend deletes old file, stores new file, returns new `avatar_path` | Auto-cleanup per D-12. |
| **Avatar remove** | Click "Remove" in avatar overlay | Clears avatar_path, reverts to initials fallback | Backend deletes file. |
| **Open lorebook editor** | Click "New lorebook" or "Edit" on lorebook card | LorebookEditor dialog opens | Wider dialog: max-w-3xl. |
| **Add lorebook entry** | Click "Add entry" in lorebook editor | New empty entry form shown, entry list updates | Inline form, no sub-dialog. |
| **Edit lorebook entry** | Click an entry in the list | Entry form pre-filled with selected entry's data | |
| **Delete lorebook entry** | Click trash icon on entry | Inline confirmation: "Delete this entry?" → Yes/No | **Accessibility:** trash icon has `aria-label="Delete entry"`. Or immediate with undo — TBD by implementer. |
| **Test Keywords panel toggle** | Click "Test Keywords" accordion header | Panel expands below entry form, showing test input + results | Collapses on second click. |
| **Keyword test input** | Type in test textarea | Debounced (300ms) call to `test_lorebook_keywords` | Results show matched keys highlighted. |
| **Dialog close (unsaved)** | Click backdrop, press Escape, click X | If form is dirty → confirmation prompt: "Discard changes?" | Uses `onBeforeClose` on Bits.ui Dialog. |

### Existing Patterns (Modified)

| Pattern | Phase 2 Behavior | Phase 3 Change |
|---------|-------------------|----------------|
| **CharacterCard actions** | Direct "Start chat" + "Edit" buttons | "Start chat" stays. "Edit" moved to kebab menu. "Delete" added to kebab menu. |
| **PersonaCard actions** | Direct "Edit" + "Duplicate" buttons | "Edit" moved to kebab menu. "Delete" added to kebab menu. "Duplicate" still no-op (deferred). |
| **Library page tabs** | Characters / Personas | Adds "Lorebooks" tab. Header buttons adapt label. |
| "New character" button | No-op | Opens CharacterEditorDialog in create mode |
| "New persona" button | No-op | Opens PersonaEditorDialog in create mode |

### Deferred Interactions (Not in Phase 3)

| Pattern | Deferred To | Reason |
|---------|-------------|--------|
| Drag-and-drop avatar upload | Future enhancement | Native file picker is the explicit requirement. |
| Character import/export (PNG) | Phase 7 | Card I/O is a separate phase. |
| Toast notifications | Phase 10 | No Toast component exists; error banners serve as interim. |
| Assign lorebooks to characters | Phase 9 | Lorebook assignment UI is part of Power Features. |
| Clone/duplicate character or persona | Future | "Duplicate" button deferred per CONTEXT. |

---

## Copywriting Contract

### CTAs and Button Labels

| Element | Copy | Context |
|---------|------|---------|
| New character button | "New character" | Library page header |
| New persona button | "New persona" | Library page header |
| New lorebook button | "New lorebook" | Library page header (Lorebooks tab) |
| Character dialog title (create) | "New character" | Dialog title bar |
| Character dialog title (edit) | "Edit {character.name}" | Dialog title bar |
| Persona dialog title (create) | "New persona" | Dialog title bar |
| Persona dialog title (edit) | "Edit {persona.name}" | Dialog title bar |
| Lorebook dialog title (create) | "New lorebook" | Dialog title bar |
| Lorebook dialog title (edit) | "Edit {lorebook.name}" | Dialog title bar |
| Wizard Next | "Next" | Step 1–3 footer |
| Wizard Back | "Back" | Step 2–4 footer |
| Wizard Save (create) | "Create character" | Step 4 footer |
| Wizard Save (edit) | "Save changes" | Step 4 footer |
| Persona Save (create) | "Create persona" | Dialog footer |
| Persona Save (edit) | "Save changes" | Dialog footer |
| Lorebook Save (create) | "Create lorebook" | Dialog footer |
| Lorebook Save (edit) | "Save changes" | Dialog footer |
| Saving button label | "Saving…" | During async save |
| Add entry button | "Add entry" | Inside lorebook editor |
| Test Keywords toggle | "Test Keywords" | Accordion header in lorebook editor |
| Test Keywords input placeholder | "Type sample conversation text here…" | Test textarea |
| Test Keywords run | "Run test" | Button in test panel |
| Avatar upload label (empty) | "Upload avatar" | Overlay on empty avatar |
| Avatar change label (exists) | "Change" | Overlay on existing avatar |
| Avatar remove label | "Remove" | Option in avatar context menu (if applicable) |
| Step 1 label | "Identity" | Stepper header |
| Step 2 label | "Personality" | Stepper header |
| Step 3 label | "Prompt" | Stepper header |
| Step 4 label | "Greeting" | Stepper header |

### Empty States

| Element | Copy | Context |
|---------|------|---------|
| Lorebooks — none | "No lorebooks yet." | First-run empty state |
| Lorebooks — none body | "Create a lorebook to add world-building context that activates during conversations." | Empty state description |
| Lorebooks — no search | "No lorebooks match." | Search filter empty |
| Lorebook entries — none | "No entries yet." | Empty entry list in editor |
| Lorebook entries — none body | "Add entries with keywords that trigger during conversations." | Description |

### Error States

| Element | Copy | Context |
|---------|------|---------|
| Name required | "Name is required." | Character/persona validation |
| Description required | "Description is required." | Character validation |
| Personality required | "Personality is required." | Character validation |
| Greeting required | "Greeting message is required." | Character validation |
| Keywords required | "At least one keyword is required." | Lorebook entry validation |
| Content required | "Content is required." | Lorebook entry validation |
| Save character failed | "Couldn't save character." | Backend error banner |
| Save persona failed | "Couldn't save persona." | Backend error banner |
| Save lorebook failed | "Couldn't save lorebook." | Backend error banner |
| Delete failed | "Couldn't delete {entity}." | Backend error banner |
| Avatar upload failed | "Couldn't upload image." | File upload error |
| Avatar upload body | "Make sure the file is a valid image (PNG, JPG, WEBP)." | Upload error hint |

### Destructive Actions

| Action | Confirmation Dialog Heading | Confirmation Body | Confirm Label | Cancel Label |
|--------|---------------------------|-------------------|---------------|--------------|
| Delete character | "Delete {name}?" | "This will permanently remove this character and all associated data. This cannot be undone." | "Delete" (destructive) | "Cancel" |
| Delete persona | "Delete {name}?" | "This will permanently remove this persona. This cannot be undone." | "Delete" (destructive) | "Cancel" |
| Delete lorebook | "Delete {name}?" | "This will permanently remove this lorebook and all its entries. This cannot be undone." | "Delete" (destructive) | "Cancel" |
| Delete lorebook entry | "Delete this entry?" | "This entry will be removed from the lorebook. This cannot be undone." | "Delete" (destructive) | "Cancel" |
| Discard unsaved changes | "Discard changes?" | "You have unsaved changes. Closing this dialog will discard them." | "Discard" (destructive) | "Keep editing" |

---

## Custom Utilities

**No new CSS utilities required.** Existing utilities from Phase 1 (`font-display`, `surface-grain`, `streaming-caret`, `drag-region`, `no-drag`) are sufficient.

Phase 3 uses Tailwind v4 built-in utilities for:
- `animate-spin` — on `Loader2` icon during save
- `data-[state=active]` — Bits-ui Tabs/Dialog state styling (already in use)

---

## Registry Safety

**No changes.** No shadcn, no third-party registries. All components remain custom-built using Bits.ui + Tailwind v4.

| Registry | Blocks Used | Safety Gate |
|----------|-------------|-------------|
| shadcn official | none | not applicable |
| Third-party | none | not applicable |

---

## Phase 3 Specifics

### Discretionary Decisions Made

The following decisions were in "Claude's Discretion" in CONTEXT.md and are hereby declared:

1. **Step header navigation UI style:** Numbered horizontal stepper bar with step labels below. Each step shows a number inside a circle (32px diameter), label below. Active step: accent-filled circle + accent text. Completed: accent border + muted text. Future: subtle border + subtle text. Connector line between steps completes with accent color as user progresses. Source: custom-built, no Bits.ui primitive needed.

2. **Avatar upload button visual design:** Circular click target wrapping the existing `Avatar` component. When no image: large circle (80px) with upload icon (`Upload` from lucide) + "Upload avatar" text centered. When image exists: image preview in circular frame (80px) with a subtle dark overlay on hover revealing "Change" text and upload icon. Click anywhere triggers file picker. Added "Remove" option as a small text link below the avatar for clearing.

3. **Loading state during save:** Disabled form with "Saving…" button label and spinning loader. All inputs and navigation elements disabled. Prevents double-submit without a blocking spinner overlay. Duration is typically <500ms.

4. **Error display inside wizard:** Inline per-field errors. Each invalid field shows a small `AlertTriangle` + error text below the input in `--color-destructive`. Step-level validation blocks "Next" until current step is valid. Final "Save" validates all required fields across all steps.

5. **Lorebook editor opens as dialog (not page):** For consistency with character and persona editors, the lorebook editor opens as a modal dialog (`max-w-3xl`, wider to accommodate entry list + form). The lorebook LIST view is a new "Lorebooks" tab in the Library page. This separates browsing (list/tab) from focused editing (dialog).

6. **"Test Keywords" panel position:** Bottom accordion within the lorebook editor dialog. Below the entry form, a collapsible section titled "Test Keywords". When expanded, shows a textarea for sample text and a results area showing matched entries with highlighted keys. Collapses to save vertical space when not in use.

### What Changes

1. **New dialog components:** `CharacterEditorDialog`, `PersonaEditorDialog`, `LorebookEditor`, `DeleteConfirmationDialog`, `Dialog` (generic wrapper), `KebabMenu`, `StepperHeader`, `AvatarUpload`, `Textarea`, `Select`.
2. **Card component modifications:** `CharacterCard` and `PersonaCard` gain kebab menus and `ondelete` callbacks. Existing buttons reorganized.
3. **Library page expansion:** New "Lorebooks" tab. "New character" / "New persona" buttons now functional.
4. **AppState expansion:** CRUD actions for characters, personas, and lorebooks. Dialog open/close state management.
5. **api.ts expansion:** New commands for lorebooks, avatar upload, and keyword testing.
6. **New types:** `Lorebook`, `LorebookEntry` (standalone, decoupled from `character_id`).
7. **Backend schema change:** Migration for standalone lorebooks (new `lorebooks` table, `lorebook_entries.lorebook_id` FK, `character_lorebooks` junction).
8. **Avatar asset handling:** Tauri file dialog → copy to app data dir → serve via asset protocol.

### What Does NOT Change

- No changes to design tokens (spacing, typography, color, radii)
- No changes to existing page routes (character library stays at `/characters`)
- No new npm dependencies
- No changes to the icon library
- No Toast component (error banners used as interim)
- No changes to ConversationSidebar, ChatHeader, MessageBubble, MessageInput
- No changes to existing navigation (Chat / Library / Settings)
- No drag-and-drop avatar upload
- No character card import/export (Phase 7)

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS
- [ ] Dimension 2 Visuals: PASS
- [ ] Dimension 3 Color: PASS
- [ ] Dimension 4 Typography: PASS
- [ ] Dimension 5 Spacing: PASS
- [ ] Dimension 6 Registry Safety: PASS

**Approval:** pending
