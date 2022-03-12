use relm4::gtk;
use gtk::{
    glib::{self, ParamSpec, Value},
    prelude::*,
};
use std::cell::{Cell, RefCell, RefMut};
use gtk::subclass::prelude::ObjectSubclass;
use relm4::gtk::glib::ValueArray;
use relm4::gtk::subclass::prelude::ObjectImpl;

// The actual data structure that stores our values. This is not accessible
// directly from the outside.
#[derive(Debug)]
pub struct TrackRowData {
    track_id: Cell<i32>,
    album_id: Cell<i32>,
    artist_id: Cell<i32>,

    title: RefCell<Option<String>>,
    album_name: RefCell<Option<String>>,
    album_fmt: RefCell<Option<String>>,
    artist_ids: RefCell<ValueArray>,
    artist_names: RefCell<ValueArray>,
    artist_fmt : RefCell<Option<String>>,
    faved: Cell<bool>,
    faved_icon: RefCell<Option<String>>,
    duration_ms: Cell<i64>,
    duration_fmt: RefCell<Option<String>>,
}

impl Default for TrackRowData {
    fn default() -> Self {
        Self{
            track_id: Cell::new(0),
            album_id: Cell::new(0),
            artist_id: Cell::new(0),

            title: RefCell::new(None),
            album_name: RefCell::new(None),
            album_fmt: RefCell::new(None),
            artist_ids: RefCell::new(ValueArray::new(0)),
            artist_names: RefCell::new(ValueArray::new(0)),
            artist_fmt: RefCell::new(None),
            faved : Cell::new(false),
            faved_icon: RefCell::new(None),
            duration_ms: Cell::new(0),
            duration_fmt: RefCell::new(None)
        }
    }
}

// Basic declaration of our type for the GObject type system
#[glib::object_subclass]
impl ObjectSubclass for TrackRowData {
    const NAME: &'static str = "TrackRowData";
    type Type = super::TrackRowData;
}

// The ObjectImpl trait provides the setters/getters for GObject properties.
// Here we need to provide the values that are internally stored back to the
// caller, or store whatever new value the caller is providing.
//
// This maps between the GObject properties and our internal storage of the
// corresponding values of the properties.
impl ObjectImpl for TrackRowData {
    fn properties() -> &'static [ParamSpec] {
        use once_cell::sync::Lazy;

        static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
            vec![
                glib::ParamSpecString::new(
                    "title", "title", "title", None, glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecString::new(
                    "albumName", "albumName", "albumName", None, glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecString::new(
                    "albumFmt", "albumFmt", "albumFmt", None, glib::ParamFlags::READABLE,
                ),
                glib::ParamSpecValueArray::new(
                    "artistIds", "artistIds", "artistIds",
                    &glib::ParamSpecInt::new("artistId", "artistId", "artistId", i32::MIN, i32::MAX, -1, glib::ParamFlags::READWRITE),
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecValueArray::new(
                    "artistNames", "artistNames", "artistNames",
                    &glib::ParamSpecString::new("artistName", "artistName", "artistName", None, glib::ParamFlags::READWRITE),
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecString::new(
                    "artistFmt", "artistFmt", "artistFmt", None, glib::ParamFlags::READABLE,
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
                glib::ParamSpecInt::new(
                    "albumId", "albumId", "albumId",
                    i32::MIN, i32::MAX, 0, glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecInt::new(
                    "artistId", "artistId", "artistId",
                    i32::MIN, i32::MAX, 0, glib::ParamFlags::READWRITE,
                ),
            ]
        });

        PROPERTIES.as_ref()
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "title" => {
                let name = value.get().unwrap();
                self.title.replace(name);
            }
            "albumName" => {
                let name = value.get().unwrap();
                let fmt = match name {
                    Some(ref val) => {
                        let fmt = format!("<a href=\"{}\">{}</a>", self.album_id.get(), val);
                        Some(fmt)
                    }
                    None => None
                };
                self.album_name.replace(name);
                self.album_fmt.replace(fmt);
            }
            "artistIds" => {
                let ids = value.get().unwrap();
                self.artist_ids.replace(ids);
                self.recalc_artist_fmt();
            }
            "artistNames" => {
                let names = value.get().unwrap();
                self.artist_names.replace(names);
                self.recalc_artist_fmt();
            }
            "faved" => {
                let faved = value.get().unwrap();
                self.faved.replace(faved);
                let icon_name = if faved { "starred-symbolic" } else { "non-starred-symbolic" };
                self.faved_icon.replace(Some(icon_name.to_string()));
            }
            "durationMs" => {
                let duration = value.get().unwrap();
                self.duration_ms.replace(duration);
                self.duration_fmt.replace(Some(self.fmt_duration(duration)));
            }
            "trackId" => {
                let track_id = value.get().unwrap();
                self.track_id.replace(track_id);
            }
            "albumId" => {
                let album_id = value.get().unwrap();
                self.album_id.replace(album_id);
            }
            "artistId" => {
                let artist_id = value.get().unwrap();
                self.artist_id.replace(artist_id);
            }
            _ => unimplemented!()
        }
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "title" => self.title.borrow().to_value(),
            "albumName" => self.album_name.borrow().to_value(),
            "albumFmt" => self.album_fmt.borrow().to_value(),
            "artistIds" => self.artist_ids.borrow().to_value(),
            "artistNames" => self.artist_names.borrow().to_value(),
            "artistFmt" => self.artist_fmt.borrow().to_value(),
            "faved" => self.faved.get().to_value(),
            "favedIcon" => self.faved_icon.borrow().to_value(),
            "durationMs" => self.duration_ms.get().to_value(),
            "durationFmt" => self.duration_fmt.borrow().to_value(),
            "trackId" => self.track_id.get().to_value(),
            "albumId" => self.album_id.get().to_value(),
            "artistId" => self.artist_id.get().to_value(),
            _ => unimplemented!()
        }
    }
}

impl TrackRowData {
    fn fmt_duration(&self, duration_ms: i64) -> String {
        let seconds: i64 = duration_ms / 1000;
        let minutes: i64 = seconds / 60;
        let minute_seconds = seconds - (60 * minutes);
        format!("{:>2}:{:0<2}", minutes, minute_seconds)
    }

    fn recalc_artist_fmt(&self) {
        let ids = self.artist_ids.borrow_mut();
        let names = self.artist_names.borrow_mut();
        if ids.len() == names.len() {
            let fmt = self.fmt_artists(ids, names);
            self.artist_fmt.replace(Some(fmt));
        }
    }

    fn fmt_artists(&self, ids : RefMut<ValueArray>, names : RefMut<ValueArray>) -> String {
        let ids = ids.to_vec();
        let names = names.to_vec();

        let mut links: Vec<String> = vec![];
        for idx in 0..ids.len() {
            let id : i32 = ids[idx].get().unwrap();
            let name : String = names[idx].get().unwrap();
            let link = format!("<a href=\"{}\">{}</a>", id, name);
            links.push(link);
        }
        links.join(", ")
    }
}