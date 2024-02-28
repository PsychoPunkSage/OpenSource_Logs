# **Building a Simple To-Do App**

## **Window**

> This mockup can be described by the following composite template.

`Filename`: 1/resources/window.ui
<details>
<summary>COde</summary>

```xml
<?xml version="1.0" encoding="UTF-8"?>
<interface>
  <template class="TodoWindow" parent="GtkApplicationWindow">
    <property name="width-request">360</property>
    <property name="title" translatable="yes">To-Do</property>
    <child>
      <object class="GtkBox">
        <property name="orientation">vertical</property>
        <property name="margin-top">12</property>
        <property name="margin-bottom">12</property>
        <property name="margin-start">12</property>
        <property name="margin-end">12</property>
        <property name="spacing">6</property>
        <child>
          <object class="GtkEntry" id="entry">
            <property name="placeholder-text" translatable="yes">Enter a Task…</property>
            <property name="secondary-icon-name">list-add-symbolic</property>
          </object>
        </child>
        <child>
          <object class="GtkScrolledWindow">
            <property name="hscrollbar-policy">never</property>
            <property name="min-content-height">360</property>
            <property name="vexpand">true</property>
            <child>
              <object class="GtkListView" id="tasks_list">
                <property name="valign">start</property>
              </object>
            </child>
          </object>
        </child>
      </object>
    </child>
  </template>
</interface>

```

</details><br>

In order to use the composite template, we create a custom widget. The `parent` is `gtk::ApplicationWindow`, so we inherit from it. As usual, we have to list all ancestors and interfaces apart from `GObject` and `GInitiallyUnowned`.

`Filename`: 1/window/mod.rs
<details>
<summary>COde</summary>

```rust
glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}
```

</details><br>

Then we initialize the composite template for `imp::Window`. We store references to the entry, the list view as well as the list model. This will come in handy when we later add methods to our window. After that, we add the typical boilerplate for initializing composite templates. We only have to assure that the `class` attribute of the template in `window.ui` matches `NAME`.

`Filename`: 1/window/imp.rs
<details>
<summary>COde</summary>

```rust
// Object holding the state
#[derive(CompositeTemplate, Default)]
#[template(resource = "/org/gtk_rs/Todo1/window.ui")]
pub struct Window {
    #[template_child]
    pub entry: TemplateChild<Entry>,
    #[template_child]
    pub tasks_list: TemplateChild<ListView>,
    pub tasks: RefCell<Option<gio::ListStore>>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for Window {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "TodoWindow";
    type Type = super::Window;
    type ParentType = gtk::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}
```

</details><br>

`Filename`: 1/main.rs
<details>
<summary>COde</summary>

```rust
fmod task_object;
mod task_row;
mod window;

use gtk::prelude::*;
use gtk::{gio, glib, Application};
use window::Window;

fn main() -> glib::ExitCode {
    // Register and include resources
    gio::resources_register_include!("todo_1.gresource")
        .expect("Failed to register resources.");

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
```

</details><br>

Finally, we specify our resources...

`Filename`: 1/resources/resources.gresource.xml
<details>
<summary>COde</summary>

```xml
<?xml version="1.0" encoding="UTF-8"?>
<gresources>
  <gresource prefix="/org/gtk_rs/Todo1/">
    <file compressed="true" preprocess="xml-stripblanks">task_row.ui</file>
    <file compressed="true" preprocess="xml-stripblanks">window.ui</file>
  </gresource>
</gresources>
```

</details><br>

## **Task Object**

>> The main user interface is done, but the entry does not react to input yet. Also, where would the input go? We haven't even set up the list model yet.

we start out by creating a custom GObject. This object will store the state of the task consisting of:

* a boolean describing whether the task is completed or not, and
* a string holding the task name.

`Filename`: 1/task_object/mod.rs
<details>
<summary>COde</summary>

```rust
glib::wrapper! {
    pub struct TaskObject(ObjectSubclass<imp::TaskObject>);
}

impl TaskObject {
    pub fn new(completed: bool, content: String) -> Self {
        Object::builder()
            .property("completed", completed)
            .property("content", content)
            .build()
    }
}
```

</details><br>

the state is stored in a struct rather than in individual members of `imp::TaskObject`. This will be very convenient when saving the state in one of the following chapters.

`Filename`: 1/task_object/mod.rs
<details>
<summary>COde</summary>

```rust
#[derive(Default)]
pub struct TaskData {
    pub completed: bool,
    pub content: String,
}
```

</details><br>

We are going to expose `completed` and `content` as properties. Since the data is now inside a struct rather than individual member variables we have to add more annotations. For each property we additionally specify the name, the type and which member variable of `TaskData` we want to access.

`Filename`: 1/task_object/imp.rs
<details>
<summary>COde</summary>

```rust
// Object holding the state
#[derive(Properties, Default)]
#[properties(wrapper_type = super::TaskObject)]
pub struct TaskObject {
    #[property(name = "completed", get, set, type = bool, member = completed)]
    #[property(name = "content", get, set, type = String, member = content)]
    pub data: RefCell<TaskData>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for TaskObject {
    const NAME: &'static str = "TodoTaskObject";
    type Type = super::TaskObject;
}

// Trait shared by all GObjects
#[glib::derived_properties]
impl ObjectImpl for TaskObject {}
```

</details><br>

## **Task Row**

>> Let's move on to the individual tasks.

`Filename`: 1/resources/task_row.ui
<details>
<summary>COde</summary>

```xml
<?xml version="1.0" encoding="UTF-8"?>
<interface>
  <template class="TodoTaskRow" parent="GtkBox">
    <child>
      <object class="GtkCheckButton" id="completed_button">
        <property name="margin-top">12</property>
        <property name="margin-bottom">12</property>
        <property name="margin-start">12</property>
        <property name="margin-end">12</property>
      </object>
    </child>
    <child>
      <object class="GtkLabel" id="content_label">
        <property name="margin-top">12</property>
        <property name="margin-bottom">12</property>
        <property name="margin-start">12</property>
        <property name="margin-end">12</property>
      </object>
    </child>
  </template>
</interface>
```

</details><br>

* In the code, we derive `TaskRow` from `gtk::Box`

`Filename`: 1/task_row/mod.rs
<details>
<summary>COde</summary>

```rust
glib::wrapper! {
    pub struct TaskRow(ObjectSubclass<imp::TaskRow>)
    @extends gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}
```

</details><br>

In `imp::TaskRow`, we hold references to `completed_button` and `content_label`. We also store a mutable vector of bindings. Why we need that will become clear as soon as we get to bind the state of `TaskObject` to the corresponding `TaskRow`.

`Filename`: 1/task_row/imp.rs
<details>
<summary>COde</summary>

```rust
// Object holding the state
#[derive(Default, CompositeTemplate)]
#[template(resource = "/org/gtk_rs/Todo1/task_row.ui")]
pub struct TaskRow {
    #[template_child]
    pub completed_button: TemplateChild<CheckButton>,
    #[template_child]
    pub content_label: TemplateChild<Label>,
    // Vector holding the bindings to properties of `TaskObject`
    pub bindings: RefCell<Vec<Binding>>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for TaskRow {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "TodoTaskRow";
    type Type = super::TaskRow;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}
```

</details><br>

Now we can bring everything together. We override the `imp::Window::constructed` in order to set up window contents at the time of its construction.

`Filename`: 1/window/imp.rs
<details>
<summary>COde</summary>

```rust
// Trait shared by all GObjects
impl ObjectImpl for Window {
    fn constructed(&self) {
        // Call "constructed" on parent
        self.parent_constructed();

        // Setup
        let obj = self.obj();
        obj.setup_tasks();
        obj.setup_callbacks();
        obj.setup_factory();
    }
}
```

</details><br>

Since we need to access the list model quite often, we add the convenience method `Window::model` for that. In `Window::setup_tasks` we create a new model. Then we store a reference to the model in `imp::Window` as well as in `gtk::ListView`.

`Filename`: 1/window/mod.rs
<details>
<summary>COde</summary>

```rust
    fn tasks(&self) -> gio::ListStore {
        // Get state
        self.imp()
            .tasks
            .borrow()
            .clone()
            .expect("Could not get current tasks.")
    }

    fn setup_tasks(&self) {
        // Create new model
        let model = gio::ListStore::new::<TaskObject>();

        // Get state and set model
        self.imp().tasks.replace(Some(model));

        // Wrap model with selection and pass it to the list view
        let selection_model = NoSelection::new(Some(self.tasks()));
        self.imp().tasks_list.set_model(Some(&selection_model));
    }
```

</details><br>

We also create a method `new_task` which takes the content of the entry, clears the entry and uses the content to create a new task.

`Filename`: 1/window/mod.rs
<details>
<summary>COde</summary>

```rust
    fn new_task(&self) {
        // Get content from entry and clear it
        let buffer = self.imp().entry.buffer();
        let content = buffer.text().to_string();
        if content.is_empty() {
            return;
        }
        buffer.set_text("");

        // Add new task to model
        let task = TaskObject::new(false, content);
        self.tasks().append(&task);
    }
```

</details><br>

In `Window::setup_callbacks` we connect to the "activate" signal of the entry. This signal is triggered when we press the enter key in the entry. Then a new `TaskObject` with the content will be created and appended to the model. Finally, the entry will be cleared.

`Filename`: 1/window/mod.rs
<details>
<summary>COde</summary>

```rust
    fn new_task(&self) {
        // Get content from entry and clear it
        let buffer = self.imp().entry.buffer();
        let content = buffer.text().to_string();
        if content.is_empty() {
            return;
        }
        buffer.set_text("");

        // Add new task to model
        let task = TaskObject::new(false, content);
        self.tasks().append(&task);
    }
```

</details><br>

In `Window::setup_callbacks` we connect to the "activate" signal of the entry. This signal is triggered when we press the enter key in the entry. Then a new `TaskObject` with the content will be created and appended to the model. Finally, the entry will be cleared.

`Filename`: 1/window/mod.rs
<details>
<summary>COde</summary>

```rust
    fn setup_callbacks(&self) {
        // Setup callback for activation of the entry
        self.imp()
            .entry
            .connect_activate(clone!(@weak self as window => move |_| {
                window.new_task();
            }));

        // Setup callback for clicking (and the releasing) the icon of the entry
        self.imp().entry.connect_icon_release(
            clone!(@weak self as window => move |_,_| {
                window.new_task();
            }),
        );
    }
```

</details><br>

The list elements for the `gtk::ListView` are produced by a factory. Before we move on to the implementation, let's take a step back and think about which behavior we expect here. `content_label` of `TaskRow` should follow `content` of `TaskObject`. We also want `completed_button` of `TaskRow` follow `completed` of `TaskObject`. This could be achieved with expressions similar to what we did in the lists chapter.

However, if we toggle the state of `completed_button` of `TaskRow`, `completed` of `TaskObject` should change too. Unfortunately, expressions cannot handle bidirectional relationships. This means we have to use property bindings. We will need to unbind them manually when they are no longer needed.

We will create empty `TaskRow` objects in the "setup" step in `Window::setup_factory` and deal with binding in the "bind" and "unbind" steps.

`Filename`: 1/window/mod.rs
<details>
<summary>COde</summary>

```rust
    fn setup_factory(&self) {
        // Create a new factory
        let factory = SignalListItemFactory::new();

        // Create an empty `TaskRow` during setup
        factory.connect_setup(move |_, list_item| {
            // Create `TaskRow`
            let task_row = TaskRow::new();
            list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
                .set_child(Some(&task_row));
        });

        // Tell factory how to bind `TaskRow` to a `TaskObject`
        factory.connect_bind(move |_, list_item| {
            // Get `TaskObject` from `ListItem`
            let task_object = list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
                .item()
                .and_downcast::<TaskObject>()
                .expect("The item has to be an `TaskObject`.");

            // Get `TaskRow` from `ListItem`
            let task_row = list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
                .child()
                .and_downcast::<TaskRow>()
                .expect("The child has to be a `TaskRow`.");

            task_row.bind(&task_object);
        });

        // Tell factory how to unbind `TaskRow` from `TaskObject`
        factory.connect_unbind(move |_, list_item| {
            // Get `TaskRow` from `ListItem`
            let task_row = list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
                .child()
                .and_downcast::<TaskRow>()
                .expect("The child has to be a `TaskRow`.");

            task_row.unbind();
        });

        // Set the factory of the list view
        self.imp().tasks_list.set_factory(Some(&factory));
    }
```

</details><br>

Binding properties in `TaskRow::bind` works just like in former chapters. The only difference is that we store the bindings in a vector. This is necessary because a `TaskRow` will be reused as you scroll through the list. That means that over time a TaskRow will need to bound to a new `TaskObject` and has to be unbound from the old one. Unbinding will only work if it can access the stored `glib::Binding`.

`Filename`: 1/task_row/mod.rs
<details>
<summary>COde</summary>

```rust
    pub fn bind(&self, task_object: &TaskObject) {
        // Get state
        let completed_button = self.imp().completed_button.get();
        let content_label = self.imp().content_label.get();
        let mut bindings = self.imp().bindings.borrow_mut();

        // Bind `task_object.completed` to `task_row.completed_button.active`
        let completed_button_binding = task_object
            .bind_property("completed", &completed_button, "active")
            .bidirectional()
            .sync_create()
            .build();
        // Save binding
        bindings.push(completed_button_binding);

        // Bind `task_object.content` to `task_row.content_label.label`
        let content_label_binding = task_object
            .bind_property("content", &content_label, "label")
            .sync_create()
            .build();
        // Save binding
        bindings.push(content_label_binding);

        // Bind `task_object.completed` to `task_row.content_label.attributes`
        let content_label_binding = task_object
            .bind_property("completed", &content_label, "attributes")
            .sync_create()
            .transform_to(|_, active| {
                let attribute_list = AttrList::new();
                if active {
                    // If "active" is true, content of the label will be strikethrough
                    let attribute = AttrInt::new_strikethrough(true);
                    attribute_list.insert(attribute);
                }
                Some(attribute_list.to_value())
            })
            .build();
        // Save binding
        bindings.push(content_label_binding);
    }
```

</details><br>

`TaskRow::unbind` takes care of the cleanup. It iterates through the vector and unbinds each binding. In the end, it clears the vector.

`Filename`: 1/task_row/mod.rs
<details>
<summary>COde</summary>

```rust
    pub fn unbind(&self) {
        // Unbind all stored bindings
        for binding in self.imp().bindings.borrow_mut().drain(..) {
            binding.unbind();
        }
    }
```

</details><br>

That was it, we created a basic To-Do app! We will extend it with additional functionality