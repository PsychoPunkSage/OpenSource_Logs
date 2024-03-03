use adw::prelude::*;
use adw::subclass::prelude::*;
use anyhow::{Context, Result};
use gettextrs::gettext;
use gtk::{
    gio,
    glib::{self, clone},
};

use crate::{
    address_dialog::AddressDialog,
    application::Application,
    config::{APP_ID, PROFILE, VERSION},
    details_view::DetailsView,
    diagram::Diagram,
    i18n::gettext_f,
    message::Message,
    message_list::MessageList,
    monitor::{Cancelled, Monitor},
    statistics::StatisticsWindow,
};

#[derive(Default, Debug, Copy, Clone, glib::Enum, PartialEq)]
#[repr(u32)]
#[enum_type(name = "BustleView")]
pub enum View {
    #[default]
    EmptyState,
    Loading,
    Diagram,
}

mod imp {
    use std::cell::RefCell;

    use super::*;
    use crate::{filter_pane::FilterPane, filtered_message_model::FilteredMessageModel};

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/org/freedesktop/Bustle/ui/window.ui")]
    pub struct Window {
        #[template_child]
        pub(super) toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub(super) tab_view: TemplateChild<adw::TabView>,
        // To be commented ------------------------------------------------
        #[template_child]
        pub(super) main_stack: TemplateChild<gtk::Stack>,
        // ----------------------------------------------------------------
        #[template_child]
        pub(super) empty_page: TemplateChild<adw::ToolbarView>,
        #[template_child]
        pub(super) empty_status_page: TemplateChild<adw::StatusPage>,
        #[template_child]
        pub(super) loading_page: TemplateChild<adw::ToolbarView>,
        #[template_child]
        pub(super) diagram_page: TemplateChild<adw::ToolbarView>,
        #[template_child]
        pub(super) record_button_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub(super) diagram_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub(super) diagram_page_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub(super) waiting_sub_page: TemplateChild<adw::StatusPage>,
        #[template_child]
        pub(super) split_view_sub_page: TemplateChild<adw::OverlaySplitView>,
        #[template_child]
        pub(super) details_view_split_view: TemplateChild<adw::OverlaySplitView>,
        #[template_child]
        pub(super) diagram: TemplateChild<Diagram>,
        #[template_child]
        pub(super) filtered_message_model: TemplateChild<FilteredMessageModel>,
        #[template_child]
        pub(super) details_view: TemplateChild<DetailsView>,

        pub(super) settings: gio::Settings,

        pub(super) monitor: RefCell<Option<Monitor>>,
        // The currently recorded filename
        pub(super) filename: RefCell<Option<String>>,
    }

    impl Default for Window {
        fn default() -> Self {
            Self {
                toast_overlay: TemplateChild::default(),
                tab_view: TemplateChild::default(),
                // To be commented ------------------------------------------------
                main_stack: TemplateChild::default(),
                // ----------------------------------------------------------------
                empty_page: TemplateChild::default(),
                empty_status_page: TemplateChild::default(),
                loading_page: TemplateChild::default(),
                diagram_page: TemplateChild::default(),
                record_button_stack: TemplateChild::default(),
                diagram_title: TemplateChild::default(),
                diagram_page_stack: TemplateChild::default(),
                waiting_sub_page: TemplateChild::default(),
                split_view_sub_page: TemplateChild::default(),
                details_view_split_view: TemplateChild::default(),
                diagram: TemplateChild::default(),
                filtered_message_model: TemplateChild::default(),
                details_view: TemplateChild::default(),
                settings: gio::Settings::new(APP_ID),
                monitor: RefCell::default(),
                filename: RefCell::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "BustleWindow";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            FilterPane::ensure_type();
            klass.bind_template();
            klass.bind_template_instance_callbacks();

            klass.install_action("win.about", None, move |window, _, _| {
                window.show_about_window();
            });

            klass.install_action("win.statistics", None, |window, _, _| {
                window.show_statistics();
            });

            klass.install_action("win.filter-services", None, |window, _, _| {
                let imp = window.imp();
                imp.split_view_sub_page
                    .set_show_sidebar(!imp.split_view_sub_page.shows_sidebar());
            });

            klass.install_action_async("win.record-session-bus", None, |window, _, _| async move {
                if let Err(err) = window
                    .start_recording(Monitor::session(), &gettext("Recording session bus…"))
                    .await
                {
                    tracing::error!("Failed to record session: {err:?}");
                    window.add_error_toast(&gettext("Failed to record session bus"));
                }
            });

            klass.install_action_async("win.record-system-bus", None, |window, _, _| async move {
                if let Err(err) = window
                    .start_recording(Monitor::system(), &gettext("Recording system bus…"))
                    .await
                {
                    tracing::error!("Failed to record system: {err:?}");
                    if !err.is::<Cancelled>() {
                        window.add_error_toast(&gettext("Failed to record system bus"));
                    }
                }
            });

            klass.install_action_async("win.record-address", None, |window, _, _| async move {
                if let Ok(address) = AddressDialog::choose(&window).await {
                    let address_display = address.to_string();

                    if let Err(err) = window
                        .start_recording(
                            Monitor::address(address),
                            // Translators: Do NOT translate the contents between '{' and '}', this
                            // is a variable name.
                            &gettext_f("Recording {address}…", &[("address", &address_display)]),
                        )
                        .await
                    {
                        tracing::error!(
                            address = address_display,
                            "Failed to record address: {err:?}"
                        );
                        if !err.is::<Cancelled>() {
                            window.add_error_toast(&gettext("Failed to record address"));
                        }
                    }
                }
            });

            klass.install_action("win.stop-recording", None, |window, _, _| {
                window.stop_recording();
            });

            klass.install_action_async("win.open-log", None, |window, _, _| async move {
                if let Err(err) = window.open_log().await {
                    tracing::error!("Could not open log: {err:?}");
                    if !err
                        .downcast_ref::<glib::Error>()
                        .is_some_and(|error| error.matches(gtk::DialogError::Dismissed))
                    {
                        window.add_error_toast(&gettext("Failed to open file"));
                    }
                }
            });

            // @new
            // klass.install_action_async("win.new-document", None, |window, _, _| async move {
            //     if window
            //         .handle_unsaved_changes(&window.document())
            //         .await
            //         .is_err()
            //     {
            //         return;
            //     }

            //     window.set_document(&Document::draft());
            //     window.add_new_page();
            // });

            // @new
            // klass.install_action_async("win.save-document", None, |obj, _, _| async move {
            //     if let Err(err) = obj.save_document(&obj.document()).await {
            //         let page = obj.selected_page().unwrap();
            //         if let Err(err) = page.save_document().await {
            //             if !err
            //                 .downcast_ref::<glib::Error>()
            //                 .is_some_and(|error| error.matches(gtk::DialogError::Dismissed))
            //             {
            //                 tracing::error!("Failed to save document: {:?}", err);
            //                 obj.add_message_toast(&gettext("Failed to save document"));
            //             }
            //         }
            //     }
            // });

            // @new
            // klass.install_action_async("win.save-document-as", None, |obj, _, _| async move {
            //     if let Err(err) = obj.save_document_as(&obj.document()).await {
            //         let page = obj.selected_page().unwrap();
            //         if let Err(err) = page.save_document_as().await {
            //             if !err
            //                 .downcast_ref::<glib::Error>()
            //                 .is_some_and(|error| error.matches(gtk::DialogError::Dismissed))
            //             {
            //                 tracing::error!("Failed to save document as: {:?}", err);
            //                 obj.add_message_toast(&gettext("Failed to save document as"));
            //             }
            //         }
            //     }
            // });

            klass.install_action_async("win.save", None, |window, _, _| async move {
                if let Err(err) = window.save().await {
                    tracing::error!("Could not save: {err:?}");
                    if !err
                        .downcast_ref::<glib::Error>()
                        .is_some_and(|error| error.matches(gtk::DialogError::Dismissed))
                    {
                        window.add_error_toast(&gettext("Failed to save as PCAP"));
                    }
                } else {
                    window.add_message_toast(&gettext("Recording saved as PCAP"));
                }
            });

            klass.install_action_async("win.save-dot", None, |window, _, _| async move {
                if let Err(err) = window.save_as_dot().await {
                    tracing::error!("Could not save: {err:?}");
                    if !err
                        .downcast_ref::<glib::Error>()
                        .is_some_and(|error| error.matches(gtk::DialogError::Dismissed))
                    {
                        window.add_error_toast(&gettext("Failed to save as DOT graph"));
                    }
                } else {
                    window.add_message_toast(&gettext("Recording saved as DOT graph"))
                }
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            // Devel Profile
            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }

            self.empty_status_page.set_icon_name(Some(APP_ID));

            // Load latest window state
            obj.load_window_size();

            obj.update_details_view();

            obj.set_view(View::EmptyState);
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {
        // Save window state on delete event
        fn close_request(&self) -> glib::Propagation {
            if let Err(err) = self.obj().save_window_size() {
                tracing::warn!("Failed to save window state: {:?}", err);
            }

            // Pass close request on to the parent
            self.parent_close_request()
        }
    }

    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup, gtk::Root;
}

impl Window {
    // @Original
    // fn new(app: &Application) -> Self {
    //     glib::Object::builder().property("application", app).build()
    // }

    // @new
    pub fn new(app: &Application) -> Self {
        let this = glib::Object::builder().property("application", app).build();
        let group = gtk::WindowGroup::new();
        group.add_window(&this);
        this
    }


    pub fn with_group(app: &Application) -> Self {
        let window = Self::new(app);
        let group = gtk::WindowGroup::new();
        group.add_window(&window);
        window
    }

    pub async fn load_log(&self, file: &gio::File) -> anyhow::Result<()> {
        let path = file.path().unwrap();

        let prev_view = self.view();
        self.set_view(View::Loading);

        let message_list = match MessageList::load_from_file(path).await {
            Ok(message_list) => message_list,
            Err(err) => {
                self.set_view(prev_view);
                return Err(err);
            }
        };

        let imp = self.imp();
        imp.filtered_message_model
            .set_message_list(Some(&message_list));
        let filename = file.basename().unwrap_or_default().display().to_string();

        imp.diagram_title.set_title(&filename);
        imp.filename
            .replace(Some(filename.trim_end_matches(".pcap").to_string()));

        self.set_view(View::Diagram);
        imp.diagram_page_stack
            .set_visible_child(&*imp.split_view_sub_page);

        Ok(())
    }

    fn set_view(&self, view: View) {
        let imp = self.imp();

        match view {
            View::EmptyState => imp.main_stack.set_visible_child(&*imp.empty_page),
            View::Loading => imp.main_stack.set_visible_child(&*imp.loading_page),
            View::Diagram => {
                imp.main_stack.set_visible_child(&*imp.diagram_page);
            }
        }

        let is_recording = imp.monitor.borrow().is_some();

        self.action_set_enabled(
            "win.record-session-bus",
            view == View::EmptyState || (view == View::Diagram && !is_recording),
        );
        self.action_set_enabled(
            "win.record-system-bus",
            view == View::EmptyState || (view == View::Diagram && !is_recording),
        );
        self.action_set_enabled(
            "win.record-address",
            view == View::EmptyState || (view == View::Diagram && !is_recording),
        );

        self.action_set_enabled("win.stop-recording", view == View::Diagram && is_recording);

        self.action_set_enabled(
            "win.open-log",
            view == View::EmptyState || (view == View::Diagram && !is_recording),
        );
        self.action_set_enabled(
            "win.open-pair-logs",
            view == View::EmptyState || (view == View::Diagram && !is_recording),
        );

        self.action_set_enabled("win.statistics", view == View::Diagram && !is_recording);
        self.action_set_enabled("win.filter-services", view == View::Diagram);
        self.action_set_enabled("win.save", view == View::Diagram && !is_recording);
        self.action_set_enabled("win.save-dot", view == View::Diagram && !is_recording);
    }

    fn view(&self) -> View {
        let imp = self.imp();

        let visible_child = imp.main_stack.visible_child().unwrap();

        if visible_child == *imp.empty_page {
            View::EmptyState
        } else if visible_child == *imp.loading_page {
            View::Loading
        } else if visible_child == *imp.diagram_page {
            View::Diagram
        } else {
            unreachable!("unexpected visible child: {:?}", visible_child)
        }
    }

    fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let imp = self.imp();

        let (width, height) = self.default_size();

        imp.settings.set_int("window-width", width)?;
        imp.settings.set_int("window-height", height)?;

        imp.settings
            .set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }

    fn load_window_size(&self) {
        let imp = self.imp();

        let width = imp.settings.int("window-width");
        let height = imp.settings.int("window-height");
        let is_maximized = imp.settings.boolean("is-maximized");

        self.set_default_size(width, height);

        if is_maximized {
            self.maximize();
        }
    }

    fn show_about_window(&self) {
        let dialog = adw::AboutDialog::builder()
            .application_name("Bustle")
            .application_icon(APP_ID)
            .copyright("© 2008–2023 Will Thompson, Collabora Ltd. and contributors")
            .license_type(gtk::License::Lgpl21)
            .website("https://gitlab.gnome.org/World/bustle")
            .version(VERSION)
            .translator_credits(gettext("translator-credits"))
            .developer_name(gettext("The Bustle developers"))
            .developers(vec![
                "Bilal Elmoussaoui",
                "Dave Patrick Caberto",
                "Maximiliano Sandoval",
                "Will Thompson <will@willthompson.co.uk>",
                "Dafydd Harries",
                "Chris Lamb",
                "Marc Kleine-Budde",
                "Cosimo Alfarano",
                "Sergei Trofimovich",
                "Alex Merry",
                "Philip Withnall",
                "Jonny Lamb",
                "Daniel Firth",
            ])
            .designers(vec!["Tobias Bernard"])
            .build();

        dialog.present(self);
    }

    fn show_statistics(&self) {
        let imp = self.imp();

        debug_assert!(imp.filtered_message_model.message_list().is_some());

        StatisticsWindow::new(&imp.filtered_message_model).present(self);
    }

    fn stop_recording(&self) {
        let imp = self.imp();

        let monitor = imp
            .monitor
            .take()
            .expect("monitor must be set when recording");
        drop(monitor);

        imp.diagram.set_should_stick(false);

        let filename = glib::DateTime::now_local()
            .unwrap()
            .format("%Y-%m-%d %H:%M:%S")
            .unwrap();
        imp.diagram_title.set_title(&format!("*{filename}.pcap"));
        imp.filename.replace(Some(filename.to_string()));

        if imp.filtered_message_model.n_items() != 0 {
            self.set_view(View::Diagram);
            imp.diagram_page_stack
                .set_visible_child(&*imp.split_view_sub_page);
            imp.record_button_stack.set_visible_child_name("record");
        } else {
            self.set_view(View::EmptyState);
        }
    }

    async fn start_recording(&self, mut monitor: Monitor, display_message: &str) -> Result<()> {
        let imp = self.imp();

        let message_list = MessageList::default();

        monitor
            .start(clone!(@weak message_list => move |event| {
                message_list.push(Message::from_event(event));
            }))
            .await
            .context("Failed to start monitor")?;
        imp.monitor.replace(Some(monitor));

        imp.diagram.set_should_stick(true);

        imp.filtered_message_model
            .set_message_list(Some(&message_list));
        imp.diagram_title.set_title(display_message);
        imp.diagram_title.set_subtitle("");

        self.set_view(View::Diagram);
        imp.diagram_page_stack
            .set_visible_child(&*imp.waiting_sub_page);
        imp.record_button_stack.set_visible_child_name("stop");

        Ok(())
    }

    async fn open_log(&self) -> anyhow::Result<()> {
        let filter = gtk::FileFilter::new();
        // Translators: PCAP is a type of file, do not translate.
        filter.set_property("name", gettext("PCAP Files"));
        filter.add_mime_type("application/vnd.tcpdump.pcap");

        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);

        let chooser = gtk::FileDialog::builder()
            .title(gettext("Open Log"))
            .filters(&filters)
            .modal(true)
            .build();

        let file = match chooser.open_future(Some(self)).await {
            Err(err) if err.matches(gtk::DialogError::Dismissed) => return Ok(()),
            res => res?,
        };
        self.load_log(&file).await?;
        Ok(())
    }

    async fn save(&self) -> anyhow::Result<()> {
        let imp = self.imp();
        let filter = gtk::FileFilter::new();
        // Translators: PCAP is a type of file, do not translate.
        filter.set_property("name", gettext("PCAP Files"));
        filter.add_mime_type("application/vnd.tcpdump.pcap");

        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);

        let mut builder = gtk::FileDialog::builder()
            .title(gettext("Save Log"))
            .filters(&filters)
            .modal(true);
        if let Some(filename) = imp.filename.borrow().as_ref() {
            builder = builder.initial_name(format!("{}.pcap", filename));
        }
        let chooser = builder.build();

        let file = match chooser.save_future(Some(self)).await {
            Err(err) if err.matches(gtk::DialogError::Dismissed) => return Ok(()),
            res => res?,
        };
        let path = file.path().unwrap();
        let message_list = imp
            .filtered_message_model
            .message_list()
            .expect("message list must be set before saving");
        message_list.save_to_file(path).await?;
        // Update the title once the save operation is done
        // removing the `*` prefix
        if let Some(filename) = imp.filename.borrow().as_ref() {
            imp.diagram_title.set_title(filename);
        }
        Ok(())
    }

    async fn save_as_dot(&self) -> Result<()> {
        let imp = self.imp();
        let filter = gtk::FileFilter::new();
        // Translators: Dot is a type of file, do not translate.
        filter.set_property("name", gettext("DOT Graph"));
        filter.add_mime_type("text/vnd.graphviz");

        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);

        let mut builder = gtk::FileDialog::builder()
            .title(gettext("Save Log as DOT Graph File"))
            .filters(&filters)
            .modal(true);
        if let Some(filename) = imp.filename.borrow().as_ref() {
            builder = builder.initial_name(format!("{}.gv", filename));
        }
        let chooser = builder.build();

        let file = match chooser.save_future(Some(self)).await {
            Err(err) if err.matches(gtk::DialogError::Dismissed) => return Ok(()),
            res => res?,
        };
        let message_list = imp
            .filtered_message_model
            .message_list()
            .expect("message list must be set before saving");
        message_list.save_as_dot(&file).await?;

        Ok(())
    }

    fn add_message_toast(&self, message: &str) {
        let toast = adw::Toast::new(message);
        self.imp().toast_overlay.add_toast(toast);
    }

    fn add_error_toast(&self, message: &str) {
        let toast = adw::Toast::builder()
            .title(message)
            .priority(adw::ToastPriority::High)
            .build();
        self.imp().toast_overlay.add_toast(toast);
    }

    fn update_details_view(&self) {
        let imp = self.imp();

        if let Some(message) = imp.diagram.selected_message() {
            imp.details_view.set_message(Some(message));
            imp.details_view_split_view.set_show_sidebar(true);
        } else {
            imp.details_view.set_message(None::<Message>);
            imp.details_view_split_view.set_show_sidebar(false);
        }
    }
}

#[gtk::template_callbacks]
impl Window {
    #[template_callback]
    fn diagram_selected_message_notify(&self) {
        self.update_details_view();
    }

    #[template_callback]
    fn filtered_message_model_items_changed(&self, _position: u32, removed: u32, added: u32) {
        if removed == 0 && added == 0 {
            return;
        }

        let imp = self.imp();
        let is_recording = imp.monitor.borrow().is_some();

        if is_recording {
            let n_messages = imp.filtered_message_model.n_items();

            imp.diagram_title.set_subtitle(&gettext_f(
                // Translators: Do NOT translate the contents between '{' and '}', this is a
                // variable name.
                "Logged {n_messages} messages",
                &[("n_messages", &n_messages.to_string())],
            ));

            if n_messages != 0 {
                imp.diagram_page_stack
                    .set_visible_child(&*imp.split_view_sub_page);
            }
        }
    }

    #[template_callback]
    fn copy_command_clicked(&self) {
        const CMD: &str = "dbus-monitor --pcap";
        self.clipboard().set_text(CMD);
        self.add_message_toast(&gettext("Copied to clipboard"));
    }

    #[template_callback]
    fn details_view_show_message_request(&self, message: &Message) {
        if let Err(err) = self
            .imp()
            .diagram
            .scroll_to(message, gtk::ListScrollFlags::SELECT)
        {
            tracing::error!("Failed to scroll to message: {:?}", err);
        }
    }
}
