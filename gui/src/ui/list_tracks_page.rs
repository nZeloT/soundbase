use glib::Object;
use gtk4::{glib};
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use crate::api::{LibraryApi};

use crate::api::services::{SimpleTrack};
use crate::model::track_data::TrackData;

mod imp {
    use std::cell::Cell;
    use gtk4::{BuilderListItemFactory, glib, ListView, SingleSelection};
    use glib::subclass::InitializingObject;
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;
    use gtk4::{CompositeTemplate};
    use gtk4::gio::ListStore;
    use once_cell::sync::OnceCell;
    use crate::api::{LibraryApi};
    use crate::model::track_data::TrackData;
    use crate::ui::list_track_row::ListTrackRow;


    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/nzelot/soundbase-gui/list_tracks_page.ui")]
    pub struct ListTracksPage {
        #[template_child]
        pub(super) track_list: TemplateChild<ListStore>,

        #[template_child]
        pub(super) track_selection_model: TemplateChild<SingleSelection>,

        #[template_child]
        pub(super) track_list_view: TemplateChild<ListView>,

        #[template_child]
        pub(super) track_item_factory: TemplateChild<BuilderListItemFactory>,

        #[template_child]
        pub(super) scrolled_window : TemplateChild<gtk4::ScrolledWindow>,

        pub(super) api_library: OnceCell<LibraryApi>,
        pub(super) loaded_count: Cell<i32>,
        pub(super) new_load_offset: Cell<i32>,
        pub(super) can_load: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ListTracksPage {
        const NAME: &'static str = "ListTracksPage";
        type Type = super::ListTracksPage;
        type ParentType = gtk4::Box;

        fn class_init(klass: &mut Self::Class) {
            ListTrackRow::static_type();
            TrackData::static_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ListTracksPage {
        fn constructed(&self, obj: &Self::Type) {
            self.scrolled_window.connect_edge_reached(glib::clone!(@weak obj => move |_scr_wd, pos_type|{
                if pos_type == gtk4::PositionType::Bottom {
                    obj.trigger_track_load();
                }
            }));

            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for ListTracksPage {}

    impl BoxImpl for ListTracksPage {}
}

glib::wrapper! {
    pub struct ListTracksPage(ObjectSubclass<imp::ListTracksPage>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Native;
}

impl ListTracksPage {
    pub fn new() -> Self {
        Object::new(&[])
            .expect("Failed to create ListTracksPage!")
    }

    pub fn init_api(&self, api_library: &LibraryApi) {
        self.imp().api_library.set(api_library.clone())
            .expect("API can only be set once!");
        self.update_can_load();
        self.trigger_track_load();
    }

    fn trigger_track_load(&self) {
        log::info!("Load Additional tracks!");
        match self.imp().api_library.get() {
            Some(api) => {
                let can_load = self.imp().can_load.get();
                if can_load {
                    let offset = self.imp().new_load_offset.get();
                    const LIMIT : i32 = 50;

                    let page = self;
                    api.load_tracks(offset, LIMIT, glib::clone!(@weak page => move |simple_track| {
                        log::info!("Received Simple Track in Callback!");
                        page.append_track(simple_track);
                    })).unwrap();

                    self.imp().new_load_offset.set(offset + LIMIT);
                    self.update_can_load();
                }
            }
            None => log::error!("API not initialized!")
        }
    }

    fn append_track(&self, track: SimpleTrack) {
        let count = self.imp().loaded_count.get();
        self.imp().track_list.append(&TrackData::new(Some(track)));
        self.imp().loaded_count.set(count + 1);
        self.update_can_load();
    }

    fn update_can_load(&self) {
        let count = self.imp().loaded_count.get();
        let requested = self.imp().new_load_offset.get();
        self.imp().can_load.set(count == requested);
    }
}