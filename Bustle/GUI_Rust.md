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

### **Memory Management**

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

### **Subclassing**

GObjects rely heavily on inheritance. Therefore, it makes sense that if we want to create a custom GObject, this is done via subclassing. Let's see how this works by replacing the button in our "Hello World!" app with a custom one. First, we need to create an implementation struct that holds the state and overrides the virtual methods.

*Filesystem*: listings/g_object_subclassing/1/custom_button/imp.rs

```rust
use gtk::glib;
use gtk::subclass::prelude::*;

// Object holding the state
#[derive(Default)]
pub struct CustomButton;

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for CustomButton {
    const NAME: &'static str = "MyGtkAppCustomButton";
    type Type = super::CustomButton;
    type ParentType = gtk::Button;
}

// Trait shared by all GObjects
impl ObjectImpl for CustomButton {}

// Trait shared by all widgets
impl WidgetImpl for CustomButton {}

// Trait shared by all buttons
impl ButtonImpl for CustomButton {}
```

The description of the subclassing is in ObjectSubclass.

* `NAME` should consist of crate-name and object-name in order to avoid name collisions. Use UpperCamelCase here.
* `Type` refers to the actual GObject that will be created afterwards.
* `ParentType` is the GObject we inherit of.

After that, we would have the option to override the virtual methods of ancestors. Since we only want to have a plain button for now, we override nothing. We still have to add the empty `impl` though. Next, we describe the public interface of our custom GObject.

*Filesystem*: ...../g_object_subclassing/1/custom_button/mod.rs

```rust
mod imp;

use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct CustomButton(ObjectSubclass<imp::CustomButton>)
        @extends gtk::Button, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl CustomButton {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn with_label(label: &str) -> Self {
        Object::builder().property("label", label).build()
    }
}

// impl Default for CustomButton {
//     fn default() -> Self {
//         Self::new()
//     }
// }
```

**glib::wrapper!** implements the same traits that our `ParentType` implements. Theoretically that would mean that the `ParentType` is also the only thing we have to specify here. Unfortunately, nobody has yet found a good way to do that. Which is why, as of today, subclassing of GObjects in Rust requires to mention all ancestors and interfaces apart from `GObject` and `GInitiallyUnowned`. For `gtk::Button`.

After these steps, nothing is stopping us from replacing gtk::Button with our CustomButton.

*Filesystem*: ...../g_object_subclassing/1/main.rs

```rust
mod custom_button;

use custom_button::CustomButton;
use gtk::prelude::*;
use gtk::{glib, Application, ApplicationWindow};

const APP_ID: &str = "org.gtk_rs.GObjectSubclassing1";

fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn build_ui(app: &Application) {
    // Create a button
    let button = CustomButton::with_label("Press me!");
    button.set_margin_top(12);
    button.set_margin_bottom(12);
    button.set_margin_start(12);
    button.set_margin_end(12);

    // Connect to "clicked" signal of `button`
    button.connect_clicked(move |button| {
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

#### Adding Functionality

We are able to use `CustomButton` as a drop-in replacement for `gtk::Button`. This is cool, but also not very tempting to do in a real application. For the gain of zero benefits, it did involve quite a bit of boilerplate after all.

So let's make it a bit more interesting! `gtk::Button` does not hold much state, but we can let `CustomButton` hold a number.

*Filesystem*: ...../g_object_subclassing/2/custom_button/imp.rs

```rust
use std::cell::Cell;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

// Object holding the state
#[derive(Default)]
pub struct CustomButton {
    number: Cell<i32>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for CustomButton {
    const NAME: &'static str = "MyGtkAppCustomButton";
    type Type = super::CustomButton;
    type ParentType = gtk::Button;
}

// Trait shared by all GObjects
impl ObjectImpl for CustomButton {
    fn constructed(&self) {
        self.parent_constructed();
        self.obj().set_label(&self.number.get().to_string());
    }
}

// Trait shared by all widgets
impl WidgetImpl for CustomButton {}

// Trait shared by all buttons
impl ButtonImpl for CustomButton {
    fn clicked(&self) {
        self.number.set(self.number.get() + 1);
        self.obj().set_label(&self.number.get().to_string())
    }
}
```

We override `constructed` in `ObjectImpl` so that the label of the button initializes with `number`. We also override `clicked` in `ButtonImpl` so that every click increases `number` and updates the label.

*Filesystem*: ...../g_object_subclassing/2/main.rs

```rust
// mod custom_button;

// use custom_button::CustomButton;
// use gtk::prelude::*;
// use gtk::{glib, Application, ApplicationWindow};

// const APP_ID: &str = "org.gtk_rs.GObjectSubclassing2";

// fn main() -> glib::ExitCode {
//     // Create a new application
//     let app = Application::builder().application_id(APP_ID).build();

//     // Connect to "activate" signal of `app`
//     app.connect_activate(build_ui);

//     // Run the application
//     app.run()
// }

fn build_ui(app: &Application) {
    // Create a button
    let button = CustomButton::new();
    button.set_margin_top(12);
    button.set_margin_bottom(12);
    button.set_margin_start(12);
    button.set_margin_end(12);

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

In `build_ui` we stop calling `connect_clicked`, and that was it. After a rebuild, the app now features our `CustomButton` with the label **"0"**. Every time we click on the button, the number displayed by the label increases by 1.


### **Generic Values**

>> Some GObject-related functions rely on generic values for their arguments or return parameters. Since GObject introspection works through a C interface, these functions cannot rely on any powerful Rust concepts. In these cases `glib::Value` or `glib::Variant` are used.

#### Value

> Conceptually, a `Value` is similar to a Rust `enum` defined like this:

```rust
enum Value <T> {
    bool(bool),
    i8(i8),
    i32(i32),
    u32(u32),
    i64(i64),
    u64(u64),
    f32(f32),
    f64(f64),
    // boxed types
    String(Option<String>),
    Object(Option<dyn IsA<glib::Object>>),
}
```


* `Value` representing an `i32`.
`Filesystem`: ...../g_object_values/1/main.rs

```rust
    // Store `i32` as `Value`
    let integer_value = 10.to_value();

    // Retrieve `i32` from `Value`
    let integer = integer_value
        .get::<i32>()
        .expect("The value needs to be of type `i32`.");

    // Check if the retrieved value is correct
    assert_eq!(integer, 10);
```

Also note that in the `enum` above boxed types such as `String` or `glib::Object` are wrapped in an `Option`. This comes from C, where every boxed type can potentially be `None` (or `NULL` in C terms). You can still access it the same way as with the `i32` above. **`get`** will then not only return `Err` if you specified the wrong type, but also if the `Value` represents `None`.

`Filesystem`: ...../g_object_values/1/main.rs

```rust
    // Store string as `Value`
    let string_value = "Hello!".to_value();

    // Retrieve `String` from `Value`
    let string = string_value
        .get::<String>()
        .expect("The value needs to be of type `String`.");

    // Check if the retrieved value is correct
    assert_eq!(string, "Hello!".to_string());
```

If want to differentiate between specifying the wrong type and a `Value` representing `None`, just call `get::<Option<T>>` instead.

`Filesystem`: ...../g_object_values/1/main.rs

```rust
    // Store `Option<String>` as `Value`
    let string_some_value = "Hello!".to_value();
    let string_none_value = None::<String>.to_value();

    // Retrieve `String` from `Value`
    let string_some = string_some_value
        .get::<Option<String>>()
        .expect("The value needs to be of type `Option<String>`.");
    let string_none = string_none_value
        .get::<Option<String>>()
        .expect("The value needs to be of type `Option<String>`.");

    // Check if the retrieved value is correct
    assert_eq!(string_some, Some("Hello!".to_string()));
    assert_eq!(string_none, None);
```
We will use `Value` when we deal with properties and signals later on.

#### Variant

A `Variant` is used whenever data needs to be serialized, for example for sending it to another process or over the network, or for storing it on disk. Although `GVariant` supports arbitrarily complex types, the Rust bindings are currently limited to `bool`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`, `f64`, `&str/String`, and [VariantDict](https://gtk-rs.org/gtk-rs-core/stable/latest/docs/glib/struct.VariantDict.html). Containers of the above types are possible as well, such as `HashMap`, `Vec`, `Option`, `tuples` up to 16 elements, and `Variant`. Variants can even be derived from Rust structs as long as its members can be represented by variants.

In the most simple case, converting Rust types to `Variant` and vice-versa is very similar to the way it worked with `Value`.

`Filesystem`: ...../g_object_values/1/main.rs

```rust
    // Store `i32` as `Variant`
    let integer_variant = 10.to_variant();

    // Retrieve `i32` from `Variant`
    let integer = integer_variant
        .get::<i32>()
        .expect("The variant needs to be of type `i32`.");

    // Check if the retrieved value is correct
    assert_eq!(integer, 10);
```

However, a `Variant` is also able to represent containers such as `HashMap` or `Vec`. The following snippet shows how to convert between `Vec` and `Variant`. More examples can be found in the docs.

`Filesystem`: ...../g_object_values/2/main.rs

```rust
    let variant = vec!["Hello", "there!"].to_variant();
    assert_eq!(variant.n_children(), 2);
    let vec = &variant
        .get::<Vec<String>>()
        .expect("The variant needs to be of type `String`.");
    assert_eq!(vec[0], "Hello");
```

We will use Variant when saving settings using `gio::Settings` or activating actions via `gio::Action`.

### **Properties**

>> Properties provide a public API for accessing state of GObjects.<br>
>> Experimentation with the `Switch` widget. One of its properties is called **active**. According to the GTK docs, it can be read and be written to. That is why `gtk-rs` provides corresponding **is_active** and **set_acti`** methods.

`Filesystem`: ...../g_object_values/1/main.rs

```rust
    // Create the switch
    let switch = Switch::new();

    // Set and then immediately obtain active property
    switch.set_active(true);
    let switch_active = switch.is_active();

    // This prints: "The active property of switch is true"
    println!("The active property of switch is {}", switch_active);
```
Properties can not only be accessed via getters & setters, they can also be bound to each other. Let's see how that would look like for two `Switch` instances.

`Filesystem`: ...../g_object_values/2/main.rs

```rust
    // Create the switches
    let switch_1 = Switch::new();
    let switch_2 = Switch::new();
```

In our case, we want to bind the "active" property of `switch_1` to the "active" property of `switch_2`. We also want the binding to be bidirectional, so we specify by calling the **bidirectional** method.

`Filesystem`: ...../g_object_values/2/main.rs

```rust
    switch_1
        .bind_property("active", &switch_2, "active")
        .bidirectional()
        .build();
```

#### Adding Properties to Custom GObjects

We can also add properties to custom GObjects. We can demonstrate that by binding the number of our `CustomButton` to a property. Most of the work is done by the **glib::Properties** derive macro. We tell it that the wrapper type is `super::CustomButton`. We also annotate `number`, so that macro knows that it should create a property "number" that is readable and writable. It also generates wrapper.

`Filesystem`: ...../g_object_properties/3/custom_button/imp.rs

```rust
// Object holding the state
#[derive(Properties, Default)]
#[properties(wrapper_type = super::CustomButton)]
pub struct CustomButton {
    #[property(get, set)]
    number: Cell<i32>,
}
```

The **glib::derived_properties** macro generates boilerplate that is the same for every GObject that generates its properties with the `Property` macro. In `constructed` we use our new property "number" by binding the "label" property to it. `bind_property` converts the integer value of "number" to the string of "label" on its own. Now we don't have to adapt the label in the "clicked" callback anymore.

`Filesystem`: ...../g_object_properties/3/custom_button/imp.rs

```rust
// Trait shared by all GObjects
#[glib::derived_properties]
impl ObjectImpl for CustomButton {
    fn constructed(&self) {
        self.parent_constructed();

        // Bind label to number
        // `SYNC_CREATE` ensures that the label will be immediately set
        let obj = self.obj();
        obj.bind_property("number", obj.as_ref(), "label")
            .sync_create()
            .build();
    }
}
```

We also have to adapt the `clicked` method. Before we modified `number` directly, now we can use the generated wrapper methods `number` and `set_number`. This way the "notify" signal will be emitted, which is necessary for the bindings to work as expected.

`Filesystem`: ...../g_object_properties/3/custom_button/imp.rs

```rust
// Trait shared by all GObjects
#[glib::derived_properties]
// Trait shared by all buttons
impl ButtonImpl for CustomButton {
    fn clicked(&self) {
        let incremented_number = self.obj().number() + 1;
        self.obj().set_number(incremented_number);
    }
}
```

Let's see what we can do with this by creating two custom buttons.

`Filesystem`: ...../g_object_properties/3/main.rs

```rust
    // Create the buttons
    let button_1 = CustomButton::new();
    let button_2 = CustomButton::new();
```

We have already seen that bound properties don't necessarily have to be of the same type. By leveraging `transform_to` and `transform_from`, we can assure that `button_2` always displays a number which is 1 higher than the number of `button_1`.

`Filesystem`: ...../g_object_properties/3/main.rs

```rust
    // Assure that "number" of `button_2` is always 1 higher than "number" of `button_1`
    button_1
        .bind_property("number", &button_2, "number")
        // How to transform "number" from `button_1` to "number" of `button_2`
        .transform_to(|_, number: i32| {
            let incremented_number = number + 1;
            Some(incremented_number.to_value())
        })
        // How to transform "number" from `button_2` to "number" of `button_1`
        .transform_from(|_, number: i32| {
            let decremented_number = number - 1;
            Some(decremented_number.to_value())
        })
        .bidirectional()
        .sync_create()
        .build();
```
Now if we click on one button, the "number" and "label" properties of the other button change as well.

Another nice feature of properties is, that you can connect a callback to the event, when a property gets changed. For example like this:

`Filesystem`: ...../g_object_properties/3/main.rs

```rust
    // The closure will be called
    // whenever the property "number" of `button_1` gets changed
    button_1.connect_number_notify(|button| {
        println!("The current number of `button_1` is {}.", button.number());
    });
```

Now, whenever the "number" property gets changed, the closure gets executed and prints the current value of "number" to standard output.

Introducing properties to your custom GObjects is useful if you want to

* bind state of (different) GObjects
* notify consumers whenever a property value changes

Note that it has a (computational) cost to send a signal each time the value changes. If you only want to expose internal state, adding getter and setter methods is the better option.