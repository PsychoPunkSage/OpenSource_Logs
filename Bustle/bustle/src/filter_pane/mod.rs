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

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();

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

            self.message_tag_list_box.bind_model(
                Some(&adw::EnumListModel::new(MessageTag::static_type())),
                clone!(@weak obj => @default-panic, move |item| {
                    let enum_list_item = item.downcast_ref::<adw::EnumListItem>().unwrap();
                    let message_tag = unsafe { MessageTag::from_glib(enum_list_item.value()) };
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

            model.connect_has_filter_notify(clone!(@weak obj => move |model| {
                obj.action_set_enabled("filter-pane.reset-all", model.has_filter());
            }));

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
    fn create_message_tag_row(&self, message_tag: MessageTag) -> MessageTagRow {
        let row = MessageTagRow::new(&message_tag);
        row.connect_is_active_notify(clone!(@weak self as obj => move |row| {
            let model = obj.model();
            let message_tag = row.message_tag();
            if row.is_active() {
                let was_removed = model.remove_message_tag_filter(message_tag);
                debug_assert!(was_removed);
            } else {
                model.add_message_tag_filter(message_tag);
            }
        }));
        row
    }

    fn create_bus_name_row(&self, bus_name_item: &BusNameItem) -> BusNameRow {
        let row = BusNameRow::new(bus_name_item);
        row.connect_is_active_notify(clone!(@weak self as obj => move |row| {
            let model = obj.model();
            let bus_name_item = row.bus_name_item();
            let name = bus_name_item.name();
            if row.is_active() {
                let was_removed = model.remove_bus_name_filter(name);
                debug_assert!(was_removed);
            } else {
                model.add_bus_name_filter(name).unwrap();
            }
        }));
        row
    }

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