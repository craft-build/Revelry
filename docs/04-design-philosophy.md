<!-- Revelry Design Specification v2.0 — Section 4 -->
<!-- Last updated: 2026-05-20 -->

# Design Philosophy: GNOME Ethos

## 4.1 Core Principles Applied

### 4.1.1 Content-First Design

In GNOME applications, the user's content is always the visual priority. Revelry applies this principle by making the conversation area the dominant element in every view. Chat messages receive the maximum available screen space, with UI chrome receding to minimal, contextually appropriate elements. Navigation elements collapse when not in use through the AdwOverlaySplitView adaptive sidebar pattern, and secondary controls are accessed through progressive disclosure rather than permanent panels. The character's responses and the user's inputs are the stars; everything else exists to support that interaction.

### 4.1.2 Progressive Disclosure

GNOME advocates showing only what is needed at each level of interaction. Revelry implements this through a tiered interface where casual users see a simple chat view with a message input, while advanced features like prompt editing, lorebook management, and response regeneration are revealed through contextual menus, expandable panels, and keyboard shortcuts. A new user can start chatting within seconds of launching the application, while power users can progressively discover and enable sophisticated customization without ever being overwhelmed by initial complexity.

### 4.1.3 Consistent Visual Language

GNOME maintains a rigorous visual consistency through its Human Interface Guidelines. Revelry achieves this natively through libadwaita widgets, which enforce visual consistency at the toolkit level. Every AdwHeaderBar, AdwAvatar, AdwClamp, AdwPreferencesWindow, and AdwAlertDialog follows the same visual grammar automatically. Custom widgets are styled through GTK4's CSS system, using named colors from the Adwaita palette to ensure they blend seamlessly with the rest of the application. This eliminates the need for a separate design token system since the Adwaita theme itself serves as the single source of truth for colors, spacing, and typographic scale.

### 4.1.4 Header Bar Pattern

Rather than traditional menu bars and separate title bars, GNOME applications use a consolidated AdwHeaderBar that combines the window title, primary navigation, and key actions into a single, space-efficient element. Revelry adopts this pattern with an AdwHeaderBar containing the application name, view switcher using AdwViewSwitcher, and contextual actions. This saves approximately thirty to forty pixels of vertical space compared to separate title bar and menu bar approaches, which directly benefits the content area.

### 4.1.5 Sidebar Navigation

GNOME applications like Files, Contacts, and Music use a left sidebar for primary navigation, allowing the main content area to remain focused. Revelry implements this with AdwOverlaySplitView, which provides an adaptive sidebar that sits alongside content in wide windows and overlays in narrow windows. The sidebar can be toggled with a keyboard shortcut (F9) and auto-hides in compact window sizes, giving users full control over their screen real estate. Selected items in the sidebar immediately update the main content view without page transitions, maintaining spatial continuity.

### 4.1.6 Adaptive Layout

While desktop-focused, GNOME design accounts for different window sizes. Revelry's layout adapts gracefully using AdwOverlaySplitView and AdwClamp: in wide windows, the sidebar and main content sit side by side; in narrower windows, the sidebar becomes an overlay; and the conversation view reflows through GtkListView's natural layout management. This ensures the application works well on screens ranging from compact laptops to ultrawide monitors without sacrificing usability or visual quality.

## 4.2 Visual Style Guide

| **Element** | **Specification** | **Rationale** |
|---|---|---|
| Color Scheme | Adwaita light/dark (system sync via AdwStyleManager) | Native GNOME theming, zero custom color management |
| Accent Color | System accent color via Adwaita, fallback Teal #4ECDC4 | Respects user's GNOME accent color preference |
| Typography | System font (Cantarell on GNOME, platform default elsewhere) | Native feel, zero font loading delay, respects OS preferences |
| Border Radius | Native Adwaita widget defaults (12px cards, 8px buttons) | Automatic consistency with GNOME HIG |
| Spacing Grid | Adwaita internal widget spacing | Handled by the toolkit, no manual grid system needed |
| Icon Style | Adwaita symbolic icons via GtkImage | Native GNOME icon pattern, automatic dark/light adaptation |
| Shadows | Native GTK4 elevation via GtkBox shadow CSS | Consistent with GTK4/libadwaita patterns |
| Animations | Native GTK4 transition durations (200-300ms) | Matches GNOME motion guidelines automatically |
| Message Bubbles | Character: left-aligned, User: right-aligned, rounded | Clear speaker attribution, chat convention familiarity |
