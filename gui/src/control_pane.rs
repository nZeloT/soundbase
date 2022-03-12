use adw::prelude::WidgetExt;
use gtk::prelude::{BoxExt, ButtonExt};
use relm4::{adw, AppUpdate, Components, ComponentUpdate, gtk, Model, RelmApp, RelmComponent, send, Sender, WidgetPlus, Widgets};
use relm4::gtk::prelude::RangeExt;

use crate::{AppModel, AppMsg};
use crate::model::TrackRowData::TrackRowData;

pub struct ControlPaneModel {
    current_track : TrackRowData
}

pub enum ControlPaneMsg {
    NextTrack,
    PreviousTrack,
    Seek(u32),
    ActivateShuffle,
    DeactivateShuffle,
    LoopAll,
    LoopOne,
    SetVolume(u32),
    ShowQueue,
    HideQueue,
    ShowSnapcastConf,
    HideSnapcastConf
}

impl Model for ControlPaneModel {
    type Msg = ControlPaneMsg;
    type Widgets = ControlPaneWidgets;
    type Components = ();
}

impl ComponentUpdate<AppModel> for ControlPaneModel {
    fn init_model(parent_model: &AppModel) -> Self {
        let current_track = TrackRowData::new(0, "Sweet Home Alabama", 0, "Second Helping", true);
        Self {
            current_track
        }
    }

    fn update(&mut self, msg: Self::Msg, components: &Self::Components, sender: Sender<Self::Msg>, parent_sender: Sender<AppMsg>) {
        //use _parent_sender to send messages to the app window
    }
}


pub struct ControlPaneWidgets {
    root : gtk::CenterBox
}

impl Widgets<ControlPaneModel, AppModel> for ControlPaneWidgets {
    type Root = gtk::CenterBox;

    fn init_view(model: &ControlPaneModel, _components: &(), sender: Sender<ControlPaneMsg>) -> Self {
        let root = gtk::CenterBox::builder()
            .orientation(gtk::Orientation::Horizontal)
            .hexpand(true)
            .build();
        root.set_height_request(96);

        //1. media meta data
        let metadata_box = build_metadata_box();

        //2. control panel
        let control_panel = build_control_box();

        //3. additional controls
        let additional_controls = build_additional_controls();

        root.set_start_widget(Some(&metadata_box));
        root.set_center_widget(Some(&control_panel));
        root.set_end_widget(Some(&additional_controls));

        Self {
            root
        }
    }

    fn root_widget(&self) -> Self::Root {
        self.root.clone()
    }

    fn view(&mut self, model: &ControlPaneModel, sender: Sender<ControlPaneMsg>) {
        todo!()
    }
}

fn build_metadata_box() -> gtk::Box {
    let album_art = gtk::Button::builder()
        .icon_name("media-optical-symbolic")
        .build();
    album_art.add_css_class("flat");
    album_art.set_valign(gtk::Align::Center);
    album_art.set_height_request(50);
    album_art.set_width_request(50);

    let title_label = gtk::Label::builder()
        .use_markup(true)
        .halign(gtk::Align::Start)
        .label("<a href=\"title\">Title</a>")
        .max_width_chars(100)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .build();
    title_label.add_css_class("caption-heading");

    let artist_label = gtk::Label::builder()
        .use_markup(true)
        .halign(gtk::Align::Start)
        .label("<a href=\"artist\">Artist</a>")
        .max_width_chars(100)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .build();

    let starred_button = gtk::Button::from_icon_name("non-starred-symbolic");
    starred_button.add_css_class("flat");
    starred_button.set_valign(gtk::Align::Center);

    let metadata_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .valign(gtk::Align::Center)
        .build();
    metadata_box.append(&album_art);

    let nested_metadata = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .valign(gtk::Align::Center)
        .spacing(3)
        .build();
    nested_metadata.set_margin_end(5);
    nested_metadata.set_margin_start(5);
    nested_metadata.append(&title_label);
    nested_metadata.append(&artist_label);
    metadata_box.append(&nested_metadata);
    metadata_box.append(&starred_button);
    metadata_box.set_margin_start(10);

    metadata_box
}

fn build_control_box() -> gtk::Box {
    let controls = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .valign(gtk::Align::Center)
        .build();

    let shuffle = gtk::ToggleButton::builder()
        .icon_name("media-playlist-shuffle-symbolic")
        .valign(gtk::Align::Center)
        .build();
    shuffle.add_css_class("flat");

    let previous = gtk::Button::builder()
        .icon_name("media-skip-backward-symbolic")
        .valign(gtk::Align::Center)
        .build();
    previous.add_css_class("flat");

    let playback = gtk::Button::builder()
        .icon_name("media-playback-start-symbolic")
        .width_request(48)
        .height_request(48)
        .build();

    let next = gtk::Button::builder()
        .icon_name("media-skip-forward-symbolic")
        .valign(gtk::Align::Center)
        .build();
    next.add_css_class("flat");

    let btn_loop = gtk::ToggleButton::builder()
        .icon_name("media-playlist-repeat-symbolic")
        .valign(gtk::Align::Center)
        .build();
    btn_loop.add_css_class("flat");

    let time_gone = gtk::Label::new(Some("0:00"));
    let duration = gtk::Label::new(Some("8:30"));
    let seeking = gtk::Scale::builder()
        .orientation(gtk::Orientation::Horizontal)
        .hexpand(true)
        .build();
    seeking.set_range(0 as f64, 100 as f64);
    seeking.set_value(50 as f64);

    let btn_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(10)
        .hexpand(true)
        .halign(gtk::Align::Center)
        .build();
    btn_box.append(&shuffle);
    btn_box.append(&previous);
    btn_box.append(&playback);
    btn_box.append(&next);
    btn_box.append(&btn_loop);

    let seek_box = gtk::CenterBox::builder()
        .orientation(gtk::Orientation::Horizontal)
        .hexpand(true)
        .build();
    seek_box.set_start_widget(Some(&time_gone));
    seek_box.set_center_widget(Some(&seeking));
    seek_box.set_end_widget(Some(&duration));

    controls.append(&btn_box);
    controls.append(&seek_box);

    controls
}

fn build_additional_controls() -> gtk::Box {
    let addition = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();

    let show_queue = gtk::ToggleButton::builder()
        .icon_name("view-list-symbolic")
        .valign(gtk::Align::Center)
        .build();

    let show_snap_conf = gtk::ToggleButton::builder()
        .icon_name("drive-multidisk-symbolic")
        .valign(gtk::Align::Center)
        .build();

    let mute = gtk::ToggleButton::builder()
        .icon_name("audio-volume-high-symbolic")
        .valign(gtk::Align::Center)
        .build();

    let volume_bar = gtk::Scale::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();
    volume_bar.set_range(0 as f64, 100 as f64);
    volume_bar.set_value(100 as f64);
    volume_bar.set_width_request(96);

    let btn_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();
    btn_box.add_css_class("linked");
    btn_box.append(&show_queue);
    btn_box.append(&show_snap_conf);
    btn_box.append(&mute);

    addition.set_spacing(5);
    addition.set_margin_end(10);
    addition.append(&btn_box);
    addition.append(&volume_bar);

    addition
}