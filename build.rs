fn main() {
    // Bundle relm4-icons for use in the application
    // Icons are from: icon-development-kit, fluentui-system-icons, material-symbols
    let icons: &[&str] = &[
        // Main view icons
        "chat",
        "list",
        "settings",
        // Action icons
        "add",
        "folder-open",
        "contact-page",
        "mood",
        "edit",
        "tune",
        "people",
        "menu",
        "done",
        "error",
        // Chat / message icons
        "send",
        "stop",
        "view-more",
        // Character card overlay icons
        "document-save",
        "user-trash",
    ];
    relm4_icons_build::bundle_icons(
        "icon_names.rs",
        Some("com.revelry.app"),
        None::<&str>,
        None::<&str>,
        icons,
    );
}
