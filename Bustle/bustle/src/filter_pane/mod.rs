mod bus_name_row;
mod message_tag_row;

use gtk::{
    glib::{self, clone, translate::FromGlib},
    prelude::*,
    subclass::prelude::*,
};

use crate::{
    bus_name_item::BusNameItem,
    filter_pane::{bus_name_row::BusNameRow, message_tag_row::MessageTagRow},
    filtered_message_model::FilteredMessageModel,
    message_tag::MessageTag,
};

mod imp {
    use std::cell::OnceCell;

    use super::*;

    #[derive(Default, glib::Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::FilterPane)]
    #[template(resource = "/org/freedesktop/Bustle/ui/filter_pane.ui")]
    pub struct FilterPane {
        #[property(get, set = Self::set_model)]
        pub(super) model: OnceCell<FilteredMessageModel>,

        #[template_child]
        pub(super) vbox: TemplateChild<gtk::Box>, // Unused, but needed for disposal
        #[template_child]
        pub(super) message_tag_list_box: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub(super) bus_name_list_box: TemplateChild<gtk::ListBox>,
        #[template_child]
        child: TemplateChild<gtk::Widget>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FilterPane {
        const NAME: &'static str = "BustleFilterPane";
        type Type = super::FilterPane;
        type ParentType = gtk::Widget;

        // initialize the class structure of a GObject subclass
        fn class_init(klass: &mut Self::Class) {
            klass.bind_template(); // binds the class to its corresponding Glade template, allowing the class to instantiate itself from a Glade UI file
            klass.bind_template_instance_callbacks(); // binds callbacks defined in the Glade UI file to methods of the class instance.

            // filter-pane.reset-all ::> in `filter_pane.ui`
            klass.install_action("filter-pane.reset-all", None, |obj, _, _| {
                obj.model().clear_filters();
                obj.reset_rows();
            })
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for FilterPane {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            // `ListBox` is a widget commonly used in graphical user interfaces (GUIs) to display a list of items to the user
            self.message_tag_list_box.bind_model(
                // starts the process of binding a model to a list box named message_tag_list_box.
                // ``EnumListModel``::> type of model in GTK, used specifically for displaying enumerated (enum) types in list-based widgets like ListBox.
                Some(&adw::EnumListModel::new(MessageTag::static_type())), // `EnumListModel` is created with the type MessageTag::static_type(), and a reference to it is wrapped in ``Some``.
                // Specifies a closure that will be called to create each item in the list box.
                // The` @default-panic` tells it to panic if the weak reference is None.
                clone!(@weak obj => @default-panic, move |item| {
                    // attempts to downcast the item received in the closure to an EnumListItem
                    let enum_list_item = item.downcast_ref::<adw::EnumListItem>().unwrap();
                    // the value stored in the EnumListItem is converted into a MessageTag. This is marked as unsafe because it involves interacting with raw pointers.
                    let message_tag = unsafe { MessageTag::from_glib(enum_list_item.value()) };
                    // upcast() is used to convert it into a glib::Object.
                    obj.create_message_tag_row(message_tag).upcast()
                }),
            );
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for FilterPane {}

    impl FilterPane {
        fn set_model(&self, model: FilteredMessageModel) {
            let obj = self.obj();

            /*
            - Establishes a connection between the `has_filter_notify` signal of the `model` object and a closure.
            When the `has_filter_notify` signal is emitted, the closure is invoked. Inside the closure, the `action_set_enabled` method of the obj object is called,
            enabling or disabling the "filter-pane.reset-all" action based on whether the model has a filter.
            */
            model.connect_has_filter_notify(clone!(@weak obj => move |model| {
                obj.action_set_enabled("filter-pane.reset-all", model.has_filter());
            }));

            /*
            - connects the `bus_name_list_notify` signal of the `filtered_bus_names` object to a closure. When the signal is emitted, the closure is executed
            - Inside the closure, the `reset_rows` method of the obj object is called to reset the rows.
            - `bind_model` method of the `bus_name_list_box` of the imp object is called to bind the `filtered_bus_names` to the `bus_name_list_box`.
            */
            // @Checkpoint
            model.filtered_bus_names().connect_bus_name_list_notify(
                clone!(@weak obj => move |filtered_bus_names| {
                    obj.reset_rows();
                    obj.imp().bus_name_list_box.bind_model(
                        filtered_bus_names.bus_name_list().as_ref(),
                        clone!(@weak obj => @default-panic, move |item| {
                            let bus_name_item = item.downcast_ref().unwrap();
                            obj.create_bus_name_row(bus_name_item).upcast()
                        }),
                    )
                }),
            );

            self.model.set(model).unwrap();
        }
    }
}

glib::wrapper! {
    pub struct FilterPane(ObjectSubclass<imp::FilterPane>)
        @extends gtk::Widget;
}

impl FilterPane {
    // @Checkpoint.... ways to add the tag.
    fn create_message_tag_row(&self, message_tag: MessageTag) -> MessageTagRow {
        // creates a new MessageTagRow object
        let row = MessageTagRow::new(&message_tag);
        println!("create_message_tag_row >> row {:?}", row);
        // connects a signal handler to the is_active property of the MessageTagRow object
        row.connect_is_active_notify(clone!(@weak self as obj => move |row| {
            let model = obj.model();
            println!("create_message_tag_row >> model {:?}", model);
            let message_tag = row.message_tag();
            println!("create_message_tag_row >> message_tag {:?}", message_tag);
            if row.is_active() {
                // attempts to remove the message_tag filter from the model.
                let was_removed = model.remove_message_tag_filter(message_tag);
                debug_assert!(was_removed);
                println!("create_message_tag_row >> was_removed {:?}", was_removed);
            } else {
                model.add_message_tag_filter(message_tag);
            }
        }));
        row
    }

    // @Checkpoint.... ways to add the tag.
    fn create_bus_name_row(&self, bus_name_item: &BusNameItem) -> BusNameRow {
        let row = BusNameRow::new(bus_name_item);
        println!("create_bus_name_row >> row {:?}", row);
        row.connect_is_active_notify(clone!(@weak self as obj => move |row| {
            let model = obj.model();
            println!("create_bus_name_row >> model {:?}", model);
            let bus_name_item = row.bus_name_item();
            println!("create_bus_name_row >> bus_name_item {:?}", bus_name_item);
            let name = bus_name_item.name();
            println!("create_bus_name_row >> name {:?}", name);
            if row.is_active() {
                let was_removed = model.remove_bus_name_filter(name);
                debug_assert!(was_removed);
                println!("create_bus_name_row >> was_removed {:?}", was_removed);
            } else {
                model.add_bus_name_filter(name).unwrap();
            }
        }));
        row
    }

    // Resets the Checkboxes <NO CHANGES REQUIRED>
    fn reset_rows(&self) {
        let imp = self.imp();

        let mut curr = imp.message_tag_list_box.first_child();
        while let Some(child) = &curr {
            let message_tag_row = child.downcast_ref::<MessageTagRow>().unwrap();
            message_tag_row.reset();
            curr = child.next_sibling();
        }

        let mut curr = imp.bus_name_list_box.first_child();
        while let Some(child) = &curr {
            let bus_name_row = child.downcast_ref::<BusNameRow>().unwrap();
            bus_name_row.reset();
            curr = child.next_sibling();
        }
    }
}

#[gtk::template_callbacks]
impl FilterPane {
    #[template_callback]
    fn message_tag_list_box_row_activated(&self, row: &gtk::ListBoxRow) {
        let message_tag_row = row.downcast_ref::<MessageTagRow>().unwrap();
        message_tag_row.handle_activation();
    }

    #[template_callback]
    fn bus_name_list_box_row_activated(&self, row: &gtk::ListBoxRow) {
        let bus_name_row = row.downcast_ref::<BusNameRow>().unwrap();
        bus_name_row.handle_activation();
    }
}
