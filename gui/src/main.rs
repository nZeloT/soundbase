mod api;
mod model;
mod ui;
mod application;
mod config;
mod utils;

fn main() {
    pretty_env_logger::init();
    gtk4::gio::resources_register_include!("soundbase.gresource")
        .expect("Failed to register resources!");

    application::Application::run()
}