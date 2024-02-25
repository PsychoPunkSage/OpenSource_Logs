use gtk::{
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};

use crate::message_tag::MessageTag;

mod imp {
    use std::{cell::OnceCell, marker::PhantomData};

    use super::*;
    use crate::color_widget::ColorWidget;

    #[derive(Default, glib::Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::MessageTagRow)]
    #[template(resource = "/org/freedesktop/Bustle/ui/filter_pane_message_tag_row.ui")]
    pub struct MessageTagRow {
        #[property(get, set, construct_only, builder(MessageTag::default()))]
        pub(super) message_tag: OnceCell<MessageTag>,
        #[property(get = Self::is_active)]
        pub(super) is_active: PhantomData<bool>,

        #[template_child]
        pub(super) color: TemplateChild<ColorWidget>,
        #[template_child]
        pub(super) title: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) check_button: TemplateChild<gtk::CheckButton>,

        pub(super) check_button_active_notify_handler_id: OnceCell<glib::SignalHandlerId>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageTagRow {
        const NAME: &'static str = "BustleFilterPaneMessageTagRow";
        type Type = super::MessageTagRow;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MessageTagRow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            let message_tag = obj.message_tag();
            self.title.set_label(&message_tag.name());
            self.color.set_rgba(message_tag.color());

            let handler_id =
                self.check_button
                    .connect_active_notify(clone!(@weak obj => move |_| {
                        obj.notify_is_active();
                    }));
            self.check_button_active_notify_handler_id
                .set(handler_id)
                .unwrap();
        }
    }

    impl WidgetImpl for MessageTagRow {}
    impl ListBoxRowImpl for MessageTagRow {}

    impl MessageTagRow {
        fn is_active(&self) -> bool {
            self.check_button.is_active()
        }
    }
}

glib::wrapper! {
    pub struct MessageTagRow(ObjectSubclass<imp::MessageTagRow>)
        @extends gtk::Widget, gtk::ListBoxRow;
}

impl MessageTagRow {
    pub fn new(bus_name_item: &MessageTag) -> Self {
        glib::Object::builder()
            .property("message-tag", bus_name_item)
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
