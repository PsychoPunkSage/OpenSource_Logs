use std::collections::HashMap;

use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*};

use crate::{
    filtered_message_model::FilteredMessageModel, message_type::MessageType, statistics::SizeItem,
};

mod imp {
    use std::cell::RefCell;

    use super::*;

    #[derive(Debug, Default, glib::Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::SizesPage)]
    #[template(resource = "/org/freedesktop/Bustle/ui/sizes_page.ui")]
    pub struct SizesPage {
        #[property(get, set = Self::set_model)]
        pub(super) model: RefCell<Option<FilteredMessageModel>>,
        #[template_child]
        pub(super) column_view: TemplateChild<gtk::ColumnView>,
        #[template_child]
        pub(super) message_type_column: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub(super) member_column: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub(super) largest_column: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub(super) mean_column: TemplateChild<gtk::ColumnViewColumn>,
        #[template_child]
        pub(super) smallest_column: TemplateChild<gtk::ColumnViewColumn>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SizesPage {
        const NAME: &'static str = "BustleSizesPage";
        type Type = super::SizesPage;
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
    impl ObjectImpl for SizesPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.member_column
                .set_sorter(Some(&gtk::StringSorter::new(Some(
                    &SizeItem::this_expression("member"),
                ))));

            self.message_type_column
                .set_sorter(Some(&gtk::CustomSorter::new(|a, b| {
                    let a = a.downcast_ref::<SizeItem>().unwrap();
                    let b = b.downcast_ref::<SizeItem>().unwrap();
                    a.message_type().cmp(&b.message_type()).into()
                })));

            self.largest_column.set_sorter(Some(
                &gtk::NumericSorter::builder()
                    .expression(SizeItem::this_expression("largest"))
                    .sort_order(gtk::SortType::Descending)
                    .build(),
            ));
            self.smallest_column.set_sorter(Some(
                &gtk::NumericSorter::builder()
                    .expression(SizeItem::this_expression("smallest"))
                    .sort_order(gtk::SortType::Descending)
                    .build(),
            ));
            self.mean_column.set_sorter(Some(
                &gtk::NumericSorter::builder()
                    .expression(SizeItem::this_expression("mean"))
                    .sort_order(gtk::SortType::Descending)
                    .build(),
            ));
        }
    }

    impl WidgetImpl for SizesPage {}

    impl BinImpl for SizesPage {}

    impl SizesPage {
        fn set_model(&self, model: &FilteredMessageModel) {
            let mut sizes_map: HashMap<(MessageType, String), (usize, Vec<usize>)> =
                std::collections::HashMap::new();
            let sizes = gio::ListStore::new::<SizeItem>();
            for message in model.iter() {
                let length = message.len();
                let message_type = message.message_type();
                let member_name = message.member_markup(true);

                sizes_map
                    .entry((message_type, member_name))
                    .and_modify(|(calls, bytes_vec)| {
                        *calls += 1;
                        bytes_vec.push(length);
                    })
                    .or_insert((1, vec![length]));
            }
            for ((message_type, member), (calls, mut bytes)) in sizes_map.into_iter() {
                bytes.sort();
                let mean = bytes.iter().copied().sum::<usize>() / calls;
                sizes.append(&SizeItem::new(
                    &member,
                    *bytes.first().unwrap_or(&0) as u32,
                    mean as u32,
                    *bytes.last().unwrap_or(&0) as u32,
                    message_type,
                ))
            }

            self.column_view
                .sort_by_column(Some(&self.mean_column), gtk::SortType::Ascending);
            let sorter = self.column_view.sorter();
            let sorted_model = gtk::SortListModel::new(Some(sizes), sorter);
            let selection_model = gtk::NoSelection::new(Some(sorted_model));
            self.column_view.set_model(Some(&selection_model));
            self.model.set(Some(model.clone()));
        }
    }
}

glib::wrapper! {
     pub struct SizesPage(ObjectSubclass<imp::SizesPage>)
        @extends gtk::Widget, adw::Bin;
}

#[gtk::template_callbacks]
impl SizesPage {
    #[template_callback]
    fn message_type(_: &glib::Object, entry: Option<&SizeItem>) -> String {
        entry.map(|e| e.message_type().i18n()).unwrap_or_default()
    }

    #[template_callback]
    fn smallest_bytes(_: &glib::Object, entry: Option<&SizeItem>) -> glib::GString {
        entry
            .map(|e| glib::format_size(e.smallest().into()))
            .unwrap_or_default()
    }

    #[template_callback]
    fn mean_bytes(_: &glib::Object, entry: Option<&SizeItem>) -> glib::GString {
        entry
            .map(|e| glib::format_size(e.mean().into()))
            .unwrap_or_default()
    }

    #[template_callback]
    fn largest_bytes(_: &glib::Object, entry: Option<&SizeItem>) -> glib::GString {
        entry
            .map(|e| glib::format_size(e.largest().into()))
            .unwrap_or_default()
    }
}
