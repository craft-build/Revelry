use adw::prelude::*;
use relm4::prelude::*;
use relm4::{adw, gtk, ComponentParts, ComponentSender};

use crate::domain::generation::GenerationPreset;

/// Preset editor component for creating and editing generation presets.
/// Uses AdwPreferencesWindow sub-dialog pattern (same as PersonaEditor).
pub struct PresetEditorModel {
    preset: GenerationPreset,
    is_new: bool,
    is_duplicate: bool,
    window: adw::PreferencesWindow,
    name_row: adw::EntryRow,
    temperature_row: adw::SpinRow,
    top_p_row: adw::SpinRow,
    max_tokens_row: adw::SpinRow,
    frequency_penalty_row: adw::SpinRow,
    presence_penalty_row: adw::SpinRow,
    stop_sequences_row: adw::EntryRow,
}

#[derive(Debug)]
pub enum PresetEditorMsg {
    Save,
    Cancel,
}

#[derive(Debug)]
pub enum PresetEditorOutput {
    Saved(GenerationPreset),
    Cancelled,
}

pub enum PresetEditorInit {
    New,
    Edit(GenerationPreset),
    Duplicate(GenerationPreset),
}

#[relm4::component(pub)]
impl Component for PresetEditorModel {
    type Init = PresetEditorInit;
    type Input = PresetEditorMsg;
    type Output = PresetEditorOutput;
    type CommandOutput = ();

    view! {
        #[root]
        adw::PreferencesWindow {
            set_modal: true,
            set_default_width: 500,
            set_default_height: 600,

            add = &adw::PreferencesPage {
                adw::PreferencesGroup {
                    set_title: "Preset Details",

                    #[name(name_row)]
                    adw::EntryRow {
                        set_title: "Name",
                    },
                },

                adw::PreferencesGroup {
                    set_title: "Sampling Parameters",

                    #[name(temperature_row)]
                    adw::SpinRow {
                        set_title: "Temperature",
                        set_subtitle: "Controls randomness. Higher = more creative, lower = more focused.",
                    },

                    #[name(top_p_row)]
                    adw::SpinRow {
                        set_title: "Top-P",
                        set_subtitle: "Nucleus sampling threshold. Lower = more focused vocabulary.",
                    },

                    #[name(max_tokens_row)]
                    adw::SpinRow {
                        set_title: "Max Tokens",
                        set_subtitle: "Maximum tokens in the AI response. 0 = no limit.",
                    },

                    #[name(frequency_penalty_row)]
                    adw::SpinRow {
                        set_title: "Frequency Penalty",
                        set_subtitle: "Reduces repetition of frequent tokens. 0 = no penalty.",
                    },

                    #[name(presence_penalty_row)]
                    adw::SpinRow {
                        set_title: "Presence Penalty",
                        set_subtitle: "Encourages new topics. 0 = no penalty.",
                    },

                    gtk::Label {
                        set_label: "Not all providers support all parameters. Unsupported parameters are silently skipped.",
                        add_css_class: "caption",
                        add_css_class: "dim-label",
                        set_wrap: true,
                        set_xalign: 0.0,
                        set_margin_top: 4,
                        set_margin_bottom: 4,
                    },
                },

                adw::PreferencesGroup {
                    set_title: "Stop Sequences",

                    #[name(stop_sequences_row)]
                    adw::EntryRow {
                        set_title: "Stop Sequences",
                    },

                    gtk::Label {
                        set_label: "Leave empty for no stop sequences. Most providers support up to 4 sequences.",
                        add_css_class: "caption",
                        add_css_class: "dim-label",
                        set_wrap: true,
                        set_xalign: 0.0,
                        set_margin_top: 4,
                        set_margin_bottom: 4,
                    },
                },

                adw::PreferencesGroup {
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 12,
                        set_halign: gtk::Align::End,

                        gtk::Button {
                            set_label: "Cancel",
                            add_css_class: "flat",
                            connect_clicked => PresetEditorMsg::Cancel,
                        },

                        gtk::Button {
                            set_label: "Save Preset",
                            add_css_class: "suggested-action",
                            connect_clicked => PresetEditorMsg::Save,
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

        let (preset, is_new, is_duplicate) = match init {
            PresetEditorInit::New => (GenerationPreset::new(String::new()), true, false),
            PresetEditorInit::Edit(p) => (p, false, false),
            PresetEditorInit::Duplicate(p) => {
                let mut dup = GenerationPreset::new(format!("{} Copy", p.name));
                dup.temperature = p.temperature;
                dup.top_p = p.top_p;
                dup.max_tokens = p.max_tokens;
                dup.frequency_penalty = p.frequency_penalty;
                dup.presence_penalty = p.presence_penalty;
                dup.stop_sequences = p.stop_sequences.clone();
                (dup, true, true)
            }
        };

        if is_new && !is_duplicate {
            root.set_title(Some("New Preset"));
        } else if is_duplicate {
            root.set_title(Some("New Preset"));
        } else {
            root.set_title(Some(&format!("Edit Preset")));
        }

        // Pre-fill fields
        widgets.name_row.set_text(&preset.name);

        // Set up SpinRow adjustments
        let temp_adj = gtk::Adjustment::new(preset.temperature, 0.0, 2.0, 0.1, 0.1, 0.0);
        widgets.temperature_row.set_adjustment(Some(&temp_adj));

        let top_p_adj = gtk::Adjustment::new(preset.top_p, 0.0, 1.0, 0.05, 0.05, 0.0);
        widgets.top_p_row.set_adjustment(Some(&top_p_adj));

        let max_tokens_adj = gtk::Adjustment::new(preset.max_tokens as f64, 0.0, 128000.0, 1.0, 100.0, 0.0);
        widgets.max_tokens_row.set_adjustment(Some(&max_tokens_adj));

        let freq_adj = gtk::Adjustment::new(preset.frequency_penalty, 0.0, 2.0, 0.1, 0.1, 0.0);
        widgets.frequency_penalty_row.set_adjustment(Some(&freq_adj));

        let pres_adj = gtk::Adjustment::new(preset.presence_penalty, 0.0, 2.0, 0.1, 0.1, 0.0);
        widgets.presence_penalty_row.set_adjustment(Some(&pres_adj));

        // Stop sequences: join with commas
        let stop_str = preset.stop_sequences.join(", ");
        widgets.stop_sequences_row.set_text(&stop_str);

        let model = PresetEditorModel {
            preset,
            is_new,
            is_duplicate,
            window: root.clone(),
            name_row: widgets.name_row.clone(),
            temperature_row: widgets.temperature_row.clone(),
            top_p_row: widgets.top_p_row.clone(),
            max_tokens_row: widgets.max_tokens_row.clone(),
            frequency_penalty_row: widgets.frequency_penalty_row.clone(),
            presence_penalty_row: widgets.presence_penalty_row.clone(),
            stop_sequences_row: widgets.stop_sequences_row.clone(),
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
            PresetEditorMsg::Save => {
                // Read fields
                let name = self.name_row.text().to_string();

                // Validate name
                if name.trim().is_empty() {
                    self.name_row.add_css_class("error");
                    return;
                }
                self.name_row.remove_css_class("error");

                // Validate name doesn't duplicate built-in
                let builtin_names = ["Creative", "Balanced", "Precise"];
                if builtin_names.contains(&name.as_str()) {
                    self.name_row.add_css_class("error");
                    return;
                }
                self.name_row.remove_css_class("error");

                // Read spin row values
                let temperature = self.temperature_row.value();
                let top_p = self.top_p_row.value();
                let max_tokens = self.max_tokens_row.value() as i64;
                let frequency_penalty = self.frequency_penalty_row.value();
                let presence_penalty = self.presence_penalty_row.value();

                // Parse stop sequences
                let stop_text = self.stop_sequences_row.text().to_string();
                let stop_sequences: Vec<String> = stop_text
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                // Update preset
                self.preset.name = name;
                self.preset.temperature = temperature;
                self.preset.top_p = top_p;
                self.preset.max_tokens = max_tokens;
                self.preset.frequency_penalty = frequency_penalty;
                self.preset.presence_penalty = presence_penalty;
                self.preset.stop_sequences = stop_sequences;
                self.preset.updated_at = chrono::Local::now().naive_local();

                let _ = sender.output(PresetEditorOutput::Saved(self.preset.clone()));
                self.window.close();
            }
            PresetEditorMsg::Cancel => {
                let _ = sender.output(PresetEditorOutput::Cancelled);
                self.window.close();
            }
        }
    }
}
