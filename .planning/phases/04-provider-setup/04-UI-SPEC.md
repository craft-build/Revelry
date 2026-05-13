---
phase: 4
slug: provider-setup
status: draft
shadcn_initialized: false
preset: none
created: 2026-05-12
---

# Phase 4 — UI Design Contract

> Visual and interaction contract for the Provider Setup phase. Users configure AI providers with API keys stored in the OS keychain, manage multiple models per provider, and set generation parameters. This phase wires the existing settings page provider cards to a real backend with a new ProviderEditorDialog. All design tokens, typography, color, spacing, and radii are inherited from Phase 1's locked design system.

---

## Design System

**Inherited from Phase 1.** No changes to tooling, tokens, or component library.

| Property          | Value                                                           |
| ----------------- | --------------------------------------------------------------- |
| Tool              | none (no shadcn — project uses Bits.ui + Tailwind v4 directly)  |
| Component library | Bits.ui ^2.18 (headless Svelte primitives)                      |
| Icon library      | @lucide/svelte ^1.14                                            |
| Font              | Geist (sans), Fraunces (serif/display), Geist Mono (mono)       |
| Styling           | Tailwind CSS v4 with custom OKLCH theme tokens in `src/app.css` |
| Variant system    | tailwind-variants ^3.2 (`tv()`)                                 |

**Reference:** `.planning/phases/01-native-desktop-shell/01-UI-SPEC.md` § Design System — all unchanged.

---

## Spacing Scale

**Inherited from Phase 1.** No changes. 4px-multiple scale (4, 8, 12, 16, 20, 24, 32, 40, 48, 64).

**Phase 4 additions:**

| Token                | Value            | Usage                                                     |
| -------------------- | ---------------- | --------------------------------------------------------- |
| `dialog-gutter`      | 24px (px-6)      | Horizontal padding inside modal dialogs (inherited Ph 3)  |
| `dialog-section`     | 24px (gap-6)     | Vertical spacing between dialog sections                  |
| `form-field-gap`     | 12px (gap-3)     | Spacing between label, input, and helper text (inherited) |
| `provider-card-gap`  | 12px (gap-3)     | Gap between provider cards in the list                    |
| `model-row-gap`      | 8px (gap-2)      | Gap between model entry rows in the editor                |
| `param-grid-col-gap` | 12px (gap-3)     | Column gap in the generation params grid                  |
| `param-grid-row-gap` | 16px (gap-4)     | Row gap in the generation params grid                     |
| `keyring-banner-pad` | 16px (px-4 py-3) | Padding for the keyring-unavailable warning banner        |

All new spacing uses existing Tailwind token values. No custom spacing values introduced.

---

## Typography

**Inherited from Phase 1.** Consolidated to 4 sizes and 2 weights per design system rules.

**Type scale (4 sizes, 2 weights):**

| Role                                                       | Size        | Weight         | Line Height     | Font Family                             |
| ---------------------------------------------------------- | ----------- | -------------- | --------------- | --------------------------------------- |
| Body (chat bubbles, input, provider card content)          | 15px        | 400 (regular)  | 1.625 (relaxed) | Geist (sans)                            |
| Heading (speaker names, card titles, dialog title)         | 14px–18px\* | 600 (semibold) | 1.2             | Fraunces (serif/display) / Geist (sans) |
| Label / small (sidebar items, nav, metadata, field labels) | 12px (xs)   | 400 (regular)  | 1.25            | Geist (sans)                            |
| Textarea body / model name display                         | 14px (sm)   | 400 (regular)  | 1.625           | Geist (sans)                            |

\* Dialog title: 18px Geist (sans), weight 600. Card titles: 14px Fraunces (serif/display), weight 600.

**Phase 4 role breakdown by element:**

| Element                       | Size | Weight | Font Family | Notes                                                                |
| ----------------------------- | ---- | ------ | ----------- | -------------------------------------------------------------------- |
| Dialog title                  | 18px | 600    | Geist sans  | "New provider" / "Edit {name}"                                       |
| Section heading in dialog     | 14px | 600    | Geist sans  | "Models", "Generation parameters"                                    |
| Field label                   | 12px | 400    | Geist sans  | "Name", "Type", "Base URL", "API Key"                                |
| Field error                   | 12px | 400    | Geist sans  | `--color-destructive`                                                |
| Provider card name            | 14px | 600    | Fraunces    | Card title in settings list                                          |
| Provider type tag             | 10px | 500    | Geist sans  | **Inherited** from Phase 3 Tag component — not a Phase 4 declaration |
| Model name (mono)             | 12px | 400    | Geist Mono  | `font-mono text-xs` in card and model rows (Label/small role)        |
| Generation param label        | 12px | 400    | Geist sans  | "Temperature", "Top P", "Max tokens"                                 |
| Generation param value        | 12px | 400    | Geist Mono  | `font-mono text-xs` for numeric values                               |
| Keyring banner text           | 14px | 400    | Geist sans  | Warning message body                                                 |
| Keyring banner heading        | 14px | 600    | Geist sans  | Warning message title                                                |
| "Key set ✓" / "No key" status | 12px | 400    | Geist sans  | API key status indicator on provider card                            |

**No 500 (medium) weight in Phase 4 declarations.** The Tag component's built-in `font-medium` at 10px is inherited from Phase 3 — it is not a Phase 4 type declaration. All Phase 4 labels, nav items, and metadata use 400. All Phase 4 headings, dialog titles, and emphasis use 600.

---

## Color

All colors defined as OKLCH values in `src/app.css` @theme block. Dark theme only — light theme deferred to Phase 10.

| Role                | OKLCH Value             | Tailwind Token             | Usage                                                   |
| ------------------- | ----------------------- | -------------------------- | ------------------------------------------------------- |
| **Dominant (60%)**  | `oklch(0.16 0.005 270)` | `--color-background`       | App background, page-level surface                      |
| **Secondary (30%)** | `oklch(0.19 0.006 270)` | `--color-surface`          | Sidebar, provider cards, input area backgrounds         |
|                     | `oklch(0.22 0.008 270)` | `--color-surface-elevated` | Elevated panels, active nav tab                         |
|                     | `oklch(0.24 0.009 270)` | `--color-surface-hover`    | Hover state backgrounds                                 |
|                     | `oklch(0.21 0.007 270)` | `--color-surface-input`    | Text input backgrounds                                  |
| **Accent (10%)**    | `oklch(0.78 0.13 195)`  | `--color-accent`           | See reserved-for list below                             |
|                     | `oklch(0.18 0.02 200)`  | `--color-accent-fg`        | Text on accent backgrounds                              |
|                     | `oklch(0.32 0.06 195)`  | `--color-accent-muted`     | Selection highlight, progress bars, subtle accent fills |
| **Foreground**      | `oklch(0.96 0.005 270)` | `--color-foreground`       | Primary text color                                      |
| **Muted**           | `oklch(0.65 0.012 270)` | `--color-muted`            | Secondary text, timestamps, previews                    |
| **Subtle**          | `oklch(0.45 0.01 270)`  | `--color-subtle`           | Placeholder text, disabled icons, metadata              |
| **Border**          | `oklch(0.28 0.008 270)` | `--color-border`           | Default borders                                         |
| **Border strong**   | `oklch(0.36 0.01 270)`  | `--color-border-strong`    | Emphasized borders                                      |
| **Destructive**     | `oklch(0.66 0.18 25)`   | `--color-destructive`      | Delete actions, validation errors                       |
| **Warning**         | `oklch(0.78 0.14 80)`   | `--color-warning`          | **Phase 4 activation:** keyring-unavailable banner      |
| **Success**         | `oklch(0.72 0.15 155)`  | `--color-success`          | **Phase 4 activation:** "Key set ✓" indicator           |

**Accent reserved for (explicit list):**

- Send button background
- New chat button (primary variant)
- Focus ring (`outline: 2px solid var(--color-accent)`)
- Context utilization progress bar fill
- Input border on focus (`focus-within:border-accent`)
- App identity gradient (accent → accent-muted)
- Streaming caret color
- Selection highlight (via accent-muted)
- Step active fill / connector completed (Phase 3)
- Saving spinner (Phase 3)
- **Phase 4 additions:**
  - "Save provider" button (primary variant) in editor dialog footer
  - Provider type group selected indicator (accent border on the active group)

**Phase 4 semantic color usages (no new tokens):**

| State                             | Token Used              | Usage                                                       |
| --------------------------------- | ----------------------- | ----------------------------------------------------------- |
| Keyring unavailable banner bg     | `--color-warning`/10    | 10% opacity warning background for the persistent banner    |
| Keyring unavailable banner border | `--color-warning`/20    | 20% opacity warning border                                  |
| Keyring unavailable icon          | `--color-warning`       | `AlertTriangle` icon tint                                   |
| API key "set" indicator           | `--color-success`       | `Check` icon + "Key set" text on provider card              |
| API key "not set" indicator       | `--color-subtle`        | "No key" text on provider card                              |
| Field error text                  | `--color-destructive`   | Inline validation message below invalid inputs              |
| Field error border                | `--color-destructive`   | Input border when validation fails                          |
| Disabled API key field (keyring)  | `--color-subtle`        | Placeholder text + reduced opacity when keyring unavailable |
| Provider type tag variant         | `Tag` component default | `bg-surface-elevated text-muted border-border`              |

No new color tokens are added.

---

## Radii

**Inherited from Phase 1.** No changes.

| Token         | Value          | Usage                                      |
| ------------- | -------------- | ------------------------------------------ |
| `--radius-sm` | 0.375rem (6px) | Small buttons, icon buttons, action chips  |
| `--radius-md` | 0.5rem (8px)   | Default buttons, inputs, stepper nodes     |
| `--radius-lg` | 0.75rem (12px) | Chat bubbles, provider cards, card content |
| `--radius-xl` | 1rem (16px)    | Dialog panels, modal containers            |

**Phase 4 note:** ProviderEditorDialog uses `--radius-xl` for the dialog panel (inherited from Dialog component). Provider cards use `--radius-lg`. Model entry rows within the editor use `--radius-md`.

---

## Component Inventory

### New UI Primitives (`src/lib/components/ui/`)

No new UI primitives required. All Phase 4 UI is built from existing components:

| Component | Phase 4 Usage                                                    |
| --------- | ---------------------------------------------------------------- |
| Dialog    | Shell for ProviderEditorDialog                                   |
| Button    | Save, Cancel, Add model, Remove model, Edit, Delete              |
| Input     | Provider name, base URL, API key, model name, model context size |
| Select    | Provider type grouped selector                                   |
| Tag       | Provider type badge on card                                      |
| Textarea  | Not used in Phase 4                                              |
| KebabMenu | Provider card actions (Edit, Delete)                             |
| Tooltip   | API key reveal toggle, keyring warning context                   |

### New Domain Components (`src/lib/components/`)

| Component                    | Domain    | Notes                                                                                                                                                                                  |
| ---------------------------- | --------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **ProviderEditorDialog**     | providers | Single-form dialog inside a Bits.ui Dialog. Mode: `'create'` or `'edit'`. Max-w-lg (~512px). Sections: Identity (name, type, base URL), API Key, Models (list), Generation Parameters. |
| **KeyringUnavailableBanner** | settings  | Inline warning banner at the top of the providers section. Persistent while `app.keyring_available === false`.                                                                         |
| **ModelEntryRow**            | providers | Single model row within ProviderEditorDialog: model name (Input), context window size (Input, optional), enabled toggle, remove button.                                                |

### Modified Existing Components

| Component                    | Change                | What Changes                                                                                                                                                                                                                                                                                     |
| ---------------------------- | --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `settings/+page.svelte`      | Wire provider cards   | Replace stub "Edit" button with kebab menu (Edit + Delete). Wire "Add provider" button to open ProviderEditorDialog. Add KeyringUnavailableBanner. Add API key status ("Key set ✓" / "No key") to cards. Show models list in card instead of single model. Add kebab menu to each provider card. |
| `AppState` (`app.svelte.ts`) | New state + actions   | Add `keyring_available = $state<boolean>(true)`. Add `createProvider`, `updateProvider`, `deleteProvider` actions. Optimistic update + backend persist + revert on failure.                                                                                                                      |
| `api.ts`                     | New invoke wrappers   | Add: `getProviders`, `saveProvider`, `deleteProvider`, `saveApiKey`, `deleteApiKey`, `checkKeyringAvailable`                                                                                                                                                                                     |
| `types/index.ts`             | Updated types         | Add `models` array to Provider, `ProviderModelEntry` type, `has_api_key` boolean, `model_name` on Conversation                                                                                                                                                                                   |
| `+layout.svelte`             | Startup keyring check | Call `checkKeyringAvailable` during hydration, set `app.keyring_available`                                                                                                                                                                                                                       |

---

## Loading States

### Saving State Design Contract

When the user clicks "Save provider" in the editor dialog, the form enters a **saving state**. This follows the Phase 3 saving state pattern.

| Element                    | State     | Visual                                                                                                            |
| -------------------------- | --------- | ----------------------------------------------------------------------------------------------------------------- |
| Save button                | Disabled  | `disabled: true`, opacity 50%, cursor not-allowed. Label changes to "Saving…" with `Loader2` icon (animate-spin). |
| Cancel button              | Disabled  | `disabled: true` during save. Prevents closing mid-submit.                                                        |
| Form fields                | Read-only | `disabled` on all inputs, selects, and toggles during save. Subtle opacity reduction.                             |
| Add model / Remove buttons | Disabled  | Prevents model list mutation during save.                                                                         |

**Spinner icon:** `Loader2` from `@lucide/svelte` with `class="animate-spin"` at `size-4`.

### Keyring Check Loading

On app startup, `check_keyring_available` is called. This is a one-time async check:

| State              | Visual                                                                 |
| ------------------ | ---------------------------------------------------------------------- |
| Checking (startup) | No visible UI — happens during splash/hydration, before user interacts |
| Available          | No banner shown. Provider editor shows API key field normally.         |
| Unavailable        | KeyringUnavailableBanner shown in settings providers section.          |

### Delete Provider Loading

When the user confirms deletion in DeleteConfirmationDialog:

| Element       | State    | Visual                                                           |
| ------------- | -------- | ---------------------------------------------------------------- |
| Delete button | Disabled | Label changes to "Deleting…". `Loader2` spinner.                 |
| Cancel button | Disabled | Prevents dismiss mid-delete.                                     |
| Provider card | Dimmed   | Slightly reduced opacity (opacity-50) on the card being deleted. |

---

## Empty States

### Empty State Specifications

| View                 | Condition          | Icon  | Heading             | Body                                                                                      | CTA                      |
| -------------------- | ------------------ | ----- | ------------------- | ----------------------------------------------------------------------------------------- | ------------------------ |
| Providers section    | No providers in DB | `Cpu` | "No providers yet." | "Add an AI provider to start chatting. API keys are stored securely in your OS keychain." | "Add provider" (primary) |
| Models list (editor) | No models added    | `Box` | "No models added."  | "Add at least one model that this provider supports. The default model is pre-filled."    | "Add model" (secondary)  |

**Empty state pattern:** Same as Phase 2/3 — centered flex column with icon, heading, body, and optional CTA. Within the dialog, the models empty state is more compact (no large icon, just text + button).

---

## Error States

### Validation Errors (Client-Side)

**Pattern:** Inline per-field error text below each invalid input, shown in `--color-destructive` with a small `AlertTriangle` icon (same as Phase 3).

**Provider editor validation:**

| Field                       | Rule                                         | Error Message                                  |
| --------------------------- | -------------------------------------------- | ---------------------------------------------- |
| `name`                      | Non-empty after trim                         | "Name is required."                            |
| `provider_type` (create)    | Must be selected                             | "Provider type is required."                   |
| `base_url` (create)         | Required for Custom and Azure OpenAI         | "Base URL is required for this provider type." |
| `api_key` (create)          | Required for non-Ollama if keyring available | "API key is required."                         |
| Model `name`                | Non-empty after trim                         | "Model name is required."                      |
| Model `context_window_size` | If provided, must be positive integer        | "Context size must be a positive number."      |
| `temperature`               | If provided, 0–2 range                       | "Temperature must be between 0 and 2."         |
| `top_p`                     | If provided, 0–1 range                       | "Top P must be between 0 and 1."               |
| `max_tokens`                | If provided, positive integer                | "Max tokens must be a positive number."        |

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

### Backend Failure Errors

**Pattern:** If `save_provider`, `delete_provider`, or keyring operations fail, show an inline error banner at the top of the dialog (same as Phase 3 pattern).

**Dialog error banner:**

```
┌─────────────────────────────────────────────┐
│ ⚠ Couldn't save provider.                    │
│   Something went wrong. Try again.           │
│                                   [Retry]    │
└─────────────────────────────────────────────┘
```

- Background: `bg-(--color-destructive)/10`
- Border: `border border-(--color-destructive)/20`
- Text: `text-sm text-(--color-foreground)`
- Icon: `AlertTriangle class="size-4 text-(--color-destructive)"`
- Retry button: `variant="secondary" size="sm"`
- Dismiss: Click X or automatic when user changes a field.

**Backend error specifications:**

| Action          | Error Banner Heading        | Body                                                               |
| --------------- | --------------------------- | ------------------------------------------------------------------ |
| Save provider   | "Couldn't save provider."   | "Something went wrong saving to the database. Try again."          |
| Delete provider | "Couldn't delete provider." | "Something went wrong deleting from the database. Try again."      |
| Save API key    | "Couldn't save API key."    | "The OS keychain could not be accessed. Check your keyring setup." |
| Delete API key  | "Couldn't delete API key."  | "The OS keychain could not be accessed. Check your keyring setup." |

---

## Interaction Patterns

### New Patterns in Phase 4

| Pattern                           | Trigger                                          | Behavior                                                                                          | Notes                                                                                             |
| --------------------------------- | ------------------------------------------------ | ------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| **Open provider editor (create)** | Click "Add provider" button on settings page     | ProviderEditorDialog opens with `mode: 'create'`, all fields empty, provider type selector active | Dialog title: "New provider". Dashed "Add provider" button at bottom of provider list.            |
| **Open provider editor (edit)**   | Click "Edit" in kebab menu on provider card      | ProviderEditorDialog opens with `mode: 'edit'`, pre-filled data, provider type locked (read-only) | Dialog title: "Edit {name}". Provider type is immutable after creation.                           |
| **Select provider type (create)** | Select from grouped `<select>` dropdown          | Auto-fills base_url from `default_base_url()` and adds default model from `default_model_name()`  | Optgroups: Cloud, Local, Gateway, Custom. Selecting Ollama hides the API key field.               |
| **Add model row**                 | Click "Add model" button in editor               | New empty ModelEntryRow appended to the models list                                               | At least one model is required. Default model pre-filled on type selection.                       |
| **Remove model row**              | Click `X` / trash icon on a model row            | Model row removed from the list                                                                   | If only one model remains, remove button is disabled. At least one model must remain.             |
| **Toggle model enabled**          | Click toggle/checkbox on model row               | Model `enabled` flag toggled. Disabled models dimmed in the row.                                  | Disabled models won't appear in conversation model selection (Phase 5).                           |
| **API key reveal toggle**         | Click eye icon on API key field in provider card | Toggles between masked ("•••••••") and plaintext display                                          | Only available if `has_api_key === true`. Key is never sent to the frontend — always masked.      |
| **Save provider (create)**        | Click "Save provider" in dialog footer           | Validates all fields → saves provider to DB → saves API key to keyring → closes dialog            | On keyring failure: saves provider to DB but shows keyring error banner.                          |
| **Save provider (edit)**          | Click "Save changes" in dialog footer            | Validates all fields → saves provider to DB → saves API key to keyring → closes dialog            | If API key field is empty and previously set, deletes key from keyring.                           |
| **Delete provider**               | Click "Delete" in kebab menu → confirm dialog    | DeleteConfirmationDialog → deletes provider from DB → deletes API key from keyring → removes card | Keyring deletion is best-effort (silently continues if keyring fails after provider DB deletion). |
| **Kebab menu on provider card**   | Click `MoreVertical` icon on card                | Dropdown menu: Edit, Delete (destructive)                                                         | Same pattern as CharacterCard / PersonaCard kebab menus from Phase 3.                             |
| **Keyring banner dismiss**        | N/A — persistent banner                          | Banner stays visible as long as `keyring_available === false`. Cannot be dismissed.               | Only visible when the user is viewing the Providers section in settings.                          |
| **Dialog close (unsaved)**        | Click backdrop, Escape, X button                 | If form is dirty → DeleteConfirmationDialog: "Discard changes?" → Discard / Keep editing          | Uses `onBeforeClose` on Bits.ui Dialog (same as Phase 3 pattern).                                 |

### Keyring Unavailability Flow

| Condition                              | UI Response                                                                                                                                                            |
| -------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Keyring available                      | Normal operation. API key field active. Banner hidden.                                                                                                                 |
| Keyring unavailable, viewing providers | KeyringUnavailableBanner visible at top of providers section. API key field in editor is disabled with placeholder "Keychain unavailable — API keys cannot be stored." |
| Keyring unavailable, Ollama type       | Provider can still be created (no API key needed). Full functionality for local providers.                                                                             |
| Keyring unavailable, cloud type        | Provider can be created/edited, but API key field is disabled. Provider card shows "No key" status. User can still configure base URL, models, and generation params.  |

### Deferred Interactions (Not in Phase 4)

| Pattern                          | Deferred To | Reason                                                      |
| -------------------------------- | ----------- | ----------------------------------------------------------- |
| Model selection in conversation  | Phase 5     | Conversation view needs to show provider + model dropdown   |
| Per-conversation param overrides | Phase 5     | Generation param overrides per conversation                 |
| Test API connection              | Future      | "Test connection" button deferred, not in PROV requirements |
| Provider import/export           | Future      | Not in requirements                                         |
| Toast notifications              | Phase 10    | Error banners serve as interim                              |

---

## Copywriting Contract

### CTAs and Button Labels

| Element                              | Copy                                  | Context                                       |
| ------------------------------------ | ------------------------------------- | --------------------------------------------- |
| Add provider button (card list)      | "Add provider"                        | Dashed button at bottom of provider list      |
| Provider dialog title (create)       | "New provider"                        | Dialog title bar                              |
| Provider dialog title (edit)         | "Edit {name}"                         | Dialog title bar                              |
| Save provider (create)               | "Save provider"                       | Dialog footer, primary button                 |
| Save provider (edit)                 | "Save changes"                        | Dialog footer, primary button                 |
| Cancel (editor dialog)               | "Cancel"                              | Dialog footer, secondary button               |
| Saving button label                  | "Saving…"                             | During async save                             |
| Add model row button                 | "Add model"                           | Below model list in editor, secondary variant |
| Remove model row button              | (X icon, `aria-label="Remove model"`) | Icon button at end of model row               |
| Provider type select placeholder     | "Select provider type…"               | Default option in `<select>`                  |
| API key input placeholder (create)   | "Enter your API key…"                 | API key field when no key exists              |
| API key input placeholder (edit)     | "Key set •••••••"                     | API key field when `has_api_key === true`     |
| API key placeholder (keyring off)    | "Keychain unavailable"                | API key field when keyring is unavailable     |
| Base URL input placeholder           | "https://api.example.com/v1"          | Base URL field placeholder                    |
| Model name input placeholder         | "e.g. gpt-4o-mini"                    | Model name field in model row                 |
| Context size input placeholder       | "e.g. 128000"                         | Context window size field in model row        |
| Provider name input placeholder      | "e.g. My OpenAI"                      | Name field in editor                          |
| Provider type select optgroup labels | "Cloud", "Local", "Gateway", "Custom" | Group headers in the `<select>` dropdown      |

### Empty States

| Element                   | Copy                                                                                      | Context                    |
| ------------------------- | ----------------------------------------------------------------------------------------- | -------------------------- |
| Providers — none          | "No providers yet."                                                                       | First-run empty state      |
| Providers — none body     | "Add an AI provider to start chatting. API keys are stored securely in your OS keychain." | Empty state description    |
| Models — none (in editor) | "No models added."                                                                        | Empty model list in editor |
| Models — none body        | "Add at least one model that this provider supports. The default model is pre-filled."    | Description                |

### Error States

| Element                  | Copy                                           | Context                          |
| ------------------------ | ---------------------------------------------- | -------------------------------- |
| Name required            | "Name is required."                            | Editor validation                |
| Provider type required   | "Provider type is required."                   | Editor validation (create only)  |
| Base URL required        | "Base URL is required for this provider type." | Custom / Azure OpenAI validation |
| API key required         | "API key is required."                         | Non-Ollama, keyring available    |
| Model name required      | "Model name is required."                      | Model row validation             |
| Context size invalid     | "Context size must be a positive number."      | Model row validation             |
| Temperature out of range | "Temperature must be between 0 and 2."         | Generation params validation     |
| Top P out of range       | "Top P must be between 0 and 1."               | Generation params validation     |
| Max tokens invalid       | "Max tokens must be a positive number."        | Generation params validation     |
| Save provider failed     | "Couldn't save provider."                      | Backend error banner             |
| Delete provider failed   | "Couldn't delete provider."                    | Backend error banner             |
| Save API key failed      | "Couldn't save API key."                       | Backend error banner             |
| Delete API key failed    | "Couldn't delete API key."                     | Backend error banner             |

### Destructive Actions

| Action                  | Confirmation Dialog Heading | Confirmation Body                                                                                                                                                           | Confirm Label           | Cancel Label   |
| ----------------------- | --------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------- | -------------- |
| Delete provider         | "Delete {name}?"            | "This will permanently remove this provider, its models, and the stored API key. Conversations using this provider will lose their model reference. This cannot be undone." | "Delete" (destructive)  | "Cancel"       |
| Discard unsaved changes | "Discard changes?"          | "You have unsaved changes. Closing this dialog will discard them."                                                                                                          | "Discard" (destructive) | "Keep editing" |

### Keyring Warning Banner

| Element        | Copy                                                                                                                                                            | Context            |
| -------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------ |
| Banner heading | "OS keychain unavailable"                                                                                                                                       | Banner title       |
| Banner body    | "API keys cannot be stored. You can still configure Ollama (local) providers. To fix this, ensure a keyring service is running (e.g., GNOME Keyring on Linux)." | Banner description |

---

## Component Contracts

### ProviderEditorDialog

**File:** `src/lib/components/providers/ProviderEditorDialog.svelte`

**Props:**

```typescript
interface Props {
  open?: boolean; // bindable
  onopenchange?: (open: boolean) => void;
  mode: "create" | "edit"; // controls type selector mutability
  provider?: Provider; // pre-fill data for edit mode
  hasApiKey?: boolean; // from backend, for showing key status
  keyringAvailable?: boolean; // controls API key field state
  onsave?: (data: ProviderFormData) => Promise<void>;
}
```

**Layout:** Single scrollable form inside Dialog (max-w-lg, ~512px). No stepper — all sections visible in one scrollable view.

**Sections (top to bottom):**

1. **Identity** — Name (Input), Provider Type (Select with optgroups, disabled in edit mode), Base URL (Input)
2. **API Key** — API Key (Input type=password, with reveal toggle). Hidden when `provider_type === 'ollama'`. Disabled when `keyringAvailable === false`. On edit: shows placeholder "Key set •••••••" if `hasApiKey === true`.
3. **Models** — Section heading "Models". List of ModelEntryRow components. "Add model" button below. At least one model required.
4. **Generation Parameters** — Grid layout (3 columns): Temperature, Top P, Max Tokens. Each is an Input with `type="number"` and step attribute.

**Footer:** Cancel (secondary) + Save (primary). Save label: "Save provider" (create) / "Save changes" (edit).

**Behavioral contract:**

- On create: Provider type selector is active. Selecting a type auto-fills base_url and adds a default model entry.
- On edit: Provider type selector is disabled (visually grayed, not removable). Base URL and models are editable.
- API key field: On create mode, required for non-Ollama types when keyring is available. On edit mode, optional — if left empty and `hasApiKey` was true, the existing key is retained (not overwritten). If explicitly cleared, key is deleted from keyring.
- At least one model must exist. If the user removes the last model, show inline error.
- Save validates all fields, then calls `onsave`. On success, dialog closes. On failure, error banner appears.
- Unsaved changes detection: compare initial form state vs. current. If dirty and user tries to close, show "Discard changes?" confirmation.

### ModelEntryRow

**File:** `src/lib/components/providers/ModelEntryRow.svelte`

**Props:**

```typescript
interface Props {
  name: string; // bindable
  contextWindowSize?: number; // bindable, optional
  enabled: boolean; // bindable
  removable: boolean; // false if this is the only model row
  onremove?: () => void;
}
```

**Layout:** Horizontal flex row with:

- Model name Input (flex-1, `placeholder="e.g. gpt-4o-mini"`)
- Context window size Input (w-28, `type="number"`, `placeholder="e.g. 128000"`)
- Enabled toggle/checkbox (small toggle switch or checkbox)
- Remove button (`X` icon, `size="icon-sm"`, disabled if `removable === false`)

**Styling:** Row has `border border-(--color-border) rounded-(--radius-md) p-2 gap-2` with `bg-(--color-surface)`. When `enabled === false`, row has `opacity-50`.

### KeyringUnavailableBanner

**File:** `src/lib/components/providers/KeyringUnavailableBanner.svelte`

**Props:** None — reads from `app.keyring_available`.

**Layout:** Full-width banner at top of providers section. Warning styling:

- Background: `bg-(--color-warning)/10`
- Border: `border border-(--color-warning)/20 rounded-(--radius-lg)`
- Icon: `AlertTriangle class="size-4 text-(--color-warning)"`
- Heading: "OS keychain unavailable" (14px, weight 600)
- Body: "API keys cannot be stored…" (14px, weight 400, `--color-muted`)
- Padding: `px-4 py-3`

**Visibility:** `app.keyring_available === false`. Cannot be dismissed — persistent while condition holds.

### Provider Card (settings page, modified)

**Current:** Shows provider name, type tag, model name (mono), generation params grid, API key field with reveal toggle.

**Phase 4 changes:**

1. Replace "Edit" button with `KebabMenu` (MoreVertical trigger). Menu items: Edit, Delete (destructive).
2. Replace single model name display with models list: show first 3 model names in `font-mono text-xs`, with "+N more" if more than 3.
3. Replace mock API key input with API key status indicator:
   - `has_api_key === true`: `Check` icon (green/success) + "Key set" text
   - `has_api_key === false`: "No key" text (subtle)
4. Kebab trigger: `size="icon-sm"` aligned to the right of the card header.

---

## Provider Type Selector Details

### Grouped Select with Optgroups

The provider type selector uses native `<select>` with `<optgroup>` elements. This leverages the existing `Select` component (native `<select>` wrapper) and adds optgroup children.

**Group structure:**

```html
<select bind:value="{providerType}">
  <option value="" disabled selected>Select provider type…</option>
  <optgroup label="Cloud">
    <option value="openai">OpenAI</option>
    <option value="anthropic">Anthropic</option>
    <option value="gemini">Gemini</option>
    <option value="cohere">Cohere</option>
    <option value="mistral">Mistral</option>
    <option value="deepseek">DeepSeek</option>
    <option value="groq">Groq</option>
    <option value="azure_openai">Azure OpenAI</option>
    <option value="huggingface">HuggingFace</option>
    <option value="perplexity">Perplexity</option>
    <option value="xai">xAI</option>
  </optgroup>
  <optgroup label="Local">
    <option value="ollama">Ollama</option>
  </optgroup>
  <optgroup label="Gateway">
    <option value="openrouter">OpenRouter</option>
    <option value="together">Together</option>
  </optgroup>
  <optgroup label="Custom">
    <option value="custom_openai_compatible">OpenAI-compatible</option>
  </optgroup>
</select>
```

**On type selection (create mode):**

1. `base_url` field auto-fills from `default_base_url()` (or clears for Custom/Azure).
2. If models list is empty, adds a default model entry with `name` from `default_model_name()`.
3. If type is Ollama, API key section hides (collapses with no space).

**On type change (create mode):**

1. If user had already modified base_url or models, prompt: "Changing provider type will reset base URL and models. Continue?" — Yes/No.
2. If Yes: reset to new type defaults. If No: revert select to previous type.

**In edit mode:** `<select>` has `disabled` attribute. Provider type is immutable after creation.

---

## Generation Parameters Grid

**Layout:** 3-column grid within the ProviderEditorDialog.

| Field       | Type           | Default | Step | Min | Max | Width  |
| ----------- | -------------- | ------- | ---- | --- | --- | ------ |
| Temperature | number (Input) | —       | 0.1  | 0   | 2   | w-full |
| Top P       | number (Input) | —       | 0.05 | 0   | 1   | w-full |
| Max tokens  | number (Input) | —       | 1    | 1   | —   | w-full |

All three fields are **optional** — leaving them blank means "use the provider/model default." No defaults are pre-filled.

**Labels above inputs:** 12px, weight 400, `--color-subtle`.

**Grid markup:**

```html
<div class="grid grid-cols-3 gap-x-3 gap-y-4">
  <div>
    <label class="mb-1.5 block text-xs text-(--color-subtle)"
      >Temperature</label
    >
    <input
      type="number"
      step="0.1"
      min="0"
      max="2"
      placeholder="—"
      bind:value="{temperature}"
    />
  </div>
  <!-- Top P, Max tokens similarly -->
</div>
```

**Provider card display (read-only):** Same 3-column grid as current stub, showing values in `font-mono text-xs`. Blank values show "—" em dash.

---

## Custom Utilities

**No new CSS utilities required.** Existing utilities from Phase 1 (`font-display`, `surface-grain`, `streaming-caret`, `drag-region`, `no-drag`) are sufficient.

Phase 4 uses Tailwind v4 built-in utilities for:

- `animate-spin` — on `Loader2` icon during save
- `data-[state=active]` — Bits.ui Dialog state styling
- Number input `step`, `min`, `max` attributes for generation params

---

## Registry Safety

**No changes.** No shadcn, no third-party registries. All components remain custom-built using Bits.ui + Tailwind v4.

| Registry        | Blocks Used | Safety Gate    |
| --------------- | ----------- | -------------- |
| shadcn official | none        | not applicable |
| Third-party     | none        | not applicable |

---

## Phase 4 Specifics

### Discretionary Decisions Made

The following decisions were in "Claude's Discretion" in CONTEXT.md and are hereby declared:

1. **Provider editor layout — single scrollable form (no stepper):** Unlike the character editor (4-step wizard), the provider editor is a simple form with at most 4 short sections. A stepper would add unnecessary navigation complexity for a form that fits in one view. The dialog is max-w-lg (~512px) with a scrollable body.

2. **Section order in ProviderEditorDialog:** Identity → API Key → Models → Generation Parameters. This order follows the mental model: "What is it?" → "How does it authenticate?" → "What models does it serve?" → "How does it generate?" The API key section is placed second so it can conditionally hide/show based on provider type without disrupting the flow.

3. **API key field behavior on edit:** When editing an existing provider, the API key field shows placeholder text "Key set •••••••" if `has_api_key === true`. The actual key is never sent to the frontend. The user can:
   - Leave the field empty → existing key is retained (no change).
   - Enter a new key → old key is replaced in keyring.
   - There is no "clear key" button in the dialog — to clear a key, the user would delete the provider and recreate it. This avoids accidental key loss.

4. **Model entry row layout — horizontal inline:** Each model row is a compact horizontal layout: model name input (flex-1) + context window size input (w-28) + enabled toggle + remove button. This keeps the models section scannable without excessive vertical space. The enabled toggle uses a simple checkbox (not a toggle switch) to stay compact.

5. **Provider type change confirmation in create mode:** If the user changes the provider type after already editing the base URL or models, a confirmation prompt asks if they want to reset. This prevents accidental data loss while keeping the create flow smooth for the common case (select type once, edit details).

6. **`has_api_key` boolean field from backend:** The backend returns `has_api_key: boolean` alongside provider data (not the actual key). This lets the frontend display "Key set ✓" / "No key" status without ever receiving the key. Confirmed per CONTEXT.md discretion item.

7. **Keyring test entry naming:** The startup keyring availability check creates and deletes a test entry with service name "revelry" and username `__keyring_test__`. The double-underscore prefix makes it clearly a system entry, not a user provider.

8. **Provider card models display — show up to 3:** On the settings page provider card, show the first 3 enabled model names in `font-mono text-xs` (12px), separated by `·`. If more than 3 models exist, append "+N more". This replaces the current single `provider.model` display.

9. **Generation params all optional:** Temperature, Top P, and Max Tokens fields in the editor are optional (no pre-filled defaults). Leaving them blank means "use the provider/model default." This avoids imposing arbitrary defaults that might not match the user's provider.

10. **`model_name` column on `providers` table — kept as default/primary model:** The existing `model` column (renamed to `model_name` conceptually) stays as a quick reference for the primary/default model. The `models` JSON array is the canonical source. When displaying a provider card, use the first enabled model from the array (or `model_name` as fallback). The planner can evaluate if this redundancy is worth keeping — the UI design works with either approach.

### What Changes

1. **New domain components:** `ProviderEditorDialog`, `KeyringUnavailableBanner`, `ModelEntryRow` (in `src/lib/components/providers/`).
2. **Settings page modifications:** Provider cards gain kebab menus, API key status indicators, models list display. "Add provider" button wired to open dialog. KeyringUnavailableBanner added.
3. **AppState expansion:** `keyring_available` state, provider CRUD actions (`createProvider`, `updateProvider`, `deleteProvider`).
4. **api.ts expansion:** New invoke wrappers for provider + keyring commands.
5. **Type expansion:** `ProviderModelEntry` type, `models` array on Provider, `has_api_key` on Provider, `model_name` on Conversation.
6. **Backend schema change:** Migration #3 adds `models` JSON column to providers, `model_name` column to conversations.
7. **Startup keyring check:** `+layout.svelte` calls `checkKeyringAvailable` during hydration.

### What Does NOT Change

- No changes to design tokens (spacing, typography, color, radii)
- No changes to existing page routes
- No new npm dependencies
- No changes to the icon library
- No Toast component (error banners used as interim)
- No changes to ConversationSidebar, ChatHeader, MessageBubble, MessageInput
- No changes to existing navigation (Chat / Library / Settings)
- No changes to Dialog, Button, Input, Select, Tag, KebabMenu UI primitives
- No new CSS utilities
- No shadcn or third-party registry components

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS
- [ ] Dimension 2 Visuals: PASS
- [ ] Dimension 3 Color: PASS
- [ ] Dimension 4 Typography: PASS
- [ ] Dimension 5 Spacing: PASS
- [ ] Dimension 6 Registry Safety: PASS

**Approval:** pending

## UI-SPEC COMPLETE
