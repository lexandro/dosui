//! The main application window: cover grid (left) + detail/Play (right).
//!
//! The grid is a `GridView` over a `ListStore` of profiles (boxed), mirroring the
//! `profiles` Vec order so the selected position indexes straight into it. Pick a
//! profile, Play, or Edit/create profiles.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk::glib::BoxedAnyObject;
use gtk::prelude::*;
use gtk::{
    gio, AlertDialog, Application, ApplicationWindow, Box as GtkBox, Button, ContentFit,
    FileLauncher, GridView, HeaderBar, Label, ListItem, Orientation, Paned, Picture,
    ScrolledWindow, SearchEntry, SignalListItemFactory, SingleSelection,
};

use crate::app::APP_NAME;
use crate::config::profile::{self, Profile};
use crate::launcher;
use crate::ui::profile_editor;

/// Loaded profile together with its on-disk directory (needed to launch).
type Entry = (PathBuf, Profile);

/// Shared, reloadable profile list. The grid's ListStore mirrors this order, so
/// the selected position is an index into it.
type Profiles = Rc<RefCell<Vec<Entry>>>;

/// Widgets in the detail pane whose contents change with the selection.
#[derive(Clone)]
struct Detail {
    container: GtkBox,
    cover: Picture,
    title: Label,
    meta: Label,
    notes: Label,
    last_played: Label,
    play: Button,
    edit: Button,
}

pub fn build(app: &Application) {
    let profiles: Profiles = Rc::new(RefCell::new(load_profiles()));

    let window = ApplicationWindow::builder()
        .application(app)
        .title(APP_NAME)
        .default_width(940)
        .default_height(600)
        .build();
    let header = build_header();
    window.set_titlebar(Some(&header.bar));

    let store = gio::ListStore::new::<BoxedAnyObject>();
    fill_store(&store, &profiles.borrow());
    let selection = SingleSelection::new(Some(store.clone()));
    let grid = GridView::builder()
        .model(&selection)
        .factory(&build_factory())
        .max_columns(8)
        .min_columns(2)
        .build();
    let grid_scroller = ScrolledWindow::builder()
        .child(&grid)
        .width_request(360)
        .build();

    let detail = build_detail();

    // Selection -> refresh the detail pane.
    {
        let profiles = profiles.clone();
        let detail = detail.clone();
        selection.connect_selected_notify(move |sel| {
            refresh_detail(sel, &profiles, &detail);
        });
    }

    // Rebuilds the store from disk and reselects the first item (used after edits).
    let reload = make_reload(&store, &profiles, &selection, &detail);

    // Buttons route through GActions: single source of truth, accelerators, and
    // external testability (`gapplication action io.github.dosui play`).
    install_actions(app, &window, &selection, &profiles, &reload);
    detail.play.set_action_name(Some("app.play"));
    detail.edit.set_action_name(Some("app.edit"));
    header.new_profile.set_action_name(Some("app.new"));
    header.settings.set_action_name(Some("app.settings"));

    // Double-click / Enter on a cover -> Play.
    grid.connect_activate(|grid, _| {
        let _ = WidgetExt::activate_action(grid, "app.play", None);
    });

    select_first(&selection, &profiles, &detail);

    let root = Paned::builder()
        .orientation(Orientation::Horizontal)
        .position(360)
        .start_child(&grid_scroller)
        .end_child(&detail.container)
        .build();
    window.set_child(Some(&root));

    window.present();
}

/// Build the factory that renders each grid cell (cover thumbnail + title).
fn build_factory() -> SignalListItemFactory {
    let factory = SignalListItemFactory::new();

    factory.connect_setup(|_, item| {
        let item = item.downcast_ref::<ListItem>().expect("ListItem");
        let cover = Picture::builder()
            .content_fit(ContentFit::Contain)
            .width_request(150)
            .height_request(110)
            .build();
        let title = Label::builder()
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .max_width_chars(20)
            .build();
        let cell = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .margin_top(6)
            .margin_bottom(6)
            .margin_start(6)
            .margin_end(6)
            .build();
        cell.append(&cover);
        cell.append(&title);
        item.set_child(Some(&cell));
    });

    factory.connect_bind(|_, item| {
        let item = item.downcast_ref::<ListItem>().expect("ListItem");
        let Some(cell) = item.child().and_downcast::<GtkBox>() else {
            return;
        };
        let Some(cover) = cell.first_child().and_downcast::<Picture>() else {
            return;
        };
        let Some(title) = cover.next_sibling().and_downcast::<Label>() else {
            return;
        };
        let Some(obj) = item.item().and_downcast::<BoxedAnyObject>() else {
            return;
        };
        let entry = obj.borrow::<Entry>();
        let (dir, profile) = &*entry;
        title.set_text(&profile.title);
        match cover_path(dir, profile) {
            Some(p) if p.exists() => cover.set_filename(p.to_str()),
            _ => cover.set_filename(None::<&str>),
        }
    });

    factory
}

/// Register the `play` / `edit` / `new` / `settings` actions and accelerators.
fn install_actions(
    app: &Application,
    window: &ApplicationWindow,
    selection: &SingleSelection,
    profiles: &Profiles,
    reload: &Rc<dyn Fn()>,
) {
    let play = gio::SimpleAction::new("play", None);
    {
        let profiles = profiles.clone();
        let selection = selection.clone();
        let window = window.downgrade();
        play.connect_activate(move |_, _| {
            if let Some(i) = selected_index(&selection) {
                launch_entry(&profiles.borrow(), window.upgrade(), i);
            }
        });
    }
    app.add_action(&play);

    let edit = gio::SimpleAction::new("edit", None);
    {
        let profiles = profiles.clone();
        let selection = selection.clone();
        let window = window.downgrade();
        let reload = reload.clone();
        edit.connect_activate(move |_, _| {
            let Some(i) = selected_index(&selection) else {
                return;
            };
            let (dir, prof) = match profiles.borrow().get(i) {
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

    let delete = gio::SimpleAction::new("delete", None);
    {
        let profiles = profiles.clone();
        let selection = selection.clone();
        let window = window.downgrade();
        let reload = reload.clone();
        delete.connect_activate(move |_, _| {
            let Some(i) = selected_index(&selection) else {
                return;
            };
            let (dir, title) = match profiles.borrow().get(i) {
                Some((dir, prof)) => (dir.clone(), prof.title.clone()),
                None => return,
            };
            let Some(window) = window.upgrade() else {
                return;
            };
            let reload = reload.clone();
            let dialog = AlertDialog::builder()
                .modal(true)
                .message(format!("Delete “{title}”?"))
                .detail("This removes the profile. The game files are not touched.")
                .buttons(["Cancel", "Delete"])
                .cancel_button(0)
                .default_button(0)
                .build();
            dialog.choose(Some(&window), gio::Cancellable::NONE, move |res| {
                if res == Ok(1) {
                    if let Err(e) = std::fs::remove_dir_all(&dir) {
                        log::error!("deleting profile failed: {e:#}");
                    }
                    reload();
                }
            });
        });
    }
    app.add_action(&delete);

    let duplicate = gio::SimpleAction::new("duplicate", None);
    {
        let profiles = profiles.clone();
        let selection = selection.clone();
        let reload = reload.clone();
        duplicate.connect_activate(move |_, _| {
            let Some(i) = selected_index(&selection) else {
                return;
            };
            let (dir, prof) = match profiles.borrow().get(i) {
                Some((dir, prof)) => (dir.clone(), prof.clone()),
                None => return,
            };
            if let Err(e) = profile::duplicate(&dir, &prof) {
                log::error!("duplicating profile failed: {e:#}");
            }
            reload();
        });
    }
    app.add_action(&duplicate);

    let open_folder = gio::SimpleAction::new("open-folder", None);
    {
        let profiles = profiles.clone();
        let selection = selection.clone();
        let window = window.downgrade();
        open_folder.connect_activate(move |_, _| {
            let Some(i) = selected_index(&selection) else {
                return;
            };
            let dir = match profiles.borrow().get(i) {
                Some((dir, _)) => dir.clone(),
                None => return,
            };
            let launcher = FileLauncher::new(Some(&gio::File::for_path(&dir)));
            launcher.launch(window.upgrade().as_ref(), gio::Cancellable::NONE, |res| {
                if let Err(e) = res {
                    log::warn!("opening folder failed: {e}");
                }
            });
        });
    }
    app.add_action(&open_folder);

    let about = gio::SimpleAction::new("about", None);
    {
        let window = window.downgrade();
        about.connect_activate(move |_, _| {
            if let Some(window) = window.upgrade() {
                AlertDialog::builder()
                    .modal(true)
                    .message(APP_NAME)
                    .detail("Lightweight native Linux frontend for DOSBox.\nRust + GTK4.")
                    .build()
                    .show(Some(&window));
            }
        });
    }
    app.add_action(&about);

    let quit = gio::SimpleAction::new("quit", None);
    {
        let app = app.clone();
        quit.connect_activate(move |_, _| app.quit());
    }
    app.add_action(&quit);

    app.set_accels_for_action("app.play", &["<Ctrl>p"]);
    app.set_accels_for_action("app.edit", &["<Ctrl>e"]);
    app.set_accels_for_action("app.new", &["<Ctrl>n"]);
    app.set_accels_for_action("app.duplicate", &["<Ctrl>d"]);
    app.set_accels_for_action("app.delete", &["Delete"]);
    app.set_accels_for_action("app.settings", &["<Ctrl>comma"]);
    app.set_accels_for_action("app.quit", &["<Ctrl>q"]);
}

/// Build a callback that rescans profiles and rebuilds the grid.
fn make_reload(
    store: &gio::ListStore,
    profiles: &Profiles,
    selection: &SingleSelection,
    detail: &Detail,
) -> Rc<dyn Fn()> {
    let store = store.clone();
    let profiles = profiles.clone();
    let selection = selection.clone();
    let detail = detail.clone();
    Rc::new(move || {
        *profiles.borrow_mut() = load_profiles();
        fill_store(&store, &profiles.borrow());
        select_first(&selection, &profiles, &detail);
    })
}

/// Replace the store's contents with one boxed entry per profile.
fn fill_store(store: &gio::ListStore, profiles: &[Entry]) {
    store.remove_all();
    for entry in profiles {
        store.append(&BoxedAnyObject::new(entry.clone()));
    }
}

/// Index of the current selection, or `None` when nothing is selected.
fn selected_index(selection: &SingleSelection) -> Option<usize> {
    let pos = selection.selected();
    if pos == gtk::INVALID_LIST_POSITION {
        None
    } else {
        Some(pos as usize)
    }
}

/// Select the first item (populating the detail pane), or clear it if empty.
fn select_first(selection: &SingleSelection, profiles: &Profiles, detail: &Detail) {
    if selection.n_items() > 0 {
        selection.set_selected(0);
    }
    refresh_detail(selection, profiles, detail);
}

/// Update the detail pane to reflect the current selection.
fn refresh_detail(selection: &SingleSelection, profiles: &Profiles, detail: &Detail) {
    match selected_index(selection).and_then(|i| profiles.borrow().get(i).cloned()) {
        Some((dir, profile)) => show_profile(detail, &dir, &profile),
        None => clear_detail(detail),
    }
}

/// Launch the profile at `index`, reporting failures in a dialog.
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

/// Build the detail pane (empty state until a profile is selected).
fn build_detail() -> Detail {
    let cover = Picture::builder()
        .content_fit(ContentFit::Contain)
        .height_request(180)
        .hexpand(true)
        .build();
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
    let last_played = Label::builder()
        .halign(gtk::Align::Start)
        .css_classes(["dim-label"])
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
    container.append(&cover);
    container.append(&title);
    container.append(&meta);
    container.append(&notes);
    container.append(&last_played);
    container.append(&actions);

    let detail = Detail {
        container,
        cover,
        title,
        meta,
        notes,
        last_played,
        play,
        edit,
    };
    clear_detail(&detail);
    detail
}

/// Fill the detail pane from a profile and enable Play/Edit.
fn show_profile(detail: &Detail, dir: &Path, profile: &Profile) {
    detail.title.set_text(&profile.title);
    detail.meta.set_text(&meta_line(profile));
    detail
        .notes
        .set_text(profile.notes.as_deref().unwrap_or(""));
    detail.last_played.set_text(&last_played_line(profile));
    set_cover(detail, cover_path(dir, profile).as_deref());
    detail.play.set_sensitive(true);
    detail.edit.set_sensitive(true);
}

/// Reset the detail pane to the empty state.
fn clear_detail(detail: &Detail) {
    detail.title.set_text("Select a profile");
    detail.meta.set_text("");
    detail.notes.set_text("");
    detail.last_played.set_text("");
    set_cover(detail, None);
    detail.play.set_sensitive(false);
    detail.edit.set_sensitive(false);
}

/// Resolve a profile's cover to an absolute path (relative covers join `dir`).
fn cover_path(dir: &Path, profile: &Profile) -> Option<PathBuf> {
    profile.cover.as_ref().map(|c| {
        if c.is_absolute() {
            c.clone()
        } else {
            dir.join(c)
        }
    })
}

/// Show the cover image (hidden when absent or missing on disk).
fn set_cover(detail: &Detail, path: Option<&Path>) {
    match path {
        Some(p) if p.exists() => {
            detail.cover.set_filename(p.to_str());
            detail.cover.set_visible(true);
        }
        _ => {
            detail.cover.set_filename(None::<&str>);
            detail.cover.set_visible(false);
        }
    }
}

/// "Last played: …" line, or "Never played".
fn last_played_line(profile: &Profile) -> String {
    match profile.last_played {
        Some(then) => format!(
            "Last played: {}",
            profile::humanize_since(profile::now_unix(), then)
        ),
        None => "Never played".to_string(),
    }
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
