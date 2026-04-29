use relm4::prelude::*;
use relm4::{adw, ComponentParts, ComponentSender};

/// Chat panel showing conversation view or empty state.
/// Phase 1: Empty state only — conversations come in Phase 3.
pub struct ChatPanel {
    has_providers: bool,
}

/// Messages handled by ChatPanel.
#[derive(Debug)]
pub enum ChatPanelMsg {
    SetHasProviders(bool),
}

#[relm4::component(pub)]
impl SimpleComponent for ChatPanel {
    type Init = bool; // has_providers
    type Input = ChatPanelMsg;
    type Output = ();

    view! {
        #[root]
        adw::StatusPage {
            set_icon_name: Some("system-users-symbolic"),

            #[watch]
            set_title: &if model.has_providers {
                "Select a Conversation"
            } else {
                "Welcome to Revelry"
            },

            #[watch]
            set_description: Some(&if model.has_providers {
                "Choose a conversation from the sidebar, or create a new one."
            } else {
                "Configure an AI provider to get started. Open Settings to add your first provider."
            }),
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = ChatPanel {
            has_providers: init,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            ChatPanelMsg::SetHasProviders(has_providers) => {
                self.has_providers = has_providers;
            }
        }
    }
}
