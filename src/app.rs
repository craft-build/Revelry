use adw::prelude::*;
use relm4::prelude::*;
use relm4::{
    adw, gtk, ComponentParts, ComponentSender, WorkerController,
};

use crate::db::{DbInput, DbOutput, DbService};
use crate::ui::{CharactersPanel, ChatPanel, ChatPanelMsg, SettingsPanel};

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
    #[allow(dead_code)] // Keep alive to run component event loop
    settings_panel: relm4::Controller<SettingsPanel>,
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
                            pack_start = &gtk::ToggleButton {
                                set_icon_name: "sidebar-show-symbolic",
                                set_tooltip_text: Some("Toggle Sidebar"),
                                #[watch]
                                set_active: model.sidebar_visible,
                                connect_clicked => AppMsg::ToggleSidebar,
                            },

                            #[name(view_switcher)]
                            pack_start = &adw::ViewSwitcher {},
                        },

                        #[name(view_stack)]
                        adw::ViewStack {
                            set_vexpand: true,
                            set_hexpand: true,

                            #[watch]
                            set_visible_child_name: match model.active_view {
                                ActiveView::Chat => "chat",
                                ActiveView::Characters => "characters",
                                ActiveView::Settings => "settings",
                            },
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

        let settings_panel = SettingsPanel::builder().launch(()).detach();

        // Get widget references before moving controllers into model.
        // Clone the GObject reference so it's independent of the Controller borrow.
        let chat_widget = chat_panel.widget().clone();
        let characters_widget = characters_panel.widget().clone();
        let settings_widget = settings_panel.widget().clone();

        let model = AppModel {
            active_view: ActiveView::Chat,
            sidebar_visible: true,
            has_providers: false,
            db_worker,
            chat_panel,
            characters_panel,
            settings_panel,
        };

        let widgets = view_output!();

        // Wire up the ViewSwitcher to the ViewStack
        widgets.view_switcher.set_stack(Some(&widgets.view_stack));

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

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::SwitchView(view) => {
                self.active_view = view;
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

                    // Forward provider state to ChatPanel
                    let _ = self.chat_panel.sender().send(ChatPanelMsg::SetHasProviders(has_providers));
                }
                DbOutput::ProviderSaved(_) => {
                    self.db_worker.sender().send(DbInput::LoadProviders).unwrap();
                }
                DbOutput::ProviderDeleted(_) => {
                    self.db_worker.sender().send(DbInput::LoadProviders).unwrap();
                }
                DbOutput::Error(e) => {
                    tracing::error!("Database error: {}", e);
                }
                _ => {}
            },
            AppMsg::Quit => {
                // The window will close via the main loop when the Quit message
                // is processed. No explicit close needed — the quit action
                // is handled by the application's shutdown.
            }
        }
    }
}
