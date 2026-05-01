pub mod input_bar;
pub mod message_row;

use adw::prelude::*;
use relm4::prelude::*;
use relm4::{adw, gtk};

use crate::domain::character::CharacterCardV2;
use crate::domain::conversation::{Conversation, Message, MessageRole};
use crate::domain::provider::ProviderConfig;
use crate::service::markdown::markdown_to_pango;

use self::input_bar::{InputBar, InputBarMsg, InputBarOutput};
use self::message_row::{MessageRow, MessageRowInit};

/// Chat panel component with streaming support.
/// Displays conversation messages and handles user input.
pub struct ChatPanel {
    active_conversation: Option<Conversation>,
    active_character: Option<CharacterCardV2>,
    active_provider: Option<ProviderConfig>,
    messages: Vec<Message>,
    partial_response: String,
    streaming: bool,
    has_providers: bool,
    characters: Vec<CharacterCardV2>,
    providers: Vec<ProviderConfig>,
    
    // Widget references
    messages_container: gtk::ListBox,
    input_bar: relm4::Controller<InputBar>,
}

/// Messages handled by ChatPanel.
#[derive(Debug)]
pub enum ChatPanelMsg {
    SetHasProviders(bool),
    SetActiveProvider(Option<ProviderConfig>),
    LoadConversation {
        conversation: Conversation,
        messages: Vec<Message>,
        character: CharacterCardV2,
    },
    ClearConversation,
    CharactersLoaded(Vec<CharacterCardV2>),
    ProvidersLoaded(Vec<ProviderConfig>),
    SendMessage(String),
    CancelStream,
    ConversationStarted(Conversation),
    MessagesLoaded(Vec<Message>),
    MessageSaved(Message),
    LoadApiKey(String),
    UpdatePartialResponse(String),
    StreamComplete(String),
    StreamError(String),
}

/// Command output from background operations (streaming, API calls).
#[derive(Debug)]
pub enum ChatCmdOut {
    TokenReceived(String),
    StreamComplete(String),
    StreamError(String),
}

/// Output messages from ChatPanel to parent (AppModel).
#[derive(Debug)]
pub enum ChatPanelOutput {
    StartConversation { character_id: String, provider_id: String },
    SaveMessage(Message),
    LoadMessages { conversation_id: String, branch_id: Option<String> },
    LoadConversations,
    DeleteConversation(String),
    RequestApiKey(String),
    ShowToast(String),
}

#[relm4::component(pub)]
impl Component for ChatPanel {
    type Init = bool;
    type Input = ChatPanelMsg;
    type Output = ChatPanelOutput;
    type CommandOutput = ChatCmdOut;

    view! {
        #[root]
        adw::ToolbarView {
            add_top_bar = &adw::HeaderBar {
                #[name(header_box)]
                pack_start = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 8,

                    #[name(character_avatar)]
                    adw::Avatar::new(32, None::<&str>, false),

                    #[name(character_name_label)]
                    gtk::Label::new(None::<&str>),
                },

                #[name(stop_button)]
                pack_end = &gtk::Button {
                    set_icon_name: "process-stop",
                    set_tooltip_text: Some("Stop Generation"),
                    #[watch]
                    set_visible: model.streaming,
                    connect_clicked => ChatPanelMsg::CancelStream,
                },
            },

            #[wrap(Some)]
            set_content = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                #[name(messages_scrolled)]
                gtk::ScrolledWindow {
                    set_vexpand: true,

                    #[name(messages_list_box)]
                    gtk::ListBox {
                        set_selection_mode: gtk::SelectionMode::None,
                        set_margin_top: 8,
                        set_margin_bottom: 8,
                    },
                },

                #[local_ref]
                input_bar_widget -> gtk::Box {},
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Create input bar controller
        let input_bar = InputBar::builder()
            .launch(())
            .forward(sender.input_sender(), |output| match output {
                InputBarOutput::SendMessage(text) => ChatPanelMsg::SendMessage(text),
            });

        // Get reference to messages container
        let messages_container = gtk::ListBox::new();
        messages_container.set_selection_mode(gtk::SelectionMode::None);
        messages_container.set_margin_top(8);
        messages_container.set_margin_bottom(8);
        messages_container.add_css_class("message-list");

        let mut model = ChatPanel {
            active_conversation: None,
            active_character: None,
            active_provider: None,
            messages: Vec::new(),
            partial_response: String::new(),
            streaming: false,
            has_providers: init,
            characters: Vec::new(),
            providers: Vec::new(),
            messages_container,
            input_bar,
        };

        let input_bar_widget = model.input_bar.widget().clone();

        let widgets = view_output!();

        // Add messages container to scrolled window
        widgets.messages_scrolled.set_child(Some(&model.messages_container));

        // Request conversations if providers exist
        if init {
            let _ = sender.output(ChatPanelOutput::LoadConversations);
        }

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            ChatPanelMsg::SetHasProviders(has_providers) => {
                self.has_providers = has_providers;
                if has_providers {
                    let _ = sender.output(ChatPanelOutput::LoadConversations);
                }
            }
            ChatPanelMsg::SetActiveProvider(provider) => {
                self.active_provider = provider;
            }
            ChatPanelMsg::LoadConversation { conversation, messages, character } => {
                self.active_conversation = Some(conversation);
                self.active_character = Some(character.clone());
                self.messages = messages;

                // Update header
                if let Some(header) = root.first_child()
                    .and_then(|c| c.downcast::<adw::HeaderBar>().ok())
                {
                    // Update character name label
                    if let Some(box_widget) = header.first_child() {
                        if let Some(name_label) = box_widget.last_child() {
                            if let Some(label) = name_label.downcast_ref::<gtk::Label>() {
                                label.set_label(&character.name);
                            }
                        }
                    }
                }

                // Rebuild message list
                self.rebuild_message_list();
            }
            ChatPanelMsg::ClearConversation => {
                self.active_conversation = None;
                self.active_character = None;
                self.messages.clear();
                self.rebuild_message_list();
            }
            ChatPanelMsg::CharactersLoaded(characters) => {
                self.characters = characters;
            }
            ChatPanelMsg::ProvidersLoaded(providers) => {
                self.providers = providers;
            }
            ChatPanelMsg::SendMessage(text) => {
                if let Some(ref conversation) = self.active_conversation {
                    if self.streaming {
                        return;
                    }

                    // Clone data we need before mutable operations
                    let conversation_id = conversation.id.clone();
                    let provider_id = conversation.provider_id.clone();

                    // Create user message
                    let user_message = Message::new(
                        conversation_id,
                        None,
                        MessageRole::User,
                        text.clone(),
                    );

                    // Save user message
                    self.messages.push(user_message.clone());
                    let _ = sender.output(ChatPanelOutput::SaveMessage(user_message));
                    self.rebuild_message_list();

                    // Start streaming
                    self.streaming = true;
                    self.partial_response.clear();
                    let _ = self.input_bar.sender().send(InputBarMsg::SetSending(true));

                    // Request API key for streaming
                    let _ = sender.output(ChatPanelOutput::RequestApiKey(provider_id));
                }
            }
            ChatPanelMsg::CancelStream => {
                self.streaming = false;
                let _ = self.input_bar.sender().send(InputBarMsg::SetSending(false));

                // Save partial response as complete message
                if let Some(ref conversation) = self.active_conversation {
                    if !self.partial_response.is_empty() {
                        let assistant_message = Message::new(
                            conversation.id.clone(),
                            None,
                            MessageRole::Assistant,
                            self.partial_response.clone(),
                        );
                        self.messages.push(assistant_message.clone());
                        let _ = sender.output(ChatPanelOutput::SaveMessage(assistant_message));
                        self.rebuild_message_list();
                    }
                }
                self.partial_response.clear();
            }
            ChatPanelMsg::ConversationStarted(conversation) => {
                self.active_conversation = Some(conversation);
            }
            ChatPanelMsg::MessagesLoaded(msgs) => {
                self.messages = msgs;
                self.rebuild_message_list();
            }
            ChatPanelMsg::MessageSaved(_msg) => {
                // Message was saved - could trigger a refresh if needed
            }
            ChatPanelMsg::LoadApiKey(_api_key) => {
                // API key loaded - streaming would start here
                sender.spawn_oneshot_command(|| {
                    ChatCmdOut::StreamComplete("This is a placeholder response. Streaming will be fully implemented in Plan 03.".to_string())
                });
            }
            ChatPanelMsg::UpdatePartialResponse(text) => {
                self.partial_response.push_str(&text);
                self.update_streaming_message();
            }
            ChatPanelMsg::StreamComplete(full_text) => {
                self.streaming = false;
                let _ = self.input_bar.sender().send(InputBarMsg::SetSending(false));

                if let Some(ref conversation) = self.active_conversation {
                    let assistant_message = Message::new(
                        conversation.id.clone(),
                        None,
                        MessageRole::Assistant,
                        full_text,
                    );
                    self.messages.push(assistant_message.clone());
                    let _ = sender.output(ChatPanelOutput::SaveMessage(assistant_message));
                    self.rebuild_message_list();
                }
                self.partial_response.clear();
            }
            ChatPanelMsg::StreamError(error) => {
                self.streaming = false;
                let _ = self.input_bar.sender().send(InputBarMsg::SetSending(false));
                let _ = sender.output(ChatPanelOutput::ShowToast(format!("Streaming error: {}", error)));

                // Save partial response if any
                if let Some(ref conversation) = self.active_conversation {
                    if !self.partial_response.is_empty() {
                        let assistant_message = Message::new(
                            conversation.id.clone(),
                            None,
                            MessageRole::Assistant,
                            self.partial_response.clone(),
                        );
                        self.messages.push(assistant_message.clone());
                        let _ = sender.output(ChatPanelOutput::SaveMessage(assistant_message));
                        self.rebuild_message_list();
                    }
                }
                self.partial_response.clear();
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            ChatCmdOut::TokenReceived(token) => {
                let _ = sender.input_sender().send(ChatPanelMsg::UpdatePartialResponse(token));
            }
            ChatCmdOut::StreamComplete(full_text) => {
                let _ = sender.input_sender().send(ChatPanelMsg::StreamComplete(full_text));
            }
            ChatCmdOut::StreamError(error) => {
                let _ = sender.input_sender().send(ChatPanelMsg::StreamError(error));
            }
        }
    }
}

impl ChatPanel {
    /// Rebuild the message list from self.messages
    fn rebuild_message_list(&mut self) {
        // Clear existing messages
        while let Some(child) = self.messages_container.first_child() {
            self.messages_container.remove(&child);
        }

        // Add each message as a MessageRow
        for message in &self.messages {
            let init = MessageRowInit {
                message: message.clone(),
                character_name: self.active_character.as_ref().map(|c| c.name.clone()),
            };
            let row = MessageRow::new(&init);
            let widget = row.widget();
            self.messages_container.append(&widget);
        }
    }

    /// Update the currently streaming message
    fn update_streaming_message(&mut self) {
        self.rebuild_message_list();
    }
}
