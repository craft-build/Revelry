use adw::prelude::*;
use relm4::prelude::*;
use relm4::{adw, gtk};

/// Input bar for composing and sending messages.
/// Contains a multi-line text view and send button.
pub struct InputBar {
    sending: bool,
    has_text: bool,
    text_buffer: gtk::TextBuffer,
}

/// Messages handled by InputBar.
#[derive(Debug)]
pub enum InputBarMsg {
    Send,
    SetText(String),
    SetSending(bool),
    TextChanged,
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
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 8,
            set_margin_start: 12,
            set_margin_end: 12,
            set_margin_bottom: 12,

            #[name(scrolled_window)]
            gtk::ScrolledWindow {
                set_hexpand: true,
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

        let model = InputBar {
            sending: false,
            has_text: false,
            text_buffer,
        };

        let widgets = view_output!();

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
