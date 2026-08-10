//! Left category sidebar: filter the library by genre / developer / year /
//! favorites, D-Fend style. Categories are derived from the current profiles and
//! rebuilt on reload. The active choice is shared via an `Rc<RefCell<Category>>`
//! that the grid's filter reads.
//!
//! Over the 150-line soft cap by design: the [`Category`] model and the widget
//! that renders it share the case-insensitivity invariant documented on
//! [`same`], and would be easy to break apart.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{Label, ListBox, ListBoxRow, ScrolledWindow, SelectionMode};

use crate::config::profile::Profile;

/// A profile filter selected in the sidebar.
#[derive(Clone, PartialEq)]
pub enum Category {
    All,
    Favorites,
    Genre(String),
    Developer(String),
    Year(u16),
    /// A non-selectable section header (no filtering effect).
    Header,
}

impl Category {
    /// Does `profile` belong to this category?
    pub fn matches(&self, profile: &Profile) -> bool {
        match self {
            Category::All | Category::Header => true,
            Category::Favorites => profile.favorite,
            Category::Genre(g) => profile.genre.as_deref().is_some_and(|v| same(v, g)),
            Category::Developer(d) => profile.developer.as_deref().is_some_and(|v| same(v, d)),
            Category::Year(y) => profile.year == Some(*y),
        }
    }

    fn label(&self) -> String {
        match self {
            Category::All => "All games".to_string(),
            Category::Favorites => "★ Favorites".to_string(),
            Category::Genre(g) => g.clone(),
            Category::Developer(d) => d.clone(),
            Category::Year(y) => y.to_string(),
            Category::Header => String::new(),
        }
    }
}

/// The sidebar widget plus the shared active-category state.
pub struct CategorySidebar {
    pub scroller: ScrolledWindow,
    list: ListBox,
    active: Rc<RefCell<Category>>,
    /// Parallel to the list rows (header rows included) so a row index maps back.
    cats: Rc<RefCell<Vec<Category>>>,
}

impl CategorySidebar {
    /// Build the sidebar. `active` is shared with the grid filter; `on_change`
    /// runs whenever the selected category changes (e.g. to re-run the filter).
    pub fn new(active: Rc<RefCell<Category>>, on_change: Rc<dyn Fn()>) -> CategorySidebar {
        let list = ListBox::new();
        list.set_selection_mode(SelectionMode::Single);
        let cats: Rc<RefCell<Vec<Category>>> = Rc::new(RefCell::new(Vec::new()));

        {
            let active = active.clone();
            let cats = cats.clone();
            list.connect_row_selected(move |_, row| {
                let Some(row) = row else { return };
                if let Some(cat) = cats.borrow().get(row.index() as usize) {
                    if *cat != Category::Header {
                        *active.borrow_mut() = cat.clone();
                        on_change();
                    }
                }
            });
        }

        let scroller = ScrolledWindow::builder()
            .child(&list)
            .width_request(200)
            .build();
        CategorySidebar {
            scroller,
            list,
            active,
            cats,
        }
    }

    /// Rebuild the category rows from the current profiles, keeping the active
    /// category selected when it survived the rebuild (falling back to "All").
    ///
    /// Every reload rebuilds the sidebar — saving an edit, toggling a favourite,
    /// importing — so resetting to "All" here would silently drop the user's
    /// filter on each of those.
    pub fn rebuild(&self, profiles: &[(PathBuf, Profile)]) {
        let previous = self.active.borrow().clone();
        while let Some(row) = self.list.row_at_index(0) {
            self.list.remove(&row);
        }
        let mut cats = Vec::new();

        self.push(&mut cats, Category::All);
        self.push(&mut cats, Category::Favorites);

        let genres = distinct(profiles, |p| p.genre.clone());
        if !genres.is_empty() {
            self.push_header(&mut cats, "Genres");
            for g in genres {
                self.push(&mut cats, Category::Genre(g));
            }
        }
        let developers = distinct(profiles, |p| p.developer.clone());
        if !developers.is_empty() {
            self.push_header(&mut cats, "Developers");
            for d in developers {
                self.push(&mut cats, Category::Developer(d));
            }
        }
        let mut years: Vec<u16> = profiles.iter().filter_map(|(_, p)| p.year).collect();
        years.sort_unstable();
        years.dedup();
        if !years.is_empty() {
            self.push_header(&mut cats, "Years");
            for y in years {
                self.push(&mut cats, Category::Year(y));
            }
        }

        // Index 0 is always `All`, so an unknown previous category lands there.
        let index = cats.iter().position(|c| *c == previous).unwrap_or(0);
        let resolved = cats.get(index).cloned().unwrap_or(Category::All);
        *self.cats.borrow_mut() = cats;
        *self.active.borrow_mut() = resolved;
        // Selecting re-fires `row-selected`, which re-applies the filter. Both
        // RefCells are unborrowed here so that handler can take them.
        if let Some(row) = self.list.row_at_index(index as i32) {
            self.list.select_row(Some(&row));
        }
    }

    /// Append one selectable category row.
    fn push(&self, cats: &mut Vec<Category>, cat: Category) {
        let widget = Label::builder()
            .label(cat.label())
            .halign(gtk::Align::Start)
            .margin_top(4)
            .margin_bottom(4)
            .margin_start(16)
            .margin_end(10)
            .build();
        let row = ListBoxRow::builder().child(&widget).build();
        self.list.append(&row);
        cats.push(cat);
    }

    /// Append a non-selectable section header row.
    fn push_header(&self, cats: &mut Vec<Category>, name: &str) {
        let widget = Label::builder()
            .label(name)
            .halign(gtk::Align::Start)
            .margin_top(8)
            .margin_bottom(2)
            .margin_start(10)
            .margin_end(10)
            .css_classes(["dim-label", "heading"])
            .build();
        let row = ListBoxRow::builder()
            .child(&widget)
            .selectable(false)
            .build();
        self.list.append(&row);
        cats.push(Category::Header);
    }
}

/// Case-insensitive equality — the single rule for both grouping rows and
/// matching profiles to them.
///
/// Invariant: [`distinct`] and [`Category::matches`] must agree. Grouping
/// "Westwood"/"westwood" into one row while matching them exactly would make
/// that row hide half the profiles that produced it.
fn same(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// Distinct, case-insensitively sorted values produced by `field`. The first
/// spelling encountered wins; [`same`] decides what counts as a duplicate.
fn distinct(
    profiles: &[(PathBuf, Profile)],
    field: impl Fn(&Profile) -> Option<String>,
) -> Vec<String> {
    let mut values: Vec<String> = profiles.iter().filter_map(|(_, p)| field(p)).collect();
    values.sort_by_key(|s| s.to_lowercase());
    values.dedup_by(|a, b| same(a, b));
    values
}
