use glib::Object;
use gtk4::{glib};
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use crate::api::{LibraryApi, PlaybackApi, AsyncRuntime};
use crate::api::services::PlaybackStateResponse;
use crate::application::Application;

mod imp {
    use gtk4::glib;
    use glib::subclass::InitializingObject;
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;
    use gtk4::{CompositeTemplate};
    use adw::subclass::prelude::AdwApplicationWindowImpl;
    use crate::ui::list_tracks_page::ListTracksPage;
    use crate::ui::list_albums_page::ListAlbumsPage;
    use crate::ui::playback_page::PlaybackPage;
    use crate::ui::playback_pane::PlaybackPane;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/nzelot/soundbase-gui/main_window.ui")]
    pub struct MainWindow {
        #[template_child]
        pub(super) stack_pages : TemplateChild<gtk4::Stack>,

        #[template_child]
        pub(super) list_tracks_page : TemplateChild<ListTracksPage>,

        #[template_child]
        pub(super) list_albums_page : TemplateChild<ListAlbumsPage>,

        #[template_child]
        pub(super) playback_page : TemplateChild<PlaybackPage>,

        #[template_child]
        pub(super) main_content_selector : TemplateChild<gtk4::ListBox>,

        #[template_child]
        pub(super) playlist_selection : TemplateChild<gtk4::ListBox>,

        #[template_child]
        pub(super) playback_pane : TemplateChild<PlaybackPane>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MainWindow {
        const NAME: &'static str = "MainWindow";
        type Type = super::MainWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            ListTracksPage::static_type();
            ListAlbumsPage::static_type();
            PlaybackPage::static_type();
            PlaybackPane::static_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MainWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            //add listeners for button clicks and similar
            self.main_content_selector.connect_row_selected(glib::clone!(@weak obj => move |_box, opt_row| {
                if let Some(row) = opt_row {
                    let named_box = row.child()
                        .expect("Some row didn't have a child!");
                    let name = named_box.widget_name();
                        // .expect("Unnamed Box in Sidebar!")
                    log::info!("Selected Page with name {}", name.as_str());

                    obj.imp().stack_pages.set_visible_child_name(&name);
                }
            }));
        }
    }

    impl WidgetImpl for MainWindow {}
    impl WindowImpl for MainWindow {}
    impl ApplicationWindowImpl for MainWindow {}
    impl AdwApplicationWindowImpl for MainWindow {}
}

glib::wrapper!{
    pub struct MainWindow(ObjectSubclass<imp::MainWindow>)
        @extends adw::ApplicationWindow, gtk4::ApplicationWindow, gtk4::Window, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Native,
                    gtk4::Root, gtk4::ShortcutManager;
}

impl MainWindow {
    pub fn new(app : &Application) -> Self {
        Object::new(&[("application", app)])
            .expect("Failed to create MainWindow")
    }

    pub fn init_api_for_pages(&self, async_runtime : &AsyncRuntime, api_library: &LibraryApi, api_playback : &PlaybackApi) {
        self.imp().list_tracks_page.get().init_api(api_library);
        self.imp().list_albums_page.get().init_api(api_library);
        self.imp().playback_page.get().init_api(api_playback);
        self.imp().playback_pane.get().init(async_runtime);
    }

    pub fn propagate_playback_state(&self, new_state : &PlaybackStateResponse) {
        log::info!("Received State Update, Propagating: {:?}!", new_state);
        self.imp().playback_pane.update_state(new_state);
    }
}
