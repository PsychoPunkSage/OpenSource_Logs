// Rust rewrite of <https://gitlab.gnome.org/GNOME/sysprof/-/blob/master/src/sysprof/sysprof-progress-cell.c>

use adw::subclass::prelude::*;
use gtk::{gdk, glib, graphene, prelude::*};

mod imp {
    use std::cell::{Cell, RefCell};

    use super::*;

    #[derive(Debug, glib::Properties)]
    #[properties(wrapper_type = super::ProgressCell)]
    pub struct ProgressCell {
        #[property(get, set = Self::set_fraction, minimum = 0.0, maximum = 1.0, default = 0.0)]
        pub(super) fraction: Cell<f32>,
        #[property(get, set = Self::set_text)]
        pub(super) text: RefCell<String>,

        pub(super) through: adw::Bin,
        pub(super) progress: adw::Bin,
        pub(super) label: gtk::Label,
        pub(super) alt_label: gtk::Label,
    }

    impl Default for ProgressCell {
        fn default() -> Self {
            Self {
                fraction: Default::default(),
                text: Default::default(),
                through: adw::Bin::builder().css_name("through").build(),
                progress: adw::Bin::builder()
                    .css_name("progress")
                    .visible(false)
                    .build(),
                label: gtk::Label::builder()
                    .halign(gtk::Align::Center)
                    .label("0")
                    .build(),
                alt_label: gtk::Label::builder()
                    .halign(gtk::Align::Center)
                    .label("0")
                    .build(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProgressCell {
        const NAME: &'static str = "BustleProgressCell";
        type Type = super::ProgressCell;
        type ParentType = gtk::Widget;
        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("progress-cell");
            klass.set_accessible_role(gtk::AccessibleRole::ProgressBar);
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for ProgressCell {
        fn constructed(&self) {
            self.parent_constructed();
            let widget = self.obj();

            self.alt_label.add_css_class("in-progress");
            self.label.add_css_class("numeric");
            self.alt_label.add_css_class("numeric");

            widget
                .bind_property("text", &self.label, "label")
                .sync_create()
                .build();
            widget
                .bind_property("text", &self.alt_label, "label")
                .sync_create()
                .build();

            self.through.set_parent(&*widget);
            self.label.set_parent(&*widget);
            self.progress.set_parent(&*widget);
            self.alt_label.set_parent(&*widget);

            widget.update_property(&[
                gtk::accessible::Property::ValueMax(1.0),
                gtk::accessible::Property::ValueMin(0.0),
                gtk::accessible::Property::ValueNow(0.0),
            ]);
        }

        fn dispose(&self) {
            self.through.unparent();
            self.progress.unparent();
            self.label.unparent();
            self.alt_label.unparent();
        }
    }

    impl WidgetImpl for ProgressCell {
        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            let widget = self.obj();

            let allocation = gdk::Rectangle::new(0, 0, width, height);
            self.label.size_allocate(&allocation, baseline);
            self.alt_label.size_allocate(&allocation, baseline);
            self.through.size_allocate(&allocation, baseline);

            if self.progress.get_visible() {
                self.progress.size_allocate(
                    &gdk::Rectangle::new(
                        0,
                        0,
                        (width as f32 * widget.fraction()).max(1.) as i32,
                        height,
                    ),
                    baseline,
                );
            }
        }

        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            let (mut minimum, mut natural) = (0, 0);
            let (minimum_baseline, natural_baseline) = (-1, -1);
            let widgets = [
                self.through.upcast_ref::<gtk::Widget>(),
                self.progress.upcast_ref::<gtk::Widget>(),
                self.label.upcast_ref::<gtk::Widget>(),
                self.alt_label.upcast_ref::<gtk::Widget>(),
            ];
            for widget in widgets.into_iter() {
                let (child_min, child_nat, _, _) = widget.measure(orientation, for_size);
                if child_min > minimum {
                    minimum = child_min;
                }
                if child_nat > natural {
                    natural = child_nat;
                }
            }
            (minimum, natural, minimum_baseline, natural_baseline)
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            widget.snapshot_child(&self.through, snapshot);
            widget.snapshot_child(&self.label, snapshot);
            widget.snapshot_child(&self.progress, snapshot);

            let w = widget.width();
            let h = widget.height();
            snapshot.push_clip(&graphene::Rect::new(
                0.,
                0.,
                w as f32 * widget.fraction(),
                h as f32,
            ));
            widget.snapshot_child(&self.alt_label, snapshot);
            snapshot.pop();
        }
    }

    impl ProgressCell {
        fn set_fraction(&self, mut fraction: f32) {
            fraction = fraction.clamp(0., 1.);
            let widget = self.obj();
            if self.fraction.get() != fraction {
                self.fraction.set(fraction);
                self.progress.set_visible(fraction > 0.);
                widget.notify_fraction();

                widget.update_property(&[gtk::accessible::Property::ValueNow(fraction.into())]);
                widget.queue_allocate();
            }
        }
        fn set_text(&self, text: String) {
            if self.text.replace(text.clone()) != text {
                self.text.set(text);
                self.obj().notify_text();
            }
        }
    }
}

glib::wrapper! {
     pub struct ProgressCell(ObjectSubclass<imp::ProgressCell>)
        @extends gtk::Widget, gtk::Accessible;
}
