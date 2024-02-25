use gettextrs::gettext;
use gtk::{glib, prelude::*, subclass::prelude::*};
use zbus::names::{BusName, UniqueName, WellKnownName};

use crate::{
    message_tag::MessageTag, message_type::MessageType, monitor::Event, timestamp::Timestamp,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReceiveIndex(u32);

mod imp {
    use std::cell::{Cell, OnceCell};

    use super::*;

    #[derive(Debug, Default, glib::Properties)]
    #[properties(wrapper_type = super::Message)]
    pub struct Message {
        #[property(get, set, construct_only, builder(MessageType::default()))]
        pub(super) message_type: Cell<MessageType>,
        #[property(get, set, construct_only)]
        pub(super) timestamp: OnceCell<Timestamp>,
        #[property(get, set, builder(MessageTag::default()))]
        pub(super) message_tag: Cell<MessageTag>,

        pub(super) inner: OnceCell<zbus::Message>,
        pub(super) receive_index: OnceCell<ReceiveIndex>,

        pub(super) associated_message: OnceCell<glib::WeakRef<super::Message>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Message {
        const NAME: &'static str = "BustleMessage";
        type Type = super::Message;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Message {}
}

glib::wrapper! {
     pub struct Message(ObjectSubclass<imp::Message>);
}

impl Message {
    pub fn from_event(event: Event) -> Self {
        let this = glib::Object::builder::<Self>()
            .property(
                "message-type",
                MessageType::from(event.message.message_type()),
            )
            .property("timestamp", event.timestamp)
            .build();

        this.imp().inner.set(event.message).unwrap();

        this
    }

    pub fn to_event(&self) -> Event {
        Event {
            message: self.imp().inner.get().unwrap().clone(),
            timestamp: self.timestamp(),
        }
    }

    fn inner(&self) -> &zbus::Message {
        self.imp().inner.get().unwrap()
    }

    /// Return the length of the whole message
    pub fn len(&self) -> usize {
        self.inner().data().len()
    }

    /// Return the message body
    pub fn body(&self) -> zbus::message::Body {
        self.inner().body()
    }

    /// Return the message header
    pub fn header(&self) -> zbus::message::Header<'_> {
        self.inner().header()
    }

    /// Return the time it took to receive a response or None if it is a signal
    /// or no message is associated to the current one
    pub fn response_time(&self) -> Option<Timestamp> {
        let Some(associated_message) = self.associated_message() else {
            return None;
        };
        match self.message_type() {
            MessageType::Signal => None,
            MessageType::MethodReturn | MessageType::Error => {
                Some(self.timestamp() - associated_message.timestamp())
            }
            MessageType::MethodCall => Some(associated_message.timestamp() - self.timestamp()),
        }
    }

    pub fn set_receive_index(&self, raw_receive_index: u32) {
        self.imp()
            .receive_index
            .set(ReceiveIndex(raw_receive_index))
            .expect("receive index must only be set once");
    }

    pub fn receive_index(&self) -> ReceiveIndex {
        *self.imp().receive_index.get().unwrap()
    }

    pub fn set_associated_message(&self, message: &Self) {
        self.imp()
            .associated_message
            .set(message.downgrade())
            .expect("associated message must not be set twice");
    }

    pub fn associated_message(&self) -> Option<Self> {
        self.imp()
            .associated_message
            .get()
            .and_then(|m| m.upgrade())
    }

    pub fn sender_display(&self) -> String {
        if self.message_type().is_method_return() {
            self.associated_message()
                .as_ref()
                .and_then(|m| m.destination())
                .map(|d| d.to_string())
        } else {
            self.sender().map(|m| m.to_string())
        }
        .unwrap_or_else(|| "(no sender)".to_owned())
    }

    pub fn member_display(&self) -> String {
        if self.message_type().is_method_return() {
            self.associated_message()
                .as_ref()
                .map(|m| m.header())
                .and_then(|h| h.member().map(|m| m.to_string()))
        } else {
            self.header().member().map(|m| m.to_string())
        }
        .unwrap_or_else(|| "(no member)".to_owned())
    }

    pub fn interface_display(&self) -> String {
        if self.message_type().is_method_return() {
            self.associated_message()
                .as_ref()
                .map(|m| m.header())
                .and_then(|h| h.interface().map(|m| m.to_string()))
        } else {
            self.header().interface().map(|s| s.to_string())
        }
        .unwrap_or_else(|| "(no interface)".to_owned())
    }

    pub fn destination_display(&self) -> String {
        if self.message_type().is_method_return() {
            self.associated_message()
                .as_ref()
                .and_then(|m| m.sender())
                .map(|s| s.to_string())
        } else {
            self.destination().map(|m| m.to_string())
        }
        .unwrap_or_else(|| "(no destination)".to_owned())
    }

    pub fn path_display(&self) -> String {
        if self.message_type().is_method_return() {
            self.associated_message()
                .as_ref()
                .and_then(|m| m.header().path().map(|p| p.to_string()))
        } else {
            self.header().path().map(|p| p.to_string())
        }
        .unwrap_or_else(|| "(no path)".to_owned())
    }

    pub fn flags_display(&self) -> String {
        self.header()
            .primary()
            .flags()
            .iter()
            .map(|flag| match flag {
                zbus::MessageFlags::NoReplyExpected => gettext("No Reply Expected"),
                zbus::MessageFlags::NoAutoStart => gettext("No Auto Start"),
                zbus::MessageFlags::AllowInteractiveAuth => {
                    gettext("Allow Interactive Authentication")
                }
            })
            .collect::<Vec<_>>()
            .join(" | ")
    }

    pub fn member_markup(&self, render_error: bool) -> String {
        let interface = self.interface_display();
        let member = self.member_display();

        if self.message_type() == MessageType::Error && render_error {
            format!(
                "<span foreground='red'>{}.<b>{}</b></span>",
                glib::markup_escape_text(&interface),
                glib::markup_escape_text(&member)
            )
        } else {
            format!(
                "{}.<b>{}</b>",
                glib::markup_escape_text(&interface),
                glib::markup_escape_text(&member)
            )
        }
    }

    pub fn sender(&self) -> Option<UniqueName<'_>> {
        self.header().sender().cloned()
    }

    pub fn destination(&self) -> Option<BusName<'_>> {
        self.header().destination().cloned().or_else(|| {
            match self.message_type() {
                MessageType::MethodCall => Some("method.call.destination"),
                MessageType::MethodReturn => Some("method.return.destination"),
                MessageType::Error => Some("method.error.destination"),
                _ => None,
            }
            .map(|raw_name| {
                let ret = BusName::WellKnown(WellKnownName::from_static_str(raw_name).unwrap());
                debug_assert!(Self::is_fallback_destination(&ret));
                ret
            })
        })
    }

    /// Whether `name` is used as a fallback destination internally
    pub fn is_fallback_destination(name: &BusName<'_>) -> bool {
        match name {
            BusName::Unique(_) => false,
            BusName::WellKnown(wk_name) => matches!(
                wk_name.as_str(),
                "method.call.destination"
                    | "method.return.destination"
                    | "method.error.destination"
            ),
        }
    }

    /// Whether this is the return/error message of `other`
    pub fn is_return_of(&self, other: &Self) -> bool {
        debug_assert!(self.message_type().is_method_return());

        other.message_type().is_method_call()
            && self.header().reply_serial() == Some(other.header().primary().serial_num())
            && self.destination() == other.sender().map(From::from)
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Message")
            .field("path", &self.path_display())
            .field("destination", &self.destination_display())
            .field("sender", &self.sender_display())
            .field("member", &self.member_display())
            .field("interface", &self.interface_display())
            .field("type", &self.message_type())
            .field("associated_message", &self.associated_message())
            .finish()
    }
}
