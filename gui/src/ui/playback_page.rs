use gtk4::glib;
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use crate::api::{PlaybackApi};

use crate::api::services::SimpleTrack;
use crate::model::track_data::TrackData;

mod imp {
    use std::cell::Cell;
    use gtk4::{glib, ListView, ScrolledWindow, SingleSelection};
    use glib::subclass::InitializingObject;
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;
    use gtk4::{CompositeTemplate};
    use gtk4::gio::ListStore;
    use once_cell::sync::OnceCell;
    use crate::api::{PlaybackApi};
    use crate::model::track_data::TrackData;
    use crate::ui::list_track_row::ListTrackRow;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/nzelot/soundbase-gui/playback_page.ui")]
    pub struct PlaybackPage {
        #[template_child]
        pub(super) queue_list : TemplateChild<ListStore>,

        #[template_child]
        pub(super) queue_selection_model : TemplateChild<SingleSelection>,

        #[template_child]
        pub(super) queue_list_view : TemplateChild<ListView>,

        #[template_child]
        pub(super) scrolled_window : TemplateChild<ScrolledWindow>,

        #[template_child]
        pub(super) refresh_queue : TemplateChild<gtk4::Button>,

        pub(super) api : OnceCell<PlaybackApi>,
        pub(super) loaded_count : Cell<i32>,
        pub(super) new_load_offset : Cell<i32>,
        pub(super) can_load : Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlaybackPage {
        const NAME: &'static str = "PlaybackPage";
        type Type = super::PlaybackPage;
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

    impl ObjectImpl for PlaybackPage {
        fn constructed(&self, obj: &Self::Type) {

            self.refresh_queue.connect_clicked(glib::clone!(@weak obj => move |_btn| {
                log::info!("Refreshing Queue!");
                obj.clear_queue();
                obj.trigger_track_load();
            }));

            self.parent_constructed(obj);
        }
    }
    impl BoxImpl for PlaybackPage {}
    impl WidgetImpl for PlaybackPage {}

}

glib::wrapper!(
    pub struct PlaybackPage(ObjectSubclass<imp::PlaybackPage>)
    @extends gtk4::Box, gtk4::Widget,
    @implements gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Native;
);

impl PlaybackPage {
    pub fn new() -> Self {
        glib::Object::new(&[])
            .expect("Failed to create Playback Page")
    }

    pub fn init_api(&self, api : &PlaybackApi) {
        self.imp().api.set(api.clone())
            .expect("API can only be set once!");
        self.update_can_load();
        self.trigger_track_load();
    }

    fn clear_queue(&self) {
        let store = self.imp().queue_list.get();
        store.remove_all();
    }

    fn trigger_track_load(&self) {
        log::info!("Loading Queued Tracks!");
        match self.imp().api.get() {
            Some(api) => {
                let offset = 0;
                const LIMIT : i32 = 50;

                let page = self;
                api.queue_load(offset, LIMIT, glib::clone!(@weak page => move |simple_track| {
                    log::info!("Received Simple Track for Queue in Callback!");
                    page.append_track(simple_track);
                })).expect("Failed to trigger Queue Load!");
            },
            None => log::error!("API not initialized!")
        }
    }

    fn append_track(&self, track : SimpleTrack) {
        let imp = self.imp();
        let count = imp.loaded_count.get();
        imp.queue_list.append(&TrackData::new(Some(track)));
        imp.loaded_count.set(count + 1);
        self.update_can_load();
    }

    fn update_can_load(&self) {
        let imp = self.imp();
        imp.can_load.set(
            imp.loaded_count.get() == imp.new_load_offset.get()
        );
    }
}