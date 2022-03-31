use gtk4::prelude::ObjectExt;
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use crate::api::services::SimpleTrack;
use crate::model::track_data::TrackData;

mod imp {
    use std::cell::{Cell, RefCell};
    use adw::glib::Value;

    use gtk4::glib;
    use gtk4::glib::ParamSpec;
    use gtk4::prelude::{StaticType, ToValue};
    use gtk4::subclass::prelude::{ObjectImpl, ObjectSubclass};

    use crate::model::track_data::TrackData;

    #[derive(Default)]
    pub struct PlaybackState {
        pub(super) has_current_track : Cell<bool>,
        pub(super) current_track : RefCell<TrackData>,
        pub(super) is_playing : Cell<bool>,
        pub(super) playing_icon : RefCell<String>,
        pub(super) has_previous : Cell<bool>,
        pub(super) has_next : Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlaybackState {
        const NAME: &'static str = "PlaybackState";
        type Type = super::PlaybackState;
    }

    impl ObjectImpl for PlaybackState {
        fn properties() -> &'static [ParamSpec] {
            use once_cell::sync::Lazy;

            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecBoolean::new(
                        "has-current-track", "has-current-track", "has-current-track", false, glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecObject::new(
                        "current-track", "current-track", "current-track", TrackData::static_type(), glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecBoolean::new(
                        "is-playing", "is-playing", "is-playing", false, glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecString::new(
                        "playing-icon", "playing-icon", "playing-icon", Some("media-playback-start-symbolic"), glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecBoolean::new(
                        "has-previous", "has-previous", "has-previous", false, glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecBoolean::new(
                        "has-next", "has-next", "has-next", false, glib::ParamFlags::READWRITE,
                    ),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "has-current-track" => {
                    let has_track = value.get().unwrap();
                    self.has_current_track.replace(has_track);
                },
                "current-track" => {
                    let track = value.get().unwrap();
                    self.current_track.replace(track);
                },
                "is-playing" => {
                    let is_playing = value.get().unwrap();
                    self.is_playing.replace(is_playing);
                },
                "playing-icon" => {
                    let playing_icon = value.get().unwrap();
                    self.playing_icon.replace(playing_icon);
                },
                "has-previous" => {
                    let has_previous = value.get().unwrap();
                    self.has_previous.replace(has_previous);
                },
                "has-next" => {
                    let has_next = value.get().unwrap();
                    self.has_next.replace(has_next);
                },
                _ => unimplemented!()
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "has-current-track" => self.has_current_track.get().to_value(),
                "current-track" => self.current_track.borrow().to_value(),
                "is-playing" => self.is_playing.get().to_value(),
                "playing-icon" => self.playing_icon.borrow().to_value(),
                "has-previous" => self.has_previous.get().to_value(),
                "has-next" => self.has_next.get().to_value(),
                _ => unimplemented!()
            }
        }
    }
}

gtk4::glib::wrapper!{
    pub struct PlaybackState(ObjectSubclass<imp::PlaybackState>);
}

impl PlaybackState {
    pub fn new() -> Self {
        gtk4::glib::Object::new(&[
            ("playing-icon", &"media-playback-start-symbolic")
        ])
            .expect("Failed to create new PlaybackState!")
    }

    pub fn set_current_track(&self, new_track : &SimpleTrack) {
        self.set_property("current-track", &TrackData::new(Some(new_track.clone())));
    }

    pub fn set_has_current_track(&self, has_track : bool) {
        self.set_property("has-current-track", has_track);
    }

    pub fn is_playing(&self) -> bool {
        self.imp().is_playing.get()
    }

    pub fn set_is_playing(&self, is_playing : bool) {
        self.set_property("is-playing", is_playing);
        let playing_icon = if is_playing {
            "media-playback-pause-symbolic"
        }else {
            "media-playback-start-symbolic"
        };
        self.set_property("playing-icon", playing_icon);
    }

    pub fn set_has_next(&self, has_next : bool) {
        self.set_property("has-next", has_next);
    }

    pub fn set_has_previous(&self, has_previous : bool) {
        self.set_property("has-previous", has_previous);
    }
}

impl Default for PlaybackState {
    fn default() -> Self {
        PlaybackState::new()
    }
}