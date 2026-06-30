mod app;
mod config;
mod launcher;
mod ui;

use gtk::prelude::*;
use gtk::{glib, Application};

fn main() -> glib::ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let application = Application::builder().application_id(app::APP_ID).build();

    application.connect_activate(|app| {
        ui::main_window::build(app);
    });

    application.run()
}
