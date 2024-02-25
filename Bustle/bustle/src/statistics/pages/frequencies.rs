use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*};

use crate::{filtered_message_model::FilteredMessageModel, statistics::FrequencyItem};

mod imp {
    use std::cell::RefCell;

    use super::*;
    use crate::statistics::pages::ProgressCell;

    #[derive(Debug, Default, glib::Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::FrequenciesPage)]
    #[template(resource = "/org/freedesktop/Bustle/ui/frequencies_page.ui")]
    pub struct FrequenciesPage {
        #[property(get, set = Self::set_model)]
        pub(super) model: RefCell<Option<FilteredMessageModel>>,
        #[template_child]
        pub(super) column_view: TemplateChild<gtk::ColumnView>,
        #[template_child]
        pub(super) message_type_column: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub(super) member_column: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub(super) frequency_column: TemplateChild<gtk::ColumnViewColumn>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FrequenciesPage {
        const NAME: &'static str = "BustleFrequenciesPage";
        type Type = super::FrequenciesPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            ProgressCell::ensure_type();
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for FrequenciesPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.frequency_column
                .set_sorter(Some(&gtk::NumericSorter::new(Some(
                    &FrequencyItem::this_expression("count"),
                ))));

            self.message_type_column
                .set_sorter(Some(&gtk::CustomSorter::new(|a, b| {
                    let a = a.downcast_ref::<FrequencyItem>().unwrap();
                    let b = b.downcast_ref::<FrequencyItem>().unwrap();
                    a.message_type().cmp(&b.message_type()).into()
                })));
            self.member_column
                .set_sorter(Some(&gtk::StringSorter::new(Some(
                    &FrequencyItem::this_expression("member"),
                ))));
        }
    }

    impl WidgetImpl for FrequenciesPage {}

    impl BinImpl for FrequenciesPage {}

    impl FrequenciesPage {
        fn set_model(&self, model: &FilteredMessageModel) {
            let mut n_items = model.n_items();

            let frequencies = gio::ListStore::new::<FrequencyItem>();
            let mut frequencies_map = std::collections::HashMap::new();
            for message in model.iter() {
                let message_type = message.message_type();
                if message_type.is_method_return() {
                    n_items -= 1;
                    continue;
                }
                let member_name = message.member_markup(true);

                frequencies_map
                    .entry((member_name, message_type))
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }

            for ((member, message_type), count) in frequencies_map.iter() {
                frequencies.append(&FrequencyItem::new(member, *count, n_items, *message_type))
            }

            self.column_view
                .sort_by_column(Some(&self.frequency_column), gtk::SortType::Descending);
            let sorter = self.column_view.sorter();
            let sorted_model = gtk::SortListModel::new(Some(frequencies), sorter);
            let selection_model = gtk::NoSelection::new(Some(sorted_model));
            self.column_view.set_model(Some(&selection_model));
            self.model.set(Some(model.clone()));
        }
    }
}

glib::wrapper! {
     pub struct FrequenciesPage(ObjectSubclass<imp::FrequenciesPage>)
        @extends gtk::Widget, adw::Bin;
}

#[gtk::template_callbacks]
impl FrequenciesPage {
    #[template_callback]
    fn fraction(_: &glib::Object, entry: Option<&FrequencyItem>) -> f64 {
        entry.map_or(0.0, |e| e.count() as f64 / e.total() as f64)
    }

    #[template_callback]
    fn message_type(_: &glib::Object, entry: Option<&FrequencyItem>) -> String {
        entry.map(|e| e.message_type().i18n()).unwrap_or_default()
    }
}
