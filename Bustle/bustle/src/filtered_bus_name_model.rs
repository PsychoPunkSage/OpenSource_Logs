use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};
use zbus::names::{BusName, WellKnownName};

use crate::{
    bus_name_item::{BusNameItem, LookupPoint},
    bus_name_list::BusNameList,
};

mod imp {
    use std::marker::PhantomData;

    use super::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::FilteredBusNameModel)]
    pub struct FilteredBusNameModel {
        #[property(get = Self::bus_name_list, set = Self::set_bus_name_list, explicit_notify, nullable)]
        pub(super) bus_name_list: PhantomData<Option<BusNameList>>,

        pub(super) inner: gtk::FilterListModel,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FilteredBusNameModel {
        const NAME: &'static str = "BustleFilteredBusNameModel";
        type Type = super::FilteredBusNameModel;
        type Interfaces = (gio::ListModel,);
    }

    #[glib::derived_properties]
    impl ObjectImpl for FilteredBusNameModel {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            self.inner.connect_items_changed(
                clone!(@weak obj => move |_, position, removed, added| {
                    obj.items_changed(position, removed, added);
                }),
            );
        }
    }

    impl ListModelImpl for FilteredBusNameModel {
        fn item_type(&self) -> glib::Type {
            BusNameItem::static_type()
        }

        fn n_items(&self) -> u32 {
            self.inner.n_items()
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            self.inner.item(position)
        }
    }

    impl FilteredBusNameModel {
        fn set_bus_name_list(&self, bus_name_list: Option<&BusNameList>) {
            let obj = self.obj();

            if bus_name_list == obj.bus_name_list().as_ref() {
                return;
            }

            self.inner.set_model(bus_name_list);
            obj.notify_bus_name_list();
        }

        fn bus_name_list(&self) -> Option<BusNameList> {
            self.inner.model().map(|model| model.downcast().unwrap())
        }
    }
}

glib::wrapper! {
    pub struct FilteredBusNameModel(ObjectSubclass<imp::FilteredBusNameModel>)
        @implements gio::ListModel;
}

impl FilteredBusNameModel {
    /// This must only be called within `FilteredMessageModel`
    pub fn set_filter(&self, filter: Option<&impl IsA<gtk::Filter>>) {
        self.imp().inner.set_filter(filter);
    }

    pub fn iter(&self) -> impl Iterator<Item = BusNameItem> + '_ {
        ListModelExtManual::iter(self).map(|item| item.unwrap())
    }

    // TODO the following could be optimize if we don't rely on GTK's
    // `FilterListModel`

    /// Returns the `BusNameItem` with the given `bus_name` as item's `name`
    pub fn get(&self, bus_name: &BusName<'_>) -> Option<BusNameItem> {
        self.iter()
            .find(|bus_name_item| *bus_name_item.name() == *bus_name)
    }

    /// Returns the index of `BusNameItem` with the given `bus_name` as item's
    /// `name`
    pub fn get_index_of(&self, bus_name: &BusName<'_>) -> Option<usize> {
        self.iter()
            .position(|bus_name_item| *bus_name_item.name() == *bus_name)
    }

    /// Returns the index of `BusNameItem` containing the `wk_name` at the given lookup point.
    pub fn get_index_of_wk_name(
        &self,
        wk_name: &WellKnownName<'_>,
        lookup_point: LookupPoint,
    ) -> Option<usize> {
        debug_assert_eq!(
            self.get(&BusName::from(wk_name.as_ref())),
            None,
            "`get` or `get_index_of` must be used first"
        );

        self.iter()
            .position(|bus_name_item| bus_name_item.wk_names(lookup_point).contains(wk_name))
    }
}

impl Default for FilteredBusNameModel {
    fn default() -> Self {
        glib::Object::new()
    }
}
