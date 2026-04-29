use adw::prelude::*;
use relm4::prelude::*;
use relm4::{
    adw, gtk, ComponentParts, ComponentSender, WorkerController,
};

use crate::db::{DbInput, DbOutput, DbService};
use crate::secrets::KeyringStore;
use crate::ui::settings::SettingsPanelMsg;
use crate::ui::{CharactersPanel, ChatPanel, ChatPanelMsg, SettingsPanel, SettingsPanelOutput};

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
    Quit,
}

/// Root application model holding global state and child panel controllers.
pub struct AppModel {
    active_view: ActiveView,
    sidebar_visible: bool,
    has_providers: bool,
    db_worker: WorkerController<DbService>,
    chat_panel: relm4::Controller<ChatPanel>,
    #[allow(dead_code)] // Keep alive to run component event loop
    characters_panel: relm4::Controller<CharactersPanel>,
    settings_panel: relm4::Controller<SettingsPanel>,
    view_stack: adw::ViewStack,
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
                                set_icon_name: Some("chat-message-new-symbolic"),
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
                                set_icon_name: "sidebar-show-symbolic",
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
            .detach();

        let characters_panel = CharactersPanel::builder().launch(()).detach();

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
            db_worker,
            chat_panel,
            characters_panel,
            settings_panel,
            view_stack: adw::ViewStack::new(),
        };

        let widgets = view_output!();

        // Replace placeholder with the actual ViewStack from the view
        model.view_stack = widgets.view_stack.clone();

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
        chat_page.set_icon_name(Some("chat-message-new-symbolic"));

        let characters_page = widgets.view_stack.add_named(&characters_widget, Some("characters"));
        characters_page.set_title(Some("Characters"));
        characters_page.set_icon_name(Some("avatar-default-symbolic"));

        let settings_page = widgets.view_stack.add_named(&settings_widget, Some("settings"));
        settings_page.set_title(Some("Settings"));
        settings_page.set_icon_name(Some("preferences-system-symbolic"));

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

                gtk::glib::Propagation::Proceed
            });
            controller
        });

        // Query providers on startup to determine first-launch state
        model.db_worker.sender().send(DbInput::LoadProviders).unwrap();

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
                DbOutput::Error(e) => {
                    tracing::error!("Database error: {}", e);
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
                    // We need to find the provider config first, but we don't have it here.
                    // Instead, forward to settings panel which has the provider list.
                    // The settings panel will handle the test via KeyLoaded message.
                    // Load the key from keyring on a background thread, then send it back
                    let id_clone = id.clone();
                    let settings_sender = self.settings_panel.sender().clone();
                    sender.spawn_oneshot_command(move || {
                        let api_key = KeyringStore::load_key(&id_clone)
                            .unwrap_or_default();
                        let _ = settings_sender.send(SettingsPanelMsg::KeyLoaded(id_clone, api_key));
                    });
                }
            },
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
}
