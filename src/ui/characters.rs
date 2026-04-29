use relm4::prelude::*;
use relm4::{adw, ComponentParts, ComponentSender};

/// Characters panel showing character library or empty state.
/// Phase 1: Empty state only — characters come in a future phase.
pub struct CharactersPanel;

/// Messages handled by CharactersPanel.
#[derive(Debug)]
pub enum CharactersPanelMsg {}

#[relm4::component(pub)]
impl SimpleComponent for CharactersPanel {
    type Init = ();
    type Input = CharactersPanelMsg;
    type Output = ();

    view! {
        #[root]
        adw::StatusPage {
            set_icon_name: Some("avatar-default-symbolic"),
            set_title: "No Characters Yet",
            set_description: Some("Character creation is coming soon. For now, configure your AI providers in Settings."),
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = CharactersPanel;
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, _msg: Self::Input, _sender: ComponentSender<Self>) {}
}
