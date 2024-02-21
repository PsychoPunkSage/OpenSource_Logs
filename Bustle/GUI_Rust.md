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

* In the "Hello World" app we wanted to react to a button click. This behavior is specific to a button, so we expect to find a suitable method in the `ButtonExt` trait. And indeed, `ButtonExt` includes the method **`connect_clicked`**.

## **GObject Concept**

>>> GTK is an object-oriented framework. Written in **C**, which does not support object-orientation. That is why GTK relies on the *GObject library* to provide the object system.

Will focus on:

* How memory of GObjects is managed
* How to create our own GObjects via subclassing
* How to deal with generic values
* How to use properties
* How to emit and receive signals

### Memory Management

>> Memory management when writing a gtk-rs app can be a bit tricky. 

With our first example, we have **window with a single button**. Every button click should increment an integer number by one.

<details>
<summary>Code</summary>

```rust
// use gtk::prelude::*;
// use gtk::{self, glib, Application, ApplicationWindow, Button};

// const APP_ID: &str = "org.gtk_rs.GObjectMemoryManagement0";

// DOES NOT COMPILE!
fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn build_ui(application: &Application) {
    // Create two buttons
    let button_increase = Button::builder()
        .label("Increase")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    // A mutable integer
    let mut number = 0;

    // Connect callbacks
    // When a button is clicked, `number` should be changed
    button_increase.connect_clicked(|_| number += 1);

    // Create a window
    let window = ApplicationWindow::builder()
        .application(application)
        .title("My GTK App")
        .child(&button_increase)
        .build();

    // Present the window
    window.present();
}
```

</details>

<details>
<summary>Error</summary>

```
error[E0373]: closure may outlive the current function, but it borrows `number`, which is owned by the current function
   |
32 |     button_increase.connect_clicked(|_| number += 1);
   |                                     ^^^ ------ `number` is borrowed here
   |                                     |
   |                                     may outlive borrowed value `number`
   |
note: function requires argument type to outlive `'static`
   |
32 |     button_increase.connect_clicked(|_| number += 1);
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
help: to force the closure to take ownership of `number` (and any other referenced variables), use the `move` keyword
   |
32 |     button_increase.connect_clicked(move |_| number += 1);
   |  
```

</details>

* Our closure only borrows `number`. Signal handlers in GTK require `static'` lifetimes for their references, so we cannot borrow a variable that only lives for the scope of the function `build_ui`. The compiler also suggests how to fix this. By adding the move keyword in front of the closure, number will be moved into the closure.

```rust
// DOES NOT COMPILE!
// A mutable integer
let mut number = 0;

// Connect callbacks
// When a button is clicked, `number` should be changed
button_increase.connect_clicked(move |_| number += 1);
```

<details>
<summary>Error</summary>

```
error[E0594]: cannot assign to `number`, as it is a captured variable in a `Fn` closure
   |
32 |     button_increase.connect_clicked(move |_| number += 1);
   |                                              ^^^^^^^^^^^ cannot assign 
```

</details>

> In order to understand that error message we have to understand the difference between the three closure traits **`FnOnce`**, **`FnMut`** and **`Fn`**. APIs that take closures implementing the 

> `FnOnce trait` give the most freedom to the API consumer. The closure is called only once, so it can even consume its state. Signal handlers can be called multiple times, so they cannot accept FnOnce.

> The more restrictive `FnMut trait` doesn't allow closures to consume their state, but they can still mutate it. Signal handlers can't allow this either, because they can be called from inside themselves. This would lead to multiple mutable references which the borrow checker doesn't appreciate at all.

> `Fn`. State can be immutably borrowed, but then how can we modify number? We need a data type with interior mutability like **`std::cell::Cell`**.

`Filesystem`: ...../g_object_memory_management/1/main.rs

```rust 
// use gtk::prelude::*;
// use gtk::{glib, Application, ApplicationWindow, Button};
// use std::cell::Cell;

// const APP_ID: &str = "org.gtk_rs.GObjectMemoryManagement1";

// fn main() -> glib::ExitCode {
//     // Create a new application
//     let app = Application::builder().application_id(APP_ID).build();

//     // Connect to "activate" signal of `app`
//     app.connect_activate(build_ui);

//     // Run the application
//     app.run()
// }

fn build_ui(application: &Application) {
    // Create two buttons
    let button_increase = Button::builder()
        .label("Increase")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    // A mutable integer
    let number = Cell::new(0);

    // Connect callbacks
    // When a button is clicked, `number` should be changed
    button_increase.connect_clicked(move |_| number.set(number.get() + 1));

    // Create a window
    let window = ApplicationWindow::builder()
        .application(application)
        .title("My GTK App")
        .child(&button_increase)
        .build();

    // Present the window
    window.present();
}
```
* compiles as expected.

#### **complicated example:) two buttons which both modify the same number**

> For that, we need a way that both closures take ownership of the same value

> That is exactly what the `std::rc::Rc` type is there for. `Rc` counts the number of strong references created via `Clone::clone` and released via `Drop::drop`, and only `deallocates the value when this number drops to zero`. If we want to modify the content of our `Rc`, we can again use the `Cell` type.

*Filesystem*: ...../g_object_memory_management/2/main.rs

```rust
// Reference-counted object with inner-mutability
let number = Rc::new(Cell::new(0));

// Connect callbacks, when a button is clicked `number` will be changed
let number_copy = number.clone();
button_increase.connect_clicked(move |_| number_copy.set(number_copy.get() + 1));
button_decrease.connect_clicked(move |_| number.set(number.get() - 1));
```
It is not very nice though to fill the scope with temporary variables like `number_copy`. We can improve that by using the `glib::clone!` macro.

*Filesystem*: ...../g_object_memory_management/3/main.rs

```rust
    button_increase.connect_clicked(clone!(@strong number => move |_| {
        number.set(number.get() + 1);
    }));
    button_decrease.connect_clicked(move |_| {
        number.set(number.get() - 1);
    });
```

* Just like Rc\<Cell<T>>, GObjects are *reference-counted* and *mutable*. Therefore, we can pass the buttons the same way to the closure as we did with number.

*Filesystem*: ...../g_object_memory_management/4/main.rs

```rust
    button_increase.connect_clicked(clone!(@strong number => move |_| {
        number.set(number.get() + 1);
    }));
    button_decrease.connect_clicked(move |_| {
        number.set(number.get() - 1);
    });
```
* If we now click on one button, the other button's label gets changed.

<details>
<summary>Error</summary>

[reference cycles](https://doc.rust-lang.org/book/ch15-06-reference-cycles.html). `button_increase` holds a *strong reference* to `button_decrease` and vice-versa. A strong reference keeps the referenced value from being deallocated. If this chain leads to a circle, none of the values in this cycle ever get deallocated. With **weak references** we can break this cycle, because they don't keep their value alive but instead provide a way to retrieve a strong reference if the value is still alive. Since we want our apps to free unneeded memory, we should use weak references for the buttons instead.

</details>

*Filesystem*: ...../g_object_memory_management/5/main.rs
```rust
    // Connect callbacks
    // When a button is clicked, `number` and label of the other button will be changed
    button_increase.connect_clicked(clone!(@weak number, @weak button_decrease =>
        move |_| {
            number.set(number.get() + 1);
            button_decrease.set_label(&number.get().to_string());
    }));
    button_decrease.connect_clicked(clone!(@weak button_increase =>
        move |_| {
            number.set(number.get() - 1);
            button_increase.set_label(&number.get().to_string());
    }));
```
* The reference cycle is broken. 
 
Every time the button is clicked, `glib::clone` tries to upgrade the weak reference. If we now for example click on one button and the other button is not there anymore, the callback will be skipped. Per default, it immediately returns from the closure with `()` as return value. In case the closure expects a different return value **`@default-return`** can be specified.

Notice that we move `number` in the second closure. If we had moved weak references in both closures, nothing would have kept number alive and the closure would have never been called. Thinking about this, `button_increase` and `button_decrease` are also dropped at the end of the scope of `build_ui`. Who then keeps the buttons alive?

*Filesystem*: ...../g_object_memory_management/5/main.rs
```rust
    // Add buttons to `gtk_box`
    let gtk_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();
    gtk_box.append(&button_increase);
    gtk_box.append(&button_decrease);

```
* When we append the buttons to the `gtk_box`, gtk_box keeps a strong reference to them.

*Filesystem*: ...../g_object_memory_management/5/main.rs
```rust
    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("My GTK App")
        .child(&gtk_box)
        .build();
```

When we set `gtk_box` as child of `window`, `window` keeps a strong reference to it. Until we close the `window` it keeps `gtk_box` and with it the buttons alive. Since our application has only one window, closing it also means exiting the application.

As long as you use weak references whenever possible, you will find it perfectly doable to avoid memory cycles within your application. Without memory cycles, you can rely on GTK to properly manage the memory of GObjects you pass to it.