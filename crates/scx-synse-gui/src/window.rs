use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use adw::{ActionRow, ApplicationWindow, Banner, PreferencesGroup, Toast, ToastOverlay};
use glib::clone;
use gtk::{glib, Box as GtkBox, Builder, Button, Image, Label, Stack};

use scx_synse_gui::catalog::{describe, Badge};
use scx_synse_gui::helper_client::{HelperClient, HelperError};
use scx_synse_gui::loader_query;
use scx_synse_gui::profile_picker::ProfilePicker;
use scx_synse_gui::recommend::{recommend, Recommendation, SystemProbe};
use scx_synse_gui::scheduler_picker::{badge_pill, SchedulerPicker};
use scx_synse_gui::system_info::{is_supported, SysfsRoot};
use scx_synse_ipc::{Request, Response, SchedMode};

const WINDOW_RESOURCE: &str = "/com/synsenetwork/scx-synse-manager/ui/window.ui";

pub struct SynseWindow;

impl SynseWindow {
    pub fn build(app: &adw::Application) -> ApplicationWindow {
        let builder = Builder::from_resource(WINDOW_RESOURCE);

        let window: ApplicationWindow = object(&builder, "main_window");
        window.set_application(Some(app));

        let banner: Banner = object(&builder, "status_banner");
        let content_stack: Stack = object(&builder, "content_stack");
        let toast_overlay: ToastOverlay = object(&builder, "toast_overlay");
        let scroll_box: GtkBox = object(&builder, "scroll_box");

        if is_supported(&SysfsRoot::default()) {
            content_stack.set_visible_child_name("main");
            banner.set_revealed(true);
            install_status_timer(&banner);
            populate_scheduler_section(&scroll_box, &toast_overlay);
        } else {
            content_stack.set_visible_child_name("unsupported");
        }

        window
    }
}

/// The "Recommended for your system" card at the top of the window. It is the
/// golden path: a newcomer reads one line and clicks Apply. It doubles as a
/// live summary of whatever scheduler is currently selected, so picking a
/// different one from the expander updates it in place.
struct Hero {
    group: PreferencesGroup,
    row: ActionRow,
    icon: Image,
    kind_badge: Label,
    badge: Label,
    apply: Button,
}

impl Hero {
    fn new() -> Self {
        let group = PreferencesGroup::builder().build();
        let row = ActionRow::builder().build();
        row.add_css_class("synse-hero");
        row.set_subtitle_lines(0);

        let icon = Image::new();
        icon.set_icon_size(gtk::IconSize::Large);
        row.add_prefix(&icon);

        // Use-case pill (Gaming / Experimental / …), updated in `show`.
        let kind_badge = badge_pill(Badge::Other);
        row.add_suffix(&kind_badge);

        let badge = Label::builder().label("★ Recommended").build();
        badge.add_css_class("synse-recommended-badge");
        badge.set_valign(gtk::Align::Center);
        badge.set_visible(false);
        row.add_suffix(&badge);

        let apply = Button::builder().label("Apply").build();
        apply.add_css_class("suggested-action");
        apply.set_valign(gtk::Align::Center);
        row.add_suffix(&apply);

        group.add(&row);
        Self { group, row, icon, kind_badge, badge, apply }
    }

    /// Point the card at `scheduler`, showing the recommendation rationale and
    /// badge when it happens to be the recommended pick.
    fn show(&self, scheduler: &str, rec: &Recommendation) {
        let info = describe(scheduler);
        self.icon.set_icon_name(Some(info.icon_name));
        self.row.set_title(&info.display_title);
        self.set_kind_badge(info.badge);

        if scheduler == rec.scheduler {
            self.row.set_subtitle(rec.reason);
            self.row.add_css_class("synse-recommended-row");
            self.badge.set_visible(true);
        } else {
            self.row.set_subtitle(&info.blurb);
            self.row.remove_css_class("synse-recommended-row");
            self.badge.set_visible(false);
        }
    }

    fn set_kind_badge(&self, badge: Badge) {
        for b in Badge::ALL {
            self.kind_badge.remove_css_class(b.css_class());
        }
        self.kind_badge.set_label(badge.label());
        self.kind_badge.add_css_class(badge.css_class());
    }

    /// Reflect whether clicking the button starts a scheduler ("Apply") or
    /// stops the one already running with this exact scheduler+mode selection
    /// ("Turn off"). Applying the running selection would be a no-op, so we
    /// offer the only meaningful action instead.
    fn set_action_for(&self, running: &Option<(String, SchedMode)>, scheduler: &str, mode: SchedMode) {
        let already_running = running
            .as_ref()
            .is_some_and(|(n, m)| n.as_str() == scheduler && *m == mode);
        if already_running {
            self.apply.set_label("Turn off");
            self.apply.remove_css_class("suggested-action");
        } else {
            self.apply.set_label("Apply");
            self.apply.add_css_class("suggested-action");
        }
    }

    fn widget(&self) -> &PreferencesGroup {
        &self.group
    }

    fn apply_button(&self) -> &Button {
        &self.apply
    }
}

fn object<T: glib::object::IsA<glib::Object>>(builder: &Builder, id: &str) -> T {
    builder
        .object::<T>(id)
        .unwrap_or_else(|| panic!("widget {id:?} missing from window.ui"))
}

/// Ask scx_loader what's running and update the banner. Polled once a second
/// so external changes (e.g. via `scxctl`) are reflected.
fn refresh_banner(banner: &Banner) {
    let banner = banner.clone();
    glib::spawn_future_local(async move {
        let title = match loader_query::current_state().await {
            Ok(Some((name, _))) => format!("● {name} is running."),
            _ => "sched_ext is supported but no scheduler is running.".to_string(),
        };
        banner.set_title(&title);
    });
}

fn install_status_timer(banner: &Banner) {
    refresh_banner(banner);
    let banner_weak = banner.downgrade();
    glib::timeout_add_seconds_local(1, move || {
        let Some(banner) = banner_weak.upgrade() else {
            return glib::ControlFlow::Break;
        };
        refresh_banner(&banner);
        glib::ControlFlow::Continue
    });
}

fn populate_scheduler_section(scroll_box: &GtkBox, toast_overlay: &ToastOverlay) {
    let scroll_box_weak = scroll_box.downgrade();
    let toast_overlay = toast_overlay.clone();

    glib::spawn_future_local(async move {
        let Some(scroll_box) = scroll_box_weak.upgrade() else { return; };
        // What scx_loader currently runs (canonical name + mode), if anything.
        let running_state = loader_query::current_state().await.ok().flatten();
        match loader_query::supported_schedulers().await {
            Ok(scheds) if !scheds.is_empty() => {
                let probe_data = SystemProbe::detect();
                let rec = recommend(&probe_data);

                // Default selection: whatever is running, else the recommended
                // scheduler (when scx_loader actually offers it), else the
                // first entry so the selection is always well-defined.
                let initial_sel = running_state
                    .as_ref()
                    .map(|(n, _)| n.clone())
                    .or_else(|| scheds.iter().find(|s| s.as_str() == rec.scheduler).cloned())
                    .or_else(|| scheds.first().cloned())
                    .expect("scheds is non-empty");
                // Start the mode picker on the running mode when the running
                // scheduler is the initial selection, so the button correctly
                // reads "Turn off" on launch.
                let initial_mode = match &running_state {
                    Some((n, m)) if *n == initial_sel => *m,
                    _ => rec.mode,
                };

                let picker = Rc::new(SchedulerPicker::new(&scheds, &rec, Some(&initial_sel)));
                let profiles = Rc::new(ProfilePicker::new(initial_mode));
                let hero = Rc::new(Hero::new());
                let running = Rc::new(RefCell::new(running_state));

                hero.show(&initial_sel, &rec);
                profiles.set_visible(describe(&initial_sel).supports_profiles);
                hero.set_action_for(&running.borrow(), &initial_sel, initial_mode);

                {
                    let profiles = profiles.clone();
                    let hero = hero.clone();
                    let rec = rec.clone();
                    let running = running.clone();
                    picker.on_change(move |name| {
                        hero.show(name, &rec);
                        profiles.set_visible(describe(name).supports_profiles);
                        hero.set_action_for(&running.borrow(), name, profiles.selected());
                    });
                }
                {
                    let hero = hero.clone();
                    let running = running.clone();
                    let picker = picker.clone();
                    profiles.on_change(move |mode| {
                        if let Some(sel) = picker.selected() {
                            hero.set_action_for(&running.borrow(), &sel, mode);
                        }
                    });
                }

                // Replace the "Loading…" placeholder with the real widgets.
                while let Some(child) = scroll_box.first_child() {
                    scroll_box.remove(&child);
                }
                scroll_box.append(hero.widget());
                scroll_box.append(profiles.widget());
                scroll_box.append(picker.widget());

                // Keep the list collapsed and move initial focus to Apply *now*,
                // before yielding to the main loop. If we deferred this, GTK's
                // first-frame focus assignment could land on the pre-selected
                // scheduler radio, and AdwExpanderRow auto-expands to reveal a
                // focused child — popping the whole list open on launch.
                picker.collapse();
                hero.apply_button().grab_focus();

                wire_action_button(picker, profiles, hero, running, &toast_overlay);
            }
            Ok(_) => replace_with_error(&scroll_box, "scx_loader reported no schedulers."),
            Err(e) => replace_with_error(&scroll_box, &format!("scx_loader unreachable: {e}")),
        }
    });
}

/// Wire the single hero button. It applies the selected scheduler+mode, unless
/// that selection is already running — then it turns the scheduler off. After
/// each action we update the tracked running state and relabel the button.
fn wire_action_button(
    picker: Rc<SchedulerPicker>,
    profiles: Rc<ProfilePicker>,
    hero: Rc<Hero>,
    running: Rc<RefCell<Option<(String, SchedMode)>>>,
    toast_overlay: &ToastOverlay,
) {
    let helper = Rc::new(tokio::sync::Mutex::new(HelperClient::pkexec()));
    let button = hero.apply_button().clone();

    button.connect_clicked(clone!(
        #[strong] picker,
        #[strong] profiles,
        #[strong] hero,
        #[strong] running,
        #[strong] helper,
        #[strong] toast_overlay,
        #[strong] button,
        move |_| {
            let Some(name) = picker.selected() else {
                toast(&toast_overlay, "Select a scheduler first.");
                return;
            };
            let mode = profiles.selected();
            let turn_off = running
                .borrow()
                .as_ref()
                .is_some_and(|(n, m)| *n == name && *m == mode);
            let req = if turn_off {
                Request::Disable
            } else {
                Request::Apply { scheduler: name.clone(), mode }
            };

            button.set_sensitive(false);
            glib::spawn_future_local(clone!(
                #[strong] helper,
                #[strong] toast_overlay,
                #[strong] running,
                #[strong] hero,
                #[strong] picker,
                #[strong] profiles,
                #[strong] button,
                async move {
                    let result = {
                        let mut guard = helper.lock().await;
                        guard.send(req).await
                    };
                    match result {
                        Ok(Response::Ok) => {
                            if turn_off {
                                running.borrow_mut().take();
                                toast(&toast_overlay, "scx scheduler turned off.");
                            } else {
                                *running.borrow_mut() = Some((name.clone(), mode));
                                toast(&toast_overlay, &format!("Switched to {name} · {mode:?}"));
                            }
                            if let Some(sel) = picker.selected() {
                                hero.set_action_for(&running.borrow(), &sel, profiles.selected());
                            }
                        }
                        Ok(Response::Err { message }) => {
                            toast(&toast_overlay, &format!("Helper error: {message}"))
                        }
                        Err(HelperError::AuthCanceled) => {
                            toast(&toast_overlay, "Authorization canceled.")
                        }
                        Err(e) => toast(&toast_overlay, &format!("Helper failed: {e}")),
                    }
                    button.set_sensitive(true);
                }
            ));
        }
    ));
}

fn toast(overlay: &ToastOverlay, msg: &str) {
    overlay.add_toast(Toast::new(msg));
}

fn replace_with_error(scroll_box: &GtkBox, msg: &str) {
    while let Some(child) = scroll_box.first_child() {
        scroll_box.remove(&child);
    }
    let label = gtk::Label::builder().label(msg).wrap(true).build();
    label.add_css_class("error");
    scroll_box.append(&label);
}
