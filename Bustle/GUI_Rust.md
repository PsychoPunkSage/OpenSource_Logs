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

* interact with the user interface.

```rust
// use gtk::prelude::*;
// use gtk::{glib, Application, ApplicationWindow, Button};
// const APP_ID: &str = "org.gtk_rs.HelloWorld3";

// fn main() -> glib::ExitCode {
//     // Create a new application
//     let app = Application::builder().application_id(APP_ID).build();

//     // Connect to "activate" signal of `app`
//     app.connect_activate(build_ui);

//     // Run the application
//     app.run()
// }

fn build_ui(app: &Application) {
    // Create a button with label and margins
    let button = Button::builder()
        .label("Press me!")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    // Connect to "clicked" signal of `button`
    button.connect_clicked(|button| {
        // Set the label to "Hello World!" after the button has been clicked on
        button.set_label("Hello World!");
    });

    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("My GTK App")
        .child(&button)
        .build();

    // Present window
    window.present();
}
```

** here is now a **button** and if we click on it, its label becomes `"Hello World!"`.

## **Widgets**

>>> Widgets are the components that make up a GTK application. GTK offers many widgets ( one can even create custom ones). There are, for example, display widgets, buttons, containers and windows. One kind of widget might be able to contain other widgets, it might present information and it might react to interaction.

[Widget Gallery](https://docs.gtk.org/gtk4/visual_index.html) - Best place to find best suited widgets.

* GTK is an object-oriented framework, so all widgets are part of an inheritance tree with GObject at the top. The inheritance tree of a Button looks like this:
```
GObject
╰── Widget
    ╰── Button
```