//! The main application window: assembles the menu bar, toolbar, category
//! sidebar, the games view (a details list / icon grid in a `Stack`), and the
//! bottom preview tabs, then wires selection → preview and the reload callback.
//! The pieces live in sibling modules (headerbar / library / list_view / grid /
//! preview / actions); this file is just the orchestration.

use std::cell::RefCell;
use std::rc::Rc;

use gtk::glib::BoxedAnyObject;
use gtk::prelude::*;
use gtk::{
    gio, Application, ApplicationWindow, Box as GtkBox, FilterChange, FilterListModel, Orientation,
    Paned, SingleSelection, SortListModel,
};

use crate::app::APP_NAME;
use crate::ui::actions;
use crate::ui::category_sidebar::{Category, CategorySidebar};
use crate::ui::games_view;
use crate::ui::headerbar;
use crate::ui::library::{self, Profiles};
use crate::ui::preview::{self, Preview};

pub fn build(app: &Application) {
    // Window/taskbar icon, resolved from the icon theme by app id.
    gtk::Window::set_default_icon_name(crate::app::APP_ID);

    let profiles: Profiles = Rc::new(RefCell::new(library::load_profiles()));

    let window = ApplicationWindow::builder()
        .application(app)
        .title(APP_NAME)
        .default_width(940)
        .default_height(640)
        .build();
    let header = headerbar::build_header();
    window.set_titlebar(Some(&header.bar));

    let store = gio::ListStore::new::<BoxedAnyObject>();
    library::fill_store(&store, &profiles.borrow());

    // Search + category filtering both drive one CustomFilter over the store.
    let query: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
    let active_category = Rc::new(RefCell::new(Category::All));
    let filter = library::build_filter(&query, &active_category);
    let filter_model = FilterListModel::new(Some(store.clone()), Some(filter.clone()));
    // The selection's backing model is wired below, once the details view exists:
    // store → filter → sort (by the list's clicked column) → selection.
    let selection = SingleSelection::new(None::<gio::ListModel>);

    let sidebar = Rc::new({
        let filter = filter.clone();
        let on_change: Rc<dyn Fn()> = Rc::new(move || filter.changed(FilterChange::Different));
        CategorySidebar::new(active_category.clone(), on_change)
    });
    sidebar.rebuild(&profiles.borrow());

    let games = games_view::build(&selection);
    let sort_model = SortListModel::new(Some(filter_model), games.list_sorter());
    selection.set_model(Some(&sort_model));
    let preview = preview::build();
    {
        let preview = preview.clone();
        selection.connect_selected_notify(move |sel| refresh_preview(sel, &preview));
    }
    {
        let query = query.clone();
        let filter = filter.clone();
        header.search.connect_search_changed(move |entry| {
            *query.borrow_mut() = entry.text().to_string();
            filter.changed(FilterChange::Different);
        });
    }

    let reload = make_reload(&store, &profiles, &selection, &preview, &sidebar);

    actions::install_actions(app, &window, &selection, &profiles, &reload);
    games_view::install_view_mode_action(app, &games.stack);
    header.new_profile.set_action_name(Some("app.new"));
    header.settings.set_action_name(Some("app.settings"));

    select_first(&selection, &preview);

    let right = Paned::builder()
        .orientation(Orientation::Vertical)
        .position(360)
        .start_child(&games.stack)
        .end_child(&preview.container)
        .vexpand(true)
        .build();
    let root = Paned::builder()
        .orientation(Orientation::Horizontal)
        .position(200)
        .start_child(&sidebar.scroller)
        .end_child(&right)
        .vexpand(true)
        .build();
    let body = GtkBox::builder().orientation(Orientation::Vertical).build();
    body.append(&headerbar::build_menubar());
    body.append(&headerbar::build_toolbar());
    body.append(&root);
    window.set_child(Some(&body));

    actions::install_drop_target(&body, &reload);
    window.present();

    // Offer to add menu/desktop shortcuts when running as an AppImage (once).
    crate::ui::desktop_integration::maybe_prompt(&window);
}

/// A callback that rescans profiles, rebuilds the views + sidebar, and reselects.
fn make_reload(
    store: &gio::ListStore,
    profiles: &Profiles,
    selection: &SingleSelection,
    preview: &Preview,
    sidebar: &Rc<CategorySidebar>,
) -> Rc<dyn Fn()> {
    let store = store.clone();
    let profiles = profiles.clone();
    let selection = selection.clone();
    let preview = preview.clone();
    let sidebar = sidebar.clone();
    Rc::new(move || {
        *profiles.borrow_mut() = library::load_profiles();
        library::fill_store(&store, &profiles.borrow());
        sidebar.rebuild(&profiles.borrow()); // refresh categories + reset to All
        select_first(&selection, &preview);
    })
}

/// Select the first item (populating the preview), or clear it if empty.
fn select_first(selection: &SingleSelection, preview: &Preview) {
    if selection.n_items() > 0 {
        selection.set_selected(0);
    }
    refresh_preview(selection, preview);
}

/// Update the preview tabs to reflect the current selection.
fn refresh_preview(selection: &SingleSelection, preview: &Preview) {
    match library::selected_entry(selection) {
        Some((dir, profile)) => preview.show(&dir, &profile),
        None => preview.clear(),
    }
}
