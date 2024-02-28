mod task_object;
mod task_row;
mod window;

use gtk::prelude::*;
use gtk::{gio, glib, Application};
use window::Window;

// ANCHOR: main
fn main() -> glib::ExitCode {
    // Register and include resources
    // let out_dir = std::env::var("OUT_DIR").expect("Failed to get OUT_DIR");
    // println!("=================< OUT_DIR: {} >=================", out_dir);
    // let resource_path = std::path::Path::new(&out_dir).join("todo_1.gresource");
    // println!(
    //     "=================< Resource path: {} >=================",
    //     resource_path.to_str().unwrap()
    // );
    // gio::resources_register_include!(resource_path.to_str().unwrap())
    //     .expect("Failed to register resources.");

    gio::resources_register_include!("todo_1.gresource").expect("Failed to register resources.");

    // Create a new application
    let app = Application::builder()
        .application_id("org.gtk_rs.Todo1")
        .build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn build_ui(app: &Application) {
    // Create a new custom window and present it
    let window = Window::new(app);
    window.present();
}
// ANCHOR_END: main
