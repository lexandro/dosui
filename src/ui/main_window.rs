//! The main application window: profile list (left) + detail/Play (right).
//!
//! Two-pane shell for now: pick a profile, Play, or Edit/create profiles. The
//! category sidebar and cover grid arrive in later milestones (plan §2.5).

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{
    gio, AlertDialog, Application, ApplicationWindow, Box as GtkBox, Button, HeaderBar, Label,
    ListBox, Orientation, Paned, ScrolledWindow, SearchEntry, SelectionMode,
};

use crate::app::APP_NAME;
use crate::config::profile::{self, Profile};
use crate::launcher;
use crate::ui::profile_editor;

/// Loaded profile together with its on-disk directory (needed to launch).
type Entry = (PathBuf, Profile);

/// Shared, reloadable profile list backing the ListBox rows by index.
type Profiles = Rc<RefCell<Vec<Entry>>>;

/// Widgets in the detail pane whose text changes with the selection.
#[derive(Clone)]
struct Detail {
    container: GtkBox,
    title: Label,
    meta: Label,
    notes: Label,
    play: Button,
    edit: Button,
}

pub fn build(app: &Application) {
    let profiles: Profiles = Rc::new(RefCell::new(load_profiles()));

    let window = ApplicationWindow::builder()
        .application(app)
        .title(APP_NAME)
        .default_width(900)
        .default_height(580)
        .build();
    let header = build_header();
    window.set_titlebar(Some(&header.bar));

    let list = ListBox::new();
    list.set_selection_mode(SelectionMode::Single);
    populate(&list, &profiles.borrow());
    let list_scroller = ScrolledWindow::builder()
        .child(&list)
        .width_request(260)
        .build();

    let detail = build_detail();

    // Reloads the list from disk and reselects the first row (used after edits).
    let reload = make_reload(&list, &profiles, &detail);

    // Selection -> refresh the detail pane.
    {
        let profiles = profiles.clone();
        let detail = detail.clone();
        list.connect_row_selected(move |_, row| match row {
            Some(row) => {
                if let Some((_, profile)) = profiles.borrow().get(row.index() as usize) {
                    show_profile(&detail, profile);
                }
            }
            None => clear_detail(&detail),
        });
    }

    // Every button maps to a GAction. This keeps one source of truth for each
    // command, gives keyboard accelerators, and — crucially — makes the app
    // driveable from outside (e.g. `gapplication action io.github.dosui play`)
    // so behaviour can be tested without hunting for on-screen pixels.
    install_actions(app, &window, &list, &profiles, &reload);
    detail.play.set_action_name(Some("app.play"));
    detail.edit.set_action_name(Some("app.edit"));
    header.new_profile.set_action_name(Some("app.new"));
    header.settings.set_action_name(Some("app.settings"));

    // Double-click / Enter on a row -> Play (D-Fend behaviour).
    list.connect_row_activated(|list, _| {
        let _ = WidgetExt::activate_action(list, "app.play", None);
    });

    // Pre-select the first profile so the detail pane is populated on start.
    select_first(&list, &detail);

    let root = Paned::builder()
        .orientation(Orientation::Horizontal)
        .position(260)
        .start_child(&list_scroller)
        .end_child(&detail.container)
        .build();
    window.set_child(Some(&root));

    window.present();
}

/// Register the `play` / `edit` / `new` app actions and their accelerators.
/// Buttons and the row-activated gesture all route through these, so there is a
/// single implementation per command and it can be triggered externally.
fn install_actions(
    app: &Application,
    window: &ApplicationWindow,
    list: &ListBox,
    profiles: &Profiles,
    reload: &Rc<dyn Fn()>,
) {
    let play = gio::SimpleAction::new("play", None);
    {
        let profiles = profiles.clone();
        let window = window.downgrade();
        let list = list.clone();
        play.connect_activate(move |_, _| {
            if let Some(row) = list.selected_row() {
                launch_entry(&profiles.borrow(), window.upgrade(), row.index() as usize);
            }
        });
    }
    app.add_action(&play);

    let edit = gio::SimpleAction::new("edit", None);
    {
        let profiles = profiles.clone();
        let window = window.downgrade();
        let list = list.clone();
        let reload = reload.clone();
        edit.connect_activate(move |_, _| {
            let Some(row) = list.selected_row() else {
                return;
            };
            let (dir, prof) = match profiles.borrow().get(row.index() as usize) {
                Some((dir, prof)) => (dir.clone(), prof.clone()),
                None => return,
            };
            if let Some(window) = window.upgrade() {
                profile_editor::open_for_edit(&window, dir, prof, reload.clone());
            }
        });
    }
    app.add_action(&edit);

    let new = gio::SimpleAction::new("new", None);
    {
        let window = window.downgrade();
        let reload = reload.clone();
        new.connect_activate(move |_, _| {
            if let Some(window) = window.upgrade() {
                crate::ui::wizard::open(&window, reload.clone());
            }
        });
    }
    app.add_action(&new);

    let settings = gio::SimpleAction::new("settings", None);
    {
        let window = window.downgrade();
        settings.connect_activate(move |_, _| {
            if let Some(window) = window.upgrade() {
                crate::ui::settings_dialog::open(&window, Rc::new(|| {}));
            }
        });
    }
    app.add_action(&settings);

    app.set_accels_for_action("app.play", &["<Ctrl>p"]);
    app.set_accels_for_action("app.edit", &["<Ctrl>e"]);
    app.set_accels_for_action("app.new", &["<Ctrl>n"]);
    app.set_accels_for_action("app.settings", &["<Ctrl>comma"]);
}

/// Build a callback that rescans profiles and rebuilds the list.
fn make_reload(list: &ListBox, profiles: &Profiles, detail: &Detail) -> Rc<dyn Fn()> {
    let list = list.clone();
    let profiles = profiles.clone();
    let detail = detail.clone();
    Rc::new(move || {
        *profiles.borrow_mut() = load_profiles();
        clear_list(&list);
        populate(&list, &profiles.borrow());
        select_first(&list, &detail);
    })
}

/// Append a row per profile.
fn populate(list: &ListBox, profiles: &[Entry]) {
    for (_, profile) in profiles {
        list.append(&profile_row(profile));
    }
}

/// Remove all rows from the list.
fn clear_list(list: &ListBox) {
    while let Some(row) = list.row_at_index(0) {
        list.remove(&row);
    }
}

/// Select the first row (populating the detail pane), or clear it if empty.
fn select_first(list: &ListBox, detail: &Detail) {
    match list.row_at_index(0) {
        Some(first) => {
            list.select_row(Some(&first));
            first.grab_focus();
        }
        None => clear_detail(detail),
    }
}

/// Launch the profile at `index`, reporting failures in a dialog.
/// Shared by the Play button and row activation (double-click / Enter).
fn launch_entry(profiles: &[Entry], window: Option<ApplicationWindow>, index: usize) {
    let Some((dir, profile)) = profiles.get(index) else {
        return;
    };
    if let Err(e) = launcher::launch(dir, profile) {
        log::error!("launch failed: {e:#}");
        if let Some(window) = window {
            AlertDialog::builder()
                .message(format!("Could not launch {}", profile.title))
                .detail(format!("{e:#}"))
                .build()
                .show(Some(&window));
        }
    }
}

/// Load every profile from the data dir; an error yields an empty list (logged).
fn load_profiles() -> Vec<Entry> {
    match crate::config::paths::profiles_dir().and_then(|dir| profile::scan(&dir)) {
        Ok(entries) => entries,
        Err(e) => {
            log::error!("loading profiles: {e:#}");
            Vec::new()
        }
    }
}

/// Header bar plus the buttons the window wires up.
struct Header {
    bar: HeaderBar,
    new_profile: Button,
    settings: Button,
}

fn build_header() -> Header {
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
    bar.set_title_widget(Some(
        &SearchEntry::builder()
            .placeholder_text("Search profiles…")
            .build(),
    ));
    Header {
        bar,
        new_profile,
        settings,
    }
}

/// One list row: the profile title.
fn profile_row(profile: &Profile) -> Label {
    Label::builder()
        .label(&profile.title)
        .halign(gtk::Align::Start)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(10)
        .margin_end(10)
        .build()
}

/// Build the detail pane (empty state until a profile is selected).
fn build_detail() -> Detail {
    let title = Label::builder()
        .halign(gtk::Align::Start)
        .css_classes(["title-2"])
        .build();
    let meta = Label::builder()
        .halign(gtk::Align::Start)
        .css_classes(["dim-label"])
        .build();
    let notes = Label::builder()
        .halign(gtk::Align::Start)
        .wrap(true)
        .build();
    let play = Button::builder()
        .label("Play")
        .css_classes(["suggested-action"])
        .sensitive(false)
        .build();
    let edit = Button::builder().label("Edit").sensitive(false).build();
    let actions = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .halign(gtk::Align::Start)
        .build();
    actions.append(&play);
    actions.append(&edit);

    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .build();
    container.append(&title);
    container.append(&meta);
    container.append(&notes);
    container.append(&actions);

    let detail = Detail {
        container,
        title,
        meta,
        notes,
        play,
        edit,
    };
    clear_detail(&detail);
    detail
}

/// Fill the detail pane from a profile and enable Play/Edit.
fn show_profile(detail: &Detail, profile: &Profile) {
    detail.title.set_text(&profile.title);
    detail.meta.set_text(&meta_line(profile));
    detail
        .notes
        .set_text(profile.notes.as_deref().unwrap_or(""));
    detail.play.set_sensitive(true);
    detail.edit.set_sensitive(true);
}

/// Reset the detail pane to the empty state.
fn clear_detail(detail: &Detail) {
    detail.title.set_text("Select a profile");
    detail.meta.set_text("");
    detail.notes.set_text("");
    detail.play.set_sensitive(false);
    detail.edit.set_sensitive(false);
}

/// "Genre · Year · Developer" from whatever fields are present.
fn meta_line(profile: &Profile) -> String {
    let mut parts = Vec::new();
    if let Some(genre) = &profile.genre {
        parts.push(genre.clone());
    }
    if let Some(year) = profile.year {
        parts.push(year.to_string());
    }
    if let Some(dev) = &profile.developer {
        parts.push(dev.clone());
    }
    parts.join(" · ")
}
