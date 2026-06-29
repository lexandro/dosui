mod app;
mod config;
mod launcher;
mod ui;

use gtk::prelude::*;
use gtk::{glib, Application};

fn main() -> glib::ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    integrate_desktop();

    let application = Application::builder().application_id(app::APP_ID).build();

    application.connect_activate(|app| {
        ui::main_window::build(app);
    });

    application.run()
}

/// Best-effort first-run desktop integration: when launched as a portable
/// AppImage with no menu entry yet, install a `.desktop` launcher + icon so
/// dosui appears in the application menu. Skipped for installed/dev runs (no
/// `$APPIMAGE`); failures are logged, never fatal.
fn integrate_desktop() {
    if std::env::var_os("APPIMAGE").is_none() {
        return;
    }
    let (Ok(home), Some(exec)) = (config::paths::data_home(), config::desktop::exec_path()) else {
        return;
    };
    match config::desktop::ensure_first_run(&home, &exec) {
        Ok(true) => log::info!("first run: installed desktop entry for dosui"),
        Ok(false) => {}
        Err(e) => log::warn!("desktop integration skipped: {e:#}"),
    }
}
