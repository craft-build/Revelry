<!-- Revelry Design Specification v2.0 — Section 10 -->
<!-- Last updated: 2026-05-20 -->

# UI/UX Design

## 10.1 Primary Views

The application is organized into three primary views, accessible through navigation in the `AppHeader`. The Chat View is the default and dominant view, where active conversations take place. The Characters View provides character management, persona management, card editing, and import/export. The Settings View handles provider configuration, application preferences, and advanced customization.

## 10.2 Chat View Layout

The Chat View is the most complex and most-used view. It uses a CSS-based sidebar/content layout managed by the `ConversationSidebar` component.

**Sidebar (ConversationSidebar):** Contains the conversation list with relative timestamps (today → time, this week → weekday, older → date), persona indicators, and search filtering. The sidebar toggles via F9, the hamburger button in the header, or collapses automatically in narrow viewports. New conversations are started from the sidebar.

**Main content area:** The `AppHeader` component provides the window drag region, active character name (displayed in Fraunces serif via `.font-display`), persona indicator (avatar + name from `PersonaSwitcher`), provider/model selector (`ProviderModelSelector`), and action buttons.

**Message area:** A scrollable list of `MessageBubble` components renders the conversation. Character messages are left-aligned with `bg-(--color-bubble-character)` background; user messages are right-aligned with `bg-(--color-bubble-user)` background. Each message includes:

- Markdown rendering via `MarkdownRenderer` (marked + DOMPurify + Shiki) for completed messages
- Raw text display during streaming with `.streaming-caret` animation
- Hover-revealed action buttons (edit, regenerate, delete, copy)
- `BranchBadge` and `BranchPopover` indicators when multiple sibling responses exist
- Token count display
- Edit and regenerate actions that trigger backend commands

**Input area (`MessageInput`):** A `Textarea` component with auto-grow behavior, send/stop button, token estimate display (using `estimateTokens()` heuristic or real counts from backend), and a context utilization bar showing how much of the model's context window is in use. Enter sends the message; Ctrl+Enter inserts a newline.

## 10.3 Characters View

The Characters View presents the user's character library as a responsive CSS grid of `CharacterCard` components. Each card displays the character's `Avatar` (with initials fallback), name (in Fraunces serif), creator, tags as `Tag` components, and a brief description excerpt. Cards use rounded corners (`rounded-(--radius-lg)`), subtle borders, and hover-activated action buttons.

Above the grid, an `Input` component with search binding provides quick character discovery using real-time filtering by name, description, and tags. A tab switcher (Characters / My Personas) provides access to the persona library (`PersonaCard` grid). When a character is selected for editing, a `CharacterEditorDialog` opens as a multi-step wizard dialog (Identity → Personality → Scenario → Advanced) with a `StepperHeader` indicating progress.

The characters view also includes:

- `ImportErrorDialog` for failed character card imports
- `DeleteConfirmationDialog` for confirm-before-delete flows
- `RegexScriptEditor` for per-character regex post-processing scripts

## 10.4 Settings View

The Settings View uses a section-based layout with categorized setting panels. Categories include:

- **AI Providers** — `ProviderEditorDialog` for adding/editing/removing provider configurations with `ModelEntryRow` components for per-provider model management, API key storage via keyring, and `KeyringUnavailableBanner` when OS keychain is unavailable
- **Generation Defaults** — Temperature, top_p, max_tokens, context length, frequency_penalty, presence_penalty
- **Prompt Presets** — Create/edit/delete named parameter profiles via `PresetEditorDialog`
- **Appearance** — `ThemeToggle` component for dark/light theme switching
- **Data Management** — Export/import, backup/restore

## 10.5 Keyboard Navigation

Every action in Revelry can be performed via keyboard. The primary shortcuts include:

| Shortcut     | Action                              |
| ------------ | ----------------------------------- |
| F9           | Toggle sidebar                      |
| Enter        | Send message                        |
| Ctrl+Enter   | Newline in message input            |
| Ctrl+N       | New conversation                    |
| Ctrl+Shift+N | New character                       |
| Escape       | Cancel generation or close overlays |

Arrow keys navigate conversation branches and sidebar items. All interactive components (Dialog, Select, Tooltip) support keyboard navigation through bits-ui's headless primitive pattern, which handles focus management and ARIA attributes automatically.

## 10.6 Theme and CSS System

### Tailwind v4 Theme Tokens

All colors are defined as OKLCH CSS custom properties in a `@theme { }` block in `src/app.css`. Token categories: surfaces (5 levels of depth), foreground (3 emphasis levels), borders, accent, semantic, chat bubbles, typography (3 font families), and radii (4 sizes). Tokens are referenced in Tailwind classes via the `-(--token)` shorthand.

### Dark/Light Theme

Dark theme is the default (`<html class="dark">`). The `ThemeToggle` component toggles between dark and light by switching the class. All color tokens are defined once and produce appropriate values for both modes through CSS custom properties. Body scroll is intentionally disabled (`body { overflow: hidden }`); scroll happens inside component containers for the desktop app feel.

### Custom Utilities

| Utility            | Purpose                                                        |
| ------------------ | -------------------------------------------------------------- |
| `.font-display`    | Fraunces serif for character names (OpenType ss01, ss02, liga) |
| `.surface-grain`   | Subtle radial-gradient texture overlay                         |
| `.streaming-caret` | Animated caret on streaming AI responses                       |
| `.drag-region`     | Tauri window drag area                                         |
| `.no-drag`         | Exclude elements from drag region                              |
