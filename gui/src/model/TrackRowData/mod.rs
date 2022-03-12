mod imp;

use relm4::gtk;
use gtk::glib;
use relm4::gtk::glib::{ToValue, Value};
use relm4::gtk::prelude::ObjectExt;

glib::wrapper! {
    pub struct TrackRowData(ObjectSubclass<imp::TrackRowData>);
}

impl TrackRowData {
    pub fn new(id: i32, title: &str, album : (i32, &str), artists : Vec<(i32, String)>, faved: bool, duration_ms : i64) -> Self {
        let mut artist_ids = glib::ValueArray::new(artists.len() as u32);
        let mut artist_names = glib::ValueArray::new(artists.len() as u32);
        for (id, name) in artists {
            artist_ids.append(&id.to_value());
            artist_names.append(&name.to_value());
        }

        glib::Object::new(&[
            ("title", &title),
            ("faved", &faved),
            ("durationMs", &duration_ms),
            ("trackId", &id),
            ("albumId", &album.0),
            ("albumName", &album.1),
            ("artistIds", &artist_ids),
            ("artistNames", &artist_names)
        ])
            .expect("Failed to create track data!")
    }

    pub fn set_active(&self, active: bool) {
        self.set_property("active", active);
    }

    pub fn get_track_id(&self) -> i32 { self.property::<i32>("trackId") }
    pub fn get_album_id(&self) -> i32 { self.property::<i32>("albumId") }
    pub fn get_artist_id(&self) -> i32 { self.property::<i32>("artistId") }
}

impl Default for TrackRowData {
    fn default() -> Self {
        TrackRowData::new(0, "Default", (0, "Default Album"), vec![], false, 0)
    }
}