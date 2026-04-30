use revelry::app::AppModel;
use relm4::RelmApp;

mod icon_names {
    include!(concat!(env!("OUT_DIR"), "/icon_names.rs"));
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("revelry=debug")
        .init();

    relm4_icons::initialize_icons(icon_names::GRESOURCE_BYTES, icon_names::RESOURCE_PREFIX);

    let app = RelmApp::new("com.revelry.app");
    app.run::<AppModel>(());
}
