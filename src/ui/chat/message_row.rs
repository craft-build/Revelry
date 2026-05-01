use adw::prelude::*;
use relm4::prelude::*;
use relm4::{adw, gtk};
use relm4::gtk::gio;

use crate::domain::conversation::{Message, MessageRole};
use crate::service::markdown::markdown_to_pango;

/// Output messages from MessageRow to parent (ChatPanel).
#[derive(Debug, Clone)]
pub enum MessageRowOutput {
    EditMessage { message_id: String, new_content: String },
    RegenerateMessage(String),
    DeleteMessage(String),
}

/// A widget for displaying a single message in the chat.
/// Character messages are left-aligned, user messages are right-aligned.
#[derive(Clone)]
pub struct MessageRow {
    pub message_id: String,
    pub role: MessageRole,
    pub content: String,
    pub formatted: String,
    pub timestamp: chrono::NaiveDateTime,
    pub character_name: Option<String>,
    pub has_siblings: bool,
    pub is_editing: bool,
    pub edit_buffer: Option<gtk::TextBuffer>,
}

/// Initialization parameters for a MessageRow.
#[derive(Clone)]
pub struct MessageRowInit {
    pub message: Message,
    pub character_name: Option<String>,
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
            has_siblings: init.has_siblings,
            is_editing: false,
            edit_buffer: None,
        }
    }

    /// Get the root widget for this message row.
    pub fn widget(&self) -> gtk::Box {
        let (alignment, css_class, name) = match self.role {
            MessageRole::Assistant => (
                gtk::Align::Start,
                "assistant-message",
                self.character_name.as_deref().unwrap_or("Assistant"),
            ),
            MessageRole::User => (
                gtk::Align::End,
                "user-message",
                "You",
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

        // Branch indicator (colored left border)
        if self.has_siblings {
            let indicator = gtk::Box::new(gtk::Orientation::Vertical, 0);
            indicator.set_width_request(4);
            indicator.add_css_class("branch-indicator");
            root.append(&indicator);
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

        // Context menu button (three dots)
        let menu_button = gtk::MenuButton::new();
        menu_button.set_icon_name("view-more-symbolic");
        menu_button.add_css_class("flat");
        menu_button.add_css_class("circular");
        menu_button.set_valign(gtk::Align::Center);
        menu_button.set_halign(gtk::Align::End);
        menu_button.set_vexpand(false);
        menu_button.set_hexpand(false);
        
        // Build menu model
        let menu = gio::Menu::new();
        
        match self.role {
            MessageRole::User => {
                let edit_item = gio::MenuItem::new(Some("Edit"), Some("message.edit"));
                menu.append_item(&edit_item);
            }
            MessageRole::Assistant => {
                let regen_item = gio::MenuItem::new(Some("Regenerate"), Some("message.regenerate"));
                menu.append_item(&regen_item);
            }
            _ => {}
        }
        
        let delete_item = gio::MenuItem::new(Some("Delete"), Some("message.delete"));
        menu.append_item(&delete_item);
        
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
