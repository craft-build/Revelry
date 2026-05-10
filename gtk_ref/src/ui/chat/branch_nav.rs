use adw::prelude::*;
use relm4::prelude::*;
use relm4::{adw, gtk};

use crate::domain::conversation::MessageBranch;

/// Branch navigator component for switching between conversation branches.
/// Shows a popover with branch list and visual indicator on messages.
pub struct BranchNav {
    branches: Vec<MessageBranch>,
    active_branch_id: Option<String>,
}

/// Initialization parameters for BranchNav.
pub struct BranchNavInit {
    pub branches: Vec<MessageBranch>,
    pub active_branch_id: Option<String>,
}

/// Messages handled by BranchNav.
#[derive(Debug)]
pub enum BranchNavMsg {
    SetBranches(Vec<MessageBranch>),
    SelectBranch(Option<String>),
    ShowPopover,
}

/// Output messages from BranchNav to parent.
#[derive(Debug)]
pub enum BranchNavOutput {
    SwitchBranch(Option<String>),
}

#[relm4::component(pub)]
impl Component for BranchNav {
    type Init = BranchNavInit;
    type Input = BranchNavMsg;
    type Output = BranchNavOutput;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 4,

            // Branch indicator (colored left border showing branch count)
            #[name(branch_indicator)]
            gtk::Box {
                #[watch]
                set_visible: !model.branches.is_empty() || model.active_branch_id.is_some(),
                set_width_request: 4,
                add_css_class: "branch-indicator",
                set_margin_start: 4,
            },

            // Branch count badge and button
            gtk::Button {
                #[watch]
                set_visible: !model.branches.is_empty() || model.active_branch_id.is_some(),
                #[watch]
                set_label: &{
                    let total = model.branches.len() + if model.active_branch_id.is_some() { 1 } else { 0 };
                    format!("{}", total)
                },
                add_css_class: "flat",
                add_css_class: "circular",
                add_css_class: "branch-count",
                set_tooltip_text: Some("Switch branch"),

                connect_clicked => BranchNavMsg::ShowPopover,
            },

            // Popover for branch selection
            #[name(popover)]
            gtk::PopoverMenu::from_model(gio::Menu::new().upcast_ref()),
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = BranchNav {
            branches: init.branches,
            active_branch_id: init.active_branch_id,
        };

        let widgets = view_output!();

        // Build the popover menu dynamically
        let menu = gio::Menu::new();

        // Add "Root" entry
        let root_item = gio::MenuItem::new(Some("Root"), None);
        root_item.set_action_and_target_value("app.switch_branch", Some(&glib::Variant::from(None::<String>)));
        menu.append_item(&root_item);

        // Add separator
        menu.append_item(&gio::MenuItem::new(Some("separator"), None));

        // Add branches
        for (i, branch) in model.branches.iter().enumerate() {
            let label = if branch.label.is_empty() {
                format!("Branch {}", i + 1)
            } else {
                branch.label.clone()
            };
            let item = gio::MenuItem::new(Some(&label), None);
            item.set_action_and_target_value("app.switch_branch", Some(&glib::Variant::from(Some(branch.id.clone()))));
            menu.append_item(&item);
        }

        widgets.popover.set_menu_model(Some(&menu));

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            BranchNavMsg::SetBranches(branches) => {
                self.branches = branches;
            }
            BranchNavMsg::SelectBranch(branch_id) => {
                let _ = sender.output(BranchNavOutput::SwitchBranch(branch_id));
            }
            BranchNavMsg::ShowPopover => {
                // Popover will be shown via GTK action or parent control
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

impl BranchNav {
    /// Check if a message has sibling branches.
    pub fn has_siblings(&self) -> bool {
        !self.branches.is_empty()
    }

    /// Get the active branch label.
    pub fn active_branch_label(&self) -> String {
        match &self.active_branch_id {
            Some(id) => {
                self.branches
                    .iter()
                    .find(|b| &b.id == id)
                    .map(|b| {
                        if b.label.is_empty() {
                            "Active Branch".to_string()
                        } else {
                            b.label.clone()
                        }
                    })
                    .unwrap_or_else(|| "Active Branch".to_string())
            }
            None => "Root".to_string(),
        }
    }
}
