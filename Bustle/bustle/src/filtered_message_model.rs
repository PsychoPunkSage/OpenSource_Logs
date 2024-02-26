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
    pub fn set_message_list(&self, message_list: Option<&MessageList>) {
        self.clear_filters();

        let imp = self.imp();

        // Must set the model for `inner` first to have used names updated
        imp.inner.set_model(message_list);

        imp.filtered_bus_names
            .set_bus_name_list(message_list.map(|l| l.bus_names()));
    }

    pub fn message_list(&self) -> Option<MessageList> {
        self.imp()
            .inner
            .model()
            .map(|model| model.downcast().unwrap())
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
        let ret = if let Some(index) = self
            .imp()
            .message_tag_filter_indices
            .borrow_mut()
            .remove(&message_tag)
        {
            self.inner_filter().remove(index);
            true
        } else {
            false
        };

        self.notify_has_filter();

        ret
    }

    /// Adds filter that filters out messages relevant to `BusNameItem` with
    /// `bus_name` equals given `name`
    ///
    /// Returns an error if self has no `message_list`
    pub fn add_bus_name_filter(&self, name: &BusName<'_>) -> Result<()> {
        let bus_name_item = self
            .message_list()
            .context("Message list was not sent")?
            .bus_names()
            .get(name)
            .unwrap();
        let custom_filter = gtk::CustomFilter::new(move |message| {
            let message = message.downcast_ref::<Message>().unwrap();
            let name = bus_name_item.name();
            !message.sender().is_some_and(|sender| *name == sender)
                && !message.destination().is_some_and(|destination| {
                    *name == destination
                        || match destination {
                            BusName::Unique(_) => false,
                            BusName::WellKnown(wk_name) => bus_name_item
                                .wk_names(message.receive_index().into())
                                .contains(&wk_name),
                        }
                })
        });

        let inner_filter = self.inner_filter();
        let index = inner_filter.n_items();
        inner_filter.append(custom_filter);
        let prev_value = self
            .imp()
            .bus_name_filter_indices
            .borrow_mut()
            .insert(name.to_owned(), index);
        debug_assert!(prev_value.is_none());
        debug_assert!(self
            .imp()
            .message_tag_filter_indices
            .borrow()
            .values()
            .all(|i| { *i != index }));

        self.notify_has_filter();

        Ok(())
    }

    /// Removes the filter that is relevant to `BusNameItem` with `bus_name`
    /// equals given `name`
    ///
    /// Returns true if the filter existed and removed
    pub fn remove_bus_name_filter(&self, name: &BusName<'static>) -> bool {
        let ret = if let Some(index) = self.imp().bus_name_filter_indices.borrow_mut().remove(name)
        {
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

        let message_tag_filter_indices = imp.message_tag_filter_indices.take();
        let bus_name_filter_indices = imp.bus_name_filter_indices.take();
        let mut indices = message_tag_filter_indices
            .values()
            .chain(bus_name_filter_indices.values())
            .collect::<Vec<_>>();

        let inner_filter = self.inner_filter();

        // Remove last indices first to not disrupt the indices
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
