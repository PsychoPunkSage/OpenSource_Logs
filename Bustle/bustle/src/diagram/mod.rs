mod header;
mod row;
mod row_tag;
mod view;

use anyhow::Result;
use gtk::{
    gdk,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};

use crate::{
    diagram::{header::Header, view::View},
    filtered_message_model::FilteredMessageModel,
    message::Message,
};

const COLUMN_WIDTH_SP: f64 = 90.0;

// This must be synced with DiagramRow label char widths
const FIRST_COLUMN_INITIAL_X_SP: f64 = 600.0;

mod imp {
    use std::{
        cell::{Cell, OnceCell},
        marker::PhantomData,
    };

    use super::*;

    #[derive(Debug, Default, glib::Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::Diagram)]
    #[template(resource = "/org/freedesktop/Bustle/ui/diagram.ui")]
    pub struct Diagram {
        #[property(get, set, construct_only)]
        pub(super) model: OnceCell<FilteredMessageModel>,
        #[property(get = Self::selected_message)]
        pub(super) selected_message: PhantomData<Option<Message>>,

        #[template_child]
        pub(super) header: TemplateChild<Header>,
        #[template_child]
        pub(super) separator: TemplateChild<gtk::Separator>, // Unused, but needed for disposal
        #[template_child]
        pub(super) scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub(super) view: TemplateChild<View>,

        pub(super) should_stick: Cell<bool>,
        pub(super) is_sticky: Cell<bool>,
        pub(super) is_auto_scrolling: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Diagram {
        const NAME: &'static str = "BustleDiagram";
        type Type = super::Diagram;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();

            klass.add_binding(gdk::Key::Escape, gdk::ModifierType::empty(), |obj| {
                obj.imp().view.unselect();
                glib::Propagation::Stop
            })
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for Diagram {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            let model = obj.model();
            self.view.set_model(&model);
            self.header.set_model(model.filtered_bus_names().clone());

            let hadj = self.view.hadjustment().unwrap();
            hadj.connect_value_changed(clone!(@weak obj => move |_| {
                obj.update_first_column_x();
            }));

            let vadj = self.view.vadjustment().unwrap();
            vadj.connect_value_changed(clone!(@weak obj => move |vadj| {
                let imp = obj.imp();

                if imp.should_stick.get() {
                    let is_at_bottom = vadj.value() + vadj.page_size() == vadj.upper();
                    if imp.is_auto_scrolling.get() {
                        if is_at_bottom {
                            imp.is_auto_scrolling.set(false);
                            imp.is_sticky.set(true);
                        } else {
                            obj.scroll_to_end();
                        }
                    } else {
                        imp.is_sticky.set(is_at_bottom);
                    }
                }
            }));
            vadj.connect_upper_notify(clone!(@weak obj => move |_| {
                let imp = obj.imp();
                if imp.should_stick.get() && imp.is_sticky.get() {
                    obj.scroll_to_end();
                }
            }));
            vadj.connect_page_size_notify(clone!(@weak obj => move |_| {
                let imp = obj.imp();
                if imp.should_stick.get() && imp.is_sticky.get() {
                    obj.scroll_to_end();
                }
            }));

            obj.settings()
                .connect_gtk_xft_dpi_notify(clone!(@weak obj => move |_| {
                    obj.update_column_width_and_first_column_initial_x();
                    obj.update_first_column_x();
                }));

            obj.update_column_width_and_first_column_initial_x();
            obj.update_first_column_x();
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for Diagram {}

    impl Diagram {
        fn selected_message(&self) -> Option<Message> {
            self.view.selected_message()
        }
    }
}

glib::wrapper! {
    pub struct Diagram(ObjectSubclass<imp::Diagram>)
        @extends gtk::Widget;
}

impl Diagram {
    /// Set whether to stick the view to the newest messages.
    pub fn set_should_stick(&self, should_stick: bool) {
        self.imp().should_stick.set(should_stick);
    }

    pub fn scroll_to(&self, message: &Message, flags: gtk::ListScrollFlags) -> Result<()> {
        self.imp().view.scroll_to(message, flags)
    }

    /// Returns the x coordinate where the first column should be drawn.
    fn first_column_initial_x(&self) -> f32 {
        adw::LengthUnit::Sp.to_px(FIRST_COLUMN_INITIAL_X_SP, Some(&self.settings())) as f32
    }

    fn scroll_to_end(&self) {
        let imp = self.imp();
        imp.is_auto_scrolling.set(true);
        imp.scrolled_window
            .emit_scroll_child(gtk::ScrollType::End, false);
    }

    /// This must be called when text scale factor changes
    fn update_column_width_and_first_column_initial_x(&self) {
        let imp = self.imp();

        let column_width =
            adw::LengthUnit::Sp.to_px(COLUMN_WIDTH_SP, Some(&self.settings())) as f32;

        imp.header.set_column_width(column_width);
        imp.view.set_column_width(column_width);

        imp.view
            .set_first_column_initial_x(self.first_column_initial_x());
    }

    /// This must be called when either:
    /// a. the view's horizontal adjustment changes
    /// b. the text scale factor changes
    fn update_first_column_x(&self) {
        let imp = self.imp();

        let first_column_x = self.first_column_initial_x()
            - self
                .imp()
                .view
                .hadjustment()
                .map_or(0.0, |hadj| hadj.value() as f32);
        imp.header.set_first_column_x(first_column_x);
        imp.view.set_first_column_x(first_column_x);
    }
}

#[gtk::template_callbacks]
impl Diagram {
    #[template_callback]
    fn view_selected_message_notify(&self) {
        self.notify_selected_message();
    }
}
