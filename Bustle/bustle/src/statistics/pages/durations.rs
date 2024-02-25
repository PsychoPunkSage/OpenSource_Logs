use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*};

use crate::{filtered_message_model::FilteredMessageModel, statistics::DurationItem};

mod imp {
    use std::cell::RefCell;

    use super::*;

    #[derive(Debug, Default, glib::Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::DurationsPage)]
    #[template(resource = "/org/freedesktop/Bustle/ui/durations_page.ui")]
    pub struct DurationsPage {
        #[property(get, set = Self::set_model)]
        pub(super) model: RefCell<Option<FilteredMessageModel>>,
        #[template_child]
        pub(super) column_view: TemplateChild<gtk::ColumnView>,
        #[template_child]
        pub(super) method_column: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub(super) total_column: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub(super) calls_column: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub(super) mean_column: TemplateChild<gtk::ColumnViewColumn>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DurationsPage {
        const NAME: &'static str = "BustleDurationsPage";
        type Type = super::DurationsPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for DurationsPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.calls_column
                .set_sorter(Some(&gtk::NumericSorter::new(Some(
                    &DurationItem::this_expression("calls"),
                ))));

            self.method_column
                .set_sorter(Some(&gtk::CustomSorter::new(|a, b| {
                    let a = a.downcast_ref::<DurationItem>().unwrap();
                    let b = b.downcast_ref::<DurationItem>().unwrap();
                    a.method().cmp(&b.method()).into()
                })));

            self.total_column
                .set_sorter(Some(&gtk::CustomSorter::new(|a, b| {
                    let a = a.downcast_ref::<DurationItem>().unwrap();
                    let b = b.downcast_ref::<DurationItem>().unwrap();
                    a.total().cmp(&b.total()).into()
                })));

            self.mean_column
                .set_sorter(Some(&gtk::CustomSorter::new(|a, b| {
                    let a = a.downcast_ref::<DurationItem>().unwrap();
                    let b = b.downcast_ref::<DurationItem>().unwrap();
                    let mean_a = a.total() / a.calls();
                    let mean_b = b.total() / b.calls();
                    mean_a.cmp(&mean_b).into()
                })));
        }
    }

    impl WidgetImpl for DurationsPage {}

    impl BinImpl for DurationsPage {}

    impl DurationsPage {
        fn set_model(&self, model: &FilteredMessageModel) {
            let mut durations_map = std::collections::HashMap::new();
            let durations = gio::ListStore::new::<DurationItem>();
            for message in model.iter() {
                if !message.message_type().is_method_call() {
                    continue;
                }

                let Some(return_message) = message.associated_message() else {
                    continue;
                };

                // Ignore errors when computing the time it took for a method to receive a reply
                // This is mostly done to be 100% compatible with Hustle
                // TODO: investigate if it makes sense to keep this or not
                if return_message.message_type().is_error() {
                    continue;
                }

                let new_duration = return_message.timestamp() - message.timestamp();
                let member_name = message.member_markup(true);
                durations_map
                    .entry(member_name)
                    .and_modify(|(calls, duration)| {
                        *calls += 1;
                        *duration += new_duration;
                    })
                    .or_insert((1, new_duration));
            }

            let total_calls = durations_map.iter().map(|(_, (call, _))| call).sum();
            for (member, (calls, total_duration)) in durations_map.into_iter() {
                durations.append(&DurationItem::new(
                    &member,
                    calls,
                    total_calls,
                    total_duration,
                ))
            }
            self.column_view
                .sort_by_column(Some(&self.calls_column), gtk::SortType::Descending);
            let sorter = self.column_view.sorter();
            let sorted_model = gtk::SortListModel::new(Some(durations), sorter);
            let selection_model = gtk::NoSelection::new(Some(sorted_model));
            self.column_view.set_model(Some(&selection_model));
            self.model.set(Some(model.clone()));
        }
    }
}

glib::wrapper! {
     pub struct DurationsPage(ObjectSubclass<imp::DurationsPage>)
        @extends gtk::Widget, adw::Bin;
}

#[gtk::template_callbacks]
impl DurationsPage {
    #[template_callback]
    fn calls_fraction(_: &glib::Object, entry: Option<&DurationItem>) -> f64 {
        entry.map_or(0.0, |e| e.calls() as f64 / e.total_calls() as f64)
    }

    #[template_callback]
    fn total_duration(_: &glib::Object, entry: Option<&DurationItem>) -> String {
        entry
            .map(|e| format!("{:.2} ms", e.total().as_millis_f64()))
            .unwrap_or_default()
    }

    #[template_callback]
    fn mean_duration(_: &glib::Object, entry: Option<&DurationItem>) -> String {
        entry
            .map(|e| format!("{:.2} ms", e.total().as_millis_f64() / e.calls() as f64))
            .unwrap_or_default()
    }
}
