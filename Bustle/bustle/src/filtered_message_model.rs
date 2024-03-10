use anyhow::{Context, Result};
use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};
use zbus::{fdo::DBusProxy, names::BusName, ProxyDefault};

use crate::{
    bus_name_item::{BusNameItem, LookupPoint},
    filtered_bus_name_model::FilteredBusNameModel,
    message::Message,
    message_list::MessageList,
    message_tag::MessageTag,
};

mod imp {
    // Import necessary standard library and external dependencies
    use std::{
        cell::RefCell,
        collections::{HashMap, HashSet},
        marker::PhantomData,
    };

    // Import the IndexSet data structure from the indexmap crate
    use indexmap::IndexSet;

    // Import the parent module
    use super::*;

    // Define the internal struct `FilteredMessageModel` with its properties
    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::FilteredMessageModel)]
    pub struct FilteredMessageModel {
        // A PhantomData field to hold the type information for `has_filter`
        #[property(get = Self::has_filter)]
        pub(super) has_filter: PhantomData<bool>,

        // The inner filter list model
        pub(super) inner: gtk::FilterListModel,
        // A RefCell to hold the index set of messages in the inner model
        pub(super) inner_index: RefCell<IndexSet<Message>>,

        // The filtered bus name model
        pub(super) filtered_bus_names: FilteredBusNameModel,

        // A RefCell to hold the indices of message tag filters
        pub(super) message_tag_filter_indices: RefCell<HashMap<MessageTag, u32>>,
        // A RefCell to hold the indices of bus name filters
        pub(super) bus_name_filter_indices: RefCell<HashMap<BusName<'static>, u32>>,

        // A RefCell to hold the set of used bus names
        pub(super) used_bus_names: RefCell<HashSet<BusName<'static>>>,
    }

    // Implement GObject subclassing for `FilteredMessageModel`
    #[glib::object_subclass]
    impl ObjectSubclass for FilteredMessageModel {
        // Define the name, type, and interfaces for the subclass
        const NAME: &'static str = "BustleFilteredMessageModel";
        type Type = super::FilteredMessageModel;
        type Interfaces = (gio::ListModel,);
    }

    // Implement object properties and methods for `FilteredMessageModel`
    #[glib::derived_properties]
    impl ObjectImpl for FilteredMessageModel {
        // Define behavior when the object is constructed
        fn constructed(&self) {
            // Call the constructed method of the parent object
            self.parent_constructed();

            // Clone a weak reference to the object for use in closures
            let obj = self.obj();

            // Create a custom filter for bus names
            let bus_names_filter =
                gtk::CustomFilter::new(clone!(@weak obj => @default-panic, move |bus_name_item| {
                    let bus_name_item = bus_name_item.downcast_ref::<BusNameItem>().unwrap();
                    obj.bus_names_filter_func(bus_name_item)
                }));

            // Connect signals to update the inner index and used names
            self.inner.connect_items_changed(
                clone!(@weak obj, @weak bus_names_filter => move |_, position, removed, added| {
                    obj.update_inner_index(position, removed, added);
                    obj.update_used_names(position, removed, added);
                    bus_names_filter.changed(gtk::FilterChange::Different);
                    obj.items_changed(position, removed, added);
                }),
            );

            // Set the filter for filtered bus names
            self.filtered_bus_names.set_filter(Some(&bus_names_filter));

            // Create an every filter for message filtering
            let filter = gtk::EveryFilter::new();
            filter.append(gtk::CustomFilter::new(|message| {
                let message = message.downcast_ref::<Message>().unwrap();

                // Filter out messages related to DBusProxy
                // dbus_bus_name_exists_in_dbus(DBusProxy::DESTINATION)
                //     && (message.destination().as_deref() != Some(DBusProxy::DESTINATION)
                //         || message.sender().as_deref() != Some(DBusProxy::DESTINATION))
                message.destination().as_deref() != DBusProxy::DESTINATION
                    && message.sender().as_deref() != DBusProxy::DESTINATION
            }));

            // Set the filter for the inner model
            self.inner.set_filter(Some(&filter));
        }
    }

    // Implement ListModel methods for `FilteredMessageModel`
    impl ListModelImpl for FilteredMessageModel {
        // Define the type of items in the model
        fn item_type(&self) -> glib::Type {
            Message::static_type()
        }

        // Get the number of items in the model
        fn n_items(&self) -> u32 {
            self.inner.n_items()
        }

        // Get an item from the model at a given position
        fn item(&self, position: u32) -> Option<glib::Object> {
            self.inner.item(position)
        }
    }

    // Implement additional methods for `FilteredMessageModel`
    impl FilteredMessageModel {
        // Check if any filter is applied
        fn has_filter(&self) -> bool {
            self.message_tag_filter_indices.borrow().len() != 0
                || self.bus_name_filter_indices.borrow().len() != 0
        }
    }
}

glib::wrapper! {
    pub struct FilteredMessageModel(ObjectSubclass<imp::FilteredMessageModel>)
        @implements gio::ListModel;
}

impl FilteredMessageModel {
    /// This also resets the filters

    /// Sets the message list for the filtered message model, clearing existing filters.
    ///
    /// # Arguments
    ///
    /// * `message_list` - An optional reference to a `MessageList`.
    ///
    /// # Remarks
    ///
    /// This method first clears any existing filters, then sets the model for the inner list
    /// (`inner`). Additionally, it sets the bus name list for the filtered bus names model
    /// (`filtered_bus_names`).
    pub fn set_message_list(&self, message_list: Option<&MessageList>) {
        self.clear_filters();

        let imp = self.imp();

        // Must set the model for `inner` first to have used names updated
        imp.inner.set_model(message_list);

        imp.filtered_bus_names
            .set_bus_name_list(message_list.map(|l| l.bus_names()));
    }

    /// Gets the message list associated with the filtered message model.
    ///
    /// # Returns
    ///
    /// An optional `MessageList`.
    pub fn message_list(&self) -> Option<MessageList> {
        self.imp()
            .inner
            .model() // retrieves the model associated with the inner field, which is expected to contain messages.
            .map(|model| model.downcast().unwrap()) // Maps the result of the `model` method call to an `Option<MessageList>`.
                                                    // If the model is not `None`, it tries to downcast it to a `MessageList`.
    }

    pub fn get_index_of(&self, message: &Message) -> Option<usize> {
        self.imp().inner_index.borrow().get_index_of(message)
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = Message> + ExactSizeIterator + DoubleEndedIterator + '_ {
        ListModelExtManual::iter(self).map(|item| item.unwrap())
    }

    /// Returns the filtered bus names from self's message list with respect to
    /// self's filters
    pub fn filtered_bus_names(&self) -> &FilteredBusNameModel {
        &self.imp().filtered_bus_names
    }

    /// Adds filter that filters out messages with the given tag
    pub fn add_message_tag_filter(&self, message_tag: MessageTag) {
        let custom_filter = gtk::CustomFilter::new(move |message| {
            let message = message.downcast_ref::<Message>().unwrap();
            message.message_tag() != message_tag
        });

        let inner_filter = self.inner_filter();
        let index = inner_filter.n_items();
        inner_filter.append(custom_filter);
        let prev_value = self
            .imp()
            .message_tag_filter_indices
            .borrow_mut()
            .insert(message_tag, index);
        debug_assert!(prev_value.is_none());
        debug_assert!(self
            .imp()
            .bus_name_filter_indices
            .borrow()
            .values()
            .all(|i| *i != index));

        self.notify_has_filter();
    }

    /// Removes the filter that filters out messages with the given tag
    pub fn remove_message_tag_filter(&self, message_tag: MessageTag) -> bool {
        // Attempt to remove the index of the message tag from the message tag filter indices
        let ret = if let Some(index) = self
            .imp()
            .message_tag_filter_indices
            .borrow_mut()
            .remove(&message_tag)
        {
            // If the index was found and removed, remove the corresponding filter from the inner filter
            self.inner_filter().remove(index);
            true // Return true indicating successful removal
        } else {
            false // Return false indicating the filter was not found
        };

        // Notify listeners that the filter has been updated
        self.notify_has_filter();

        ret // Return the result of the removal operation
    }

    /// Adds filter that filters out messages relevant to `BusNameItem` with
    /// `bus_name` equals given `name`
    ///
    /// Returns an error if self has no `message_list`
    pub fn add_bus_name_filter(&self, name: &BusName<'_>) -> Result<()> {
        // Retrieve the `BusNameItem` corresponding to the provided `name` from the message list.
        // If the message list is not provided, return an error.
        let bus_name_item = self
            .message_list()
            .context("Message list was not sent")?
            .bus_names()
            .get(name)
            .unwrap();

        // Create a custom filter that filters out messages based on the provided `name`.
        let custom_filter = gtk::CustomFilter::new(move |message| {
            let message = message.downcast_ref::<Message>().unwrap();
            let name = bus_name_item.name();
            !message.sender().is_some_and(|sender| *name == sender)
                && !message.destination().is_some_and(|destination| {
                    // If the destination is a Unique name, it is not filtered out.
                    *name == destination
                        || match destination {
                            BusName::Unique(_) => false,
                            // If the destination is a WellKnown name, check if the BusNameItem's
                            // WellKnown names contain the destination name.
                            BusName::WellKnown(wk_name) => bus_name_item
                                .wk_names(message.receive_index().into())
                                .contains(&wk_name),
                        }
                })
        });

        // Get the inner filter and append the custom filter to it.
        let inner_filter = self.inner_filter();
        let index = inner_filter.n_items();
        inner_filter.append(custom_filter);

        // Update the bus name filter indices with the provided `name`.
        let prev_value = self
            .imp()
            .bus_name_filter_indices
            .borrow_mut()
            .insert(name.to_owned(), index);

        // Ensure that the previous value for the index was not already set.
        debug_assert!(prev_value.is_none());

        // Ensure that the index is not already present in the message tag filter indices.
        debug_assert!(self
            .imp()
            .message_tag_filter_indices
            .borrow()
            .values()
            .all(|i| { *i != index }));

        // Notify that a filter has been added.
        self.notify_has_filter();

        Ok(())
    }

    /// Removes the filter that is relevant to `BusNameItem` with `bus_name`
    /// equals given `name`
    ///
    /// Returns true if the filter existed and removed
    pub fn remove_bus_name_filter(&self, name: &BusName<'static>) -> bool {
        // Try to remove the filter for the given `BusName`
        let ret = if let Some(index) = self.imp().bus_name_filter_indices.borrow_mut().remove(name)
        {
            // If the filter was found and removed, also remove it from the inner filter
            self.inner_filter().remove(index);
            true
        } else {
            false
        };

        self.notify_has_filter();

        ret
    }

    pub fn clear_filters(&self) {
        let imp = self.imp();

        // Take ownership of the message tag and bus name filter indices
        let message_tag_filter_indices = imp.message_tag_filter_indices.take();
        let bus_name_filter_indices = imp.bus_name_filter_indices.take();

        // Collect all filter indices into a vector
        let mut indices = message_tag_filter_indices
            .values()
            .chain(bus_name_filter_indices.values())
            .collect::<Vec<_>>();

        // Sort indices to ensure proper removal order
        let inner_filter = self.inner_filter();

        // Remove filters from the inner filter in reverse order to avoid index disruption
        indices.sort();
        for index in indices.iter().rev() {
            inner_filter.remove(**index);
        }

        self.notify_has_filter();
    }

    fn inner_filter(&self) -> gtk::EveryFilter {
        self.imp()
            .inner
            .filter()
            .expect("filter was not set on constructed")
            .downcast()
            .unwrap()
    }

    fn bus_names_filter_func(&self, bus_name_item: &BusNameItem) -> bool {
        let used_names = self.imp().used_bus_names.borrow();

        used_names.contains(bus_name_item.name())
            || bus_name_item
                .wk_names(LookupPoint::All)
                .iter()
                .any(|wk_name| used_names.contains(&BusName::from(wk_name.as_ref())))
    }

    fn update_inner_index(&self, position: u32, removed: u32, added: u32) {
        let imp = self.imp();

        let mut inner_index = imp.inner_index.borrow_mut();

        // FIXME I wish there is IndexMap.splice
        let end_part = inner_index.split_off(position as usize);
        let new_items =
            (0..added).map(|i| imp.inner.item(position + i).unwrap().downcast().unwrap());
        inner_index.extend(new_items.chain(end_part.into_iter().skip(removed as usize)));

        debug_assert_eq!(inner_index.len(), imp.inner.n_items() as usize);
        debug_assert!(self
            .iter()
            .zip(inner_index.iter())
            .all(|(inner_item, inner_index_item)| inner_item == *inner_index_item));
    }

    fn update_used_names(&self, _position: u32, _removed: u32, _added: u32) {
        // TODO optimize by considering only what actually changed instead of computing
        // everytime the message list changes

        let used_sender_names = self
            .iter()
            .filter_map(|message| message.sender().map(|s| BusName::from(s.to_owned())));
        let used_destination_names = self
            .iter()
            .filter_map(|message| message.destination().map(|d| d.to_owned()));
        self.imp()
            .used_bus_names
            .replace(used_sender_names.chain(used_destination_names).collect());
    }
}
// ********** @CheckPoint **********
// pub fn dbus_bus_name_exists_in_dbus(self, bus_name: &str) -> bool {
//     // Retrieve the message list from the filtered message model
//     let message_list = self.message_list();

//     // Check if the message list is available
//     if let Some(message_list) = message_list {
//         // Iterate over each message in the message list
//         for message in message_list.iter() {
//             // Check if the sender or destination of the message matches the given bus name
//             if let Some(sender) = message.sender() {
//                 if sender == bus_name {
//                     return true; // Found a match, return true
//                 }
//             }
//             if let Some(destination) = message.destination() {
//                 if destination == bus_name {
//                     return true; // Found a match, return true
//                 }
//             }
//         }
//     }

//     false // No match found, return false
// }
// }
