// Import necessary modules from gtk-rs crate
use gtk::{gdk, glib, graphene, prelude::*, subclass::prelude::*};

// Define a constant SIZE with value 16
const SIZE: i32 = 16;

// Define a module called imp for internal implementation details
mod imp {
    // Import Cell for interior mutability
    use std::cell::Cell;

    // Import structs and traits from parent module
    use super::*;

    // Define ColorWidget struct with gdk::RGBA property
    #[derive(Debug, glib::Properties)]
    #[properties(wrapper_type = super::ColorWidget)]
    pub struct ColorWidget {
        #[property(get, set = Self::set_rgba, explicit_notify)]
        pub(super) rgba: Cell<gdk::RGBA>,
    }

    // Implement default method for ColorWidget
    impl Default for ColorWidget {
        // Initialize rgba property with TRANSPARENT
        fn default() -> Self {
            Self {
                rgba: Cell::new(gdk::RGBA::TRANSPARENT),
            }
        }
    }

    // Implement object subclass for ColorWidget
    #[glib::object_subclass]
    impl ObjectSubclass for ColorWidget {
        // Define the name of the subclass
        const NAME: &'static str = "BustleColorWidget";
        // Define the type of the subclass
        type Type = super::ColorWidget;
        // Define the parent type of the subclass
        type ParentType = gtk::Widget;

        // Define class initialization method
        fn class_init(klass: &mut Self::Class) {
            // Set CSS name for the class
            klass.set_css_name("colorwidget");
        }
    }

    // Implement derived properties for ColorWidget
    #[glib::derived_properties]
    impl ObjectImpl for ColorWidget {
        // Define constructed method
        fn constructed(&self) {
            // Call parent's constructed method
            self.parent_constructed();
            // Set overflow property to Hidden
            self.obj().set_overflow(gtk::Overflow::Hidden);
        }
    }

    // Implement WidgetImpl for ColorWidget
    impl WidgetImpl for ColorWidget {
        // Define measure method
        fn measure(&self, _orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            // Return tuple with dimensions of the widget
            (SIZE, SIZE, -1, -1)
        }

        // Define snapshot method
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            // Get RGBA color of the widget
            let widget = self.obj();
            let color = widget.rgba();
            // Get width and height of the widget
            let width = widget.width() as f32;
            let height = widget.height() as f32;
            // Append color and rectangle to snapshot
            snapshot.append_color(&color, &graphene::Rect::new(0.0, 0.0, width, height));
        }
    }

    // Implement methods for ColorWidget
    impl ColorWidget {
        // Define set_rgba method to set RGBA color
        fn set_rgba(&self, rgba: gdk::RGBA) {
            let obj = self.obj();

            // Check if RGBA color is unchanged
            if self.rgba.get() == rgba {
                return;
            }

            // Set new RGBA color
            self.rgba.set(rgba);
            // Queue draw operation
            obj.queue_draw();
            // Notify about RGBA change
            obj.notify_rgba();
        }
    }
}

// Define ColorWidget struct using glib wrapper macro
glib::wrapper! {
    pub struct ColorWidget(ObjectSubclass<imp::ColorWidget>)
        @extends gtk::Widget;
}
