use std::borrow::Borrow;
use std::cell::RefCell;
use gtk4::{glib};
use gtk4::subclass::prelude::ObjectSubclassIsExt;

use crate::api::services::{SimpleTrack, SimpleArtist};
use crate::model::track_data::imp::TrackDataInt;

mod imp {
    use std::cell::RefCell;
    use std::rc::Rc;
    use gtk4::glib;
    use gtk4::glib::{ParamSpec, Value};
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;

    #[derive(Debug, Default)]
    pub struct TrackDataInt {
        pub(super) track_id : i32,

        pub(super) title : String,
        pub(super) album_fmt : String,
        pub(super) artist_fmt : String,

        pub(super) is_faved : bool,
        pub(super) faved_icon : String,

        pub(super) duration_ms : i64,
        pub(super) duration_fmt : String,
    }

    #[derive(Default)]
    pub struct TrackData {
        pub data : Rc<RefCell<TrackDataInt>>
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TrackData {
        const NAME: &'static str = "TrackData";
        type Type = super::TrackData;
    }

    impl ObjectImpl for TrackData {
        fn properties() -> &'static [ParamSpec] {
            use once_cell::sync::Lazy;

            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::new(
                        "title", "title", "title", None, glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecString::new(
                        "albumFmt", "albumFmt", "albumFmt", None, glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecString::new(
                        "artistFmt", "artistFmt", "artistFmt", None, glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecBoolean::new(
                        "faved", "faved", "faved", false, glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecString::new(
                        "favedIcon", "favedIcon", "favedIcon", Some("non-starred-symbolic"), glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpecInt64::new(
                        "durationMs", "durationMs", "durationMs", i64::MIN, i64::MAX, 0, glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecString::new(
                        "durationFmt", "durationFmt", "durationFmt", None, glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpecInt::new(
                        "trackId", "trackId", "trackId",
                        i32::MIN, i32::MAX, 0, glib::ParamFlags::READWRITE,
                    ),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "trackId" => {
                    let track_id = value.get().unwrap();
                    self.data.borrow_mut().track_id = track_id;
                },
                "title" => {
                    let title = value.get().unwrap();
                    self.data.borrow_mut().title = title;
                },
                "albumFmt" => {
                    let fmt = value.get().unwrap();
                    self.data.borrow_mut().album_fmt = fmt;
                },
                "artistFmt" => {
                    let fmt = value.get().unwrap();
                    self.data.borrow_mut().artist_fmt = fmt;
                },
                "faved" => {
                    let faved = value.get().unwrap();
                    self.data.borrow_mut().is_faved = faved;
                    self.data.borrow_mut().faved_icon = (if faved { "starred-symbolic" } else { "non-starred-symbolic" }).to_string();
                },
                "durationMs" => {
                    let duration = value.get().unwrap();
                    self.data.borrow_mut().duration_ms = duration;
                    self.data.borrow_mut().duration_fmt = TrackData::fmt_duration(duration);
                },
                _ => panic!("Tried to set unknown property {:?}", pspec.name())
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "trackId" => self.data.borrow().track_id.to_value(),
                "title" => self.data.borrow().title.to_value(),
                "albumFmt" => self.data.borrow().album_fmt.to_value(),
                "artistFmt" => self.data.borrow().artist_fmt.to_value(),
                "faved" => self.data.borrow().is_faved.to_value(),
                "favedIcon" => self.data.borrow().faved_icon.to_value(),
                "durationMs" => self.data.borrow().duration_ms.to_value(),
                "durationFmt" => self.data.borrow().duration_fmt.to_value(),
                _ => panic!("Tried to read unknown property {:?}", pspec.name())
            }
        }
    }

    impl TrackData {
        fn fmt_duration(duration_ms : i64) -> String {
            let seconds: i64 = duration_ms / 1000;
            let minutes: i64 = seconds / 60;
            let minute_seconds = seconds - (60 * minutes);
            format!("{:>2}:{:0<2}", minutes, minute_seconds)
        }
    }
}

glib::wrapper! {
    pub struct TrackData(ObjectSubclass<imp::TrackData>);
}

impl TrackData {
    pub fn new(track : Option<SimpleTrack>) -> Self {
        match track {
            Some(track) => {
                let album_fmt = match track.album {
                    Some(ref album) => format!("<a href=\"{}\">{}</a>", album.album_id, album.name),
                    None => "".to_string()
                };
                let album_fmt = album_fmt.replace('&', "&amp;");
                let artist_fmt = TrackData::fmt_artists(&track).replace('&', "&amp;");
                let title_fmt = track.title.replace('&', "&amp;");

                glib::Object::new(&[
                    ("trackId", &track.track_id),
                    ("title", &title_fmt),
                    ("albumFmt", &album_fmt),
                    ("artistFmt", &artist_fmt),
                    ("faved", &track.is_faved),
                    ("durationMs", &track.duration_ms)
                ])
                    .expect("Failed to create new TrackData!")
            },
            None => TrackData::default()
        }
    }

    pub fn track_id(&self) -> i32 {
        let data : &RefCell<TrackDataInt> = self.imp().data.borrow();
        data.borrow().track_id
    }

    fn fmt_artists(track : &SimpleTrack) -> String {
        let artists : &Vec<SimpleArtist> = &track.artists;

        let mut links: Vec<String> = vec![];
        for artist in artists {
            let link = format!("<a href=\"{}\">{}</a>", artist.artist_id, artist.name);
            links.push(link);
        }
        links.join(", ")
    }
}

impl Default for TrackData {
    fn default() -> Self {
        glib::Object::new(&[
            ("trackId", &0),
            ("title", &"Default Title"),
            ("albumFmt", &"Default Album"),
            ("artistFmt", &"Default Artist"),
            ("faved", &false),
            ("durationMs", &(0 as i64))
        ])
            .expect("Failed to create new TrackData!")
    }
}