use adw::prelude::*;
use relm4::{adw, gtk};

use crate::domain::character::CharacterCardV2;

/// Build a card widget for display in a GtkFlowBox.
///
/// Returns a vertical gtk::Box containing:
/// - AdwAvatar (64px, circular)
/// - Name label (bold, ellipsized)
/// - Creator label (secondary style)
/// - Tags row (up to 3 tags, horizontal)
pub fn create_card_widget(
    character: &CharacterCardV2,
    thumbnail_path: Option<&str>,
) -> gtk::Box {
    let card = gtk::Box::new(gtk::Orientation::Vertical, 6);
    card.add_css_class("card");
    card.set_margin_start(6);
    card.set_margin_end(6);
    card.set_margin_top(6);
    card.set_margin_bottom(6);
    card.set_spacing(4);

    // Avatar: 64px, circular. Use thumbnail if available, otherwise first letter.
    let avatar = adw::Avatar::new(64, None, true);
    avatar.set_text(Some(&character.name));

    // Try to load custom image from thumbnail path
    if let Some(path) = thumbnail_path {
        if let Ok(texture) = gtk::gdk::Texture::from_filename(path) {
            avatar.set_custom_image(Some(&texture));
        }
    }

    card.append(&avatar);

    // Name label: bold, ellipsize end, max 1 line
    let name_label = gtk::Label::new(Some(&character.name));
    name_label.set_ellipsize(pango::EllipsizeMode::End);
    name_label.set_max_width_chars(20);
    name_label.set_lines(1);
    name_label.add_css_class("heading");
    name_label.set_halign(gtk::Align::Center);
    card.append(&name_label);

    // Creator label: secondary style, smaller, max 1 line
    if !character.creator.is_empty() {
        let creator_label = gtk::Label::new(Some(&format!("by {}", character.creator)));
        creator_label.set_ellipsize(pango::EllipsizeMode::End);
        creator_label.set_max_width_chars(20);
        creator_label.set_lines(1);
        creator_label.add_css_class("caption");
        creator_label.add_css_class("dim-label");
        creator_label.set_halign(gtk::Align::Center);
        card.append(&creator_label);
    }

    // Tags row: up to 3 tags as labels, truncate with "..." if more
    if !character.tags.is_empty() {
        let tags_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        tags_box.set_halign(gtk::Align::Center);

        let display_tags: Vec<&str> = character
            .tags
            .iter()
            .take(3)
            .map(|s| s.as_str())
            .collect();

        for tag in display_tags {
            let tag_label = gtk::Label::new(Some(tag));
            tag_label.add_css_class("tag");
            tag_label.add_css_class("caption");
            tag_label.set_valign(gtk::Align::Center);
            tags_box.append(&tag_label);
        }

        if character.tags.len() > 3 {
            let more = gtk::Label::new(Some("..."));
            more.add_css_class("caption");
            more.add_css_class("dim-label");
            tags_box.append(&more);
        }

        card.append(&tags_box);
    }

    card
}
