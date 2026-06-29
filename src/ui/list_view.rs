//! The details `ColumnView` (the "Details" view mode): one row per profile with
//! sortable-looking columns Title · Genre · Year · Developer · Publisher · Last
//! played, D-Fend style. Shares the window's `SingleSelection` with the icon
//! grid so switching view modes keeps the selection.

use gtk::glib::BoxedAnyObject;
use gtk::prelude::*;
use gtk::{
    Box as GtkBox, ColumnView, ColumnViewColumn, ContentFit, Label, ListItem, Orientation, Picture,
    PopoverMenu, SignalListItemFactory, SingleSelection,
};

use crate::config::profile::Profile;
use crate::ui::display::{apply_cover, display_title, last_played_cell};
use crate::ui::library::Entry;
use crate::ui::row_menu;

/// Build the details list over `selection`, parenting the context `menu` to it.
pub(crate) fn build(selection: &SingleSelection, menu: &PopoverMenu) -> ColumnView {
    let view = ColumnView::builder()
        .model(selection)
        .show_row_separators(true)
        .build();

    view.append_column(&title_column(selection, &view, menu));
    view.append_column(&text_column("Genre", false, |p| {
        p.genre.clone().unwrap_or_default()
    }));
    view.append_column(&text_column("Year", false, |p| {
        p.year.map(|y| y.to_string()).unwrap_or_default()
    }));
    view.append_column(&text_column("Developer", true, |p| {
        p.developer.clone().unwrap_or_default()
    }));
    view.append_column(&text_column("Publisher", true, |p| {
        p.publisher.clone().unwrap_or_default()
    }));
    view.append_column(&text_column("Last played", false, last_played_cell));
    view
}

/// The leftmost column: a small cover/console icon plus the (starred) title,
/// and the row's secondary-click context menu.
fn title_column(
    selection: &SingleSelection,
    view: &ColumnView,
    menu: &PopoverMenu,
) -> ColumnViewColumn {
    let factory = SignalListItemFactory::new();
    let selection = selection.clone();
    let view = view.clone();
    let menu = menu.clone();

    factory.connect_setup(move |_, item| {
        let item = item.downcast_ref::<ListItem>().expect("ListItem");
        let icon = Picture::builder()
            .content_fit(ContentFit::Contain)
            .width_request(28)
            .height_request(20)
            .build();
        let label = Label::builder()
            .halign(gtk::Align::Start)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .build();
        let cell = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .build();
        cell.append(&icon);
        cell.append(&label);
        item.set_child(Some(&cell));
        row_menu::wire(item, &cell, &selection, &view, &menu);
    });

    factory.connect_bind(|_, item| {
        let item = item.downcast_ref::<ListItem>().expect("ListItem");
        let Some(cell) = item.child().and_downcast::<GtkBox>() else {
            return;
        };
        let Some(icon) = cell.first_child().and_downcast::<Picture>() else {
            return;
        };
        let Some(label) = icon.next_sibling().and_downcast::<Label>() else {
            return;
        };
        let Some(obj) = item.item().and_downcast::<BoxedAnyObject>() else {
            return;
        };
        let entry = obj.borrow::<Entry>();
        let (dir, profile) = &*entry;
        label.set_text(&display_title(profile));
        apply_cover(&icon, dir, profile);
    });

    let column = ColumnViewColumn::new(Some("Title"), Some(factory));
    column.set_expand(true);
    column.set_resizable(true);
    column
}

/// A plain text column whose cell text comes from `get(profile)`.
fn text_column(
    title: &str,
    expand: bool,
    get: impl Fn(&Profile) -> String + 'static,
) -> ColumnViewColumn {
    let factory = SignalListItemFactory::new();
    factory.connect_setup(|_, item| {
        let item = item.downcast_ref::<ListItem>().expect("ListItem");
        let label = Label::builder()
            .halign(gtk::Align::Start)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .build();
        item.set_child(Some(&label));
    });
    factory.connect_bind(move |_, item| {
        let item = item.downcast_ref::<ListItem>().expect("ListItem");
        let Some(label) = item.child().and_downcast::<Label>() else {
            return;
        };
        let Some(obj) = item.item().and_downcast::<BoxedAnyObject>() else {
            return;
        };
        let entry = obj.borrow::<Entry>();
        label.set_text(&get(&entry.1));
    });

    let column = ColumnViewColumn::new(Some(title), Some(factory));
    column.set_expand(expand);
    column.set_resizable(true);
    column
}
