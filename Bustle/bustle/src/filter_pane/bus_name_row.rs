use gtk::{
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};

use crate::bus_name_item::BusNameItem;

mod imp {
    use std::{cell::OnceCell, marker::PhantomData};

    use crate::bus_name_item::LookupPoint;

    use super::*;

    #[derive(Default, glib::Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::BusNameRow)]
    #[template(resource = "/org/freedesktop/Bustle/ui/filter_pane_bus_name_row.ui")]
    pub struct BusNameRow {
        #[property(get, set, construct_only)]
        pub(super) bus_name_item: OnceCell<BusNameItem>,
        #[property(get = Self::is_active)]
        pub(super) is_active: PhantomData<bool>,

        #[template_child]
        pub(super) title: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) subtitle: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) check_button: TemplateChild<gtk::CheckButton>,

        pub(super) check_button_active_notify_handler_id: OnceCell<glib::SignalHandlerId>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BusNameRow {
        const NAME: &'static str = "BustleFilterPaneBusNameRow";
        type Type = super::BusNameRow;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for BusNameRow {
        fn constructed(&self) {
            // ensures that the parent object's construction process is completed before continuing
            self.parent_constructed();

            let obj = self.obj();
            let bus_name_item = obj.bus_name_item();

            self.title.set_label(bus_name_item.name()); // sets the label of the title widget of the BusNameRow to the name of the associated BusNameItem.
            let subtitle_text = bus_name_item
                .wk_names(LookupPoint::Last)
                .iter()
                .filter_map(|n| n.split('.').last()) // filtering out the last part of each name
                .collect::<Vec<_>>()
                .join(", ");
            self.subtitle.set_label(&subtitle_text); // sets the label of the subtitle widget of the BusNameRow to the constructed subtitle text.
            self.subtitle.set_visible(!subtitle_text.is_empty()); // sets the visibility of the subtitle widget based on whether the subtitle text is empty or not.

            let handler_id = self.check_button.connect_active_notify(
                clone!(@weak obj => move |_| { // `connect_active_notify` is used to connect a callback function to a signal, specifically the active property change signal of the check_button.
                    obj.notify_is_active();
                }),
            );
            self.check_button_active_notify_handler_id
                .set(handler_id)
                .unwrap();
        }
    }

    impl WidgetImpl for BusNameRow {}
    impl ListBoxRowImpl for BusNameRow {}

    impl BusNameRow {
        fn is_active(&self) -> bool {
            self.check_button.is_active()
        }
    }
}

glib::wrapper! {
    pub struct BusNameRow(ObjectSubclass<imp::BusNameRow>)
        @extends gtk::Widget, gtk::ListBoxRow;
}

impl BusNameRow {
    pub fn new(bus_name_item: &BusNameItem) -> Self {
        glib::Object::builder()
            .property("bus-name-item", bus_name_item)
            .build()
    }

    pub fn handle_activation(&self) {
        let was_activated = self.imp().check_button.activate();
        debug_assert!(was_activated);
    }

    pub fn reset(&self) {
        let imp = self.imp();
        let handler_id = imp.check_button_active_notify_handler_id.get().unwrap();
        imp.check_button.block_signal(handler_id);
        imp.check_button.set_active(true);
        imp.check_button.unblock_signal(handler_id);
    }
}
