//! Title bar, classic menu bar, and quick-action toolbar. Every command is an
//! app action, so this module only references action names.

use gtk::prelude::*;
use gtk::{
    gio, Box as GtkBox, Button, HeaderBar, Orientation, PopoverMenuBar, SearchEntry, Separator,
};

/// Header bar plus the widgets the window wires up.
pub(crate) struct Header {
    pub bar: HeaderBar,
    pub new_profile: Button,
    pub settings: Button,
    pub search: SearchEntry,
}

pub(crate) fn build_header() -> Header {
    let bar = HeaderBar::new();
    let new_profile = Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("New profile")
        .build();
    bar.pack_start(&new_profile);
    let settings = Button::builder()
        .icon_name("emblem-system-symbolic")
        .tooltip_text("Settings & global defaults")
        .build();
    bar.pack_end(&settings);
    let search = SearchEntry::builder()
        .placeholder_text("Search profiles…")
        .build();
    bar.set_title_widget(Some(&search));
    Header {
        bar,
        new_profile,
        settings,
        search,
    }
}

/// Classic D-Fend-style menu bar (File / Profile / Tools / Settings / Help).
pub(crate) fn build_menubar() -> PopoverMenuBar {
    let file = gio::Menu::new();
    file.append(Some("New profile"), Some("app.new"));
    file.append(Some("Add DOS console"), Some("app.add-console"));
    file.append(Some("Import dosbox.conf…"), Some("app.import"));
    file.append(Some("Import zipped game…"), Some("app.import-zip"));
    file.append(Some("Quit"), Some("app.quit"));

    let tools = gio::Menu::new();
    tools.append(Some("Bulk edit…"), Some("app.bulk-edit"));

    let settings = gio::Menu::new();
    settings.append(Some("Preferences"), Some("app.settings"));

    let help = gio::Menu::new();
    help.append(Some("About dosui"), Some("app.about"));

    let menu = gio::Menu::new();
    menu.append_submenu(Some("File"), &file);
    menu.append_submenu(Some("View"), &build_view_menu());
    menu.append_submenu(Some("Profile"), &build_profile_menu());
    menu.append_submenu(Some("Tools"), &tools);
    menu.append_submenu(Some("Settings"), &settings);
    menu.append_submenu(Some("Help"), &help);

    PopoverMenuBar::from_model(Some(&menu))
}

/// The View menu: switch the games list between the details and icons modes.
fn build_view_menu() -> gio::Menu {
    let menu = gio::Menu::new();
    for (label, mode) in [("Details", "details"), ("Icons", "icons")] {
        let item = gio::MenuItem::new(Some(label), None);
        item.set_action_and_target_value(Some("app.view-mode"), Some(&mode.to_variant()));
        menu.append_item(&item);
    }
    menu
}

/// The profile command menu, reused by the menu bar and the grid context menu.
pub(crate) fn build_profile_menu() -> gio::Menu {
    let menu = gio::Menu::new();
    menu.append(Some("Run"), Some("app.play"));
    menu.append(Some("Edit"), Some("app.edit"));
    menu.append(Some("Duplicate"), Some("app.duplicate"));
    menu.append(Some("Toggle favorite"), Some("app.favorite"));
    menu.append(Some("Delete"), Some("app.delete"));
    menu.append(Some("Open folder"), Some("app.open-folder"));
    menu
}

/// D-Fend-style quick-action toolbar (all commands are app actions).
pub(crate) fn build_toolbar() -> GtkBox {
    let bar = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(2)
        .margin_top(4)
        .margin_bottom(4)
        .margin_start(4)
        .margin_end(4)
        .build();
    bar.append(&tool_button("list-add-symbolic", "New profile", "app.new"));
    bar.append(&tool_button(
        "utilities-terminal-symbolic",
        "Add DOS console",
        "app.add-console",
    ));
    bar.append(&Separator::new(Orientation::Vertical));
    bar.append(&tool_button(
        "media-playback-start-symbolic",
        "Run",
        "app.play",
    ));
    bar.append(&tool_button("document-edit-symbolic", "Edit", "app.edit"));
    bar.append(&tool_button(
        "edit-copy-symbolic",
        "Duplicate",
        "app.duplicate",
    ));
    bar.append(&tool_button("user-trash-symbolic", "Delete", "app.delete"));
    bar.append(&tool_button(
        "folder-open-symbolic",
        "Open folder",
        "app.open-folder",
    ));
    bar.append(&Separator::new(Orientation::Vertical));
    bar.append(&tool_button(
        "emblem-system-symbolic",
        "Settings",
        "app.settings",
    ));
    bar
}

/// A flat icon toolbar button bound to an app action.
fn tool_button(icon: &str, tooltip: &str, action: &str) -> Button {
    let button = Button::builder()
        .icon_name(icon)
        .tooltip_text(tooltip)
        .css_classes(["flat"])
        .build();
    button.set_action_name(Some(action));
    button
}
