#![warn(rust_2018_idioms)]

mod address_dialog;
mod application;
#[rustfmt::skip]
mod config;
mod bus_name_item;
mod bus_name_list;
mod color_widget;
mod colors;
mod details_view;
mod diagram;
mod filter_pane;
mod filtered_bus_name_model;
mod filtered_message_model;
mod i18n;
mod message;
mod message_list;
mod message_tag;
mod message_type;
mod monitor;
mod statistics;
mod timestamp;
mod window;

use gettextrs::LocaleCategory;
use gtk::{gio, glib};
use once_cell::sync::Lazy;

pub static RUNTIME: Lazy<tokio::runtime::Runtime> =
    Lazy::new(|| tokio::runtime::Runtime::new().unwrap());

use self::{
    application::Application,
    config::{GETTEXT_PACKAGE, LOCALEDIR, RESOURCES_FILE},
};

fn main() -> glib::ExitCode {
    // Initialize logger
    tracing_subscriber::fmt::init();

    // Prepare i18n
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    gettextrs::textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    glib::set_application_name("Bustle");

    let res = gio::Resource::load(RESOURCES_FILE).expect("Could not load gresource file");
    gio::resources_register(&res);

    let app = Application::default();
    app.run()
}
