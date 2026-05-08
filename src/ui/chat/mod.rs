pub mod input_bar;
pub mod message_row;

use adw::prelude::*;
use futures::StreamExt as _;
use relm4::prelude::*;
use relm4::{adw, gtk};
use std::rc::Rc;

use crate::domain::character::CharacterCardV2;
use crate::domain::conversation::{Conversation, Message, MessageBranch, MessageRole};
use crate::domain::lorebook::LorebookEntry;
use crate::domain::persona::Persona;
use crate::domain::provider::ProviderConfig;
use crate::service::ai::{AiService, StreamChunk};
use crate::service::prompt::PromptEngine;
use crate::service::token_count::TokenCountService;

use self::input_bar::{InputBar, InputBarMsg, InputBarOutput};
use self::message_row::{MessageRow, MessageRowInit, MessageRowAction};

/// Chat panel component with streaming support.
/// Displays conversation messages and handles user input.
pub struct ChatPanel {
    active_conversation: Option<Conversation>,
    active_character: Option<CharacterCardV2>,
    active_provider: Option<ProviderConfig>,
    active_branch_id: Option<String>,
    active_persona: Option<Persona>,
    available_personas: Vec<Persona>,
    messages: Vec<Message>,
    branches: Vec<MessageBranch>,
    partial_response: String,
    streaming: bool,
    has_providers: bool,
    characters: Vec<CharacterCardV2>,
    providers: Vec<ProviderConfig>,
    editing_message_id: Option<String>,
    pending_user_text: Option<String>,
    lorebook_entries: Vec<LorebookEntry>,
    context_utilization: f64,
    
    stream_cancel: Option<tokio::sync::watch::Sender<bool>>,
    
    // Widget references
    messages_container: gtk::ListBox,
    input_bar: relm4::Controller<InputBar>,
    persona_button: gtk::MenuButton,
    persona_listbox: gtk::ListBox,
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
    // Branch operations
    EditMessage { message_id: String, new_content: String },
    RegenerateMessage(String),
    DeleteMessage(String),
    SwitchBranch(Option<String>),
    BranchesLoaded(Vec<MessageBranch>),
    BranchCreated(MessageBranch),
    MessageContentUpdated,
    MessageDeleted(String),
    // Context menu
    ShowEditDialog(String),
    // Branch navigation from message row click
    SwitchBranchForMessage(String),
    // Persona operations
    SetActivePersona(Option<Persona>),
    PersonasLoaded(Vec<Persona>),
    ChangeConversationPersona(String), // persona ID, empty string = none
    // Context engineering
    UpdateContextUtilization(f64), // context utilization ratio (0.0-1.0)
    LorebookEntriesLoaded(Vec<LorebookEntry>),
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
    // Branch operations
    EditMessage { message_id: String, new_content: String },
    RegenerateMessage(String),
    DeleteMessage(String),
    LoadBranches(String),
    CreateBranch(MessageBranch),
    SwitchBranch(Option<String>),
    // Persona operations
    SetConversationPersona { conversation_id: String, persona_id: Option<String> },
    // Context engineering
    LoadLorebookEntries(String), // character_id — request lorebook entries from DB
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
                    adw::Avatar {
                        set_size: 32,
                        #[watch]
                        set_text: model.active_character.as_ref().map(|c| c.name.as_str()),
                    },

                    #[name(character_name_label)]
                    gtk::Label {
                        #[watch]
                        set_label: &model.active_character.as_ref()
                            .map(|c| c.name.clone())
                            .unwrap_or_default(),
                    },
                },

                #[name(stop_button)]
                pack_end = &gtk::Button {
                    set_icon_name: "stop",
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

        // Create persona selector widgets
        let persona_listbox = gtk::ListBox::new();
        persona_listbox.set_selection_mode(gtk::SelectionMode::None);

        let persona_button = gtk::MenuButton::new();
        persona_button.set_label("as User");
        persona_button.add_css_class("flat");
        persona_button.add_css_class("caption");
        persona_button.add_css_class("dim-label");
        persona_button.set_has_frame(false);
        persona_button.set_visible(false);

        let model = ChatPanel {
            active_conversation: None,
            active_character: None,
            active_provider: None,
            active_branch_id: None,
            active_persona: None,
            available_personas: Vec::new(),
            messages: Vec::new(),
            branches: Vec::new(),
            partial_response: String::new(),
            streaming: false,
            has_providers: init,
            characters: Vec::new(),
            providers: Vec::new(),
            editing_message_id: None,
            pending_user_text: None,
            lorebook_entries: Vec::new(),
            context_utilization: 0.0,
            stream_cancel: None,
            messages_container,
            input_bar,
            persona_button: persona_button.clone(),
            persona_listbox: persona_listbox.clone(),
        };

        let input_bar_widget = model.input_bar.widget().clone();

        let widgets = view_output!();

        // Add messages container to scrolled window
        widgets.messages_scrolled.set_child(Some(&model.messages_container));

        // Set up persona popover and add button to header
        let persona_popover = gtk::Popover::new();
        persona_popover.set_child(Some(&persona_listbox));
        persona_button.set_popover(Some(&persona_popover));
        widgets.header_box.append(&persona_button);

        // Populate initial "User (No Persona)" row in popover
        {
            let none_row = gtk::ListBoxRow::new();
            let label = gtk::Label::new(Some("User (No Persona)"));
            label.set_xalign(0.0);
            label.set_margin_start(8);
            label.set_margin_end(8);
            label.set_margin_top(4);
            label.set_margin_bottom(4);
            none_row.set_child(Some(&label));
            persona_listbox.append(&none_row);
        }

        // Connect row-activated signal on persona listbox
        let sender_clone = sender.clone();
        persona_listbox.connect_row_activated(move |_, row| {
            let index = row.index();
            if index == 0 {
                let _ = sender_clone.input_sender().send(ChatPanelMsg::ChangeConversationPersona(String::new()));
            }
            // Other rows are handled after PersonasLoaded rebuilds the listbox
        });

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
                self.active_conversation = Some(conversation.clone());
                self.active_character = Some(character.clone());
                self.messages = messages;

                // Look up the provider for this conversation
                self.active_provider = self.providers.iter()
                    .find(|p| p.id == conversation.provider_id)
                    .cloned();

                // Show persona button since we have a conversation
                self.persona_button.set_visible(true);

                // Request lorebook entries for this character per D-08
                let _ = sender.output(ChatPanelOutput::LoadLorebookEntries(character.id.clone()));

                // Rebuild message list
                self.rebuild_message_list(&sender);
            }
            ChatPanelMsg::ClearConversation => {
                self.active_conversation = None;
                self.active_character = None;
                self.active_persona = None;
                self.messages.clear();
                self.lorebook_entries.clear();
                self.context_utilization = 0.0;
                self.persona_button.set_visible(false);
                self.update_persona_display();
                let _ = self.input_bar.sender().send(InputBarMsg::UpdateContextUtilization {
                    pct: 0.0,
                    truncated: 0,
                    tokens_used: 0,
                    context_window: 0,
                });
                self.rebuild_message_list(&sender);
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
                    self.rebuild_message_list(&sender);

                    // Start streaming
                    self.streaming = true;
                    self.partial_response.clear();
                    self.pending_user_text = Some(text);
                    let _ = self.input_bar.sender().send(InputBarMsg::SetSending(true));

                    // Request API key for streaming
                    let _ = sender.output(ChatPanelOutput::RequestApiKey(provider_id));
                }
            }
            ChatPanelMsg::CancelStream => {
                // Signal the background stream task to stop
                if let Some(tx) = self.stream_cancel.take() {
                    let _ = tx.send(true);
                }
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
                        self.partial_response.clear();
                        self.rebuild_message_list(&sender);
                    }
                }
            }
            ChatPanelMsg::ConversationStarted(conversation) => {
                self.active_conversation = Some(conversation);
            }
            ChatPanelMsg::MessagesLoaded(msgs) => {
                self.messages = msgs;
                self.rebuild_message_list(&sender);
            }
            ChatPanelMsg::MessageSaved(_msg) => {
                // Message was saved - could trigger a refresh if needed
            }
            ChatPanelMsg::LoadApiKey(api_key) => {
                let Some(ref provider) = self.active_provider else {
                    let _ = sender.spawn_oneshot_command(|| {
                        ChatCmdOut::StreamError("No active provider configured.".to_string())
                    });
                    return;
                };
                let Some(ref character) = self.active_character else {
                    let _ = sender.spawn_oneshot_command(|| {
                        ChatCmdOut::StreamError("No character loaded.".to_string())
                    });
                    return;
                };

                // Assemble prompt with budget-aware context engineering.
                // PromptEngine injects persona block, lorebook entries, and applies
                // compression/truncation based on context window budget.
                let token_service = TokenCountService::new();
                let context_window = token_service.resolve_context_window_size(provider);
                let assembled = PromptEngine::assemble_with_budget(
                    character,
                    self.active_persona.as_ref(),
                    &self.messages,
                    &self.lorebook_entries,
                    &token_service,
                    context_window,
                );

                // Update context utilization indicator in InputBar per D-14, D-15, D-16
                let tokens_used = (assembled.context_utilization * context_window as f64) as usize;
                let _ = self.input_bar.sender().send(InputBarMsg::UpdateContextUtilization {
                    pct: assembled.context_utilization,
                    truncated: assembled.truncated_messages,
                    tokens_used,
                    context_window: context_window as usize,
                });
                self.context_utilization = assembled.context_utilization;

                let system_prompt = assembled.preamble;
                let messages = self.messages.clone();
                let provider_cfg = provider.clone();
                let key = api_key;

                // Create cancellation channel
                let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
                self.stream_cancel = Some(cancel_tx);

                sender.command(move |out, _shutdown| async move {
                    let mut stream = AiService::stream_chat(
                        &provider_cfg,
                        &system_prompt,
                        &messages,
                        &key,
                    );
                    while let Some(chunk) = stream.next().await {
                        // Check cancellation between tokens
                        if *cancel_rx.borrow() {
                            return; // Silently drop — parent already saved partial
                        }
                        match chunk {
                            StreamChunk::Token(token) => {
                                if out.send(ChatCmdOut::TokenReceived(token)).is_err() {
                                    break;
                                }
                            }
                            StreamChunk::Complete(full_text) => {
                                let _ = out.send(ChatCmdOut::StreamComplete(full_text));
                                return;
                            }
                            StreamChunk::Error(err) => {
                                let _ = out.send(ChatCmdOut::StreamError(err));
                                return;
                            }
                        }
                    }
                });
            }
            ChatPanelMsg::UpdatePartialResponse(text) => {
                self.partial_response.push_str(&text);
                self.update_streaming_message(&sender);
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
                }
                self.partial_response.clear();
                self.rebuild_message_list(&sender);
            }
            ChatPanelMsg::StreamError(error) => {
                self.streaming = false;
                let _ = self.input_bar.sender().send(InputBarMsg::SetSending(false));
                let _ = sender.output(ChatPanelOutput::ShowToast(format!("Streaming error: {}", error)));
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
                    }
                }
                self.partial_response.clear();
                self.rebuild_message_list(&sender);
            }
            // Branch operations
            ChatPanelMsg::ShowEditDialog(message_id) => {
                // Find the message
                if let Some(msg) = self.messages.iter().find(|m| m.id == message_id).cloned() {
                    let dialog = adw::AlertDialog::new(
                        Some("Edit Message"),
                        Some("Edit this message. The original will be kept as a branch."),
                    );
                    dialog.add_response("cancel", "Cancel");
                    dialog.add_response("save", "Save");
                    dialog.set_response_appearance("save", adw::ResponseAppearance::Suggested);
                    dialog.set_default_response(Some("save"));
                    dialog.set_close_response("cancel");

                    // Content entry
                    let entry = gtk::Entry::new();
                    entry.set_text(&msg.content);
                    entry.set_activates_default(true);
                    dialog.set_extra_child(Some(&entry));

                    let sender_clone = sender.clone();
                    let msg_id = message_id.clone();
                    dialog.connect_response(None, move |_, response| {
                        if response == "save" {
                            let new_content = entry.text().to_string();
                            if !new_content.is_empty() {
                                let _ = sender_clone.input_sender().send(ChatPanelMsg::EditMessage {
                                    message_id: msg_id.clone(),
                                    new_content,
                                });
                            }
                        }
                    });
                    dialog.present(None::<&gtk::Widget>);
                }
            }
            ChatPanelMsg::EditMessage { message_id, new_content } => {
                // Send edit to parent (AppModel) to update in DB
                let _ = sender.output(ChatPanelOutput::EditMessage { message_id, new_content });
            }
            ChatPanelMsg::RegenerateMessage(message_id) => {
                // Trigger regeneration for the message
                let _ = sender.output(ChatPanelOutput::RegenerateMessage(message_id));
            }
            ChatPanelMsg::DeleteMessage(message_id) => {
                // Clone message_id for output before using it in retain
                let id_clone = message_id.clone();
                // Delete the message
                let _ = sender.output(ChatPanelOutput::DeleteMessage(message_id));
                // Remove from local messages
                self.messages.retain(|m| m.id != id_clone);
                self.rebuild_message_list(&sender);
            }
            ChatPanelMsg::MessageDeleted(message_id) => {
                // Confirmation from parent after DB deletion
                self.messages.retain(|m| m.id != message_id);
                self.rebuild_message_list(&sender);
            }
            ChatPanelMsg::BranchesLoaded(branches) => {
                self.branches = branches;
                self.rebuild_message_list(&sender);
            }
            ChatPanelMsg::BranchCreated(branch) => {
                self.branches.push(branch);
            }
            ChatPanelMsg::MessageContentUpdated => {
                // Message content was updated in DB
                // Reload messages to reflect changes
                if let Some(ref conversation) = self.active_conversation {
                    let _ = sender.output(ChatPanelOutput::LoadMessages {
                        conversation_id: conversation.id.clone(),
                        branch_id: self.active_branch_id.clone(),
                    });
                }
            }
            ChatPanelMsg::SwitchBranch(branch_id) => {
                self.active_branch_id = branch_id.clone();
                if let Some(ref conversation) = self.active_conversation {
                    let _ = sender.output(ChatPanelOutput::LoadMessages {
                        conversation_id: conversation.id.clone(),
                        branch_id,
                    });
                }
            }
            ChatPanelMsg::SwitchBranchForMessage(message_id) => {
                // Simple cycle: root (None) <-> first branch for this message
                let sibling = self.branches.iter()
                    .find(|b| b.parent_message_id.as_ref() == Some(&message_id));
                let new_branch = match self.active_branch_id {
                    Some(ref active) => {
                        if sibling.map(|b| &b.id) == Some(active) {
                            None // cycle back to root
                        } else {
                            sibling.map(|b| b.id.clone())
                        }
                    }
                    None => sibling.map(|b| b.id.clone()),
                };
                self.active_branch_id = new_branch.clone();
                if let Some(ref conversation) = self.active_conversation {
                    let _ = sender.output(ChatPanelOutput::LoadMessages {
                        conversation_id: conversation.id.clone(),
                        branch_id: new_branch,
                    });
                }
            }
            // Persona operations
            ChatPanelMsg::SetActivePersona(persona) => {
                self.active_persona = persona;
                self.update_persona_display();
                // Rebuild message list to update user persona labels
                self.rebuild_message_list(&sender);
            }
            ChatPanelMsg::PersonasLoaded(personas) => {
                self.available_personas = personas;
                self.rebuild_persona_popover(&sender);
            }
            ChatPanelMsg::ChangeConversationPersona(persona_id) => {
                // Update local state
                if persona_id.is_empty() {
                    self.active_persona = None;
                } else {
                    self.active_persona = self.available_personas.iter()
                        .find(|p| p.id == persona_id)
                        .cloned();
                }
                self.update_persona_display();

                // Emit to AppModel to update DB
                if let Some(ref conv) = self.active_conversation {
                    let _ = sender.output(ChatPanelOutput::SetConversationPersona {
                        conversation_id: conv.id.clone(),
                        persona_id: if persona_id.is_empty() { None } else { Some(persona_id) },
                    });
                }

                // Rebuild message list to update user persona labels
                self.rebuild_message_list(&sender);
            }
            ChatPanelMsg::UpdateContextUtilization(pct) => {
                self.context_utilization = pct;
            }
            ChatPanelMsg::LorebookEntriesLoaded(entries) => {
                self.lorebook_entries = entries;
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        // Ignore stale outputs if streaming was cancelled
        if !self.streaming {
            return;
        }
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
    fn rebuild_message_list(&mut self, sender: &ComponentSender<ChatPanel>) {
        // Clear existing messages
        while let Some(child) = self.messages_container.first_child() {
            self.messages_container.remove(&child);
        }

        // Add each persisted message as a MessageRow
        for message in &self.messages {
            let has_siblings = self.branches.iter().any(|b| {
                b.parent_message_id.as_ref() == Some(&message.id)
            });

            let init = MessageRowInit {
                message: message.clone(),
                character_name: self.active_character.as_ref().map(|c| c.name.clone()),
                user_persona_name: self.active_persona.as_ref().map(|p| p.name.clone()),
                has_siblings,
            };
            let row = MessageRow::new(&init);
            let sender_clone = sender.clone();
            let widget = row.widget(Rc::new(move |action| {
                let msg = match action {
                    MessageRowAction::Edit(id) => ChatPanelMsg::ShowEditDialog(id),
                    MessageRowAction::Regenerate(id) => ChatPanelMsg::RegenerateMessage(id),
                    MessageRowAction::Delete(id) => ChatPanelMsg::DeleteMessage(id),
                    MessageRowAction::NavigateBranches(id) => ChatPanelMsg::SwitchBranchForMessage(id),
                };
                let _ = sender_clone.input_sender().send(msg);
            }));
            self.messages_container.append(&widget);
        }

        // Append a transient streaming message row when actively streaming
        if self.streaming && !self.partial_response.is_empty() {
            let streaming_msg = Message {
                id: "streaming".to_string(),
                conversation_id: self.active_conversation.as_ref().map(|c| c.id.clone()).unwrap_or_default(),
                branch_id: self.active_branch_id.clone(),
                role: MessageRole::Assistant,
                content: self.partial_response.clone(),
                timestamp: chrono::Local::now().naive_local(),
                sort_order: self.messages.len() as i32,
            };
            let init = MessageRowInit {
                message: streaming_msg,
                character_name: self.active_character.as_ref().map(|c| c.name.clone()),
                user_persona_name: self.active_persona.as_ref().map(|p| p.name.clone()),
                has_siblings: false,
            };
            let row = MessageRow::new(&init);
            let widget = row.widget(Rc::new(|_| {}));
            self.messages_container.append(&widget);
        }
    }

    /// Update the currently streaming message
    pub fn update_streaming_message(&mut self, sender: &ComponentSender<ChatPanel>) {
        self.rebuild_message_list(sender);
    }

    /// Update the persona button label to reflect the active persona.
    fn update_persona_display(&self) {
        let label = self.active_persona.as_ref()
            .map(|p| format!("as {}", p.name))
            .unwrap_or_else(|| "as User".to_string());
        self.persona_button.set_label(&label);
    }

    /// Rebuild the persona selection popover with the current persona list.
    fn rebuild_persona_popover(&self, sender: &ComponentSender<ChatPanel>) {
        // Clear existing rows
        while let Some(child) = self.persona_listbox.first_child() {
            self.persona_listbox.remove(&child);
        }

        // "User (No Persona)" row (index 0)
        let none_row = gtk::ListBoxRow::new();
        let none_label = gtk::Label::new(Some("User (No Persona)"));
        none_label.set_xalign(0.0);
        none_label.set_margin_start(8);
        none_label.set_margin_end(8);
        none_label.set_margin_top(4);
        none_label.set_margin_bottom(4);
        none_row.set_child(Some(&none_label));
        self.persona_listbox.append(&none_row);

        // Persona rows (index 1..N)
        for persona in &self.available_personas {
            let row = gtk::ListBoxRow::new();
            let label = gtk::Label::new(Some(&persona.name));
            label.set_xalign(0.0);
            label.set_margin_start(8);
            label.set_margin_end(8);
            label.set_margin_top(4);
            label.set_margin_bottom(4);
            row.set_child(Some(&label));
            self.persona_listbox.append(&row);
        }

        // Disconnect old row-activated handlers and connect new one
        // We store persona IDs by index for lookup in the signal handler
        let persona_ids: Vec<String> = std::iter::once(String::new())
            .chain(self.available_personas.iter().map(|p| p.id.clone()))
            .collect();
        let sender_clone = sender.clone();
        self.persona_listbox.connect_row_activated(move |_, row| {
            let index = row.index() as usize;
            if let Some(id) = persona_ids.get(index) {
                let _ = sender_clone.input_sender().send(ChatPanelMsg::ChangeConversationPersona(id.clone()));
            }
        });
    }
}
