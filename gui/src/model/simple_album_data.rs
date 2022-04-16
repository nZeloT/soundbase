use std::borrow::Borrow;
use std::cell::RefCell;
use gtk4::{glib};
use gtk4::subclass::prelude::ObjectSubclassIsExt;

use crate::api::services::{SimpleAlbum, SimpleArtist};
use crate::model::simple_album_data::imp::SimpleAlbumDataInt;

mod imp {
    use std::cell::RefCell;
    use std::rc::Rc;
    use gtk4::glib;
    use gtk4::glib::{ParamSpec, Value};
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;
    use crate::utils;

    #[derive(Debug, Default)]
    pub struct SimpleAlbumDataInt {
        pub(super) album_id : i32,

        pub(super) name : String,
    }

    #[derive(Default)]
    pub struct SimpleAlbumData {
        pub data : Rc<RefCell<SimpleAlbumDataInt>>
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SimpleAlbumData {
        const NAME: &'static str = "SimpleAlbumData";
        type Type = super::SimpleAlbumData;
    }

    impl ObjectImpl for SimpleAlbumData {
        fn properties() -> &'static [ParamSpec] {
            use once_cell::sync::Lazy;

            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::new(
                        "name", "name", "name", None, glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecInt::new(
                        "album-id", "album-id", "album-id",
                        i32::MIN, i32::MAX, 0, glib::ParamFlags::READWRITE,
                    ),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "album-id" => {
                    let album_id = value.get().unwrap();
                    self.data.borrow_mut().album_id = album_id;
                },
                "name" => {
                    let name = value.get().unwrap();
                    self.data.borrow_mut().name = name;
                },
                _ => panic!("Tried to set unknown property {:?}", pspec.name())
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "album-id" => self.data.borrow().album_id.to_value(),
                "name" => self.data.borrow().name.to_value(),
                _ => panic!("Tried to read unknown property {:?}", pspec.name())
            }
        }
    }
}

glib::wrapper! {
    pub struct SimpleAlbumData(ObjectSubclass<imp::SimpleAlbumData>);
}

impl SimpleAlbumData {
    pub fn new(album : Option<SimpleAlbum>) -> Self {
        match album {
            Some(album) => {
                let album_fmt = album.name.replace('&', "&amp;");

                glib::Object::new(&[
                    ("album-id", &album.album_id),
                    ("name", &album_fmt),
                ])
                    .expect("Failed to create new SimpleAlbumData!")
            },
            None => SimpleAlbumData::default()
        }
    }

    pub fn album_id(&self) -> i32 {
        let data : &RefCell<SimpleAlbumDataInt> = self.imp().data.borrow();
        data.borrow().album_id
    }
}

impl Default for SimpleAlbumData {
    fn default() -> Self {
        glib::Object::new(&[
            ("albumId", &0),
            ("name", &"Default Album Name"),
        ])
            .expect("Failed to create new SimpleAlbumData!")
    }
}