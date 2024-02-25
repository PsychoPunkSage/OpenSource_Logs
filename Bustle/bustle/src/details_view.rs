use adw::prelude::*;
use gtk::{
    glib::{self, closure_local},
    subclass::prelude::*,
};
use zbus::zvariant;

use crate::message::Message;

mod imp {
    use std::cell::RefCell;

    use glib::subclass::Signal;
    use once_cell::sync::Lazy;

    use super::*;
    use crate::color_widget::ColorWidget;

    #[derive(Debug, Default, glib::Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::DetailsView)]
    #[template(resource = "/org/freedesktop/Bustle/ui/details_view.ui")]
    pub struct DetailsView {
        #[property(get, set = Self::set_message, explicit_notify, nullable)]
        pub(super) message: RefCell<Option<Message>>,

        #[template_child]
        pub(super) scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub(super) associated_message_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub(super) type_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(super) sender_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(super) destination_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(super) component_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(super) component_color: TemplateChild<ColorWidget>,
        #[template_child]
        pub(super) path_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(super) member_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(super) interface_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(super) error_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(super) flags_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(super) size_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(super) signature_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(super) response_time_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub(super) arguments_text_buffer: TemplateChild<gtk::TextBuffer>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DetailsView {
        const NAME: &'static str = "BustleDetailsView";
        type Type = super::DetailsView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for DetailsView {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().update_rows();
        }

        fn dispose(&self) {
            self.dispose_template();
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("show-message-request")
                    .param_types([Message::static_type()])
                    .build()]
            });

            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for DetailsView {}

    impl DetailsView {
        fn set_message(&self, message: Option<Message>) {
            if message == self.message.replace(message.clone()) {
                return;
            }

            let obj = self.obj();

            self.message.replace(message.clone());
            obj.update_rows();
            obj.notify_message();
        }
    }
}

glib::wrapper! {
     pub struct DetailsView(ObjectSubclass<imp::DetailsView>)
        @extends gtk::Widget;
}

impl DetailsView {
    pub fn connect_show_message_request<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self, &Message) + 'static,
    {
        self.connect_closure(
            "show-message-request",
            false,
            closure_local!(|obj: &Self, message: &Message| {
                f(obj, message);
            }),
        )
    }

    fn update_rows(&self) {
        let imp = self.imp();

        let message = imp.message.borrow();
        let message = message.as_ref();

        let header = message.map(|message| message.header());

        imp.associated_message_group
            .set_visible(message.is_some_and(|message| message.associated_message().is_some()));
        imp.type_row
            .set_subtitle(&message.map(|m| m.message_type().i18n()).unwrap_or_default());
        imp.sender_row
            .set_subtitle(&message.map(|m| m.sender_display()).unwrap_or_default());
        imp.destination_row
            .set_subtitle(&message.map(|m| m.destination_display()).unwrap_or_default());
        let message_tag = message.map(|m| m.message_tag()).unwrap_or_default();
        imp.component_color.set_rgba(message_tag.color());
        imp.component_row.set_subtitle(&message_tag.name());
        imp.path_row
            .set_subtitle(&message.map(|m| m.path_display()).unwrap_or_default());
        imp.interface_row
            .set_subtitle(&message.map(|m| m.interface_display()).unwrap_or_default());
        imp.member_row
            .set_subtitle(&message.map(|m| m.member_display()).unwrap_or_default());

        imp.error_row.set_visible(
            message
                .map(|m| m.message_type().is_error())
                .unwrap_or(false),
        );
        imp.error_row.set_subtitle(
            &header
                .as_ref()
                .and_then(|m| m.error_name().map(|e| e.to_string()))
                .unwrap_or_default(),
        );

        let flags = message.map(|i| i.flags_display()).unwrap_or_default();
        imp.flags_row.set_visible(!flags.is_empty());
        imp.flags_row.set_subtitle(&flags);

        imp.size_row.set_subtitle(
            &header
                .as_ref()
                .map(|h| glib::format_size(h.primary().body_len() as u64))
                .unwrap_or_default(),
        );
        let signature = header
            .as_ref()
            .and_then(|h| h.signature())
            .map(|s| s.to_string())
            .unwrap_or_default();
        imp.signature_row.set_visible(!signature.is_empty());
        imp.signature_row.set_subtitle(&signature);

        let response_time = message.and_then(|m| m.response_time());
        imp.response_time_row.set_visible(
            response_time.is_some()
                && message.map_or(false, |m| m.message_type().is_method_return()),
        );
        imp.response_time_row.set_subtitle(
            &response_time
                .map(|ts| format!("{:.2} ms", ts.as_millis_f64()))
                .unwrap_or_default(),
        );

        imp.arguments_text_buffer.set_text(
            &message
                .and_then(|m| {
                    m.body()
                        .deserialize::<zvariant::Structure<'_>>()
                        .ok()
                        .map(|s| s.to_string())
                })
                .unwrap_or_default(),
        );
    }
}

#[gtk::template_callbacks]
impl DetailsView {
    #[template_callback]
    fn associated_message_activated(&self) {
        // It is safe to unwrap as the row is only visible when there is a message
        // that has an associated message.
        let associated_message = self.message().unwrap().associated_message().unwrap();
        self.emit_by_name::<()>("show-message-request", &[&associated_message]);
    }
}
