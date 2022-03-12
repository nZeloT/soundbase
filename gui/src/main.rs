use gtk::prelude::{BoxExt, ButtonExt, ListBoxRowExt};
use relm4::{gtk, send, AppUpdate, Model, RelmApp, Sender, WidgetPlus, Widgets, adw, RelmComponent, Components};
use adw::prelude::{AdwApplicationWindowExt, WidgetExt};
use crate::control_pane::ControlPaneModel;
use crate::tracks_page::TracksPageModel;

mod tracks_page;
mod control_pane;
mod model;
mod api;

struct AppModel {
    active_page : StackPages,
}

enum AppMsg {
    Discover,
    Artists,
    Albums,
    Tracks,
    Playlist(i32),
    Playback
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum StackPages {
    Discover,
    Artists,
    Albums,
    Tracks,
    Playlist,
    Playback
}

impl Into<&str> for StackPages {
    fn into(self) -> &'static str {
        match self {
            StackPages::Discover => "discover",
            StackPages::Artists => "artists",
            StackPages::Albums => "albums",
            StackPages::Tracks => "tracks",
            StackPages::Playlist => "playlist",
            StackPages::Playback => "playback",
        }
    }
}

impl From<&str> for StackPages {
    fn from(from : &str) -> StackPages {
        match from {
            "discover" => StackPages::Discover,
            "artists" => StackPages::Artists,
            "albums" => StackPages::Albums,
            "tracks" => StackPages::Tracks,
            "playlist" => StackPages::Playlist,
            "playback" => StackPages::Playback,
            _ => unimplemented!()
        }
    }
}

#[derive(relm4::Components)]
struct AppComponents {
    tracks: RelmComponent<TracksPageModel, AppModel>,
    controls : RelmComponent<ControlPaneModel, AppModel>
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = AppComponents;
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, _components: &AppComponents, _sender: Sender<AppMsg>) -> bool {
        match msg {
            AppMsg::Discover => {
                println!("Discover Message");
                self.active_page = StackPages::Discover;
            },
            AppMsg::Artists => {
                println!("Artists Message");
                self.active_page = StackPages::Artists;
            },
            AppMsg::Albums => {
                println!("Albums Message");
                self.active_page = StackPages::Albums;
            },
            AppMsg::Tracks => {
                println!("Tracks Message");
                self.active_page = StackPages::Tracks;
            },
            AppMsg::Playlist(id) => {
                println!("Playlist Message with id {}", id);
                self.active_page = StackPages::Playlist;
            },
            AppMsg::Playback => {
                println!("Playback Message");
                self.active_page = StackPages::Playback;
            }
        }
        true
    }
}

struct AppWidgets {
    window: adw::ApplicationWindow,
    root_content : gtk::Stack,
}

impl Widgets<AppModel, ()> for AppWidgets {
    type Root = adw::ApplicationWindow;

    /// Initialize the UI.
    fn init_view(model: &AppModel, components: &AppComponents, sender: Sender<AppMsg>) -> Self {
        adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark);
        let window = adw::ApplicationWindow::builder()
            .default_width(800)
            .default_height(600)
            .title("Soundbase Client")
            .build();

        let main_content = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .vexpand(true)
            .hexpand(true)
            .build();

        let header_bar = adw::HeaderBar::builder()
            .show_end_title_buttons(true)
            .title_widget(&adw::WindowTitle::new("Soundbase", ""))
            .build();
        main_content.append(&header_bar);

        let browsing_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .vexpand(true)
            .hexpand(true)
            .build();
        main_content.append(&browsing_box);

        let sidebar_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build();

        let sidebar_content = gtk::Box::builder()
            .spacing(5)
            .orientation(gtk::Orientation::Vertical)
            .build();
        sidebar_content.set_margin_all(5);

        let soundbase_label = gtk::Label::builder()
            .label("Soundbase")
            .css_classes(vec!["large-title".to_string()])
            .margin_start(20)
            .margin_end(20)
            .build();

        sidebar_content.append(&soundbase_label);
        sidebar_content.append(&gtk::Separator::new(gtk::Orientation::Vertical));
        sidebar_content.append(&build_sidebar_main_sections(sender.clone()));
        sidebar_content.append(&gtk::Separator::new(gtk::Orientation::Vertical));
        sidebar_content.append(&build_sidebar_playlists(sender));
        sidebar_box.append(&sidebar_content);
        browsing_box.append(&sidebar_box);
        browsing_box.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

        let content_stack = gtk::Stack::builder()
            .vexpand(true)
            .vhomogeneous(true)
            .build();
        // navigation_bar.set_stack(&content_stack);
        let content_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build();
        content_box.append(&content_stack);
        browsing_box.append(&content_box);


        //Discover Page
        content_stack.add_titled(&gtk::Label::new(Some("Discover Page")), Some("discover"), "Discover");

        //Artists Page
        content_stack.add_titled(&gtk::Label::new(Some("Artists Page")), Some("artists"), "Artists");

        //Album Page
        content_stack.add_titled(&gtk::Label::new(Some("Album Page")), Some("albums"), "Albums");

        //Tracks Page
        content_stack.add_titled(components.tracks.root_widget(), Some("tracks"), "Tracks");

        //Playlist Page
        content_stack.add_titled(&gtk::Label::new(Some("Playlist Page")), Some("playlist"), "Playlist");

        //Playback Page
        content_stack.add_titled(&gtk::Label::new(Some("Playback Page")), Some("playback"), "Playback");


        main_content.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        //build the media control pane

        main_content.append(components.controls.root_widget());

        window.set_content(Some(&main_content));

        Self {
            window,
            root_content: content_stack
        }
    }
    /// Return the root widget
    fn root_widget(&self) -> Self::Root {
        self.window.clone()
    }
    /// Update the view to represent the updated model.
    fn view(&mut self, model: &AppModel, _sender: Sender<AppMsg>) {
        let active_page = self.root_content.visible_child_name().unwrap().to_string();
        if StackPages::from(&*active_page) != model.active_page {
            println!("Setting new stack page '{:?}'", model.active_page);
            self.root_content.set_visible_child_name(model.active_page.into());
        }
    }
}

fn build_sidebar_main_sections(sender: Sender<AppMsg>) -> gtk::ListBox {
    let navigation_bar = gtk::ListBox::builder()
        .width_request(270)
        .selection_mode(gtk::SelectionMode::Browse)
        .valign(gtk::Align::Start)
        .build();
    navigation_bar.set_class_active("navigation-sidebar", true);
    navigation_bar.connect_row_selected(move |_, row| {
        if let Some(row) = row {
            let css_name = row.to_owned().child().unwrap().to_string();
            println!("{}", css_name);
            let msg = match &*css_name {
                "discover" => AppMsg::Discover,
                "artists" =>  AppMsg::Artists,
                "albums"  => AppMsg::Albums,
                "tracks" => AppMsg::Tracks,
                _ => panic!("Failed to resolve main content selection!")
            };
            send!(sender, msg);
        }
    });

    navigation_bar.append(&build_sidebar_list_row("Discover", "discover"));
    navigation_bar.append(&build_sidebar_list_row("Artists", "artists"));
    navigation_bar.append(&build_sidebar_list_row("Albums", "albums"));
    navigation_bar.append(&build_sidebar_list_row("Tracks", "tracks"));

    navigation_bar
}

fn build_sidebar_playlists(sender: Sender<AppMsg>) -> gtk::ScrolledWindow {
    let playlist_bar = gtk::ListBox::builder()
        .width_request(270)
        .vexpand(true)
        .selection_mode(gtk::SelectionMode::Browse)
        .valign(gtk::Align::Start)
        .build();
    playlist_bar.set_class_active("navigation-sidebar", true);
    playlist_bar.connect_row_selected(move |_, row| {
        if let Some(row) = row {
            let css_name = row.to_owned().child().unwrap().to_string();
            println!("{}", css_name);
            let id : i32 = css_name[css_name.rfind('-').unwrap()+1..].parse::<i32>().unwrap();
            send!(sender, AppMsg::Playlist(id));
        }
    });

    let add_playlist = |id : i32, name: &str| {
      let mut css_name = String::from("pl-");
        css_name += id.to_string().as_str();
      build_sidebar_list_row(name, &css_name)
    };

    playlist_bar.append(&add_playlist(0, "Christmas Songs"));
    playlist_bar.append(&add_playlist(1, "Southern Rock"));
    playlist_bar.append(&add_playlist(2, "Hard Rock"));
    playlist_bar.append(&add_playlist(3, "A"));
    playlist_bar.append(&add_playlist(4, "nother"));
    playlist_bar.append(&add_playlist(5, "Playlist"));
    playlist_bar.append(&add_playlist(6, "And"));
    playlist_bar.append(&add_playlist(7, "Another"));
    playlist_bar.append(&add_playlist(8, "One"));
    playlist_bar.append(&add_playlist(9, "right"));
    playlist_bar.append(&add_playlist(10, "here"));
    playlist_bar.append(&add_playlist(11, "to"));
    playlist_bar.append(&add_playlist(12, "occupy"));
    playlist_bar.append(&add_playlist(13, "more"));
    playlist_bar.append(&add_playlist(14, "space"));

    gtk::ScrolledWindow::builder()
        .vexpand(true)
        .child(&playlist_bar)
        .build()
}

fn build_sidebar_list_row(label : &str, name : &str) -> gtk::Box {
    let row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .vexpand(true)
        .halign(gtk::Align::Start)
        .name(name)
        .build();
    row.append(&gtk::Label::new(Some(label)));
    row
}

fn main() {
    let model = AppModel {
        active_page: StackPages::Tracks,
    };
    let app = RelmApp::new(model);
    app.run();
}
