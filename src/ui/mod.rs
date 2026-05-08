mod character_card;
#[allow(deprecated)] // FileChooserNative deprecated in GTK 4.10 but correct for 0.11.x
mod character_editor;
pub mod chat;
mod characters;
mod lorebook_editor;
mod persona_editor;
pub mod preset_editor;
pub mod regex_editor;
pub mod settings;

pub use character_card::create_card_widget;
pub use character_editor::{CharacterEditorInit, CharacterEditorMsg, CharacterEditorOutput};
pub use characters::{CharactersPanelModel, CharactersPanelMsg, CharactersPanelOutput};
pub use chat::{ChatPanel, ChatPanelMsg, ChatPanelOutput, ChatCmdOut};
pub use lorebook_editor::{LorebookEditorModel, LorebookEditorMsg, LorebookEditorOutput, LorebookField};
pub use persona_editor::{PersonaEditorInit, PersonaEditorOutput};
pub use preset_editor::{PresetEditorInit, PresetEditorOutput};
pub use regex_editor::{RegexScriptEditorModel, RegexScriptEditorMsg, RegexScriptEditorOutput};

/// Type alias — downstream consumers reference the component as `LorebookEditor`.
pub type LorebookEditor = LorebookEditorModel;

/// Type alias — downstream consumers reference the component as `PersonaEditor`.
pub type PersonaEditor = persona_editor::PersonaEditorModel;
pub use settings::{SettingsPanel, SettingsPanelOutput};

/// Type alias for backward compatibility — app.rs stores this as Controller<CharactersPanel>.
pub type CharactersPanel = CharactersPanelModel;

/// Type alias — downstream consumers reference the component as `CharacterEditor`.
pub type CharacterEditor = character_editor::CharacterEditorModel;
