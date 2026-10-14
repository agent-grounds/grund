//! Which row owns a file, asked once for every per-file rule
//! (§FS-config-v2.schema.places.ownership, §AR-config.3.2): the scanner's
//! citing-side classification, the checker's misplaced-declaration, grounding
//! and index passes, and the stub test all read [`Schema::owner`] rather than
//! matching paths against homes themselves. v1 keeps its rule — a file in two
//! nested homes falls to the complement (§FS-config.3.9.2) — and v2 gives it to
//! the listed file's row, else the deepest folder's.
//!
//! A row's places, every one of them, are read here too, beside the first-place
//! helpers in `rows.rs` that the v1 views still ask for.

use std::path::{Path, PathBuf};

use super::frame::Frame;
use super::kind::{DEFAULT_KIND_INDEX, KindIndex};
use super::project::Schema;
use super::rows::{Extent, Nesting, Place, Row};
use crate::model::{configured_home_path_key, scanned_decl_relative_path, scanned_path_key};
use crate::sources::physical_path_key;

/// The row a file belongs to and the place of it that holds the file
/// (§FS-config-v2.schema.places.ownership).
#[derive(Clone, Copy, Debug)]
pub struct Owner<'a> {
    pub row: &'a Row,
    pub place: &'a Place,
}

/// [`Schema::owner`] asked of a scanned path, under whichever spelling of the
/// config root reaches it (§FS-declarations.checks.misplaced-declaration): the
/// roots are resolved once, so a pass asks per file without resolving them again.
pub(crate) struct Owners<'a> {
    schema: &'a Schema,
    configured_root: PathBuf,
    physical_root: PathBuf,
}

impl<'a> Owners<'a> {
    pub(crate) fn new(schema: &'a Schema, frame: Frame<'_>) -> Self {
        Self {
            schema,
            configured_root: scanned_path_key(frame.root()),
            physical_root: physical_path_key(frame.root()),
        }
    }

    /// The schema whose places these are.
    pub(crate) fn schema(&self) -> &'a Schema {
        self.schema
    }

    /// `path` relative to the config root, `None` for a file outside the project.
    pub(crate) fn relative<'p>(&self, path: &'p Path) -> Option<std::borrow::Cow<'p, Path>> {
        scanned_decl_relative_path(path, &self.configured_root, &self.physical_root)
    }

    /// The owner of the scanned file at `path`; `None` for the complement.
    pub(crate) fn of(&self, path: &Path) -> Option<Owner<'a>> {
        self.schema.owner(self.relative(path)?.as_ref())
    }

    /// Whether a place other than the complement's holds `path`.
    pub(crate) fn in_some_home(&self, path: &Path) -> bool {
        self.relative(path)
            .is_some_and(|relative| self.schema.in_some_home(&relative))
    }
}

impl Owner<'_> {
    /// The owning place, named the way a finding names a place (§FS-check.3.11).
    pub fn label(&self) -> String {
        self.place.label().unwrap_or_default()
    }
}

impl Place {
    /// The path this place names, `None` for the complement's root extent.
    pub fn path(&self) -> Option<&str> {
        match &self.extent {
            Extent::Folder(path) | Extent::File(path) => Some(path),
            Extent::Complement => None,
        }
    }

    /// How this place is named in a finding: `<folder>/`, or the file
    /// (§FS-check.3.11, §FS-config-v2.schema.places.ownership).
    pub fn label(&self) -> Option<String> {
        match &self.extent {
            Extent::Folder(path) => Some(format!("{path}/")),
            Extent::File(path) => Some(path.clone()),
            Extent::Complement => None,
        }
    }

    /// Whether this place holds `relative`, a config-root-relative path under the
    /// scanner's lexical key: a file place by equality, a folder by prefix, on the
    /// path as written rather than its link target (§FS-config.3.4.11).
    fn holds(&self, relative: &Path) -> bool {
        match &self.extent {
            Extent::File(file) => relative == configured_home_path_key(file),
            Extent::Folder(folder) => relative.starts_with(configured_home_path_key(folder)),
            Extent::Complement => false,
        }
    }

    /// How deep the place sits: the deepest folder wins (§FS-config-v2.schema.places.ownership).
    fn depth(&self) -> usize {
        self.path().map_or(0, |path| {
            configured_home_path_key(path).components().count()
        })
    }
}

impl Row {
    /// Every folder this row lists, in written order (§FS-config-v2.schema.places).
    pub fn folders(&self) -> impl Iterator<Item = &str> {
        self.places.iter().filter_map(|place| match &place.extent {
            Extent::Folder(path) => Some(path.as_str()),
            _ => None,
        })
    }

    /// Every file this row lists, in written order (§FS-config-v2.schema.places).
    pub fn files(&self) -> impl Iterator<Item = &str> {
        self.places.iter().filter_map(|place| match &place.extent {
            Extent::File(path) => Some(path.as_str()),
            _ => None,
        })
    }

    /// Every place of this row named as a finding names one, comma-separated in
    /// place order (§FS-schema.4); `None` for a row with no place to look at.
    pub fn place_labels(&self) -> Option<String> {
        let labels: Vec<String> = self.places.iter().filter_map(Place::label).collect();
        (!labels.is_empty()).then(|| labels.join(", "))
    }

    /// The index each folder of a citable row keeps, as `(folder, index)` relative
    /// to the config root: one per folder (§FS-config-v2.schema.places.ownership),
    /// none for a row with no kind or `index = false` (§FS-config.3.4).
    pub fn index_paths(&self) -> Vec<(&str, PathBuf)> {
        let Some(kind) = &self.kind else {
            return Vec::new();
        };
        let name = match &kind.index {
            KindIndex::Disabled => return Vec::new(),
            KindIndex::Default => DEFAULT_KIND_INDEX,
            KindIndex::Named(name) => name.as_str(),
        };
        self.folders()
            .map(|folder| (folder, Path::new(folder).join(name)))
            .collect()
    }
}

impl Schema {
    /// The row that owns `relative`, a config-root-relative path under the
    /// scanner's lexical key, and its place that holds it; `None` when the
    /// complement owns it (§FS-config.3.9.2, §FS-config-v2.schema.places.ownership).
    ///
    /// v1 (§AR-config.3.2 `NestedPlaceFallsToComplement`): the one row whose home
    /// holds the file, and the complement when none or several do. v2
    /// (`DeepestPlaceWins`): the row that lists the file, else the row of the
    /// deepest folder holding it, and the complement when that row is `code`.
    pub fn owner(&self, relative: &Path) -> Option<Owner<'_>> {
        let mut holders = self.holders(relative);
        match self.nesting {
            Nesting::NestedPlaceFallsToComplement => {
                let first = holders.next()?;
                holders
                    .all(|other| other.row.name == first.row.name)
                    .then_some(first)
            }
            Nesting::DeepestPlaceWins => {
                let holders: Vec<Owner<'_>> = holders.collect();
                let listed = holders
                    .iter()
                    .find(|owner| matches!(owner.place.extent, Extent::File(_)));
                let owner = listed
                    .or_else(|| holders.iter().max_by_key(|owner| owner.place.depth()))
                    .copied()?;
                (!owner.row.is_complement()).then_some(owner)
            }
        }
    }

    /// Whether the folder place `folder` owns `relative` for its index
    /// (§FS-check.3.18.2): v1 counts the folder's whole subtree, and v2 leaves a
    /// file to the nested place that owns it (§FS-config-v2.schema.places.ownership).
    pub fn folder_owns(&self, folder: &str, relative: &Path) -> bool {
        if !relative.starts_with(configured_home_path_key(folder)) {
            return false;
        }
        match self.nesting {
            Nesting::NestedPlaceFallsToComplement => true,
            Nesting::DeepestPlaceWins => self.owner(relative).is_some_and(
                |owner| matches!(&owner.place.extent, Extent::Folder(owned) if owned == folder),
            ),
        }
    }

    /// Whether any place but the complement's holds `relative`: the home test of a
    /// stub (§FS-declarations.stubs.shape), read off the same places as
    /// [`Schema::owner`] (§AR-scanner.4.7).
    pub fn in_some_home(&self, relative: &Path) -> bool {
        self.holders(relative)
            .any(|owner| !owner.row.is_complement())
    }

    /// Whether a run without `--full` reads `relative` by its owner: in v2 a file
    /// is unwalked only when the place that owns it is `scan = false`, so a scanned
    /// place under an unscanned one is read (§FS-config-v2.schema.places.boundaries.6).
    pub fn owner_walked(&self, relative: &Path) -> bool {
        self.owner(relative).is_none_or(|owner| owner.place.scanned)
    }

    /// Every place that holds `relative`, in row order, the complement's own
    /// places included.
    fn holders<'a, 'p>(&'a self, relative: &'p Path) -> impl Iterator<Item = Owner<'a>> + 'p
    where
        'a: 'p,
    {
        self.rows.iter().flat_map(move |row| {
            row.places
                .iter()
                .filter(move |place| place.holds(relative))
                .map(move |place| Owner { row, place })
        })
    }
}
