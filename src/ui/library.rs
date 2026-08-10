//! Shared profile-library glue for the main window: the loaded entries and the
//! GTK store / filter / selection helpers that operate on them.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk::glib::BoxedAnyObject;
use gtk::prelude::*;
use gtk::{gio, CustomFilter, SingleSelection};

use crate::config::profile::{self, Profile};
use crate::ui::category_sidebar::Category;

/// Loaded profile together with its on-disk directory (needed to launch).
pub(crate) type Entry = (PathBuf, Profile);

/// Shared, reloadable profile list mirrored by the grid's ListStore.
pub(crate) type Profiles = Rc<RefCell<Vec<Entry>>>;

/// Load every profile from the data dir; an error yields an empty list (logged).
pub(crate) fn load_profiles() -> Vec<Entry> {
    match crate::config::paths::profiles_dir().and_then(|dir| profile::scan(&dir)) {
        Ok(entries) => entries,
        Err(e) => {
            log::error!("loading profiles: {e:#}");
            Vec::new()
        }
    }
}

/// Replace the store's contents with one boxed entry per profile.
pub(crate) fn fill_store(store: &gio::ListStore, profiles: &[Entry]) {
    store.remove_all();
    for entry in profiles {
        store.append(&BoxedAnyObject::new(entry.clone()));
    }
}

/// Filter on the title substring (search) AND the active category.
pub(crate) fn build_filter(
    query: &Rc<RefCell<String>>,
    category: &Rc<RefCell<Category>>,
) -> CustomFilter {
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
pub(crate) fn selected_entry(selection: &SingleSelection) -> Option<Entry> {
    selection
        .selected_item()
        .and_downcast::<BoxedAnyObject>()
        .map(|o| o.borrow::<Entry>().clone())
}

/// The selected profile's id, if any. Paired with [`select_by_id`] to hold a
/// user's place across a reload, which rebuilds the whole store.
pub(crate) fn selected_id(selection: &SingleSelection) -> Option<String> {
    selected_entry(selection).map(|(_, p)| p.id)
}

/// Select the entry with this profile id. Returns `false` when it isn't in the
/// view — it may have been deleted, renamed, or filtered out by the sidebar.
pub(crate) fn select_by_id(selection: &SingleSelection, id: &str) -> bool {
    for i in 0..selection.n_items() {
        let Some(obj) = selection.item(i).and_downcast::<BoxedAnyObject>() else {
            continue;
        };
        if obj.borrow::<Entry>().1.id == id {
            selection.set_selected(i);
            return true;
        }
    }
    false
}
