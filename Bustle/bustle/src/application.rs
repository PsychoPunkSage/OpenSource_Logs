use adw::subclass::prelude::*;
use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
};

use crate::{
    config::{APP_ID, PKGDATADIR, PROFILE, VERSION},
    window::Window,
};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct Application;

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "BustleApplication";
        type Type = super::Application;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for Application {}

    impl ApplicationImpl for Application {
        fn activate(&self) {
            tracing::debug!("GtkApplication<Application>::activate");
            self.parent_activate();
            let app = self.obj();

            if let Some(window) = app.active_window() {
                window.present();
                return;
            }

            let window = Window::with_group(&app);
            window.present();
        }

        fn startup(&self) {
            tracing::debug!("GtkApplication<Application>::startup");
            self.parent_startup();
            let app = self.obj();

            // Set icons for shell
            gtk::Window::set_default_icon_name(APP_ID);

            app.setup_gactions();
            app.setup_accels();
        }

        fn open(&self, files: &[gio::File], _hint: &str) {
            let app = self.obj();
            for file in files {
                let window = Window::with_group(&app);
                glib::spawn_future_local(clone!(@strong window, @strong file => async move {
                    if let Err(err) = window.load_log(&file).await {
                        tracing::error!("Failed to load log {err:?}")
                    }
                }));
                window.present();
            }
        }
    }

    impl GtkApplicationImpl for Application {}
    impl AdwApplicationImpl for Application {}
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl Application {
    fn setup_gactions(&self) {
        // Quit
        let action_quit = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| {
                // This is needed to trigger the delete event and saving the window state
                if let Some(window) = app.active_window() {
                    window.close();
                }
                app.quit();
            })
            .build();
        let new_window_action = gio::ActionEntryBuilder::new("new-window")
            .activate(|app, _, _| {
                let window = Window::with_group(app);
                window.present();
            })
            .build();
        self.add_action_entries([action_quit, new_window_action]);
    }

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<Control>q"]);
        self.set_accels_for_action("app.new-window", &["<Control>n"]);
        self.set_accels_for_action("window.close", &["<Control>w"]);

        self.set_accels_for_action("win.statistics", &["F9"]);
        self.set_accels_for_action("win.filter-services", &["<Control>f"]);
        self.set_accels_for_action("win.record-session-bus", &["<Control>e"]);
        self.set_accels_for_action("win.record-system-bus", &["<Control>y"]);
        self.set_accels_for_action("win.record-address", &["<Control>a"]);
        self.set_accels_for_action("win.open-log", &["<Control>o"]);

        self.set_accels_for_action("win.save", &["<Control>s"]);
        self.set_accels_for_action("win.save-dot", &["<Control><Alt>s"]);
    }

    pub fn run(&self) -> glib::ExitCode {
        tracing::info!("Bustle ({})", APP_ID);
        tracing::info!("Version: {} ({})", VERSION, PROFILE);
        tracing::info!("Datadir: {}", PKGDATADIR);

        ApplicationExtManual::run(self)
    }
}

impl Default for Application {
    fn default() -> Self {
        glib::Object::builder()
            .property("application-id", APP_ID)
            .property("resource-base-path", "/org/freedesktop/Bustle/")
            .property("flags", gio::ApplicationFlags::HANDLES_OPEN)
            .build()
    }
}
