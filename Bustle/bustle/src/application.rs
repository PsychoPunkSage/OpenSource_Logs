// Import necessary modules and traits
use adw::subclass::prelude::*;
use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
};

// Import local modules
use crate::{
    config::{APP_ID, PKGDATADIR, PROFILE, VERSION},
    window::Window,
};

// Define module
mod imp {
    use super::*;

    // Define Application struct
    #[derive(Debug, Default)]
    pub struct Application;

    // Implement ObjectSubclass for Application
    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "BustleApplication";
        type Type = super::Application;
        type ParentType = adw::Application;
    }

    // Implement ObjectImpl for Application
    impl ObjectImpl for Application {}

    // Implement ApplicationImpl for Application
    impl ApplicationImpl for Application {
        // Define behavior when application is activated
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

        // Define behavior when application is started up
        fn startup(&self) {
            tracing::debug!("GtkApplication<Application>::startup"); // This line logs a debug message indicating that the startup method of the Application struct is being called
            self.parent_startup(); // calls the parent_startup method of the current object.
            let app = self.obj();

            // Set icons for shell
            gtk::Window::set_default_icon_name(APP_ID);

            app.setup_gactions();
            app.setup_accels();
        }

        // Define behavior when application is opened
        fn open(&self, files: &[gio::File], _hint: &str) {
            let app = self.obj();
            for file in files {
                let window = Window::with_group(&app);
                glib::spawn_future_local(
                    clone!(@strong window, @strong file => async move { // load the content of the file into the window
                        if let Err(err) = window.load_log(&file).await { // Attempts to load the log from the file asynchronously
                            tracing::error!("Failed to load log {err:?}")
                        }
                    }),
                );
                window.present();
            }
        }
    }

    // Implement GtkApplicationImpl for Application
    impl GtkApplicationImpl for Application {}
    // Implement AdwApplicationImpl for Application
    impl AdwApplicationImpl for Application {}
}

// Define Application struct
glib::wrapper! { // wrapper type around a GObject subclass, which allows you to interact with it more easily from Rust code
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

// Implement methods for Application struct
impl Application {
    // Setup actions for GActions
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

        // >>>>>>>>>>>>>>>>>>>><<<<<<<<<<<<<<<<<<<<<<<<<<<
        // >>>>>>> Probabale area to solve it <<<<<<<<<<<<
        // >>>>>>>>>>>>>>>>>>>><<<<<<<<<<<<<<<<<<<<<<<<<<<

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
        // More shortcuts setup...
    }

    // Run the application
    pub fn run(&self) -> glib::ExitCode {
        tracing::info!("Bustle ({})", APP_ID);
        tracing::info!("Version: {} ({})", VERSION, PROFILE);
        tracing::info!("Datadir: {}", PKGDATADIR);

        ApplicationExtManual::run(self)
    }
}

// Implement Default trait for Application
impl Default for Application {
    fn default() -> Self {
        glib::Object::builder()
            .property("application-id", APP_ID)
            .property("resource-base-path", "/org/freedesktop/Bustle/")
            .property("flags", gio::ApplicationFlags::HANDLES_OPEN)
            .build()
    }
}
