use adw::prelude::*;
use relm4::prelude::*;
use relm4::{adw, gtk, ComponentParts, ComponentSender};

/// Settings panel with provider configuration.
/// Phase 1: Placeholder with "AI Providers" group — full UI in Plan 03.
pub struct SettingsPanel;

/// Messages handled by SettingsPanel.
#[derive(Debug)]
pub enum SettingsPanelMsg {}

#[relm4::component(pub)]
impl SimpleComponent for SettingsPanel {
    type Init = ();
    type Input = SettingsPanelMsg;
    type Output = ();

    view! {
        #[root]
        adw::Clamp {
            set_maximum_size: 600,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 24,
                set_margin_top: 24,
                set_margin_bottom: 24,
                set_margin_start: 12,
                set_margin_end: 12,

                adw::PreferencesGroup {
                    set_title: "AI Providers",
                    set_description: Some("Configure AI providers for chat and roleplay."),

                    gtk::Label {
                        set_label: "Provider configuration will be available in a future update.",
                        add_css_class: "dim-label",
                        set_wrap: true,
                    },
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = SettingsPanel;
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, _msg: Self::Input, _sender: ComponentSender<Self>) {}
}
