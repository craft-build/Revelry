use adw::prelude::*;
use relm4::prelude::*;
use relm4::{adw, gtk, Component, ComponentParts, ComponentSender};

use crate::domain::provider::{ProviderConfig, ProviderType};
use crate::secrets::KeyringStore;
use crate::service::provider_test::{self, ProviderTestResult};

/// State of a provider connectivity test for the UI.
#[derive(Debug, Clone)]
pub enum TestState {
    Idle,
    Testing,
    Success,
    Failure(String),
}

/// Settings panel with provider configuration.
pub struct SettingsPanel {
    providers: Vec<ProviderConfig>,
    test_states: std::collections::HashMap<String, TestState>,
    // Widget references needed for imperative updates in update()
    providers_group: adw::PreferencesGroup,
    toast_overlay: adw::ToastOverlay,
}

/// Messages handled by SettingsPanel.
#[derive(Debug)]
pub enum SettingsPanelMsg {
    ProvidersLoaded(Vec<ProviderConfig>),
    AddProvider,
    EditProvider(String),
    DeleteProvider(String),
    ConfirmDelete(String),
    ProviderSaved(ProviderConfig),
    ProviderDeleted(String),
    TestProvider(String),
    KeyLoaded(String, String), // (provider_id, api_key) — sent by parent after keyring load
    TestResult(String, ProviderTestResult),
    KeyringError(String),
}

/// Command output from background threads.
#[derive(Debug)]
pub enum SettingsPanelCmd {
    TestResult(String, ProviderTestResult),
}

/// Output messages from SettingsPanel to parent (AppModel).
/// The parent handles all DB/keyring operations since it owns the DbService Worker.
#[derive(Debug)]
pub enum SettingsPanelOutput {
    SaveProvider(ProviderConfig),
    DeleteProvider(String),
    DeleteKey(String),
    TestProvider(String),
}

#[relm4::component(pub)]
impl Component for SettingsPanel {
    type Init = ();
    type Input = SettingsPanelMsg;
    type Output = SettingsPanelOutput;
    type CommandOutput = SettingsPanelCmd;

    view! {
        #[root]
        adw::ToastOverlay {
            #[name(clamp)]
            adw::Clamp {
                set_maximum_size: 600,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 24,
                    set_margin_top: 24,
                    set_margin_bottom: 24,
                    set_margin_start: 12,
                    set_margin_end: 12,

                    #[name(providers_group)]
                    adw::PreferencesGroup {
                        set_title: "AI Providers",
                        set_description: Some("Configure AI providers for chat and roleplay."),
                    },
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widgets = view_output!();

        let model = SettingsPanel {
            providers: Vec::new(),
            test_states: std::collections::HashMap::new(),
            providers_group: widgets.providers_group.clone(),
            toast_overlay: root.clone(),
        };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            SettingsPanelMsg::ProvidersLoaded(providers) => {
                self.providers = providers;
                self.rebuild_provider_list(&sender);
            }
            SettingsPanelMsg::AddProvider => {
                self.open_provider_config(None, &sender);
            }
            SettingsPanelMsg::EditProvider(id) => {
                let provider = self.providers.iter().find(|p| p.id == id).cloned();
                self.open_provider_config(provider, &sender);
            }
            SettingsPanelMsg::DeleteProvider(id) => {
                let provider = self.providers.iter().find(|p| p.id == id);
                let name = provider.map(|p| p.name.as_str()).unwrap_or("this provider");

                let dialog = adw::MessageDialog::new(
                    None::<&gtk::Window>,
                    Some("Remove Provider?"),
                    Some(&format!(
                        "This will remove {} and delete its API key from the system keychain. This cannot be undone.",
                        name
                    )),
                );
                dialog.add_response("cancel", "Cancel");
                dialog.add_response("remove", "Remove");
                dialog.set_response_appearance("remove", adw::ResponseAppearance::Destructive);
                dialog.set_close_response("cancel");

                let sender_clone = sender.clone();
                let id_clone = id.clone();
                dialog.connect_response(None, move |_, response| {
                    if response == "remove" {
                        let _ = sender_clone.input_sender().send(SettingsPanelMsg::ConfirmDelete(id_clone.clone()));
                    }
                });

                dialog.present();
            }
            SettingsPanelMsg::ConfirmDelete(id) => {
                // Tell parent to delete from DB and keyring
                let _ = sender.output(SettingsPanelOutput::DeleteProvider(id.clone()));
                let _ = sender.output(SettingsPanelOutput::DeleteKey(id));
            }
            SettingsPanelMsg::ProviderSaved(_config) => {
                self.show_toast("Provider saved");
                // Parent will reload providers after SaveProvider output
            }
            SettingsPanelMsg::ProviderDeleted(_id) => {
                self.show_toast("Provider removed");
            }
            SettingsPanelMsg::TestProvider(id) => {
                self.test_states.insert(id.clone(), TestState::Testing);
                self.rebuild_provider_list(&sender);
                // Tell parent to load key and then test
                let _ = sender.output(SettingsPanelOutput::TestProvider(id));
            }
            SettingsPanelMsg::KeyLoaded(id, api_key) => {
                // Parent loaded the API key from keyring — now run the test
                let Some(config) = self.providers.iter().find(|p| p.id == id).cloned() else {
                    return;
                };

                let id_clone = id.clone();
                let provider_type = config.provider_type.clone();
                let base_url = config.base_url.clone();
                let api_key_opt = if api_key.is_empty() && !provider_type.requires_api_key() {
                    None
                } else {
                    Some(api_key)
                };

                sender.spawn_oneshot_command(move || {
                    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
                    let result = rt.block_on(provider_test::test_provider(
                        &provider_type,
                        base_url.as_deref(),
                        api_key_opt.as_deref(),
                    ));
                    SettingsPanelCmd::TestResult(id_clone, result)
                });
            }
            SettingsPanelMsg::TestResult(id, result) => {
                if result.success {
                    self.test_states.insert(id.clone(), TestState::Success);
                    self.show_toast(&format!("Connected — {}", result.message));
                } else {
                    self.test_states.insert(id.clone(), TestState::Failure(result.message.clone()));
                    self.show_toast(&format!("Connection failed: {}", result.message));
                }
                self.rebuild_provider_list(&sender);
            }
            SettingsPanelMsg::KeyringError(msg) => {
                self.show_toast(&msg);
            }
        }
    }

    fn update_cmd(&mut self, msg: Self::CommandOutput, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            SettingsPanelCmd::TestResult(id, result) => {
                let _ = sender.input_sender().send(SettingsPanelMsg::TestResult(id, result));
            }
        }
    }
}

impl SettingsPanel {
    /// Rebuild the provider list UI in the PreferencesGroup.
    fn rebuild_provider_list(&self, sender: &ComponentSender<Self>) {
        let group = &self.providers_group;

        // Remove all existing children
        while let Some(child) = group.last_child() {
            group.remove(&child);
        }

        if self.providers.is_empty() {
            // Show Add Provider button per UI-SPEC.md
            let add_btn = gtk::Button::with_label("Add Provider");
            add_btn.add_css_class("suggested-action");
            add_btn.add_css_class("pill");
            add_btn.set_halign(gtk::Align::Center);

            let sender_clone = sender.clone();
            add_btn.connect_clicked(move |_| {
                let _ = sender_clone.input_sender().send(SettingsPanelMsg::AddProvider);
            });

            group.add(&add_btn);
        } else {
            // Show provider rows per UI-SPEC.md
            for provider in &self.providers {
                let row = adw::ActionRow::builder()
                    .title(&provider.name)
                    .subtitle(&format!(
                        "{:?}{}",
                        provider.provider_type,
                        if provider.enabled { "" } else { " (disabled)" }
                    ))
                    .build();

                // Edit button
                let edit_btn = gtk::Button::with_label("Edit");
                edit_btn.add_css_class("flat");
                let sender_clone = sender.clone();
                let id = provider.id.clone();
                edit_btn.connect_clicked(move |_| {
                    let _ = sender_clone.input_sender().send(SettingsPanelMsg::EditProvider(id.clone()));
                });
                row.add_suffix(&edit_btn);

                // Test state indicator
                let test_state = self.test_states.get(&provider.id);
                match test_state {
                    Some(TestState::Testing) => {
                        let spinner = gtk::Spinner::new();
                        spinner.start();
                        row.add_suffix(&spinner);
                    }
                    Some(TestState::Success) => {
                        let icon = gtk::Image::from_icon_name("emblem-ok-symbolic");
                        icon.add_css_class("success");
                        row.add_suffix(&icon);
                    }
                    Some(TestState::Failure(_)) => {
                        let icon = gtk::Image::from_icon_name("dialog-error-symbolic");
                        icon.add_css_class("error");
                        row.add_suffix(&icon);
                    }
                    _ => {}
                }

                // Test button
                let test_btn = gtk::Button::with_label("Test");
                test_btn.add_css_class("flat");
                if matches!(test_state, Some(TestState::Testing)) {
                    test_btn.set_sensitive(false);
                }
                let sender_clone = sender.clone();
                let id = provider.id.clone();
                test_btn.connect_clicked(move |_| {
                    let _ = sender_clone.input_sender().send(SettingsPanelMsg::TestProvider(id.clone()));
                });
                row.add_suffix(&test_btn);

                // Delete button
                let delete_btn = gtk::Button::with_label("Delete");
                delete_btn.add_css_class("flat");
                let sender_clone = sender.clone();
                let id = provider.id.clone();
                delete_btn.connect_clicked(move |_| {
                    let _ = sender_clone.input_sender().send(SettingsPanelMsg::DeleteProvider(id.clone()));
                });
                row.add_suffix(&delete_btn);

                group.add(&row);
            }
        }
    }

    /// Open the provider configuration page.
    fn open_provider_config(
        &self,
        existing: Option<ProviderConfig>,
        sender: &ComponentSender<Self>,
    ) {
        let is_editing = existing.is_some();
        let config = existing.unwrap_or_else(|| {
            ProviderConfig::new("New Provider".to_string(), ProviderType::OpenAI)
        });

        let page = ProviderConfigPage::builder()
            .launch((config, is_editing))
            .forward(sender.input_sender(), |output| match output {
                ProviderConfigOutput::SaveConfig(config) => {
                    SettingsPanelMsg::ProviderSaved(config)
                }
            });

        let widget = page.widget().clone();

        // Present in a PreferencesWindow
        let window = adw::PreferencesWindow::new();
        window.set_title(Some(if is_editing { "Edit Provider" } else { "Add Provider" }));
        window.set_content(Some(&widget));

        // Find the parent window via the ToastOverlay's ancestor
        if let Some(toplevel) = self.toast_overlay.ancestor(gtk::Window::static_type()) {
            if let Some(parent_window) = toplevel.downcast_ref::<gtk::Window>() {
                window.set_transient_for(Some(parent_window));
            }
        }

        window.present();
    }

    /// Show a toast message.
    fn show_toast(&self, message: &str) {
        let toast = adw::Toast::new(message);
        toast.set_timeout(3);
        self.toast_overlay.add_toast(toast);
    }
}

// ---- ProviderConfigPage (sub-component) ----

/// Provider configuration page opened as AdwPreferencesWindow sub-page.
pub struct ProviderConfigPage {
    config: ProviderConfig,
    #[allow(dead_code)] // Used for UI conditional logic (future: disable type changes on edit)
    is_editing: bool,
    api_key: String,
    test_state: TestState,
    // Widget references needed for imperative updates
    base_url_row: adw::EntryRow,
    api_key_row: adw::PasswordEntryRow,
    test_btn: gtk::Button,
    test_indicator: gtk::Box,
    root_page: adw::PreferencesPage,
}

/// Messages handled by ProviderConfigPage.
#[derive(Debug)]
pub enum ProviderConfigPageMsg {
    SetProviderType(u32),
    SetName(String),
    SetBaseUrl(String),
    SetApiKey(String),
    SetEnabled(bool),
    TestConnection,
    TestResult(ProviderTestResult),
    Save,
}

/// Command output from background threads.
#[derive(Debug)]
pub enum ProviderConfigPageCmd {
    TestResult(ProviderTestResult),
    KeySaved,
}

/// Output messages from ProviderConfigPage to parent.
#[derive(Debug)]
pub enum ProviderConfigOutput {
    SaveConfig(ProviderConfig),
}

/// All provider types for the combo row.
const PROVIDER_TYPE_NAMES: &[&str] = &[
    "OpenAI", "Anthropic", "Ollama", "Gemini", "Cohere",
    "Mistral", "Groq", "DeepSeek", "OpenRouter", "xAI",
    "Perplexity", "Together", "AzureOpenAI", "HuggingFace", "Custom",
];

fn provider_type_from_index(index: u32) -> ProviderType {
    match index {
        0 => ProviderType::OpenAI,
        1 => ProviderType::Anthropic,
        2 => ProviderType::Ollama,
        3 => ProviderType::Gemini,
        4 => ProviderType::Cohere,
        5 => ProviderType::Mistral,
        6 => ProviderType::Groq,
        7 => ProviderType::DeepSeek,
        8 => ProviderType::OpenRouter,
        9 => ProviderType::Xai,
        10 => ProviderType::Perplexity,
        11 => ProviderType::Together,
        12 => ProviderType::AzureOpenAI,
        13 => ProviderType::HuggingFace,
        14 => ProviderType::Custom,
        _ => ProviderType::OpenAI,
    }
}

fn provider_type_to_index(pt: &ProviderType) -> u32 {
    match pt {
        ProviderType::OpenAI => 0,
        ProviderType::Anthropic => 1,
        ProviderType::Ollama => 2,
        ProviderType::Gemini => 3,
        ProviderType::Cohere => 4,
        ProviderType::Mistral => 5,
        ProviderType::Groq => 6,
        ProviderType::DeepSeek => 7,
        ProviderType::OpenRouter => 8,
        ProviderType::Xai => 9,
        ProviderType::Perplexity => 10,
        ProviderType::Together => 11,
        ProviderType::AzureOpenAI => 12,
        ProviderType::HuggingFace => 13,
        ProviderType::Custom => 14,
    }
}

#[relm4::component(pub)]
impl Component for ProviderConfigPage {
    type Init = (ProviderConfig, bool); // (config, is_editing)
    type Input = ProviderConfigPageMsg;
    type Output = ProviderConfigOutput;
    type CommandOutput = ProviderConfigPageCmd;

    view! {
        #[root]
        adw::PreferencesPage {
            set_title: "Provider Configuration",

            // Provider Type group
            adw::PreferencesGroup {
                set_title: "Provider Type",

                #[name(combo_row)]
                adw::ComboRow {
                    set_title: "Provider Type",
                    set_model: Some(&gtk::StringList::new(PROVIDER_TYPE_NAMES)),
                },
            },

            // Connection group
            adw::PreferencesGroup {
                set_title: "Connection",

                #[name(name_row)]
                adw::EntryRow {
                    set_title: "Name",
                },

                #[name(base_url_row)]
                adw::EntryRow {
                    set_title: "Base URL",
                },

                #[name(api_key_row)]
                adw::PasswordEntryRow {
                    set_title: "API Key",
                },

                #[name(enabled_row)]
                adw::SwitchRow {
                    set_title: "Enabled",
                },
            },

            // Connection Test group
            adw::PreferencesGroup {
                set_title: "Connection Test",

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 12,

                    #[name(test_btn)]
                    gtk::Button {
                        set_label: "Test Connection",
                        add_css_class: "suggested-action",
                    },

                    #[name(test_indicator)]
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                    },
                },
            },

            // Save group
            adw::PreferencesGroup {
                gtk::Button {
                    set_label: "Save Provider",
                    add_css_class: "suggested-action",
                    connect_clicked => ProviderConfigPageMsg::Save,
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let (config, is_editing) = init;

        let widgets = view_output!();

        // Set initial values
        widgets.name_row.set_text(&config.name);
        if let Some(ref url) = config.base_url {
            widgets.base_url_row.set_text(url);
        }
        widgets.enabled_row.set_active(config.enabled);
        widgets.combo_row.set_selected(provider_type_to_index(&config.provider_type));

        let model = ProviderConfigPage {
            config: config.clone(),
            is_editing,
            api_key: String::new(),
            test_state: TestState::Idle,
            base_url_row: widgets.base_url_row.clone(),
            api_key_row: widgets.api_key_row.clone(),
            test_btn: widgets.test_btn.clone(),
            test_indicator: widgets.test_indicator.clone(),
            root_page: root.clone(),
        };

        // Show/hide rows based on provider type
        update_dynamic_rows(&model, &config.provider_type);

        // Connect signals
        let sender_clone = sender.clone();
        widgets.combo_row.connect_selected_notify(move |combo| {
            let _ = sender_clone.input_sender().send(ProviderConfigPageMsg::SetProviderType(combo.selected()));
        });

        let sender_clone = sender.clone();
        widgets.name_row.connect_changed(move |row| {
            let _ = sender_clone.input_sender().send(ProviderConfigPageMsg::SetName(row.text().to_string()));
        });

        let sender_clone = sender.clone();
        model.base_url_row.connect_changed(move |row| {
            let _ = sender_clone.input_sender().send(ProviderConfigPageMsg::SetBaseUrl(row.text().to_string()));
        });

        let sender_clone = sender.clone();
        model.api_key_row.connect_changed(move |row| {
            let _ = sender_clone.input_sender().send(ProviderConfigPageMsg::SetApiKey(row.text().to_string()));
        });

        let sender_clone = sender.clone();
        widgets.enabled_row.connect_active_notify(move |row| {
            let _ = sender_clone.input_sender().send(ProviderConfigPageMsg::SetEnabled(row.is_active()));
        });

        let sender_clone = sender.clone();
        model.test_btn.connect_clicked(move |_| {
            let _ = sender_clone.input_sender().send(ProviderConfigPageMsg::TestConnection);
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            ProviderConfigPageMsg::SetProviderType(index) => {
                let provider_type = provider_type_from_index(index);
                self.config.provider_type = provider_type.clone();
                // Update base_url to default if current is empty
                if let Some(default_url) = provider_type.default_base_url() {
                    if self.config.base_url.is_none() {
                        self.config.base_url = Some(default_url.to_string());
                        self.base_url_row.set_text(default_url);
                    }
                }
                update_dynamic_rows(self, &provider_type);
            }
            ProviderConfigPageMsg::SetName(name) => {
                self.config.name = name;
            }
            ProviderConfigPageMsg::SetBaseUrl(url) => {
                self.config.base_url = if url.is_empty() { None } else { Some(url) };
            }
            ProviderConfigPageMsg::SetApiKey(key) => {
                self.api_key = key;
            }
            ProviderConfigPageMsg::SetEnabled(enabled) => {
                self.config.enabled = enabled;
            }
            ProviderConfigPageMsg::TestConnection => {
                self.test_state = TestState::Testing;
                update_test_indicator(&self.test_btn, &self.test_indicator, &self.test_state);

                let provider_type = self.config.provider_type.clone();
                let base_url = self.config.base_url.clone();
                let api_key = if self.api_key.is_empty() && !provider_type.requires_api_key() {
                    None
                } else {
                    Some(self.api_key.clone())
                };

                sender.spawn_oneshot_command(move || {
                    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
                    let result = rt.block_on(provider_test::test_provider(
                        &provider_type,
                        base_url.as_deref(),
                        api_key.as_deref(),
                    ));
                    ProviderConfigPageCmd::TestResult(result)
                });
            }
            ProviderConfigPageMsg::TestResult(result) => {
                if result.success {
                    self.test_state = TestState::Success;
                } else {
                    self.test_state = TestState::Failure(result.message.clone());
                }
                update_test_indicator(&self.test_btn, &self.test_indicator, &self.test_state);
            }
            ProviderConfigPageMsg::Save => {
                self.config.updated_at = chrono::Local::now().naive_local();

                // Save API key to keyring on background thread
                if self.config.provider_type.requires_api_key() && !self.api_key.is_empty() {
                    let id = self.config.id.clone();
                    let key = self.api_key.clone();
                    sender.spawn_oneshot_command(move || {
                        match KeyringStore::save_key(&id, &key) {
                            Ok(()) => tracing::info!("API key saved to keychain for provider {}", id),
                            Err(e) => tracing::error!("Failed to save key to keyring: {}", e),
                        }
                        ProviderConfigPageCmd::KeySaved
                    });
                }

                let _ = sender.output(ProviderConfigOutput::SaveConfig(self.config.clone()));

                // Close the preferences window
                if let Some(widget) = self.root_page.ancestor(gtk::Window::static_type()) {
                    if let Some(window) = widget.downcast_ref::<adw::PreferencesWindow>() {
                        window.close();
                    }
                }
            }
        }
    }

    fn update_cmd(&mut self, msg: Self::CommandOutput, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            ProviderConfigPageCmd::TestResult(result) => {
                // Command output is handled via the sender in the closure
                // We'll set the test state and update indicator directly
                if result.success {
                    self.test_state = TestState::Success;
                } else {
                    self.test_state = TestState::Failure(result.message.clone());
                }
                update_test_indicator(&self.test_btn, &self.test_indicator, &self.test_state);
            }
            ProviderConfigPageCmd::KeySaved => {
                // Key saved successfully, nothing else to do
            }
        }
    }
}

/// Show/hide dynamic rows based on provider type (D-08).
fn update_dynamic_rows(model: &ProviderConfigPage, provider_type: &ProviderType) {
    // Base URL row: shown for providers that have a default, or for custom/ollama/azure
    let show_base_url = provider_type.default_base_url().is_some()
        || matches!(provider_type, ProviderType::Custom | ProviderType::AzureOpenAI | ProviderType::Ollama);
    model.base_url_row.set_visible(show_base_url);

    // API Key row: shown when provider requires an API key
    model.api_key_row.set_visible(provider_type.requires_api_key());
}

/// Update the test indicator based on test state.
fn update_test_indicator(test_btn: &gtk::Button, test_indicator: &gtk::Box, state: &TestState) {
    // Clear existing indicator children
    while let Some(child) = test_indicator.first_child() {
        test_indicator.remove(&child);
    }

    match state {
        TestState::Idle => {
            test_btn.set_sensitive(true);
            test_btn.set_label("Test Connection");
        }
        TestState::Testing => {
            test_btn.set_sensitive(false);
            test_btn.set_label("Testing...");
            let spinner = gtk::Spinner::new();
            spinner.start();
            test_indicator.append(&spinner);
        }
        TestState::Success => {
            test_btn.set_sensitive(true);
            test_btn.set_label("Test Connection");
            let icon = gtk::Image::from_icon_name("emblem-ok-symbolic");
            icon.add_css_class("success");
            test_indicator.append(&icon);
        }
        TestState::Failure(msg) => {
            test_btn.set_sensitive(true);
            test_btn.set_label("Test Connection");
            let icon = gtk::Image::from_icon_name("dialog-error-symbolic");
            icon.add_css_class("error");
            test_indicator.append(&icon);
            let label = gtk::Label::new(Some(msg));
            label.add_css_class("caption");
            label.add_css_class("error");
            label.set_wrap(true);
            test_indicator.append(&label);
        }
    }
}
