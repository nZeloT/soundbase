use relm4::gtk;
use gtk::{
    glib::{self, ParamSpec, Value},
    prelude::*
};
use std::cell::{Cell, RefCell};
use gtk::subclass::prelude::ObjectSubclass;
use relm4::gtk::subclass::prelude::ObjectImpl;

// The actual data structure that stores our values. This is not accessible
// directly from the outside.
#[derive(Default, Debug)]
pub struct TrackRowData {
    active : Cell<bool>,

    track_id : Cell<i32>,
    album_id : Cell<i32>,
    artist_id : Cell<i32>,

    title : RefCell<Option<String>>,
    album_name : RefCell<Option<String>>,
    faved : Cell<bool>
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
                    "title", "title", "title", None, glib::ParamFlags::READWRITE
                ),
                glib::ParamSpecString::new(
                    "albumName", "albumName", "albumName", None, glib::ParamFlags::READWRITE
                ),
                glib::ParamSpecBoolean::new(
                    "faved", "faved", "faved", false, glib::ParamFlags::READWRITE
                ),
                glib::ParamSpecBoolean::new(
                    "active", "active", "active", false, glib::ParamFlags::READWRITE
                ),
                glib::ParamSpecInt::new(
                    "trackId", "trackId", "trackId",
                    i32::MIN, i32::MAX, 0, glib::ParamFlags::READWRITE
                ),
                glib::ParamSpecInt::new(
                    "albumId", "albumId", "albumId",
                    i32::MIN, i32::MAX, 0, glib::ParamFlags::READWRITE
                ),
                glib::ParamSpecInt::new(
                    "artistId", "artistId", "artistId",
                    i32::MIN, i32::MAX, 0, glib::ParamFlags::READWRITE
                )
            ]
        });

        PROPERTIES.as_ref()
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "title" => {
                let name = value.get().unwrap();
                self.title.replace(name);
            },
            "albumName" => {
                let name = value.get().unwrap();
                self.album_name.replace(name);
            },
            "faved" => {
                let faved = value.get().unwrap();
                self.faved.replace(faved);
            },
            "active" => {
                let active = value.get().unwrap();
                self.active.replace(active);
            },
            "trackId" => {
                let track_id = value.get().unwrap();
                self.track_id.replace(track_id);
            },
            "albumId" => {
                let album_id = value.get().unwrap();
                self.album_id.replace(album_id);
            },
            "artistId" => {
                let artist_id = value.get().unwrap();
                self.artist_id.replace(artist_id);
            },
            _ => unimplemented!()
        }
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "title" => self.title.borrow().to_value(),
            "albumName" => self.album_name.borrow().to_value(),
            "faved" => self.faved.get().to_value(),
            "active" => self.active.get().to_value(),
            "trackId" => self.track_id.get().to_value(),
            "albumId" => self.album_id.get().to_value(),
            "artistId" => self.artist_id.get().to_value(),
            _ => unimplemented!()
        }
    }
}