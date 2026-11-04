//! The tally behind a rule folder's `unread-scan-root` verdict
//! (§FS-scan.checks.unread-scan-root.rule-folder, §AR-scanner.1.6). A rule folder is read or
//! not file by file, and more than one root can reach its files, so no root gate can judge it.
//! The walk's own loop hands over what the one predicate answered of each file it yields
//! (§AR-scanner.1.13), each answer is credited to the home that owns the file, and the verdict
//! is taken once, after every root. There is no second listing whose rules could drift.

use std::collections::BTreeSet;
use std::path::Path;

use super::read_predicate::Unread;
use crate::config::{ConfigLocation, Extent, Form, Frame, Owners, Place, Schema};
use crate::model::{
    Refused, RootOrigin, RootRow, configured_home_path_key, scanned_path_key, spelled_extension,
};

/// One scanned folder place of a rule kind, and what the walk has said of its files so far.
/// One per place rather than per kind, so a read file in one entry of `folders` clears that
/// entry and no other.
struct Home<'a> {
    kind: &'a str,
    place: &'a Place,
    written: &'a str,
    source: &'a ConfigLocation,
    /// A walked root is at or above this folder, so the walk saw all of it.
    whole: bool,
    read: bool,
    refused: bool,
    /// Each refused file's extension, once, as its name spells it, in byte order.
    extensions: BTreeSet<String>,
}

impl<'a> Home<'a> {
    /// `place` as a home to keep a tally for: a built-in row has no key to report at.
    fn of(kind: &'a str, place: &'a Place) -> Option<Self> {
        let (Extent::Folder(written), Some(source), true) =
            (&place.extent, &place.source, place.scanned)
        else {
            return None;
        };
        Some(Self {
            kind,
            place,
            written,
            source,
            whole: false,
            read: false,
            refused: false,
            extensions: BTreeSet::new(),
        })
    }

    /// The row a folder the walk read no file of records (§FS-scan.checks.unread-scan-root.location).
    fn into_row(self) -> RootRow {
        RootRow {
            origin: RootOrigin::RuleFolder {
                kind: self.kind.to_string(),
                extensions: self.extensions.into_iter().collect(),
            },
            written: self.written.to_string(),
            config: self.source.path.clone(),
            line: self.source.line,
            refused: Refused {
                hidden: false,
                extension: true,
            },
        }
    }
}

/// What one walk has answered of the files its rule folder homes own (§AR-scanner.1.6).
pub(super) struct RuleFolderTally<'a> {
    owners: Owners<'a>,
    homes: Vec<Home<'a>>,
}

impl<'a> RuleFolderTally<'a> {
    /// The tally for this schema's rule folder homes. `None` where it has none, so such a
    /// project keeps no tally and asks no lookup (§GOAL-fast-feedback).
    pub(super) fn of(schema: &'a Schema, frame: Frame<'_>) -> Option<Self> {
        let rule_rows = schema.rows.iter().filter(|row| {
            row.kind
                .as_ref()
                .is_some_and(|kind| kind.form == Form::Rule)
        });
        let homes: Vec<Home<'a>> = rule_rows
            .flat_map(|row| {
                let kind = row.name.as_str();
                row.places
                    .iter()
                    .filter_map(move |place| Home::of(kind, place))
            })
            .collect();
        (!homes.is_empty()).then(|| Self {
            owners: Owners::new(schema, frame),
            homes,
        })
    }

    /// The walk is starting at the directory `root`. An editor scoped to a path in a project
    /// that declares `[workspace]` walks that path alone (§AR-lsp.5.1.1), and a walk of part of
    /// a folder can answer for its files but not for the folder, so only a home some walked
    /// root is at or above gets a verdict.
    pub(super) fn walking(&mut self, root: &Path) {
        let Some(root) = self.owners.relative(root) else {
            return;
        };
        let root = scanned_path_key(&root);
        for home in &mut self.homes {
            home.whole |= configured_home_path_key(home.written).starts_with(&root);
        }
    }

    /// Credit `answer`, the predicate's answer about `file`, to the home that owns the file,
    /// whichever root reached it (§FS-config-v2.schema.places.ownership). A file a nested home
    /// owns counts for that home, and a hidden name counts as neither a read nor an extension
    /// refusal (§AR-scanner.1.13).
    pub(super) fn count(&mut self, file: &Path, answer: Option<Unread>) {
        if answer == Some(Unread::HiddenName) {
            return;
        }
        let Some(owner) = self.owners.of(file) else {
            return;
        };
        // The very place `Schema::owner` chose, so a folder written twice has one answer.
        let home = self
            .homes
            .iter_mut()
            .find(|home| std::ptr::eq(home.place, owner.place));
        let Some(home) = home else {
            return;
        };
        if answer.is_none() {
            home.read = true;
            return;
        }
        home.refused = true;
        if let Some(extension) = spelled_extension(file) {
            home.extensions.insert(extension.to_string());
        }
    }

    /// One row per folder the run read no file of and refused at least one of by its extension
    /// (§FS-scan.checks.unread-scan-root.rule-folder): asked once, after every root.
    pub(super) fn unread(self) -> impl Iterator<Item = RootRow> + 'a {
        self.homes
            .into_iter()
            .filter(|home| home.whole && home.refused && !home.read)
            .map(Home::into_row)
    }
}
