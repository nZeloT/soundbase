use gtk4::glib;
use gtk4::prelude::WidgetExt;
use gtk4::subclass::prelude::ObjectSubclassIsExt;

use crate::api::services::PlaybackStateResponse;
use crate::api::AsyncRuntime;
use crate::utils;

mod imp {
    use std::cell::RefCell;

    use adw::glib::Value;
    use glib::subclass::InitializingObject;
    use gtk4::glib;
    use gtk4::glib::ParamSpec;
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;
    use gtk4::CompositeTemplate;

    use crate::model::playback_state::PlaybackState;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/nzelot/soundbase-gui/playback_pane.ui")]
    pub struct PlaybackPane {
        #[template_child]
        pub(super) metadata_stack: TemplateChild<gtk4::Stack>,
        #[template_child]
        pub(super) current_track_metadata: TemplateChild<gtk4::Box>,
        #[template_child]
        pub(super) current_track_album_art: TemplateChild<gtk4::Image>,
        #[template_child]
        pub(super) current_track_title: TemplateChild<gtk4::Label>,
        #[template_child]
        pub(super) current_track_artists: TemplateChild<gtk4::Label>,
        #[template_child]
        pub(super) current_track_faved: TemplateChild<gtk4::Button>,

        #[template_child]
        pub(super) playback_previous: TemplateChild<gtk4::Button>,
        #[template_child]
        pub(super) playback_play_pause: TemplateChild<gtk4::Button>,
        #[template_child]
        pub(super) playback_next: TemplateChild<gtk4::Button>,
        #[template_child]
        pub(super) playback_time_passed: TemplateChild<gtk4::Label>,
        #[template_child]
        pub(super) playback_seeking: TemplateChild<gtk4::Scale>,
        #[template_child]
        pub(super) playback_time_total: TemplateChild<gtk4::Label>,

        pub(super) playback_state: RefCell<PlaybackState>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlaybackPane {
        const NAME: &'static str = "PlaybackPane";
        type Type = super::PlaybackPane;
        type ParentType = gtk4::Box;

        fn class_init(klass: &mut Self::Class) {
            PlaybackState::static_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PlaybackPane {
        fn properties() -> &'static [ParamSpec] {
            use once_cell::sync::Lazy;

            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
                    "state",
                    "state",
                    "state",
                    PlaybackState::static_type(),
                    glib::ParamFlags::READWRITE,
                )]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "state" => {
                    let state = value.get().unwrap();
                    self.playback_state.replace(state);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "state" => self.playback_state.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.playback_play_pause
                .connect_clicked(glib::clone!(@weak obj => move |_btn|{
                    obj.toggle_playback_state();
                }));
            self.playback_next
                .connect_clicked(glib::clone!(@weak obj => move |_btn| {
                    obj.next_track();
                }));
            self.parent_constructed(obj);
        }
    }
    impl WidgetImpl for PlaybackPane {}
    impl BoxImpl for PlaybackPane {}
}

glib::wrapper! {
    pub struct PlaybackPane(ObjectSubclass<imp::PlaybackPane>)
    @extends gtk4::Box, gtk4::Widget,
    @implements gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Native;
}

impl PlaybackPane {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create PlaybackPane!")
    }

    pub fn init(&self, async_runtime: &AsyncRuntime) {
        //launch a task which updates the playback position once a second
        let (glix_tx, glib_rx) = gtk4::glib::MainContext::channel(gtk4::glib::PRIORITY_DEFAULT);
        {
            let obj = self;
            let context = gtk4::glib::MainContext::default();
            let _guard = context
                .acquire()
                .expect("Couldn't acquire glib main context!");
            glib_rx.attach(
                Some(&context),
                glib::clone!(@weak obj => @default-panic, move |_| {
                    obj.imp().playback_state.borrow().update_playback_position(false);
                    glib::Continue(true)
                }),
            );
        }

        async_runtime.rt().spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
            loop{
                interval.tick().await;
                glix_tx.send(()).expect("Failed to send to GLib Main Context!");
            }
        });
    }

    pub fn update_state(&self, state_update: &PlaybackStateResponse) {
        self.imp().playback_state.borrow().update_from_server_response(state_update);
        let child_name = if self.imp().playback_state.borrow().has_current_track() {
            "has_current_track"
        } else {
            "has_no_track"
        };
        self.imp().metadata_stack.set_visible_child_name(child_name);
    }

    fn toggle_playback_state(&self) {
        let is_playing = self.imp().playback_state.borrow().is_playing();
        if is_playing {
            self.pause_playback();
        } else {
            self.start_playback();
        }
    }

    fn start_playback(&self) {
        self.activate_action(utils::ApplicationActions::Play.call(), None)
            .expect("Failed to activate playback-start Action!");
    }

    fn pause_playback(&self) {
        self.activate_action(utils::ApplicationActions::Pause.call(), None)
            .expect("Failed to activate playback-pause Action!");
    }

    fn next_track(&self) {
        self.activate_action(utils::ApplicationActions::Next.call(), None)
            .expect("Failed to activate Next Track Action!");
    }

    fn trigger_state_update(&self) {
        self.activate_action(utils::ApplicationActions::UpdateState.call(), None)
            .expect("Failed to activate Update Playback State Action!");
    }
}
