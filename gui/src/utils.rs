use gtk4::{gio, glib};

//taken from https://gitlab.gnome.org/GNOME/gnome-tour/-/blob/master/src/utils.rs
pub fn action<T, F>(thing: &T, name: &str, param: Option<&gtk4::glib::VariantTy>, action: F)
where
    T: gio::traits::ActionMapExt,
    for<'r, 's> F: Fn(&'r gio::SimpleAction, Option<&glib::Variant>) + 'static,
{
    //create a stateless parameter action)
    let act = gio::SimpleAction::new(name, param);
    //Connect the handler
    act.connect_activate(action);
    //Add action to the map
    thing.add_action(&act);
}

pub enum ApplicationActions {
    Play,
    PlayTrack,
    Pause,
    Next,
    QueueAppendTrack,
    UpdateState,
}

impl ApplicationActions {
    pub fn call(&self) -> &'static str {
        match self {
            ApplicationActions::QueueAppendTrack => "app.playback-queue-append",
            ApplicationActions::Play => "app.playback-start",
            ApplicationActions::Pause => "app.playback-pause",
            ApplicationActions::PlayTrack => "app.playback-track",
            ApplicationActions::UpdateState => "app.update-playback-state",
            ApplicationActions::Next => "app.playback-next",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ApplicationActions::QueueAppendTrack => "playback-queue-append",
            ApplicationActions::Play => "playback-start",
            ApplicationActions::Pause => "playback-pause",
            ApplicationActions::PlayTrack => "playback-track",
            ApplicationActions::UpdateState => "update-playback-state",
            ApplicationActions::Next => "playback-next",
        }
    }
}

pub fn fmt_duration(duration_ms : i64) -> String {
    let seconds: i64 = duration_ms / 1000;
    let minutes: i64 = seconds / 60;
    let minute_seconds = seconds - (60 * minutes);
    format!("{:>2}:{:0>2}", minutes, minute_seconds)
}