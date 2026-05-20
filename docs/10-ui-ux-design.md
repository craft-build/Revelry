<!-- Revelry Design Specification v2.0 — Section 10 -->
<!-- Last updated: 2026-05-20 -->

# UI/UX Design

## 10.1 Primary Views

The application is organized into three primary views, accessible through the AdwHeaderBar view switcher. The Chat View is the default and dominant view, where active conversations take place. The Characters View provides character management, persona management, card editing, and import/export. The Settings View handles provider configuration, application preferences, and advanced customization.

## 10.2 Chat View Layout

The Chat View is the most complex and most-used view. The view uses an AdwOverlaySplitView for the adaptive sidebar/content layout. The sidebar contains the conversation list with persona indicators. The main content area contains an AdwHeaderBar at the top with the active character name, persona indicator (AdwAvatar + name), model indicator, and action buttons. Below the header, a GtkListView renders messages as custom widget rows, with character messages left-aligned and user messages right-aligned. Each message row includes hover-revealed action buttons (edit, regenerate, delete, copy) and branch navigation indicators when multiple sibling responses exist.

The input area at the bottom provides a GtkTextView for multi-line text input with send/stop buttons, a token counter using tiktoken-rs estimates, and a context utilization bar. The input area is wrapped in an AdwClamp to maintain comfortable line width on wide screens. The streaming AI response is rendered in a GtkLabel with Pango markup, updated incrementally as token chunks arrive through the glib channel bridge from the tokio runtime.

## 10.3 Characters View

The Characters View presents the user's character library as a responsive GtkFlowBox grid of character cards. Each card displays the character's AdwAvatar, name, creator, tags, and a brief description excerpt. Cards use the GNOME card pattern with rounded corners, subtle elevation shadow, and hover-activated action buttons. The grid adapts from three columns on wide screens to two on medium and one on narrow windows, handled automatically by GtkFlowBox's minimum child width property.

Above the grid, a GtkSearchEntry with GtkFilterControls allows quick character discovery using SQLite FTS5 for instant full-text matching across names, descriptions, and tags. A tab switcher (Characters / My Personas) provides access to the persona library. When a character is selected for editing, an AdwNavigationView slides in from the right with a tabbed card editor (Identity, Personality, Scenario, Advanced, Lorebook).

## 10.4 Settings View

The Settings View uses an AdwPreferencesWindow with categorized setting panels. Categories include: AI Providers (add/edit/remove provider configurations using AdwEntryRow and AdwPasswordEntryRow, test connectivity through rig client), Generation Defaults (temperature, top_p, max tokens, context length via AdwSpinRow), Prompt Presets (create/edit/delete named parameter profiles), Appearance (AdwStyleManager-based dark/light/system theme, accent color, font size, message density), Keyboard Shortcuts (view and customize all shortcuts), Data Management (export/import all data, backup/restore, clear history), and About (AdwAboutWindow with version, licenses, links).

## 10.5 Keyboard Navigation

Following GNOME accessibility standards, every action in Revelry can be performed via keyboard. The primary shortcuts include: Ctrl+N for new chat, Ctrl+Shift+N for new character, Ctrl+B to toggle sidebar, Ctrl+F to search, Ctrl+comma to open settings, Escape to cancel generation or close overlays, and Ctrl+Enter to send a message. Arrow keys navigate conversation branches and sidebar items. All shortcuts are documented in the Keyboard Shortcuts settings panel and can be customized through the GNOME shortcuts infrastructure.
