// Import necessary traits and modules from adw and gtk crates
use adw::{prelude::*, subclass::prelude::*};
use gtk::glib;

// Define a custom struct to represent the Cancelled error
pub struct Cancelled;

// Define an inner module for implementation details
mod imp {
    use super::*;

    // Define the AddressDialog struct with CompositeTemplate trait
    #[derive(Default, gtk::CompositeTemplate)]
    // Specify the UI template resource file
    #[template(resource = "/org/freedesktop/Bustle/ui/address_dialog.ui")]
    pub struct AddressDialog {
        // Define a template child entry_row of type adw::EntryRow
        #[template_child]
        pub(super) entry_row: TemplateChild<adw::EntryRow>,
    }

    // Implement the ObjectSubclass trait for AddressDialog
    #[glib::object_subclass]
    impl ObjectSubclass for AddressDialog {
        // Specify the type name and parent type
        const NAME: &'static str = "BustleAddressDialog";
        type Type = super::AddressDialog;
        type ParentType = adw::AlertDialog;

        // Initialize the class
        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        // Initialize the instance
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // Implement ObjectImpl for AddressDialog
    impl ObjectImpl for AddressDialog {
        // Define the constructed method
        fn constructed(&self) {
            // Call the parent_constructed method
            self.parent_constructed();

            // Update the record response enabled status
            self.obj().update_record_response_enabled();
            // Set focus on the entry_row
            self.entry_row.grab_focus();
        }
    }

    // Implement WidgetImpl for AddressDialog
    impl WidgetImpl for AddressDialog {}
    // Implement AdwDialogImpl for AddressDialog
    impl AdwDialogImpl for AddressDialog {}
    // Implement AdwAlertDialogImpl for AddressDialog
    impl AdwAlertDialogImpl for AddressDialog {}
}

// Define a wrapper struct for AddressDialog using ObjectSubclass
glib::wrapper! {
    pub struct AddressDialog(ObjectSubclass<imp::AddressDialog>)
        @extends gtk::Widget, adw::Dialog, adw::AlertDialog;
}

// Implement template callbacks for AddressDialog
#[gtk::template_callbacks]
impl AddressDialog {
    // Define an asynchronous method to choose an address
    pub async fn choose(parent: &impl IsA<gtk::Widget>) -> Result<zbus::Address, Cancelled> {
        // Create a new instance of AddressDialog
        let this = glib::Object::new::<Self>();
        // Get the entry_row from the implementation
        let entry_row = this.imp().entry_row.get();

        // Await the future result of choose_future method
        match this.choose_future(parent).await.as_str() {
            // If response is "cancel", return Cancelled error
            "cancel" => Err(Cancelled),
            // If response is "record", parse the address and return
            "record" => Ok(entry_row
                .text()
                .parse()
                .expect("address must have been validated")),
            // Otherwise, panic with unexpected response id
            response_id => unreachable!("unexpected response id `{}`", response_id),
        }
    }

    // Define a callback to update the record response enabled status
    #[template_callback]
    fn update_record_response_enabled(&self) {
        // Set the response enabled status based on entry_row text validity
        self.set_response_enabled(
            "record",
            self.imp().entry_row.text().parse::<zbus::Address>().is_ok(),
        );
    }
}
