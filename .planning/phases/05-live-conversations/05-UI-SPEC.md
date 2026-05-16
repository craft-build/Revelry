---
phase: 5
slug: live-conversations
status: draft
shadcn_initialized: false
preset: none
created: 2026-05-15
---

# Phase 5 — UI Design Contract

> Visual and interaction contract for the Live Conversations phase. Real AI streaming replaces mock responses. Users can send messages and receive token-by-token AI responses, stop generation mid-stream, regenerate the last AI response, and edit/resend their own messages. Inline error messages appear in the chat when generation fails. All design tokens, typography, color, spacing, and radii are inherited from Phase 1's locked design system.

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

**Phase 5 additions:**

| Token               | Value         | Usage                                                           |
| ------------------- | ------------- | --------------------------------------------------------------- |
| `edit-area-pad`     | 12px (p-3)    | Padding for the inline edit textarea inside MessageBubble       |
| `edit-actions-gap`  | 8px (gap-2)   | Gap between Save & Resend / Cancel buttons in edit mode        |
| `error-bubble-pad`  | 16px (px-4 py-3) | Padding for inline error message bubbles (same as normal)  |
| `error-icon-gap`    | 8px (gap-2)   | Gap between AlertTriangle icon and error text                  |

All new spacing uses existing Tailwind token values. No custom spacing values introduced.

---

## Typography

**Inherited from Phase 1.** Consolidated to 4 sizes and 2 weights per design system rules.

**Type scale (4 sizes, 2 weights):**

| Role                                                       | Size        | Weight         | Line Height     | Font Family                             |
| ---------------------------------------------------------- | ----------- | -------------- | --------------- | --------------------------------------- |
| Body (chat bubbles, input, error messages)                 | 15px        | 400 (regular)  | 1.625 (relaxed) | Geist (sans)                            |
| Heading (speaker names, card titles, dialog title)         | 14px–18px\* | 600 (semibold) | 1.2             | Fraunces (serif/display) / Geist (sans) |
| Label / small (sidebar items, nav, metadata, field labels) | 12px (xs)   | 400 (regular)  | 1.25            | Geist (sans)                            |
| Textarea body / edit textarea                              | 14px (sm)   | 400 (regular)  | 1.625           | Geist (sans)                            |

\* Dialog title: 18px Geist (sans), weight 600. Card titles / speaker names: 14px Fraunces (serif/display), weight 600.

**Phase 5 role breakdown by element:**

| Element                        | Size | Weight | Font Family | Notes                                                        |
| ------------------------------ | ---- | ------ | ----------- | ------------------------------------------------------------ |
| Chat bubble text (user)        | 15px | 400    | Geist sans  | Normal message rendering                                     |
| Chat bubble text (AI)          | 15px | 400    | Geist sans  | Normal message rendering, with streaming caret               |
| Chat bubble text (error)       | 15px | 400    | Geist sans  | Inline error message — `--color-destructive` text           |
| Error label                    | 12px | 600    | Geist sans  | "Generation failed" speaker name area on error bubbles       |
| Edit textarea                  | 14px | 400    | Geist sans  | Inline edit textarea replacing bubble content                |
| Save & resend button label     | 12px | 400    | Geist sans  | "Save & resend" in edit mode action row                     |
| Cancel edit button label       | 12px | 400    | Geist sans  | "Cancel" in edit mode action row                            |
| Retry link (error bubble)      | 12px | 400    | Geist sans  | "Retry" or "Switch provider" links below error text         |
| Stop reason badge              | 10px | 500    | Geist sans  | "cancelled" / "length" stop reason on AI bubbles            |

**No 500 (medium) weight in Phase 5 declarations.** The stop reason badge uses inherited Tag component styling (Phase 3). All Phase 5 body text, error text, and labels use 400. All Phase 5 emphasis (error label, headings) uses 600.

---

## Color

All colors defined as OKLCH values in `src/app.css` @theme block. Dark theme only — light theme deferred to Phase 10.

| Role                | OKLCH Value             | Tailwind Token             | Usage                                                   |
| ------------------- | ----------------------- | -------------------------- | ------------------------------------------------------- |
| **Dominant (60%)**  | `oklch(0.16 0.005 270)` | `--color-background`       | App background, page-level surface                      |
| **Secondary (30%)** | `oklch(0.19 0.006 270)` | `--color-surface`          | Sidebar, header, input area backgrounds                 |
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
| **Destructive**     | `oklch(0.66 0.18 25)`   | `--color-destructive`      | Stop-generation button, delete actions, generation errors |
| **Warning**         | `oklch(0.78 0.14 80)`   | `--color-warning`          | Reserved for future                                    |
| **Success**         | `oklch(0.72 0.15 155)`  | `--color-success`          | Reserved for future                                    |
| **Bubble user**     | `oklch(0.26 0.02 220)`  | `--color-bubble-user`      | User message bubble background                          |
| **Bubble character**| `oklch(0.21 0.007 270)` | `--color-bubble-character` | Character message bubble background                     |

**Accent reserved for (explicit list):**

- Send button background
- New chat button (primary variant)
- Focus ring (`outline: 2px solid var(--color-accent)`)
- Context utilization progress bar fill
- Input border on focus (`focus-within:border-accent`)
- App identity gradient (accent → accent-muted)
- Streaming caret color (`.streaming-caret::after`)
- Selection highlight (via accent-muted)
- Step active fill / connector completed (Phase 3)
- Saving spinner (Phase 3)
- "Save provider" button (Phase 4)
- Provider type group selected indicator (Phase 4)

**Phase 5 semantic color usages (no new tokens):**

| State                                  | Token Used                | Usage                                                          |
| -------------------------------------- | ------------------------- | -------------------------------------------------------------- |
| Error bubble background                | `--color-destructive/10`  | 10% opacity destructive background for inline error messages   |
| Error bubble border                    | `--color-destructive/20`  | 20% opacity destructive border                                |
| Error bubble icon                      | `--color-destructive`     | `AlertTriangle` icon tint in error message                     |
| Error bubble text                      | `--color-destructive`     | Error description text                                         |
| Error bubble link (Retry)              | `--color-accent`          | "Retry" action link below error text                          |
| Error bubble link (Switch provider)    | `--color-accent`          | "Switch provider" action link below error text                |
| Stop reason "cancelled" badge          | `--color-warning/20` bg   | "cancelled" stop reason on AI messages                        |
| Stop reason "length" badge             | `--color-warning/20` bg  | "length" stop reason on AI messages                           |
| Edit textarea border (active)          | `--color-accent`          | Border highlight when edit textarea is focused                 |
| Save & resend button                   | `--color-accent` bg       | Primary action in edit mode (accent background)               |
| Cancel edit button                     | `--color-subtle`          | Ghost/outline style cancel in edit mode                       |
| Edit action icon (hover)               | `--color-foreground`      | Pencil icon on hover (same as existing hover action pattern)  |
| Regenerate action icon (hover)         | `--color-foreground`      | Refresh icon on hover (same as existing hover action pattern) |

No new color tokens are added.

---

## Radii

**Inherited from Phase 1.** No changes.

| Token         | Value          | Usage                                              |
| ------------- | -------------- | -------------------------------------------------- |
| `--radius-sm` | 0.375rem (6px) | Small buttons, icon buttons, action chips          |
| `--radius-md` | 0.5rem (8px)   | Default buttons, inputs, edit textarea             |
| `--radius-lg` | 0.75rem (12px) | Chat bubbles, error bubbles, edit textarea frame   |
| `--radius-xl` | 1rem (16px)    | Dialog panels, modal containers                   |

**Phase 5 note:** The inline edit textarea uses `--radius-lg` to match the chat bubble shape it replaces. Error message bubbles use `--radius-lg` consistent with regular chat bubbles. Stop reason badges use `--radius-sm`.

---

## Component Inventory

### New UI Primitives (`src/lib/components/ui/`)

No new UI primitives required. All Phase 5 UI is built from existing components.

### New Domain Components (`src/lib/components/`)

No new domain components. All changes are modifications to existing components.

### Modified Existing Components

| Component                                | Change                     | What Changes                                                                                                                                                                                                                                                                                            |
| ---------------------------------------- | -------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `MessageBubble.svelte`                   | Edit mode + error + regen  | (1) Edit hover action now functional: click enters inline edit mode — bubble content becomes textarea, action row shows "Save & resend" / "Cancel". (2) Error message rendering: when `metadata.stop_reason === "error"`, bubble uses destructive-tinted style with AlertTriangle icon, retry/provider links. (3) Regenerate hover action: only visible on the **last AI message** in the conversation; click triggers regeneration. (4) Edit hover action: only visible on **user messages**. (5) Stop reason badge on AI messages when `metadata.stop_reason` is "cancelled" or "length". |
| `chat/[id]/+page.svelte`                 | Replace mock streaming     | Remove `handleSend` mock streaming setTimeout loop. Replace with real Tauri Channel listener: `handleSend` calls `app.startChat()`, `handleCancel` calls `app.stopGeneration()`. Auto-scroll remains. No provider check — handled by AppState.                                                            |
| `AppState` (`app.svelte.ts`)             | New actions + streaming    | Add `startChat(content: string)` action that invokes `start_chat`, receives Channel, listens for StreamChunk events (Token/Complete/Error), calls appendMessage/updateStreamingMessage accordingly. Add `stopGeneration()`, `regenerate()`, `editResend(messageId, newContent)` actions.             |
| `api.ts`                                 | New invoke wrappers        | Add: `startChat`, `stopGeneration`, `regenerate`, `editAndResend` wrappers with Tauri Channel support.                                                                                                                                                                                                  |
| `types/index.ts`                         | Updated types              | Add `StreamChunk` type for frontend channel handling: `{ type: "token" | "complete" | "error", content?: string, message_id?: string, error?: string }`. May add `is_error` flag to MessageMetadata.                                                                                                  |
| `MessageInput.svelte`                    | Minimal change             | Stop button behavior confirmed: already wired to `oncancel`. With real streaming, `handleCancel` in the page calls `app.stopGeneration()` which invokes `stop_generation`. No visual changes needed.                                                                                                 |
| `ChatHeader.svelte`                      | No visual changes          | ProviderModelSelector already wired. No Phase 5 changes to this component.                                                                                                                                                                                                                             |

---

## Loading States

### Streaming State Design Contract

This is the core state machine for Phase 5. The UI transitions through these states during a real AI generation:

| State        | Trigger                           | `is_generating` | Visual Indicators                                                                                                  |
| ------------ | --------------------------------- | ---------------- | ------------------------------------------------------------------------------------------------------------------ |
| **Idle**     | No active generation              | `false`          | Send button visible. Input enabled. No streaming caret.                                                            |
| **Sending**  | User submits message              | `true`           | Send button → Stop button. Input disabled (prevent double-send). User message rendered. AI placeholder bubble with streaming caret. |
| **Streaming**| First `StreamChunk.Token` arrives | `true`           | Stop button visible. AI bubble content updates token-by-token. Streaming caret `▍` at end. Auto-scroll active.     |
| **Complete** | `StreamChunk.Complete` received   | `false`          | Send button restored. Input re-enabled. Streaming caret removed. AI message saved to DB. Stop reason badge shown if not "end". |
| **Error**    | `StreamChunk.Error` received      | `false`          | Send button restored. Input re-enabled. Error bubble rendered inline. No streaming caret.                          |
| **Stopped**  | User clicks Stop / `stop_generation` invoked | `false` | Send button restored. Input re-enabled. AI message finalized with current partial content. `stop_reason: "cancelled"` badge shown. |

**Transition details:**

1. **Idle → Sending:** User presses Enter or clicks Send. `handleSend(text)` in `chat/[id]/+page.svelte` calls `app.startChat(text)`. Inside `startChat`:
   - User message is created and appended to `messages` (and saved to DB per D-04).
   - AI placeholder message is created with `is_streaming: true`, empty content, appended to `messages`.
   - `app.is_generating` set to `true`.
   - `invoke('start_chat', ...)` called, Channel listener begins.

2. **Sending → Streaming:** First `StreamChunk.Token` arrives via Channel. `updateStreamingMessage(aiId, token)` called. Bubble content updates.

3. **Streaming → Complete:** `StreamChunk.Complete` arrives. `updateStreamingMessage(aiId, fullContent, done=true)`. `is_generating` set to `false`. AI message saved to DB. Streaming caret removed.

4. **Streaming → Error:** `StreamChunk.Error` arrives. Error message created as a Message with `metadata.stop_reason: "error"`, replacing or appended after the AI placeholder. `is_generating` set to `false`.

5. **Streaming → Stopped:** User clicks Stop. `app.stopGeneration()` calls `invoke('stop_generation')`. Backend drops CancellationSender. Partial AI response is finalized. `stop_reason: "cancelled"` set.

**MessageInput state transitions:**

| `is_generating` | Send Button   | Stop Button | Input State  |
| ---------------- | ------------- | ----------- | ------------ |
| `false`          | Visible       | Hidden      | Enabled      |
| `true`           | Hidden        | Visible     | Disabled     |

The input textarea is **disabled** during generation to prevent double-send. Text is cleared immediately on send (as current behavior). If the user needs to queue a follow-up, they must wait for generation to complete or stop it first.

### Edit Mode State

When the user clicks the Edit hover action on a user message:

| State           | Visual                                                                                                    |
| --------------- | --------------------------------------------------------------------------------------------------------- |
| **Viewing**     | Normal bubble rendering. Hover actions visible (Copy, Edit, Delete).                                      |
| **Editing**     | Bubble content replaced by `<textarea>` pre-filled with message content. Border accent on focus. Normal hover actions hidden; replaced by "Save & resend" (primary) and "Cancel" (ghost) buttons. |
| **Saving**      | "Save & resend" button shows `Loader2` spinner + label changes to "Saving…". Cancel button disabled. Textarea disabled. |
| **Saved**       | Edit mode exits. Truncated messages removed from UI. New AI stream begins. Same visual as Sending state.  |

### Regenerate Loading State

When the user clicks Regenerate on the last AI message:

| State           | Visual                                                                                                         |
| --------------- | -------------------------------------------------------------------------------------------------------------- |
| **Idle**        | Normal bubble rendering. Regenerate hover action visible (last AI message only).                              |
| **Regenerating**| Old AI message removed from UI. New AI placeholder with streaming caret appears in its place. Stop button shown. Same visual as Sending/Streaming states. |

---

## Empty States

No new empty states in Phase 5. The existing empty state from `chat/[id]/+page.svelte` ("No messages yet. Send something to begin.") remains unchanged.

**Error state replaces empty state conceptually** — when a generation fails, the inline error message fills the role of "something went wrong, here's what to do next."

---

## Error States

### Inline Generation Errors (NEW — Critical Phase 5 Pattern)

When AI generation fails (context overflow, auth failure, network error, rate limit, etc.), an **inline error message** appears in the chat flow. This follows D-07: errors are shown as inline messages, not toasts or modals.

**Error message model:**

The error is rendered as a `Message` object with:
- `role: "assistant"` (it's a response to the user's message)
- `content: "{error description}"` (the specific error text)
- `metadata.stop_reason: "error"`
- `metadata.error_type`: category for programmatic use (e.g., "context_length_exceeded", "authentication_failed", "network_error", "rate_limited", "provider_error")
- `is_streaming: false`

**Error bubble visual contract:**

```
┌────────────────────────────────────────────────────────┐
│ ⚠ Generation failed                                    │ ← speaker name area
│ ┌────────────────────────────────────────────────────┐ │
│ │ ▲ Context length exceeded. The conversation is    │ │ ← destructive-tinted bubble
│ │   too long for this model's context window.       │ │
│ │                                                    │ │
│ │   Retry  ·  Switch provider                        │ │ ← action links
│ └────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────┘
```

**Styling:**

- Bubble background: `bg-(--color-destructive)/10`
- Bubble border: `border border-(--color-destructive)/20`
- Bubble text: `text-[15px] text-(--color-destructive) leading-relaxed`
- AlertTriangle icon: `size-4 text-(--color-destructive)` inline before text
- Speaker name: "Generation failed" (12px, weight 600, `text-(--color-destructive)`)
- Avatar: Hidden for error messages — no avatar column. Error message spans full width for visual distinction.
- Action links: `text-xs text-(--color-accent) hover:underline`, separated by `·`
  - "Retry" — calls `app.regenerate()` (resends with same context)
  - "Switch provider" — navigates focus to ProviderModelSelector in ChatHeader

**Error type to display copy mapping:**

| Error Type                     | Error Bubble Heading | Error Bubble Body                                                                               | Actions                  |
| ------------------------------ | --------------------- | ----------------------------------------------------------------------------------------------- | ------------------------ |
| `context_length_exceeded`      | Generation failed     | Context length exceeded. The conversation is too long for this model's context window.          | Retry · Switch provider  |
| `authentication_failed`        | Generation failed     | Authentication failed. Check that your API key is correct in Settings.                          | Retry · Switch provider  |
| `network_error`                | Generation failed     | Network error. Check your internet connection and try again.                                    | Retry                    |
| `rate_limited`                 | Generation failed     | Rate limited. You've hit the provider's usage limit. Wait a moment and try again.               | Retry · Switch provider  |
| `provider_error` (generic)     | Generation failed     | The provider returned an error: {error_detail}.                                                | Retry · Switch provider  |
| `no_provider_configured`       | Generation failed     | No provider configured. Select a provider and model from the header.                            | Switch provider          |
| `no_api_key`                   | Generation failed     | No API key set for this provider. Add your key in Settings.                                     | Switch provider          |

**Error bubble is NOT dismissible** — it remains in the chat flow until the user takes action (retry, switch provider, or sends a new message which implicitly moves past it).

**Error bubble vs. AI bubble with stop_reason "cancelled"/"length":**

- **Error bubble**: `stop_reason === "error"`. Destructive styling. Shows retry links.
- **AI bubble with stop reason**: `stop_reason === "cancelled" | "length"`. Normal bubble styling. Shows a small badge below the bubble indicating why generation stopped. No retry links (the content is valid, just incomplete).

### Stop Reason Badges

On AI messages where `metadata.stop_reason` is not `"end"`:

| Stop Reason   | Badge Text     | Badge Style                                                                |
| ------------- | -------------- | -------------------------------------------------------------------------- |
| `cancelled`   | "Stopped"      | `bg-(--color-warning)/20 text-(--color-warning) text-[10px] font-medium rounded-(--radius-sm) px-1.5 py-0.5` |
| `length`      | "Max tokens"   | `bg-(--color-warning)/20 text-(--color-warning) text-[10px] font-medium rounded-(--radius-sm) px-1.5 py-0.5` |
| `end`         | (no badge)     | Normal completion, no badge shown                                          |
| `error`       | (error bubble) | See error bubble styling above — not a badge, full error rendering        |

**Badge position:** Below the AI bubble, before the hover actions row. Only visible when `stop_reason !== "end"` and `stop_reason !== undefined`.

---

## Interaction Patterns

### New Patterns in Phase 5

| Pattern                    | Trigger                                         | Behavior                                                                                                                                                   | Notes                                                                                         |
| -------------------------- | ----------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| **Send message (real)**     | Enter / Ctrl+Enter in MessageInput              | User message created and saved to DB immediately (D-04). AI placeholder created in memory. `invoke('start_chat')` called with conversation_id + content. Channel listener begins. | Replaces mock streaming setTimeout loop. Input cleared and disabled during generation.        |
| **Stop generation**        | Click Stop button in MessageInput               | `invoke('stop_generation')` called. Backend drops CancellationSender, stream stops. Partial AI response finalized with `stop_reason: "cancelled"`. `is_generating = false`. | Stop button already exists in MessageInput — behavior confirmed.                              |
| **Regenerate last AI msg** | Click Regenerate icon on last AI MessageBubble  | Last AI message deleted from UI and DB. New AI placeholder created at same sort_order. `invoke('regenerate')` called. New stream begins with same provider/model (D-11, D-13). | Regenerate icon only visible on the **last** AI message. Conditional rendering based on message index + role. |
| **Edit user message**      | Click Edit (Pencil) icon on user MessageBubble  | MessageBubble enters edit mode: bubble content replaced by textarea pre-filled with message text. Hover actions hidden. "Save & resend" + "Cancel" buttons shown below textarea. | Only on user messages (D-10). Edit icon hidden on AI messages.                                |
| **Save & resend**          | Click "Save & resend" in edit mode              | Calls `app.editResend(messageId, newContent)`. All messages after the edited message are deleted from UI and DB (D-08). New AI placeholder created. Stream begins from edited point. | Truncation includes the old AI response to the edited message and any subsequent exchanges.   |
| **Cancel edit**            | Click "Cancel" or press Escape in edit mode     | Edit mode exits. Original message text restored. No changes made.                                                                                          | Escape key also cancels edit.                                                                 |
| **Retry after error**      | Click "Retry" link in error bubble              | Calls `app.regenerate()`. Same as Regenerate — deletes error message, creates new AI placeholder, starts new stream.                                      | Uses same provider/model.                                                                     |
| **Switch provider (error)** | Click "Switch provider" link in error bubble    | Opens ProviderModelSelector popover in ChatHeader (via DOM focus/click). Error message remains in chat until user sends again.                             | Focus the selector, let user pick. They then manually retry (send new message or regenerate). |
| **Channel listener loop**  | `StreamChunk` events from Tauri Channel        | On `Token`: append to AI message content, update UI. On `Complete`: finalize message, save to DB, set `is_generating = false`. On `Error`: render inline error bubble. | Runs inside `app.startChat()` action.                                                         |

### Conditional Hover Action Visibility

The hover actions on MessageBubble change based on message role and position:

| Action      | User Message | AI Message (last) | AI Message (not last) | Error Message |
| ----------- | ------------- | ----------------- | --------------------- | ------------- |
| Copy        | ✅            | ✅                | ✅                    | ✅            |
| Edit        | ✅            | —                 | —                     | —             |
| Regenerate  | —             | ✅                 | —                     | —             |
| Delete      | ✅            | ✅                 | ✅                    | —             |

**"Last AI message"** is determined by: message is the most recent message in `app.messages` where `role === "assistant"` and `stop_reason !== "error"`.

**Error messages** show Copy only (no Edit, Regenerate, or Delete). Error messages are replaced when the user retries or regenerates.

### Edit Mode Detail Contract

When edit mode is active on a user MessageBubble:

1. **Bubble transformation:** The `<p>` content area is replaced by a `<textarea>`:
   - `class="w-full resize-none bg-(--color-surface-input) text-[14px] leading-relaxed text-(--color-foreground) p-3 rounded-(--radius-md) border border-(--color-border) focus:border-(--color-accent) focus:outline-none"`
   - Pre-filled with `message.content`
   - Auto-grow behavior (same as MessageInput textarea, max 240px)
   - On focus: accent border highlight

2. **Action row transformation:** Normal hover actions (Copy, Edit, Delete) are hidden. Replaced by:
   - "Save & resend" button: `variant="primary" size="sm"` — accent background
   - "Cancel" button: `variant="ghost" size="sm"` — subtle text

3. **Keyboard shortcuts in edit mode:**
   - Escape: Cancel edit (restores original)
   - Ctrl/Cmd+Enter: Save & resend (same convention as message sending)

4. **No confirmation dialog for edit/resend:** The destructive nature (truncating subsequent messages) is communicated by the "Save & resend" label itself — the word "resend" implies the conversation continues from this point. Per D-08, truncation is the expected behavior. No "This will delete N messages" confirmation — the pattern matches ChatGPT and SillyTavern.

### Deferred Interactions (Not in Phase 5)

| Pattern                         | Deferred To | Reason                                                     |
| ------------------------------- | ----------- | ---------------------------------------------------------- |
| Message branching / forking     | Phase 8     | Branch navigation UI is a separate phase                   |
| Full-text search across messages| Phase 8     | FTS5 integration                                           |
| Per-conversation gen param overrides | Phase 6 | Only provider-level defaults in Phase 5                   |
| Preset selection in chat         | Phase 6     | Preset picker deferred to Smart Prompts                   |
| Markdown rendering              | Phase 7     | Current naive italic-asterisk rendering continues          |
| Persona in system prompt        | Phase 6     | Minimal prompt in Phase 5 (D-05)                           |
| Lorebook in prompt              | Phase 9     | Not yet implemented                                        |
| Delete message (functional)     | Phase 8     | Delete button exists but behavior TBD with branching       |

---

## Copywriting Contract

### CTAs and Button Labels

| Element                           | Copy                | Context                                          |
| --------------------------------- | ------------------- | ------------------------------------------------ |
| Stop generation button            | Square icon only    | In MessageInput, `aria-label="Stop generation"`  |
| Send button                       | Send icon only      | In MessageInput, `aria-label="Send"`             |
| Edit hover action                 | Pencil icon only    | `aria-label="Edit"`                              |
| Regenerate hover action           | Refresh icon only   | `aria-label="Regenerate"`                        |
| Copy hover action                 | Copy icon only      | `aria-label="Copy"`                              |
| Delete hover action               | Trash icon only     | `aria-label="Delete"`                            |
| Save & resend (edit mode)        | "Save & resend"    | Primary button in edit action row                |
| Cancel (edit mode)                | "Cancel"            | Ghost button in edit action row                  |
| Saving… (edit mode)               | "Saving…"           | During edit/resend async operation               |
| Retry link (error bubble)         | "Retry"             | Accent-colored link in error bubble              |
| Switch provider link (error bubble)| "Switch provider"  | Accent-colored link in error bubble              |

### Empty States

| Element                    | Copy                                  | Context                  |
| -------------------------- | ------------------------------------- | ------------------------ |
| No messages in conversation| "No messages yet. Send something to begin." | Existing — unchanged |

### Error States

| Element                              | Copy                                                                      | Context                           |
| ------------------------------------ | ------------------------------------------------------------------------- | --------------------------------- |
| Error bubble heading                 | "Generation failed"                                                       | All generation error types        |
| Context length exceeded body         | "Context length exceeded. The conversation is too long for this model's context window." | `context_length_exceeded`     |
| Authentication failed body           | "Authentication failed. Check that your API key is correct in Settings."   | `authentication_failed`           |
| Network error body                   | "Network error. Check your internet connection and try again."            | `network_error`                   |
| Rate limited body                    | "Rate limited. You've hit the provider's usage limit. Wait a moment and try again." | `rate_limited`           |
| Provider error body                  | "The provider returned an error: {error_detail}."                         | `provider_error` (generic)        |
| No provider configured body          | "No provider configured. Select a provider and model from the header."    | `no_provider_configured`          |
| No API key body                      | "No API key set for this provider. Add your key in Settings."             | `no_api_key`                      |
| Stop reason badge "cancelled"        | "Stopped"                                                                 | User stopped generation            |
| Stop reason badge "length"           | "Max tokens"                                                              | Model hit max_tokens limit         |

### Destructive Actions

| Action           | Confirmation | Rationale                                                                                              |
| ---------------- | ------------ | ------------------------------------------------------------------------------------------------------ |
| Edit & resend    | None         | Truncates subsequent messages (D-08). "Save & resend" label communicates the destructive nature.      |
| Regenerate       | None         | Replaces last AI message in-place (D-11). Non-destructive from user perspective — old response replaced. |
| Stop generation  | None         | Immediately stops stream. Partial response preserved. User expectation is immediate stop.              |

No confirmation dialogs in Phase 5. All destructive actions are self-explanatory from their labels.

---

## Custom Utilities

**No new CSS utilities required.** Existing utilities from Phase 1 (`font-display`, `surface-grain`, `streaming-caret`, `drag-region`, `no-drag`) are sufficient.

Phase 5 uses Tailwind v4 built-in utilities for:

- `animate-spin` — on `Loader2` icon during edit/resend save
- `opacity` transitions — hover action reveal (existing pattern)
- `resize-none` — edit textarea disables manual resize
- `max-h-[240px]` — edit textarea auto-grow max height (same as MessageInput)

---

## Registry Safety

**No changes.** No shadcn, no third-party registries. All components remain custom-built using Bits.ui + Tailwind v4.

| Registry        | Blocks Used | Safety Gate    |
| --------------- | ----------- | -------------- |
| shadcn official | none        | not applicable |
| Third-party     | none        | not applicable |

---

## Phase 5 Specifics

### Discretionary Decisions Made

The following decisions were in "Claude's Discretion" in CONTEXT.md and are hereby declared:

1. **Inline error message rendering — distinct bubble styling within MessageBubble, NOT a separate component:** Error messages use the existing MessageBubble component with conditional rendering. When `metadata.stop_reason === "error"`, the bubble renders with destructive-tinted styling (red-10 background, red-20 border, AlertTriangle icon, retry links). No avatar column is shown for error messages — they span the full content width for visual distinction. The error is a `Message` object with `role: "assistant"` and `metadata.stop_reason: "error"`, not a separate `role: "system"` variant. This approach:
   - Keeps errors in the conversation flow (per D-07)
   - Avoids a new component (ErrorBubble) that would need its own rendering path
   - Uses the existing MessageBubble rendering infrastructure with conditional branches
   - Makes error handling transparent to the message list (it's just another message in `app.messages`)

2. **Edit mode — inline textarea overlay replacing bubble content:** When the user clicks Edit, the `<p>` content area within MessageBubble transforms into a `<textarea>` pre-filled with the message content. The bubble's outer container (border, padding, radius) remains — only the inner content changes from rendered text to editable textarea. The hover action row switches from the normal icon-only actions to text buttons: "Save & resend" (primary) and "Cancel" (ghost). This approach:
   - Preserves spatial context (the user sees exactly where the message was)
   - Follows ChatGPT/SillyTavern conventions (familiar UX)
   - Avoids a separate EditMessage component that would need position tracking
   - The textarea inherits the bubble's visual container — same border-radius, similar padding

3. **Generation params resolution — provider-level defaults only:** When `start_chat` is called, the backend reads the provider's `default_params` (temperature, top_p, max_tokens, etc.) from the `ProviderConfig`. No per-conversation overrides or preset selection in Phase 5. The conversation's `generation_preset_id` field exists but is not wired. This matches D-05 (minimal prompt) and keeps Phase 5 focused on streaming infrastructure.

4. **Error messages excluded from hover action patterns:** Error bubbles (where `stop_reason === "error"`) only show a Copy action in their hover row. No Edit, Regenerate, or Delete actions on errors. Error messages are replaced by the Retry action (which calls regenerate, deleting the error message and streaming a new response). This prevents confusion — editing or deleting an error message doesn't make sense, and regeneration is handled by the explicit "Retry" link.

5. **Stop reason badges — small inline indicators below AI bubbles:** When an AI message finishes with `stop_reason` of "cancelled" or "length", a small badge appears below the bubble text (inside the bubble, before the hover actions row). The badge uses warning-tinted styling (yellow-20 background, yellow text) at 10px size. This is subtle enough not to disrupt conversation flow but informative enough to indicate why generation stopped.

6. **No provider/model badge change on AI bubbles during streaming:** The existing model name badge (shown in the speaker name area of AI MessageBubbles) remains unchanged. During streaming, the badge still shows the model name from `metadata.model`. No new "streaming…" indicator beyond the caret animation.

7. **Input disabled during generation:** The MessageInput textarea is `disabled` while `app.is_generating === true`. This prevents double-send. Text is cleared immediately on send (existing behavior). Users who want to type a follow-up while waiting must stop generation first, then type. This matches the behavior of most AI chat interfaces.

### What Changes

1. **MessageBubble.svelte — major modification:** Add conditional rendering for (a) edit mode (textarea overlay), (b) error message styling, (c) regenerate action visibility (last AI message only), (d) edit action visibility (user messages only), (e) stop reason badges.
2. **chat/[id]/+page.svelte — replace mock streaming:** Remove `handleSend` mock setTimeout loop. Replace with `app.startChat(text)`. Replace `handleCancel` with `app.stopGeneration()`. Add `handleRegenerate()` and `handleEditResend(messageId, newContent)`.
3. **AppState — new actions:** Add `startChat`, `stopGeneration`, `regenerate`, `editResend` actions. Add `StreamChunk` event handling logic.
4. **api.ts — new invoke wrappers:** Add `startChat`, `stopGeneration`, `regenerate`, `editAndResend`.
5. **types/index.ts — updated types:** Add `StreamChunk` type. Add `error_type` to `MessageMetadata`. Add `is_error` flag to `MessageMetadata`.

### What Does NOT Change

- No changes to design tokens (spacing, typography, color, radii)
- No changes to existing page routes (only chat/[id] content changes)
- No new npm dependencies (Tauri Channel API is in @tauri-apps/api)
- No changes to icon library (all needed icons already exist: Pencil, RotateCcw, Copy, Trash2, AlertTriangle, Square, Send)
- No Toast component (inline error messages serve the error display role)
- No changes to ConversationSidebar, AppHeader, PersonaSwitcher
- No changes to Dialog, Button, Input, Select, Tag, KebabMenu UI primitives
- No new CSS utilities
- No shadcn or third-party registry components
- No changes to ChatHeader visual (ProviderModelSelector already wired)
- No changes to MessageInput visual (stop button already exists)
- No changes to the welcome page, characters page, or settings page

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS
- [ ] Dimension 2 Visuals: PASS
- [ ] Dimension 3 Color: PASS
- [ ] Dimension 4 Typography: PASS
- [ ] Dimension 5 Spacing: PASS
- [ ] Dimension 6 Registry Safety: PASS

**Approval:** pending
