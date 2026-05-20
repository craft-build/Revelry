<!-- Revelry Design Specification v2.0 — Section 4 -->
<!-- Last updated: 2026-05-20 -->

# Design Philosophy

## Design Principles

### Content-First Interface

The conversation is the primary activity in Revelry; all UI chrome recedes to make chat content dominant. Chat messages receive the maximum available screen space, with navigation and controls appearing only when contextually needed. The sidebar collapses when not in use (toggled via F9 or automatically in compact layouts), and secondary controls are accessed through progressive disclosure rather than permanent panels. The character's responses and the user's inputs are the visual priority — everything else exists to support that interaction.

### Progressive Disclosure

Revelry implements a tiered interface where casual users see a simple chat view with a message input, while advanced features like prompt editing, lorebook management, regex scripting, and provider configuration are revealed through contextual menus, expandable panels, and dedicated settings views. A new user can start chatting within seconds of launching the application, while power users can progressively discover and enable sophisticated customization without ever being overwhelmed by initial complexity. Features are organized into tiers: Tier 1 (core chat, character management) is immediately visible; Tier 2 (lorebooks, regex, advanced search) is discoverable but not prominent; Tier 3 (vault mode, persona evolution) is available but specialized.

### Dark-First Aesthetic

Dark theme is the default — creative roleplay users overwhelmingly work at night, and a dark interface reduces eye strain during extended sessions. OKLCH color tokens ensure perceptual uniformity across both dark and light modes, with carefully calibrated surface hierarchies (background → surface → surface-elevated → surface-hover) that maintain visual depth in either theme. A light theme toggle is available via the ThemeToggle component, and all custom colors are defined in a way that works for both modes through CSS custom properties on the `<html>` element's `class` attribute.

### Desktop-Native Feel

Revelry runs as a native desktop application inside a Tauri window, not a browser tab. This means native window controls, drag regions (via `.drag-region` CSS utility), keyboard shortcuts (F9 sidebar toggle, Ctrl+Enter send, Ctrl+N new chat), and a layout that fills the entire window without browser-style scrollbars on the body. Body scroll is intentionally disabled (`body { overflow: hidden }`); scrolling happens inside component containers (message list, sidebar conversation list) for a desktop app feel. The application respects the user's operating system conventions for window management and input methods.

### Accessibility

The UI uses bits-ui headless primitives, which provide full keyboard navigation and screen reader compatibility out of the box. Sufficient contrast ratios are enforced through the OKLCH token system. Focus management follows standard web conventions, and all interactive elements are reachable via keyboard. The component library (Dialog, Tooltip, Select, etc.) handles ARIA attributes automatically through the headless primitive pattern.

## Visual Language

### Color System

All colors are defined as OKLCH tokens in a `@theme { }` block within `src/app.css`. This provides perceptual uniformity — colors that appear equidistant in lightness to human perception — which is critical for a dark-theme-first application. The token categories are:

- **Surfaces:** `--color-background`, `--color-surface`, `--color-surface-elevated`, `--color-surface-hover`, `--color-surface-input` — layered depth hierarchy
- **Foreground:** `--color-foreground`, `--color-muted`, `--color-subtle` — three levels of text emphasis
- **Borders:** `--color-border`, `--color-border-strong` — subtle structural delineation
- **Accent:** `--color-accent`, `--color-accent-fg`, `--color-accent-muted` — teal-based interactive color
- **Semantic:** `--color-destructive`, `--color-warning`, `--color-success` — status colors
- **Chat bubbles:** `--color-bubble-user`, `--color-bubble-character` — speaker attribution via background color

Tokens are referenced in Tailwind classes using the `-(--token)` shorthand: `bg-(--color-surface)`, `text-(--color-muted)`, etc.

### Typography

Three font families are loaded via Google Fonts and defined as CSS custom properties:

- **Geist** (`--font-sans`) — sans-serif body font for all UI text
- **Fraunces** (`--font-serif`) — serif display font for character names via `.font-display` utility, with OpenType features `ss01`, `ss02`, and `liga` enabled for editorial flair
- **Geist Mono** (`--font-mono`) — monospace for code blocks and token counts

### Spacing and Radii

Spacing follows Tailwind's default scale. Border radii are tokenized: `--radius-sm`, `--radius-md`, `--radius-lg`, `--radius-xl`. Cards use `rounded-(--radius-lg)`, buttons use `rounded-(--radius-md)`.

### Custom Utilities

| Utility            | Purpose                                                            |
| ------------------ | ------------------------------------------------------------------ |
| `.font-display`    | Fraunces serif with OpenType features for character names          |
| `.surface-grain`   | Subtle radial-gradient texture overlay for atmospheric depth       |
| `.streaming-caret` | Animated `▍` caret on streaming AI responses                       |
| `.drag-region`     | `-webkit-app-region: drag` for Tauri window dragging               |
| `.no-drag`         | `-webkit-app-region: no-drag` to exclude elements from drag region |

## Interaction Patterns

### Keyboard Shortcuts

| Shortcut     | Action                             |
| ------------ | ---------------------------------- |
| F9           | Toggle sidebar                     |
| Enter        | Send message                       |
| Ctrl+Enter   | Newline in message input           |
| Ctrl+N       | New conversation                   |
| Ctrl+Shift+N | New character                      |
| Escape       | Cancel generation / close overlays |

### Sidebar Navigation

A left sidebar (`ConversationSidebar` component) displays the conversation list with relative timestamps. The sidebar toggles via F9, a hamburger button, or collapses automatically in narrow windows. It follows a CSS-based layout pattern (not a native overlay split view) with smooth transitions between collapsed and expanded states. Selected conversations immediately update the main content area without page transitions, maintaining spatial continuity.

### Drag Regions

The `AppHeader` component uses the `.drag-region` utility to make the header bar act as a native window title bar, allowing users to move the window by dragging the header. Interactive elements within the header use `.no-drag` to remain clickable. This replicates the consolidated header bar pattern from native desktop applications, combining window controls and navigation into a single space-efficient element.
