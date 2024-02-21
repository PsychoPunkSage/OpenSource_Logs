# **GUI development with Rust and GTK 4**

## **Hello World!**

>>> Need to create a `gtk::Application` instance with an `application id`. For that we use the **builder pattern** which many `gtk-rs` objects support. Note that we also import the prelude to bring the necessary traits into scope.

*Filesystem*: ...../hello_world/1/main.rs

```rust
use gtk::prelude::*;
use gtk::{glib, Application};

const APP_ID: &str = "org.gtk_rs.HelloWorld1";

fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Run the application
    app.run()
}
```

* It builds fine, though a warning in our terminal appears.

<details>
<summary>Warning</summary>
****
```
GLib-GIO-WARNING: Your application does not implement g_application_activate()
and has no handlers connected to the 'activate' signal. It should do one of these.
```

</details>

* GTK tells us that something should be called in its **activate** step. So let's create a `gtk::ApplicationWindow` there.

*Filesystem*: ...../hello_world/2/main.rs

```rust
use gtk::prelude::*;
use gtk::{glib, Application, ApplicationWindow};

const APP_ID: &str = "org.gtk_rs.HelloWorld2";

fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn build_ui(app: &Application) {
    // Create a window and set the title
    let window = ApplicationWindow::builder()
        .application(app)
        .title("My GTK App")
        .build();

    // Present window
    window.present();
}
```