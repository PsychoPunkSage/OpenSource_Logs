use adw::{prelude::*, subclass::prelude::*};
use gtk::glib;

pub struct Cancelled;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/freedesktop/Bustle/ui/address_dialog.ui")]
    pub struct AddressDialog {
        #[template_child]
        pub(super) entry_row: TemplateChild<adw::EntryRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AddressDialog {
        const NAME: &'static str = "BustleAddressDialog";
        type Type = super::AddressDialog;
        type ParentType = adw::AlertDialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AddressDialog {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().update_record_response_enabled();
            self.entry_row.grab_focus();
        }
    }

    impl WidgetImpl for AddressDialog {}
    impl AdwDialogImpl for AddressDialog {}
    impl AdwAlertDialogImpl for AddressDialog {}
}

glib::wrapper! {
    pub struct AddressDialog(ObjectSubclass<imp::AddressDialog>)
        @extends gtk::Widget, adw::Dialog, adw::AlertDialog;
}

#[gtk::template_callbacks]
impl AddressDialog {
    pub async fn choose(parent: &impl IsA<gtk::Widget>) -> Result<zbus::Address, Cancelled> {
        let this = glib::Object::new::<Self>();
        let entry_row = this.imp().entry_row.get();

        match this.choose_future(parent).await.as_str() {
            "cancel" => Err(Cancelled),
            "record" => Ok(entry_row
                .text()
                .parse()
                .expect("address must have been validated")),
            response_id => unreachable!("unexpected response id `{}`", response_id),
        }
    }

    #[template_callback]
    fn update_record_response_enabled(&self) {
        self.set_response_enabled(
            "record",
            self.imp().entry_row.text().parse::<zbus::Address>().is_ok(),
        );
    }
}
