//! Shared secondary-click context menu wiring for the games views.
//!
//! A right-click on a cell selects its row and pops the profile menu up at the
//! pointer. Used by both the icon grid and the details list so they behave
//! identically; `anchor` is the view the popover is parented to (grid / column
//! view), needed to translate the click point into the popover's coordinates.

use gtk::prelude::*;
use gtk::{ListItem, PopoverMenu, SingleSelection};

/// Wire a secondary-click on `cell` to select `item`'s row and open `menu`.
pub(crate) fn wire(
    item: &ListItem,
    cell: &impl IsA<gtk::Widget>,
    selection: &SingleSelection,
    anchor: &impl IsA<gtk::Widget>,
    menu: &PopoverMenu,
) {
    let gesture = gtk::GestureClick::new();
    gesture.set_button(gtk::gdk::BUTTON_SECONDARY);
    let selection = selection.clone();
    let anchor = anchor.clone().upcast::<gtk::Widget>();
    let menu = menu.clone();
    let item = item.clone();
    let cell_click = cell.clone().upcast::<gtk::Widget>();
    gesture.connect_pressed(move |gesture, _, x, y| {
        let pos = item.position();
        if pos != gtk::INVALID_LIST_POSITION {
            selection.set_selected(pos);
        }
        let point = gtk::graphene::Point::new(x as f32, y as f32);
        let (ax, ay) = cell_click
            .compute_point(&anchor, &point)
            .map(|p| (p.x() as i32, p.y() as i32))
            .unwrap_or((x as i32, y as i32));
        menu.set_pointing_to(Some(&gtk::gdk::Rectangle::new(ax, ay, 1, 1)));
        menu.popup();
        gesture.set_state(gtk::EventSequenceState::Claimed);
    });
    cell.add_controller(gesture);
}
