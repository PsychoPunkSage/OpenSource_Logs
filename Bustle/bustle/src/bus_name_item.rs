// Import necessary modules from gtk, glib, indexmap, and zbus crates
use gtk::{glib, subclass::prelude::*};
use indexmap::IndexSet;
use zbus::names::{BusName, WellKnownName};

// Define an enumeration to represent the point at which to look up for well-known names
#[derive(Clone, Copy)]
pub enum LookupPoint {
    // Union of all well-known names at all indices
    All,
    // Well-known names at the given index
    Index(ReceiveIndex),
    // Well-known names at the last index
    Last,
}

// Implement conversion from ReceiveIndex to LookupPoint
impl From<ReceiveIndex> for LookupPoint {
    fn from(receive_index: ReceiveIndex) -> Self {
        Self::Index(receive_index)
    }
}

// Define a submodule `imp` for internal implementation details
mod imp {
    use std::{
        cell::{OnceCell, RefCell},
        collections::BTreeMap,
    };

    use super::*;

    // Define a struct to hold information about each bus name item
    #[derive(Default)]
    pub struct BusNameItem {
        // OnceCell to store the bus name
        pub(super) name: OnceCell<BusName<'static>>,
        // RefCell to store the well-known names log
        pub(super) wk_name_log: RefCell<BTreeMap<ReceiveIndex, IndexSet<WellKnownName<'static>>>>,
    }

    // Implement the ObjectSubclass trait for BusNameItem
    #[glib::object_subclass]
    impl ObjectSubclass for BusNameItem {
        const NAME: &'static str = "BustleBusNameItem";
        type Type = super::BusNameItem;
    }

    // Implement ObjectImpl trait for BusNameItem
    impl ObjectImpl for BusNameItem {}
}

// Use glib::wrapper macro to define a wrapper struct for BusNameItem
glib::wrapper! {
    pub struct BusNameItem(ObjectSubclass<imp::BusNameItem>);
}

// Implement methods for BusNameItem
impl BusNameItem {
    // Method to get the bus name
    pub fn name(&self) -> &BusName<'static> {
        self.imp().name.get().unwrap()
    }

    // Method to get the well-known names at the specified lookup point
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

    // Method to create a new BusNameItem
    pub fn new(name: BusName<'static>) -> Self {
        let this = glib::Object::new::<Self>();
        this.imp().name.set(name).unwrap();
        this
    }

    // Method to insert well-known names into the log
    pub fn insert_wk_name_log(
        &self,
        receive_index: ReceiveIndex,
        wk_names: IndexSet<WellKnownName<'static>>,
    ) {
        let prev_entry = self
            .imp() // Internal `imp`
            .wk_name_log
            .borrow_mut() // This method call borrows the wk_name_log field mutably, allowing us to modify its contents.
            .insert(receive_index, wk_names.clone());
        debug_assert_eq!(
            prev_entry,
            None, // panics with a provided message if they (prev_entry, None) are not equal
            "duplicate entry `{:?}` for the same receive index `{:?}`",
            wk_names,
            receive_index
        );
    }
}
