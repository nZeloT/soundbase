use gtk4::{glib, gio};

//taken from https://gitlab.gnome.org/GNOME/gnome-tour/-/blob/master/src/utils.rs
pub fn action<T, F>(thing : &T, name : &str, param : Option<&gtk4::glib::VariantTy>, action : F)
where T: gio::traits::ActionMapExt,
      for<'r, 's> F: Fn(&'r gio::SimpleAction, Option<&glib::Variant>) + 'static,
{
    //create a stateless parameter action)
    let act = gio::SimpleAction::new(name, param);
    //Connect the handler
    act.connect_activate(action);
    //Add action to the map
    thing.add_action(&act);
}