use glib::Object;
use gtk4::{glib};
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use crate::api::{LibraryApi};

use crate::api::services::{SimpleAlbum};
use crate::model::simple_album_data::SimpleAlbumData;

mod imp {
    use std::cell::Cell;
    use gtk4::{BuilderListItemFactory, glib, GridView, SingleSelection};
    use glib::subclass::InitializingObject;
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;
    use gtk4::{CompositeTemplate};
    use gtk4::gio::ListStore;
    use once_cell::sync::OnceCell;
    use crate::api::{LibraryApi};
    use crate::model::simple_album_data::SimpleAlbumData;
    use crate::ui::list_albums_item_element::ListAlbumsItemElement;


    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/nzelot/soundbase-gui/list_albums_page.ui")]
    pub struct ListAlbumsPage {
        #[template_child]
        pub(super) album_list: TemplateChild<ListStore>,

        #[template_child]
        pub(super) album_selection_model: TemplateChild<SingleSelection>,

        #[template_child]
        pub(super) album_grid_view: TemplateChild<GridView>,

        #[template_child]
        pub(super) album_item_factory: TemplateChild<BuilderListItemFactory>,

        #[template_child]
        pub(super) scrolled_window : TemplateChild<gtk4::ScrolledWindow>,

        pub(super) api_library: OnceCell<LibraryApi>,
        pub(super) loaded_count: Cell<i32>,
        pub(super) new_load_offset: Cell<i32>,
        pub(super) can_load: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ListAlbumsPage {
        const NAME: &'static str = "ListAlbumsPage";
        type Type = super::ListAlbumsPage;
        type ParentType = gtk4::Box;

        fn class_init(klass: &mut Self::Class) {
            ListAlbumsItemElement::static_type();
            SimpleAlbumData::static_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ListAlbumsPage {
        fn constructed(&self, obj: &Self::Type) {
            self.scrolled_window.connect_edge_reached(glib::clone!(@weak obj => move |_scr_wd, pos_type|{
                if pos_type == gtk4::PositionType::Bottom {
                    obj.trigger_album_load();
                }
            }));

            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for ListAlbumsPage {}

    impl BoxImpl for ListAlbumsPage {}
}

glib::wrapper! {
    pub struct ListAlbumsPage(ObjectSubclass<imp::ListAlbumsPage>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Native;
}

impl ListAlbumsPage {
    pub fn new() -> Self {
        Object::new(&[])
            .expect("Failed to create ListAlbumsPage!")
    }

    pub fn init_api(&self, api_library: &LibraryApi) {
        self.imp().api_library.set(api_library.clone())
            .expect("API can only be set once!");
        self.update_can_load();
        self.trigger_album_load();
    }

    fn trigger_album_load(&self) {
        log::info!("Load Additional albums!");
        match self.imp().api_library.get() {
            Some(api) => {
                let can_load = self.imp().can_load.get();
                if can_load {
                    let offset = self.imp().new_load_offset.get();
                    const LIMIT : i32 = 50;

                    let page = self;
                    api.load_albums(offset, LIMIT, glib::clone!(@weak page => move |simple_album| {
                        log::info!("Received Simple Album in Callback!");
                        page.append_album(simple_album);
                    })).unwrap();

                    self.imp().new_load_offset.set(offset + LIMIT);
                    self.update_can_load();
                }
            }
            None => log::error!("API not initialized!")
        }
    }

    fn append_album(&self, album: SimpleAlbum) {
        let count = self.imp().loaded_count.get();
        self.imp().album_list.append(&SimpleAlbumData::new(Some(album)));
        self.imp().loaded_count.set(count + 1);
        self.update_can_load();
    }

    fn update_can_load(&self) {
        let count = self.imp().loaded_count.get();
        let requested = self.imp().new_load_offset.get();
        self.imp().can_load.set(count == requested);
    }
}