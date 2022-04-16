use gtk4::prelude::ObjectExt;
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use crate::api::services::SimpleTrack;
use crate::model::track_data::TrackData;
use crate::utils;
use crate::api::services::PlaybackStateResponse;

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

        pub(super) playback_track_start : Cell<i64>,
        pub(super) playback_pos_fmt : RefCell<String>,
        pub(super) playback_progress : Cell<i32>,
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
                    glib::ParamSpecInt64::new(
                        "playback-track-start", "playback-track-start", "playback-track-start", i64::MIN, i64::MAX, 0 as i64, glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecString::new(
                        "playback-pos-fmt", "playback-pos-fmt", "playback-pos-fmt", Some("0:00"), glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecInt::new(
                        "playback-progress", "playback-progress", "playback-progress", i32::MIN, i32::MAX, 0 as i32, glib::ParamFlags::READWRITE,
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
                "playback-track-start" => {
                    let playback_pos_ms = value.get().unwrap();
                    self.playback_track_start.replace(playback_pos_ms);
                },
                "playback-pos-fmt" => {
                    let playback_pos_fmt = value.get().unwrap();
                    self.playback_pos_fmt.replace(playback_pos_fmt);
                },
                "playback-progress" => {
                    let playback_progress = value.get().unwrap();
                    self.playback_progress.replace(playback_progress);
                }
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
                "playback-track-start" => self.playback_track_start.get().to_value(),
                "playback-pos-fmt" => self.playback_pos_fmt.borrow().to_value(),
                "playback-progress" => self.playback_progress.get().to_value(),
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

    pub fn set_playback_track_start(&self, new_start_time : i64) {
        let delta_time = chrono::offset::Utc::now().timestamp_millis() - new_start_time;
        self.set_property("playback-track-start", new_start_time);
        self.set_property("playback-pos-fmt", utils::fmt_duration(delta_time));
        self.set_property("playback-progress", 0);
    }

    pub fn has_current_track(&self) -> bool {
        self.imp().has_current_track.get()
    }

    pub fn update_from_server_response(&self, state_update : &PlaybackStateResponse) {
        log::info!("Received Playback State Update {:?}", state_update);
        let mut song_change = false;
        let old_track_id = self.imp().current_track.borrow().track_id();

        self.set_is_playing(state_update.is_playing);
        self.set_has_previous(state_update.has_previous);
        self.set_has_next(state_update.has_next);
        let has_current_track = state_update.playing_track.is_some();

        self.set_has_current_track(has_current_track);

        if let Some(track) = &state_update.playing_track {
            log::info!("Updating current track!");
            song_change = track.track_id != old_track_id && track.track_id != 0;
            self.set_current_track(track);
        }

        self.update_playback_position(song_change);
    }

    pub fn update_playback_position(&self, track_changed : bool) {
        if track_changed {
            self.set_playback_track_start(chrono::offset::Utc::now().timestamp_millis());
        }else{
            let delta_time = chrono::offset::Utc::now().timestamp_millis() - self.imp().playback_track_start.get();
            self.set_property("playback-pos-fmt", utils::fmt_duration(delta_time));

            let dur_ms = self.imp().current_track.borrow().duration_ms();
            let progress : i32 = (( (delta_time as f64) / (dur_ms as f64) ) * 100 as f64) as i32;
            self.set_property("playback-progress", progress);
        }
    }
}

impl Default for PlaybackState {
    fn default() -> Self {
        PlaybackState::new()
    }
}