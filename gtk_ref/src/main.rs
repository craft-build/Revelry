use revelry::app::AppModel;
use relm4::RelmApp;

mod icon_names {
    include!(concat!(env!("OUT_DIR"), "/icon_names.rs"));
}

fn main() {
    // On macOS (and other non-Linux platforms), GTK file dialogs require
    // GSettings schemas to be findable. Homebrew installs them at
    // /opt/homebrew/share/glib-2.0/schemas but GTK won't find them
    // unless GSETTINGS_SCHEMA_DIR or XDG_DATA_DIRS is set.
    #[cfg(target_os = "macos")]
    {
        let schema_dir = "/opt/homebrew/share/glib-2.0/schemas";
        if std::path::Path::new(schema_dir).exists() {
            std::env::set_var("GSETTINGS_SCHEMA_DIR", schema_dir);
        }
    }

    tracing_subscriber::fmt()
        .with_env_filter("revelry=debug")
        .init();

    relm4_icons::initialize_icons(icon_names::GRESOURCE_BYTES, icon_names::RESOURCE_PREFIX);

    let app = RelmApp::new("com.revelry.app");
    app.run::<AppModel>(());
}
