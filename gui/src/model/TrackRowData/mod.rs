mod imp;

use relm4::gtk;
use gtk::glib;
use relm4::gtk::prelude::ObjectExt;

glib::wrapper! {
    pub struct TrackRowData(ObjectSubclass<imp::TrackRowData>);
}

impl TrackRowData {
    pub fn new(id: i32, title: &str, album_id: i32, album_name: &str, faved: bool) -> Self {
        //TODO use addition parameters
        glib::Object::new(&[
            ("title", &title),
            ("faved", &faved),
            ("trackId", &id),
            ("albumId", &album_id),
            ("albumName", &album_name)
        ])
            .expect("Failed to create track data!")
    }

    pub fn set_active(&self, active: bool) {
        self.set_property("active", active);
    }

    pub fn get_track_id(&self) -> i32 { self.property::<i32>("trackId") }
    pub fn get_album_id(&self) -> i32 { self.property::<i32>("albumId") }
    pub fn get_artist_id(&self) -> i32 { self.property::<i32>("artistId") }

    pub fn get_album_name(&self) -> String { self.property::<String>("albumName") }
}

impl Default for TrackRowData {
    fn default() -> Self {
        TrackRowData::new(0, "Default", 0, "Default Album", false)
    }
}