//! Modal profile editor (GtkNotebook with one tab per concern).
//!
//! The editor loads a full [`Profile`], lets the user edit the fields the tabs
//! expose, and on Save writes `profile.toml` back. Any field a tab does not
//! expose is preserved from the loaded profile (see [`collect`]). `on_saved`
//! lets the caller refresh the main list.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk::gio;
use gtk::glib::WeakRef;
use gtk::prelude::*;
use gtk::{
    AlertDialog, ApplicationWindow, Box as GtkBox, Button, CheckButton, DropDown, Entry,
    FileDialog, Label, Notebook, Orientation, ScrolledWindow, TextView, Window,
};
use indexmap::IndexMap;

use crate::config::dosbox_conf::DosboxConfig;
use crate::config::profile::{Mount, MountKind, Profile, RunSpec};
use crate::ui::widgets;

/// Sentinel first option meaning "don't set this key — use the DOSBox default".
const DEFAULT: &str = "(default)";

// Curated option lists for the DOSBox tabs. The profile's current value is added
// dynamically if it isn't in the list, so arbitrary values are never lost.
const OUTPUT_OPTS: [&str; 4] = [DEFAULT, "texture", "texturenb", "opengl"];
const MACHINE_OPTS: [&str; 8] = [
    DEFAULT,
    "svga_s3",
    "svga_et4000",
    "vesa_nolfb",
    "vgaonly",
    "ega",
    "cga",
    "hercules",
];
const MEMSIZE_OPTS: [&str; 7] = [DEFAULT, "1", "4", "8", "16", "32", "64"];
const CORE_OPTS: [&str; 5] = [DEFAULT, "auto", "normal", "dynamic", "simple"];
const CPUTYPE_OPTS: [&str; 6] = [
    DEFAULT,
    "auto",
    "386",
    "386_slow",
    "486_slow",
    "pentium_slow",
];
const CYCLES_OPTS: [&str; 7] = [
    DEFAULT,
    "auto",
    "max",
    "fixed 3000",
    "fixed 10000",
    "fixed 20000",
    "fixed 30000",
];
const SCALER_OPTS: [&str; 4] = [DEFAULT, "none", "normal2x", "normal3x"];
const ASPECT_OPTS: [&str; 3] = [DEFAULT, "on", "off"];
const SBTYPE_OPTS: [&str; 6] = [DEFAULT, "sb16", "sbpro2", "sb2", "gb", "none"];
const RATE_OPTS: [&str; 6] = [DEFAULT, "22050", "32000", "44100", "48000", "49716"];

/// Drive-letter options (A–Z) shared by the working-drive and mount dropdowns.
/// A static table so the `&str` refs are valid for both building and read-back.
fn drive_letters() -> Vec<&'static str> {
    const L: [&str; 26] = [
        "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R",
        "S", "T", "U", "V", "W", "X", "Y", "Z",
    ];
    L.to_vec()
}

const KIND_OPTIONS: [&str; 4] = ["Directory", "CD image", "Floppy image", "HDD image"];

fn kind_to_text(kind: MountKind) -> &'static str {
    match kind {
        MountKind::Directory => "Directory",
        MountKind::CdImage => "CD image",
        MountKind::FloppyImage => "Floppy image",
        MountKind::HddImage => "HDD image",
    }
}

fn text_to_kind(text: &str) -> MountKind {
    match text {
        "CD image" => MountKind::CdImage,
        "Floppy image" => MountKind::FloppyImage,
        "HDD image" => MountKind::HddImage,
        _ => MountKind::Directory,
    }
}

/// All editor input widgets, grouped per tab and read back on Save.
struct Fields {
    general: General,
    run: Run,
    dos: Dos,
}

/// DOSBox config widgets (CPU / Graphics / Sound) + Advanced passthrough/preview.
struct Dos {
    output: DropDown,
    machine: DropDown,
    memsize: DropDown,
    scaler: DropDown,
    aspect: DropDown,
    core: DropDown,
    cputype: DropDown,
    cycles: DropDown,
    sbtype: DropDown,
    rate: DropDown,
    passthrough: TextView,
    preview: TextView,
}

struct General {
    title: Entry,
    genre: Entry,
    year: Entry,
    developer: Entry,
    publisher: Entry,
    www: Entry,
    notes: TextView,
}

struct Run {
    working_drive: DropDown,
    command: Entry,
    args: Entry,
    exit_after: CheckButton,
    mounts: Rc<RefCell<Vec<MountRow>>>,
}

/// One mount entry's widgets, kept so we can read it back and remove it.
struct MountRow {
    container: GtkBox,
    drive: DropDown,
    kind: DropDown,
    path: Entry,
    label: Entry,
}

/// Where a save goes: an existing directory, or a new one derived from the title.
/// Open the editor for an existing profile stored in `dir`. New profiles are
/// created via the wizard ([`crate::ui::wizard`]); the editor only edits.
pub fn open_for_edit(
    parent: &ApplicationWindow,
    dir: PathBuf,
    profile: Profile,
    on_saved: Rc<dyn Fn()>,
) {
    let window = Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(format!("Edit — {}", profile.title))
        .default_width(620)
        .default_height(560)
        .build();

    let notebook = Notebook::new();
    notebook.set_vexpand(true);

    let general = build_general(&profile);
    notebook.append_page(&general.0, Some(&Label::new(Some("General"))));
    let run = build_run(&profile, &window);
    notebook.append_page(&run.0, Some(&Label::new(Some("Mounts & Run"))));
    let (cpu_page, graphics_page, sound_page, advanced_page, dos) = build_dos_tabs(&profile);
    notebook.append_page(&cpu_page, Some(&Label::new(Some("CPU"))));
    notebook.append_page(&graphics_page, Some(&Label::new(Some("Graphics"))));
    notebook.append_page(&sound_page, Some(&Label::new(Some("Sound"))));
    let advanced_index = notebook.append_page(&advanced_page, Some(&Label::new(Some("Advanced"))));

    let fields = Rc::new(Fields {
        general: general.1,
        run: run.1,
        dos,
    });
    let original = Rc::new(profile);

    // Refresh the read-only conf preview whenever the Advanced tab is shown.
    {
        let fields = fields.clone();
        let original = original.clone();
        notebook.connect_switch_page(move |_, _, index| {
            if index == advanced_index {
                let p = collect(&fields, &original);
                fields
                    .dos
                    .preview
                    .buffer()
                    .set_text(&p.dosbox.render(&p.run));
            }
        });
    }

    let actions = action_bar();
    let outer = GtkBox::builder().orientation(Orientation::Vertical).build();
    outer.append(&notebook);
    outer.append(&actions.container);
    window.set_child(Some(&outer));
    window.set_default_widget(Some(&actions.save)); // Enter confirms

    // Expose the editor's Save/Cancel as app actions while it is open, so the
    // whole flow is driveable from outside (e.g. `gapplication action … editor-save`).
    if let Some(app) = parent.application() {
        register_editor_actions(&app, &window, &actions.save, &actions.cancel);
    }

    {
        let window = window.clone();
        actions.cancel.connect_clicked(move |_| window.close());
    }
    {
        let window = window.clone();
        actions.save.connect_clicked(move |_| {
            let updated = collect(&fields, &original);
            match updated.save(&dir) {
                Ok(()) => {
                    on_saved();
                    window.close();
                }
                Err(e) => {
                    log::error!("saving profile failed: {e:#}");
                    show_error(&window, "Could not save profile", &e);
                }
            }
        });
    }

    window.present();
}

struct ActionBar {
    container: GtkBox,
    cancel: Button,
    save: Button,
}

fn action_bar() -> ActionBar {
    let cancel = Button::with_label("Cancel");
    let save = Button::builder()
        .label("Save")
        .css_classes(["suggested-action"])
        .build();
    let container = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .halign(gtk::Align::End)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(10)
        .margin_end(10)
        .build();
    container.append(&cancel);
    container.append(&save);
    ActionBar {
        container,
        cancel,
        save,
    }
}

/// General tab: metadata.
fn build_general(profile: &Profile) -> (GtkBox, General) {
    let page = widgets::page();

    let (row, title) = widgets::entry_row("Title", &profile.title);
    page.append(&row);
    let (row, genre) = widgets::entry_row("Genre", opt(&profile.genre));
    page.append(&row);
    let (row, year) = widgets::entry_row(
        "Year",
        &profile.year.map(|y| y.to_string()).unwrap_or_default(),
    );
    page.append(&row);
    let (row, developer) = widgets::entry_row("Developer", opt(&profile.developer));
    page.append(&row);
    let (row, publisher) = widgets::entry_row("Publisher", opt(&profile.publisher));
    page.append(&row);
    let (row, www) = widgets::entry_row("Website", opt(&profile.www));
    page.append(&row);

    page.append(
        &Label::builder()
            .label("Notes")
            .halign(gtk::Align::Start)
            .build(),
    );
    let notes = TextView::new();
    notes
        .buffer()
        .set_text(profile.notes.as_deref().unwrap_or(""));
    notes.set_wrap_mode(gtk::WrapMode::WordChar);
    page.append(
        &ScrolledWindow::builder()
            .child(&notes)
            .min_content_height(110)
            .vexpand(true)
            .build(),
    );

    let widgets = General {
        title,
        genre,
        year,
        developer,
        publisher,
        www,
        notes,
    };
    (page, widgets)
}

/// Mounts & Run tab: the run command plus a dynamic list of mounts.
fn build_run(profile: &Profile, window: &Window) -> (GtkBox, Run) {
    let page = widgets::page();

    let (row, working_drive) = widgets::dropdown_row(
        "Working drive",
        &drive_letters(),
        Some(&profile.run.working_drive.to_string()),
    );
    page.append(&row);
    let (row, command) = widgets::entry_row("Command", &profile.run.command);
    page.append(&row);
    let (row, args) = widgets::entry_row("Arguments", &profile.run.args.join(" "));
    page.append(&row);
    let (row, exit_after) =
        widgets::check_row("Exit DOSBox when the program quits", profile.run.exit_after);
    page.append(&row);

    page.append(
        &Label::builder()
            .label("Mounts")
            .halign(gtk::Align::Start)
            .build(),
    );
    let mounts_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .build();
    page.append(
        &ScrolledWindow::builder()
            .child(&mounts_box)
            .min_content_height(140)
            .vexpand(true)
            .build(),
    );

    let mounts: Rc<RefCell<Vec<MountRow>>> = Rc::new(RefCell::new(Vec::new()));
    for mount in &profile.run.mounts {
        add_mount_row(window, &mounts_box, &mounts, Some(mount));
    }

    let add = Button::builder()
        .label("Add mount")
        .halign(gtk::Align::Start)
        .build();
    {
        let window = window.clone();
        let mounts_box = mounts_box.clone();
        let mounts = mounts.clone();
        add.connect_clicked(move |_| add_mount_row(&window, &mounts_box, &mounts, None));
    }
    page.append(&add);

    let widgets = Run {
        working_drive,
        command,
        args,
        exit_after,
        mounts,
    };
    (page, widgets)
}

/// Append one mount row (pre-filled from `mount` when editing) and track it.
fn add_mount_row(
    window: &Window,
    mounts_box: &GtkBox,
    mounts: &Rc<RefCell<Vec<MountRow>>>,
    mount: Option<&Mount>,
) {
    let drive = widgets::dropdown(
        &drive_letters(),
        Some(&mount.map(|m| m.drive).unwrap_or('C').to_string()),
    );
    let kind = widgets::dropdown(
        &KIND_OPTIONS,
        Some(kind_to_text(
            mount.map(|m| m.kind).unwrap_or(MountKind::Directory),
        )),
    );
    let path = Entry::builder()
        .text(
            mount
                .map(|m| m.path.display().to_string())
                .unwrap_or_default(),
        )
        .hexpand(true)
        .build();
    let browse = Button::with_label("Browse…");
    let label = Entry::builder()
        .placeholder_text("label (optional)")
        .text(mount.and_then(|m| m.label.clone()).unwrap_or_default())
        .width_request(120)
        .build();
    let remove = Button::from_icon_name("list-remove-symbolic");

    let container = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();
    container.append(&drive);
    container.append(&kind);
    container.append(&path);
    container.append(&browse);
    container.append(&label);
    container.append(&remove);
    mounts_box.append(&container);

    // Browse: folder picker for directories, file picker for images.
    {
        let window = window.downgrade();
        let path = path.clone();
        let kind = kind.clone();
        browse.connect_clicked(move |_| pick_path(&window, &path, &kind));
    }
    // Remove: drop the row from the UI and the tracking vec.
    {
        let mounts_box = mounts_box.clone();
        let mounts = mounts.clone();
        let container = container.clone();
        remove.connect_clicked(move |_| {
            mounts_box.remove(&container);
            mounts.borrow_mut().retain(|r| r.container != container);
        });
    }

    mounts.borrow_mut().push(MountRow {
        container,
        drive,
        kind,
        path,
        label,
    });
}

/// Open a file/folder chooser and write the chosen path into `entry`.
fn pick_path(window: &WeakRef<Window>, entry: &Entry, kind: &DropDown) {
    let dialog = FileDialog::builder().title("Select mount path").build();
    let parent = window.upgrade();
    let entry = entry.clone();
    let is_dir = widgets::dropdown_selected(kind).as_deref() == Some("Directory");

    let on_done = move |res: Result<gio::File, gtk::glib::Error>| {
        if let Ok(file) = res {
            if let Some(path) = file.path() {
                entry.set_text(&path.display().to_string());
            }
        }
    };
    if is_dir {
        dialog.select_folder(parent.as_ref(), gio::Cancellable::NONE, on_done);
    } else {
        dialog.open(parent.as_ref(), gio::Cancellable::NONE, on_done);
    }
}

/// Build the CPU / Graphics / Sound / Advanced tabs and their widget group.
/// Returns the four pages in tab order plus the [`Dos`] widgets.
fn build_dos_tabs(profile: &Profile) -> (GtkBox, GtkBox, GtkBox, GtkBox, Dos) {
    let d = &profile.dosbox;

    let cpu_page = widgets::page();
    let (row, core) = config_row("Core", &CORE_OPTS, d.core.as_deref());
    cpu_page.append(&row);
    let (row, cputype) = config_row("CPU type", &CPUTYPE_OPTS, d.cputype.as_deref());
    cpu_page.append(&row);
    let (row, cycles) = config_row("Cycles", &CYCLES_OPTS, d.cycles.as_deref());
    cpu_page.append(&row);

    let graphics_page = widgets::page();
    let (row, output) = config_row("Output", &OUTPUT_OPTS, d.output.as_deref());
    graphics_page.append(&row);
    let (row, machine) = config_row("Machine", &MACHINE_OPTS, d.machine.as_deref());
    graphics_page.append(&row);
    let memsize_cur = d.memsize.map(|v| v.to_string());
    let (row, memsize) = config_row("Memory (MB)", &MEMSIZE_OPTS, memsize_cur.as_deref());
    graphics_page.append(&row);
    let (row, scaler) = config_row("Scaler", &SCALER_OPTS, d.scaler.as_deref());
    graphics_page.append(&row);
    let aspect_cur = d.aspect.map(|b| if b { "on" } else { "off" });
    let (row, aspect) = config_row("Aspect correction", &ASPECT_OPTS, aspect_cur);
    graphics_page.append(&row);

    let sound_page = widgets::page();
    let (row, sbtype) = config_row("Sound Blaster", &SBTYPE_OPTS, d.sbtype.as_deref());
    sound_page.append(&row);
    let rate_cur = d.rate.map(|v| v.to_string());
    let (row, rate) = config_row("Mixer rate (Hz)", &RATE_OPTS, rate_cur.as_deref());
    sound_page.append(&row);

    let advanced_page = widgets::page();
    advanced_page.append(
        &Label::builder()
            .label("Advanced keys — one per line, e.g. cpu.cycleup = 500")
            .halign(gtk::Align::Start)
            .build(),
    );
    let passthrough = TextView::new();
    passthrough.set_monospace(true);
    passthrough
        .buffer()
        .set_text(&serialize_passthrough(&d.passthrough));
    advanced_page.append(
        &ScrolledWindow::builder()
            .child(&passthrough)
            .min_content_height(120)
            .vexpand(true)
            .build(),
    );
    advanced_page.append(
        &Label::builder()
            .label("Generated dosbox.conf preview")
            .halign(gtk::Align::Start)
            .build(),
    );
    let preview = TextView::builder().editable(false).monospace(true).build();
    advanced_page.append(
        &ScrolledWindow::builder()
            .child(&preview)
            .min_content_height(150)
            .vexpand(true)
            .build(),
    );

    let dos = Dos {
        output,
        machine,
        memsize,
        scaler,
        aspect,
        core,
        cputype,
        cycles,
        sbtype,
        rate,
        passthrough,
        preview,
    };
    (cpu_page, graphics_page, sound_page, advanced_page, dos)
}

/// A config dropdown row: curated `base` options, plus the profile's current
/// value if it isn't already listed (so custom values survive a round-trip).
fn config_row(label: &str, base: &[&str], current: Option<&str>) -> (GtkBox, DropDown) {
    let current = current.filter(|c| !c.is_empty());
    let mut opts: Vec<&str> = base.to_vec();
    if let Some(c) = current {
        if !opts.contains(&c) {
            opts.insert(1, c); // right after "(default)"
        }
    }
    widgets::dropdown_row(label, &opts, Some(current.unwrap_or(DEFAULT)))
}

/// Read the DOSBox tabs into a [`DosboxConfig`].
fn collect_dosbox(dos: &Dos) -> DosboxConfig {
    DosboxConfig {
        output: cfg_opt(&dos.output),
        machine: cfg_opt(&dos.machine),
        memsize: cfg_opt(&dos.memsize).and_then(|s| s.parse().ok()),
        core: cfg_opt(&dos.core),
        cputype: cfg_opt(&dos.cputype),
        cycles: cfg_opt(&dos.cycles),
        aspect: cfg_bool(&dos.aspect),
        scaler: cfg_opt(&dos.scaler),
        sbtype: cfg_opt(&dos.sbtype),
        rate: cfg_opt(&dos.rate).and_then(|s| s.parse().ok()),
        passthrough: parse_passthrough(&textview_text(&dos.passthrough)),
    }
}

/// Selected dropdown text, treating the `(default)` sentinel as `None`.
fn cfg_opt(dd: &DropDown) -> Option<String> {
    match widgets::dropdown_selected(dd) {
        Some(s) if s != DEFAULT => Some(s),
        _ => None,
    }
}

/// `on`/`off` dropdown -> `Option<bool>` (`(default)` -> `None`).
fn cfg_bool(dd: &DropDown) -> Option<bool> {
    match widgets::dropdown_selected(dd).as_deref() {
        Some("on") => Some(true),
        Some("off") => Some(false),
        _ => None,
    }
}

/// Parse "section.key = value" lines into the passthrough map. Blank lines and
/// `#` comments are ignored; malformed lines are skipped.
fn parse_passthrough(text: &str) -> IndexMap<String, IndexMap<String, String>> {
    let mut map: IndexMap<String, IndexMap<String, String>> = IndexMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((lhs, value)) = line.split_once('=') else {
            continue;
        };
        let Some((section, key)) = lhs.trim().split_once('.') else {
            continue;
        };
        map.entry(section.trim().to_string())
            .or_default()
            .insert(key.trim().to_string(), value.trim().to_string());
    }
    map
}

/// Inverse of [`parse_passthrough`]: render the map back to editable lines.
fn serialize_passthrough(map: &IndexMap<String, IndexMap<String, String>>) -> String {
    let mut out = String::new();
    for (section, keys) in map {
        for (key, value) in keys {
            out.push_str(&format!("{section}.{key} = {value}\n"));
        }
    }
    out
}

/// Whole-buffer text of a `TextView`.
fn textview_text(view: &TextView) -> String {
    let buffer = view.buffer();
    buffer
        .text(&buffer.start_iter(), &buffer.end_iter(), false)
        .to_string()
}

/// Read the editor widgets into a profile, preserving fields no tab edits.
fn collect(fields: &Fields, original: &Profile) -> Profile {
    let mut p = original.clone();

    // General
    let g = &fields.general;
    let title = g.title.text().trim().to_string();
    if !title.is_empty() {
        p.title = title; // title is required; keep the old one if cleared
    }
    p.genre = none_if_empty(&g.genre.text());
    p.year = none_if_empty(&g.year.text()).and_then(|s| s.parse().ok());
    p.developer = none_if_empty(&g.developer.text());
    p.publisher = none_if_empty(&g.publisher.text());
    p.www = none_if_empty(&g.www.text());
    p.notes = none_if_empty(&textview_text(&g.notes));

    // Mounts & Run
    let r = &fields.run;
    p.run = RunSpec {
        working_drive: first_char(&widgets::dropdown_selected(&r.working_drive), 'C'),
        command: r.command.text().trim().to_string(),
        args: split_args(&r.args.text()),
        exit_after: r.exit_after.is_active(),
        mounts: collect_mounts(&r.mounts.borrow()),
    };

    p.dosbox = collect_dosbox(&fields.dos);

    p
}

fn collect_mounts(rows: &[MountRow]) -> Vec<Mount> {
    rows.iter()
        .filter_map(|row| {
            let path = row.path.text().trim().to_string();
            if path.is_empty() {
                return None; // a mount with no path is meaningless; drop it
            }
            Some(Mount {
                drive: first_char(&widgets::dropdown_selected(&row.drive), 'C'),
                kind: text_to_kind(&widgets::dropdown_selected(&row.kind).unwrap_or_default()),
                path: PathBuf::from(path),
                label: none_if_empty(&row.label.text()),
            })
        })
        .collect()
}

fn first_char(text: &Option<String>, fallback: char) -> char {
    text.as_deref()
        .and_then(|s| s.chars().next())
        .unwrap_or(fallback)
}

fn split_args(text: &str) -> Vec<String> {
    text.split_whitespace().map(str::to_string).collect()
}

/// Register temporary `editor-save` / `editor-cancel` app actions that click the
/// editor's buttons, removed when the editor closes. Editors are modal, so only
/// one set is active at a time.
fn register_editor_actions(
    app: &gtk::Application,
    window: &Window,
    save: &Button,
    cancel: &Button,
) {
    let save_action = gtk::gio::SimpleAction::new("editor-save", None);
    {
        let save = save.clone();
        save_action.connect_activate(move |_, _| save.emit_clicked());
    }
    app.add_action(&save_action);

    let cancel_action = gtk::gio::SimpleAction::new("editor-cancel", None);
    {
        let cancel = cancel.clone();
        cancel_action.connect_activate(move |_, _| cancel.emit_clicked());
    }
    app.add_action(&cancel_action);

    let app = app.clone();
    window.connect_close_request(move |_| {
        app.remove_action("editor-save");
        app.remove_action("editor-cancel");
        gtk::glib::Propagation::Proceed
    });
}

/// Show an error in a modal alert tied to the editor window.
fn show_error(window: &Window, message: &str, error: &anyhow::Error) {
    AlertDialog::builder()
        .message(message.to_string())
        .detail(format!("{error:#}"))
        .build()
        .show(Some(window));
}

fn opt(value: &Option<String>) -> &str {
    value.as_deref().unwrap_or("")
}

fn none_if_empty(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passthrough_round_trips() {
        let text = "cpu.cycleup = 500\nrender.glshader = crt\n";
        let map = parse_passthrough(text);
        assert_eq!(map["cpu"]["cycleup"], "500");
        assert_eq!(map["render"]["glshader"], "crt");
        assert_eq!(serialize_passthrough(&map), text);
    }

    #[test]
    fn passthrough_skips_blank_and_malformed_lines() {
        let map = parse_passthrough("\n# comment\nnonsense\ncpu.core = dynamic\n");
        assert_eq!(map.len(), 1);
        assert_eq!(map["cpu"]["core"], "dynamic");
    }
}
