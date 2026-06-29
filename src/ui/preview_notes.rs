//! The preview "Notes" tab: an editable text area bound to the selected
//! profile's `notes`. Persists straight to `profile.toml` on focus-out and when
//! switching profiles — it re-reads the profile from disk before writing so it
//! never clobbers other fields, and never triggers a library reload (notes are
//! not shown in the list, so there is nothing to refresh).

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{EventControllerFocus, ScrolledWindow, TextView, WrapMode};

use crate::config::profile::Profile;

#[derive(Clone)]
pub(crate) struct NotesTab {
    pub root: ScrolledWindow,
    view: TextView,
    /// Directory of the profile currently shown (for the focus-out flush).
    current: Rc<RefCell<Option<PathBuf>>>,
}

pub(crate) fn build() -> NotesTab {
    let view = TextView::builder()
        .wrap_mode(WrapMode::WordChar)
        .left_margin(8)
        .right_margin(8)
        .top_margin(8)
        .bottom_margin(8)
        .build();
    let root = ScrolledWindow::builder().child(&view).build();
    let current: Rc<RefCell<Option<PathBuf>>> = Rc::new(RefCell::new(None));

    // Save when focus leaves the editor (clicking away, switching window).
    let focus = EventControllerFocus::new();
    {
        let view = view.clone();
        let current = current.clone();
        focus.connect_leave(move |_| flush(&view, &current.borrow()));
    }
    view.add_controller(focus);

    NotesTab {
        root,
        view,
        current,
    }
}

impl NotesTab {
    /// Show a profile's notes, flushing any pending edits of the previous one.
    pub(crate) fn show(&self, dir: &Path, profile: &Profile) {
        flush(&self.view, &self.current.borrow());
        self.view
            .buffer()
            .set_text(profile.notes.as_deref().unwrap_or(""));
        *self.current.borrow_mut() = Some(dir.to_path_buf());
    }

    /// Flush pending edits and clear the editor (nothing selected).
    pub(crate) fn clear(&self) {
        flush(&self.view, &self.current.borrow());
        self.view.buffer().set_text("");
        *self.current.borrow_mut() = None;
    }
}

/// Persist the editor's text into `<dir>/profile.toml` if it changed.
fn flush(view: &TextView, dir: &Option<PathBuf>) {
    let Some(dir) = dir else { return };
    let buffer = view.buffer();
    let text = buffer
        .text(&buffer.start_iter(), &buffer.end_iter(), false)
        .to_string();
    let notes = (!text.trim().is_empty()).then_some(text);

    let mut profile = match Profile::load(dir) {
        Ok(p) => p,
        Err(e) => {
            log::warn!("notes: reloading {} failed: {e:#}", dir.display());
            return;
        }
    };
    if profile.notes == notes {
        return; // unchanged
    }
    profile.notes = notes;
    if let Err(e) = profile.save(dir) {
        log::error!("saving notes to {} failed: {e:#}", dir.display());
    }
}
