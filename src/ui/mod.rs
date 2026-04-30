mod character_card;
mod chat;
mod characters;
pub mod settings;

pub use character_card::create_card_widget;
pub use characters::{CharactersPanelModel, CharactersPanelMsg, CharactersPanelOutput};
pub use chat::{ChatPanel, ChatPanelMsg};
pub use settings::{SettingsPanel, SettingsPanelOutput};

/// Type alias for backward compatibility — app.rs stores this as Controller<CharactersPanel>.
pub type CharactersPanel = CharactersPanelModel;
