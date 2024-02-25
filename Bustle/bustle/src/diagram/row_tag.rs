use gtk::{glib, prelude::*, subclass::prelude::*};

use crate::message_tag::MessageTag;
mod imp {
    use std::cell::Cell;

    use super::*;
    use crate::color_widget::ColorWidget;

    #[derive(Debug, glib::Properties, Default, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::RowTag)]
    #[template(resource = "/org/freedesktop/Bustle/ui/diagram_row_tag.ui")]
    pub struct RowTag {
        #[property(get, set = Self::set_message_tag, explicit_notify, builder(MessageTag::default()))]
        pub(super) message_tag: Cell<MessageTag>,
        #[template_child]
        pub(super) label: TemplateChild<gtk::Inscription>,
        #[template_child]
        pub(super) color: TemplateChild<ColorWidget>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RowTag {
        const NAME: &'static str = "BustleDiagramRowTag";
        type Type = super::RowTag;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for RowTag {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().update_color_and_label();
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for RowTag {}

    impl RowTag {
        fn set_message_tag(&self, tag: MessageTag) {
            let obj = self.obj();

            if self.message_tag.get() == tag {
                return;
            }

            self.message_tag.set(tag);
            obj.update_color_and_label();
            obj.notify_message_tag();
        }
    }
}

glib::wrapper! {
    pub struct RowTag(ObjectSubclass<imp::RowTag>)
        @extends gtk::Widget;
}

impl RowTag {
    fn update_color_and_label(&self) {
        let imp = self.imp();

        let tag = self.message_tag();
        imp.color.set_rgba(tag.color());
        imp.label.set_text(Some(&tag.name()));
    }
}
