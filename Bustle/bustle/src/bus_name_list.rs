use std::collections::HashSet;

use anyhow::{bail, Context, Result};
use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use indexmap::{map::Entry, IndexSet};
use zbus::{
    fdo::DBusProxy,
    names::{BusName, UniqueName, WellKnownName},
    zvariant::Optional,
    ProxyDefault,
};

use crate::{
    bus_name_item::{BusNameItem, LookupPoint},
    message::{Message, ReceiveIndex},
    message_type::MessageType,
};

mod imp {
    use std::cell::RefCell;

    use indexmap::IndexMap;

    use super::*;

    #[derive(Default)]
    pub struct BusNameList {
        pub(super) inner: RefCell<IndexMap<BusName<'static>, BusNameItem>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BusNameList {
        const NAME: &'static str = "BustleBusNameList";
        type Type = super::BusNameList;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for BusNameList {}

    impl ListModelImpl for BusNameList {
        fn item_type(&self) -> glib::Type {
            BusNameItem::static_type()
        }

        fn n_items(&self) -> u32 {
            self.inner.borrow().len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            self.inner
                .borrow()
                .get_index(position as usize)
                .map(|(_, v)| v.upcast_ref::<glib::Object>())
                .cloned()
        }
    }
}

glib::wrapper! {
    pub struct BusNameList(ObjectSubclass<imp::BusNameList>)
        @implements gio::ListModel;
}

impl BusNameList {
    pub fn handle_message(&self, message: &Message) -> Result<()> {
        match message.message_type() {
            MessageType::MethodCall => self.handle_method_call_message(message),
            MessageType::MethodReturn | MessageType::Error => {
                self.handle_method_return_message(message)
            }
            MessageType::Signal => self.handle_signal_message(message),
        }
    }

    /// Returns the `BusNameItem` with the given `bus_name` as item's `bus_name`
    pub fn get(&self, bus_name: &BusName<'_>) -> Option<BusNameItem> {
        self.imp().inner.borrow().get(bus_name).cloned()
    }

    fn handle_method_call_message(&self, message: &Message) -> Result<()> {
        debug_assert!(message.message_type().is_method_call());

        let sender = message.sender().context("Call message has no sender")?;
        self.insert_bus_name(sender.to_owned().into());

        let destination = message
            .destination()
            .context("Call message has no destination")?;
        self.insert_bus_name(destination.to_owned());

        Ok(())
    }

    fn handle_method_return_message(&self, message: &Message) -> Result<()> {
        debug_assert!(message.message_type().is_method_return());

        let call_message = message
            .associated_message()
            .context("Return message has no associated call message")?;

        let call_header = call_message.header();

        if call_header
            .interface()
            .is_some_and(|i| i.as_str() == DBusProxy::INTERFACE.unwrap())
            && call_header.member().is_some_and(|m| m == "GetNameOwner")
        {
            self.handle_get_name_owner_message(&call_message, message)
                .context("Failed to handle get name owner message")?;
            return Ok(());
        }

        let imp = self.imp();

        match call_message
            .destination()
            .context("Call message has no destination")?
        {
            BusName::Unique(unique_name) => {
                // Already inserted on MethodCall handling
                debug_assert!(
                    imp.inner
                        .borrow()
                        .contains_key(&BusName::from(unique_name.as_ref())),
                    "{:?} was not found",
                    unique_name
                );
            }
            BusName::WellKnown(dest_wk_name) => {
                if !Message::is_fallback_destination(&BusName::from(dest_wk_name.as_ref())) {
                    // TODO All of these could be optimized by having a specialized `insert_wk_name`
                    // function, so instead of emitting items-changed thrice, we only emit it once.

                    // FIXME This disrupts ordering of names, see `IndexMap.move_index`
                    let removed = imp
                        .inner
                        .borrow_mut()
                        .shift_remove_full(&BusName::from(dest_wk_name.to_owned()));

                    if let Some((old_index, old_name, old_bus_name_item)) = removed {
                        debug_assert!(matches!(old_name, BusName::WellKnown(_)));
                        debug_assert_eq!(
                            old_bus_name_item.wk_names(LookupPoint::All),
                            IndexSet::new()
                        );

                        self.items_changed(old_index as u32, 1, 0);

                        // There was a well-known name in bus names map (which we removed), and now
                        // that we already know its unique name through this return message, we can
                        // now add that unique name to the map as a key, and move the well-known name
                        // to the items's well-known names.
                        let sender = message.sender().context("Return message has no sender")?;
                        self.insert_wk_name(
                            sender.to_owned().into(),
                            dest_wk_name.to_owned(),
                            message.receive_index(),
                        );

                        // After that, we need to go back in time when the well-known name was first
                        // discovered, and add the well-known name to the entry at that index.

                        // First, get the index and the bus name item we just inserted.
                        let inner = imp.inner.borrow();
                        let (new_index, _, new_bus_name_item) =
                            inner.get_full(&BusName::from(sender)).unwrap();

                        // Then add to the log entry the well-known name when it was first discovered,
                        // which is the call message's receive index
                        let mut wk_names =
                            new_bus_name_item.wk_names(call_message.receive_index().into());
                        wk_names.insert(dest_wk_name.to_owned());
                        new_bus_name_item
                            .insert_wk_name_log(call_message.receive_index(), wk_names);

                        // Finally, notify that we have modified an item's well-known names
                        drop(inner);
                        self.items_changed(new_index as u32, 1, 1);
                    } else {
                        // It was already handled by other return messages.
                        debug_assert!(imp.inner.borrow().values().any(|bus_name_item| {
                            bus_name_item
                                .wk_names(message.receive_index().into())
                                .contains(&dest_wk_name)
                        }));
                    }
                }
            }
        }

        match message
            .destination()
            .context("Return message has no destination")?
        {
            BusName::Unique(unique_name) => {
                // Already inserted on MethodCall handling
                debug_assert!(
                    imp.inner
                        .borrow()
                        .contains_key(&BusName::from(unique_name.as_ref())),
                    "{:?} was not found",
                    unique_name
                );
            }
            BusName::WellKnown(dest_wk_name) => {
                let sender = call_message
                    .sender()
                    .context("Call message has no sender")?;
                self.insert_wk_name(
                    sender.to_owned().into(),
                    dest_wk_name.to_owned(),
                    message.receive_index(),
                );
            }
        }

        Ok(())
    }

    fn handle_get_name_owner_message(
        &self,
        call_message: &Message,
        return_message: &Message,
    ) -> Result<()> {
        debug_assert!(call_message.message_type().is_method_call());
        debug_assert!(return_message.message_type().is_method_return());

        if return_message.message_type().is_error() {
            return Ok(());
        }

        debug_assert_eq!(
            call_message.header().member().map(|m| m.as_str()),
            Some("GetNameOwner")
        );

        let call_message_body = call_message.body();
        let return_message_body = return_message.body();
        let (name,) = call_message_body
            .deserialize::<(BusName<'_>,)>()
            .context("Failed to get call message body")?;
        let (owner,) = return_message_body
            .deserialize::<(UniqueName<'_>,)>()
            .context("Failed to get return message body")?;

        match name {
            BusName::Unique(unique_name) => {
                debug_assert_eq!(owner, unique_name);
            }
            BusName::WellKnown(wk_name) => {
                self.insert_wk_name(
                    owner.to_owned().into(),
                    wk_name.to_owned(),
                    return_message.receive_index(),
                );
            }
        }

        Ok(())
    }

    fn handle_signal_message(&self, message: &Message) -> Result<()> {
        debug_assert!(message.message_type().is_signal());
        let header = message.header();
        if header.interface().cloned().as_deref() == DBusProxy::INTERFACE
            && header.member().cloned().as_deref() == Some("NameOwnerChanged")
        {
            self.handle_name_owner_changed_message(message)
                .context("Failed to handle NOC message")?;
            return Ok(());
        }

        if let Some(sender) = message.sender() {
            self.insert_bus_name(sender.to_owned().into());
        }

        if let Some(destination) = message.destination() {
            self.insert_bus_name(destination.to_owned());
        }

        Ok(())
    }

    fn handle_name_owner_changed_message(&self, message: &Message) -> Result<()> {
        debug_assert_eq!(message.message_type(), MessageType::Signal);
        debug_assert_eq!(
            message.header().member().cloned().as_deref(),
            Some("NameOwnerChanged")
        );
        let body = message.body();
        let (name, old_owner, new_owner) = body
            .deserialize::<(
                BusName<'_>,
                Optional<UniqueName<'_>>,
                Optional<UniqueName<'_>>,
            )>()
            .context("Failed to get NOC message body")?;

        if old_owner.is_none() && new_owner.is_none() {
            bail!("Invalid NOC message; both old and new owner are none");
        }

        let wk_name = match name {
            BusName::Unique(ref unique_name) => {
                if let Some(old_owner) = old_owner.as_ref() {
                    debug_assert_eq!(unique_name, old_owner);
                }
                if let Some(new_owner) = new_owner.as_ref() {
                    debug_assert_eq!(unique_name, new_owner);
                }
                return Ok(());
            }
            BusName::WellKnown(wk_name) => wk_name,
        };

        // FIXME This should be specialized because, while it is harmless as
        // we remove first, there is a warning since we are trying to add a
        // log entry to the same receive index. Also, we emit items-changed twice

        if let Some(old_owner) = old_owner.as_ref() {
            self.remove_wk_name(
                old_owner.to_owned().into(),
                wk_name.to_owned(),
                message.receive_index(),
            );
        }

        if let Some(new_owner) = new_owner.as_ref() {
            self.insert_wk_name(
                new_owner.to_owned().into(),
                wk_name.to_owned(),
                message.receive_index(),
            );
        }

        Ok(())
    }

    /// Insert `bus_name` with empty `wk_names`
    fn insert_bus_name(&self, bus_name: BusName<'static>) {
        let index = match self.imp().inner.borrow_mut().entry(bus_name) {
            Entry::Occupied(_) => None,
            Entry::Vacant(entry) => {
                let index = entry.index();
                let bus_name_item = BusNameItem::new(entry.key().to_owned());
                entry.insert(bus_name_item);
                Some(index)
            }
        };
        if let Some(index) = index {
            self.items_changed(index as u32, 0, 1);
        }
    }

    /// Insert `wk_name` to `bus_name`'s `wk_names` or create `bus_name` entry first
    fn insert_wk_name(
        &self,
        bus_name: BusName<'static>,
        wk_name: WellKnownName<'static>,
        receive_index: ReceiveIndex,
    ) {
        let imp = self.imp();

        let (index, removed, added) = match imp.inner.borrow_mut().entry(bus_name) {
            Entry::Occupied(entry) => {
                let bus_name_item = entry.get();

                // Get last snapshot and create a new snapshot with the new well-known name
                let mut wk_names = bus_name_item.wk_names(receive_index.into());
                // TODO We can return here early and dont emit items-changed if the given well-known
                // name exist already in the current snapshot, but we want to ensure first that
                // the messages we handle come in a chronological order.
                wk_names.insert(wk_name);
                bus_name_item.insert_wk_name_log(receive_index, wk_names);

                (entry.index(), 1, 1)
            }
            Entry::Vacant(entry) => {
                let bus_name_item = BusNameItem::new(entry.key().to_owned());

                let wk_names = IndexSet::from([wk_name]);
                bus_name_item.insert_wk_name_log(receive_index, wk_names);

                let index = entry.index();
                entry.insert(bus_name_item);
                (index, 0, 1)
            }
        };
        self.items_changed(index as u32, removed, added);

        if cfg!(debug_assertions) {
            // Ensure that all well-known names has a unique owner
            let mut unique = HashSet::new();
            for (bus_name, bus_name_item) in imp.inner.borrow().iter() {
                assert_eq!(bus_name, bus_name_item.name());
                for wk_name in bus_name_item.wk_names(receive_index.into()) {
                    let was_inserted = unique.insert(wk_name.clone());
                    assert!(was_inserted, "duplicate well-known name: {}", wk_name);
                }
            }
        }
    }

    /// Remove `wk_name` from `bus_name`'s `wk_names` only if `bus_name` exists
    fn remove_wk_name(
        &self,
        bus_name: BusName<'static>,
        wk_name: WellKnownName<'static>,
        receive_index: ReceiveIndex,
    ) {
        let imp = self.imp();

        let index = match imp.inner.borrow_mut().entry(bus_name) {
            Entry::Occupied(entry) => {
                let bus_name_item = entry.get();

                // Get last snapshot and create a new snapshot without the well-known name
                let mut wk_names = bus_name_item.wk_names(receive_index.into());
                // TODO We can return here early and dont emit items-changed if the given well-known
                // name does not exist anyway in the current snapshot, but we want to ensure first that
                // the messages we handle come in a chronological order.
                wk_names.remove(&wk_name);
                bus_name_item.insert_wk_name_log(receive_index, wk_names);

                Some(entry.index())
            }
            Entry::Vacant(_) => None,
        };
        if let Some(index) = index {
            self.items_changed(index as u32, 1, 1);
        }

        if cfg!(debug_assertions) {
            // Ensure that all well-known names has a unique owner
            let mut unique = HashSet::new();
            for (bus_name, bus_name_item) in imp.inner.borrow().iter() {
                assert_eq!(bus_name, bus_name_item.name());
                for wk_name in bus_name_item.wk_names(receive_index.into()) {
                    let was_inserted = unique.insert(wk_name.clone());
                    assert!(was_inserted, "duplicate well-known name: {}", wk_name);
                }
            }
        }
    }
}

impl Default for BusNameList {
    fn default() -> Self {
        glib::Object::new()
    }
}
