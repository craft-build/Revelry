# Phase 1: Foundation - UI Design Contract

**Phase:** 01-foundation
**Date:** 2025-04-28
**Status:** Auto-generated (auto_chain)

---

## Visual Language

### Design System: GNOME Adwaita

- **Palette:** Adwaita light/dark with accent blue (#3584e4)
- **Typography:** System font stack (Cantarell on GNOME, SF Pro on macOS, Segoe UI on Windows)
- **Spacing:** 4px grid system, 12px base unit
- **Border radius:** 8px cards, 12px panels, 4px buttons
- **Icon set:** Lucide React (symbolic/outline variants)
- **Shadows:** Subtle, only for overlays and panels (z-depth pattern)

### Color Tokens

| Token | Light | Dark | Usage |
|-------|-------|------|-------|
| `--color-bg` | `#ffffff` | `#242424` | Page background |
| `--color-surface` | `#f6f5f4` | `#303030` | Sidebar, panels |
| `--color-card` | `#ffffff` | `#383838` | Cards, list items |
| `--color-text` | `#2e3436` | `#ffffff` | Primary text |
| `--color-text-secondary` | `#77767b` | `#9a9996` | Labels, descriptions |
| `--color-accent` | `#3584e4` | `#3584e4` | Interactive elements |
| `--color-accent-hover` | `#2a76d6` | `#2a76d6` | Hover state |
| `--color-border` | `#c0bfbc` | `#505050` | Borders, dividers |
| `--color-headerbar` | `#303030` | `#303030` | Header bar bg |
| `--color-destructive` | `#e01b24` | `#e01b24` | Delete actions |
| `--color-success` | `#33d17a` | `#33d17a` | Connection success |

---

## Layout Structure

### Application Shell

```
┌──────────────────────────────────────────────────────────┐
│  Header Bar (48px)                                        │
│  [Persona Logo]  [Chat | Characters | Settings]  [👤 Name]│
├────────────┬─────────────────────────────────────────────┤
│            │                                             │
│  Sidebar   │     Content Area                            │
│  240px     │     (fills remaining space)                 │
│            │                                             │
│            │                                             │
│            │                                             │
│            │                                             │
│  [≡ Toggle]│                                             │
├────────────┴─────────────────────────────────────────────┤
```

### Header Bar

- **Height:** 48px fixed
- **Background:** Dark (#303030) in both themes
- **Left:** App logo/name "Persona" (16px font, white text)
- **Center:** View switcher tabs — Chat, Characters, Settings (flat tab style, active tab underline in accent blue)
- **Right:** Active persona avatar (24px circle) + name, then menu button (3-dot icon)
- **Persona indicator:** When a default persona is set, show avatar circle + name. When no persona, show "Set Persona" link in accent color

### Sidebar

- **Width:** 240px expanded, 48px collapsed
- **Background:** Surface color (#f6f5f4 light, #1e1e1e dark)
- **Toggle:** Ctrl+B keyboard shortcut, or hamburger icon button
- **Content (Chat view):** Conversation list items — character avatar (32px) + character name + last message preview (truncated)
- **Content (Characters view):** Hidden (content area shows grid)
- **Content (Settings view):** Hidden (full-width content)
- **Collapsed state:** Show only conversation avatar icons (32px centered)
- **Responsive:** <1024px → sidebar auto-collapses to overlay with backdrop

### Content Area

Three views switched by header bar tabs:

1. **Chat View** (placeholder in Phase 1): Shows "Select a conversation or start a new one" with persona indicator
2. **Characters View**: "My Personas" tab with persona grid/list + create button
3. **Settings View**: Provider configuration, keychain setup, about section

---

## Component Specifications

### Persona Card (Grid Item)

- **Size:** 160×200px card
- **Layout:** Avatar (80px circle) on top, name below, tags below name
- **Hover:** Subtle elevation (shadow increase), accent border on left
- **Click:** Opens persona editor sliding panel
- **Context menu:** Set as Default, Delete

### Persona Editor (Sliding Panel)

- **Position:** Slides in from right, 360px wide
- **Overlay:** Semi-transparent backdrop (rgba(0,0,0,0.4))
- **Close:** X button top-right, Escape key, or click backdrop
- **Sections (collapsible, progressive disclosure):**

  1. **Identity** (expanded by default): Avatar picker (64px circle, click-to-upload), Name field
  2. **Description** (collapsed): Appearance, Personality, Background text areas
  3. **Role** (collapsed): Role Description, Relationships, Speech Style text areas
  4. **Meta** (collapsed): Notes textarea, Tags input (chip-style)

- **Actions:** Save button (bottom), Delete button (destructive, with confirmation dialog)
- **Token estimate:** Placeholder text "Token count: available in Phase 2" (reserved space per D-26)

### Avatar Picker

- **Pattern:** GNOME Contacts style
- **Display:** 64px circle with current avatar or placeholder user icon
- **Interaction:** Click opens file dialog (Tauri dialog::open), filtered for image types
- **Selected file:** Copied to `$APPDATA/persona/avatars/`, thumbnail generated (128×128)
- **Placeholder:** Lucide `User` icon in circle with surface background

### Toast Notification

- **Pattern:** Non-modal, bottom-center overlay
- **Duration:** 5 seconds auto-dismiss
- **Types:** Success (green accent), Error (destructive red), Info (neutral)
- **Content:** Icon + message text + optional action button

### Confirmation Dialog

- **Pattern:** GNOME HIG modal dialog
- **Delete persona:** "Delete [name]?" with "Cancel" and "Delete" buttons
- **Delete button:** Destructive red styling
- **Escape:** Closes dialog (cancel action)

---

## Interaction Patterns

### Persona CRUD Flow

1. **Create:** Click "New Persona" button → empty editor panel slides in → fill fields → Save → panel closes, card appears in grid
2. **Edit:** Click persona card → editor panel slides in with populated fields → edit → Save → panel closes, card updates
3. **Delete:** Right-click card → "Delete" → confirmation dialog → confirm → card removed, toast "Persona deleted"
4. **Set Default:** Right-click card → "Set as Default" → is_default toggles, star/badge appears on card, old default loses badge

### Sidebar Toggle

- Ctrl+B or click hamburger icon
- Animation: 200ms ease-out width transition
- Collapsed: avatar-only icons (32px centered in 48px sidebar)
- Expanded: full list with character name + preview

### View Switching

- Click tab in header bar → content area transitions
- Active tab: accent underline (2px), bold text
- Inactive tab: neutral text, no underline
- No animation on content switch (instant swap per GNOME pattern)

---

## Responsive Behavior

| Viewport | Layout | Sidebar | Details |
|----------|--------|---------|---------|
| >=1024px | Full | 240px visible | Sidebar + content side by side |
| 768-1023px | Compact | Overlay | Sidebar slides over content with backdrop |
| <768px | Mobile-like | Overlay | Same overlay pattern, content fills width |

---

## Accessibility Notes

- All interactive elements keyboard-navigable
- Avatar images have `alt` attributes with persona name
- Color contrast: text meets WCAG AA (4.5:1 ratio)
- Focus indicators: accent blue outline (2px)
- Screen reader: proper ARIA labels on all controls

---

*UI-SPEC generated: 2025-04-28*
