//! The main application window: assembles the menu bar, toolbar, category
//! sidebar, cover grid, and detail pane, then wires selection → detail and the
//! reload callback. The pieces live in sibling modules (headerbar / library /
//! grid / detail / actions); this file is just the orchestration.

use std::cell::RefCell;
use std::rc::Rc;

use gtk::glib::BoxedAnyObject;
use gtk::prelude::*;
use gtk::{
    gio, Application, ApplicationWindow, Box as GtkBox, FilterChange, FilterListModel, GridView,
    Orientation, Paned, PopoverMenu, ScrolledWindow, SingleSelection,
};

use crate::app::APP_NAME;
use crate::ui::actions;
use crate::ui::category_sidebar::{Category, CategorySidebar};
use crate::ui::detail::{self, Detail};
use crate::ui::grid;
use crate::ui::headerbar;
use crate::ui::library::{self, Profiles};

pub fn build(app: &Application) {
    let profiles: Profiles = Rc::new(RefCell::new(library::load_profiles()));

    let window = ApplicationWindow::builder()
        .application(app)
        .title(APP_NAME)
        .default_width(940)
        .default_height(600)
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
    let selection = SingleSelection::new(Some(filter_model));

    let sidebar = Rc::new({
        let filter = filter.clone();
        let on_change: Rc<dyn Fn()> = Rc::new(move || filter.changed(FilterChange::Different));
        CategorySidebar::new(active_category.clone(), on_change)
    });
    sidebar.rebuild(&profiles.borrow());

    let grid_view = GridView::builder()
        .model(&selection)
        .max_columns(8)
        .min_columns(2)
        .build();
    let context_menu = PopoverMenu::from_model(Some(&headerbar::build_profile_menu()));
    context_menu.set_parent(&grid_view);
    context_menu.set_has_arrow(false);
    grid_view.set_factory(Some(&grid::build_factory(
        &selection,
        &grid_view,
        &context_menu,
    )));
    let grid_scroller = ScrolledWindow::builder()
        .child(&grid_view)
        .width_request(360)
        .build();

    let detail = detail::build_detail();
    {
        let detail = detail.clone();
        selection.connect_selected_notify(move |sel| refresh_detail(sel, &detail));
    }
    {
        let query = query.clone();
        let filter = filter.clone();
        header.search.connect_search_changed(move |entry| {
            *query.borrow_mut() = entry.text().to_string();
            filter.changed(FilterChange::Different);
        });
    }

    let reload = make_reload(&store, &profiles, &selection, &detail, &sidebar);

    actions::install_actions(app, &window, &selection, &profiles, &reload);
    detail.play.set_action_name(Some("app.play"));
    detail.edit.set_action_name(Some("app.edit"));
    header.new_profile.set_action_name(Some("app.new"));
    header.settings.set_action_name(Some("app.settings"));

    // Double-click / Enter on a cover -> Play.
    grid_view.connect_activate(|grid, _| {
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
    body.append(&headerbar::build_menubar());
    body.append(&headerbar::build_toolbar());
    body.append(&root);
    window.set_child(Some(&body));

    actions::install_drop_target(&body, &reload);
    window.present();
}

/// A callback that rescans profiles, rebuilds the grid + sidebar, and reselects.
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
        *profiles.borrow_mut() = library::load_profiles();
        library::fill_store(&store, &profiles.borrow());
        sidebar.rebuild(&profiles.borrow()); // refresh categories + reset to All
        select_first(&selection, &detail);
    })
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
    match library::selected_entry(selection) {
        Some((dir, profile)) => detail::show_profile(detail, &dir, &profile),
        None => detail::clear_detail(detail),
    }
}
