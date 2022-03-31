use adw::gtk;
use crate::{config, utils};
use crate::ui::main_window::MainWindow;
use adw::prelude::*;
use gtk4::{
    gio,
    glib::{self},
    subclass::prelude::*,
};
use crate::api::{ApiRuntime, LibraryApi, PlaybackApi};

mod imp {
    use super::*;
    use adw::subclass::prelude::*;
    use gtk4::glib::{once_cell::sync::OnceCell, WeakRef};
    use crate::api::LibraryApi;

    #[derive(Debug, Default)]
    pub struct Application {
        pub(super) main_window: OnceCell<WeakRef<MainWindow>>,

        pub(super) api_library: OnceCell<LibraryApi>,
        pub(super) api_playback: OnceCell<PlaybackApi>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "Application";
        type Type = super::Application;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for Application {}

    impl ApplicationImpl for Application {
        fn activate(&self, application: &Self::Type) {
            adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark);
            let window = MainWindow::new(application);
            application.add_window(&window);
            window.present();
            self.main_window.set(window.downgrade()).unwrap();
            application.distribute_api_connections();
            application.init_ui_state();
            self.parent_activate(application);
        }

        fn startup(&self, application: &Self::Type) {
            application.setup_api_connections();
            application.setup_actions();
            self.parent_startup(application);
        }
    }

    impl GtkApplicationImpl for Application {}

    impl AdwApplicationImpl for Application {}
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl Application {
    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &config::APP_ID),
            ("resource-base-path", &Some("/org/nzelot/soundbase-gui"))
        ])
            .unwrap()
    }

    pub fn run() {
        log::info!("Soundbase-Gui ({})", config::APP_ID);
        let app = Self::new();
        gtk::prelude::ApplicationExtManual::run(&app);
    }

    fn window(&self) -> MainWindow {
        self.imp().main_window.get().and_then(|w| w.upgrade()).unwrap()
    }

    fn setup_actions(&self) {
        let imp = self.imp();
        let playback_api = imp.api_playback.get()
            .expect("Playback API not initialized!");
        let api = playback_api.clone();
        utils::action(
            self,
            "playback-queue-append",
            Some(&i32::static_variant_type()),
            move |_action, param| {

                let track_id = param
                    .expect("Queue Append Action Expected Track Id Parameter!")
                    .get::<i32>()
                    .expect("Failed to pared parameter to i32!");

                api.queue_append(track_id, move || {
                    log::info!("Successfully appended Track to Queue!");
                }).expect("Couldn't call API!");
            }
        );

        let api = playback_api.clone();
        let app = self;
        utils::action(
            self,
            "playback-start",
            None,
            glib::clone!(@weak app => move |_action, _param| {

                api.play(glib::clone!(@weak app => move |state_update| {
                    app.window().propagate_playback_state(&state_update);
                })).expect("Failed to send Play Message!");
            }
        ));

        let api = playback_api.clone();
        utils::action(
            self,
            "playback-pause",
            None,
            glib::clone!(@weak app => move |_action, _param| {

                    api.pause(glib::clone!(@weak app => move |state_update| {
                        app.window().propagate_playback_state(&state_update);
                    })).expect("Failed to send Pause Message!");
                })
        );

        let api = playback_api.clone();
        utils::action(self, "update-playback-state", None, glib::clone!(@weak app => move |_action, _param| {
            log::info!("Action Update Playback State is called!");
            api.current_state(glib::clone!(@weak app => move |state_update| {
                app.window().propagate_playback_state(&state_update);
            })).expect("Failed to Send Current State Message!");
        }));
    }

    fn setup_api_connections(&self) {
        let api_address = std::env::var("API_URL")
            .unwrap_or("http://philly.local:3333".to_string());
        let api_runtime = ApiRuntime::new();
        let api_library = LibraryApi::new(api_runtime.clone(), api_address.clone());
        let api_playback = PlaybackApi::new(api_runtime, api_address.clone());
        self.imp().api_library.set(api_library.clone()).unwrap();
        self.imp().api_playback.set(api_playback.clone()).unwrap();
        log::info!("Connected to {}", api_address);
    }

    fn distribute_api_connections(&self) {
        let api_library = self.imp().api_library.get()
            .expect("Library API is not initialized!");

        let api_playback = self.imp().api_playback.get()
            .expect("Playback API is not initialized!");

        self.window().init_api_for_pages(api_library, api_playback);
    }

    fn init_ui_state(&self) {
        log::info!("Init UI state!");
        self.activate_action("update-playback-state", None)
    }
}