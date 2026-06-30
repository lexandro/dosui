//! Input tab: keyboard layout, mouse, and joystick — grouped on one page.

use gtk::prelude::*;
use gtk::{Box as GtkBox, DropDown, Entry};

use super::rows::{bool_opt, cfg_bool, cfg_opt, config_row, heading, Ctx, DEFAULT};
use crate::config::dosbox_conf::DosboxConfig;
use crate::ui::widgets;

const CAPTURE_OPTS: [&str; 5] = [DEFAULT, "onclick", "onstart", "seamless", "nomouse"];
const JOYSTICK_OPTS: [&str; 9] = [
    DEFAULT, "auto", "2axis", "4axis", "4axis_2", "fcs", "ch", "hidden", "disabled",
];
const ONOFF_OPTS: [&str; 3] = [DEFAULT, "on", "off"];

const DEF_KEYBOARDLAYOUT: &str = "auto";
const DEF_CAPTURE: &str = "onclick";
const DEF_SENSITIVITY: &str = "100";
const DEF_JOYSTICK: &str = "auto";
const DEF_AUTOFIRE: &str = "off";
const DEF_SWAP34: &str = "off";

/// Input-tab widgets, read back by [`Widgets::apply`].
pub(super) struct Widgets {
    keyboardlayout: Entry,
    mouse_capture: DropDown,
    mouse_sensitivity: Entry,
    joysticktype: DropDown,
    autofire: DropDown,
    swap34: DropDown,
}

/// Build the Input page and its read-back widgets.
pub(super) fn build(config: &DosboxConfig, ctx: &Ctx) -> (GtkBox, Widgets) {
    let page = widgets::page();

    page.append(&heading("Keyboard"));
    let (row, keyboardlayout) = widgets::entry_row("Layout", widgets::opt(&config.keyboardlayout));
    keyboardlayout.set_placeholder_text(Some(&ctx.placeholder(DEF_KEYBOARDLAYOUT)));
    page.append(&row);

    page.append(&heading("Mouse"));
    let (row, mouse_capture) = config_row(
        "Capture",
        &CAPTURE_OPTS,
        config.mouse_capture.as_deref(),
        &ctx.sentinel(DEF_CAPTURE),
    );
    page.append(&row);
    let (row, mouse_sensitivity) =
        widgets::entry_row("Sensitivity (%)", widgets::opt(&config.mouse_sensitivity));
    mouse_sensitivity.set_placeholder_text(Some(&ctx.placeholder(DEF_SENSITIVITY)));
    page.append(&row);

    page.append(&heading("Joystick"));
    let (row, joysticktype) = config_row(
        "Type",
        &JOYSTICK_OPTS,
        config.joysticktype.as_deref(),
        &ctx.sentinel(DEF_JOYSTICK),
    );
    page.append(&row);
    let (row, autofire) = config_row(
        "Autofire",
        &ONOFF_OPTS,
        bool_opt(config.joy_autofire),
        &ctx.sentinel(DEF_AUTOFIRE),
    );
    page.append(&row);
    let (row, swap34) = config_row(
        "Swap buttons 3 & 4",
        &ONOFF_OPTS,
        bool_opt(config.joy_swap34),
        &ctx.sentinel(DEF_SWAP34),
    );
    page.append(&row);

    (
        page,
        Widgets {
            keyboardlayout,
            mouse_capture,
            mouse_sensitivity,
            joysticktype,
            autofire,
            swap34,
        },
    )
}

impl Widgets {
    /// Write the Input fields into `cfg`.
    pub(super) fn apply(&self, cfg: &mut DosboxConfig) {
        cfg.keyboardlayout = widgets::none_if_empty(&self.keyboardlayout.text());
        cfg.mouse_capture = cfg_opt(&self.mouse_capture);
        cfg.mouse_sensitivity = widgets::none_if_empty(&self.mouse_sensitivity.text());
        cfg.joysticktype = cfg_opt(&self.joysticktype);
        cfg.joy_autofire = cfg_bool(&self.autofire);
        cfg.joy_swap34 = cfg_bool(&self.swap34);
    }
}
