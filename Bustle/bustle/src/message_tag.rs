use gettextrs::gettext;
use gtk::{gdk, glib};

use crate::message::Message;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, glib::Enum)]
#[enum_type(name = "BustleMessageTag")]
pub enum MessageTag {
    Accessibility,
    Bluetooth,
    Flatpak,
    Geoclue,
    Gtk,
    Gvfs,
    IBus,
    Logind,
    NetworkManager,
    PolicyKit,
    Portals,
    SearchProvider,
    SessionManager,
    Shell,
    Systemd,
    Tracker,
    UPower,
    #[default]
    Unknown,
}

impl MessageTag {
    pub fn guess(message: &Message) -> Self {
        let object_path = message.path_display();
        let interface = message.interface_display();
        if object_path.contains("/org/a11y/bus") || interface.contains("org.a11y.Bus") {
            Self::Accessibility
        } else if object_path.contains("/org/bluez") {
            Self::Bluetooth
        } else if object_path.contains("/org/freedesktop/Flatpak")
            || interface.contains("org.freedesktop.Flatpak")
        {
            Self::Flatpak
        } else if object_path.contains("/org/freedesktop/GeoClue") {
            Self::Geoclue
        } else if interface.contains("org.gtk.Actions") || object_path.contains("/org/gtk/Settings")
        {
            Self::Gtk
        } else if object_path.contains("/org/gtk/vfs")
            || object_path.contains("/org/gtk/Private/RemoteVolumeMonitor")
        {
            Self::Gvfs
        } else if object_path.contains("/org/freedesktop/IBus") {
            Self::IBus
        } else if object_path.contains("/org/freedesktop/login") {
            Self::Logind
        } else if object_path.contains("/org/freedesktop/NetworkManager")
            || object_path.contains("/fi/w1/wpa_supplicant")
        {
            Self::NetworkManager
        } else if object_path.contains("/org/freedesktop/PolicyKit") {
            Self::PolicyKit
        } else if object_path.contains("/org/freedesktop/portal")
            || object_path.contains("/org/freedesktop/impl/portal")
        {
            Self::Portals
        } else if interface.contains("org.gnome.Shell.SearchProvider") {
            Self::SearchProvider
        } else if object_path.contains("/org/gnome/SessionManager") {
            Self::SessionManager
        } else if object_path.contains("/org/gnome/Shell")
            || object_path.contains("/org/gnome/Mutter")
        {
            Self::Shell
        } else if object_path.contains("/org/freedesktop/systemd") {
            Self::Systemd
        } else if object_path.contains("/org/freedesktop/Tracker") {
            Self::Tracker
        } else if object_path.contains("/org/freedesktop/UPower") {
            Self::UPower
        } else {
            Self::Unknown
        }
    }

    pub fn color(self) -> gdk::RGBA {
        use crate::colors;
        match self {
            Self::Accessibility => colors::PURPLE_5,
            Self::Bluetooth => colors::RED_3,
            Self::Flatpak => colors::RED_5,
            Self::Geoclue => colors::PURPLE_3,
            Self::Gtk => colors::YELLOW_5,
            Self::Gvfs => colors::BLUE_5,
            Self::IBus => colors::YELLOW_4,
            Self::Logind => colors::BROWN_3,
            Self::NetworkManager => colors::YELLOW_1,
            Self::PolicyKit => colors::GREEN_4,
            Self::Portals => colors::RED_1,
            Self::SearchProvider => colors::BLUE_2,
            Self::SessionManager => colors::ORANGE_3,
            Self::Shell => gdk::RGBA::BLACK,
            Self::Systemd => colors::RED_2,
            Self::Tracker => colors::BROWN_1,
            Self::UPower => colors::ORANGE_5,
            Self::Unknown => gdk::RGBA::TRANSPARENT,
        }
    }

    pub fn name(self) -> String {
        match self {
            Self::Accessibility => gettext("Accessibility"),
            Self::Bluetooth => gettext("Bluetooth"),
            Self::Flatpak => gettext("Flatpak"),
            Self::Geoclue => gettext("Geolocalization"),
            Self::Gtk => gettext("GTK"),
            Self::Gvfs => gettext("GVFS"),
            Self::IBus => gettext("Input"),
            Self::Logind => gettext("Login"),
            Self::NetworkManager => gettext("Network"),
            Self::PolicyKit => gettext("Policy Control"),
            Self::Portals => gettext("Portals"),
            Self::SearchProvider => gettext("Search Provider"),
            Self::SessionManager => gettext("Session"),
            // TRANSLATORS As in the GNOME Shell
            Self::Shell => gettext("Shell"),
            Self::Systemd => gettext("systemd"),
            // TRANSLATORS As in the Tracker software. https://tracker.gnome.org/
            Self::Tracker => gettext("Tracker"),
            Self::UPower => gettext("Power"),
            Self::Unknown => gettext("Unknown"),
        }
    }
}
