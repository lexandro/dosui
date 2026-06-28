//! The cover `GridView` factory: renders each cell (cover + title) and wires the
//! secondary-click context menu.

use gtk::glib::BoxedAnyObject;
use gtk::prelude::*;
use gtk::{
    Box as GtkBox, ContentFit, GridView, Label, ListItem, Orientation, Picture, PopoverMenu,
    SignalListItemFactory, SingleSelection,
};

use crate::ui::detail::{cover_path, display_title};
use crate::ui::library::Entry;

/// Build the factory for the grid, given the selection (for click-to-select),
/// the grid (to anchor the menu), and the shared context menu.
pub(crate) fn build_factory(
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
        wire_context_menu(item, &cell, &selection, &grid, &menu);
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

/// Secondary-click on a cell selects it and opens the context menu at the pointer.
fn wire_context_menu(
    item: &ListItem,
    cell: &GtkBox,
    selection: &SingleSelection,
    grid: &GridView,
    menu: &PopoverMenu,
) {
    let gesture = gtk::GestureClick::new();
    gesture.set_button(gtk::gdk::BUTTON_SECONDARY);
    let selection = selection.clone();
    let grid = grid.clone();
    let menu = menu.clone();
    let item = item.clone();
    let cell_click = cell.clone();
    gesture.connect_pressed(move |gesture, _, x, y| {
        let pos = item.position();
        if pos != gtk::INVALID_LIST_POSITION {
            selection.set_selected(pos);
        }
        let point = gtk::graphene::Point::new(x as f32, y as f32);
        let (gx, gy) = cell_click
            .compute_point(&grid, &point)
            .map(|p| (p.x() as i32, p.y() as i32))
            .unwrap_or((x as i32, y as i32));
        menu.set_pointing_to(Some(&gtk::gdk::Rectangle::new(gx, gy, 1, 1)));
        menu.popup();
        gesture.set_state(gtk::EventSequenceState::Claimed);
    });
    cell.add_controller(gesture);
}
