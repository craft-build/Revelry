<!-- Revelry Design Specification v2.0 — Section 16 -->
<!-- Last updated: 2026-05-20 -->

# Appendix B: GTK4/libadwaita Widget Mapping

The following table maps the original Tauri/React UI design to the corresponding GTK4/libadwaita widgets and Relm4 patterns used in Revelry. This mapping ensures that every UI element from the original specification has a native GNOME equivalent.

| **Original Design Element** | **GTK4/libadwaita Widget** | **Relm4 Pattern** |
|---|---|---|
| React App Shell | AdwApplicationWindow | Relm4 Application component |
| Header Bar (custom) | AdwHeaderBar | Built-in Relm4 header bar support |
| View Switcher (tabs) | AdwViewSwitcher + AdwViewStack | Relm4 view switcher component |
| Sidebar Navigation | AdwOverlaySplitView | Relm4 split view with sidebar model |
| Conversation List | GtkListView with GtkSignalListItemFactory | Relm4 list component with factory |
| Character Grid | GtkFlowBox with custom card widgets | Relm4 flow box component |
| Chat Messages | GtkListView with custom message rows | Relm4 list component with factory |
| Message Input | GtkTextView in AdwClamp | Relm4 text input component |
| Circular Avatars | AdwAvatar | Direct widget usage |
| Card Editor Panel | AdwNavigationView + AdwNavigationPage | Relm4 navigation component |
| Settings Panel | AdwPreferencesWindow | Relm4 preferences component |
| Preferences Rows | AdwEntryRow, AdwSpinRow, AdwSwitchRow | Direct widget usage |
| Context Menu | GtkPopoverMenu | Relm4 popover component |
| Persona Switcher | GtkPopover with GtkListBox | Relm4 popover component |
| Search Bar | GtkSearchEntry with GtkFilterController | Relm4 search component |
| Toast Notifications | AdwToastOverlay + AdwToast | Relm4 toast component |
| About Dialog | AdwAboutWindow | Direct widget usage |
| File Picker | GtkFileChooserNative | Native file dialog |
| Branch Indicator | Custom widget (GtkBox with buttons) | Relm4 custom widget |
| Token Counter Bar | GtkProgressBar | Relm4 progress component |
