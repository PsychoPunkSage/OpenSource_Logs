use std::{
    borrow::Cow,
    collections::{hash_map::Entry, HashMap},
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use anyhow::{bail, Context, Ok, Result};
use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use pcap_file::pcap::{PcapHeader, PcapPacket, PcapReader, PcapWriter};

use crate::{
    bus_name_list::BusNameList, message::Message, message_tag::MessageTag, monitor::Event, RUNTIME,
};

mod imp {
    use std::cell::RefCell;

    // Import the items from the parent module (`super`), which in this case is the outer module
    // where `imp` is defined.
    use super::*;

    // Define a new struct named `MessageList`.
    #[derive(Debug, Default)]
    pub struct MessageList {
        // Define a field named `inner` of type `RefCell<Vec<Message>>`. `RefCell` allows interior
        // mutability, enabling shared mutable access to the vector of `Message` instances.
        pub(super) inner: RefCell<Vec<Message>>,
        pub(super) bus_names: BusNameList,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageList {
        const NAME: &'static str = "BustleMessageList";
        type Type = super::MessageList;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for MessageList {}

    impl ListModelImpl for MessageList {
        // Define a method to retrieve the type of items in the list model.
        fn item_type(&self) -> glib::Type {
            Message::static_type()
        }

        // Define a method to get the number of items in the list model.
        fn n_items(&self) -> u32 {
            self.inner.borrow().len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            // Borrow the inner vector, attempt to get the item at the specified position, map
            // it to an `glib::Object`, and then clone it.
            self.inner
                .borrow()
                .get(position as usize)
                .map(|item| item.upcast_ref())
                .cloned()
        }
    }
}

glib::wrapper! {
    pub struct MessageList(ObjectSubclass<imp::MessageList>)
        @implements gio::ListModel;
}

impl MessageList {
    pub async fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        // Retrieve events from the inner model and convert them to `Event` instances.
        let events = self
            .imp()
            .inner
            .borrow()
            .iter()
            .map(|message| message.to_event())
            .collect::<Vec<_>>();

        let path = path.as_ref().to_owned();

        // Asynchronously spawn a blocking task to write events to the file.
        RUNTIME
            .spawn_blocking(move || {
                // Create a PCAP header with default values.
                let header = PcapHeader {
                    datalink: pcap_file::DataLink::DBUS,
                    ..Default::default()
                };

                // Create the file and initialize a PCAP writer
                let file = File::create(path).context("Failed to create file")?;
                let mut writer = PcapWriter::with_header(BufWriter::new(file), header)
                    .context("Failed to create writer")?;

                // Write each event as a PCAP packet to the file.
                for event in events {
                    let message_bytes = event.message.data();
                    writer
                        .write_packet(&PcapPacket {
                            timestamp: event.timestamp.into(),
                            orig_len: message_bytes.len() as u32,
                            data: Cow::Borrowed(message_bytes),
                        })
                        .context("Failed to write packet")?;
                }

                Ok(())
            })
            .await
            .context("Failed to spawn blocking task")?
            .context("Failed to save to file")
    }

    pub async fn save_as_dot(&self, dest: &gio::File) -> Result<()> {
        // Create a buffer to store DOT data
        let mut buffer = Vec::new();
        // Create a HashMap to track unique combinations of sender and destination
        let mut combinations = HashMap::new();

        // Write the start of the DOT file
        writeln!(&mut buffer, "digraph bustle {{")?;
        // Iterate over each message in the inner list
        for message in self.imp().inner.borrow().iter() {
            // Get the sender and destination display names from the message
            let sender = message.sender_display();
            let destination = message.destination_display();
            // Check if the combination of sender and destination has been encountered before
            match combinations.entry((sender, destination)) {
                // If the combination exists, skip to the next iteration
                Entry::Occupied(_) => continue,
                // If the combination is new, add it to the HashMap and write it to the buffer
                Entry::Vacant(entry) => {
                    let (sender, destination) = entry.key();
                    writeln!(&mut buffer, "\t\"{sender}\" -> \"{destination}\";")?;
                    entry.insert(());
                }
            }
        }
        // Write the end of the DOT file
        writeln!(&mut buffer, "}}")?;

        // Replace the contents of the destination file with the DOT data
        dest.replace_contents_future(
            buffer,
            None,
            false,
            gio::FileCreateFlags::REPLACE_DESTINATION,
        )
        .await
        .map_err(|e| e.1)?;

        Ok(())
    }

    pub async fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        // Convert the path to an owned PathBuf
        let path = path.as_ref().to_owned();
        // Spawn a blocking task to read the file and parse its contents
        let events = RUNTIME
            .spawn_blocking(move || {
                // Open the file
                let file = File::open(&path)
                    .with_context(|| format!("Failed to open file at `{}`", path.display()))?;
                // Create a reader for the pcap file
                let mut reader = PcapReader::new(file).context("Failed to create reader")?;

                // Read the header of the pcap file
                let header = reader.header();
                // Check if the datalink type is compatible with dbus
                if header.datalink != pcap_file::DataLink::DBUS {
                    bail!("Invalid datalink type `{:?}`", header.datalink)
                }

                tracing::debug!(?path, ?header, "Loaded PCAP file");

                let mut events = Vec::new();
                // Iterate over each packet in the pcap file
                while let Some(packet) = reader.next_packet() {
                    // Parse the packet into an event
                    let packet = packet.context("Failed to get packet")?;
                    let event = Event::from_packet(packet)
                        .context("Failed to construct event from packet")?;
                    events.push(event);
                }

                Ok(events)
            })
            .await
            .context("Failed to join handle")?
            .context("Failed to load from file")?;

        let this = Self::default();

        for event in events {
            this.push_inner(Message::from_event(event));
        }

        Ok(this)
    }

    pub fn push(&self, message: Message) {
        self.push_inner(message);
        self.items_changed(self.n_items() - 1, 0, 1);
    }

    pub fn bus_names(&self) -> &BusNameList {
        &self.imp().bus_names
    }

    fn push_inner(&self, message: Message) {
        // Get the current position (index) of the message in the model.
        let position = self.n_items();

        // Set the receive index of the message to its position in the model.
        message.set_receive_index(position);

        // Get a reference to the implementation of the filtered message model.
        let imp = self.imp();

        if message.message_type().is_method_return() {
            // Reverse so we first look at the most recent call. This speeds up the search
            // substantially in the common case where the return is close to the call.
            if let Some(associated_message) = imp
                .inner
                .borrow()
                .iter()
                .rev()
                .find(|other_message| message.is_return_of(other_message))
            {
                message.set_associated_message(associated_message);
                associated_message.set_associated_message(&message);
            }
        }

        // Only handle message when we have its associated message
        if let Err(err) = imp.bus_names.handle_message(&message) {
            tracing::warn!(%message, "Failed to handle message: {:?}", err);
        }

        // Only try to guess the component once we have an associated message
        let message_tag = MessageTag::guess(&message);
        message.set_message_tag(message_tag);

        imp.inner.borrow_mut().push(message);
    }
}

impl Default for MessageList {
    fn default() -> Self {
        glib::Object::new()
    }
}
