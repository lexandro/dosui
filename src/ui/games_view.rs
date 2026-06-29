//! The games view: a `Stack` holding the details list and the icon grid, both
//! driven by the window's shared `SingleSelection` so switching modes keeps the
//! selection. Also owns the `app.view-mode` action that flips between them.

use gtk::glib;
use gtk::prelude::*;
use gtk::{
    gio, Application, ColumnView, GridView, PopoverMenu, ScrolledWindow, SingleSelection, Sorter,
    Stack,
};

use crate::ui::headerbar;
use crate::ui::{grid, list_view};

pub(crate) struct GamesView {
    pub stack: Stack,
    list: ColumnView,
}

/// Build the stack with the details list (default) and the icon grid.
pub(crate) fn build(selection: &SingleSelection) -> GamesView {
    let list = build_list(selection);
    let list_scroller = ScrolledWindow::builder().child(&list).vexpand(true).build();
    let grid_scroller = ScrolledWindow::builder()
        .child(&build_grid(selection))
        .vexpand(true)
        .build();

    let stack = Stack::new();
    stack.add_named(&list_scroller, Some("details"));
    stack.add_named(&grid_scroller, Some("icons"));
    stack.set_visible_child_name("details");
    GamesView { stack, list }
}

impl GamesView {
    /// The details view's header sorter — wire a `SortListModel` to it so column
    /// clicks reorder the list. Unsorted until a header is clicked.
    pub(crate) fn list_sorter(&self) -> Option<Sorter> {
        self.list.sorter()
    }
}

/// The details `ColumnView` with its own context menu, Enter/double-click → Run.
fn build_list(selection: &SingleSelection) -> gtk::ColumnView {
    let menu = PopoverMenu::from_model(Some(&headerbar::build_profile_menu()));
    let list = list_view::build(selection, &menu);
    menu.set_parent(&list);
    menu.set_has_arrow(false);
    list.connect_activate(|view, _| {
        let _ = WidgetExt::activate_action(view, "app.play", None);
    });
    list
}

/// The icon `GridView` with its own context menu, Enter/double-click → Run.
fn build_grid(selection: &SingleSelection) -> GridView {
    let menu = PopoverMenu::from_model(Some(&headerbar::build_profile_menu()));
    let grid_view = GridView::builder()
        .model(selection)
        .max_columns(8)
        .min_columns(2)
        .build();
    menu.set_parent(&grid_view);
    menu.set_has_arrow(false);
    grid_view.set_factory(Some(&grid::build_factory(selection, &grid_view, &menu)));
    grid_view.connect_activate(|grid, _| {
        let _ = WidgetExt::activate_action(grid, "app.play", None);
    });
    grid_view
}

/// Stateful `app.view-mode` action ("details" / "icons") switching the stack.
pub(crate) fn install_view_mode_action(app: &Application, stack: &Stack) {
    let action = gio::SimpleAction::new_stateful(
        "view-mode",
        Some(glib::VariantTy::STRING),
        &"details".to_variant(),
    );
    let stack = stack.clone();
    action.connect_change_state(move |action, value| {
        if let Some(name) = value.and_then(|v| v.get::<String>()) {
            stack.set_visible_child_name(&name);
            action.set_state(&name.to_variant());
        }
    });
    app.add_action(&action);
}
