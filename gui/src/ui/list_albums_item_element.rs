use gtk4::{glib};
use glib::Object;
use gtk4::prelude::{ObjectExt, ToVariant};
use gtk4::traits::WidgetExt;
use crate::model::simple_album_data::SimpleAlbumData;
use crate::utils;

mod imp {
    use std::borrow::Borrow;
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use glib::{ParamSpec, Value, WeakRef};
    use gtk4::{glib, ListItem};
    use glib::subclass::InitializingObject;
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;
    use gtk4::{CompositeTemplate};
    use gtk4::glib::SignalHandlerId;
    use crate::model::simple_album_data::SimpleAlbumData;


    #[derive(CompositeTemplate, Default)]
    #[template(resource="/org/nzelot/soundbase-gui/list_albums_grid_item_element.ui")]
    pub struct ListAlbumsItemElement {
        pub(super) list_item : Rc<RefCell<WeakRef<gtk4::ListItem>>>,

        pub(super) is_highlighted : Cell<bool>,

        pub(super) selected_signal_handler : RefCell<Option<SignalHandlerId>>,

        #[template_child]
        pub(super) lbl_name : TemplateChild<gtk4::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ListAlbumsItemElement {
        const NAME: &'static str = "ListAlbumsItemElement";
        type Type = super::ListAlbumsItemElement;
        type ParentType = gtk4::Box;

        fn class_init(klass: &mut Self::Class) {
            SimpleAlbumData::static_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ListAlbumsItemElement {
        fn properties() -> &'static [ParamSpec] {
            use once_cell::sync::Lazy;

            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "list-item", "list-item", "list-item", gtk4::ListItem::static_type(), glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecBoolean::new(
                        "is-highlighted", "is-highlighted", "is-highlighted", false, glib::ParamFlags::READWRITE
                    ),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "list-item" => {
                    let item : &RefCell<WeakRef<ListItem>> = self.list_item.borrow();
                    let item = item.borrow().upgrade();
                    match item {
                        Some(ref item) => {
                            if let Some(id) = self.selected_signal_handler.borrow_mut().take() {
                                item.disconnect(id);
                            }
                        }
                        None => {}
                    }

                    let list_item : gtk4::ListItem = value.get().unwrap();
                    let obj = obj.clone();
                    let signal_handler = list_item.connect_selected_notify(move |item| {
                        obj.set_property("is-highlighted", &item.is_selected());
                    });
                    self.selected_signal_handler.replace(Some(signal_handler));
                    self.list_item.replace(list_item.downgrade());
                },
                "is-highlighted" => {
                    let is_highlighted = value.get().unwrap();
                    self.is_highlighted.replace(is_highlighted);
                },
                _ => unimplemented!()
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "list-item" => {
                    let item : &RefCell<WeakRef<ListItem>> = self.list_item.borrow();
                    item.borrow().upgrade().to_value()
                },
                "is-highlighted" => self.is_highlighted.get().to_value(),
                _ => unimplemented!()
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            let motion_controller = gtk4::EventControllerMotion::new();
            // let stack = &self.playback_action_stack;
            let item = self.list_item.clone();
            let obj_enter = obj.clone();
            motion_controller.connect_enter(move |_ctrl, _pos_x, _pos_y| {
                obj_enter.set_property("is-highlighted", &true);
            });

            let obj_leave = obj.clone();
            motion_controller.connect_leave(move |_ctrl| {
                let item : &RefCell<WeakRef<ListItem>> = item.borrow();
                let item = item.borrow().upgrade();
                match item {
                    Some(item) => {
                        obj_leave.set_property("is-highlighted", &item.is_selected());
                    },
                    None => {}
                }
            });
            obj.add_controller(&motion_controller);

            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for ListAlbumsItemElement {}
    impl BoxImpl for ListAlbumsItemElement {}
}

glib::wrapper!{
    pub struct ListAlbumsItemElement(ObjectSubclass<imp::ListAlbumsItemElement>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Native;
}

impl ListAlbumsItemElement {
    pub fn new() -> Self {
        Object::new(&[])
            .expect("Failed to create ListTrackRow!")
    }
}