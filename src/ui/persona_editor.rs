use adw::prelude::*;
use relm4::prelude::*;
use relm4::{adw, gtk, ComponentParts, ComponentSender};

use crate::domain::persona::Persona;

/// Persona editor component for creating and editing personas.
pub struct PersonaEditorModel {
    persona: Persona,
    is_new: bool,
    window: adw::PreferencesWindow,
    name_row: adw::EntryRow,
    appearance_row: adw::EntryRow,
    speech_style_row: adw::EntryRow,
    role_row: adw::EntryRow,
    tags_row: adw::EntryRow,
    personality_buffer: gtk::TextBuffer,
    background_buffer: gtk::TextBuffer,
    relationships_buffer: gtk::TextBuffer,
}

#[derive(Debug)]
pub enum PersonaEditorMsg {
    Save,
    Cancel,
}

#[derive(Debug)]
pub enum PersonaEditorOutput {
    Saved(Persona),
    Cancelled,
}

pub enum PersonaEditorInit {
    New,
    Edit(Persona),
}

#[relm4::component(pub)]
impl Component for PersonaEditorModel {
    type Init = PersonaEditorInit;
    type Input = PersonaEditorMsg;
    type Output = PersonaEditorOutput;
    type CommandOutput = ();

    view! {
        #[root]
        adw::PreferencesWindow {
            set_title: Some("Persona Editor"),
            set_modal: true,
            set_default_width: 500,
            set_default_height: 600,

            // Identity Page
            add = &adw::PreferencesPage {
                set_title: "Identity",
                set_icon_name: Some("contact-page"),

                adw::PreferencesGroup {
                    set_title: "Basic Info",
                    set_description: Some("Name and personality are required."),

                    #[name(name_row)]
                    adw::EntryRow {
                        set_title: "Name*",
                    },

                    #[name(appearance_row)]
                    adw::EntryRow {
                        set_title: "Appearance",
                    },
                },

                adw::PreferencesGroup {
                    set_title: "Personality",

                    gtk::ScrolledWindow {
                        set_min_content_height: 80,
                        set_vexpand: false,

                        #[name(personality_view)]
                        gtk::TextView {
                            set_wrap_mode: gtk::WrapMode::WordChar,
                        },
                    },
                },

                adw::PreferencesGroup {
                    set_title: "Speech Style",

                    #[name(speech_style_row)]
                    adw::EntryRow {
                        set_title: "Speech Style",
                    },
                },
            },

            // Background Page
            add = &adw::PreferencesPage {
                set_title: "Background",
                set_icon_name: Some("mood"),

                adw::PreferencesGroup {
                    set_title: "Background Story",

                    gtk::ScrolledWindow {
                        set_min_content_height: 120,
                        set_vexpand: false,

                        #[name(background_view)]
                        gtk::TextView {
                            set_wrap_mode: gtk::WrapMode::WordChar,
                        },
                    },
                },

                adw::PreferencesGroup {
                    set_title: "Role / Occupation",

                    #[name(role_row)]
                    adw::EntryRow {
                        set_title: "Role",
                    },
                },
            },

            // Relationships Page
            add = &adw::PreferencesPage {
                set_title: "Relationships",
                set_icon_name: Some("emblem-people"),

                adw::PreferencesGroup {
                    set_title: "Relationships",

                    gtk::ScrolledWindow {
                        set_min_content_height: 80,
                        set_vexpand: false,

                        #[name(relationships_view)]
                        gtk::TextView {
                            set_wrap_mode: gtk::WrapMode::WordChar,
                        },
                    },
                },

                adw::PreferencesGroup {
                    set_title: "Tags",

                    #[name(tags_row)]
                    adw::EntryRow {
                        set_title: "Tags (comma-separated)",
                    },
                },
            },

            // Review/Save Page
            add = &adw::PreferencesPage {
                set_title: "Review",
                set_icon_name: Some("checkmark"),

                adw::PreferencesGroup {
                    set_title: "Review Your Persona",

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 12,
                        set_halign: gtk::Align::End,

                        gtk::Button {
                            set_label: "Cancel",
                            connect_clicked => PersonaEditorMsg::Cancel,
                        },

                        gtk::Button {
                            set_label: "Save",
                            add_css_class: "suggested-action",
                            connect_clicked => PersonaEditorMsg::Save,
                        },
                    },
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widgets = view_output!();

        let (persona, is_new) = match init {
            PersonaEditorInit::New => (Persona::new(String::new(), String::new()), true),
            PersonaEditorInit::Edit(p) => (p, false),
        };

        if is_new {
            root.set_title(Some("New Persona"));
        } else {
            root.set_title(Some(&format!("Edit: {}", persona.name)));
        }

        // Pre-fill fields
        widgets.name_row.set_text(&persona.name);
        widgets.appearance_row.set_text(&persona.appearance);
        widgets.speech_style_row.set_text(&persona.speech_style);
        widgets.role_row.set_text(&persona.role_description);
        widgets.tags_row.set_text(&persona.tags.join(", "));
        widgets.personality_view.buffer().set_text(&persona.personality);
        widgets.background_view.buffer().set_text(&persona.background);
        widgets.relationships_view.buffer().set_text(&persona.relationships);

        let model = PersonaEditorModel {
            persona,
            is_new,
            window: root.clone(),
            name_row: widgets.name_row.clone(),
            appearance_row: widgets.appearance_row.clone(),
            speech_style_row: widgets.speech_style_row.clone(),
            role_row: widgets.role_row.clone(),
            tags_row: widgets.tags_row.clone(),
            personality_buffer: widgets.personality_view.buffer(),
            background_buffer: widgets.background_view.buffer(),
            relationships_buffer: widgets.relationships_view.buffer(),
        };

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            PersonaEditorMsg::Save => {
                // Read fields
                let name = self.name_row.text().to_string();
                let appearance = self.appearance_row.text().to_string();
                let speech_style = self.speech_style_row.text().to_string();
                let role_description = self.role_row.text().to_string();
                let tags_str = self.tags_row.text().to_string();

                let personality = self.personality_buffer.text(
                    &self.personality_buffer.start_iter(),
                    &self.personality_buffer.end_iter(),
                    true,
                ).to_string();
                let background = self.background_buffer.text(
                    &self.background_buffer.start_iter(),
                    &self.background_buffer.end_iter(),
                    true,
                ).to_string();
                let relationships = self.relationships_buffer.text(
                    &self.relationships_buffer.start_iter(),
                    &self.relationships_buffer.end_iter(),
                    true,
                ).to_string();

                // Validate required fields
                if name.trim().is_empty() || personality.trim().is_empty() {
                    self.name_row.add_css_class("error");
                    return;
                }
                self.name_row.remove_css_class("error");

                // Update persona
                self.persona.name = name;
                self.persona.appearance = appearance;
                self.persona.personality = personality;
                self.persona.background = background;
                self.persona.role_description = role_description;
                self.persona.relationships = relationships;
                self.persona.speech_style = speech_style;
                self.persona.tags = tags_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                self.persona.updated_at = chrono::Local::now().naive_local();

                let _ = sender.output(PersonaEditorOutput::Saved(self.persona.clone()));
                self.window.close();
            }
            PersonaEditorMsg::Cancel => {
                let _ = sender.output(PersonaEditorOutput::Cancelled);
                self.window.close();
            }
        }
    }
}
