// Import necessary modules from the gtk and zbus crates
use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};
use zbus::names::{BusName, WellKnownName};

// Import local modules
use crate::{
    bus_name_item::{BusNameItem, LookupPoint},
    bus_name_list::BusNameList,
};

// Define a submodule named 'imp'
mod imp {
    use std::marker::PhantomData;

    use super::*;

    // Define the properties for the FilteredBusNameModel
    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::FilteredBusNameModel)]
    pub struct FilteredBusNameModel {
        // specifies the getter method for the property, which is Self::bus_name_list  ||
        #[property(get = Self::bus_name_list, set = Self::set_bus_name_list, explicit_notify, nullable)]
        pub(super) bus_name_list: PhantomData<Option<BusNameList>>,

        pub(super) inner: gtk::FilterListModel,
    }

    // Implement the ObjectSubclass trait for FilteredBusNameModel
    // sets up the necessary information for registering FilteredBusNameModel as a GObject subclass with specific characteristics and interfaces.
    #[glib::object_subclass]
    impl ObjectSubclass for FilteredBusNameModel {
        const NAME: &'static str = "BustleFilteredBusNameModel";
        type Type = super::FilteredBusNameModel;
        type Interfaces = (gio::ListModel,);
    }

    // Implement methods for FilteredBusNameModel
    // enhances the behavior of the `FilteredBusNameModel` type by connecting it to a signal handler that updates the model when its items are changed.
    #[glib::derived_properties]
    impl ObjectImpl for FilteredBusNameModel {
        // Handle the constructed signal
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            // Connect the items changed signal to update the model
            self.inner.connect_items_changed(
                clone!(@weak obj => move |_, position, removed, added| {
                    obj.items_changed(position, removed, added);
                }),
            );
        }
    }

    // Implement ListModelImpl trait for FilteredBusNameModel

    /*
    Abstraction: A ListModel provides an abstraction over the data, allowing for separation of concerns between the data source and the UI components that display it. This separation makes the code more modular and easier to maintain.

    Lazy Loading: ListModel implementations can support lazy loading of data, which means that only the items that are currently    visible in the UI need to be loaded into memory. This can significantly improve performance, especially when dealing with large datasets.

    Dynamic Updates: ListModel implementations can handle dynamic updates to the underlying data source, such as additions, deletions,  or modifications of items. This allows the UI to automatically reflect changes in the data without manual intervention.

    Customization: ListModel implementations can be customized to provide additional functionalities specific to the application's  requirements, such as filtering, sorting, or grouping of data.

    Integration with GUI Frameworks: Many GUI frameworks provide built-in support for ListModel interfaces, making it easier to     integrate with UI components such as list views, tables, or grids.
    */
    impl ListModelImpl for FilteredBusNameModel {
        // Return the item type
        fn item_type(&self) -> glib::Type {
            BusNameItem::static_type()
        }

        // Return the number of items in the model
        fn n_items(&self) -> u32 {
            self.inner.n_items()
        }

        // Return the item at the specified position
        fn item(&self, position: u32) -> Option<glib::Object> {
            self.inner.item(position)
        }
    }

    // Implement additional methods for FilteredBusNameModel
    impl FilteredBusNameModel {
        // Set the bus name list
        fn set_bus_name_list(&self, bus_name_list: Option<&BusNameList>) {
            let obj = self.obj();

            if bus_name_list == obj.bus_name_list().as_ref() {
                return;
            }

            // `set_model` ::> filter_list_model
            self.inner.set_model(bus_name_list);
            obj.notify_bus_name_list();
        }

        // Get the bus name list
        fn bus_name_list(&self) -> Option<BusNameList> {
            self.inner.model().map(|model| model.downcast().unwrap())
        }
    }
}

// Create a wrapper for FilteredBusNameModel
glib::wrapper! {
    pub struct FilteredBusNameModel(ObjectSubclass<imp::FilteredBusNameModel>)
        @implements gio::ListModel;
}

impl FilteredBusNameModel {
    // Set the filter for the model
    pub fn set_filter(&self, filter: Option<&impl IsA<gtk::Filter>>) {
        self.imp().inner.set_filter(filter);
    }

    // Return an iterator over the items in the model
    pub fn iter(&self) -> impl Iterator<Item = BusNameItem> + '_ {
        ListModelExtManual::iter(self).map(|item| item.unwrap())
    }

    // Get the BusNameItem with the given bus name
    pub fn get(&self, bus_name: &BusName<'_>) -> Option<BusNameItem> {
        self.iter()
            .find(|bus_name_item| *bus_name_item.name() == *bus_name)
    }

    // Get the index of the BusNameItem with the given bus name
    pub fn get_index_of(&self, bus_name: &BusName<'_>) -> Option<usize> {
        self.iter()
            .position(|bus_name_item| *bus_name_item.name() == *bus_name)
    }

    // Get the index of the BusNameItem containing the well-known name at the specified lookup point
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

// Implement the Default trait for FilteredBusNameModel
impl Default for FilteredBusNameModel {
    fn default() -> Self {
        glib::Object::new()
    }
}
