use gtk::{glib, prelude::*, subclass::prelude::*};

use crate::timestamp::Timestamp;

mod imp {
    use std::cell::{Cell, OnceCell};

    use super::*;

    #[derive(Debug, Default, glib::Properties)]
    #[properties(wrapper_type = super::DurationItem)]
    pub struct DurationItem {
        #[property(get, set, construct_only)]
        pub(super) method: OnceCell<String>,
        #[property(get, set, construct_only)]
        pub(super) total: Cell<Timestamp>,
        #[property(get, set, construct_only)]
        pub(super) calls: Cell<u32>,
        #[property(get, set, construct_only)]
        pub(super) total_calls: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DurationItem {
        const NAME: &'static str = "BustleDurationItem";
        type Type = super::DurationItem;
    }
    #[glib::derived_properties]
    impl ObjectImpl for DurationItem {}
}

glib::wrapper! {
    pub struct DurationItem(ObjectSubclass<imp::DurationItem>);
}

impl DurationItem {
    pub fn new(method: &str, calls: u32, total_calls: u32, total: Timestamp) -> Self {
        glib::Object::builder()
            .property("method", method)
            .property("calls", calls)
            .property("total-calls", total_calls)
            .property("total", total)
            .build()
    }
}
