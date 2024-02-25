use gettextrs::gettext;
use gtk::glib;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, glib::Enum)]
#[enum_type(name = "BustleMessageType")]
pub enum MessageType {
    MethodCall,
    MethodReturn,
    #[default]
    Error,
    Signal,
}

impl MessageType {
    /// Returns true if self is a method return or error
    pub fn is_method_return(self) -> bool {
        matches!(self, Self::MethodReturn | Self::Error)
    }

    pub fn is_error(self) -> bool {
        matches!(self, Self::Error)
    }

    pub fn is_signal(self) -> bool {
        matches!(self, Self::Signal)
    }

    pub fn is_method_call(self) -> bool {
        matches!(self, Self::MethodCall)
    }

    pub fn i18n(self) -> String {
        match self {
            Self::MethodCall => gettext("Method Call"),
            Self::MethodReturn => gettext("Method Return"),
            Self::Error => gettext("Error"),
            Self::Signal => gettext("Signal"),
        }
    }
}

impl From<zbus::MessageType> for MessageType {
    fn from(value: zbus::MessageType) -> Self {
        match value {
            zbus::MessageType::MethodCall => Self::MethodCall,
            zbus::MessageType::MethodReturn => Self::MethodReturn,
            zbus::MessageType::Error => Self::Error,
            zbus::MessageType::Signal => Self::Signal,
        }
    }
}

impl From<MessageType> for zbus::MessageType {
    fn from(value: MessageType) -> Self {
        match value {
            MessageType::MethodCall => Self::MethodCall,
            MessageType::MethodReturn => Self::MethodReturn,
            MessageType::Error => Self::Error,
            MessageType::Signal => Self::Signal,
        }
    }
}
