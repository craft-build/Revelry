---
phase: 1
slug: native-desktop-shell
status: approved
reviewed_at: 2026-05-10
shadcn_initialized: false
preset: none
created: 2026-05-10
---

# Phase 1 — UI Design Contract

> Visual and interaction contract for the Native Desktop Shell phase. This phase adds NO new UI — it wraps the existing SvelteKit SPA in a Tauri desktop window. This document locks in the existing design system as the design baseline that all future phases build upon.

---

## Design System

| Property | Value |
|----------|-------|
| Tool | none (no shadcn — project uses Bits.ui + Tailwind v4 directly) |
| Preset | not applicable |
| Component library | Bits.ui ^2.18 (headless Svelte primitives — NOT shadcn/radix) |
| Icon library | @lucide/svelte ^1.14 |
| Font | Geist (sans), Fraunces (serif/display), Geist Mono (mono) |
| Styling | Tailwind CSS v4 with custom OKLCH theme tokens in `src/app.css` |
| Variant system | tailwind-variants ^3.2 (`tv()`) for component variant APIs |

---

## Spacing Scale

Declared values (multiples of 4 only, per Tailwind v4 default scale):

| Token | Value | Usage |
|-------|-------|-------|
| 1 | 4px | Inline gaps, icon-to-text spacing, compact padding |
| 2 | 8px | Compact element spacing, small button padding |
| 3 | 12px | Inner component padding, avatar-to-text gaps |
| 4 | 16px | Default element spacing, card padding, section gaps |
| 5 | 20px | Medium section gaps |
| 6 | 24px | Section padding, input area padding |
| 8 | 32px | Layout gaps, sidebar width subdivisions |
| 10 | 40px | Major section breaks |
| 12 | 48px | Major section breaks |
| 16 | 64px | Page-level spacing |

Exceptions: none — all observed spacing in the codebase follows multiples of 4.

---

## Typography

| Role | Size | Weight | Line Height | Font Family |
|------|------|--------|-------------|-------------|
| Body (chat bubbles, input) | 15px | 400 (regular) | 1.625 (relaxed) | Geist (sans) |
| Label / small (sidebar items, nav, metadata) | 12px (xs) | 500 (medium) | 1.25 | Geist (sans) |
| Heading (speaker names, card titles) | 14px (sm) | 600 (semibold) | 1.2 | Fraunces (serif/display) |
| Context / micro (timestamps, token counts, hints) | 10px | 400 (regular) | 1.0 | Geist (sans) |

**Font families (from `src/app.css` @theme):**
- `--font-sans`: Geist, ui-sans-serif, system-ui, sans-serif — body text, UI chrome
- `--font-serif`: Fraunces, ui-serif, Georgia, serif — character names, app identity, display headings (via `.font-display` utility)
- `--font-mono`: Geist Mono, ui-monospace, SF Mono, Menlo, monospace — code blocks, token counts

**Font feature settings:**
- Sans: `'cv11', 'ss01', 'ss03'` (Geist alternates)
- Serif (`.font-display`): `'ss01', 'ss02', 'liga'` with `font-optical-sizing: auto`

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
| **Destructive** | `oklch(0.66 0.18 25)` | `--color-destructive` | Stop-generation button, delete actions only |
| **Warning** | `oklch(0.78 0.14 80)` | `--color-warning` | Not currently used in Phase 1; reserved for future |
| **Success** | `oklch(0.72 0.15 155)` | `--color-success` | Not currently used in Phase 1; reserved for future |
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

---

## Radii

| Token | Value | Usage |
|-------|-------|-------|
| `--radius-sm` | 0.375rem (6px) | Small buttons, icon buttons, action chips, nav pills |
| `--radius-md` | 0.5rem (8px) | Default buttons, inputs, nav switcher, sidebar items |
| `--radius-lg` | 0.75rem (12px) | Chat bubbles, input frames, conversation cards |
| `--radius-xl` | 1rem (16px) | Reserved for future use (modals, dialogs) |

---

## Component Inventory

Existing components at Phase 1 baseline. No new components added this phase.

### UI Primitives (`src/lib/components/ui/`)

| Component | Source | Variants | Notes |
|-----------|--------|----------|-------|
| **Avatar** | Custom (Bits.ui-free) | `size`: sm/md | Initials fallback or image |
| **Button** | Custom (`tv()` variants) | `variant`: primary/secondary/ghost/outline/destructive/link; `size`: sm/md/lg/icon/icon-sm | Primary = accent bg + accent-fg text |
| **Input** | Custom (Bits.ui-free) | — | Styled for sidebar search |
| **Tag** | Custom | — | Not observed in active use |
| **Tooltip** | Custom | `text` prop | Wraps action buttons in header/sidebar |

### Domain Components (`src/lib/components/`)

| Component | Domain | Notes |
|-----------|--------|-------|
| **AppHeader** | layout | 48px tall, drag region, nav switcher (3 views), sidebar toggle, app identity |
| **ConversationSidebar** | layout | 288px wide (w-72), collapsible, search, conversation list with avatars |
| **ChatHeader** | chat | Character info bar above message area |
| **MessageBubble** | chat | Chat messages with avatar, speaker name, bubble, hover actions (copy/edit/regenerate/delete) |
| **MessageInput** | chat | Auto-grow textarea, context bar (token count + progress), send/stop buttons, attach button |
| **PersonaSwitcher** | chat | Popover for switching active persona in conversation |
| **CharacterCard** | characters | Card in library grid |
| **PersonaCard** | personas | Card in library grid |

### Layout Pattern

```
┌──────────────────────────────────────────────────────┐
│ AppHeader (h-12, drag region)                         │
│ [☰] [🔷 Revelry]    [Chat|Library|Settings]    [   ] │
├──────────┬───────────────────────────────────────────┤
│ Sidebar  │ Main Content                               │
│ (w-72)   │ (flex-1)                                   │
│          │                                            │
│ [+Chat]  │ ┌─ ChatHeader ─────────────────────────┐   │
│ [Search] │ │ Character name + info                  │   │
│          │ ├────────────────────────────────────────┤   │
│ Conv 1   │ │ Messages (scroll)                     │   │
│ Conv 2   │ │ MessageBubble × N                     │   │
│ Conv 3   │ │                                       │   │
│          │ ├────────────────────────────────────────┤   │
│          │ │ MessageInput                           │   │
│          │ │ [📎] [textarea...] [Send/Stop]         │   │
│          │ │ 8,192 / 8,192 tok ████░░░░ Enter→send │   │
│          │ └────────────────────────────────────────┘   │
└──────────┴───────────────────────────────────────────┘
```

**Key dimensions:**
- AppHeader: `h-12` (48px)
- Sidebar: `w-72` (288px), collapsible to `w-0`
- Full viewport: `h-dvh` (100dvh), `overflow: hidden` on body

---

## Interaction Patterns

Existing interactions at Phase 1 baseline. No new interactions added this phase.

| Pattern | Trigger | Behavior |
|---------|---------|----------|
| **Sidebar toggle** | F9 key / panel-left button | Collapses/expands sidebar with 200ms width transition |
| **Send message** | Enter (or Ctrl/Cmd+Enter) | Submits text, clears input, resets auto-grow |
| **Newline in input** | Shift+Enter | Inserts line break in textarea |
| **Message actions** | Hover on MessageBubble | Reveals action row (copy, edit, regenerate, delete) with opacity transition |
| **Auto-grow textarea** | Input event | Grows to content, max 240px height |
| **Context bar** | Derived from messages + input | Real-time token count with accent progress bar |
| **Nav switch** | Click nav tab | SvelteKit client-side navigation (no full reload) |
| **Drag region** | Header area | `-webkit-app-region: drag` on header, `no-drag` on interactive elements |
| **Streaming caret** | AI response streaming | Blinking `▍` character via `caret-blink` keyframes (1s, steps(2)) |
| **Focus ring** | `:focus-visible` | 2px accent outline, 2px offset, sm radius |

---

## Copywriting Contract

Phase 1 has no new user-facing copy — all existing copy is carried over as-is.

| Element | Copy | Source |
|---------|------|--------|
| App title | "Revelry" | CONTEXT.md D-04 |
| Nav tabs | "Chat", "Library", "Settings" | AppHeader.svelte |
| New chat button | "New chat" | ConversationSidebar.svelte |
| Search placeholder | "Search conversations" | ConversationSidebar.svelte |
| Empty state (no search match) | "No conversations match." | ConversationSidebar.svelte |
| Empty state (no conversations) | "No conversations yet." | ConversationSidebar.svelte |
| Input placeholder | "Message {character name}…" | MessageInput.svelte |
| Context hint | "Enter to send · Shift+Enter for newline" | MessageInput.svelte |
| Token display | "{used} / {limit} tok" | MessageInput.svelte |
| Sidebar toggle tooltip | "Toggle sidebar (F9)" | AppHeader.svelte |
| Stop generation | Square icon (no label) | MessageInput.svelte |
| Send | Send icon (no label) | MessageInput.svelte |
| Message actions (icon-only) | Copy, Edit, Regenerate, Delete | MessageBubble.svelte |

**Destructive actions in Phase 1:** None — delete buttons exist on messages but are non-functional (mock data).

---

## Custom Utilities

Defined in `src/app.css` `@layer utilities`:

| Utility | Purpose | Usage |
|---------|---------|-------|
| `.font-display` | Fraunces serif with optical sizing | Character names, app identity text |
| `.surface-grain` | Subtle radial gradient atmosphere | Root layout container |
| `.streaming-caret` | Blinking caret for streaming | Active AI message bubbles |
| `.drag-region` | Tauri window drag area | AppHeader |
| `.no-drag` | Exclude from drag area | Interactive elements within drag region |

**Scrollbar styling (base layer):**
- 10px width, transparent track
- Thumb: `--color-border` with content-box clip (2px transparent border)
- Hover: `--color-border-strong`

**Selection styling (base layer):**
- Background: `--color-accent-muted`
- Color: `--color-foreground`

---

## Registry Safety

| Registry | Blocks Used | Safety Gate |
|----------|-------------|-------------|
| shadcn official | none | not applicable (no shadcn) |
| Third-party | none | not applicable |

**No third-party registries.** All components are custom-built using Bits.ui headless primitives + Tailwind v4 styling + tailwind-variants.

---

## Phase 1 Specifics

This phase adds NO visual changes. The contract above documents the **existing** design system as the baseline that Tauri wraps. The only UI-relevant work is:

1. **Drag region activation** — The `.drag-region` / `.no-drag` CSS classes are already defined in `app.css`. When Tauri is initialized, the AppHeader becomes a functional drag region for window movement.
2. **DevTools toggle** — Tauri production builds will have DevTools disabled per CONTEXT.md D-04.
3. **Window properties** — Title "Revelry", size 1200×800, set in `tauri.conf.json` (not in CSS).
4. **CSP enforcement** — Production CSP restricts inline scripts and eval; dev CSP allows them for Vite HMR.

No design tokens, components, or visual patterns change in this phase.

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS
- [ ] Dimension 2 Visuals: PASS
- [ ] Dimension 3 Color: PASS
- [ ] Dimension 4 Typography: PASS
- [ ] Dimension 5 Spacing: PASS
- [ ] Dimension 6 Registry Safety: PASS

**Approval:** approved 2026-05-10
