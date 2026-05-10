use adw::prelude::*;
use relm4::prelude::*;
use relm4::{adw, gtk};

/// Input bar for composing and sending messages.
/// Contains a multi-line text view, send button, and context utilization progress bar.
pub struct InputBar {
    sending: bool,
    has_text: bool,
    text_buffer: gtk::TextBuffer,
    /// Context utilization ratio (0.0 to 1.0). 0.0 = no active conversation.
    context_utilization: f64,
    /// Number of truncated messages for tooltip.
    truncated_messages: usize,
    /// Total tokens used for tooltip.
    tokens_used: usize,
    /// Total context window for tooltip.
    context_window: usize,
    /// System prompt tokens for tooltip breakdown.
    system_tokens: usize,
    /// Chat history tokens for tooltip breakdown.
    history_tokens: usize,
    /// Lorebook tokens for tooltip breakdown.
    lorebook_tokens: usize,
    /// Widget reference for imperative CSS class updates.
    context_progress_bar: gtk::ProgressBar,
    /// Warning/error icon shown at high utilization.
    context_warning_icon: gtk::Image,
}

/// Messages handled by InputBar.
#[derive(Debug)]
pub enum InputBarMsg {
    Send,
    SetText(String),
    SetSending(bool),
    TextChanged,
    /// Update context utilization display (pct: 0.0 to 1.0).
    UpdateContextUtilization {
        pct: f64,
        truncated: usize,
        tokens_used: usize,
        context_window: usize,
        system_tokens: usize,
        history_tokens: usize,
        lorebook_tokens: usize,
    },
}

/// Output messages from InputBar to parent.
#[derive(Debug)]
pub enum InputBarOutput {
    SendMessage(String),
}

#[relm4::component(pub)]
impl Component for InputBar {
    type Init = ();
    type Input = InputBarMsg;
    type Output = InputBarOutput;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 4,

                #[name(context_progress_bar)]
                gtk::ProgressBar {
                    set_hexpand: true,
                    #[watch]
                    set_fraction: model.context_utilization,
                    #[watch]
                    set_visible: model.context_utilization > 0.0,
                },

                #[name(context_warning_icon)]
                gtk::Image {
                    #[watch]
                    set_visible: model.context_utilization > 0.8,
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 8,
                set_margin_start: 12,
                set_margin_end: 12,
                set_margin_bottom: 12,

                #[name(scrolled_window)]
                gtk::ScrolledWindow {
                    set_hexpand: true,
                    set_height_request: 60,
                    set_min_content_height: 60,
                    set_max_content_height: 200,

                    #[name(text_view)]
                    gtk::TextView {
                        set_wrap_mode: gtk::WrapMode::WordChar,
                        #[watch]
                        set_editable: !model.sending,
                        set_buffer: Some(&model.text_buffer),
                        set_top_margin: 8,
                        set_bottom_margin: 8,
                        set_left_margin: 8,
                        set_right_margin: 8,
                    },
                },

                gtk::Button {
                    set_icon_name: "send",
                    set_tooltip_text: Some("Send (Ctrl+Enter)"),
                    #[watch]
                    set_sensitive: !model.sending && model.has_text,

                    connect_clicked => InputBarMsg::Send,
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let text_buffer = gtk::TextBuffer::new(None);

        // Connect text changed signal
        let sender_clone = sender.input_sender().clone();
        text_buffer.connect_changed(move |_| {
            let _ = sender_clone.send(InputBarMsg::TextChanged);
        });

        let mut model = InputBar {
            sending: false,
            has_text: false,
            text_buffer,
            context_utilization: 0.0,
            truncated_messages: 0,
            tokens_used: 0,
            context_window: 0,
            system_tokens: 0,
            history_tokens: 0,
            lorebook_tokens: 0,
            context_progress_bar: gtk::ProgressBar::new(), // placeholder, replaced below
            context_warning_icon: gtk::Image::new(), // placeholder, replaced below
        };

        let widgets = view_output!();

        // Update the progress bar reference from the actual widget
        model.context_progress_bar = widgets.context_progress_bar.clone();
        model.context_warning_icon = widgets.context_warning_icon.clone();

        // Set up CSS for the context utilization progress bar per UI-SPEC
        let css_provider = gtk::CssProvider::new();
        css_provider.load_from_data(
            "progressbar.context-utilization-bar { min-height: 4px; } \
             progressbar.context-utilization-bar trough { min-height: 4px; } \
             progressbar.context-utilization-bar.success progress { background-color: var(--success-bg-color, #33d17a); } \
             progressbar.context-utilization-bar.warning progress { background-color: var(--warning-bg-color, #e66100); } \
             progressbar.context-utilization-bar.error progress { background-color: var(--destructive-bg-color, #e01b24); }"
        );
        gtk::StyleContext::add_provider_for_display(
            &widgets.context_progress_bar.display(),
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_USER,
        );

        // Set widget name for CSS targeting per UI-SPEC
        widgets.context_progress_bar.set_widget_name("context-utilization-bar");
        // Initial CSS class (success = green)
        widgets.context_progress_bar.add_css_class("success");

        // Set up Ctrl+Enter (or Cmd+Enter on macOS) shortcut
        let sender_clone = sender.input_sender().clone();
        widgets.text_view.add_controller({
            let controller = gtk::EventControllerKey::new();
            controller.connect_key_pressed(move |_, key, _code, modifier| {
                // Check for Ctrl+Enter or Cmd+Enter
                let modifier_ctrl = if cfg!(target_os = "macos") {
                    modifier.contains(gtk::gdk::ModifierType::META_MASK)
                } else {
                    modifier.contains(gtk::gdk::ModifierType::CONTROL_MASK)
                };

                if modifier_ctrl && (key == gtk::gdk::Key::Return || key == gtk::gdk::Key::KP_Enter) {
                    let _ = sender_clone.send(InputBarMsg::Send);
                    return gtk::glib::Propagation::Stop;
                }

                gtk::glib::Propagation::Proceed
            });
            controller
        });

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            InputBarMsg::Send => {
                // Get text from buffer
                let text = self
                    .text_buffer
                    .text(&self.text_buffer.start_iter(), &self.text_buffer.end_iter(), true);
                let text_str = text.as_str();

                let trimmed = text_str.trim();
                if !trimmed.is_empty() && !self.sending {
                    // Emit output message
                    let _ = sender.output(InputBarOutput::SendMessage(trimmed.to_string()));

                    // Clear buffer
                    self.text_buffer.set_text("");
                    self.has_text = false;
                }
            }
            InputBarMsg::SetText(text) => {
                self.text_buffer.set_text(&text);
                self.has_text = !text.trim().is_empty();
            }
            InputBarMsg::SetSending(sending) => {
                self.sending = sending;
            }
            InputBarMsg::TextChanged => {
                // Update has_text when text changes
                let text = self
                    .text_buffer
                    .text(&self.text_buffer.start_iter(), &self.text_buffer.end_iter(), true);
                self.has_text = !text.as_str().trim().is_empty();
            }
            InputBarMsg::UpdateContextUtilization { pct, truncated, tokens_used, context_window, system_tokens, history_tokens, lorebook_tokens } => {
                self.context_utilization = pct;
                self.truncated_messages = truncated;
                self.tokens_used = tokens_used;
                self.context_window = context_window;
                self.system_tokens = system_tokens;
                self.history_tokens = history_tokens;
                self.lorebook_tokens = lorebook_tokens;

                // Update CSS class based on percentage per D-15 and UI-SPEC
                // Remove all tier CSS classes first
                self.context_progress_bar.remove_css_class("success");
                self.context_progress_bar.remove_css_class("warning");
                self.context_progress_bar.remove_css_class("error");

                if pct < 0.7 {
                    self.context_progress_bar.add_css_class("success");
                } else if pct < 0.9 {
                    self.context_progress_bar.add_css_class("warning");
                } else {
                    self.context_progress_bar.add_css_class("error");
                }

                // Update warning/error icon
                if pct > 0.95 {
                    self.context_warning_icon.set_icon_name(Some("dialog-error"));
                    self.context_warning_icon.set_visible(true);
                } else if pct > 0.8 {
                    self.context_warning_icon.set_icon_name(Some("dialog-warning"));
                    self.context_warning_icon.set_visible(true);
                } else {
                    self.context_warning_icon.set_visible(false);
                }

                // Update tooltip with breakdown per UI-SPEC
                let pct_display = (pct * 100.0) as usize;
                let mut tooltip = format!(
                    "Using {} / {} tokens ({}%)\nSystem: {} tokens\nHistory: {} tokens\nLorebook: {} tokens",
                    tokens_used, context_window, pct_display, system_tokens, history_tokens, lorebook_tokens
                );
                if truncated > 0 {
                    tooltip.push_str("\nHistory truncated to fit context");
                }
                self.context_progress_bar.set_tooltip_text(Some(&tooltip));
            }
        }
    }

    fn update_cmd(
        &mut self,
        _msg: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
    }
}
