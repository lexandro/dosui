mod app;
mod config;
mod integration;
mod launcher;
mod ui;

use gtk::prelude::*;
use gtk::{glib, Application};

fn main() -> glib::ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    if let Some(code) = handle_cli() {
        return code;
    }

    let application = Application::builder().application_id(app::APP_ID).build();

    application.connect_activate(|app| {
        ui::main_window::build(app);
    });

    application.run()
}

/// Handle dosui's own CLI flags before starting the GUI. Returns `Some(code)`
/// when a flag was handled (and the process should exit), `None` to launch the
/// application as usual.
fn handle_cli() -> Option<glib::ExitCode> {
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                return Some(glib::ExitCode::SUCCESS);
            }
            "-V" | "--version" => {
                println!("dosui {}", env!("CARGO_PKG_VERSION"));
                return Some(glib::ExitCode::SUCCESS);
            }
            "--install" => return Some(cli_result("added", integration::install().map(|()| true))),
            "--uninstall" => return Some(cli_result("removed", integration::uninstall())),
            _ => {} // unknown args fall through to the GUI / GTK option parsing
        }
    }
    None
}

/// Print the result of an integration action and map it to an exit code.
/// `verb` describes what happened on success (`Ok(true)`); `Ok(false)` means
/// there was nothing to do, `Err` a failure.
fn cli_result(verb: &str, outcome: anyhow::Result<bool>) -> glib::ExitCode {
    match outcome {
        Ok(true) => {
            println!("dosui: shortcuts {verb} (applications menu and desktop).");
            glib::ExitCode::SUCCESS
        }
        Ok(false) => {
            println!("dosui: nothing to do — no shortcuts were installed.");
            glib::ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("dosui: failed: {e:#}");
            glib::ExitCode::FAILURE
        }
    }
}

fn print_help() {
    println!("dosui — lightweight native frontend for DOSBox");
    println!();
    println!("Usage:");
    println!("  dosui [OPTION]");
    println!();
    println!("With no option, dosui launches its graphical interface.");
    println!();
    println!("Options:");
    println!("      --install      add applications-menu and desktop shortcuts");
    println!("      --uninstall    remove those shortcuts");
    println!("  -V, --version      print version and exit");
    println!("  -h, --help         show this help and exit");
    println!();
    println!("Standard GTK options (e.g. --display) are also accepted.");
}
