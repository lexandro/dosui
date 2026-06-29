//! The cover `GridView` factory (the "Icons" view mode): renders each cell
//! (cover + title) and wires the secondary-click context menu.

use gtk::glib::BoxedAnyObject;
use gtk::prelude::*;
use gtk::{
    Box as GtkBox, ContentFit, GridView, Label, ListItem, Orientation, Picture, PopoverMenu,
    SignalListItemFactory, SingleSelection,
};

use crate::ui::display::{apply_cover, display_title};
use crate::ui::library::Entry;
use crate::ui::row_menu;

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
        row_menu::wire(item, &cell, &selection, &grid, &menu);
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
        apply_cover(&cover, dir, profile);
    });

    factory
}
