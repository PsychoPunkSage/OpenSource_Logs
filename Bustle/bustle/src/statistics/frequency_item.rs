use gtk::{glib, prelude::*, subclass::prelude::*};

use crate::message_type::MessageType;

mod imp {
    use std::cell::{Cell, OnceCell};

    use super::*;

    #[derive(Debug, Default, glib::Properties)]
    #[properties(wrapper_type = super::FrequencyItem)]
    pub struct FrequencyItem {
        #[property(get, set, construct_only)]
        pub(super) member: OnceCell<String>,
        #[property(get, set, construct_only)]
        pub(super) count: Cell<u32>,
        #[property(get, set, construct_only)]
        pub(super) total: Cell<u32>,
        #[property(get, set, construct_only, builder(MessageType::default()))]
        pub(super) message_type: Cell<MessageType>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FrequencyItem {
        const NAME: &'static str = "BustleFrequencyItem";
        type Type = super::FrequencyItem;
    }
    #[glib::derived_properties]
    impl ObjectImpl for FrequencyItem {}
}

glib::wrapper! {
    pub struct FrequencyItem(ObjectSubclass<imp::FrequencyItem>);
}

impl FrequencyItem {
    pub fn new(member: &str, count: u32, total: u32, message_type: MessageType) -> Self {
        glib::Object::builder()
            .property("member", member)
            .property("count", count)
            .property("total", total)
            .property("message-type", message_type)
            .build()
    }
}
