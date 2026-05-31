use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use adw::{ActionRow, ExpanderRow, PreferencesGroup};
use glib::clone;
use gtk::{CheckButton, Image};

use crate::catalog::{describe, SchedulerInfo};
use crate::recommend::Recommendation;

/// The full scheduler list, tucked inside a collapsed "Choose a different
/// scheduler" expander so newcomers aren't faced with ~18 options up front.
/// Exposes the currently selected name via `selected()` and fires `on_change`
/// whenever the user picks a new scheduler.
pub struct SchedulerPicker {
    group: PreferencesGroup,
    expander: ExpanderRow,
    state: Rc<RefCell<State>>,
}

type ChangeListener = Box<dyn Fn(&str)>;

struct State {
    selected: Option<String>,
    listeners: Vec<ChangeListener>,
}

impl SchedulerPicker {
    pub fn new(
        schedulers: &[String],
        recommended: &Recommendation,
        initial: Option<&str>,
    ) -> Self {
        let group = PreferencesGroup::builder().build();
        let expander = ExpanderRow::builder()
            .title("Choose a different scheduler")
            .subtitle("Browse every scheduler scx_loader offers")
            .build();
        group.add(&expander);

        let state = Rc::new(RefCell::new(State {
            selected: initial.map(str::to_owned),
            listeners: vec![],
        }));

        // Off-screen anchor so every radio is in the same group (including
        // the very first one — without that, the first CheckButton renders
        // as a checkmark instead of a radio dot).
        let anchor = CheckButton::new();

        for name in schedulers {
            let info = describe(name);
            let is_recommended = name == recommended.scheduler;
            let radio = CheckButton::new();
            radio.set_group(Some(&anchor));
            if initial.map(|n| n == name.as_str()).unwrap_or(false) {
                radio.set_active(true);
            }

            let row = build_row(&info, is_recommended, &radio, recommended);
            expander.add_row(&row);

            let name_owned = name.clone();
            radio.connect_toggled(clone!(
                #[strong]
                state,
                #[strong]
                name_owned,
                move |r| {
                    if !r.is_active() {
                        return;
                    }
                    let mut s = state.borrow_mut();
                    if s.selected.as_deref() == Some(name_owned.as_str()) {
                        return;
                    }
                    s.selected = Some(name_owned.clone());
                    for cb in &s.listeners {
                        cb(&name_owned);
                    }
                }
            ));
        }

        Self { group, expander, state }
    }

    pub fn widget(&self) -> &PreferencesGroup {
        &self.group
    }

    /// Force the list closed. Used right after layout because a pre-selected
    /// radio can pull focus and make AdwExpanderRow auto-expand on launch.
    pub fn collapse(&self) {
        self.expander.set_expanded(false);
    }

    pub fn selected(&self) -> Option<String> {
        self.state.borrow().selected.clone()
    }

    pub fn on_change(&self, cb: impl Fn(&str) + 'static) {
        self.state.borrow_mut().listeners.push(Box::new(cb));
    }
}

fn build_row(
    info: &SchedulerInfo,
    is_recommended: bool,
    radio: &CheckButton,
    recommended: &Recommendation,
) -> ActionRow {
    let row = ActionRow::builder()
        .title(&info.display_title)
        .subtitle(&info.blurb)
        .activatable_widget(radio)
        .build();

    let prefix_icon = Image::from_icon_name(info.icon_name);
    prefix_icon.set_icon_size(gtk::IconSize::Normal);
    row.add_prefix(&prefix_icon);

    if is_recommended {
        // Subtle accent stripe marks the recommendation inside the expanded
        // list. The prominent "★ Recommended" badge lives on the hero card.
        row.add_css_class("synse-recommended-row");
        row.set_tooltip_text(Some(recommended.reason));
    }

    row.add_suffix(&badge_pill(info.badge));
    row.add_suffix(radio);

    row
}

/// Build the small colored use-case pill (Gaming, Experimental, …).
pub fn badge_pill(badge: crate::catalog::Badge) -> gtk::Label {
    let label = gtk::Label::builder().label(badge.label()).build();
    label.add_css_class("synse-badge");
    label.add_css_class(badge.css_class());
    label.set_valign(gtk::Align::Center);
    label
}
