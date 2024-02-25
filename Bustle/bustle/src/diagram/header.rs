use adw::prelude::*;
use gtk::{
    glib::{self, clone},
    graphene::Point,
    pango,
    subclass::prelude::*,
};
use zbus::names::BusName;

use crate::{
    bus_name_item::{BusNameItem, LookupPoint},
    filtered_bus_name_model::FilteredBusNameModel,
};

const TEXT_PADDING: i32 = 3;
const EXPAND_ANIMATION_DURATION_MS: u32 = 200;

mod imp {
    use std::cell::{Cell, OnceCell, RefCell};

    use super::*;

    #[derive(Default, glib::Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::Header)]
    #[template(resource = "/org/freedesktop/Bustle/ui/diagram_header.ui")]
    pub struct Header {
        #[property(get, set = Self::set_column_width, explicit_notify)]
        pub(super) column_width: Cell<f32>,
        #[property(get, set = Self::set_first_column_x, explicit_notify)]
        pub(super) first_column_x: Cell<f32>,

        #[template_child]
        pub(super) arrow: TemplateChild<gtk::Image>,

        pub(super) default_height: Cell<i32>,
        pub(super) expanded_height: Cell<i32>,
        pub(super) current_height: Cell<i32>,

        pub(super) is_expanded: Cell<bool>,

        pub(super) expand_animation: OnceCell<adw::TimedAnimation>,

        pub(super) default_layout: OnceCell<pango::Layout>,
        pub(super) layouts: RefCell<Vec<pango::Layout>>,

        pub(super) model: OnceCell<FilteredBusNameModel>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Header {
        const NAME: &'static str = "BustleDiagramHeader";
        type Type = super::Header;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for Header {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.set_direction(gtk::TextDirection::Ltr);

            let animation_target =
                adw::CallbackAnimationTarget::new(clone!(@weak obj => move |value| {
                    obj.imp().current_height.set(value.round() as i32);
                    obj.queue_resize();
                }));
            let animation = adw::TimedAnimation::builder()
                .widget(&*obj)
                .duration(EXPAND_ANIMATION_DURATION_MS)
                .target(&animation_target)
                .build();
            animation.connect_done(clone!(@weak obj => move |_| {
                let imp = obj.imp();
                imp.is_expanded.set(!imp.is_expanded.get());
            }));
            self.expand_animation.set(animation).unwrap();

            let default_layout = obj.create_pango_layout(None);
            default_layout.set_alignment(pango::Alignment::Center);
            default_layout.set_ellipsize(pango::EllipsizeMode::End);
            self.default_layout.set(default_layout).unwrap();

            obj.settings()
                .connect_gtk_xft_dpi_notify(clone!(@weak obj => move |_| {
                    obj.update_heights();
                    obj.queue_resize();
                    obj.queue_draw();
                }));
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for Header {
        fn measure(&self, orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            if orientation == gtk::Orientation::Vertical {
                let height = self.current_height.get();
                (height, height, -1, -1)
            } else {
                debug_assert_eq!(orientation, gtk::Orientation::Horizontal);
                (-1, -1, -1, -1)
            }
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.arrow
                .size_allocate(&gtk::Allocation::new(0, 0, width, height), baseline);
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let obj = self.obj();

            let color = obj.color();
            let column_width = obj.column_width();

            let mut cursor_x = obj.first_column_x() - column_width / 2.0;
            for layout in self.layouts.borrow().iter() {
                snapshot.save();
                snapshot.translate(&Point::new(cursor_x, TEXT_PADDING as f32));
                snapshot.append_layout(layout, &color);
                snapshot.restore();
                cursor_x += column_width;
            }

            self.parent_snapshot(snapshot);
        }
    }

    impl Header {
        fn set_column_width(&self, column_width: f32) {
            if column_width == self.column_width.get() {
                return;
            }

            self.column_width.set(column_width);

            for layout in self.layouts.borrow().iter() {
                layout.set_width(pango::units_from_double(column_width as f64));
            }

            let obj = self.obj();
            obj.queue_draw();
            obj.notify_column_width();
        }

        fn set_first_column_x(&self, first_column_x: f32) {
            if first_column_x == self.first_column_x.get() {
                return;
            }

            self.first_column_x.set(first_column_x);

            let obj = self.obj();
            obj.queue_draw();
            obj.notify_first_column_x();
        }
    }
}

glib::wrapper! {
    pub struct Header(ObjectSubclass<imp::Header>)
        @extends gtk::Widget;
}

impl Header {
    pub fn set_model(&self, model: FilteredBusNameModel) {
        let imp = self.imp();

        model.connect_items_changed(
            clone!(@weak self as obj => move |model, position, removed, added| {
                obj.on_model_items_changed(model, position, removed, added);
            }),
        );

        imp.model.set(model).unwrap();
    }

    fn on_model_items_changed(
        &self,
        model: &FilteredBusNameModel,
        position: u32,
        removed: u32,
        added: u32,
    ) {
        let imp = self.imp();

        let default_layout = imp
            .default_layout
            .get()
            .expect("default layout was not set");
        let column_width = self.column_width();

        let new_layouts = (0..added).map(|i| {
            let bus_name_item = model
                .item(position + i)
                .unwrap()
                .downcast::<BusNameItem>()
                .unwrap();

            let name = bus_name_item.name();
            let name_display = match *name {
                BusName::Unique(ref unique_name) => unique_name.as_str(),
                BusName::WellKnown(ref wk_name) => wk_name.split('.').last().unwrap_or_default(),
            };
            let mut lines = vec![format!(
                r#"<b><span size="small">{}</span></b>"#,
                name_display
            )];
            lines.extend(
                bus_name_item
                    // FIXME We don't always want this as some well-known names are not valid at all indices
                    // We want to somehow represent which well-known names are valid as we scroll on the view
                    .wk_names(LookupPoint::All)
                    .iter()
                    .filter_map(|other_name| {
                        other_name
                            .split('.')
                            .last()
                            .map(|last| format!(r#"<span size="x-small">{}</span>"#, last))
                    }),
            );
            let text = lines.join("\n");

            let layout = default_layout.copy();
            layout.set_width(pango::units_from_double(column_width as f64));
            layout.set_markup(&text);
            layout
        });
        imp.layouts.borrow_mut().splice(
            position as usize..(removed + position) as usize,
            new_layouts,
        );

        debug_assert_eq!(imp.layouts.borrow().len(), model.n_items() as usize);

        self.update_heights();
        self.queue_resize();
        self.queue_draw();
    }

    fn update_heights(&self) {
        let imp = self.imp();

        let layouts = imp.layouts.borrow();

        let Some(first_layout) = layouts.first() else {
            imp.current_height.set(0);
            imp.default_height.set(0);
            imp.expanded_height.set(0);
            return;
        };

        let first_line = first_layout
            .downcast_ref::<pango::Layout>()
            .unwrap()
            .line(0)
            .expect("there must be at least one line");
        let (_, logical_extents) = first_line.pixel_extents();

        let default_height = logical_extents.height() + TEXT_PADDING;
        imp.default_height.set(default_height);

        let max_line_count = layouts
            .iter()
            .map(|layout| layout.line_count())
            .max()
            .unwrap();
        debug_assert!(max_line_count > 0);

        // Since somehow a line's height extent can be greater than the entire layout's
        // height extent, workaround it by just using default height if the maximum
        // amount of line is 1.
        let expanded_height = if max_line_count == 1 {
            default_height
        } else {
            layouts
                .iter()
                .map(|layout| {
                    let (_, logical_extents) = layout.pixel_extents();
                    logical_extents.height()
                })
                .max()
                .unwrap()
                + TEXT_PADDING * 2
        };
        imp.expanded_height.set(expanded_height);

        if imp.is_expanded.get() {
            imp.current_height.set(expanded_height);
        } else {
            imp.current_height.set(default_height);
        }
    }
}

#[gtk::template_callbacks]
impl Header {
    #[template_callback]
    fn gesture_click_released(&self, _n_press: i32, x: f64, y: f64, gesture: &gtk::GestureClick) {
        gesture.set_state(gtk::EventSequenceState::Claimed);

        if !gesture.widget().contains(x, y) {
            return;
        }

        let imp = self.imp();

        let animation = imp
            .expand_animation
            .get()
            .expect("animation was not set on constructed");

        if animation.state() == adw::AnimationState::Playing {
            return;
        }

        if imp.is_expanded.get() {
            animation.set_value_from(imp.expanded_height.get() as f64);
            animation.set_value_to(imp.default_height.get() as f64);

            imp.arrow.remove_css_class("expanded");
            imp.arrow.remove_css_class("accent");
            imp.arrow.add_css_class("dim-label");
        } else {
            animation.set_value_from(imp.default_height.get() as f64);
            animation.set_value_to(imp.expanded_height.get() as f64);

            imp.arrow.remove_css_class("dim-label");
            imp.arrow.add_css_class("accent");
            imp.arrow.add_css_class("expanded");
        }

        animation.play();
    }
}
