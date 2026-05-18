pub mod character;
pub mod conversation;
pub mod generation;
pub mod lorebook;
pub mod persona;
pub mod provider;

pub use character::{CharacterCardV2, CharacterV1, CharaCardV2Spec, CharaCardV2SpecData};
pub use conversation::{Conversation, Message};
pub use generation::GenerationPreset;
// LorebookActivation, LorebookEntry, LorebookPosition re-exported for public API surface
#[allow(unused_imports)]
pub use lorebook::{Lorebook, LorebookActivation, LorebookEntry, LorebookPosition};
pub use persona::Persona;
pub use provider::{GenerationParams, ProviderConfig, ProviderModelEntry, ProviderType};
