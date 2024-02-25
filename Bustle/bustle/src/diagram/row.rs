use gtk::{glib, prelude::*, subclass::prelude::*};

use crate::{message::Message, timestamp::Timestamp};

mod imp {
    use std::cell::RefCell;

    use super::*;
    use crate::diagram::row_tag::RowTag;

    #[derive(Debug, Default, glib::Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::Row)]
    #[template(resource = "/org/freedesktop/Bustle/ui/diagram_row.ui")]
    pub struct Row {
        #[property(get, set = Self::set_message, explicit_notify, nullable)]
        pub(super) message: RefCell<Option<Message>>,
        #[property(get, set = Self::set_head_message, explicit_notify, nullable)]
        pub(super) head_message: RefCell<Option<Message>>,

        #[template_child]
        pub(super) timestamp_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) tag: TemplateChild<RowTag>,
        #[template_child]
        pub(super) vbox: TemplateChild<gtk::Box>, // Unused, but needed for disposal
        #[template_child]
        pub(super) title_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) subtitle_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Row {
        const NAME: &'static str = "BustleDiagramRow";
        type Type = super::Row;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for Row {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for Row {}

    impl Row {
        fn set_message(&self, message: Option<Message>) {
            if message == self.message.replace(message.clone()) {
                return;
            }

            let obj = self.obj();
            let message_tag = message.map(|m| m.message_tag()).unwrap_or_default();
            self.tag.set_message_tag(message_tag);
            obj.update_timestamp_label();
            obj.update_title_subtitle_labels();
            obj.notify_message();
        }

        fn set_head_message(&self, head_message: Option<Message>) {
            if head_message == self.head_message.replace(head_message.clone()) {
                return;
            }

            let obj = self.obj();
            obj.update_timestamp_label();
            obj.notify_head_message();
        }
    }
}

glib::wrapper! {
     pub struct Row(ObjectSubclass<imp::Row>)
        @extends gtk::Widget;
}

impl Row {
    fn update_timestamp_label(&self) {
        let imp = self.imp();

        if let Some(message) = self.message() {
            let elapsed_ms = {
                let head_message = self
                    .head_message()
                    .expect("there must always be a head message if there is a message");
                if head_message == message {
                    Timestamp::default()
                } else {
                    message.timestamp() - head_message.timestamp()
                }
            };
            imp.timestamp_label
                .set_label(&format!("{} ms", elapsed_ms.as_millis_f64().round()));
        } else {
            imp.timestamp_label.set_label("");
        }
    }

    fn update_title_subtitle_labels(&self) {
        let imp = self.imp();

        if let Some(message) = self.message() {
            let title = message.path_display();
            if message.message_type().is_method_return() {
                // We need to use `set_markup` even we don't use any markup because it somehow
                // fix issues where random part of the label is bold. Possibly,
                // a `GtkInscription` issue?
                imp.title_label.set_markup(&title);
                imp.subtitle_label
                    .set_markup(&format!("<i>{}</i>", &message.member_markup(true)));
            } else {
                imp.title_label
                    .set_markup(&format!("<b>{}</b>", glib::markup_escape_text(&title)));
                imp.subtitle_label.set_markup(&message.member_markup(true));
            }
        } else {
            imp.title_label.set_text("");
            imp.subtitle_label.set_text("");
        }
    }
}

impl Default for Row {
    fn default() -> Self {
        glib::Object::new()
    }
}
