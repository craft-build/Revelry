mod character_card;
#[allow(deprecated)] // FileChooserNative deprecated in GTK 4.10 but correct for 0.11.x
mod character_editor;
pub mod chat;
mod characters;
pub mod settings;

pub use character_card::create_card_widget;
pub use character_editor::{CharacterEditorInit, CharacterEditorMsg, CharacterEditorOutput};
pub use characters::{CharactersPanelModel, CharactersPanelMsg, CharactersPanelOutput};
pub use chat::{ChatPanel, ChatPanelMsg, ChatPanelOutput, ChatCmdOut};
pub use settings::{SettingsPanel, SettingsPanelOutput};

/// Type alias for backward compatibility — app.rs stores this as Controller<CharactersPanel>.
pub type CharactersPanel = CharactersPanelModel;

/// Type alias — downstream consumers reference the component as `CharacterEditor`.
pub type CharacterEditor = character_editor::CharacterEditorModel;
