use std::path::{Path, PathBuf};

use crate::config::{Extent, Frame, Owners, Schema};
use crate::sources::physical_path_key;

pub(super) struct DeclarationHome<'a> {
    pub(super) kind: &'a str,
    /// The owning place as written in the config, a folder without its `/`.
    pub(super) path: &'a str,
    /// Whether the home's kind declares IDs (§FS-config.3.4). A non-citable home
    /// admits no declaration at all, so the finding it produces names the home
    /// rather than a kind the author was supposed to have written.
    pub(super) citable: bool,
    /// Whether the home is one exact `file` rather than a `folder`, so the label
    /// below reads as the directory it is.
    pub(super) exact: bool,
}

impl DeclarationHome<'_> {
    /// How a non-citable home is named in a finding (§FS-declarations.checks.misplaced-declaration.3,
    /// §FS-check.3.6) — the same `<folder>/` label the citation-direction
    /// findings and the generated block use, so one home reads one way
    /// everywhere.
    pub(super) fn place(&self) -> String {
        if self.exact {
            self.path.to_string()
        } else {
            format!("{}/", self.path)
        }
    }
}

/// A single-file kind: a citable row whose places are all files
/// (§FS-declarations.checks.misplaced-declaration.1). v1 has one; a v2 row may
/// list several, and is single-file across them.
pub(super) struct SingleFileHome<'a> {
    kind: &'a str,
    /// Each file as written, with its physical key, in written order.
    files: Vec<(&'a str, PathBuf)>,
}

impl SingleFileHome<'_> {
    /// The first file, as written and as its physical key: the built-in home a
    /// zero-config `FS` has (§FS-check.checks.fs-home-moves).
    pub(super) fn first(&self) -> (&str, &Path) {
        let (path, physical) = &self.files[0];
        (path, physical)
    }

    /// Whether `file` is one of the kind's files, compared by physical location.
    pub(super) fn holds(&self, file: &Path) -> bool {
        let key = physical_path_key(file);
        self.files.iter().any(|(_, physical)| *physical == key)
    }

    /// The files, comma-separated in written order, as a finding names them.
    pub(super) fn label(&self) -> String {
        let files: Vec<&str> = self.files.iter().map(|(path, _)| *path).collect();
        files.join(", ")
    }
}

/// The checker's reading of where a declaration may sit: the single-file kinds,
/// and the owner of every file by the one ownership rule
/// (§FS-config-v2.schema.places.ownership, §AR-scanner.4.7).
pub(super) struct KindHomeIndex<'a> {
    owners: Owners<'a>,
    single_files: Vec<SingleFileHome<'a>>,
}

impl<'a> KindHomeIndex<'a> {
    pub(super) fn new(schema: &'a Schema, frame: Frame<'_>) -> Self {
        // §FS-declarations.checks.misplaced-declaration: only a citable kind has
        // declarations to keep in one document, so the single-file rule says nothing
        // about a non-citable `file` home — the home-kind rule reports what is there.
        let single_files = schema
            .rows
            .iter()
            .filter(|row| row.kind.is_some() && row.folders().next().is_none())
            .filter_map(|row| {
                let files: Vec<(&str, PathBuf)> = row
                    .files()
                    .map(|file| (file, physical_path_key(&frame.root().join(file))))
                    .collect();
                (!files.is_empty()).then_some(SingleFileHome {
                    kind: row.name.as_str(),
                    files,
                })
            })
            .collect();
        Self {
            owners: Owners::new(schema, frame),
            single_files,
        }
    }

    /// The single-file home of `kind`, if it has one — the documents every
    /// declaration of that kind must live in (§FS-config.3.4). `None` for a kind
    /// with a folder.
    pub(super) fn single_file_for_kind(&self, kind: &str) -> Option<&SingleFileHome<'a>> {
        self.single_files.iter().find(|home| home.kind == kind)
    }

    /// The place that owns `path`, other than the complement's, by the one
    /// ownership rule: v1's unique home, v2's listed file or deepest folder.
    /// Ownership is decided on the scanner-recorded path, not the symlink target
    /// (§FS-config.3.4.11, §FS-declarations.checks.misplaced-declaration.2).
    pub(super) fn unique_decl_home_for_file(&self, path: &Path) -> Option<DeclarationHome<'a>> {
        let owner = self.owners.of(path)?;
        Some(DeclarationHome {
            kind: owner.row.name.as_str(),
            path: owner.place.path()?,
            citable: owner.row.kind.is_some(),
            exact: matches!(owner.place.extent, Extent::File(_)),
        })
    }
}

/// The one location test written against a key that has already been taken
/// (§FS-declarations.checks.misplaced-declaration.1): the single-file rule holds one `physical_path` per kind home
/// and compares every declaration's file to it, so re-deriving the right-hand
/// side per declaration would canonicalize the same home once per ID.
pub(super) fn paths_same_location_key(left: &Path, right: &Path) -> bool {
    physical_path_key(left) == right
}
