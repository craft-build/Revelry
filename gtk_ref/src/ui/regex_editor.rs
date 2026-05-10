use adw::prelude::*;
use relm4::prelude::*;
use relm4::{adw, gtk, ComponentParts, ComponentSender};

use crate::domain::regex_script::RegexScript;
use crate::service::regex::{validate_regex_pattern, validate_regex_flags, apply_single_regex};

/// Regex script editor sub-dialog — per UI-SPEC §8.
pub struct RegexScriptEditorModel {
    script: RegexScript,
    is_new: bool,
    pattern_valid: bool,
    flags_valid: bool,
    // Widget references for imperative updates
    window: adw::PreferencesWindow,
    name_row: adw::EntryRow,
    find_row: adw::EntryRow,
    redos_warning: gtk::Label,
    replace_row: adw::EntryRow,
    flags_row: adw::EntryRow,
    flags_error_label: gtk::Label,
    enabled_switch: adw::SwitchRow,
    save_button: gtk::Button,
    test_input_buffer: gtk::TextBuffer,
    before_buffer: gtk::TextBuffer,
    after_buffer: gtk::TextBuffer,
    after_frame: gtk::Frame,
}

#[derive(Debug)]
pub enum RegexScriptEditorMsg {
    UpdateName(String),
    UpdateFindPattern(String),
    UpdateReplacePattern(String),
    UpdateFlags(String),
    UpdateSortOrder(i32),
    UpdateEnabled(bool),
    UpdateTestInput(String),
    Save,
    Cancel,
    ConfirmDiscard,
}

#[derive(Debug)]
pub enum RegexScriptEditorOutput {
    SaveScript(RegexScript),
    Cancel,
}

#[relm4::component(pub)]
impl Component for RegexScriptEditorModel {
    type Init = (RegexScript, bool);
    type Input = RegexScriptEditorMsg;
    type Output = RegexScriptEditorOutput;
    type CommandOutput = ();

    view! {
        #[root]
        adw::PreferencesWindow {
            set_title: Some("Edit Script"),
            set_modal: true,
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let (script, is_new) = init;
        if is_new {
            root.set_title(Some("New Script"));
        }
        root.set_default_width(500);
        root.set_default_height(600);

        let widgets = view_output!();

        // Build page imperatively (matches LorebookEditor pattern)
        let page = adw::PreferencesPage::new();

        // === Script Details Group ===
        let details_group = adw::PreferencesGroup::new();
        details_group.set_title("Script Details");

        let name_row = adw::EntryRow::new();
        name_row.set_title("Name");
        name_row.set_text(&script.name);
        let sc = sender.clone();
        name_row.connect_changed(move |r| {
            let _ = sc.input_sender().send(RegexScriptEditorMsg::UpdateName(r.text().to_string()));
        });
        details_group.add(&name_row);

        let find_row = adw::EntryRow::new();
        find_row.set_title("Find Pattern");
        find_row.set_text(&script.find_pattern);
        let sc = sender.clone();
        find_row.connect_changed(move |r| {
            let _ = sc.input_sender().send(RegexScriptEditorMsg::UpdateFindPattern(r.text().to_string()));
        });
        details_group.add(&find_row);

        let redos_warning = gtk::Label::new(Some(
            "This pattern may cause catastrophic backtracking (ReDoS). Please simplify the regex or add more specific constraints.",
        ));
        redos_warning.add_css_class("error");
        redos_warning.set_wrap(true);
        redos_warning.set_xalign(0.0);
        redos_warning.set_margin_start(12);
        redos_warning.set_margin_end(12);
        redos_warning.set_margin_top(4);
        redos_warning.set_margin_bottom(4);
        redos_warning.set_visible(false);
        let redos_row = adw::PreferencesRow::new();
        redos_row.set_child(Some(&redos_warning));
        details_group.add(&redos_row);

        let replace_row = adw::EntryRow::new();
        replace_row.set_title("Replace Pattern");
        replace_row.set_text(&script.replace_pattern);
        let sc = sender.clone();
        replace_row.connect_changed(move |r| {
            let _ = sc.input_sender().send(RegexScriptEditorMsg::UpdateReplacePattern(r.text().to_string()));
        });
        details_group.add(&replace_row);

        let flags_row = adw::EntryRow::new();
        flags_row.set_title("Flags");
        flags_row.set_text(&script.flags);
        // Placeholder hint via subtitle instead of placeholder_text (not available on EntryRow)
        let sc = sender.clone();
        flags_row.connect_changed(move |r| {
            let _ = sc.input_sender().send(RegexScriptEditorMsg::UpdateFlags(r.text().to_string()));
        });
        details_group.add(&flags_row);

        let flags_error_label = gtk::Label::new(Some("Flags can only contain: i, g, m, s"));
        flags_error_label.add_css_class("error");
        flags_error_label.set_wrap(true);
        flags_error_label.set_xalign(0.0);
        flags_error_label.set_margin_start(12);
        flags_error_label.set_margin_end(12);
        flags_error_label.set_margin_top(4);
        flags_error_label.set_margin_bottom(4);
        flags_error_label.set_visible(false);
        let ferr_row = adw::PreferencesRow::new();
        ferr_row.set_child(Some(&flags_error_label));
        details_group.add(&ferr_row);

        let order_adjustment = gtk::Adjustment::new(script.sort_order as f64, 0.0, 99.0, 1.0, 5.0, 0.0);
        let order_spin = adw::SpinRow::new(
            Some(&order_adjustment),
            1.0, 0,
        );
        order_spin.set_title("Execution Order");
        order_spin.set_subtitle("Scripts run in ascending order. Lower numbers run first.");
        let sc = sender.clone();
        order_adjustment.connect_value_changed(move |adj| {
            let _ = sc.input_sender().send(RegexScriptEditorMsg::UpdateSortOrder(adj.value() as i32));
        });
        details_group.add(&order_spin);

        let enabled_switch = adw::SwitchRow::new();
        enabled_switch.set_title("Enabled");
        enabled_switch.set_subtitle("When disabled, this script is skipped during post-processing");
        enabled_switch.set_active(script.enabled);
        let sc = sender.clone();
        enabled_switch.connect_active_notify(move |r| {
            let _ = sc.input_sender().send(RegexScriptEditorMsg::UpdateEnabled(r.is_active()));
        });
        details_group.add(&enabled_switch);

        page.add(&details_group);

        // === Test Panel Group ===
        let test_group = adw::PreferencesGroup::new();
        test_group.set_title("Test Panel");

        let th = gtk::Label::new(Some("Test Script"));
        th.add_css_class("caption-heading");
        th.set_xalign(0.0);
        th.set_margin_bottom(2);
        let thr = adw::PreferencesRow::new();
        thr.set_child(Some(&th));
        test_group.add(&thr);

        let td = gtk::Label::new(Some("Enter sample text below to test this regex script. Output updates as you type."));
        td.add_css_class("caption");
        td.add_css_class("dim-label");
        td.set_wrap(true);
        td.set_xalign(0.0);
        td.set_margin_bottom(8);
        let tdr = adw::PreferencesRow::new();
        tdr.set_child(Some(&td));
        test_group.add(&tdr);

        let il = gtk::Label::new(Some("Input"));
        il.add_css_class("heading");
        il.set_xalign(0.0);
        il.set_margin_bottom(4);
        let ilr = adw::PreferencesRow::new();
        ilr.set_child(Some(&il));
        test_group.add(&ilr);

        let test_input_buffer = gtk::TextBuffer::new(None);
        let test_input_view = gtk::TextView::new();
        test_input_view.set_buffer(Some(&test_input_buffer));
        test_input_view.set_wrap_mode(gtk::WrapMode::WordChar);
        let test_scroll = gtk::ScrolledWindow::new();
        test_scroll.set_min_content_height(80);
        test_scroll.set_child(Some(&test_input_view));
        let tsr = adw::PreferencesRow::new();
        tsr.set_child(Some(&test_scroll));
        test_group.add(&tsr);

        // Side-by-side Before/After
        let side_box = gtk::Box::new(gtk::Orientation::Horizontal, 16);
        side_box.set_margin_top(8);
        side_box.set_homogeneous(true);

        // Before
        let before_col = gtk::Box::new(gtk::Orientation::Vertical, 0);
        before_col.set_hexpand(true);
        let bl = gtk::Label::new(Some("Before"));
        bl.add_css_class("caption-heading");
        bl.set_xalign(0.0);
        bl.set_margin_bottom(4);
        before_col.append(&bl);
        let before_buffer = gtk::TextBuffer::new(None);
        let before_view = gtk::TextView::new();
        before_view.set_buffer(Some(&before_buffer));
        before_view.set_editable(false);
        before_view.set_wrap_mode(gtk::WrapMode::WordChar);
        let before_frame = gtk::Frame::new(None);
        // GTK4 removed shadow types — use CSS for inset appearance
        before_frame.add_css_class("frame");
        let bs = gtk::ScrolledWindow::new();
        bs.set_min_content_height(80);
        bs.set_child(Some(&before_view));
        before_frame.set_child(Some(&bs));
        before_col.append(&before_frame);
        side_box.append(&before_col);

        // After
        let after_col = gtk::Box::new(gtk::Orientation::Vertical, 0);
        after_col.set_hexpand(true);
        let al = gtk::Label::new(Some("After"));
        al.add_css_class("caption-heading");
        al.set_xalign(0.0);
        al.set_margin_bottom(4);
        after_col.append(&al);
        let after_buffer = gtk::TextBuffer::new(None);
        let after_view = gtk::TextView::new();
        after_view.set_buffer(Some(&after_buffer));
        after_view.set_editable(false);
        after_view.set_wrap_mode(gtk::WrapMode::WordChar);
        let after_frame = gtk::Frame::new(None);
        // GTK4 removed shadow types — use CSS for inset appearance
        after_frame.add_css_class("frame");
        let as_ = gtk::ScrolledWindow::new();
        as_.set_min_content_height(80);
        as_.set_child(Some(&after_view));
        after_frame.set_child(Some(&as_));
        after_col.append(&after_frame);
        side_box.append(&after_col);

        let sbr = adw::PreferencesRow::new();
        sbr.set_child(Some(&side_box));
        test_group.add(&sbr);

        let tip = gtk::Label::new(Some("Changes are highlighted when the output differs from the input."));
        tip.add_css_class("caption");
        tip.add_css_class("dim-label");
        tip.set_wrap(true);
        tip.set_xalign(0.0);
        tip.set_margin_top(4);
        let tipr = adw::PreferencesRow::new();
        tipr.set_child(Some(&tip));
        test_group.add(&tipr);

        page.add(&test_group);

        // === Actions Group ===
        let actions_group = adw::PreferencesGroup::new();
        let actions_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        actions_box.set_halign(gtk::Align::End);

        let cancel_btn = gtk::Button::with_label("Cancel");
        cancel_btn.add_css_class("flat");
        let sc = sender.clone();
        cancel_btn.connect_clicked(move |_| {
            let _ = sc.input_sender().send(RegexScriptEditorMsg::Cancel);
        });
        actions_box.append(&cancel_btn);

        let save_button = gtk::Button::with_label("Save Script");
        save_button.add_css_class("suggested-action");
        let sc = sender.clone();
        save_button.connect_clicked(move |_| {
            let _ = sc.input_sender().send(RegexScriptEditorMsg::Save);
        });
        actions_box.append(&save_button);

        actions_group.add(&actions_box);
        page.add(&actions_group);

        root.add(&page);

        // Connect test input buffer signal
        let sc = sender.clone();
        test_input_buffer.connect_changed(move |buf| {
            let (s, e) = buf.bounds();
            let text = buf.text(&s, &e, false).to_string();
            let _ = sc.input_sender().send(RegexScriptEditorMsg::UpdateTestInput(text));
        });

        let mut model = RegexScriptEditorModel {
            script: script.clone(),
            is_new,
            pattern_valid: false,
            flags_valid: true,
            window: root,
            name_row,
            find_row,
            redos_warning,
            replace_row,
            flags_row,
            flags_error_label,
            enabled_switch,
            save_button,
            test_input_buffer: test_input_buffer.clone(),
            before_buffer: before_buffer.clone(),
            after_buffer: after_buffer.clone(),
            after_frame,
        };

        // Perform initial validation
        let pv = Self::check_pattern_valid(&model.script.find_pattern, &model.script.flags);
        model.apply_pattern_validation(pv);
        let fv = Self::check_flags_valid(&model.script.flags);
        model.apply_flags_validation(fv);
        model.refresh_test_panel();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            RegexScriptEditorMsg::UpdateName(v) => {
                self.script.name = v;
            }
            RegexScriptEditorMsg::UpdateFindPattern(v) => {
                self.script.find_pattern = v;
                let pv = Self::check_pattern_valid(&self.script.find_pattern, &self.script.flags);
                self.apply_pattern_validation(pv);
                self.refresh_test_panel();
            }
            RegexScriptEditorMsg::UpdateReplacePattern(v) => {
                self.script.replace_pattern = v;
                self.refresh_test_panel();
            }
            RegexScriptEditorMsg::UpdateFlags(v) => {
                self.script.flags = v;
                let fv = Self::check_flags_valid(&self.script.flags);
                self.apply_flags_validation(fv);
                let pv = Self::check_pattern_valid(&self.script.find_pattern, &self.script.flags);
                self.apply_pattern_validation(pv);
                self.refresh_test_panel();
            }
            RegexScriptEditorMsg::UpdateSortOrder(v) => {
                self.script.sort_order = v;
            }
            RegexScriptEditorMsg::UpdateEnabled(v) => {
                self.script.enabled = v;
            }
            RegexScriptEditorMsg::UpdateTestInput(_) => {
                self.refresh_test_panel();
            }
            RegexScriptEditorMsg::Save => {
                if self.script.name.trim().is_empty()
                    || self.script.find_pattern.trim().is_empty()
                    || !self.pattern_valid
                    || !self.flags_valid
                {
                    if self.script.name.trim().is_empty() {
                        self.name_row.add_css_class("error");
                    }
                    if self.script.find_pattern.trim().is_empty() {
                        self.find_row.add_css_class("error");
                    }
                    return;
                }
                self.script.updated_at = chrono::Local::now().naive_local();
                let _ = sender.output(RegexScriptEditorOutput::SaveScript(self.script.clone()));
                self.window.close();
            }
            RegexScriptEditorMsg::Cancel => {
                let dialog = adw::AlertDialog::new(
                    Some("Discard changes?"),
                    Some("Your changes will be lost."),
                );
                dialog.add_response("keep", "Keep Editing");
                dialog.add_response("discard", "Discard");
                dialog.set_close_response("keep");
                let sc = sender.clone();
                dialog.connect_response(None, move |_, resp| {
                    if resp == "discard" {
                        let _ = sc.input_sender().send(RegexScriptEditorMsg::ConfirmDiscard);
                    }
                });
                dialog.present(Some(&self.window));
            }
            RegexScriptEditorMsg::ConfirmDiscard => {
                let _ = sender.output(RegexScriptEditorOutput::Cancel);
                self.window.close();
            }
        }
    }
}

impl RegexScriptEditorModel {
    /// Check if the pattern is valid. Returns (is_valid, is_redos).
    fn check_pattern_valid(pattern: &str, flags: &str) -> (bool, bool) {
        if pattern.is_empty() {
            return (false, false);
        }
        match validate_regex_pattern(pattern, flags) {
            Ok(_) => (true, false),
            Err(e) => {
                let is_redos = e.to_lowercase().contains("size limit")
                    || e.to_lowercase().contains("backtrack");
                (false, is_redos)
            }
        }
    }

    fn check_flags_valid(flags: &str) -> bool {
        validate_regex_flags(flags).is_ok()
    }

    fn apply_pattern_validation(&mut self, (valid, is_redos): (bool, bool)) {
        self.pattern_valid = valid;
        if valid {
            self.redos_warning.set_visible(false);
            self.find_row.remove_css_class("error");
        } else {
            self.redos_warning.set_visible(true);
            if is_redos {
                self.redos_warning.set_label(
                    "This pattern may cause catastrophic backtracking (ReDoS). Please simplify the regex or add more specific constraints.",
                );
            } else if !self.script.find_pattern.is_empty() {
                self.redos_warning.set_label(&format!(
                    "Invalid regex: {}",
                    validate_regex_pattern(&self.script.find_pattern, &self.script.flags)
                        .unwrap_err()
                ));
            } else {
                self.redos_warning.set_visible(false);
            }
            self.find_row.add_css_class("error");
            // Force enabled switch off per D-06
            self.enabled_switch.set_active(false);
            self.script.enabled = false;
        }
        self.update_save_button();
    }

    fn apply_flags_validation(&mut self, valid: bool) {
        self.flags_valid = valid;
        if valid {
            self.flags_error_label.set_visible(false);
            self.flags_row.remove_css_class("error");
        } else {
            self.flags_error_label.set_visible(true);
            self.flags_row.add_css_class("error");
        }
        self.update_save_button();
    }

    fn update_save_button(&self) {
        self.save_button
            .set_sensitive(self.pattern_valid && self.flags_valid);
    }

    fn refresh_test_panel(&self) {
        let (s, e) = self.test_input_buffer.bounds();
        let test_input = self.test_input_buffer.text(&s, &e, false).to_string();

        self.before_buffer.set_text(&test_input);

        if self.script.find_pattern.is_empty() {
            self.after_buffer.set_text(&test_input);
            self.after_frame.remove_css_class("accent");
            return;
        }

        match apply_single_regex(
            &test_input,
            &self.script.find_pattern,
            &self.script.replace_pattern,
            &self.script.flags,
        ) {
            Ok(output) => {
                self.after_buffer.set_text(&output);
                if output != test_input {
                    self.after_frame.add_css_class("accent");
                } else {
                    self.after_frame.remove_css_class("accent");
                }
            }
            Err(e) => {
                self.after_buffer.set_text(&format!("Error: {}", e));
                self.after_frame.remove_css_class("accent");
            }
        }
    }
}
