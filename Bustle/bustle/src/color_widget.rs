use gtk::{gdk, glib, graphene, prelude::*, subclass::prelude::*};

const SIZE: i32 = 16;

mod imp {
    use std::cell::Cell;

    use super::*;

    #[derive(Debug, glib::Properties)]
    #[properties(wrapper_type = super::ColorWidget)]
    pub struct ColorWidget {
        #[property(get, set = Self::set_rgba, explicit_notify)]
        pub(super) rgba: Cell<gdk::RGBA>,
    }

    impl Default for ColorWidget {
        fn default() -> Self {
            Self {
                rgba: Cell::new(gdk::RGBA::TRANSPARENT),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ColorWidget {
        const NAME: &'static str = "BustleColorWidget";
        type Type = super::ColorWidget;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("colorwidget");
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for ColorWidget {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().set_overflow(gtk::Overflow::Hidden);
        }
    }

    impl WidgetImpl for ColorWidget {
        fn measure(&self, _orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            (SIZE, SIZE, -1, -1)
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let color = widget.rgba();
            let width = widget.width() as f32;
            let height = widget.height() as f32;
            snapshot.append_color(&color, &graphene::Rect::new(0.0, 0.0, width, height));
        }
    }

    impl ColorWidget {
        fn set_rgba(&self, rgba: gdk::RGBA) {
            let obj = self.obj();

            if self.rgba.get() == rgba {
                return;
            }

            self.rgba.set(rgba);
            obj.queue_draw();
            obj.notify_rgba();
        }
    }
}

glib::wrapper! {
    pub struct ColorWidget(ObjectSubclass<imp::ColorWidget>)
        @extends gtk::Widget;
}
