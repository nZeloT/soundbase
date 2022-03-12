use adw::prelude::WidgetExt;
use gtk::prelude::{BoxExt, ButtonExt};
use relm4::{adw, AppUpdate, Components, ComponentUpdate, gtk, Model, RelmApp, RelmComponent, RelmMsgHandler, send, Sender, WidgetPlus, Widgets};
use relm4::gtk::gio::ListStore;
use relm4::gtk::glib;
use relm4::gtk::glib::clone;
use relm4::gtk::prelude::{Cast, GObjectPropertyExpressionExt, ListModelExt, SelectionModelExt, StaticType};
use relm4::gtk::Widget;
use relm4::gtk::glib::signal::Inhibit;

use crate::{AppModel, AppMsg};
use crate::api::{AsyncLibraryHandler, AsyncLibraryHandlerMsg, AsyncLibraryKind};
use crate::gtk::ListItem;
use crate::model::TrackRowData::TrackRowData;
use crate::api::services::SimpleTrack as ApiSimpleTrack;

pub struct TracksPageModel {
    model: ListStore,

    can_request: bool,
    track_request_offset: i32,
    track_count: i32,
}

pub enum TracksPageMsg {
    GoToArtist(i32),
    GoToAlbum(i32),
    ToggleFavTrack(i32),
    AddToQueue(i32),

    TriggerLoad,
    AddApiTrack(ApiSimpleTrack),
}

pub struct TracksPageComponents {
    async_api: RelmMsgHandler<AsyncLibraryHandler, TracksPageModel>,
}

impl Components<TracksPageModel> for TracksPageComponents {
    fn init_components(parent_model: &TracksPageModel, parent_sender: Sender<TracksPageMsg>) -> Self {
        Self {
            async_api: RelmMsgHandler::new(parent_model, parent_sender)
        }
    }

    fn connect_parent(&mut self, _parent_widgets: &TracksPageWidgets) {}
}

impl Model for TracksPageModel {
    type Msg = TracksPageMsg;
    type Widgets = TracksPageWidgets;
    type Components = TracksPageComponents;
}

impl ComponentUpdate<AppModel> for TracksPageModel {
    fn init_model(_parent_model: &AppModel) -> Self {
        let model = ListStore::new(TrackRowData::static_type());
        Self {
            model,
            can_request: true,
            track_request_offset: 0,
            track_count: 0,
        }
    }

    fn update(&mut self, msg: Self::Msg, components: &Self::Components, _sender: Sender<Self::Msg>, _parent_sender: Sender<AppMsg>) {
        //use _parent_sender to send messages to the app window
        match msg {
            TracksPageMsg::TriggerLoad => {
                if !self.can_request {
                    println!("Requested all known tracks");
                    return;
                }
                println!("Requesting Tracks ...");
                let receiver = components.async_api.sender();
                receiver.blocking_send(
                    AsyncLibraryHandlerMsg::LoadPage(
                        AsyncLibraryKind::Track, self.track_request_offset, 50))
                    .expect("Async Receiver Dropped!");
                self.track_request_offset += 50;
                self.can_request = false;
            }

            TracksPageMsg::AddApiTrack(new_api_track) => {
                println!("Received a new track on tracks page");
                let album = new_api_track.album.unwrap();
                let artists: Vec<(i32, String)> = new_api_track.artists.iter()
                    .map(|simple_artist| (simple_artist.artist_id, simple_artist.name.clone()))
                    .collect::<Vec<_>>();
                self.model.append(&TrackRowData::new(
                    new_api_track.track_id,
                    &*new_api_track.title,
                    (album.album_id, &*album.name),
                    artists,
                    new_api_track.is_faved,
                    new_api_track.duration_ms,
                ));
                self.track_count += 1;
                self.can_request = self.track_count == self.track_request_offset;
            }

            TracksPageMsg::GoToArtist(artist_id) => {
                println!("Going to artist with id: {}", artist_id);
            }
            TracksPageMsg::GoToAlbum(album_id) => {
                println!("Going to album with id: {}", album_id);
            }
            TracksPageMsg::AddToQueue(track_id) => {
                println!("Adding Track with id {} to Queue", track_id);
            }
            TracksPageMsg::ToggleFavTrack(track_id) => {
                println!("Toggel Fav State for track with id {}", track_id);
            }
        }
    }
}

pub struct TracksPageWidgets {
    root: gtk::Box,
}

impl Widgets<TracksPageModel, AppModel> for TracksPageWidgets {
    type Root = gtk::Box;

    fn init_view(model: &TracksPageModel, _components: &TracksPageComponents, sender: Sender<TracksPageMsg>) -> Self {
        let root = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .vexpand(true)
            .hexpand(true)
            .build();

        let factory = gtk::SignalListItemFactory::new();

        // the "setup" stage is used for creating the widgets
        let fac_sender = sender.clone();
        factory.connect_setup(move |_factory, item| {
            create_track_row(item, fac_sender.clone());
        });

        let selection_model = gtk::SingleSelection::new(Some(&model.model));

        let list_view = gtk::ListView::new(Some(&selection_model), Some(&factory));

        list_view.set_margin_all(10);
        list_view.set_hexpand(true);
        list_view.set_vexpand(true);

        let scrolled_window = gtk::ScrolledWindow::builder()
            .child(&list_view)
            .min_content_width(400)
            .vexpand(true)
            .build();

        let edge_sender = sender.clone();
        scrolled_window.connect_edge_reached(move |_scrolled_window, _pos_type| {
            send!(edge_sender, TracksPageMsg::TriggerLoad);
        });

        root.append(&scrolled_window);

        //Request an initial set of tracks to fill the page
        send!(sender, TracksPageMsg::TriggerLoad);
        Self {
            root
        }
    }

    fn root_widget(&self) -> Self::Root {
        self.root.clone()
    }

    fn view(&mut self, model: &TracksPageModel, sender: Sender<TracksPageMsg>) {
        //todo!()
    }
}

fn create_track_row(item: &gtk::ListItem, sender: Sender<TracksPageMsg>) {
    let hbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(5)
        .hexpand(true)
        .build();
    hbox.set_margin_all(2);

    let stack = gtk::Stack::new();

    let album_art = gtk::Image::builder()
        .icon_name("media-optical-symbolic")
        .build();

    let btn_play = gtk::Button::builder()
        .icon_name("media-playback-start-symbolic")
        .valign(gtk::Align::Center)
        .build();
    btn_play.add_css_class("flat");
    stack.add_named(&album_art, Some("album-art"));
    stack.add_named(&btn_play, Some("media-control"));
    stack.set_visible_child_name("album-art");
    hbox.append(&stack);

    let meta_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(5)
        .hexpand(true)
        .build();

    let lbl_title = gtk::Label::new(None);
    lbl_title.set_halign(gtk::Align::Start);
    lbl_title.add_css_class("caption-heading");
    meta_box.append(&lbl_title);

    let meta_detail_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(5)
        .hexpand(true)
        .build();

    let lbl_artists = gtk::Label::new(None);
    lbl_artists.set_halign(gtk::Align::Start);
    lbl_artists.set_use_markup(true);
    let artist_sender = sender.clone();
    lbl_artists.connect_activate_link(move |_label, url| {
        let artist_id = match url.parse::<i32>() {
            Ok(id) => id,
            Err(e) => panic!("This should not happen! {:?}", e)
        };
        artist_sender.send(TracksPageMsg::GoToArtist(artist_id)).unwrap();
        Inhibit(false)
    });

    let lbl_album = gtk::Label::new(Some("Album"));
    lbl_album.set_use_markup(true);
    lbl_album.set_halign(gtk::Align::Start);
    let album_sender = sender.clone();
    lbl_album.connect_activate_link(move |_label, url| {
        let album_id = match url.parse::<i32>() {
            Ok(id) => id,
            Err(e) => panic!("This should not happen! {:?}", e)
        };
        album_sender.send(TracksPageMsg::GoToAlbum(album_id)).unwrap();
        Inhibit(false)
    });

    meta_detail_box.append(&lbl_artists);
    meta_detail_box.append(&lbl_album);
    meta_detail_box.set_valign(gtk::Align::Start);
    meta_box.append(&meta_detail_box);
    hbox.append(&meta_box);

    let btn_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();
    btn_box.set_class_active("linked", true);

    let is_faved = gtk::ToggleButton::builder()
        .icon_name("non-starred-symbolic")
        .valign(gtk::Align::Center)
        .build();
    let is_fav_sender = sender.clone();
    is_faved.connect_clicked(clone!(@weak item => move |_btn| {
        let track_id = get_track_row_data(&item).get_track_id();
        is_fav_sender.send(TracksPageMsg::ToggleFavTrack(track_id)).unwrap()
    }));
    btn_box.set_visible(false);
    btn_box.append(&is_faved);

    let btn_add_to_queue = gtk::Button::builder()
        .icon_name("list-add-symbolic")
        .valign(gtk::Align::Center)
        .build();
    let append_fac = sender.clone();
    btn_add_to_queue.connect_clicked(clone!(@weak item => move |_btn|{
        let track_id = get_track_row_data(&item).get_track_id();
        append_fac.send(TracksPageMsg::AddToQueue(track_id)).unwrap();
    }));
    btn_box.append(&btn_add_to_queue);

    let btn_more = gtk::Button::builder()
        .icon_name("view-more-symbolic")
        .valign(gtk::Align::Center)
        .build();
    btn_more.connect_clicked(|_btn| {
        //TODO create popover menu
    });
    btn_box.append(&btn_more);
    hbox.append(&btn_box);

    let lbl_duration = gtk::Label::new(Some("8:30"));
    lbl_duration.set_width_request(40);
    lbl_duration.set_halign(gtk::Align::End);
    hbox.append(&lbl_duration);

    let motion_controller = gtk::EventControllerMotion::new();
    motion_controller.connect_enter(clone!(@weak btn_box, @weak stack => move |_ctlr, _pos_x, _pos_y| {
        btn_box.set_visible(true);
        stack.set_visible_child_name("media-control");
    }));
    motion_controller.connect_leave(clone!(@weak btn_box, @weak stack, @weak item => move |_ctrl| {
        btn_box.set_visible(item.is_selected());
        if item.is_selected() {
            stack.set_visible_child_name("media-control");
        }else{
            stack.set_visible_child_name("album-art");
        }
    }));
    hbox.add_controller(&motion_controller);
    item.set_child(Some(&hbox));
    item.connect_selected_notify(clone!(@weak btn_box, @weak stack => move |item| {
        btn_box.set_visible(item.is_selected());
        if item.is_selected() {
            stack.set_visible_child_name("media-control");
        }else{
            stack.set_visible_child_name("album-art");
        }
    }));

    item
        .property_expression("item")
        .chain_property::<TrackRowData>("title")
        .bind(&lbl_title, "label", Widget::NONE);

    item
        .property_expression("item")
        .chain_property::<TrackRowData>("albumFmt")
        .bind(&lbl_album, "label", Widget::NONE);

    item
        .property_expression("item")
        .chain_property::<TrackRowData>("artistFmt")
        .bind(&lbl_artists, "label", Widget::NONE);

    item
        .property_expression("item")
        .chain_property::<TrackRowData>("durationFmt")
        .bind(&lbl_duration, "label", Widget::NONE);

    item
        .property_expression("item")
        .chain_property::<TrackRowData>("favedIcon")
        .bind(&is_faved, "icon-name", Widget::NONE);

    item
        .property_expression("item")
        .chain_property::<TrackRowData>("faved")
        .bind(&is_faved, "active", Widget::NONE);
}

fn get_track_row_data(item: &ListItem) -> TrackRowData {
    item.item().unwrap().downcast::<TrackRowData>().unwrap()
}