//! The details `ColumnView` (the "Details" view mode): one row per profile with
//! click-to-sort columns Title · Genre · Year · Developer · Publisher · Last
//! played, D-Fend style. Shares the window's `SingleSelection` with the icon
//! grid so switching view modes keeps the selection; the window wires a
//! `SortListModel` to this view's [`ColumnView::sorter`] to apply header sorts.
//!
//! Over the 150-line soft cap by design: one cohesive widget — the column
//! definitions plus their per-column cell factories and sorters belong together.

use std::rc::Rc;

use gtk::glib::{self, BoxedAnyObject};
use gtk::prelude::*;
use gtk::{
    Box as GtkBox, ColumnView, ColumnViewColumn, ContentFit, CustomSorter, Label, ListItem,
    Orientation, Picture, PopoverMenu, SignalListItemFactory, SingleSelection,
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
    view.append_column(&column(
        "Genre",
        false,
        Rc::new(|p| p.genre.clone().unwrap_or_default()),
        string_sorter(Rc::new(|p| p.genre.clone().unwrap_or_default())),
    ));
    view.append_column(&column(
        "Year",
        false,
        Rc::new(|p| p.year.map(|y| y.to_string()).unwrap_or_default()),
        num_sorter(|p| p.year.unwrap_or(0) as u64),
    ));
    view.append_column(&column(
        "Developer",
        true,
        Rc::new(|p| p.developer.clone().unwrap_or_default()),
        string_sorter(Rc::new(|p| p.developer.clone().unwrap_or_default())),
    ));
    view.append_column(&column(
        "Publisher",
        true,
        Rc::new(|p| p.publisher.clone().unwrap_or_default()),
        string_sorter(Rc::new(|p| p.publisher.clone().unwrap_or_default())),
    ));
    view.append_column(&column(
        "Last played",
        false,
        Rc::new(last_played_cell),
        num_sorter(|p| p.last_played.unwrap_or(0)),
    ));
    view
}

/// The leftmost column: a small cover/console icon plus the (starred) title,
/// the row's secondary-click context menu, and a title sorter.
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
    column.set_sorter(Some(&string_sorter(Rc::new(|p| p.title.clone()))));
    column
}

/// A plain text column: `display` fills each cell, `sorter` orders the header.
fn column(
    title: &str,
    expand: bool,
    display: Rc<dyn Fn(&Profile) -> String>,
    sorter: CustomSorter,
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
        label.set_text(&display(&entry.1));
    });

    let column = ColumnViewColumn::new(Some(title), Some(factory));
    column.set_expand(expand);
    column.set_resizable(true);
    column.set_sorter(Some(&sorter));
    column
}

/// Case-insensitive sorter on a string key.
fn string_sorter(key: Rc<dyn Fn(&Profile) -> String>) -> CustomSorter {
    CustomSorter::new(move |a, b| {
        with_profiles(a, b, |pa, pb| {
            key(pa).to_lowercase().cmp(&key(pb).to_lowercase())
        })
    })
}

/// Numeric sorter on an integer key (year, timestamp).
fn num_sorter(key: impl Fn(&Profile) -> u64 + 'static) -> CustomSorter {
    CustomSorter::new(move |a, b| with_profiles(a, b, |pa, pb| key(pa).cmp(&key(pb))))
}

/// Run `cmp` on the two boxed profiles, yielding a GTK ordering (Equal on a
/// downcast miss — never happens, the store only holds boxed entries).
fn with_profiles(
    a: &glib::Object,
    b: &glib::Object,
    cmp: impl Fn(&Profile, &Profile) -> std::cmp::Ordering,
) -> gtk::Ordering {
    let (Some(oa), Some(ob)) = (
        a.downcast_ref::<BoxedAnyObject>(),
        b.downcast_ref::<BoxedAnyObject>(),
    ) else {
        return gtk::Ordering::Equal;
    };
    cmp(&oa.borrow::<Entry>().1, &ob.borrow::<Entry>().1).into()
}
