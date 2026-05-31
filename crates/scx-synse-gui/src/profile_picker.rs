use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use adw::PreferencesGroup;
use glib::clone;
use gtk::{glib, Builder, CheckButton};

use scx_synse_ipc::SchedMode;

const PROFILE_RESOURCE: &str = "/com/synsenetwork/scx-synse-manager/ui/profile_picker.ui";

type ChangeListener = Box<dyn Fn(SchedMode)>;

struct State {
    selected: SchedMode,
    listeners: Vec<ChangeListener>,
}

pub struct ProfilePicker {
    group: PreferencesGroup,
    state: Rc<RefCell<State>>,
}

impl ProfilePicker {
    pub fn new(initial: SchedMode) -> Self {
        let builder = Builder::from_resource(PROFILE_RESOURCE);
        let group: PreferencesGroup = object(&builder, "profile_group");

        let state = Rc::new(RefCell::new(State { selected: initial, listeners: vec![] }));

        for (mode, id) in radio_ids() {
            let radio: CheckButton = object(&builder, id);
            if mode == initial {
                radio.set_active(true);
            }
            radio.connect_toggled(clone!(
                #[strong] state,
                move |r| {
                    if !r.is_active() {
                        return;
                    }
                    let mut s = state.borrow_mut();
                    if s.selected == mode {
                        return;
                    }
                    s.selected = mode;
                    for cb in &s.listeners {
                        cb(mode);
                    }
                }
            ));
        }

        Self { group, state }
    }

    pub fn widget(&self) -> &PreferencesGroup {
        &self.group
    }

    pub fn selected(&self) -> SchedMode {
        self.state.borrow().selected
    }

    pub fn on_change(&self, cb: impl Fn(SchedMode) + 'static) {
        self.state.borrow_mut().listeners.push(Box::new(cb));
    }

    pub fn set_visible(&self, visible: bool) {
        self.group.set_visible(visible);
    }
}

fn radio_ids() -> [(SchedMode, &'static str); 5] {
    [
        (SchedMode::Auto, "profile_radio_auto"),
        (SchedMode::Gaming, "profile_radio_gaming"),
        (SchedMode::PowerSave, "profile_radio_powersave"),
        (SchedMode::LowLatency, "profile_radio_lowlatency"),
        (SchedMode::Server, "profile_radio_server"),
    ]
}

fn object<T: glib::object::IsA<glib::Object>>(builder: &Builder, id: &str) -> T {
    builder
        .object::<T>(id)
        .unwrap_or_else(|| panic!("widget {id:?} missing from profile_picker.ui"))
}
