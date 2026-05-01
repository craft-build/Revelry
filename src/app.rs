use adw::prelude::*;
use relm4::prelude::*;
use relm4::{
    adw, gtk, ComponentParts, ComponentSender, WorkerController,
};

use crate::db::{DbInput, DbOutput, DbService};
use crate::domain::character::CharacterCardV2;
use crate::domain::conversation::{Conversation, Message};
use crate::domain::provider::ProviderConfig;
use crate::secrets::KeyringStore;
use crate::service::card_io;
use crate::ui::settings::SettingsPanelMsg;
use crate::ui::{
    CharacterEditor, CharacterEditorInit, CharacterEditorOutput, CharactersPanel,
    CharactersPanelMsg, CharactersPanelOutput, ChatPanel, ChatPanelMsg, ChatPanelOutput,
    SettingsPanel, SettingsPanelOutput,
};

/// Active view in the main content area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    Chat,
    Characters,
    Settings,
}

/// Messages handled by the root AppModel.
#[derive(Debug)]
pub enum AppMsg {
    SwitchView(ActiveView),
    ToggleSidebar,
    DbResult(DbOutput),
    SettingsOutput(SettingsPanelOutput),
    CharactersOutput(CharactersPanelOutput),
    ChatOutput(ChatPanelOutput),
    EditorOutput(CharacterEditorOutput),
    ShowToast(String),
    Quit,
}

/// Root application model holding global state and child panel controllers.
pub struct AppModel {
    active_view: ActiveView,
    sidebar_visible: bool,
    has_providers: bool,
    characters: Vec<CharacterCardV2>,
    providers: Vec<ProviderConfig>,
    conversations: Vec<Conversation>,
    conversation_messages: Vec<Message>,
    db_worker: WorkerController<DbService>,
    chat_panel: relm4::Controller<ChatPanel>,
    #[allow(dead_code)] // Keep alive to run component event loop
    characters_panel: relm4::Controller<CharactersPanel>,
    settings_panel: relm4::Controller<SettingsPanel>,
    character_editor: Option<relm4::Controller<CharacterEditor>>,
    view_stack: adw::ViewStack,
    toast_overlay: adw::ToastOverlay,
}

#[relm4::component(pub)]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = AppMsg;
    type Output = ();

    view! {
        #[root]
        adw::ApplicationWindow {
            set_title: Some("Revelry"),
            set_default_width: 1024,
            set_default_height: 768,

            add_breakpoint = adw::Breakpoint::new(
                adw::BreakpointCondition::new_length(
                    adw::BreakpointConditionLengthType::MaxWidth,
                    720.0,
                    adw::LengthUnit::Pt,
                )
            ) {
                add_setter: (&split_view, "collapsed", Some(&gtk::glib::Value::from(true))),
            },

            #[name(toast_overlay)]
            adw::ToastOverlay {
                #[name(split_view)]
                adw::OverlaySplitView {
                    set_sidebar_width_fraction: 0.30,
                    set_min_sidebar_width: 280.0,
                    set_max_sidebar_width: 360.0,
                    set_collapsed: false,

                    #[watch]
                    set_show_sidebar: model.sidebar_visible,

                    #[wrap(Some)]
                    set_sidebar = &adw::ToolbarView {
                        add_top_bar = &adw::HeaderBar {
                            set_show_title: false,
                        },

                        #[wrap(Some)]
                        set_content = &gtk::Stack {
                            set_hhomogeneous: true,
                            set_vhomogeneous: true,

                            #[name(conversation_list)]
                            gtk::ListBox {
                                add_css_class: "navigation-sidebar",
                                set_valign: gtk::Align::Fill,
                                set_vexpand: true,
                            },

                            #[name(sidebar_empty)]
                            adw::StatusPage {
                                set_icon_name: Some("chat"),
                                set_title: "No Conversations Yet",
                                set_description: Some("Start by configuring a provider in Settings."),
                            },
                        },
                    },

                    #[wrap(Some)]
                    set_content = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        adw::HeaderBar {
                            #[name(sidebar_toggle)]
                        pack_start = &gtk::ToggleButton {
                                set_icon_name: "menu",
                                set_tooltip_text: Some("Toggle Sidebar"),
                                connect_clicked => AppMsg::ToggleSidebar,
                            },

                            #[name(view_switcher)]
                            pack_start = &adw::ViewSwitcher {},
                        },

                        #[name(view_stack)]
                        adw::ViewStack {
                            set_vexpand: true,
                            set_hexpand: true,
                        },
                    },
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Initialize database path from platform data directory
        let data_dir = dirs::data_dir()
            .expect("Could not determine platform data directory")
            .join("revelry");
        std::fs::create_dir_all(&data_dir).expect("Could not create data directory");
        let db_path = data_dir.join("revelry.db");

        // Launch DbService Worker on background thread
        let db_worker = DbService::builder()
            .detach_worker(db_path)
            .forward(sender.input_sender(), AppMsg::DbResult);

        // Launch child panel components
        let chat_panel = ChatPanel::builder()
            .launch(false)
            .forward(sender.input_sender(), AppMsg::ChatOutput);

        let characters_panel = CharactersPanel::builder()
            .launch(())
            .forward(sender.input_sender(), AppMsg::CharactersOutput);

        // SettingsPanel init takes () — all DB operations go through output messages
        let settings_panel = SettingsPanel::builder()
            .launch(())
            .forward(sender.input_sender(), AppMsg::SettingsOutput);

        // Get widget references before moving controllers into model.
        // Clone the GObject reference so it's independent of the Controller borrow.
        let chat_widget = chat_panel.widget().clone();
        let characters_widget = characters_panel.widget().clone();
        let settings_widget = settings_panel.widget().clone();

        let mut model = AppModel {
            active_view: ActiveView::Chat,
            sidebar_visible: true,
            has_providers: false,
            characters: Vec::new(),
            providers: Vec::new(),
            conversations: Vec::new(),
            conversation_messages: Vec::new(),
            db_worker,
            chat_panel,
            characters_panel,
            settings_panel,
            character_editor: None,
            view_stack: adw::ViewStack::new(),
            toast_overlay: adw::ToastOverlay::new(),
        };

        let widgets = view_output!();

        // Replace placeholder with the actual ViewStack from the view
        model.view_stack = widgets.view_stack.clone();
        model.toast_overlay = widgets.toast_overlay.clone();

        // Wire up the ViewSwitcher to the ViewStack
        widgets.view_switcher.set_stack(Some(&widgets.view_stack));

        // Bind sidebar toggle button to split_view's show-sidebar property.
        // Using a GObject property binding avoids the "Broken accounting of active state"
        // GTK warnings that occur when #[watch] set_active fights with GTK's own state tracking
        // during window activation/deactivation (e.g. moving the window).
        widgets.sidebar_toggle.bind_property("active", &widgets.split_view, "show-sidebar")
            .bidirectional()
            .sync_create()
            .build();

        // Add child panels to ViewStack with names and page metadata
        let chat_page = widgets.view_stack.add_named(&chat_widget, Some("chat"));
        chat_page.set_title(Some("Chat"));
        chat_page.set_icon_name(Some("chat"));

        let characters_page = widgets.view_stack.add_named(&characters_widget, Some("characters"));
        characters_page.set_title(Some("Characters"));
        characters_page.set_icon_name(Some("list"));

        let settings_page = widgets.view_stack.add_named(&settings_widget, Some("settings"));
        settings_page.set_title(Some("Settings"));
        settings_page.set_icon_name(Some("settings"));

        // Set up keyboard shortcuts via EventControllerKey
        // Also set the initial visible page — must happen AFTER pages are added
        model.sync_view_stack();
        let sender_clone = sender.input_sender().clone();
        root.add_controller({
            let controller = gtk::EventControllerKey::new();
            controller.connect_key_pressed(move |_, key, _code, modifier| {
                // F9 → Toggle sidebar
                if key == gtk::gdk::Key::F9 && modifier.is_empty() {
                    let _ = sender_clone.send(AppMsg::ToggleSidebar);
                    return gtk::glib::Propagation::Stop;
                }

                // Platform-aware modifier: Cmd on macOS, Ctrl elsewhere
                let modifier_ctrl = if cfg!(target_os = "macos") {
                    modifier.contains(gtk::gdk::ModifierType::META_MASK)
                } else {
                    modifier.contains(gtk::gdk::ModifierType::CONTROL_MASK)
                };

                if modifier_ctrl {
                    match key {
                        gtk::gdk::Key::_1 | gtk::gdk::Key::KP_1 => {
                            let _ = sender_clone.send(AppMsg::SwitchView(ActiveView::Chat));
                            return gtk::glib::Propagation::Stop;
                        }
                        gtk::gdk::Key::_2 | gtk::gdk::Key::KP_2 => {
                            let _ = sender_clone.send(AppMsg::SwitchView(ActiveView::Characters));
                            return gtk::glib::Propagation::Stop;
                        }
                        gtk::gdk::Key::_3 | gtk::gdk::Key::KP_3 => {
                            let _ = sender_clone.send(AppMsg::SwitchView(ActiveView::Settings));
                            return gtk::glib::Propagation::Stop;
                        }
                        gtk::gdk::Key::comma => {
                            let _ = sender_clone.send(AppMsg::SwitchView(ActiveView::Settings));
                            return gtk::glib::Propagation::Stop;
                        }
                        gtk::gdk::Key::q => {
                            let _ = sender_clone.send(AppMsg::Quit);
                            return gtk::glib::Propagation::Stop;
                        }
                        _ => {}
                    }
                }

                // Ctrl+Shift+N → New character
                if modifier_ctrl
                    && modifier.contains(gtk::gdk::ModifierType::SHIFT_MASK)
                    && (key == gtk::gdk::Key::n || key == gtk::gdk::Key::N)
                {
                    let _ = sender_clone.send(AppMsg::CharactersOutput(
                        CharactersPanelOutput::EditCharacter(String::new()),
                    ));
                    return gtk::glib::Propagation::Stop;
                }

                gtk::glib::Propagation::Proceed
            });
            controller
        });

        // Query providers, characters, and conversations on startup
        model.db_worker.sender().send(DbInput::LoadProviders).unwrap();
        model.db_worker.sender().send(DbInput::LoadCharacters).unwrap();
        model.db_worker.sender().send(DbInput::LoadConversations).unwrap();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            AppMsg::SwitchView(view) => {
                self.active_view = view;
                self.sync_view_stack();
            }
            AppMsg::ToggleSidebar => {
                self.sidebar_visible = !self.sidebar_visible;
            }
            AppMsg::DbResult(output) => match output {
                DbOutput::ProvidersLoaded(providers) => {
                    let has_providers = !providers.is_empty();
                    self.has_providers = has_providers;
                    self.providers = providers.clone();

                    // Per D-04 and D-12: show Settings if no providers, Chat otherwise
                    if !has_providers {
                        self.active_view = ActiveView::Settings;
                    }
                    self.sync_view_stack();

                    // Forward provider state to ChatPanel
                    let _ = self.chat_panel.sender().send(ChatPanelMsg::SetHasProviders(has_providers));

                    // Forward provider list to SettingsPanel
                    let _ = self.settings_panel.sender().send(SettingsPanelMsg::ProvidersLoaded(providers));
                }
                DbOutput::ProviderSaved(_) => {
                    // Reload providers after save
                    self.db_worker.sender().send(DbInput::LoadProviders).unwrap();
                    // Update has_providers flag
                    self.has_providers = true;
                    let _ = self.chat_panel.sender().send(ChatPanelMsg::SetHasProviders(true));
                }
                DbOutput::ProviderDeleted(_) => {
                    // Reload providers after delete
                    self.db_worker.sender().send(DbInput::LoadProviders).unwrap();
                }
                DbOutput::CharactersLoaded(chars) => {
                    self.characters = chars.clone();
                    let _ = self.characters_panel.sender().send(CharactersPanelMsg::CharactersLoaded(chars));
                }
                DbOutput::CharacterSaved(_) => {
                    // Reload characters after save
                    self.db_worker.sender().send(DbInput::LoadCharacters).unwrap();
                    self.show_toast("Character saved");
                }
                DbOutput::CharacterDeleted(_) => {
                    // Reload characters after delete
                    self.db_worker.sender().send(DbInput::LoadCharacters).unwrap();
                    self.show_toast("Character deleted");
                }
                DbOutput::CharactersSearched(chars) => {
                    let _ = self.characters_panel.sender().send(CharactersPanelMsg::CharactersLoaded(chars));
                }
                DbOutput::ConversationsLoaded(convs) => {
                    self.conversations = convs;
                    // Forward providers to ChatPanel for conversation creation
                    let providers = self.providers.clone();
                    let _ = self.chat_panel.sender().send(ChatPanelMsg::ProvidersLoaded(providers));
                }
                DbOutput::ConversationStarted(conv) => {
                    self.conversations.push(conv.clone());
                    // Reload conversations to get the updated list
                    self.db_worker.sender().send(DbInput::LoadConversations).unwrap();
                    // Forward to chat panel
                    let _ = self.chat_panel.sender().send(ChatPanelMsg::ConversationStarted(conv));
                }
                DbOutput::MessagesLoaded(msgs) => {
                    self.conversation_messages = msgs.clone();
                    // Forward to chat panel
                    let _ = self.chat_panel.sender().send(ChatPanelMsg::MessagesLoaded(msgs));
                }
                DbOutput::MessageSaved(msg) => {
                    let _ = self.chat_panel.sender().send(ChatPanelMsg::MessageSaved(msg));
                }
                DbOutput::ConversationDeleted(id) => {
                    self.conversations.retain(|c| c.id != id);
                    self.db_worker.sender().send(DbInput::LoadConversations).unwrap();
                }
                DbOutput::Error(e) => {
                    tracing::error!("Database error: {}", e);
                    self.show_toast(&format!("Error: {}", e));
                }
                _ => {}
            },
            AppMsg::SettingsOutput(output) => match output {
                SettingsPanelOutput::SaveProvider { config, api_key } => {
                    // Save API key to keyring on background thread
                    if config.provider_type.requires_api_key() && !api_key.is_empty() {
                        let id = config.id.clone();
                        let key = api_key.clone();
                        sender.spawn_oneshot_command(move || {
                            match KeyringStore::save_key(&id, &key) {
                                Ok(()) => tracing::info!("API key saved to keychain for provider {}", id),
                                Err(e) => tracing::error!("Failed to save key to keyring: {}", e),
                            }
                        });
                    }
                    // Save provider config to DB via worker
                    self.db_worker.sender().send(DbInput::SaveProvider(config)).unwrap();
                }
                SettingsPanelOutput::DeleteProvider(id) => {
                    // Delete provider from DB via worker
                    self.db_worker.sender().send(DbInput::DeleteProvider(id)).unwrap();
                }
                SettingsPanelOutput::DeleteKey(id) => {
                    // Delete API key from keyring on background thread
                    let id_clone = id.clone();
                    sender.spawn_oneshot_command(move || {
                        match KeyringStore::delete_key(&id_clone) {
                            Ok(()) => tracing::info!("API key deleted from keychain for provider {}", id),
                            Err(e) => tracing::error!("Failed to delete key from keyring: {}", e),
                        }
                    });
                }
                SettingsPanelOutput::TestProvider(id) => {
                    // Load API key from keyring and provider config, then run test
                    let id_clone = id.clone();
                    let settings_sender = self.settings_panel.sender().clone();
                    sender.spawn_oneshot_command(move || {
                        let api_key = KeyringStore::load_key(&id_clone)
                            .unwrap_or_default();
                        let _ = settings_sender.send(SettingsPanelMsg::KeyLoaded(id_clone, api_key));
                    });
                }
            },
            AppMsg::CharactersOutput(output) => match output {
                CharactersPanelOutput::SaveCharacter(character) => {
                    self.db_worker.sender().send(DbInput::SaveCharacter(character)).unwrap();
                }
                CharactersPanelOutput::DeleteCharacter(id) => {
                    self.db_worker.sender().send(DbInput::DeleteCharacter(id.clone())).unwrap();
                    // Delete avatar file on background thread
                    let character = self.characters.iter().find(|c| c.id == id);
                    if let Some(Some(path)) = character.map(|c| c.avatar_path.as_ref()) {
                        let path = path.clone();
                        sender.spawn_oneshot_command(move || {
                            if let Err(e) = std::fs::remove_file(&path) {
                                tracing::warn!("Failed to delete avatar file {}: {}", path, e);
                            }
                        });
                    }
                }
                CharactersPanelOutput::EditCharacter(id) => {
                    let editor_init = if id.is_empty() {
                        CharacterEditorInit::New
                    } else {
                        let character = self.characters.iter().find(|c| c.id == id).cloned();
                        match character {
                            Some(c) => CharacterEditorInit::Edit(c),
                            None => {
                                self.show_toast("Character not found");
                                return;
                            }
                        }
                    };

                    let editor = CharacterEditor::builder()
                        .launch(editor_init)
                        .forward(sender.input_sender(), AppMsg::EditorOutput);

                    editor.widget().present();
                    self.character_editor = Some(editor);
                }
                CharactersPanelOutput::RequestImport => {
                    let filter = gtk::FileFilter::new();
                    filter.set_name(Some("Character Cards"));
                    filter.add_mime_type("image/png");
                    filter.add_pattern("*.json");

                    let filters = gtk::gio::ListStore::new::<gtk::FileFilter>();
                    filters.append(&filter);

                    let dialog = gtk::FileDialog::builder()
                        .title("Import Character")
                        .modal(true)
                        .accept_label("Import")
                        .filters(&filters)
                        .default_filter(&filter)
                        .build();

                    let sender_clone = sender.input_sender().clone();
                    let db_sender = self.db_worker.sender().clone();
                    let parent = self.toast_overlay.root().and_downcast::<gtk::Window>();
                    dialog.open(
                        parent.as_ref(),
                        gtk::gio::Cancellable::NONE,
                        move |result| {
                            if let Ok(file) = result {
                                if let Some(path) = file.path() {
                                    let path_str = path.to_string_lossy().to_string();
                                    let db_sender = db_sender.clone();
                                    let sender_clone = sender_clone.clone();

                                    // Run import on a background thread
                                    std::thread::spawn(move || {
                                        let result = (|| -> Result<CharacterCardV2, String> {
                                            let ext = std::path::Path::new(&path_str)
                                                .extension()
                                                .and_then(|e| e.to_str())
                                                .unwrap_or("")
                                                .to_lowercase();

                                            let bytes = std::fs::read(&path_str)
                                                .map_err(|e| format!("Failed to read file: {}", e))?;

                                            let mut character = if ext == "json" {
                                                let json_str = String::from_utf8(bytes)
                                                    .map_err(|e| format!("Invalid UTF-8: {}", e))?;
                                                card_io::import_from_json(&json_str)
                                                    .map_err(|e| format!("JSON import failed: {}", e))?
                                            } else {
                                                // Try PNG import — the PNG itself is the avatar
                                                let (ch, avatar_bytes) = card_io::import_from_png(&bytes)
                                                    .map_err(|e| format!("PNG import failed: {}", e))?;
                                                if let Some(png_data) = avatar_bytes {
                                                    // Save avatar to data dir
                                                    let avatar_dir = dirs::data_dir()
                                                        .expect("data dir")
                                                        .join("revelry")
                                                        .join("avatars");
                                                    std::fs::create_dir_all(&avatar_dir)
                                                        .map_err(|e| format!("Failed to create avatars dir: {}", e))?;
                                                    let avatar_path = avatar_dir.join(format!("{}.png", ch.id));
                                                    std::fs::write(&avatar_path, &png_data)
                                                        .map_err(|e| format!("Failed to save avatar: {}", e))?;
                                                    let mut ch = ch;
                                                    ch.avatar_path = Some(avatar_path.to_string_lossy().to_string());
                                                    ch
                                                } else {
                                                    ch
                                                }
                                            };

                                            Ok(character)
                                        })();

                                        match result {
                                            Ok(character) => {
                                                let _ = db_sender.send(DbInput::SaveCharacter(character));
                                                let _ = db_sender.send(DbInput::LoadCharacters);
                                            }
                                            Err(msg) => {
                                                let _ = sender_clone.send(AppMsg::ShowToast(msg));
                                            }
                                        }
                                    });
                                }
                            }
                        },
                    );
                }
                CharactersPanelOutput::RequestExport(id) => {
                    let character = self.characters.iter().find(|c| c.id == id).cloned();
                    let Some(character) = character else {
                        self.show_toast("Character not found for export");
                        return;
                    };

                    // Suggest a filename
                    let slug: String = character
                        .name
                        .to_lowercase()
                        .chars()
                        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
                        .collect();

                    let png_filter = gtk::FileFilter::new();
                    png_filter.set_name(Some("PNG (with embedded data)"));
                    png_filter.add_mime_type("image/png");

                    let json_filter = gtk::FileFilter::new();
                    json_filter.set_name(Some("JSON"));
                    json_filter.add_pattern("*.json");

                    let filters = gtk::gio::ListStore::new::<gtk::FileFilter>();
                    filters.append(&png_filter);
                    filters.append(&json_filter);

                    let dialog = gtk::FileDialog::builder()
                        .title("Export Character")
                        .modal(true)
                        .accept_label("Export")
                        .initial_name(format!("{}.png", slug))
                        .filters(&filters)
                        .default_filter(&png_filter)
                        .build();

                    let sender_clone = sender.input_sender().clone();
                    let parent = self.toast_overlay.root().and_downcast::<gtk::Window>();
                    dialog.save(
                        parent.as_ref(),
                        gtk::gio::Cancellable::NONE,
                        move |result| {
                            if let Ok(file) = result {
                                if let Some(path) = file.path() {
                                    let path_str = path.to_string_lossy().to_string();
                                    let sender_clone = sender_clone.clone();

                                    // Clone character for the thread
                                    let ch = character.clone();
                                    let avatar_path = character.avatar_path.clone();

                                    std::thread::spawn(move || {
                                        let result = (|| -> Result<(), String> {
                                            let ext = std::path::Path::new(&path_str)
                                                .extension()
                                                .and_then(|e| e.to_str())
                                                .unwrap_or("")
                                                .to_lowercase();

                                            if ext == "json" {
                                                let json = card_io::export_to_json(&ch)
                                                    .map_err(|e| format!("JSON export failed: {}", e))?;
                                                std::fs::write(&path_str, json)
                                                    .map_err(|e| format!("Write failed: {}", e))?;
                                            } else {
                                                // PNG export
                                                let avatar_bytes = avatar_path
                                                    .as_ref()
                                                    .and_then(|p| std::fs::read(p).ok());
                                                let png = card_io::export_to_png(
                                                    &ch,
                                                    avatar_bytes.as_deref(),
                                                )
                                                .map_err(|e| format!("PNG export failed: {}", e))?;
                                                std::fs::write(&path_str, png)
                                                    .map_err(|e| format!("Write failed: {}", e))?;
                                            }
                                            Ok(())
                                        })();

                                        match result {
                                            Ok(()) => {
                                                let _ = sender_clone.send(AppMsg::ShowToast(
                                                    "Character exported".to_string(),
                                                ));
                                            }
                                            Err(msg) => {
                                                let _ =
                                                    sender_clone.send(AppMsg::ShowToast(msg));
                                            }
                                        }
                                    });
                                }
                            }
                        },
                    );
                }
            },
            AppMsg::ChatOutput(output) => match output {
                ChatPanelOutput::StartConversation { character_id, provider_id } => {
                    self.db_worker.sender().send(DbInput::StartConversation { character_id, provider_id }).unwrap();
                }
                ChatPanelOutput::SaveMessage(msg) => {
                    self.db_worker.sender().send(DbInput::SaveMessage(msg)).unwrap();
                }
                ChatPanelOutput::LoadMessages { conversation_id, branch_id } => {
                    self.db_worker.sender().send(DbInput::LoadMessages { conversation_id, branch_id }).unwrap();
                }
                ChatPanelOutput::LoadConversations => {
                    self.db_worker.sender().send(DbInput::LoadConversations).unwrap();
                }
                ChatPanelOutput::DeleteConversation(id) => {
                    self.db_worker.sender().send(DbInput::DeleteConversation(id)).unwrap();
                }
                ChatPanelOutput::RequestApiKey(provider_id) => {
                    // Load API key from keyring and send back to chat panel
                    let chat_sender = self.chat_panel.sender().clone();
                    sender.spawn_oneshot_command(move || {
                        let api_key = KeyringStore::load_key(&provider_id).unwrap_or_default();
                        let _ = chat_sender.send(ChatPanelMsg::LoadApiKey(api_key));
                    });
                }
                ChatPanelOutput::ShowToast(msg) => {
                    self.show_toast(&msg);
                }
            },
            AppMsg::EditorOutput(output) => match output {
                CharacterEditorOutput::Saved(character) => {
                    self.db_worker.sender().send(DbInput::SaveCharacter(character)).unwrap();
                    self.character_editor = None;
                }
                CharacterEditorOutput::Cancelled => {
                    self.character_editor = None;
                }
            },
            AppMsg::ShowToast(message) => {
                self.show_toast(&message);
            }
            AppMsg::Quit => {
                // The window will close via the main loop when the Quit message
                // is processed. No explicit close needed — the quit action
                // is handled by the application's shutdown.
            }
        }
    }
}

impl AppModel {
    fn sync_view_stack(&self) {
        let name = match self.active_view {
            ActiveView::Chat => "chat",
            ActiveView::Characters => "characters",
            ActiveView::Settings => "settings",
        };
        self.view_stack.set_visible_child_name(name);
    }

    fn show_toast(&self, message: &str) {
        let toast = adw::Toast::new(message);
        toast.set_timeout(3);
        self.toast_overlay.add_toast(toast);
    }
}
