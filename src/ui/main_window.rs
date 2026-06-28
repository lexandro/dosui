//! The main application window: cover grid (left) + detail/Play (right).
//!
//! The grid is a `GridView` whose model is a `SingleSelection` over a
//! `FilterListModel` over a `ListStore` of boxed profile entries. Selection-aware
//! commands read the selected [`Entry`] from the selected item (not by index), so
//! filtering (search/categories) doesn't disturb them.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk::glib::BoxedAnyObject;
use gtk::prelude::*;
use gtk::{
    gio, AlertDialog, Application, ApplicationWindow, Box as GtkBox, Button, ContentFit,
    CustomFilter, FileDialog, FileLauncher, FilterChange, FilterListModel, GridView, HeaderBar,
    Label, ListItem, Orientation, Paned, Picture, PopoverMenu, PopoverMenuBar, ScrolledWindow,
    SearchEntry, Separator, SignalListItemFactory, SingleSelection,
};

use crate::app::APP_NAME;
use crate::config::conf_import;
use crate::config::profile::{self, Profile};
use crate::launcher;
use crate::ui::category_sidebar::{Category, CategorySidebar};
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

    // Find-as-you-type + category filtering: a shared query string and active
    // category drive a CustomFilter over the store.
    let query: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
    let active_category = Rc::new(RefCell::new(Category::All));
    let filter = build_filter(&query, &active_category);
    let filter_model = FilterListModel::new(Some(store.clone()), Some(filter.clone()));
    let selection = SingleSelection::new(Some(filter_model));

    // Category sidebar drives the same filter.
    let sidebar = Rc::new({
        let filter = filter.clone();
        let on_change: Rc<dyn Fn()> = Rc::new(move || filter.changed(FilterChange::Different));
        CategorySidebar::new(active_category.clone(), on_change)
    });
    sidebar.rebuild(&profiles.borrow());

    let grid = GridView::builder()
        .model(&selection)
        .max_columns(8)
        .min_columns(2)
        .build();
    // Right-click context menu (parented to the grid, anchored at the pointer).
    let context_menu = PopoverMenu::from_model(Some(&build_profile_menu()));
    context_menu.set_parent(&grid);
    context_menu.set_has_arrow(false);
    grid.set_factory(Some(&build_factory(&selection, &grid, &context_menu)));
    let grid_scroller = ScrolledWindow::builder()
        .child(&grid)
        .width_request(360)
        .build();

    let detail = build_detail();

    // Selection -> refresh the detail pane.
    {
        let detail = detail.clone();
        selection.connect_selected_notify(move |sel| {
            refresh_detail(sel, &detail);
        });
    }

    // Search entry re-runs the filter.
    {
        let query = query.clone();
        let filter = filter.clone();
        header.search.connect_search_changed(move |entry| {
            *query.borrow_mut() = entry.text().to_string();
            filter.changed(FilterChange::Different);
        });
    }

    // Rebuilds the store from disk and reselects the first item (used after edits).
    let reload = make_reload(&store, &profiles, &selection, &detail, &sidebar);

    // Buttons route through GActions: single source of truth, accelerators, and
    // external testability (`gapplication action io.github.dosui play`).
    install_actions(app, &window, &selection, &reload);
    detail.play.set_action_name(Some("app.play"));
    detail.edit.set_action_name(Some("app.edit"));
    header.new_profile.set_action_name(Some("app.new"));
    header.settings.set_action_name(Some("app.settings"));

    // Double-click / Enter on a cover -> Play.
    grid.connect_activate(|grid, _| {
        let _ = WidgetExt::activate_action(grid, "app.play", None);
    });

    select_first(&selection, &detail);

    let content = Paned::builder()
        .orientation(Orientation::Horizontal)
        .position(360)
        .start_child(&grid_scroller)
        .end_child(&detail.container)
        .build();
    let root = Paned::builder()
        .orientation(Orientation::Horizontal)
        .position(200)
        .start_child(&sidebar.scroller)
        .end_child(&content)
        .vexpand(true)
        .build();
    let body = GtkBox::builder().orientation(Orientation::Vertical).build();
    body.append(&build_menubar());
    body.append(&build_toolbar());
    body.append(&root);
    window.set_child(Some(&body));

    window.present();
}

/// Classic D-Fend-style menu bar (File / Profile / Settings / Help), bound to
/// the app actions.
fn build_menubar() -> PopoverMenuBar {
    let file = gio::Menu::new();
    file.append(Some("New profile"), Some("app.new"));
    file.append(Some("Import dosbox.conf…"), Some("app.import"));
    file.append(Some("Quit"), Some("app.quit"));

    let settings = gio::Menu::new();
    settings.append(Some("Preferences"), Some("app.settings"));

    let help = gio::Menu::new();
    help.append(Some("About dosui"), Some("app.about"));

    let menu = gio::Menu::new();
    menu.append_submenu(Some("File"), &file);
    menu.append_submenu(Some("Profile"), &build_profile_menu());
    menu.append_submenu(Some("Settings"), &settings);
    menu.append_submenu(Some("Help"), &help);

    PopoverMenuBar::from_model(Some(&menu))
}

/// The profile command menu, reused by the menu bar and the grid context menu.
fn build_profile_menu() -> gio::Menu {
    let menu = gio::Menu::new();
    menu.append(Some("Run"), Some("app.play"));
    menu.append(Some("Edit"), Some("app.edit"));
    menu.append(Some("Duplicate"), Some("app.duplicate"));
    menu.append(Some("Toggle favorite"), Some("app.favorite"));
    menu.append(Some("Delete"), Some("app.delete"));
    menu.append(Some("Open folder"), Some("app.open-folder"));
    menu
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

/// D-Fend-style quick-action toolbar (all commands are app actions).
fn build_toolbar() -> GtkBox {
    let bar = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(2)
        .margin_top(4)
        .margin_bottom(4)
        .margin_start(4)
        .margin_end(4)
        .build();
    bar.append(&tool_button("list-add-symbolic", "New profile", "app.new"));
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

/// Build the factory that renders each grid cell (cover thumbnail + title) and
/// wires a secondary-click context menu.
fn build_factory(
    selection: &SingleSelection,
    grid: &GridView,
    menu: &PopoverMenu,
) -> SignalListItemFactory {
    let factory = SignalListItemFactory::new();
    let selection = selection.clone();
    let grid = grid.clone();
    let menu = menu.clone();

    factory.connect_setup(move |_, item| {
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

        // Secondary-click: select this cell and open the profile context menu.
        let gesture = gtk::GestureClick::new();
        gesture.set_button(gtk::gdk::BUTTON_SECONDARY);
        let selection = selection.clone();
        let grid = grid.clone();
        let menu = menu.clone();
        let item = item.clone();
        let cell_for_click = cell.clone();
        gesture.connect_pressed(move |gesture, _, x, y| {
            let pos = item.position();
            if pos != gtk::INVALID_LIST_POSITION {
                selection.set_selected(pos);
            }
            let point = gtk::graphene::Point::new(x as f32, y as f32);
            let (gx, gy) = cell_for_click
                .compute_point(&grid, &point)
                .map(|p| (p.x() as i32, p.y() as i32))
                .unwrap_or((x as i32, y as i32));
            menu.set_pointing_to(Some(&gtk::gdk::Rectangle::new(gx, gy, 1, 1)));
            menu.popup();
            gesture.set_state(gtk::EventSequenceState::Claimed);
        });
        cell.add_controller(gesture);
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
        title.set_text(&display_title(profile));
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
    reload: &Rc<dyn Fn()>,
) {
    let play = gio::SimpleAction::new("play", None);
    {
        let selection = selection.clone();
        let window = window.downgrade();
        play.connect_activate(move |_, _| {
            if let Some((dir, profile)) = selected_entry(&selection) {
                launch_profile(&dir, &profile, window.upgrade());
            }
        });
    }
    app.add_action(&play);

    let edit = gio::SimpleAction::new("edit", None);
    {
        let selection = selection.clone();
        let window = window.downgrade();
        let reload = reload.clone();
        edit.connect_activate(move |_, _| {
            let Some((dir, prof)) = selected_entry(&selection) else {
                return;
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
    let import = gio::SimpleAction::new("import", None);
    {
        let window = window.downgrade();
        let reload = reload.clone();
        import.connect_activate(move |_, _| {
            let Some(window) = window.upgrade() else {
                return;
            };
            let dialog = FileDialog::builder().title("Import dosbox.conf").build();
            let reload = reload.clone();
            let parent = window.clone();
            dialog.open(Some(&window), gio::Cancellable::NONE, move |res| {
                let Ok(file) = res else { return };
                let Some(path) = file.path() else { return };
                match std::fs::read_to_string(&path) {
                    Ok(text) => {
                        let title = path
                            .parent()
                            .and_then(|p| p.file_name())
                            .or_else(|| path.file_stem())
                            .and_then(|s| s.to_str())
                            .unwrap_or("Imported game")
                            .to_string();
                        if let Err(e) = save_imported(&text, &title) {
                            log::error!("import failed: {e:#}");
                            AlertDialog::builder()
                                .message("Could not import dosbox.conf")
                                .detail(format!("{e:#}"))
                                .build()
                                .show(Some(&parent));
                        }
                        reload();
                    }
                    Err(e) => log::error!("reading {} failed: {e:#}", path.display()),
                }
            });
        });
    }
    app.add_action(&import);

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
        let selection = selection.clone();
        let window = window.downgrade();
        let reload = reload.clone();
        delete.connect_activate(move |_, _| {
            let Some((dir, prof)) = selected_entry(&selection) else {
                return;
            };
            let title = prof.title.clone();
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
        let selection = selection.clone();
        let reload = reload.clone();
        duplicate.connect_activate(move |_, _| {
            let Some((dir, prof)) = selected_entry(&selection) else {
                return;
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
        let selection = selection.clone();
        let window = window.downgrade();
        open_folder.connect_activate(move |_, _| {
            let Some((dir, _)) = selected_entry(&selection) else {
                return;
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

    let favorite = gio::SimpleAction::new("favorite", None);
    {
        let selection = selection.clone();
        let reload = reload.clone();
        favorite.connect_activate(move |_, _| {
            let Some((dir, mut prof)) = selected_entry(&selection) else {
                return;
            };
            prof.favorite = !prof.favorite;
            if let Err(e) = prof.save(&dir) {
                log::error!("toggling favorite failed: {e:#}");
            }
            reload();
        });
    }
    app.add_action(&favorite);

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

    // Disable selection-dependent commands when nothing is selected.
    let dependent = vec![play, edit, duplicate, delete, open_folder, favorite];
    let update_enabled = {
        let selection = selection.clone();
        move || {
            let enabled = selection.selected_item().is_some();
            for action in &dependent {
                action.set_enabled(enabled);
            }
        }
    };
    update_enabled();
    selection.connect_selected_notify(move |_| update_enabled());

    app.set_accels_for_action("app.play", &["<Ctrl>p"]);
    app.set_accels_for_action("app.edit", &["<Ctrl>e"]);
    app.set_accels_for_action("app.new", &["<Ctrl>n"]);
    app.set_accels_for_action("app.import", &["<Ctrl>i"]);
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
    sidebar: &Rc<CategorySidebar>,
) -> Rc<dyn Fn()> {
    let store = store.clone();
    let profiles = profiles.clone();
    let selection = selection.clone();
    let detail = detail.clone();
    let sidebar = sidebar.clone();
    Rc::new(move || {
        *profiles.borrow_mut() = load_profiles();
        fill_store(&store, &profiles.borrow());
        sidebar.rebuild(&profiles.borrow()); // refresh categories + reset to All
        select_first(&selection, &detail);
    })
}

/// Replace the store's contents with one boxed entry per profile.
fn fill_store(store: &gio::ListStore, profiles: &[Entry]) {
    store.remove_all();
    for entry in profiles {
        store.append(&BoxedAnyObject::new(entry.clone()));
    }
}

/// Filter on the title substring (search) AND the active category.
fn build_filter(query: &Rc<RefCell<String>>, category: &Rc<RefCell<Category>>) -> CustomFilter {
    let query = query.clone();
    let category = category.clone();
    CustomFilter::new(move |obj| {
        let Some(obj) = obj.downcast_ref::<BoxedAnyObject>() else {
            return true;
        };
        let entry = obj.borrow::<Entry>();
        let profile = &entry.1;
        if !category.borrow().matches(profile) {
            return false;
        }
        let needle = query.borrow().to_lowercase();
        needle.is_empty() || profile.title.to_lowercase().contains(&needle)
    })
}

/// The selected profile entry (directory + profile), if any.
fn selected_entry(selection: &SingleSelection) -> Option<Entry> {
    selection
        .selected_item()
        .and_downcast::<BoxedAnyObject>()
        .map(|o| o.borrow::<Entry>().clone())
}

/// Select the first item (populating the detail pane), or clear it if empty.
fn select_first(selection: &SingleSelection, detail: &Detail) {
    if selection.n_items() > 0 {
        selection.set_selected(0);
    }
    refresh_detail(selection, detail);
}

/// Update the detail pane to reflect the current selection.
fn refresh_detail(selection: &SingleSelection, detail: &Detail) {
    match selected_entry(selection) {
        Some((dir, profile)) => show_profile(detail, &dir, &profile),
        None => clear_detail(detail),
    }
}

/// Launch a profile, reporting failures in a dialog.
fn launch_profile(dir: &Path, profile: &Profile, window: Option<ApplicationWindow>) {
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

/// Import a dosbox.conf into a fresh profile directory.
fn save_imported(text: &str, title: &str) -> anyhow::Result<()> {
    let mut profile = conf_import::import_profile(text, title);
    let (id, dir) = profile::new_profile_dir(&profile.title)?;
    profile.id = id;
    profile.save(&dir)
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

/// Header bar plus the widgets the window wires up.
struct Header {
    bar: HeaderBar,
    new_profile: Button,
    settings: Button,
    search: SearchEntry,
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
    detail.title.set_text(&display_title(profile));
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

/// Title with a leading star for favorites.
fn display_title(profile: &Profile) -> String {
    if profile.favorite {
        format!("★ {}", profile.title)
    } else {
        profile.title.clone()
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
