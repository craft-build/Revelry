---
phase: 2
slug: persistent-data
status: approved
reviewed_at: 2026-05-11
shadcn_initialized: false
preset: none
created: 2026-05-11
---

# Phase 2 — UI Design Contract

> Visual and interaction contract for the Persistent Data phase. This phase adds NO new UI components — it replaces mock data with real SQLite data and introduces loading, empty, and error states that didn't exist with mock data. All design tokens, typography, color, spacing, radii, and component inventory are inherited from Phase 1's locked design system (`.planning/phases/01-native-desktop-shell/01-UI-SPEC.md`).

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

**Reference:** `.planning/phases/01-native-desktop-shell/01-UI-SPEC.md` § Design System, § Spacing Scale, § Typography, § Color, § Radii, § Component Inventory — all unchanged.

---

## Spacing Scale

**Inherited from Phase 1.** No changes.

See Phase 1 UI-SPEC § Spacing Scale for the full 4px-multiple scale (4, 8, 12, 16, 20, 24, 32, 40, 48, 64).

Exceptions: none.

---

## Typography

**Inherited from Phase 1.** No changes.

| Role | Size | Weight | Line Height | Font Family |
|------|------|--------|-------------|-------------|
| Body | 15px | 400 (regular) | 1.625 (relaxed) | Geist (sans) |
| Label / small | 12px (xs) | 500 (medium) | 1.25 | Geist (sans) |
| Heading | 14px (sm) | 600 (semibold) | 1.2 | Fraunces (serif/display) |
| Context / micro | 10px | 400 (regular) | 1.0 | Geist (sans) |

---

## Color

**Inherited from Phase 1.** No changes. All OKLCH values unchanged.

Phase 2 introduces **semantic usage** of existing tokens for new states:

| State | Token Used | Usage |
|-------|------------|-------|
| Loading skeleton | `--color-surface-elevated` | Skeleton pulse animation background |
| Loading skeleton shimmer | `--color-surface-hover` | Shimmer highlight sweep |
| Empty state icon | `--color-subtle` | Large center icon in empty states |
| Empty state heading | `--color-foreground` | Primary text in empty state |
| Empty state body | `--color-muted` | Secondary text in empty state |
| Error state icon | `--color-destructive` | Alert icon in error states |
| Error state heading | `--color-foreground` | Primary text in error state |
| Error state body | `--color-muted` | Secondary text / recovery instructions |

No new color tokens are added. The destructive and warning tokens from Phase 1 (reserved but unused) are now activated for error states.

---

## Radii

**Inherited from Phase 1.** No changes.

---

## Component Inventory

**No new components added.** Existing components gain new states (loading, empty, error) but their visual structure does not change.

### Components Modified in Phase 2

| Component | Change Type | What Changes |
|-----------|-------------|--------------|
| `ConversationSidebar` | Add loading state | Show skeleton conversation items during initial data fetch |
| `ConversationSidebar` | Empty state update | "No conversations yet." stays; becomes the real empty-DB state instead of just the mock-absent state |
| `+layout.svelte` | Add loading gate | Full-app loading indicator during initial `invoke()` hydration |
| `+page.svelte` (home) | Add loading/empty states | Suggested characters section shows skeletons while loading, shows empty state when DB has no characters |
| `characters/+page.svelte` | Add loading/empty states | Character grid shows skeletons while loading; empty state differentiates "no characters exist" from "no search match" |
| `characters/+page.svelte` | Persona tab empty state | "No personas yet." becomes real empty-DB state |

---

## Loading States

Phase 2 introduces real async data fetching via Tauri `invoke()`. During the initial hydration (app startup), all data views show a **skeleton/placeholder** pattern — not a spinner. This aligns with the content-first philosophy established in Phase 1.

### App Startup Loading Flow

```
App Launch
  │
  ├─ +layout.svelte onMount
  │   ├─ app.is_loading = true
  │   ├─ invoke('get_characters'), invoke('get_personas'), invoke('get_conversations')
  │   ├─ app.hydrate(results)
  │   └─ app.is_loading = false
  │
  └─ While is_loading === true:
      ├─ ConversationSidebar: skeleton conversation list
      ├─ Home page: skeleton character suggestions
      ├─ Library page: skeleton card grid
      └─ Chat page: skeleton message list (if conversation active)
```

### Skeleton Design Contract

**Pattern:** Pulse animation on placeholder shapes matching the real content layout. Not shimmer (complex), not spinner (content-agnostic). The skeleton should **occupy the same space** as the real content to prevent layout shift.

| Skeleton Element | Shape | Dimensions | Animation |
|------------------|-------|------------|-----------|
| Conversation item (sidebar) | Rounded rect | `w-full h-14` | `animate-pulse` on `--color-surface-elevated` bg |
| Character card | Rounded rect | `w-full aspect-[3/4]` | `animate-pulse` on `--color-surface-elevated` bg |
| Persona card | Rounded rect | `w-full aspect-[3/4]` | `animate-pulse` on `--color-surface-elevated` bg |
| Message bubble | Rounded rect | `w-3/4 h-12` (character), `w-2/3 h-10` (user) | `animate-pulse` on `--color-surface-elevated` bg |
| Chat header | Rounded rect | `w-48 h-4` | `animate-pulse` on `--color-surface-elevated` bg |

**Skeleton count:**
- Sidebar: 5 skeleton conversation items (matches typical list density)
- Character grid: 8 skeleton cards (matches 4-col grid at xl)
- Persona grid: 8 skeleton cards
- Messages: 4 skeleton bubbles (alternating user/character)

**Skeleton styling:**
```
bg-(--color-surface-elevated) rounded-(--radius-lg) animate-pulse
```

No new CSS utilities needed — `animate-pulse` is a Tailwind v4 built-in.

### Loading State Duration Target

With local SQLite, initial data fetch should complete in **< 200ms** for a typical database. The skeleton state is primarily for:
1. First launch (schema creation + migrations)
2. Very large databases (thousands of conversations)
3. Disk I/O latency on slow drives

The skeleton should feel like a brief flicker, not a sustained wait state.

---

## Empty States

With mock data removed, the app starts with an empty database. Empty states must guide the user toward their first action. Phase 1's empty states were incidental (no search matches) — Phase 2 makes them the **initial experience**.

### Empty State Design Pattern

All empty states follow the same visual structure:

```
┌──────────────────────────────────┐
│                                  │
│         [Icon 48px]              │  ← --color-subtle, @lucide/svelte icon
│                                  │
│        Heading text              │  ← text-sm font-medium --color-foreground
│                                  │
│   Description with next step.    │  ← text-xs --color-muted
│                                  │
│      [Primary CTA]              │  ← Button variant="primary" size="sm"
│                                  │
└──────────────────────────────────┘
```

**Layout:** `flex flex-col items-center justify-center gap-3` centered in the content area. Padding: `py-20` for vertical centering within scroll containers.

**Border:** `border border-dashed border-(--color-border)` for search-empty states (existing pattern). No border for first-run empty states (cleaner, less "error-like").

### Empty State Specifications

| View | Condition | Icon | Heading | Body | CTA |
|------|-----------|------|---------|------|-----|
| Conversation sidebar | No conversations in DB | `MessageSquareDashed` | "No conversations yet." | "Start a new chat with any character." | — (use "New chat" button already in sidebar header) |
| Conversation sidebar | Search with no matches | `MessageSquareDashed` | "No conversations match." | "Try a different search term." | — |
| Home page | No characters in DB | `Sparkles` | "Bring a character to life." | "Import a character card or create one from scratch in your library." | → "Browse library" link to `/characters` |
| Characters tab | No characters in DB | `Users` | "No characters yet." | "Import a character card or create one from scratch to get started." | "Import card" (primary), "New character" (secondary) |
| Characters tab | Search with no matches | `Search` | "No characters match." | "Try a different search term." | — |
| Personas tab | No personas in DB | `UserCircle` | "No personas yet." | "Create a persona to define how you appear in conversations." | "New persona" (primary) |
| Chat view | Empty conversation (no messages) | — | — | Character greeting message | — (first_mes from character card) |

**Important distinction:** The home page `+page.svelte` currently hardcodes suggested characters from `app.characters.slice(0, 3)`. When the DB is empty, this section should be replaced with the "Bring a character to life." empty state (which already exists in the markup but is currently unreachable because mock data always provides characters).

---

## Error States

Phase 2 introduces database initialization, which can fail. Error states are new to the app — they didn't exist with mock data because there was no backend to fail.

### Database Initialization Failure

**This is the critical error state.** If SQLite cannot initialize (corrupted file, permissions failure, disk full), the app cannot function. The UI must communicate this clearly and offer recovery.

**Design:** Full-screen error overlay (covers the entire app shell, not a dialog). The user cannot interact with the app until the database is available.

```
┌──────────────────────────────────────────────────┐
│  AppHeader (non-functional, dimmed)              │
├──────────────────────────────────────────────────┤
│                                                  │
│               [Database icon 48px]               │  ← Server or AlertTriangle
│                                                  │
│          "Couldn't open the database."           │  ← text-base font-display
│                                                  │
│    "Revelry stores your conversations and        │  ← text-sm --color-muted
│     characters locally. Something went           │
│     wrong reading the database file."            │
│                                                  │
│     "Database path: ~/Library/Application        │  ← text-xs font-mono --color-subtle
│      Support/com.revelry.desktop/revelry.db"     │
│                                                  │
│          [Retry]  [Show logs]                    │  ← Button primary, Button secondary
│                                                  │
└──────────────────────────────────────────────────┘
```

**Styling:**
- Background: `--color-background` (full app background, not a dialog overlay)
- Error icon: `--color-destructive`
- Heading: `text-base font-display font-semibold --color-foreground`
- Body: `text-sm --color-muted`, max-width `ch-60` for readability
- Path display: `text-xs font-mono --color-subtle` in a code-like container (`rounded-md bg-surface p-2`)
- Buttons: `Retry` = `variant="primary"`, `Show logs` = `variant="secondary"`

**Actions:**
- **Retry:** Re-attempts database initialization (calls the same Tauri command). If successful, removes the overlay and loads data. If still failing, updates error message (no change to visual structure).
- **Show logs:** Opens the Tauri log file or console. Implementation detail — for Phase 2, this could open DevTools console or `open` the log file path.

### Individual Command Failure

If a single `invoke()` call fails (e.g., `get_characters` returns an error but `get_personas` succeeds), the UI should:

1. **Show the data that loaded successfully** — don't block the entire app if one query fails.
2. **Replace the failed section with an inline error** — same visual pattern as empty state but with error icon and different copy.
3. **Offer retry** on the failed section.

```
┌──────────────────────────────────┐
│         [AlertTriangle 48px]     │  ← --color-destructive
│                                  │
│    "Couldn't load characters."   │  ← text-sm font-medium --color-foreground
│                                  │
│    "Try again in a moment."      │  ← text-xs --color-muted
│                                  │
│          [Retry]                 │  ← Button variant="secondary" size="sm"
│                                  │
└──────────────────────────────────┘
```

**Styling:** Same centered layout as empty state, but with `--color-destructive` icon instead of `--color-subtle`.

### Error State Specifications

| View | Condition | Icon | Heading | Body | CTA |
|------|-----------|------|---------|------|-----|
| Full app overlay | DB init failure | `AlertTriangle` | "Couldn't open the database." | "Revelry stores your conversations and characters locally. Something went wrong reading the database file." | Retry (primary), Show logs (secondary) |
| Character grid | `get_characters` failed | `AlertTriangle` | "Couldn't load characters." | "Try again in a moment." | Retry (secondary) |
| Persona grid | `get_personas` failed | `AlertTriangle` | "Couldn't load personas." | "Try again in a moment." | Retry (secondary) |
| Conversation sidebar | `get_conversations` failed | `AlertTriangle` | "Couldn't load conversations." | "Try again in a moment." | Retry (secondary) |
| Chat messages | `get_messages` failed | `AlertTriangle` | "Couldn't load messages." | "Try again in a moment." | Retry (secondary) |

---

## Interaction Patterns

### New Patterns in Phase 2

| Pattern | Trigger | Behavior |
|---------|---------|----------|
| **Initial data load** | App startup (`onMount`) | `is_loading = true` → `invoke()` parallel fetches → `hydrate()` → `is_loading = false`. UI shows skeletons during load. |
| **Skeleton transition** | `is_loading` changes | Skeletons fade out (150ms) as real content fades in. No abrupt layout shift. |
| **Empty library first-run** | User launches app for first time | All data views show empty states. Sidebar shows "No conversations yet." Home shows "Bring a character to life." Library shows "No characters yet." / "No personas yet." |
| **DB error overlay** | DB init `invoke()` returns error | Full-screen error overlay. App chrome visible but non-functional. User must Retry or quit. |
| **Partial load failure** | One `invoke()` fails among several | Failed section shows inline error with Retry button. Other sections display normally. |
| **Retry** | User clicks Retry button | Re-invokes the failed Tauri command. On success, replaces error with data. On failure, keeps error (no visual change). |
| **Delete conversation** | User clicks delete on conversation | Confirmation dialog → `invoke('delete_conversation')` → remove from local state → sidebar updates. Destructive action, see copywriting contract. |

### Existing Patterns (Modified)

| Pattern | Phase 1 Behavior | Phase 2 Change |
|---------|-------------------|----------------|
| **Hydrate** | `app.hydrate(mockData)` in `onMount` | `app.hydrate(invokeResults)` — data comes from `invoke()` calls, not imports |
| **Sidebar empty** | "No conversations yet." (always had mock data, so rarely visible) | Same copy, now the **real** initial state for first-run users |
| **Character grid empty** | "No characters match." (search-only) | Now also shows when DB is empty — differentiated copy |
| **Suggested characters (home)** | `app.characters.slice(0, 3)` — always 3 items | Same pattern, but may be 0 items → shows empty state instead |
| **Message list** | Loaded from mock `getMessages()` | Loaded from `invoke('get_messages', { conversation_id })` |

### Deferred Interactions (Not in Phase 2)

| Pattern | Deferred To | Reason |
|---------|-------------|--------|
| Create character (CHAR-02) | Phase 7 | Editors are a separate phase |
| Edit character (CHAR-03) | Phase 7 | Editors are a separate phase |
| Delete character (CHAR-04) | Phase 7 | Editors are a separate phase |
| Create persona (PERS-02) | Phase 7 | Editors are a separate phase |
| Edit persona (PERS-03) | Phase 7 | Editors are a separate phase |
| Import character card (CHAR-06) | Phase 6 | Card IO is a separate phase |
| FTS5 search (CONV-08) | Phase 8 | Full-text search UI is Phase 8 |
| Branching (CONV-06, 07) | Phase 8 | Tree structure is Phase 8 |

**Important:** The "New character", "Import card", "New persona", and "Edit" buttons on library cards **exist visually** in Phase 1 but are non-functional. In Phase 2 they remain non-functional. They should be visually present (not hidden) but may show a brief tooltip or disabled state. Do NOT remove them — they'll be wired in later phases.

---

## Copywriting Contract

Phase 2 introduces copy for empty states and error states that didn't exist with mock data. All existing copy from Phase 1 is carried forward unchanged.

### Empty States

| Element | Copy | Context |
|---------|------|---------|
| Sidebar — no conversations | "No conversations yet." | First-run; user hasn't started any chats |
| Sidebar — no search matches | "No conversations match." | Search filter returned zero results |
| Home — no characters | "Bring a character to life." | Hero heading (already exists, becomes reachable) |
| Home — no characters body | "Import a character card or create one from scratch in your library." | Replaces current "Pick a character to begin a conversation, or import a CharacterCardV2 from your library." — more actionable for empty DB |
| Library — no characters heading | "No characters yet." | Character tab empty state |
| Library — no characters body | "Import a character card or create one from scratch to get started." | Guides user to first action |
| Library — no characters search | "No characters match." | Search filter returned zero results |
| Library — no personas heading | "No personas yet." | Persona tab empty state |
| Library — no personas body | "Create a persona to define how you appear in conversations." | Guides user to first action |
| Chat — empty conversation | *(Character's `first_mes` rendered as a message bubble)* | Not a true "empty state" — the character's greeting message is the first content |

### Error States

| Element | Copy | Context |
|---------|------|---------|
| DB init failure heading | "Couldn't open the database." | Full-screen error overlay |
| DB init failure body | "Revelry stores your conversations and characters locally. Something went wrong reading the database file." | Explains what happened |
| DB init failure path | "Database path: {platform_path}" | Shows the actual file path for debugging |
| DB init failure retry | "Retry" | Re-attempts initialization |
| DB init failure logs | "Show logs" | Opens log output |
| Load failure heading | "Couldn't load {entity}." | Inline error (entity = characters / personas / conversations / messages) |
| Load failure body | "Try again in a moment." | Generic retry suggestion |
| Load failure retry | "Retry" | Re-invokes the failed command |

### Destructive Actions

| Action | Confirmation | Copy |
|--------|-------------|------|
| Delete conversation | Confirmation dialog | Heading: "Delete conversation?" · Body: "This will permanently remove this conversation and all its messages. This cannot be undone." · Confirm: "Delete" (destructive variant) · Cancel: "Cancel" |

**Delete conversation** is the only destructive action with a real backend in Phase 2. Message delete buttons and character/persona delete buttons exist in the UI but remain non-functional (deferred to Phase 7).

---

## Custom Utilities

**No new CSS utilities in Phase 2.** The existing utilities from Phase 1 (`font-display`, `surface-grain`, `streaming-caret`, `drag-region`, `no-drag`) are all carried forward unchanged.

The skeleton loading pattern uses Tailwind v4's built-in `animate-pulse` — no custom keyframes needed.

---

## Registry Safety

**No changes.** No shadcn, no third-party registries. All components remain custom-built using Bits.ui + Tailwind v4.

| Registry | Blocks Used | Safety Gate |
|----------|-------------|-------------|
| shadcn official | none | not applicable |
| Third-party | none | not applicable |

---

## Phase 2 Specifics

### What Changes

1. **`+layout.svelte`** — Replace mock data imports with `invoke()` calls. Add `is_loading` gate that shows skeleton UI during initial fetch. Add error overlay for DB init failure.
2. **`AppState.hydrate()`** — Accept data from `invoke()` results instead of mock imports. Add `is_loading` and `init_error` reactive state.
3. **`mock.ts`** — Deleted after wiring is complete. No UI references it after migration.
4. **Empty states** — Previously unreachable (mock data always populated), now the real first-run experience. Copy and CTA guidance added per copywriting contract.
5. **Skeleton loading** — New visual pattern during async data fetch. Uses existing tokens and Tailwind's `animate-pulse`.
6. **Error overlay** — New full-screen pattern for DB init failure. Uses existing destructive color token.
7. **Conversation delete** — Now has a real backend. Confirmation dialog added.

### What Does NOT Change

- No new design tokens (spacing, typography, color, radii)
- No new components (no Dialog, no Toast, no Skeleton component — skeleton is inline pattern, not a reusable component)
- No new CSS utilities
- No changes to existing component visual structure (CharacterCard, PersonaCard, MessageBubble, etc. look the same)
- No new routes or navigation
- No changes to the icon library

### Data Flow Change

```
Phase 1:  mock.ts ──import──→ +layout.svelte ──hydrate──→ AppState
Phase 2:  Tauri invoke() ←──→ Rust DbService ──→ SQLite
              │
              └── +layout.svelte onMount ──hydrate──→ AppState
```

The `AppState` class structure is preserved — only the data source changes. This means:
- All derived getters (`active_conversation`, `active_character`, `filtered_characters`, etc.) work unchanged
- All component subscriptions (`app.characters`, `app.conversations`, etc.) work unchanged
- Only `hydrate()` and `onMount` logic in `+layout.svelte` are rewritten

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS
- [ ] Dimension 2 Visuals: PASS
- [ ] Dimension 3 Color: PASS
- [ ] Dimension 4 Typography: PASS
- [ ] Dimension 5 Spacing: PASS
- [ ] Dimension 6 Registry Safety: PASS

**Approval:** pending
