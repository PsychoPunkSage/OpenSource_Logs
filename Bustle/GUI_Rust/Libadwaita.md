# **Libadwaita**

>>> Libadwaita is a library augmenting GTK 4 which:
>>>* provides widgets to better follow GNOME's HIG
>>>* provides widgets to let applications change their layout based on the available space
>>>* integrates the Adwaita stylesheet
>>>* allows runtime recoloring with named colors
>>>* adds API to support the cross-desktop dark style preference

>> **Usage::>**
```bash
cargo add libadwaita --rename adw --features v1_4
```

The versions of the `gtk4` and `libadwaita` crates need to be synced. Just remember that when we update one of them to the newest version to update the other one as well.

## **Let To-Do App use Libadwaita**


>> We will adapt our To-Do app so that it follows GNOME's HIG<br>The simplest way to take advantage of Libadwaita is by replacing `gtk::Application` with `adw::Application`.

`Filesystem`: ...../todo/5/main.rs

```rust
// mod task_object;
// mod task_row;
// mod utils;
// mod window;

// use gtk::prelude::*;
// use gtk::{gio, glib};
// use window::Window;

// const APP_ID: &str = "org.gtk_rs.Todo5";

fn main() -> glib::ExitCode {
    gio::resources_register_include!("todo_5.gresource")
        .expect("Failed to register resources.");

    // Create a new application
    //        👇 changed
    let app = adw::Application::builder().application_id(APP_ID).build();

    // Connect to signals
    app.connect_startup(setup_shortcuts);
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

//                       👇 changed
fn setup_shortcuts(app: &adw::Application) {
    app.set_accels_for_action("win.filter('All')", &["<Ctrl>a"]);
    app.set_accels_for_action("win.filter('Open')", &["<Ctrl>o"]);
    app.set_accels_for_action("win.filter('Done')", &["<Ctrl>d"]);
}

//                👇 changed
fn build_ui(app: &adw::Application) {
    // Create a new custom window and present it
    let window = Window::new(app);
    window.present();
}
```

`Filesystem`: ...../todo/5/window/mod.rs

```rust
    pub fn new(app: &adw::Application) -> Self {
        // Create new window
        Object::builder().property("application", app).build()
    }
```
<details>
<summary>Full Code</summary>

```rust
mod imp;

use std::fs::File;

use gio::Settings;
use glib::{clone, Object};
use gtk::subclass::prelude::*;
use gtk::{
    gio, glib, CustomFilter, FilterListModel, NoSelection, SignalListItemFactory,
};
use gtk::{prelude::*, ListItem};

use crate::task_object::{TaskData, TaskObject};
use crate::task_row::TaskRow;
use crate::utils::data_path;
use crate::APP_ID;

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &adw::Application) -> Self {
        // Create new window
        Object::builder().property("application", app).build()
    }

    fn setup_settings(&self) {
        let settings = Settings::new(APP_ID);
        self.imp()
            .settings
            .set(settings)
            .expect("`settings` should not be set before calling `setup_settings`.");
    }

    fn settings(&self) -> &Settings {
        self.imp()
            .settings
            .get()
            .expect("`settings` should be set in `setup_settings`.")
    }

    fn tasks(&self) -> gio::ListStore {
        self.imp()
            .tasks
            .borrow()
            .clone()
            .expect("Could not get current tasks.")
    }

    fn filter(&self) -> Option<CustomFilter> {
        // Get filter state from settings
        let filter_state: String = self.settings().get("filter");

        // Create custom filters
        let filter_open = CustomFilter::new(|obj| {
            // Get `TaskObject` from `glib::Object`
            let task_object = obj
                .downcast_ref::<TaskObject>()
                .expect("The object needs to be of type `TaskObject`.");

            // Only allow completed tasks
            !task_object.is_completed()
        });
        let filter_done = CustomFilter::new(|obj| {
            // Get `TaskObject` from `glib::Object`
            let task_object = obj
                .downcast_ref::<TaskObject>()
                .expect("The object needs to be of type `TaskObject`.");

            // Only allow done tasks
            task_object.is_completed()
        });

        // Return the correct filter
        match filter_state.as_str() {
            "All" => None,
            "Open" => Some(filter_open),
            "Done" => Some(filter_done),
            _ => unreachable!(),
        }
    }

    fn setup_tasks(&self) {
        // Create new model
        let model = gio::ListStore::new::<TaskObject>();

        // Get state and set model
        self.imp().tasks.replace(Some(model));

        // Wrap model with filter and selection and pass it to the list view
        let filter_model = FilterListModel::new(Some(self.tasks()), self.filter());
        let selection_model = NoSelection::new(Some(filter_model.clone()));
        self.imp().tasks_list.set_model(Some(&selection_model));

        // Filter model whenever the value of the key "filter" changes
        self.settings().connect_changed(
            Some("filter"),
            clone!(@weak self as window, @weak filter_model => move |_, _| {
                filter_model.set_filter(window.filter().as_ref());
            }),
        );
    }

    fn restore_data(&self) {
        if let Ok(file) = File::open(data_path()) {
            // Deserialize data from file to vector
            let backup_data: Vec<TaskData> = serde_json::from_reader(file).expect(
                "It should be possible to read `backup_data` from the json file.",
            );

            // Convert `Vec<TaskData>` to `Vec<TaskObject>`
            let task_objects: Vec<TaskObject> = backup_data
                .into_iter()
                .map(TaskObject::from_task_data)
                .collect();

            // Insert restored objects into model
            self.tasks().extend_from_slice(&task_objects);
        }
    }

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

    fn setup_actions(&self) {
        // Create action from key "filter" and add to action group "win"
        let action_filter = self.settings().create_action("filter");
        self.add_action(&action_filter);
    }

    fn remove_done_tasks(&self) {
        let tasks = self.tasks();
        let mut position = 0;
        while let Some(item) = tasks.item(position) {
            // Get `TaskObject` from `glib::Object`
            let task_object = item
                .downcast_ref::<TaskObject>()
                .expect("The object needs to be of type `TaskObject`.");

            if task_object.is_completed() {
                tasks.remove(position);
            } else {
                position += 1;
            }
        }
    }
}
```

</details>

* **`adw::Application`** calls `adw::init` internally and makes sure that translations, types, stylesheets, and icons are set up properly for Libadwaita. It also loads stylesheets automatically from resources as long as they are named correctly.


### **Boxed List**

>> Of course Libadwaita is more than just a couple of stylesheets and a `StyleManager`. But before we get to the interesting stuff, we will make our lives easier for the future by replacing all occurrences of `gtk::prelude` and `gtk::subclass::prelude` with `adw::prelude` and `adw::subclass::prelude`. This works because the `adw` preludes, in addition to the Libadwaita-specific traits, re-export the corresponding `gtk` preludes.

We can use boxed lists by using `gtk::ListBox` instead of `gtk::ListView`. We will also add the boxed-list style class provided by Libadwaita.

Let's implement all these changes in the **window.ui** file. All of the changes are confined within the second child of the **ApplicationWindow**. To see the complete file, just click on the link after "Filename".

`Filesystem`: ...../todo/6/resources/window.ui

```html
<child>
  <object class="GtkScrolledWindow">
    <property name="hscrollbar-policy">never</property>
    <property name="min-content-height">420</property>
    <property name="vexpand">True</property>
    <property name="child">
      <object class="AdwClamp">
        <property name="child">
          <object class="GtkBox">
            <property name="orientation">vertical</property>
            <property name="spacing">18</property>
            <property name="margin-top">24</property>
            <property name="margin-bottom">24</property>
            <property name="margin-start">12</property>
            <property name="margin-end">12</property>
            <child>
              <object class="GtkEntry" id="entry">
                <property name="placeholder-text" translatable="yes">Enter a Task…</property>
                <property name="secondary-icon-name">list-add-symbolic</property>
              </object>
            </child>
            <child>
              <object class="GtkListBox" id="tasks_list">
                <property name="visible">False</property>
                <property name="selection-mode">none</property>
                <style>
                  <class name="boxed-list" />
                </style>
              </object>
            </child>
          </object>
        </property>
      </object>
    </property>
  </object>
</child>
```

In order to follow the boxed list pattern, we switched to **gtk::ListBox**, set its property "selection-mode" to "none" and added the boxed-list style class.

Let's continue with **window/imp.rs**. The member variable `tasks_list` now describes a `ListBox` rather than a `ListView`.

`Filesystem`: ...../todo/6/window/imp.rs

```rust
// Object holding the state
#[derive(CompositeTemplate, Default)]
#[template(resource = "/org/gtk_rs/Todo6/window.ui")]
pub struct Window {
    #[template_child]
    pub entry: TemplateChild<Entry>,
    #[template_child]
    pub tasks_list: TemplateChild<ListBox>,
    pub tasks: RefCell<Option<gio::ListStore>>,
    pub settings: OnceCell<Settings>,
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

        // Create action to remove done tasks and add to action group "win"
        klass.install_action("win.remove-done-tasks", None, |window, _, _| {
            window.remove_done_tasks();
        });
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}
```

<details>
<summary>Full Code</summary>

```rust
use std::cell::RefCell;
use std::fs::File;

use adw::subclass::prelude::*;

use gio::Settings;
use glib::subclass::InitializingObject;

use adw::prelude::*;
use gtk::{gio, glib, CompositeTemplate, Entry, ListBox};
use std::cell::OnceCell;

use crate::task_object::{TaskData, TaskObject};
use crate::utils::data_path;

// Object holding the state
#[derive(CompositeTemplate, Default)]
#[template(resource = "/org/gtk_rs/Todo6/window.ui")]
pub struct Window {
    #[template_child]
    pub entry: TemplateChild<Entry>,
    #[template_child]
    pub tasks_list: TemplateChild<ListBox>,
    pub tasks: RefCell<Option<gio::ListStore>>,
    pub settings: OnceCell<Settings>,
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

        // Create action to remove done tasks and add to action group "win"
        klass.install_action("win.remove-done-tasks", None, |window, _, _| {
            window.remove_done_tasks();
        });
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

// Trait shared by all GObjects
impl ObjectImpl for Window {
    fn constructed(&self) {
        // Call "constructed" on parent
        self.parent_constructed();

        // Setup
        let obj = self.obj();
        obj.setup_settings();
        obj.setup_tasks();
        obj.restore_data();
        obj.setup_callbacks();
        obj.setup_actions();
    }
}

// Trait shared by all widgets
impl WidgetImpl for Window {}

// Trait shared by all windows
impl WindowImpl for Window {
    fn close_request(&self) -> glib::Propagation {
        // Store task data in vector
        let backup_data: Vec<TaskData> = self
            .obj()
            .tasks()
            .iter::<TaskObject>()
            .filter_map(Result::ok)
            .map(|task_object| task_object.task_data())
            .collect();

        // Save state to file
        let file = File::create(data_path()).expect("Could not create json file.");
        serde_json::to_writer(file, &backup_data)
            .expect("Could not write data to json file");

        // Pass close request on to the parent
        self.parent_close_request()
    }
}

// Trait shared by all application windows
impl ApplicationWindowImpl for Window {}
```

</details>

We now move on to **window/mod.rs**. **ListBox** supports models just fine, but without any widget recycling we don't need factories anymore. setup_factory can therefore be safely deleted. To setup the **ListBox**, we call **bind_model** in **setup_tasks**. There we specify the model, as well as a closure describing how to transform the given GObject into a widget the list box can display.

`Filesystem`: ...../todo/6/window/mod.rs

```rust
        // Wrap model with filter and selection and pass it to the list box
        let filter_model = FilterListModel::new(Some(self.tasks()), self.filter());
        let selection_model = NoSelection::new(Some(filter_model.clone()));
        self.imp().tasks_list.bind_model(
            Some(&selection_model),
            clone!(@weak self as window => @default-panic, move |obj| {
                let task_object = obj.downcast_ref().expect("The object should be of type `TaskObject`.");
                let row = window.create_task_row(task_object);
                row.upcast()
            }),
        );
```

<details>
<summary>Full Code</summary>

```rust
mod imp;

use std::fs::File;

use adw::subclass::prelude::*;
use adw::{prelude::*, ActionRow};
use gio::Settings;
use glib::{clone, Object};
use gtk::{gio, glib, Align, CheckButton, CustomFilter, FilterListModel, NoSelection};

use crate::task_object::{TaskData, TaskObject};
use crate::utils::data_path;
use crate::APP_ID;

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &adw::Application) -> Self {
        // Create new window
        Object::builder().property("application", app).build()
    }

    fn setup_settings(&self) {
        let settings = Settings::new(APP_ID);
        self.imp()
            .settings
            .set(settings)
            .expect("`settings` should not be set before calling `setup_settings`.");
    }

    fn settings(&self) -> &Settings {
        self.imp()
            .settings
            .get()
            .expect("`settings` should be set in `setup_settings`.")
    }

    fn tasks(&self) -> gio::ListStore {
        self.imp()
            .tasks
            .borrow()
            .clone()
            .expect("Could not get current tasks.")
    }

    fn filter(&self) -> Option<CustomFilter> {
        // Get filter state from settings
        let filter_state: String = self.settings().get("filter");

        // Create custom filters
        let filter_open = CustomFilter::new(|obj| {
            // Get `TaskObject` from `glib::Object`
            let task_object = obj
                .downcast_ref::<TaskObject>()
                .expect("The object needs to be of type `TaskObject`.");

            // Only allow completed tasks
            !task_object.is_completed()
        });
        let filter_done = CustomFilter::new(|obj| {
            // Get `TaskObject` from `glib::Object`
            let task_object = obj
                .downcast_ref::<TaskObject>()
                .expect("The object needs to be of type `TaskObject`.");

            // Only allow done tasks
            task_object.is_completed()
        });

        // Return the correct filter
        match filter_state.as_str() {
            "All" => None,
            "Open" => Some(filter_open),
            "Done" => Some(filter_done),
            _ => unreachable!(),
        }
    }

    fn setup_tasks(&self) {
        // Create new model
        let model = gio::ListStore::new::<TaskObject>();

        // Get state and set model
        self.imp().tasks.replace(Some(model));

        // Wrap model with filter and selection and pass it to the list box
        let filter_model = FilterListModel::new(Some(self.tasks()), self.filter());
        let selection_model = NoSelection::new(Some(filter_model.clone()));
        self.imp().tasks_list.bind_model(
            Some(&selection_model),
            clone!(@weak self as window => @default-panic, move |obj| {
                let task_object = obj.downcast_ref().expect("The object should be of type `TaskObject`.");
                let row = window.create_task_row(task_object);
                row.upcast()
            }),
        );

        // Filter model whenever the value of the key "filter" changes
        self.settings().connect_changed(
            Some("filter"),
            clone!(@weak self as window, @weak filter_model => move |_, _| {
                filter_model.set_filter(window.filter().as_ref());
            }),
        );

        // Assure that the task list is only visible when it is supposed to
        self.set_task_list_visible(&self.tasks());
        self.tasks().connect_items_changed(
            clone!(@weak self as window => move |tasks, _, _, _| {
                window.set_task_list_visible(tasks);
            }),
        );
    }

    /// Assure that `tasks_list` is only visible
    /// if the number of tasks is greater than 0
    fn set_task_list_visible(&self, tasks: &gio::ListStore) {
        self.imp().tasks_list.set_visible(tasks.n_items() > 0);
    }

    fn restore_data(&self) {
        if let Ok(file) = File::open(data_path()) {
            // Deserialize data from file to vector
            let backup_data: Vec<TaskData> = serde_json::from_reader(file).expect(
                "It should be possible to read `backup_data` from the json file.",
            );

            // Convert `Vec<TaskData>` to `Vec<TaskObject>`
            let task_objects: Vec<TaskObject> = backup_data
                .into_iter()
                .map(TaskObject::from_task_data)
                .collect();

            // Insert restored objects into model
            self.tasks().extend_from_slice(&task_objects);
        }
    }

    fn create_task_row(&self, task_object: &TaskObject) -> ActionRow {
        // Create check button
        let check_button = CheckButton::builder()
            .valign(Align::Center)
            .can_focus(false)
            .build();

        // Create row
        let row = ActionRow::builder()
            .activatable_widget(&check_button)
            .build();
        row.add_prefix(&check_button);

        // Bind properties
        task_object
            .bind_property("completed", &check_button, "active")
            .bidirectional()
            .sync_create()
            .build();
        task_object
            .bind_property("content", &row, "title")
            .sync_create()
            .build();

        // Return row
        row
    }

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

    fn setup_actions(&self) {
        // Create action from key "filter" and add to action group "win"
        let action_filter = self.settings().create_action("filter");
        self.add_action(&action_filter);
    }

    fn remove_done_tasks(&self) {
        let tasks = self.tasks();
        let mut position = 0;
        while let Some(item) = tasks.item(position) {
            // Get `TaskObject` from `glib::Object`
            let task_object = item
                .downcast_ref::<TaskObject>()
                .expect("The object needs to be of type `TaskObject`.");

            if task_object.is_completed() {
                tasks.remove(position);
            } else {
                position += 1;
            }
        }
    }
}
```

</details>

We still have to specify the `create_task_row` method. Here, we create an `adw::ActionRow` with a `gtk::CheckButton` as activatable widget. Without recycling, a GObject will always belong to the same widget. That means we can just bind their properties without having to worry about unbinding them later on.

`Filesystem`: ...../todo/6/window/mod.rs

```rust
    fn create_task_row(&self, task_object: &TaskObject) -> ActionRow {
        // Create check button
        let check_button = CheckButton::builder()
            .valign(Align::Center)
            .can_focus(false)
            .build();

        // Create row
        let row = ActionRow::builder()
            .activatable_widget(&check_button)
            .build();
        row.add_prefix(&check_button);

        // Bind properties
        task_object
            .bind_property("completed", &check_button, "active")
            .bidirectional()
            .sync_create()
            .build();
        task_object
            .bind_property("content", &row, "title")
            .sync_create()
            .build();

        // Return row
        row
    }
```

<details>
<summary>Full Code</summary>

```rust
mod imp;

use std::fs::File;

use adw::subclass::prelude::*;
use adw::{prelude::*, ActionRow};
use gio::Settings;
use glib::{clone, Object};
use gtk::{gio, glib, Align, CheckButton, CustomFilter, FilterListModel, NoSelection};

use crate::task_object::{TaskData, TaskObject};
use crate::utils::data_path;
use crate::APP_ID;

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &adw::Application) -> Self {
        // Create new window
        Object::builder().property("application", app).build()
    }

    fn setup_settings(&self) {
        let settings = Settings::new(APP_ID);
        self.imp()
            .settings
            .set(settings)
            .expect("`settings` should not be set before calling `setup_settings`.");
    }

    fn settings(&self) -> &Settings {
        self.imp()
            .settings
            .get()
            .expect("`settings` should be set in `setup_settings`.")
    }

    fn tasks(&self) -> gio::ListStore {
        self.imp()
            .tasks
            .borrow()
            .clone()
            .expect("Could not get current tasks.")
    }

    fn filter(&self) -> Option<CustomFilter> {
        // Get filter state from settings
        let filter_state: String = self.settings().get("filter");

        // Create custom filters
        let filter_open = CustomFilter::new(|obj| {
            // Get `TaskObject` from `glib::Object`
            let task_object = obj
                .downcast_ref::<TaskObject>()
                .expect("The object needs to be of type `TaskObject`.");

            // Only allow completed tasks
            !task_object.is_completed()
        });
        let filter_done = CustomFilter::new(|obj| {
            // Get `TaskObject` from `glib::Object`
            let task_object = obj
                .downcast_ref::<TaskObject>()
                .expect("The object needs to be of type `TaskObject`.");

            // Only allow done tasks
            task_object.is_completed()
        });

        // Return the correct filter
        match filter_state.as_str() {
            "All" => None,
            "Open" => Some(filter_open),
            "Done" => Some(filter_done),
            _ => unreachable!(),
        }
    }

    fn setup_tasks(&self) {
        // Create new model
        let model = gio::ListStore::new::<TaskObject>();

        // Get state and set model
        self.imp().tasks.replace(Some(model));

        // Wrap model with filter and selection and pass it to the list box
        let filter_model = FilterListModel::new(Some(self.tasks()), self.filter());
        let selection_model = NoSelection::new(Some(filter_model.clone()));
        self.imp().tasks_list.bind_model(
            Some(&selection_model),
            clone!(@weak self as window => @default-panic, move |obj| {
                let task_object = obj.downcast_ref().expect("The object should be of type `TaskObject`.");
                let row = window.create_task_row(task_object);
                row.upcast()
            }),
        );

        // Filter model whenever the value of the key "filter" changes
        self.settings().connect_changed(
            Some("filter"),
            clone!(@weak self as window, @weak filter_model => move |_, _| {
                filter_model.set_filter(window.filter().as_ref());
            }),
        );

        // Assure that the task list is only visible when it is supposed to
        self.set_task_list_visible(&self.tasks());
        self.tasks().connect_items_changed(
            clone!(@weak self as window => move |tasks, _, _, _| {
                window.set_task_list_visible(tasks);
            }),
        );
    }

    /// Assure that `tasks_list` is only visible
    /// if the number of tasks is greater than 0
    fn set_task_list_visible(&self, tasks: &gio::ListStore) {
        self.imp().tasks_list.set_visible(tasks.n_items() > 0);
    }

    fn restore_data(&self) {
        if let Ok(file) = File::open(data_path()) {
            // Deserialize data from file to vector
            let backup_data: Vec<TaskData> = serde_json::from_reader(file).expect(
                "It should be possible to read `backup_data` from the json file.",
            );

            // Convert `Vec<TaskData>` to `Vec<TaskObject>`
            let task_objects: Vec<TaskObject> = backup_data
                .into_iter()
                .map(TaskObject::from_task_data)
                .collect();

            // Insert restored objects into model
            self.tasks().extend_from_slice(&task_objects);
        }
    }

    fn create_task_row(&self, task_object: &TaskObject) -> ActionRow {
        // Create check button
        let check_button = CheckButton::builder()
            .valign(Align::Center)
            .can_focus(false)
            .build();

        // Create row
        let row = ActionRow::builder()
            .activatable_widget(&check_button)
            .build();
        row.add_prefix(&check_button);

        // Bind properties
        task_object
            .bind_property("completed", &check_button, "active")
            .bidirectional()
            .sync_create()
            .build();
        task_object
            .bind_property("content", &row, "title")
            .sync_create()
            .build();

        // Return row
        row
    }

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

    fn setup_actions(&self) {
        // Create action from key "filter" and add to action group "win"
        let action_filter = self.settings().create_action("filter");
        self.add_action(&action_filter);
    }

    fn remove_done_tasks(&self) {
        let tasks = self.tasks();
        let mut position = 0;
        while let Some(item) = tasks.item(position) {
            // Get `TaskObject` from `glib::Object`
            let task_object = item
                .downcast_ref::<TaskObject>()
                .expect("The object needs to be of type `TaskObject`.");

            if task_object.is_completed() {
                tasks.remove(position);
            } else {
                position += 1;
            }
        }
    }
}
```

</details>

When using boxed lists, we also have to take care to hide the `ListBox` when there is no task present.

`Filesystem`: ...../todo/6/window/mod.rs

```rust
        // Assure that the task list is only visible when it is supposed to
        self.set_task_list_visible(&self.tasks());
        self.tasks().connect_items_changed(
            clone!(@weak self as window => move |tasks, _, _, _| {
                window.set_task_list_visible(tasks);
            }),
        );
```

Finally, we define the `set_task_list_visible` method.

`Filesystem`: ...../todo/6/window/mod.rs

```rust
    /// Assure that `tasks_list` is only visible
    /// if the number of tasks is greater than 0
    fn set_task_list_visible(&self, tasks: &gio::ListStore) {
        self.imp().tasks_list.set_visible(tasks.n_items() > 0);
    }

```

## **Adding Collections**

### **Sidebar**

>> These collections will get their own sidebar on the left of the app. We will start by adding an empty sidebar without any functionality.

There are a couple of steps we have to go through to get to this state. First, we have to replace `gtk::ApplicationWindow` with `adw::ApplicationWindow`. The main difference between those two is that `adw::ApplicationWindow` has no **titlebar area**. That comes in handy when we build up our interface with `adw::NavigationSplitView`. The `NavigationSplitView` adds a sidebar for the collection view to the left, while the task view occupies the space on the right. When using `adw::ApplicationWindow` the collection view and task view have their own `adw::HeaderBar` and the separator spans over the whole window.

`Filesystem`: ...../todo/7/resources/window.ui

```html
<?xml version="1.0" encoding="UTF-8"?>
<interface>
  <menu id="main-menu">
    <!--Menu implementation-->      
  </menu>
  <template class="TodoWindow" parent="AdwApplicationWindow">
    <property name="title" translatable="yes">To-Do</property>
    <property name="width-request">360</property>
    <property name="height-request">200</property>
    <child>
      <object class="AdwBreakpoint">
        <condition>max-width: 500sp</condition>
        <setter object="split_view" property="collapsed">True</setter>
      </object>
    </child>
    <property name="content">
      <object class="AdwNavigationSplitView" id="split_view">
        <property name="min-sidebar-width">200</property>
        <property name="sidebar">
          <object class="AdwNavigationPage">
            <!--Collection view implementation-->
          </object>
        </property>
        <property name="content">
          <object class="AdwNavigationPage">
            <!--Task view implementation-->
          </object>
        </property>
      </object>
    </property>
  </template>
</interface>
```

`NavigationSplitView` also helps with making your app **adaptive** As soon as the requested size is too small to fit all children at the same time, the splitview collapses, and starts behaving like a `gtk::Stack`. This means that it only displays one of its children at a time. The adaptive behavior of the leaflet allows the To-Do app to work on smaller screen sizes (like e.g. phones) even with the added collection view.

We add the necessary UI elements for the collection view, such as a header bar with a button to add a new collection, as well as the list box `collections_list` to display the collections later on. We also add the style **navigations-sidebar** to `collections_list`.

`Filesystem`: ...../todo/7/resources/window.ui

```html
<object class="AdwNavigationPage">
  <property name="title" bind-source="TodoWindow"
    bind-property="title" bind-flags="sync-create" />
  <property name="child">
    <object class="AdwToolbarView">
      <child type="top">
        <object class="AdwHeaderBar">
          <child type="start">
            <object class="GtkToggleButton">
              <property name="icon-name">list-add-symbolic</property>
              <property name="tooltip-text" translatable="yes">New Collection</property>
              <property name="action-name">win.new-collection</property>
            </object>
          </child>
        </object>
      </child>
      <property name="content">
        <object class="GtkScrolledWindow">
          <property name="child">
            <object class="GtkListBox" id="collections_list">
              <style>
                <class name="navigation-sidebar" />
              </style>
            </object>
          </property>
        </object>
      </property>
    </object>
```

