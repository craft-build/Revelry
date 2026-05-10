use adw::prelude::*;
use relm4::prelude::*;
use relm4::{adw, gtk, ComponentParts, ComponentSender};

use crate::domain::lorebook::{InsertionPosition, LorebookEntry};

/// Lorebook editor component — manages a list of lorebook entries for one character.
/// Per D-08: keyword-activated text injection entries with full CRUD.
pub struct LorebookEditorModel {
    entries: Vec<LorebookEntry>,
    character_id: String,
    editing_entry: Option<LorebookEntry>,
    // Widget references for imperative updates
    entry_listbox: gtk::ListBox,
    window: adw::PreferencesWindow,
}

/// Fields that can be updated in the lorebook entry editor.
#[derive(Debug, Clone)]
pub enum LorebookField {
    Name,
    Keywords,
    SecondaryTerms,
    Content,
    DepthFromBottom,
    TokenBudget,
    InsertionPosition,
    Enabled,
}

/// Messages handled by LorebookEditor.
#[derive(Debug)]
pub enum LorebookEditorMsg {
    SetEntries(Vec<LorebookEntry>),
    AddEntry,
    EditEntry(String),   // entry ID
    SaveEntry,
    DeleteEntry(String), // entry ID
    UpdateField(LorebookField, String),
    ConfirmDelete(String),
}

/// Output messages from LorebookEditor to parent (AppModel).
#[derive(Debug)]
pub enum LorebookEditorOutput {
    SaveEntry(LorebookEntry),
    DeleteEntry(String, String), // (entry_id, character_id)
}

#[relm4::component(pub)]
impl Component for LorebookEditorModel {
    type Init = String; // character_id
    type Input = LorebookEditorMsg;
    type Output = LorebookEditorOutput;
    type CommandOutput = ();

    view! {
        #[root]
        adw::PreferencesWindow {
            set_title: Some("Lorebook Editor"),
            set_modal: true,

            // Entry list page
            add = &adw::PreferencesPage {
                set_title: "World Info Entries",

                adw::PreferencesGroup {
                    set_title: "World Info Entries",
                    set_description: Some("Keyword-activated entries that inject world-building content into the AI context when keywords appear in recent messages."),

                    #[local_ref]
                    entry_listbox_widget -> gtk::ListBox {
                        add_css_class: "boxed-list",
                        set_selection_mode: gtk::SelectionMode::None,
                    },

                    gtk::Button {
                        set_label: "Add Entry",
                        add_css_class: "flat",
                        set_halign: gtk::Align::Start,
                        set_margin_top: 8,
                        connect_clicked => LorebookEditorMsg::AddEntry,
                    },
                },

                adw::PreferencesGroup {
                    set_title: "Activation Settings",

                    gtk::Label {
                        set_label: "Scan Depth and Recursive Activation Depth are configured in the character editor's Lorebook tab.",
                        add_css_class: "caption",
                        add_css_class: "dim-label",
                        set_wrap: true,
                        set_xalign: 0.0,
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
        let character_id = init;
        let entry_listbox = gtk::ListBox::new();
        entry_listbox.add_css_class("boxed-list");
        entry_listbox.set_selection_mode(gtk::SelectionMode::None);

        let model = LorebookEditorModel {
            entries: Vec::new(),
            character_id,
            editing_entry: None,
            entry_listbox: entry_listbox.clone(),
            window: root.clone(),
        };

        let entry_listbox_widget = &entry_listbox;

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            LorebookEditorMsg::SetEntries(entries) => {
                self.entries = entries;
                self.rebuild_entry_list(&sender);
            }
            LorebookEditorMsg::AddEntry => {
                // Create a new blank entry for editing
                let new_entry = LorebookEntry::new(
                    self.character_id.clone(),
                    String::new(),
                    Vec::new(),
                    String::new(),
                );
                self.editing_entry = Some(new_entry);
                self.open_entry_editor(&sender);
            }
            LorebookEditorMsg::EditEntry(id) => {
                if let Some(entry) = self.entries.iter().find(|e| e.id == id).cloned() {
                    self.editing_entry = Some(entry);
                    self.open_entry_editor(&sender);
                }
            }
            LorebookEditorMsg::SaveEntry => {
                if let Some(entry) = self.editing_entry.take() {
                    // Validate per UI-SPEC: name required, keywords required, content required
                    if entry.name.is_empty() || entry.keywords.is_empty() || entry.content.is_empty() {
                        // Could show error toast, but for now just don't save invalid entries
                        return;
                    }
                    let _ = sender.output(LorebookEditorOutput::SaveEntry(entry));
                    // Reload entries will come back from DB
                }
            }
            LorebookEditorMsg::DeleteEntry(id) => {
                // Show confirmation dialog per UI-SPEC
                let dialog = adw::AlertDialog::new(
                    Some("Delete Entry?"),
                    Some("This lorebook entry will be permanently deleted."),
                );
                dialog.add_response("cancel", "Cancel");
                dialog.add_response("delete", "Delete");
                dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
                dialog.set_close_response("cancel");

                let sender_clone = sender.clone();
                let id_clone = id.clone();
                dialog.connect_response(None, move |_, response| {
                    if response == "delete" {
                        let _ = sender_clone.input_sender().send(LorebookEditorMsg::ConfirmDelete(id_clone.clone()));
                    }
                });
                dialog.present(Some(&self.window));
            }
            LorebookEditorMsg::ConfirmDelete(id) => {
                let _ = sender.output(LorebookEditorOutput::DeleteEntry(id, self.character_id.clone()));
            }
            LorebookEditorMsg::UpdateField(field, value) => {
                if let Some(ref mut entry) = self.editing_entry {
                    match field {
                        LorebookField::Name => entry.name = value,
                        LorebookField::Keywords => {
                            entry.keywords = value.split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect();
                        }
                        LorebookField::SecondaryTerms => {
                            entry.secondary_terms = value.split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect();
                        }
                        LorebookField::Content => entry.content = value,
                        LorebookField::DepthFromBottom => {
                            entry.depth_from_bottom = value.parse().unwrap_or(4);
                        }
                        LorebookField::TokenBudget => {
                            entry.token_budget = if value.is_empty() || value == "0" {
                                None
                            } else {
                                value.parse().ok()
                            };
                        }
                        LorebookField::InsertionPosition => {
                            entry.insertion_position = if value == "before_char" {
                                InsertionPosition::BeforeChar
                            } else {
                                InsertionPosition::AfterChar
                            };
                        }
                        LorebookField::Enabled => {
                            entry.enabled = value != "false";
                        }
                    }
                }
            }
        }
    }
}

impl LorebookEditorModel {
    /// Rebuild the entry list in the listbox.
    fn rebuild_entry_list(&self, sender: &ComponentSender<Self>) {
        // Clear existing children
        while let Some(child) = self.entry_listbox.first_child() {
            self.entry_listbox.remove(&child);
        }

        if self.entries.is_empty() {
            let label = gtk::Label::new(Some("No lorebook entries yet. Click \"Add Entry\" to create one."));
            label.add_css_class("dim-label");
            label.set_margin_top(12);
            label.set_margin_bottom(12);
            let row = adw::ActionRow::builder().build();
            row.add_suffix(&label);
            self.entry_listbox.append(&row);
            return;
        }

        for entry in &self.entries {
            let row = adw::ActionRow::builder()
                .title(&entry.name)
                .subtitle(&format!("Keywords: {} | Depth: {} | {}",
                    entry.keywords.join(", "),
                    entry.depth_from_bottom,
                    if entry.enabled { "Enabled" } else { "Disabled" }
                ))
                .build();

            // Enabled toggle
            let switch = gtk::Switch::new();
            switch.set_active(entry.enabled);
            switch.set_valign(gtk::Align::Center);
            let entry_id = entry.id.clone();
            let sender_clone = sender.clone();
            // Note: toggling enabled is a save operation — for now, clicking Edit is required

            // Edit button
            let edit_btn = gtk::Button::from_icon_name("edit");
            edit_btn.add_css_class("flat");
            edit_btn.set_valign(gtk::Align::Center);
            let edit_id = entry.id.clone();
            let sender_clone = sender.clone();
            edit_btn.connect_clicked(move |_| {
                let _ = sender_clone.input_sender().send(LorebookEditorMsg::EditEntry(edit_id.clone()));
            });

            // Delete button
            let del_btn = gtk::Button::from_icon_name("user-trash");
            del_btn.add_css_class("flat");
            del_btn.set_valign(gtk::Align::Center);
            let del_id = entry.id.clone();
            let sender_clone = sender.clone();
            del_btn.connect_clicked(move |_| {
                let _ = sender_clone.input_sender().send(LorebookEditorMsg::DeleteEntry(del_id.clone()));
            });

            row.add_suffix(&edit_btn);
            row.add_suffix(&del_btn);

            self.entry_listbox.append(&row);
        }
    }

    /// Open a sub-dialog for editing a lorebook entry.
    fn open_entry_editor(&self, sender: &ComponentSender<Self>) {
        let Some(ref entry) = self.editing_entry else {
            return;
        };

        let dialog = adw::PreferencesWindow::new();
        dialog.set_title(Some("Edit Lorebook Entry"));
        dialog.set_modal(true);
        dialog.set_transient_for(Some(&self.window));

        let page = adw::PreferencesPage::new();

        // Entry Details group
        let details_group = adw::PreferencesGroup::new();
        details_group.set_title("Entry Details");

        let name_row = adw::EntryRow::new();
        name_row.set_title("Name");
        name_row.set_text(&entry.name);
        let sender_clone = sender.clone();
        name_row.connect_changed(move |row| {
            let _ = sender_clone.input_sender().send(LorebookEditorMsg::UpdateField(
                LorebookField::Name,
                row.text().to_string(),
            ));
        });

        let keywords_row = adw::EntryRow::new();
        keywords_row.set_title("Keywords");
        keywords_row.set_text(&entry.keywords.join(", "));
        let sender_clone = sender.clone();
        keywords_row.connect_changed(move |row| {
            let _ = sender_clone.input_sender().send(LorebookEditorMsg::UpdateField(
                LorebookField::Keywords,
                row.text().to_string(),
            ));
        });

        let secondary_row = adw::EntryRow::new();
        secondary_row.set_title("Secondary Terms");
        secondary_row.set_text(&entry.secondary_terms.join(", "));
        let sender_clone = sender.clone();
        secondary_row.connect_changed(move |row| {
            let _ = sender_clone.input_sender().send(LorebookEditorMsg::UpdateField(
                LorebookField::SecondaryTerms,
                row.text().to_string(),
            ));
        });

        let enabled_row = adw::SwitchRow::new();
        enabled_row.set_title("Enabled");
        enabled_row.set_active(entry.enabled);
        let sender_clone = sender.clone();
        enabled_row.connect_active_notify(move |row| {
            let _ = sender_clone.input_sender().send(LorebookEditorMsg::UpdateField(
                LorebookField::Enabled,
                if row.is_active() { "true" } else { "false" }.to_string(),
            ));
        });

        details_group.add(&name_row);
        details_group.add(&keywords_row);
        details_group.add(&secondary_row);
        details_group.add(&enabled_row);

        // Content group
        let content_group = adw::PreferencesGroup::new();
        content_group.set_title("Content");

        let content_buffer = gtk::TextBuffer::new(None);
        content_buffer.set_text(&entry.content);
        let content_view = gtk::TextView::new();
        content_view.set_buffer(Some(&content_buffer));
        content_view.set_wrap_mode(gtk::WrapMode::WordChar);

        let content_scroll = gtk::ScrolledWindow::new();
        content_scroll.set_min_content_height(120);
        content_scroll.set_child(Some(&content_view));

        let sender_clone = sender.clone();
        content_buffer.connect_changed(move |buf| {
            let text = buf.text(&buf.start_iter(), &buf.end_iter(), true);
            let _ = sender_clone.input_sender().send(LorebookEditorMsg::UpdateField(
                LorebookField::Content,
                text.to_string(),
            ));
        });

        let content_row = adw::PreferencesRow::new();
        content_row.set_child(Some(&content_scroll));
        content_group.add(&content_row);

        // Insertion group
        let insertion_group = adw::PreferencesGroup::new();
        insertion_group.set_title("Insertion");

        let depth_adjustment = gtk::Adjustment::new(entry.depth_from_bottom as f64, 0.0, 999.0, 1.0, 10.0, 0.0);
        let depth_spin = gtk::SpinButton::new(Some(&depth_adjustment), 1.0, 0);
        let depth_row = adw::PreferencesRow::new();
        depth_row.set_title("Depth from Bottom");
        let depth_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        let depth_label = gtk::Label::new(Some("Number of messages from the end to insert this entry. 0 = after last message."));
        depth_label.add_css_class("caption");
        depth_label.add_css_class("dim-label");
        depth_label.set_wrap(true);
        depth_label.set_xalign(0.0);
        depth_box.append(&depth_spin);
        depth_box.append(&depth_label);
        depth_row.set_child(Some(&depth_box));
        let sender_clone = sender.clone();
        depth_adjustment.connect_value_changed(move |adj| {
            let _ = sender_clone.input_sender().send(LorebookEditorMsg::UpdateField(
                LorebookField::DepthFromBottom,
                (adj.value() as i32).to_string(),
            ));
        });

        let position_row = adw::ComboRow::new();
        position_row.set_title("Insertion Position");
        let position_model = gtk::StringList::new(&["After Character", "Before Character"]);
        position_row.set_model(Some(&position_model));
        position_row.set_selected(if entry.insertion_position == InsertionPosition::BeforeChar { 1 } else { 0 });
        let sender_clone = sender.clone();
        position_row.connect_selected_notify(move |combo| {
            let pos = if combo.selected() == 1 { "before_char" } else { "after_char" };
            let _ = sender_clone.input_sender().send(LorebookEditorMsg::UpdateField(
                LorebookField::InsertionPosition,
                pos.to_string(),
            ));
        });

        insertion_group.add(&depth_row);
        insertion_group.add(&position_row);

        // Budget group
        let budget_group = adw::PreferencesGroup::new();
        budget_group.set_title("Budget");

        let budget_row = adw::EntryRow::new();
        budget_row.set_title("Token Budget");
        budget_row.set_text(&entry.token_budget.map(|b| b.to_string()).unwrap_or_default());
        let budget_tip = gtk::Label::new(Some("Maximum tokens for this entry. Leave empty for unlimited. Content is truncated from the end if exceeded."));
        budget_tip.add_css_class("caption");
        budget_tip.add_css_class("dim-label");
        budget_tip.set_wrap(true);
        budget_tip.set_xalign(0.0);
        let budget_row_with_tip = adw::PreferencesRow::new();
        let budget_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        budget_box.append(&budget_row);
        budget_box.append(&budget_tip);
        budget_row_with_tip.set_child(Some(&budget_box));
        let sender_clone = sender.clone();
        budget_row.connect_changed(move |row| {
            let _ = sender_clone.input_sender().send(LorebookEditorMsg::UpdateField(
                LorebookField::TokenBudget,
                row.text().to_string(),
            ));
        });

        budget_group.add(&budget_row_with_tip);

        // Save/Cancel group
        let actions_group = adw::PreferencesGroup::new();
        let actions_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);

        let cancel_btn = gtk::Button::with_label("Cancel");
        cancel_btn.add_css_class("flat");
        cancel_btn.set_hexpand(true);
        let dialog_clone = dialog.clone();
        cancel_btn.connect_clicked(move |_| {
            dialog_clone.close();
        });

        let save_btn = gtk::Button::with_label("Save Entry");
        save_btn.add_css_class("suggested-action");
        save_btn.set_hexpand(true);
        let sender_clone = sender.clone();
        let dialog_clone = dialog.clone();
        save_btn.connect_clicked(move |_| {
            let _ = sender_clone.input_sender().send(LorebookEditorMsg::SaveEntry);
            dialog_clone.close();
        });

        actions_box.append(&cancel_btn);
        actions_box.append(&save_btn);
        actions_group.add(&actions_box);

        page.add(&details_group);
        page.add(&content_group);
        page.add(&insertion_group);
        page.add(&budget_group);
        page.add(&actions_group);

        dialog.add(&page);
        dialog.present();
    }
}
