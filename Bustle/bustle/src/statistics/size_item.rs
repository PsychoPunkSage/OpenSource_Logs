use gtk::{glib, prelude::*, subclass::prelude::*};

use crate::message_type::MessageType;

mod imp {
    use std::cell::{Cell, OnceCell};

    use super::*;

    #[derive(Debug, Default, glib::Properties)]
    #[properties(wrapper_type = super::SizeItem)]
    pub struct SizeItem {
        #[property(get, set, construct_only)]
        pub(super) member: OnceCell<String>,
        #[property(get, set, construct_only)]
        pub(super) smallest: Cell<u32>,
        #[property(get, set, construct_only)]
        pub(super) mean: Cell<u32>,
        #[property(get, set, construct_only)]
        pub(super) largest: Cell<u32>,
        #[property(get, set, construct_only, builder(MessageType::default()))]
        pub(super) message_type: OnceCell<MessageType>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SizeItem {
        const NAME: &'static str = "BustleSizeItem";
        type Type = super::SizeItem;
    }
    #[glib::derived_properties]
    impl ObjectImpl for SizeItem {}
}

glib::wrapper! {
    pub struct SizeItem(ObjectSubclass<imp::SizeItem>);
}

impl SizeItem {
    pub fn new(
        member: &str,
        smallest: u32,
        mean: u32,
        largest: u32,
        message_type: MessageType,
    ) -> Self {
        glib::Object::builder()
            .property("member", member)
            .property("smallest", smallest)
            .property("mean", mean)
            .property("largest", largest)
            .property("message-type", message_type)
            .build()
    }
}
