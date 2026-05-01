use adw::prelude::*;
use relm4::prelude::*;
use relm4::{adw, gtk};

use crate::domain::conversation::{Message, MessageRole};
use crate::service::markdown::markdown_to_pango;

/// A widget for displaying a single message in the chat.
/// Character messages are left-aligned, user messages are right-aligned.
#[derive(Clone)]
pub struct MessageRow {
    pub role: MessageRole,
    pub content: String,
    pub formatted: String,
    pub timestamp: chrono::NaiveDateTime,
    pub character_name: Option<String>,
}

/// Initialization parameters for a MessageRow.
#[derive(Clone)]
pub struct MessageRowInit {
    pub message: Message,
    pub character_name: Option<String>,
}

impl MessageRow {
    /// Create a new MessageRow from initialization parameters.
    pub fn new(init: &MessageRowInit) -> Self {
        let formatted = if init.message.role == MessageRole::Assistant {
            markdown_to_pango(&init.message.content)
        } else {
            // User messages are plain text - just escape for display
            gtk::glib::markup_escape_text(&init.message.content).to_string()
        };

        Self {
            role: init.message.role.clone(),
            content: init.message.content.clone(),
            formatted,
            timestamp: init.message.timestamp,
            character_name: init.character_name.clone(),
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

        // For assistant messages, add avatar on the left
        if self.role == MessageRole::Assistant {
            let avatar = adw::Avatar::new(32, Some(name), false);
            // TODO: Load character avatar from avatar_path when available
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
}
