use adw::prelude::*;
use adw::Application;
use gio::resources_register_include;

use crate::window::SynseWindow;

const APP_ID: &str = "com.synsenetwork.scx-synse-manager";

pub struct SynseApplication {
    inner: Application,
}

impl SynseApplication {
    pub fn new() -> Self {
        resources_register_include!("scx-synse-manager.gresource")
            .expect("embedded resources missing");

        let inner = Application::builder()
            .application_id(APP_ID)
            .flags(gio::ApplicationFlags::FLAGS_NONE)
            .build();

        inner.connect_startup(|_| {
            let Some(display) = gtk::gdk::Display::default() else {
                return;
            };

            // Ship our own (Font Awesome) icons and ignore whatever icon theme
            // the host system has installed, so the app looks identical
            // everywhere. Clearing the search path drops system icon dirs;
            // GTK's own built-in widget icons (window controls, expander
            // chevrons, …) live in a builtin resource and stay available.
            let icon_theme = gtk::IconTheme::for_display(&display);
            icon_theme.set_search_path(&[] as &[&std::path::Path]);
            icon_theme.add_resource_path("/com/synsenetwork/scx-synse-manager/icons");

            // Tell GTK which icon to hand the compositor for the window (the
            // app icon is bundled at the resource path above). Without this,
            // and with the system search path cleared, windows get a generic
            // placeholder icon.
            gtk::Window::set_default_icon_name(APP_ID);

            let provider = gtk::CssProvider::new();
            provider.load_from_resource("/com/synsenetwork/scx-synse-manager/style.css");
            gtk::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        });

        inner.connect_activate(|app| {
            let window = SynseWindow::build(app);
            window.present();
        });

        Self { inner }
    }

    pub fn run(&self) -> i32 {
        self.inner.run().value()
    }
}
