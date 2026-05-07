use adw::prelude::*;
use relm4::{adw, gtk};
use std::rc::Rc;

use crate::domain::conversation::{Message, MessageRole};
use crate::service::markdown::markdown_to_pango;

/// Actions a user can trigger from a message row.
#[derive(Debug, Clone)]
pub enum MessageRowAction {
    Edit(String),
    Regenerate(String),
    Delete(String),
    NavigateBranches(String), // message_id that has sibling branches
}

/// A widget for displaying a single message in the chat.
#[derive(Clone)]
pub struct MessageRow {
    pub message_id: String,
    pub role: MessageRole,
    pub content: String,
    pub formatted: String,
    pub timestamp: chrono::NaiveDateTime,
    pub character_name: Option<String>,
    pub user_persona_name: Option<String>,
    pub has_siblings: bool,
    pub is_editing: bool,
    pub edit_buffer: Option<gtk::TextBuffer>,
}

/// Initialization parameters for a MessageRow.
#[derive(Clone)]
pub struct MessageRowInit {
    pub message: Message,
    pub character_name: Option<String>,
    pub user_persona_name: Option<String>,
    pub has_siblings: bool,
}

impl MessageRow {
    /// Create a new MessageRow from initialization parameters.
    pub fn new(init: &MessageRowInit) -> Self {
        let formatted = if init.message.role == MessageRole::Assistant {
            markdown_to_pango(&init.message.content)
        } else {
            gtk::glib::markup_escape_text(&init.message.content).to_string()
        };

        Self {
            message_id: init.message.id.clone(),
            role: init.message.role.clone(),
            content: init.message.content.clone(),
            formatted,
            timestamp: init.message.timestamp,
            character_name: init.character_name.clone(),
            user_persona_name: init.user_persona_name.clone(),
            has_siblings: init.has_siblings,
            is_editing: false,
            edit_buffer: None,
        }
    }

    /// Get the root widget for this message row.
    /// The `on_action` callback is invoked when a context menu item is activated.
    pub fn widget(&self, on_action: Rc<dyn Fn(MessageRowAction)>) -> gtk::Box {
        let (alignment, css_class, name) = match self.role {
            MessageRole::Assistant => (
                gtk::Align::Start,
                "assistant-message",
                self.character_name.as_deref().unwrap_or("Assistant"),
            ),
            MessageRole::User => (
                gtk::Align::End,
                "user-message",
                self.user_persona_name.as_deref().unwrap_or("You"),
            ),
            MessageRole::System => (
                gtk::Align::Start,
                "system-message",
                "System",
            ),
        };

        let root = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        root.set_halign(alignment);
        root.set_margin_start(12);
        root.set_margin_end(12);
        root.set_margin_top(4);
        root.set_margin_bottom(4);
        root.add_css_class("message-row");
        root.add_css_class(css_class);

        // Branch indicator (colored left border) — clickable to navigate branches
        if self.has_siblings {
            let indicator = gtk::Button::new();
            indicator.set_width_request(4);
            indicator.add_css_class("branch-indicator");
            indicator.set_has_frame(false);
            let msg_id_nav = self.message_id.clone();
            let on_action_nav = Rc::clone(&on_action);
            indicator.connect_clicked(move |_| {
                on_action_nav(MessageRowAction::NavigateBranches(msg_id_nav.clone()));
            });
            root.prepend(&indicator);
        }

        // For assistant messages, add avatar on the left
        if self.role == MessageRole::Assistant {
            let avatar = adw::Avatar::new(32, Some(name), false);
            root.append(&avatar);
        }

        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        content_box.set_halign(alignment);

        // Name label
        let name_label = gtk::Label::new(Some(name));
        name_label.add_css_class("dim-label");
        name_label.add_css_class("caption");
        name_label.set_halign(alignment);
        content_box.append(&name_label);

        // Message content label with Pango markup
        let content_label = gtk::Label::new(None);
        content_label.set_use_markup(true);
        content_label.set_wrap(true);
        content_label.set_selectable(true);
        content_label.set_xalign(match self.role {
            MessageRole::User => 1.0,
            _ => 0.0,
        });
        content_label.set_markup(&self.formatted);
        content_box.append(&content_label);

        // Timestamp label
        let time_str = self.timestamp.format("%H:%M").to_string();
        let time_label = gtk::Label::new(Some(&time_str));
        time_label.add_css_class("dim-label");
        time_label.add_css_class("caption");
        time_label.set_halign(alignment);
        content_box.append(&time_label);

        root.append(&content_box);

        // ---- Context menu via GtkPopoverMenu + Gio simple actions on widget root ----
        let menu_button = gtk::MenuButton::new();
        menu_button.set_icon_name("view-more");
        menu_button.add_css_class("flat");
        menu_button.add_css_class("circular");
        menu_button.set_valign(gtk::Align::Center);

        // Wire actions via action_group on the root widget
        let group = gtk::gio::SimpleActionGroup::new();
        let msg_id_edit = self.message_id.clone();
        let msg_id_delete = self.message_id.clone();
        let msg_id_regen = self.message_id.clone();

        let edit_action = gtk::gio::SimpleAction::new("edit", None);
        let on_action_edit = Rc::clone(&on_action);
        edit_action.connect_activate(move |_, _| {
            on_action_edit(MessageRowAction::Edit(msg_id_edit.clone()));
        });
        group.add_action(&edit_action);

        let delete_action = gtk::gio::SimpleAction::new("delete", None);
        let on_action_delete = Rc::clone(&on_action);
        delete_action.connect_activate(move |_, _| {
            on_action_delete(MessageRowAction::Delete(msg_id_delete.clone()));
        });
        group.add_action(&delete_action);

        let regen_action = gtk::gio::SimpleAction::new("regenerate", None);
        let on_action_regen = Rc::clone(&on_action);
        regen_action.connect_activate(move |_, _| {
            on_action_regen(MessageRowAction::Regenerate(msg_id_regen.clone()));
        });
        group.add_action(&regen_action);

        root.insert_action_group("msg", Some(&group));

        // Build menu model pointing at our action group
        let menu = gtk::gio::Menu::new();
        match self.role {
            MessageRole::User => {
                menu.append(Some("Edit"), Some("msg.edit"));
            }
            MessageRole::Assistant => {
                menu.append(Some("Regenerate"), Some("msg.regenerate"));
            }
            _ => {}
        }
        menu.append(Some("Delete"), Some("msg.delete"));
        menu_button.set_menu_model(Some(&menu));

        root.append(&menu_button);

        root
    }

    /// Update the content of this message row.
    pub fn update_content(&mut self, new_content: String) {
        self.content = new_content;
        self.formatted = match self.role {
            MessageRole::Assistant => markdown_to_pango(&self.content),
            _ => gtk::glib::markup_escape_text(&self.content).to_string(),
        };
    }

    /// Set sibling status for branch indicator.
    pub fn set_has_siblings(&mut self, has_siblings: bool) {
        self.has_siblings = has_siblings;
    }

    /// Get the message ID.
    pub fn id(&self) -> &str {
        &self.message_id
    }

    /// Get the role.
    pub fn role(&self) -> &MessageRole {
        &self.role
    }

    /// Get the content.
    pub fn content(&self) -> &str {
        &self.content
    }
}
