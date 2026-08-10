//! The two modal message dialogs the whole UI shares.
//!
//! Every failure a user should see goes through [`error`]; completions and
//! "nothing to do" notices through [`note`]. Both accept any window, so the main
//! window, the editor, the wizard, and the settings dialog use the same pair
//! instead of hand-rolling an `AlertDialog` each — they had drifted into six
//! near-identical copies.
//!
//! Rule of thumb: if a user action can fail, it reports through [`error`]. A
//! bare `log::error!` is invisible to someone running dosui from a launcher.

use gtk::prelude::*;
use gtk::AlertDialog;

/// Report a failed action: `message` is the headline, the error's full `{:#}`
/// chain the detail.
pub(crate) fn error(window: &impl IsA<gtk::Window>, message: &str, error: &anyhow::Error) {
    AlertDialog::builder()
        .modal(true)
        .message(message.to_string())
        .detail(format!("{error:#}"))
        .build()
        .show(Some(window));
}

/// Confirm that something happened (or that there was nothing to do).
pub(crate) fn note(window: &impl IsA<gtk::Window>, message: &str, detail: &str) {
    AlertDialog::builder()
        .modal(true)
        .message(message.to_string())
        .detail(detail.to_string())
        .build()
        .show(Some(window));
}
