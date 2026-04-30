use adw::prelude::*;
use relm4::prelude::*;
use relm4::{adw, gtk, ComponentParts, ComponentSender};

use crate::domain::character::CharacterCardV2;

// Type alias — the #[relm4::component] macro generates a `CharacterEditorModel` widgets
// struct, but we export the component type as `CharacterEditor` for downstream consumers.
// The macro creates the wrapper type automatically via the `pub` attribute.

// ---------------------------------------------------------------------------
// Model
// ---------------------------------------------------------------------------

/// Unified character editor that adapts between creation and edit modes.
#[allow(dead_code)]
pub struct CharacterEditorModel {
    character: CharacterCardV2,
    is_new: bool,
    active_page: EditorPage,
    avatar_bytes: Option<Vec<u8>>,
    file_chooser: Option<gtk::FileChooserNative>,
    // Widget references for imperative updates
    tags_container: gtk::Box,
    greetings_container: gtk::ListBox,
    name_row: adw::EntryRow,
    description_buffer: gtk::TextBuffer,
    personality_buffer: gtk::TextBuffer,
    scenario_buffer: gtk::TextBuffer,
    first_mes_buffer: gtk::TextBuffer,
    mes_example_buffer: gtk::TextBuffer,
    system_prompt_buffer: gtk::TextBuffer,
    post_history_buffer: gtk::TextBuffer,
    creator_row: adw::EntryRow,
    avatar_button: gtk::Button,
    window: adw::PreferencesWindow,
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditorPage {
    Identity,
    Personality,
    Scenario,
    Advanced,
}

#[derive(Debug)]
pub enum CharacterEditorMsg {
    SetField { field: EditorField, value: String },
    AddAlternateGreeting,
    RemoveAlternateGreeting(usize),
    SetAlternateGreeting { index: usize, value: String },
    AddTag,
    RemoveTag(usize),
    SetTag { index: usize, value: String },
    ChooseAvatar,
    AvatarSelected(String),
    SwitchPage(EditorPage),
    Save,
    Cancel,
}

#[derive(Debug)]
pub enum CharacterEditorOutput {
    Saved(CharacterCardV2),
    Cancelled,
}

#[derive(Debug, Clone)]
pub enum EditorField {
    Name,
    Description,
    Personality,
    Scenario,
    FirstMes,
    MesExample,
    SystemPrompt,
    PostHistoryInstructions,
    Creator,
}

/// Initialization variant for the editor.
pub enum CharacterEditorInit {
    New,
    Edit(CharacterCardV2),
}

#[derive(Debug)]
pub enum CharacterEditorCmd {
    AvatarLoaded { bytes: Vec<u8>, path: String },
}

// ---------------------------------------------------------------------------
// Component implementation
// ---------------------------------------------------------------------------

#[relm4::component(pub)]
impl Component for CharacterEditorModel {
    type Init = CharacterEditorInit;
    type Input = CharacterEditorMsg;
    type Output = CharacterEditorOutput;
    type CommandOutput = CharacterEditorCmd;

    view! {
        #[root]
        adw::PreferencesWindow {
            set_title: Some("Character Editor"),
            set_modal: true,
            set_default_width: 600,
            set_default_height: 700,

            // ---- Identity Page ----
            add = &adw::PreferencesPage {
                set_title: "Identity",
                set_icon_name: Some("contact-page"),

                adw::PreferencesGroup {
                    set_title: "Basic Info",
                    set_description: Some("Name is required."),

                    #[name(name_row)]
                    adw::EntryRow {
                        set_title: "Name",
                    },

                    #[name(creator_row)]
                    adw::EntryRow {
                        set_title: "Creator",
                    },
                },

                adw::PreferencesGroup {
                    set_title: "Avatar",

                    #[name(avatar_button)]
                    gtk::Button {
                        set_label: "Choose Avatar…",
                        set_halign: gtk::Align::Start,
                        connect_clicked => CharacterEditorMsg::ChooseAvatar,
                    },
                },

                adw::PreferencesGroup {
                    set_title: "Description",

                    gtk::ScrolledWindow {
                        set_min_content_height: 80,
                        set_vexpand: false,

                        #[name(description_view)]
                        gtk::TextView {
                            set_wrap_mode: gtk::WrapMode::WordChar,
                        },
                    },
                },
            },

            // ---- Personality Page ----
            add = &adw::PreferencesPage {
                set_title: "Personality",
                set_icon_name: Some("mood"),

                adw::PreferencesGroup {
                    set_title: "Personality",

                    gtk::ScrolledWindow {
                        set_min_content_height: 120,
                        set_vexpand: false,

                        #[name(personality_view)]
                        gtk::TextView {
                            set_wrap_mode: gtk::WrapMode::WordChar,
                        },
                    },
                },

                adw::PreferencesGroup {
                    set_title: "Tags",

                    #[name(tags_container)]
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 4,
                    },

                    gtk::Button {
                        set_label: "Add Tag",
                        set_halign: gtk::Align::Start,
                        add_css_class: "flat",
                        connect_clicked => CharacterEditorMsg::AddTag,
                    },
                },
            },

            // ---- Scenario Page ----
            add = &adw::PreferencesPage {
                set_title: "Scenario",
                set_icon_name: Some("edit"),

                adw::PreferencesGroup {
                    set_title: "Scenario",

                    gtk::ScrolledWindow {
                        set_min_content_height: 80,
                        set_vexpand: false,

                        #[name(scenario_view)]
                        gtk::TextView {
                            set_wrap_mode: gtk::WrapMode::WordChar,
                        },
                    },
                },

                adw::PreferencesGroup {
                    set_title: "First Message",

                    gtk::ScrolledWindow {
                        set_min_content_height: 80,
                        set_vexpand: false,

                        #[name(first_mes_view)]
                        gtk::TextView {
                            set_wrap_mode: gtk::WrapMode::WordChar,
                        },
                    },
                },

                adw::PreferencesGroup {
                    set_title: "Alternate Greetings",

                    #[name(greetings_container)]
                    gtk::ListBox {
                        set_selection_mode: gtk::SelectionMode::None,
                    },

                    gtk::Button {
                        set_label: "Add Greeting",
                        set_halign: gtk::Align::Start,
                        add_css_class: "flat",
                        connect_clicked => CharacterEditorMsg::AddAlternateGreeting,
                    },
                },
            },

            // ---- Advanced Page ----
            add = &adw::PreferencesPage {
                set_title: "Advanced",
                set_icon_name: Some("tune"),

                adw::PreferencesGroup {
                    set_title: "System Prompt",

                    gtk::ScrolledWindow {
                        set_min_content_height: 80,
                        set_vexpand: false,

                        #[name(system_prompt_view)]
                        gtk::TextView {
                            set_wrap_mode: gtk::WrapMode::WordChar,
                        },
                    },
                },

                adw::PreferencesGroup {
                    set_title: "Post-History Instructions",

                    gtk::ScrolledWindow {
                        set_min_content_height: 80,
                        set_vexpand: false,

                        #[name(post_history_view)]
                        gtk::TextView {
                            set_wrap_mode: gtk::WrapMode::WordChar,
                        },
                    },
                },

                adw::PreferencesGroup {
                    set_title: "Example Messages",

                    gtk::ScrolledWindow {
                        set_min_content_height: 120,
                        set_vexpand: false,

                        #[name(mes_example_view)]
                        gtk::TextView {
                            set_wrap_mode: gtk::WrapMode::WordChar,
                        },
                    },
                },

                // Save / Cancel
                adw::PreferencesGroup {
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 12,
                        set_halign: gtk::Align::End,

                        gtk::Button {
                            set_label: "Cancel",
                            connect_clicked => CharacterEditorMsg::Cancel,
                        },

                        gtk::Button {
                            set_label: "Save",
                            add_css_class: "suggested-action",
                            connect_clicked => CharacterEditorMsg::Save,
                        },
                    },
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widgets = view_output!();

        let (character, is_new) = match init {
            CharacterEditorInit::New => (CharacterCardV2::new(String::new()), true),
            CharacterEditorInit::Edit(c) => (c, false),
        };

        // Set window title
        if is_new {
            root.set_title(Some("New Character"));
        } else {
            root.set_title(Some(&format!("Edit: {}", character.name)));
        }

        // Pre-fill single-line fields
        widgets.name_row.set_text(&character.name);
        widgets.creator_row.set_text(&character.creator);

        // Get text buffers from TextViews
        let description_buffer = widgets.description_view.buffer();
        let personality_buffer = widgets.personality_view.buffer();
        let scenario_buffer = widgets.scenario_view.buffer();
        let first_mes_buffer = widgets.first_mes_view.buffer();
        let mes_example_buffer = widgets.mes_example_view.buffer();
        let system_prompt_buffer = widgets.system_prompt_view.buffer();
        let post_history_buffer = widgets.post_history_view.buffer();

        // Pre-fill multi-line fields
        if !character.description.is_empty() {
            description_buffer.set_text(&character.description);
        }
        if !character.personality.is_empty() {
            personality_buffer.set_text(&character.personality);
        }
        if !character.scenario.is_empty() {
            scenario_buffer.set_text(&character.scenario);
        }
        if !character.first_mes.is_empty() {
            first_mes_buffer.set_text(&character.first_mes);
        }
        if !character.mes_example.is_empty() {
            mes_example_buffer.set_text(&character.mes_example);
        }
        if !character.system_prompt.is_empty() {
            system_prompt_buffer.set_text(&character.system_prompt);
        }
        if !character.post_history_instructions.is_empty() {
            post_history_buffer.set_text(&character.post_history_instructions);
        }

        // Connect entry-row changed signals
        let sender_clone = sender.clone();
        widgets.name_row.connect_changed(move |row| {
            let _ = sender_clone.input_sender().send(CharacterEditorMsg::SetField {
                field: EditorField::Name,
                value: row.text().to_string(),
            });
        });

        let sender_clone = sender.clone();
        widgets.creator_row.connect_changed(move |row| {
            let _ = sender_clone.input_sender().send(CharacterEditorMsg::SetField {
                field: EditorField::Creator,
                value: row.text().to_string(),
            });
        });

        // Connect text-buffer changed signals
        let sc = sender.clone();
        description_buffer.connect_changed(move |buf| {
            let (start, end) = buf.bounds();
            let _ = sc.input_sender().send(CharacterEditorMsg::SetField {
                field: EditorField::Description,
                value: buf.text(&start, &end, false).to_string(),
            });
        });

        let sc = sender.clone();
        personality_buffer.connect_changed(move |buf| {
            let (start, end) = buf.bounds();
            let _ = sc.input_sender().send(CharacterEditorMsg::SetField {
                field: EditorField::Personality,
                value: buf.text(&start, &end, false).to_string(),
            });
        });

        let sc = sender.clone();
        scenario_buffer.connect_changed(move |buf| {
            let (start, end) = buf.bounds();
            let _ = sc.input_sender().send(CharacterEditorMsg::SetField {
                field: EditorField::Scenario,
                value: buf.text(&start, &end, false).to_string(),
            });
        });

        let sc = sender.clone();
        first_mes_buffer.connect_changed(move |buf| {
            let (start, end) = buf.bounds();
            let _ = sc.input_sender().send(CharacterEditorMsg::SetField {
                field: EditorField::FirstMes,
                value: buf.text(&start, &end, false).to_string(),
            });
        });

        let sc = sender.clone();
        mes_example_buffer.connect_changed(move |buf| {
            let (start, end) = buf.bounds();
            let _ = sc.input_sender().send(CharacterEditorMsg::SetField {
                field: EditorField::MesExample,
                value: buf.text(&start, &end, false).to_string(),
            });
        });

        let sc = sender.clone();
        system_prompt_buffer.connect_changed(move |buf| {
            let (start, end) = buf.bounds();
            let _ = sc.input_sender().send(CharacterEditorMsg::SetField {
                field: EditorField::SystemPrompt,
                value: buf.text(&start, &end, false).to_string(),
            });
        });

        let sc = sender.clone();
        post_history_buffer.connect_changed(move |buf| {
            let (start, end) = buf.bounds();
            let _ = sc.input_sender().send(CharacterEditorMsg::SetField {
                field: EditorField::PostHistoryInstructions,
                value: buf.text(&start, &end, false).to_string(),
            });
        });

        let model = CharacterEditorModel {
            character: character.clone(),
            is_new,
            active_page: EditorPage::Identity,
            avatar_bytes: None,
            file_chooser: None,
            tags_container: widgets.tags_container.clone(),
            greetings_container: widgets.greetings_container.clone(),
            name_row: widgets.name_row.clone(),
            description_buffer,
            personality_buffer,
            scenario_buffer,
            first_mes_buffer,
            mes_example_buffer,
            system_prompt_buffer,
            post_history_buffer,
            creator_row: widgets.creator_row.clone(),
            avatar_button: widgets.avatar_button.clone(),
            window: root.clone(),
        };

        // Build dynamic lists (tags, alternate greetings) from initial data
        model.rebuild_tags(&sender);
        model.rebuild_greetings(&sender);

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            CharacterEditorMsg::SetField { field, value } => {
                match field {
                    EditorField::Name => self.character.name = value,
                    EditorField::Description => self.character.description = value,
                    EditorField::Personality => self.character.personality = value,
                    EditorField::Scenario => self.character.scenario = value,
                    EditorField::FirstMes => self.character.first_mes = value,
                    EditorField::MesExample => self.character.mes_example = value,
                    EditorField::SystemPrompt => self.character.system_prompt = value,
                    EditorField::PostHistoryInstructions => {
                        self.character.post_history_instructions = value
                    }
                    EditorField::Creator => self.character.creator = value,
                }
            }
            CharacterEditorMsg::AddTag => {
                self.character.tags.push(String::new());
                self.rebuild_tags(&sender);
            }
            CharacterEditorMsg::RemoveTag(index) => {
                if index < self.character.tags.len() {
                    self.character.tags.remove(index);
                    self.rebuild_tags(&sender);
                }
            }
            CharacterEditorMsg::SetTag { index, value } => {
                if index < self.character.tags.len() {
                    self.character.tags[index] = value;
                }
            }
            CharacterEditorMsg::AddAlternateGreeting => {
                self.character.alternate_greetings.push(String::new());
                self.rebuild_greetings(&sender);
            }
            CharacterEditorMsg::RemoveAlternateGreeting(index) => {
                if index < self.character.alternate_greetings.len() {
                    self.character.alternate_greetings.remove(index);
                    self.rebuild_greetings(&sender);
                }
            }
            CharacterEditorMsg::SetAlternateGreeting { index, value } => {
                if index < self.character.alternate_greetings.len() {
                    self.character.alternate_greetings[index] = value;
                }
            }
            CharacterEditorMsg::ChooseAvatar => {
                let chooser = gtk::FileChooserNative::builder()
                    .title("Choose Avatar")
                    .action(gtk::FileChooserAction::Open)
                    .modal(true)
                    .accept_label("Select")
                    .cancel_label("Cancel")
                    .build();

                let filter = gtk::FileFilter::new();
                filter.set_name(Some("Images"));
                filter.add_mime_type("image/png");
                filter.add_mime_type("image/jpeg");
                filter.add_mime_type("image/webp");
                chooser.set_filter(&filter);

                let sender_clone = sender.clone();
                chooser.connect_response(move |chooser, response| {
                    if response == gtk::ResponseType::Accept {
                        if let Some(file) = chooser.file() {
                            if let Some(path) = file.path() {
                                let path_str = path.to_string_lossy().to_string();
                                let _ = sender_clone
                                    .input_sender()
                                    .send(CharacterEditorMsg::AvatarSelected(path_str));
    }
}
                    }
                });

                // Store chooser to keep it alive
                self.file_chooser = Some(chooser.clone());

                // Present — parented to the editor window
                chooser.set_transient_for(Some(&self.window));
                chooser.show();
            }
            CharacterEditorMsg::AvatarSelected(path) => {
                // Read avatar bytes on background thread
                sender.spawn_oneshot_command(move || {
                    let bytes = std::fs::read(&path).unwrap_or_default();
                    CharacterEditorCmd::AvatarLoaded {
                        bytes,
                        path,
                    }
                });
            }
            CharacterEditorMsg::SwitchPage(page) => {
                self.active_page = page;
            }
            CharacterEditorMsg::Save => {
                // Validate name is non-empty
                if self.character.name.trim().is_empty() {
                    self.name_row.add_css_class("error");
                    return;
                }
                self.name_row.remove_css_class("error");

                // Update timestamp
                self.character.updated_at = chrono::Local::now().naive_local();

                // If new character without a name (edge case), give it a default
                if self.is_new && self.character.name.is_empty() {
                    self.character.name = "Unnamed Character".to_string();
                }

                let _ = sender.output(CharacterEditorOutput::Saved(self.character.clone()));
                self.window.close();
            }
            CharacterEditorMsg::Cancel => {
                let _ = sender.output(CharacterEditorOutput::Cancelled);
                self.window.close();
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            CharacterEditorCmd::AvatarLoaded { bytes, path } => {
                self.avatar_bytes = Some(bytes);
                self.character.avatar_path = Some(path);
                self.avatar_button.set_label("Avatar selected ✓");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

impl CharacterEditorModel {
    /// Rebuild the tags list from the model data.
    fn rebuild_tags(&self, sender: &ComponentSender<Self>) {
        // Clear existing children
        while let Some(child) = self.tags_container.first_child() {
            self.tags_container.remove(&child);
        }

        for (i, tag) in self.character.tags.iter().enumerate() {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 4);

            let entry = gtk::Entry::new();
            entry.set_hexpand(true);
            entry.set_text(tag);
            entry.set_placeholder_text(Some("Tag"));

            let sender_clone = sender.clone();
            let idx = i;
            entry.connect_changed(move |e| {
                let _ = sender_clone.input_sender().send(CharacterEditorMsg::SetTag {
                    index: idx,
                    value: e.text().to_string(),
                });
            });

            let remove_btn = gtk::Button::with_label("×");
            remove_btn.add_css_class("flat");
            let sender_clone = sender.clone();
            let idx = i;
            remove_btn.connect_clicked(move |_| {
                let _ = sender_clone
                    .input_sender()
                    .send(CharacterEditorMsg::RemoveTag(idx));
            });

            row.append(&entry);
            row.append(&remove_btn);
            self.tags_container.append(&row);
        }
    }

    /// Rebuild the alternate greetings list from the model data.
    fn rebuild_greetings(&self, sender: &ComponentSender<Self>) {
        // Clear existing children
        while let Some(child) = self.greetings_container.first_child() {
            self.greetings_container.remove(&child);
        }

        for (i, greeting) in self.character.alternate_greetings.iter().enumerate() {
            let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
            row_box.set_margin_start(4);
            row_box.set_margin_end(4);
            row_box.set_margin_top(2);
            row_box.set_margin_bottom(2);

            let scrolled = gtk::ScrolledWindow::new();
            scrolled.set_min_content_height(60);
            scrolled.set_vexpand(false);
            scrolled.set_hexpand(true);

            let view = gtk::TextView::new();
            view.set_wrap_mode(gtk::WrapMode::WordChar);
            view.buffer().set_text(greeting);

            let sender_clone = sender.clone();
            let idx = i;
            let buffer = view.buffer();
            buffer.connect_changed(move |buf| {
                let (start, end) = buf.bounds();
                let _ = sender_clone
                    .input_sender()
                    .send(CharacterEditorMsg::SetAlternateGreeting {
                        index: idx,
                        value: buf.text(&start, &end, false).to_string(),
                    });
            });

            scrolled.set_child(Some(&view));

            let remove_btn = gtk::Button::with_label("×");
            remove_btn.add_css_class("flat");
            remove_btn.set_valign(gtk::Align::Start);
            let sender_clone = sender.clone();
            let idx = i;
            remove_btn.connect_clicked(move |_| {
                let _ = sender_clone
                    .input_sender()
                    .send(CharacterEditorMsg::RemoveAlternateGreeting(idx));
            });

            row_box.append(&scrolled);
            row_box.append(&remove_btn);

            let row = gtk::ListBoxRow::new();
            row.set_child(Some(&row_box));
            row.set_activatable(false);
            row.set_selectable(false);

            self.greetings_container.append(&row);
        }
    }
}
