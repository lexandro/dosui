//! The Settings "Application" tab: the DOSBox binary path plus (on the AppImage)
//! the desktop-shortcut add/remove actions, each in a framed section.

use gtk::gio;
use gtk::prelude::*;
use gtk::{
    Box as GtkBox, Button, Entry, FileDialog, Frame, Label, Orientation, SizeGroup, SizeGroupMode,
    Widget, Window,
};

use crate::config::settings::AppSettings;
use crate::ui::widgets;

/// Build the Application tab, returning the page and its DOSBox-path entry.
pub(super) fn build(settings: &AppSettings, window: &Window) -> (GtkBox, Entry) {
    let page = widgets::page();

    // DOSBox section.
    let current = settings
        .dosbox_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let (row, dosbox_path, browse) = widgets::file_row("DOSBox binary", &current);
    page.append(&section(
        "DOSBox",
        "Leave empty to auto-detect dosbox-staging / dosbox on your PATH.",
        &row,
    ));
    wire_browse(window, &dosbox_path, &browse);

    // Desktop-shortcut section (AppImage only).
    if crate::integration::is_appimage() {
        page.append(&shortcuts_section(window));
    }

    (page, dosbox_path)
}

/// The "Desktop shortcuts" section: stacked Add/Remove buttons wired to the
/// desktop integration. AppImage-only.
fn shortcuts_section(window: &Window) -> GtkBox {
    let add = Button::with_label("Add to applications menu & desktop");
    add.add_css_class("suggested-action");
    let remove = Button::with_label("Remove from menu & desktop");
    remove.add_css_class("destructive-action");
    remove.set_sensitive(crate::integration::is_installed());

    // Stack the buttons, same width.
    let sizes = SizeGroup::new(SizeGroupMode::Horizontal);
    sizes.add_widget(&add);
    sizes.add_widget(&remove);
    let buttons = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .halign(gtk::Align::Start)
        .build();
    buttons.append(&add);
    buttons.append(&remove);

    {
        let win = window.clone();
        let remove = remove.clone();
        add.connect_clicked(move |_| {
            crate::ui::desktop_integration::integrate_now(&win);
            remove.set_sensitive(true);
        });
    }
    {
        let win = window.clone();
        let remove2 = remove.clone();
        remove.connect_clicked(move |_| {
            crate::ui::desktop_integration::uninstall_now(&win);
            remove2.set_sensitive(false);
        });
    }

    section(
        "Desktop shortcuts",
        "Add dosui to your applications menu and desktop so you can launch \
         it without locating the AppImage file.",
        &buttons,
    )
}

/// A settings group: a heading, a dim description, and `content` in a padded
/// frame. Keeps the tab visually structured instead of a loose stack.
fn section(title: &str, description: &str, content: &impl IsA<Widget>) -> GtkBox {
    let group = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .margin_top(6)
        .build();
    group.append(
        &Label::builder()
            .label(title)
            .halign(gtk::Align::Start)
            .css_classes(["heading"])
            .build(),
    );
    group.append(
        &Label::builder()
            .label(description)
            .halign(gtk::Align::Start)
            .wrap(true)
            .css_classes(["dim-label"])
            .build(),
    );
    content.set_margin_top(10);
    content.set_margin_bottom(10);
    content.set_margin_start(12);
    content.set_margin_end(12);
    let frame = Frame::new(None);
    frame.set_child(Some(content));
    group.append(&frame);
    group
}

/// Wire the DOSBox-path "Browse…" button to a file chooser.
fn wire_browse(window: &Window, entry: &Entry, browse: &Button) {
    let window = window.downgrade();
    let entry = entry.clone();
    browse.connect_clicked(move |_| {
        let dialog = FileDialog::builder().title("Select DOSBox binary").build();
        let entry = entry.clone();
        dialog.open(
            window.upgrade().as_ref(),
            gio::Cancellable::NONE,
            move |res| {
                if let Ok(Some(path)) = res.map(|f| f.path()) {
                    entry.set_text(&path.display().to_string());
                }
            },
        );
    });
}
