//! Folder-home evidence recorded during the existing walk, never a second traversal
//! (§FS-scan.checks.unread-scan-root.folder). The text and value passes discharge a row
//! when they actually read a file in it.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::read_predicate::Unread;
use crate::config::{Extent, Frame, Schema};
use crate::model::{
    Catalog, Refused, RootOrigin, RootRow, normalize_path_lexically, sort_path_key,
};

/// Spelling eligibility and actual reads are separate evidence. Join them only after both
/// passes, through the run's existing identities (§FS-scan.checks.unread-scan-root.folder).
#[derive(Default)]
pub(super) struct FolderReads {
    eligible: Vec<PathBuf>,
    read: Vec<PathBuf>,
}

impl FolderReads {
    /// Discovery admits this spelling, even if the file's one reading is not a value format
    /// (§FS-scan.checks.unread-scan-root.folder). Neither resolution nor discovery proves a read.
    pub(super) fn entered(&mut self, path: &Path, findings: &Catalog) {
        if findings
            .unread_roots
            .iter()
            .any(|row| row.folder_contains(path))
        {
            self.eligible.push(path.to_path_buf());
        }
    }

    /// Record only a successful body read (§FS-scan.checks.unread-scan-root.folder).
    pub(super) fn read(&mut self, path: &Path) {
        self.read.push(path.to_path_buf());
    }

    /// A read through any eligible name satisfies every containing home. Use the walk's
    /// retained names and the value discovery's names, not the content-report name list;
    /// ordinary paths need no new resolution (§AR-scanner.1.8).
    pub(super) fn discharge(
        mut self,
        text_names: Vec<PathBuf>,
        frame: Frame<'_>,
        findings: &mut Catalog,
    ) {
        if findings.unread_roots.is_empty() {
            return;
        }
        self.eligible.extend(text_names);
        // §AR-scanner.1.13: this also includes admitted buffers without a disk entry to walk.
        self.eligible.extend_from_slice(&findings.scanned_files);
        self.read.extend_from_slice(&findings.scanned_files);
        let identity = |path: &PathBuf| {
            frame
                .run
                .spellings
                .physical(path)
                .unwrap_or_else(|| path.clone())
        };
        let read: HashSet<_> = self.read.iter().map(identity).collect();
        self.eligible.retain(|path| read.contains(&identity(path)));
        findings
            .unread_roots
            .retain(|row| !self.eligible.iter().any(|path| row.folder_contains(path)));
    }
}

/// Configured, walked folder homes and the first eligible refusal each walk met
/// (§FS-scan.checks.unread-scan-root.folder).
pub(super) struct FolderHomes(Vec<(PathBuf, RootRow, Option<PathBuf>)>);

impl FolderHomes {
    /// Every authored folder place, including later entries and non-citable rows
    /// (§FS-scan.checks.unread-scan-root.folder).
    pub(super) fn new(schema: &Schema, frame: Frame<'_>) -> Self {
        Self(
            schema
                .rows
                .iter()
                .flat_map(|row| {
                    row.places.iter().filter_map(move |place| {
                        let (Extent::Folder(written), Some(source), true) =
                            (&place.extent, &place.source, place.scanned)
                        else {
                            return None;
                        };
                        let home = normalize_path_lexically(&frame.root().join(written));
                        let record = RootRow {
                            origin: RootOrigin::FolderHome {
                                kind: row.name.clone(),
                                citable: row.kind.is_some(),
                                home: home.clone(),
                                first_refused: PathBuf::new(),
                            },
                            written: written.clone(),
                            config: source.path.clone(),
                            line: source.line,
                            refused: Refused {
                                hidden: false,
                                extension: true,
                            },
                        };
                        Some((home, record, None))
                    })
                })
                .collect(),
        )
    }

    /// Only an extension/language refusal of a non-hidden file counts. Ignore/exclude gates
    /// already kept their files out before the walker hands them here
    /// (§FS-scan.checks.unread-scan-root.folder).
    pub(super) fn refused(&mut self, path: &Path, refusal: Option<Unread>) {
        if refusal != Some(Unread::Extension) {
            return;
        }
        for (home, _, first) in &mut self.0 {
            if path.starts_with(home)
                && first
                    .as_ref()
                    .is_none_or(|old| sort_path_key(path) < sort_path_key(old))
            {
                *first = Some(path.to_path_buf());
            }
        }
    }

    /// A row without eligible refused content is silent, even if the home is empty or absent
    /// (§FS-scan.checks.unread-scan-root.folder).
    pub(super) fn rows(self) -> impl Iterator<Item = RootRow> {
        self.0.into_iter().filter_map(|(_, mut row, first)| {
            let first = first?;
            if let RootOrigin::FolderHome { first_refused, .. } = &mut row.origin {
                *first_refused = first;
            }
            Some(row)
        })
    }
}
