use gtk::{glib, subclass::prelude::*};
use indexmap::IndexSet;
use zbus::names::{BusName, WellKnownName};

use crate::message::ReceiveIndex;

/// The point at which to look up for well-known names.
#[derive(Clone, Copy)]
pub enum LookupPoint {
    /// Union of all well-known names at all indices.
    All,
    /// Well-known names at the given index.
    Index(ReceiveIndex),
    /// Well-known names at the last index.
    Last,
}

impl From<ReceiveIndex> for LookupPoint {
    fn from(receive_index: ReceiveIndex) -> Self {
        Self::Index(receive_index)
    }
}

mod imp {
    use std::{
        cell::{OnceCell, RefCell},
        collections::BTreeMap,
    };

    use super::*;

    #[derive(Default)]
    pub struct BusNameItem {
        pub(super) name: OnceCell<BusName<'static>>,
        pub(super) wk_name_log: RefCell<BTreeMap<ReceiveIndex, IndexSet<WellKnownName<'static>>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BusNameItem {
        const NAME: &'static str = "BustleBusNameItem";
        type Type = super::BusNameItem;
    }

    impl ObjectImpl for BusNameItem {}
}

glib::wrapper! {
    pub struct BusNameItem(ObjectSubclass<imp::BusNameItem>);
}

impl BusNameItem {
    pub fn name(&self) -> &BusName<'static> {
        self.imp().name.get().unwrap()
    }

    /// Returns a copy of the well-known names that were known at the given lookup point.
    pub fn wk_names(&self, lookup_point: LookupPoint) -> IndexSet<WellKnownName<'static>> {
        let wk_name_log = self.imp().wk_name_log.borrow();
        match lookup_point {
            LookupPoint::All => wk_name_log.values().flatten().cloned().collect(),
            LookupPoint::Index(receive_index) => wk_name_log
                .range(..=receive_index)
                .next_back()
                .map(|(_, wk_names)| wk_names.clone())
                .unwrap_or_default(),
            LookupPoint::Last => wk_name_log
                .last_key_value()
                .map(|(_, wk_names)| wk_names.clone())
                .unwrap_or_default(),
        }
    }

    /// This must only be called on `BusNameList`
    pub fn new(name: BusName<'static>) -> Self {
        let this = glib::Object::new::<Self>();
        this.imp().name.set(name).unwrap();
        this
    }

    /// This must only be called on `BusNameList`
    pub fn insert_wk_name_log(
        &self,
        receive_index: ReceiveIndex,
        wk_names: IndexSet<WellKnownName<'static>>,
    ) {
        let prev_entry = self
            .imp()
            .wk_name_log
            .borrow_mut()
            .insert(receive_index, wk_names.clone());
        debug_assert_eq!(
            prev_entry, None,
            "duplicate entry `{:?}` for the same receive index `{:?}`",
            wk_names, receive_index
        );
    }
}
