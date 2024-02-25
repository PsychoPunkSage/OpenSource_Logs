use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

use crate::filtered_message_model::FilteredMessageModel;

mod imp {
    use std::cell::OnceCell;

    use super::*;
    use crate::statistics::{DurationsPage, FrequenciesPage, SizesPage};

    #[derive(Debug, Default, glib::Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::StatisticsWindow)]
    #[template(resource = "/org/freedesktop/Bustle/ui/statistics.ui")]
    pub struct StatisticsWindow {
        #[property(get, set, construct_only)]
        pub(super) model: OnceCell<FilteredMessageModel>,
        #[template_child]
        pub(super) durations_page: TemplateChild<DurationsPage>,
        #[template_child]
        pub(super) sizes_page: TemplateChild<SizesPage>,
        #[template_child]
        pub(super) frequencies_page: TemplateChild<FrequenciesPage>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StatisticsWindow {
        const NAME: &'static str = "BustleStatisticsWindow";
        type Type = super::StatisticsWindow;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for StatisticsWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let model = self.obj().model();
            self.durations_page.set_model(&model);
            self.frequencies_page.set_model(&model);
            self.sizes_page.set_model(&model);
        }
    }

    impl WidgetImpl for StatisticsWindow {}
    impl AdwDialogImpl for StatisticsWindow {}
}

glib::wrapper! {
     pub struct StatisticsWindow(ObjectSubclass<imp::StatisticsWindow>)
        @extends gtk::Widget, adw::Dialog;
}

impl StatisticsWindow {
    pub fn new(model: &FilteredMessageModel) -> Self {
        glib::Object::builder().property("model", model).build()
    }
}
