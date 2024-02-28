# **Building a Simple To-Do App**

## **Window**

> This mockup can be described by the following composite template.

Filename: 1/resources/window.ui
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

Filename: 1/window/mod.rs
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

Filename: 1/window/imp.rs
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