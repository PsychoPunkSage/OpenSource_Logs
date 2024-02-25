use anyhow::{Context, Result};
use gtk::{
    gdk,
    glib::{self, clone},
    graphene::{Point, Rect},
    gsk, pango,
    prelude::*,
    subclass::prelude::*,
};
use indexmap::IndexMap;
use zbus::names::BusName;

use crate::{
    diagram::row::Row,
    filtered_message_model::FilteredMessageModel,
    message::{Message, ReceiveIndex},
    message_type::MessageType,
};

const COLUMN_LINE_WIDTH: f32 = 1.0;
const ARROW_LINE_WIDTH: f32 = 2.0;

/// Determines on what side the arc will curve
enum ArcSide {
    Left,
    Right,
}

/// Determines the type of arrow tip
enum ArrowTipType {
    Full,
    TopHalf,
    BottomHalf,
}

mod imp {
    use std::{
        cell::{Cell, OnceCell, RefCell},
        marker::PhantomData,
    };

    use gtk::glib::WeakRef;

    use super::*;

    #[derive(Debug, Default, glib::Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::View)]
    #[template(resource = "/org/freedesktop/Bustle/ui/diagram_view.ui")]
    pub struct View {
        #[property(get = Self::selected_message)]
        pub(super) selected_message: PhantomData<Option<Message>>,
        #[property(get, set = Self::set_column_width, explicit_notify)]
        pub(super) column_width: Cell<f32>,
        #[property(get, set = Self::set_first_column_x, explicit_notify)]
        pub(super) first_column_x: Cell<f32>,
        #[property(get, set = Self::set_first_column_initial_x, explicit_notify)]
        pub(super) first_column_initial_x: Cell<f32>,

        #[property(get = Self::hscroll_policy, set = Self::set_hscroll_policy, override_interface = gtk::Scrollable)]
        pub(super) hscroll_policy: PhantomData<gtk::ScrollablePolicy>,
        #[property(get = Self::hadjustment, set = Self::set_hadjustment, override_interface = gtk::Scrollable)]
        pub(super) hadjustment: PhantomData<Option<gtk::Adjustment>>,
        #[property(get = Self::vscroll_policy, set = Self::set_vscroll_policy, override_interface = gtk::Scrollable)]
        pub(super) vscroll_policy: PhantomData<gtk::ScrollablePolicy>,
        #[property(get = Self::vadjustment, set = Self::set_vadjustment, override_interface = gtk::Scrollable)]
        pub(super) vadjustment: PhantomData<Option<gtk::Adjustment>>,

        #[template_child]
        pub(super) list_view: TemplateChild<gtk::ListView>,
        #[template_child]
        pub(super) selection_model: TemplateChild<gtk::SingleSelection>,

        pub(super) rows: RefCell<Vec<WeakRef<Row>>>,
        pub(super) row_width_request: Cell<i32>,
        pub(super) distance_between_rows_center: Cell<Option<f32>>,

        pub(super) response_time_layout: OnceCell<pango::Layout>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for View {
        const NAME: &'static str = "BustleDiagramView";
        type Type = super::View;
        type ParentType = gtk::Widget;
        type Interfaces = (gtk::Scrollable,);

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for View {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.set_direction(gtk::TextDirection::Ltr);

            let layout = obj.create_pango_layout(None);
            self.response_time_layout.set(layout).unwrap();

            obj.settings()
                .connect_gtk_xft_dpi_notify(clone!(@weak obj => move |_| {
                    obj.imp().distance_between_rows_center.set(None);
                }));
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for View {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let obj = self.obj();

            if obj.model().n_items() == 0 {
                tracing::debug!("Filter list model is empty, not drawing");
                self.parent_snapshot(snapshot);
                return;
            };

            obj.draw_columns(snapshot);

            for row in self.rows.borrow().iter() {
                let Some(row) = row.upgrade() else {
                    continue;
                };

                if !row.is_drawable() {
                    continue;
                }

                let Some(message) = row.message() else {
                    continue;
                };

                // Translate from row center coordinates to self coordinates
                let row_center_y = row
                    .compute_point(
                        &*obj,
                        &Point::new(row.width() as f32 / 2.0, row.height() as f32 / 2.0),
                    )
                    .unwrap()
                    .y();

                match message.message_type() {
                    MessageType::MethodCall => {
                        if let Err(err) = obj.draw_method_call(snapshot, &message, row_center_y) {
                            tracing::warn!(%message, "Can't draw method call: {:?}", err);
                        }
                    }
                    MessageType::MethodReturn | MessageType::Error => {
                        let Some(call_message) = message.associated_message() else {
                            tracing::warn!(%message, "Can't draw return message; it has no call message");
                            continue;
                        };

                        if let Err(err) = obj.draw_method_return(
                            snapshot,
                            &call_message,
                            &message,
                            &row,
                            row_center_y,
                        ) {
                            tracing::warn!(%call_message, %message, "Can't draw method return: {:?}", err);
                        }
                    }
                    MessageType::Signal => {
                        if let Err(err) = obj.draw_signal(snapshot, &message, row_center_y) {
                            tracing::warn!(%message, "Can't draw signal: {:?}", err);
                        }
                    }
                }
            }

            self.parent_snapshot(snapshot);
        }

        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            self.list_view.measure(orientation, for_size)
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.list_view
                .size_allocate(&gtk::Allocation::new(0, 0, width, height), baseline)
        }
    }

    impl ScrollableImpl for View {}

    impl View {
        fn selected_message(&self) -> Option<Message> {
            self.selection_model
                .selected_item()
                .map(|selected_item| selected_item.downcast().unwrap())
        }

        fn set_column_width(&self, column_width: f32) {
            if column_width == self.column_width.get() {
                return;
            }

            self.column_width.set(column_width);

            let obj = self.obj();
            obj.update_row_width_request();
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

        fn set_first_column_initial_x(&self, first_column_initial_x: f32) {
            if first_column_initial_x == self.first_column_initial_x.get() {
                return;
            }

            self.first_column_initial_x.set(first_column_initial_x);

            let obj = self.obj();
            obj.update_row_width_request();
            obj.notify_first_column_x();
        }

        fn hscroll_policy(&self) -> gtk::ScrollablePolicy {
            self.list_view.hscroll_policy()
        }

        fn set_hscroll_policy(&self, policy: gtk::ScrollablePolicy) {
            self.list_view.set_hscroll_policy(policy);
        }

        fn hadjustment(&self) -> Option<gtk::Adjustment> {
            self.list_view.hadjustment()
        }

        fn set_hadjustment(&self, adj: Option<&gtk::Adjustment>) {
            self.list_view.set_hadjustment(adj);
        }

        fn vscroll_policy(&self) -> gtk::ScrollablePolicy {
            self.list_view.vscroll_policy()
        }

        fn set_vscroll_policy(&self, policy: gtk::ScrollablePolicy) {
            self.list_view.set_vscroll_policy(policy);
        }

        fn vadjustment(&self) -> Option<gtk::Adjustment> {
            self.list_view.vadjustment()
        }

        fn set_vadjustment(&self, adj: Option<&gtk::Adjustment>) {
            self.list_view.set_vadjustment(adj);
        }
    }
}

glib::wrapper! {
    pub struct View(ObjectSubclass<imp::View>)
        @extends gtk::Widget, @implements gtk::Scrollable;
}

impl View {
    pub fn set_model(&self, model: &FilteredMessageModel) {
        let imp = self.imp();

        model.filtered_bus_names().connect_items_changed(
            clone!(@weak self as obj => move |_, _, _, _| {
                obj.update_row_width_request();
            }),
        );

        imp.selection_model.set_model(Some(model));

        self.update_row_width_request();
    }

    pub fn model(&self) -> FilteredMessageModel {
        self.imp()
            .selection_model
            .model()
            .expect("model must be set")
            .downcast()
            .unwrap()
    }

    pub fn unselect(&self) {
        self.imp()
            .selection_model
            .set_selected(gtk::INVALID_LIST_POSITION);
    }

    pub fn scroll_to(&self, message: &Message, flags: gtk::ListScrollFlags) -> Result<()> {
        let index = self
            .model()
            .get_index_of(message)
            .context("Message does not exist on the model")?;
        self.imp().list_view.scroll_to(index as u32, flags, None);
        Ok(())
    }

    /// Returns the x coordinate where the given name must be placed
    fn x_for_name(&self, name: &BusName<'_>, receive_index: ReceiveIndex) -> Result<f32> {
        let model = self.model();
        let bus_names = model.filtered_bus_names();

        let bus_name_index = bus_names.get_index_of(name).or_else(|| {
            tracing::trace!("`{}` was not found in keys; looking for other names", name);
            match name {
                BusName::Unique(_) => None,
                BusName::WellKnown(ref wk_name) => {
                    bus_names.get_index_of_wk_name(wk_name, receive_index.into())
                }
            }
        });

        if tracing::enabled!(tracing::Level::DEBUG) && bus_name_index.is_none() {
            // This makes it easier to read the names when debugging
            let bus_names = bus_names
                .iter()
                .map(|bus_name_item| {
                    let name = bus_name_item.name();
                    let wk_names = bus_name_item.wk_names(receive_index.into());
                    (
                        name.to_string(),
                        wk_names
                            .iter()
                            .map(|name| name.to_string())
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<IndexMap<_, _>>();
            tracing::debug!(?bus_names, "Index of `{}` in bus names was not found", name);
        }

        let bus_name_index = bus_name_index.context("Name not found in bus names")?;

        Ok(self.first_column_x() + self.column_width() * bus_name_index as f32)
    }

    /// Computes the distance from row center to an adjacent row's center,
    /// then caches and return it. This assumes that distance between row
    /// centers is consistent accross rows, and thus heights.
    ///
    /// Returns None if there is no visible adjacent row
    ///
    /// This must be invalidated when the text scale factor changes
    fn distance_between_rows_center(&self, row: &Row, row_center_y: f32) -> Option<f32> {
        // FIXME Don't have any better idea to compute distance between rows centers

        let imp = self.imp();

        if let Some(distance) = imp.distance_between_rows_center.get() {
            return Some(distance);
        }

        let parent = row.parent().unwrap(); // Get GtkListItemWidget

        let first_prev_adjacent_parent = parent.prev_sibling().filter(|w| w.is_drawable());
        let second_prev_adjacent_parent = first_prev_adjacent_parent
            .as_ref()
            .and_then(|w| w.prev_sibling())
            .filter(|w| w.is_drawable());

        let first_next_adjacent_parent = parent.next_sibling().filter(|w| w.is_drawable());
        let second_next_adjacent_parent = first_next_adjacent_parent
            .as_ref()
            .and_then(|w| w.next_sibling())
            .filter(|w| w.is_drawable());

        // Get the adjacent row which is farthest to the edge. For example,
        // we don't want to use an adjacent row clipped to the edge since GTK
        // doesn't correctly compute the row's center coordinates in that case.
        let adjacent_parent = match (
            second_prev_adjacent_parent,
            first_prev_adjacent_parent,
            first_next_adjacent_parent,
            second_next_adjacent_parent,
        ) {
            (Some(_), Some(w), Some(_), Some(_))
            | (Some(_), Some(w), Some(_), None)
            | (None, Some(_), Some(w), Some(_))
            | (None, None, Some(w), Some(_))
            | (Some(_), Some(w), None, None)
            | (None, Some(w), Some(_), None)
            | (None, None, Some(w), None)
            | (None, Some(w), None, None) => w,
            (None, None, None, None) => return None,
            (_, _, _, _) => unreachable!(),
        };

        let adjacent_row = adjacent_parent
            .first_child() // Get our DiagramRow
            .unwrap()
            .downcast::<Row>()
            .unwrap();
        debug_assert_eq!(adjacent_row.height(), row.height());

        // Translate the adjacent row center coordinates into self's coordinates
        let adjacent_row_center_y = adjacent_row
            .compute_point(
                self,
                &Point::new(
                    adjacent_row.width() as f32 / 2.0,
                    adjacent_row.height() as f32 / 2.0,
                ),
            )
            .unwrap()
            .y();
        let ret = (adjacent_row_center_y - row_center_y).abs();
        debug_assert!(ret >= row.height() as f32);

        imp.distance_between_rows_center.set(Some(ret));

        Some(ret)
    }

    fn bounds(&self) -> Rect {
        Rect::new(0.0, 0.0, self.width() as f32, self.height() as f32)
    }

    fn arrow_stroke(&self) -> gsk::Stroke {
        gsk::Stroke::builder(ARROW_LINE_WIDTH)
            .line_join(gsk::LineJoin::Round)
            .line_cap(gsk::LineCap::Round)
            .build()
    }

    fn error_arrow_color(&self) -> gdk::RGBA {
        match adw::StyleManager::default().color_scheme() {
            adw::ColorScheme::Default
            | adw::ColorScheme::ForceLight
            | adw::ColorScheme::PreferLight => crate::colors::RED_3,
            adw::ColorScheme::PreferDark | adw::ColorScheme::ForceDark => crate::colors::RED_1,
            _ => unreachable!(),
        }
    }

    fn method_arc_color(&self) -> gdk::RGBA {
        match adw::StyleManager::default().color_scheme() {
            adw::ColorScheme::Default
            | adw::ColorScheme::ForceLight
            | adw::ColorScheme::PreferLight => crate::colors::GREEN_5,
            adw::ColorScheme::PreferDark | adw::ColorScheme::ForceDark => crate::colors::GREEN_3,
            _ => unreachable!(),
        }
    }

    fn draw_columns(&self, snapshot: &gtk::Snapshot) {
        let color = self.color();
        let height = self.height();

        let first_column_x = self.first_column_x();
        let column_width = self.column_width();

        let model = self.model();
        let bus_names = model.filtered_bus_names();

        let path_builder = gsk::PathBuilder::new();

        let mut cursor_x = first_column_x;
        for _ in bus_names.iter() {
            let x = cursor_x.round();
            path_builder.move_to(x, 0.0);
            path_builder.line_to(x, height as f32);
            cursor_x += column_width;
        }

        snapshot.push_stroke(
            &path_builder.to_path(),
            &gsk::Stroke::new(COLUMN_LINE_WIDTH),
        );
        snapshot.append_color(&color.with_alpha(color.alpha() * 0.3), &self.bounds());
        snapshot.pop();
    }

    fn draw_method_call(
        &self,
        snapshot: &gtk::Snapshot,
        message: &Message,
        row_center_y: f32,
    ) -> Result<()> {
        let sender = message.sender().context("Call message has no sender")?;
        let destination = message
            .destination()
            .context("Call message has no destination")?;

        let start_x = self.x_for_name(&BusName::from(sender), message.receive_index())?;
        let end_x = self.x_for_name(&destination, message.receive_index())?;

        let path_builder = gsk::PathBuilder::new();
        path_builder.add_arrow(
            &Point::new(start_x, row_center_y),
            &Point::new(end_x, row_center_y),
            ArrowTipType::TopHalf,
        );
        snapshot.push_stroke(&path_builder.to_path(), &self.arrow_stroke());
        snapshot.append_color(&self.color(), &self.bounds());
        snapshot.pop();

        Ok(())
    }

    fn draw_method_return(
        &self,
        snapshot: &gtk::Snapshot,
        call_message: &Message,
        return_message: &Message,
        return_row: &Row,
        return_row_center_y: f32,
    ) -> Result<()> {
        const RESPONSE_TIME_LAYOUT_OFFSET_X: f32 = 20.0;

        let model = self.model();

        let return_message_sender = return_message
            .sender()
            .context("Return message has no sender")?;
        debug_assert!(call_message
            .destination()
            .is_some_and(|call_message_destination| call_message_destination
                == return_message_sender
                || match call_message_destination {
                    BusName::Unique(_) => false,
                    BusName::WellKnown(ref wk_name) => model
                        .filtered_bus_names()
                        .get(&BusName::from(return_message_sender.as_ref()))
                        .unwrap()
                        .wk_names(return_message.receive_index().into())
                        .contains(wk_name),
                }));

        let call_message_sender = call_message
            .sender()
            .context("Call message has no sender")?;
        debug_assert!(return_message
            .destination()
            .is_some_and(|return_message_destination| return_message_destination
                == call_message_sender
                || match return_message_destination {
                    BusName::Unique(_) => false,
                    BusName::WellKnown(ref wk_name) => model
                        .filtered_bus_names()
                        .get(&BusName::from(call_message_sender.as_ref()))
                        .unwrap()
                        .wk_names(return_message.receive_index().into())
                        .contains(wk_name),
                }));

        let start_x = self.x_for_name(
            &BusName::from(return_message_sender),
            return_message.receive_index(),
        )?;
        let end_x = self.x_for_name(
            &BusName::from(call_message_sender),
            return_message.receive_index(),
        )?;

        let arrow_path_builder = gsk::PathBuilder::new();
        arrow_path_builder.add_arrow(
            &Point::new(start_x, return_row_center_y),
            &Point::new(end_x, return_row_center_y),
            ArrowTipType::BottomHalf,
        );
        let arrow_color = if return_message.message_type() == MessageType::Error {
            self.error_arrow_color()
        } else {
            self.color()
        };
        snapshot.push_stroke(&arrow_path_builder.to_path(), &self.arrow_stroke());
        snapshot.append_color(&arrow_color, &self.bounds());
        snapshot.pop();

        let call_row_center_y = {
            let return_message_index = model.get_index_of(return_message).unwrap();
            let call_message_index = model.get_index_of(call_message).unwrap();

            // We can't just use row heights as we also need to account for the paddings.
            let other_row_offset_y = self
                .distance_between_rows_center(return_row, return_row_center_y)
                .unwrap()
                * (return_message_index as f32 - call_message_index as f32);
            return_row_center_y - other_row_offset_y
        };

        // TODO draw this arc as long as the rows in the middle of call and return
        // row is visible
        let arc_path_builder = gsk::PathBuilder::new();
        arc_path_builder.add_arc(
            &Point::new(start_x, return_row_center_y),
            &Point::new(start_x, call_row_center_y),
            if (end_x - start_x).is_sign_positive() {
                ArcSide::Left
            } else {
                ArcSide::Right
            },
        );
        let arc_stroke = gsk::Stroke::new(ARROW_LINE_WIDTH);
        arc_stroke.set_dash(&[3.0, 3.0]);
        snapshot.push_stroke(&arc_path_builder.to_path(), &arc_stroke);
        snapshot.append_color(&self.method_arc_color(), &self.bounds());
        snapshot.pop();

        let response_time_layout = self
            .imp()
            .response_time_layout
            .get()
            .expect("response time layout must be set on construcetd");
        response_time_layout.set_markup(&format!(
            r#"<span size="x-small">{} ms</span>"#,
            (return_message.timestamp() - call_message.timestamp())
                .as_millis_f64()
                .round()
        ));
        let (ink_extents, _) = response_time_layout.pixel_extents();

        snapshot.save();
        if (end_x - start_x).is_sign_positive() {
            snapshot.translate(&Point::new(
                start_x - RESPONSE_TIME_LAYOUT_OFFSET_X - ink_extents.width() as f32,
                return_row_center_y - ink_extents.height() as f32,
            ));
        } else {
            snapshot.translate(&Point::new(
                start_x + RESPONSE_TIME_LAYOUT_OFFSET_X,
                return_row_center_y - ink_extents.height() as f32,
            ));
        }
        snapshot.append_layout(response_time_layout, &self.color());
        snapshot.restore();

        Ok(())
    }

    fn draw_signal(
        &self,
        snapshot: &gtk::Snapshot,
        message: &Message,
        row_center_y: f32,
    ) -> Result<()> {
        const CIRCLE_RADIUS: f32 = 5.0;
        const ARROW_END_OFFSET: f32 = 20.0;

        let sender = message.sender().context("Signal message has no sender")?;
        let start_x = self.x_for_name(&BusName::from(sender), message.receive_index())?;

        let path_builder = gsk::PathBuilder::new();

        path_builder.add_circle(&Point::new(start_x, row_center_y), CIRCLE_RADIUS);

        if let Some(destination) = message.destination() {
            // This is a targeted signal

            let end_x = self.x_for_name(&destination, message.receive_index())?;

            path_builder.add_arrow(
                &Point::new(
                    start_x + negate_if((end_x - start_x).is_sign_negative(), CIRCLE_RADIUS),
                    row_center_y,
                ),
                &Point::new(end_x, row_center_y),
                ArrowTipType::Full,
            );
        } else {
            let model = self.model();
            let bus_names = model.filtered_bus_names();

            let first_column_x = self.first_column_x();
            let column_width = self.column_width();

            // FIXME Is this correct that the arrows extend on all columns/names?

            // Left arrow
            path_builder.add_arrow(
                &Point::new(start_x - CIRCLE_RADIUS, row_center_y),
                &Point::new(first_column_x - ARROW_END_OFFSET, row_center_y),
                ArrowTipType::Full,
            );

            // Right arrow
            path_builder.add_arrow(
                &Point::new(start_x + CIRCLE_RADIUS, row_center_y),
                &Point::new(
                    first_column_x
                        + column_width * (bus_names.n_items() - 1) as f32
                        + ARROW_END_OFFSET,
                    row_center_y,
                ),
                ArrowTipType::Full,
            );
        }

        snapshot.push_stroke(&path_builder.to_path(), &self.arrow_stroke());
        snapshot.append_color(&self.color(), &self.bounds());
        snapshot.pop();

        Ok(())
    }

    // This must be called when either:
    // a. the model is replaced
    // b. the filtered bus names items changed
    // c. first_column_initial_x or column_width changed
    fn update_row_width_request(&self) {
        let imp = self.imp();

        let n_bus_names = self.model().filtered_bus_names().n_items();

        let new_value = if n_bus_names == 0 {
            -1
        } else {
            let column_width = self.column_width();
            (self.first_column_initial_x() + column_width * n_bus_names as f32 - column_width / 2.0)
                .round() as i32
        };
        imp.row_width_request.set(new_value);

        for row in imp.rows.borrow().iter() {
            if let Some(row) = row.upgrade() {
                row.set_width_request(new_value);
            }
        }
    }
}

#[gtk::template_callbacks]
impl View {
    #[template_callback]
    fn factory_setup(&self, list_item: &glib::Object) {
        let imp = self.imp();

        let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
        let row = Row::default();
        row.set_width_request(imp.row_width_request.get());
        list_item.set_child(Some(&row));

        // Remove dead weak references
        imp.rows.borrow_mut().retain(|i| i.upgrade().is_some());

        debug_assert_eq!(imp.rows.borrow().iter().filter(|i| **i == row).count(), 0);
        imp.rows.borrow_mut().push(row.downgrade());
    }

    #[template_callback]
    fn factory_teardown(&self, list_item: &glib::Object) {
        let imp = self.imp();

        let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
        if let Some(row) = list_item.child() {
            let row = row.downcast_ref::<Row>().unwrap();

            debug_assert_eq!(imp.rows.borrow().iter().filter(|i| *i == row).count(), 1);
            imp.rows
                .borrow_mut()
                .retain(|i| i.upgrade().is_some_and(|i| &i != row));
        }
    }

    #[template_callback]
    fn factory_bind(&self, list_item: &glib::Object) {
        let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
        let row = list_item.child().unwrap().downcast::<Row>().unwrap();
        row.set_width_request(self.imp().row_width_request.get());
        row.set_head_message(
            self.model()
                .item(0)
                .map(|model| model.downcast::<Message>().unwrap()),
        );
        row.set_message(
            list_item
                .item()
                .map(|item| item.downcast::<Message>().unwrap()),
        );
    }

    #[template_callback]
    fn factory_unbind(&self, list_item: &glib::Object) {
        let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
        let row = list_item.child().unwrap().downcast::<Row>().unwrap();
        row.set_width_request(-1);
        row.set_message(None::<Message>);
        row.set_head_message(None::<Message>);
    }

    #[template_callback]
    fn selection_model_selected_item_notify(&self) {
        self.notify_selected_message();
    }
}

fn negate_if(negate: bool, num: f32) -> f32 {
    if negate {
        -num
    } else {
        num
    }
}

trait PathBuilderExt {
    fn add_arrow(&self, start: &Point, end: &Point, tip_type: ArrowTipType);
    fn add_arc(&self, start: &Point, end: &Point, arc_side: ArcSide);
}

impl PathBuilderExt for gsk::PathBuilder {
    fn add_arrow(&self, start: &Point, end: &Point, tip_type: ArrowTipType) {
        const TIP_OFFSET_X: f32 = 10.0;
        const TIP_OFFSET_Y: f32 = 5.0;

        self.move_to(start.x(), start.y());
        self.line_to(end.x(), end.y());

        let tip_offset_x = negate_if((end.x() - start.x()).is_sign_positive(), TIP_OFFSET_X);

        match tip_type {
            ArrowTipType::Full => {
                self.rel_move_to(tip_offset_x, -TIP_OFFSET_Y);
                self.line_to(end.x(), end.y());
                self.rel_line_to(tip_offset_x, TIP_OFFSET_Y);
            }
            ArrowTipType::TopHalf => {
                self.rel_line_to(tip_offset_x, -TIP_OFFSET_Y);
            }
            ArrowTipType::BottomHalf => {
                self.rel_line_to(tip_offset_x, TIP_OFFSET_Y);
            }
        }
    }

    fn add_arc(&self, start: &Point, end: &Point, arc_side: ArcSide) {
        const MID_OFFSET_X: f32 = 50.0;

        let mid_x = (start.x() + end.x()) / 2.0;
        let mid_y = (start.y() + end.y()) / 2.0;

        self.move_to(start.x(), start.y());
        self.quad_to(
            mid_x + negate_if(matches!(arc_side, ArcSide::Left), MID_OFFSET_X),
            mid_y,
            end.x(),
            end.y(),
        );
    }
}
