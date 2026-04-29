use revelry::app::AppModel;
use relm4::RelmApp;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("revelry=debug")
        .init();

    let app = RelmApp::new("com.revelry.app");
    app.run::<AppModel>(());
}
