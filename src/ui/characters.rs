use adw::prelude::*;
use relm4::prelude::*;
use relm4::{adw, gtk, ComponentParts, ComponentSender};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::domain::character::CharacterCardV2;
use crate::ui::character_card::create_card_widget;

/// Characters panel showing character library with search, tabs, and import/export.
pub struct CharactersPanelModel {
    characters: Vec<CharacterCardV2>,
    search_query: Rc<RefCell<String>>,
    searchable_map: Rc<RefCell<HashMap<String, String>>>,
    active_tab: CharacterTab,

    flowbox: gtk::FlowBox,
    empty_state: adw::StatusPage,
}

/// Tab identifiers for the character/persona switcher.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CharacterTab {
    Characters,
    Personas,
}

/// Messages handled by CharactersPanelModel.
#[derive(Debug)]
pub enum CharactersPanelMsg {
    CharactersLoaded(Vec<CharacterCardV2>),
    SearchChanged(String),
    NewCharacter,
    ImportCharacter,
    ImportResult(Result<CharacterCardV2, String>),
    ExportCharacter(String),
    ExportComplete(Result<(), String>),
    DeleteCharacter(String),
    ConfirmDelete(String),
    EditCharacter(String),
    StartConversation(String),
    SwitchTab(CharacterTab),
}

/// Command output from background threads (reserved for future file operations).
#[derive(Debug)]
pub enum CharactersPanelCmd {}

/// Output messages from CharactersPanelModel to parent (AppModel).
/// The parent handles all DB operations since it owns the DbService Worker.
#[derive(Debug)]
pub enum CharactersPanelOutput {
    SaveCharacter(CharacterCardV2),
    DeleteCharacter(String),
    EditCharacter(String),
    StartConversation(String),
    RequestImport,
    RequestExport(String),
}

#[relm4::component(pub)]
impl Component for CharactersPanelModel {
    type Init = ();
    type Input = CharactersPanelMsg;
    type Output = CharactersPanelOutput;
    type CommandOutput = CharactersPanelCmd;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            // Top bar: search + action buttons
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 6,
                set_margin_start: 8,
                set_margin_end: 8,
                set_margin_top: 8,
                set_margin_bottom: 4,

                #[name(search_entry)]
                gtk::SearchEntry {
                    set_hexpand: true,
                    set_placeholder_text: Some("Search characters..."),
                    connect_search_changed[sender] => move |entry| {
                        let query = entry.text().to_string();
                        let _ = sender.input_sender().send(CharactersPanelMsg::SearchChanged(query));
                    },
                },

                gtk::Button {
                    set_icon_name: "add",
                    set_tooltip_text: Some("New Character"),
                    connect_clicked => CharactersPanelMsg::NewCharacter,
                },

                gtk::Button {
                    set_icon_name: "folder-open",
                    set_tooltip_text: Some("Import Character"),
                    connect_clicked => CharactersPanelMsg::ImportCharacter,
                },
            },

            // Content area with adw::ViewStack for tab pages
            #[name(content_stack)]
            adw::ViewStack {
                set_vexpand: true,
            },

            // Tab bar for Characters / Personas
            #[name(tab_bar)]
            adw::ViewSwitcher {
                set_stack: Some(&content_stack),
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widgets = view_output!();

        // Build the characters page content
        let characters_page_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let flowbox = gtk::FlowBox::new();
        flowbox.set_min_children_per_line(2);
        flowbox.set_max_children_per_line(6);
        flowbox.set_homogeneous(false);
        flowbox.set_selection_mode(gtk::SelectionMode::None);
        flowbox.set_margin_start(8);
        flowbox.set_margin_end(8);
        flowbox.set_margin_top(4);
        flowbox.set_margin_bottom(8);

        let search_query = Rc::new(RefCell::new(String::new()));
        let searchable_map: Rc<RefCell<HashMap<String, String>>> =
            Rc::new(RefCell::new(HashMap::new()));

        // Set up FlowBox filter function that reads from shared state
        let search_query_clone = search_query.clone();
        let searchable_map_clone = searchable_map.clone();
        flowbox.set_filter_func(move |child| {
            let query = search_query_clone.borrow();
            if query.is_empty() {
                return true;
            }
            let q = query.to_lowercase();
            let id = child.widget_name().to_string();
            let map = searchable_map_clone.borrow();
            map.get(&id)
                .map(|s| s.to_lowercase().contains(&q))
                .unwrap_or(false)
        });

        // Connect child_activated for starting conversation
        let sender_clone = sender.clone();
        flowbox.connect_child_activated(move |_, child| {
            let id = child.widget_name().to_string();
            let _ = sender_clone
                .input_sender()
                .send(CharactersPanelMsg::StartConversation(id));
        });

        let empty_state = adw::StatusPage::new();
        empty_state.set_icon_name(Some("contact-page"));
        empty_state.set_title("No Characters Yet");
        empty_state.set_description(Some(
            "Import a character card or create a new one to get started.",
        ));
        empty_state.set_vexpand(true);

        characters_page_box.append(&flowbox);
        characters_page_box.append(&empty_state);

        // Build the personas page (placeholder)
        let personas_page = adw::StatusPage::new();
        personas_page.set_icon_name(Some("mood"));
        personas_page.set_title("My Personas");
        personas_page.set_description(Some(
            "Personas are coming in a future update. They'll let you define your own identity for roleplay.",
        ));
        personas_page.set_vexpand(true);

        // Add pages to ViewStack
        let char_page = widgets
            .content_stack
            .add_named(&characters_page_box, Some("characters"));
        char_page.set_title(Some("Characters"));
        char_page.set_icon_name(Some("list"));

        let pers_page = widgets
            .content_stack
            .add_named(&personas_page, Some("personas"));
        pers_page.set_title(Some("My Personas"));
        pers_page.set_icon_name(Some("mood"));

        // Listen for tab changes
        let sender_clone = sender.clone();
        widgets
            .content_stack
            .connect_visible_child_name_notify(move |stack| {
                if let Some(name) = stack.visible_child_name() {
                    let tab = if name == "personas" {
                        CharacterTab::Personas
                    } else {
                        CharacterTab::Characters
                    };
                    let _ = sender_clone
                        .input_sender()
                        .send(CharactersPanelMsg::SwitchTab(tab));
                }
            });

        let model = CharactersPanelModel {
            characters: Vec::new(),
            search_query,
            searchable_map,
            active_tab: CharacterTab::Characters,

            flowbox,
            empty_state,
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
            CharactersPanelMsg::CharactersLoaded(chars) => {
                self.characters = chars;
                self.rebuild_flowbox(&sender);
            }
            CharactersPanelMsg::SearchChanged(query) => {
                *self.search_query.borrow_mut() = query;
                self.flowbox.invalidate_filter();
                self.update_empty_state();
            }
            CharactersPanelMsg::NewCharacter => {
                let _ = sender.output(CharactersPanelOutput::EditCharacter(String::new()));
            }
            CharactersPanelMsg::ImportCharacter => {
                let _ = sender.output(CharactersPanelOutput::RequestImport);
            }
            CharactersPanelMsg::ImportResult(result) => match result {
                Ok(character) => {
                    self.characters.push(character);
                    self.rebuild_flowbox(&sender);
                }
                Err(_msg) => {
                    // Error toast will be handled by parent
                }
            },
            CharactersPanelMsg::ExportCharacter(id) => {
                let _ = sender.output(CharactersPanelOutput::RequestExport(id));
            }
            CharactersPanelMsg::ExportComplete(_result) => {
                // Toast handled by parent
            }
            CharactersPanelMsg::DeleteCharacter(id) => {
                let name = self.characters.iter()
                    .find(|c| c.id == id)
                    .map(|c| c.name.as_str())
                    .unwrap_or("this character");

                let dialog = adw::AlertDialog::new(
                    Some("Delete Character?"),
                    Some(&format!(
                        "This will permanently delete {} and its avatar. This cannot be undone.",
                        name
                    )),
                );
                dialog.add_response("cancel", "Cancel");
                dialog.add_response("delete", "Delete");
                dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
                dialog.set_close_response("cancel");

                let sender_clone = sender.clone();
                let id_clone = id.clone();
                dialog.connect_response(None, move |_, response| {
                    if response == "delete" {
                        let _ = sender_clone.input_sender().send(CharactersPanelMsg::ConfirmDelete(id_clone.clone()));
                    }
                });

                dialog.present(None::<&gtk::Window>);
            }
            CharactersPanelMsg::ConfirmDelete(id) => {
                let _ = sender.output(CharactersPanelOutput::DeleteCharacter(id));
            }
            CharactersPanelMsg::EditCharacter(id) => {
                let _ = sender.output(CharactersPanelOutput::EditCharacter(id));
            }
            CharactersPanelMsg::StartConversation(id) => {
                let _ = sender.output(CharactersPanelOutput::StartConversation(id));
            }
            CharactersPanelMsg::SwitchTab(tab) => {
                self.active_tab = tab;
            }
        }
    }

    fn update_cmd(
        &mut self,
        _msg: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        // Reserved for future background operations
    }
}

impl CharactersPanelModel {
    /// Build searchable text from all character text fields.
    fn searchable_text(c: &CharacterCardV2) -> String {
        let mut parts = vec![
            c.name.clone(),
            c.description.clone(),
            c.creator.clone(),
            c.personality.clone(),
            c.scenario.clone(),
            c.first_mes.clone(),
            c.mes_example.clone(),
            c.creator_notes.clone(),
        ];
        parts.extend(c.tags.iter().cloned());
        parts.extend(c.alternate_greetings.iter().cloned());
        parts.join(" ")
    }

    /// Update empty state visibility based on whether any children are visible.
    fn update_empty_state(&self) {
        let mut has_visible = false;
        let mut child = self.flowbox.first_child();
        while let Some(c) = child {
            if c.is_visible() {
                has_visible = true;
                break;
            }
            child = c.next_sibling();
        }
        self.flowbox.set_visible(has_visible);
        self.empty_state.set_visible(!has_visible);
    }

    /// Rebuild the GtkFlowBox from the current characters list.
    fn rebuild_flowbox(&self, sender: &ComponentSender<Self>) {
        // Remove existing children
        while let Some(child) = self.flowbox.first_child() {
            self.flowbox.remove(&child);
        }

        // Rebuild searchable map
        let mut map = self.searchable_map.borrow_mut();
        map.clear();
        for character in &self.characters {
            map.insert(character.id.clone(), Self::searchable_text(character));
        }
        drop(map);

        let is_empty = self.characters.is_empty();
        self.flowbox.set_visible(!is_empty);
        self.empty_state.set_visible(is_empty);

        // Add cards for each character
        for character in &self.characters {
            let card_widget =
                create_card_widget(character, character.avatar_path.as_deref());

            // Delete button in the top-right of the card header
            let export_btn = gtk::Button::new();
            export_btn.add_css_class("flat");
            export_btn.add_css_class("circular");
            export_btn.set_tooltip_text(Some("Export Character"));
            export_btn.set_icon_name("document-save-symbolic");
            export_btn.set_halign(gtk::Align::End);
            export_btn.set_valign(gtk::Align::Start);
            export_btn.set_margin_top(4);
            export_btn.set_margin_end(4);

            let id_clone = character.id.clone();
            let sender_clone = sender.clone();
            let gesture = gtk::GestureClick::new();
            gesture.set_propagation_phase(gtk::PropagationPhase::Capture);
            gesture.connect_released(move |_, _, _, _| {
                let _ = sender_clone.input_sender().send(CharactersPanelMsg::ExportCharacter(id_clone.clone()));
            });
            export_btn.add_controller(gesture);

            // Edit button
            let edit_btn = gtk::Button::new();
            edit_btn.add_css_class("flat");
            edit_btn.add_css_class("circular");
            edit_btn.set_tooltip_text(Some("Edit Character"));
            edit_btn.set_icon_name("document-edit-symbolic");
            edit_btn.set_halign(gtk::Align::End);
            edit_btn.set_valign(gtk::Align::Start);
            edit_btn.set_margin_top(68);
            edit_btn.set_margin_end(4);

            let id_clone = character.id.clone();
            let sender_clone = sender.clone();
            let gesture = gtk::GestureClick::new();
            gesture.set_propagation_phase(gtk::PropagationPhase::Capture);
            gesture.connect_released(move |_, _, _, _| {
                let _ = sender_clone.input_sender().send(CharactersPanelMsg::EditCharacter(id_clone.clone()));
            });
            edit_btn.add_controller(gesture);

            let delete_btn = gtk::Button::new();
            delete_btn.add_css_class("flat");
            delete_btn.add_css_class("circular");
            delete_btn.add_css_class("destructive-action");
            delete_btn.set_tooltip_text(Some("Delete Character"));
            delete_btn.set_icon_name("user-trash-symbolic");
            delete_btn.set_halign(gtk::Align::End);
            delete_btn.set_valign(gtk::Align::Start);
            delete_btn.set_margin_top(100);
            delete_btn.set_margin_end(4);

            let id_clone = character.id.clone();
            let sender_clone = sender.clone();
            let gesture = gtk::GestureClick::new();
            gesture.set_propagation_phase(gtk::PropagationPhase::Capture);
            gesture.connect_released(move |_, _, _, _| {
                let _ = sender_clone.input_sender().send(CharactersPanelMsg::DeleteCharacter(id_clone.clone()));
            });
            delete_btn.add_controller(gesture);

            // Wrap card + buttons in an overlay so they sit inside the card
            let overlay = gtk::Overlay::new();
            overlay.set_child(Some(&card_widget));
            overlay.add_overlay(&export_btn);
            overlay.add_overlay(&edit_btn);
            overlay.add_overlay(&delete_btn);

            let child = gtk::FlowBoxChild::new();
            child.set_widget_name(&character.id);
            child.set_child(Some(&overlay));

            self.flowbox.append(&child);
        }

        // Apply current search filter to new children
        self.flowbox.invalidate_filter();
    }
}
