//! The tally behind a rule folder's `unread-scan-root` verdict
//! (§FS-scan.checks.unread-scan-root.rule-folder, §AR-scanner.1.6). A rule folder is read or
//! not file by file, and more than one root can reach its files, so no root gate can judge it.
//! The walk's own loop hands over what the one predicate answered of each file it yields
//! (§AR-scanner.1.13), each answer is credited to the homes it counts for, and the verdict is
//! taken once, after every root. That is not ownership: a file version 1 leaves to the complement
//! for citing is read, and its rule is in force. There is no second listing whose rules could
//! drift. A file under no home costs a comparison, because every key is worked out once and the
//! early return in front of the lookup joins nothing.

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use super::read_predicate::Unread;
use crate::config::{ConfigLocation, Extent, Form, Frame, KeyedPlace, Owners, Place, Row, Schema};
use crate::model::{Refused, RootOrigin, RootRow, spelled_extension};

/// What a rule folder home reports with, `None` for any other place: a built-in row has no key
/// to report at.
fn home_parts<'a>(row: &Row, place: &'a Place) -> Option<(&'a str, &'a ConfigLocation)> {
    let rule = row
        .kind
        .as_ref()
        .is_some_and(|kind| kind.form == Form::Rule);
    match (&place.extent, &place.source, place.scanned, rule) {
        (Extent::Folder(written), Some(source), true, true) => Some((written, source)),
        _ => None,
    }
}

/// One scanned folder place of a rule kind, and what the walk has said of its files so far.
/// One per place rather than per kind, so a read file in one entry of `folders` clears that
/// entry and no other.
struct Home<'a> {
    /// Where this home's own place is among the tally's `places`.
    at: usize,
    kind: &'a str,
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

/// What one walk has answered of the files that count for its rule folder homes (§AR-scanner.1.6).
pub(super) struct RuleFolderTally<'a> {
    owners: Owners<'a>,
    /// Every place of the schema, because a deeper place of any row can take a file from a home.
    places: Vec<KeyedPlace<'a>>,
    homes: Vec<Home<'a>>,
    /// What a plain relative path starts with under each home that can hold one.
    plain_prefixes: Vec<PathBuf>,
}

impl<'a> RuleFolderTally<'a> {
    /// The tally for this schema's rule folder homes. `None` where it has none, so such a
    /// project resolves no root, keys no place and asks no lookup (§GOAL-fast-feedback).
    pub(super) fn of(schema: &'a Schema, frame: Frame<'_>) -> Option<Self> {
        let has_home = schema.rows.iter().any(|row| {
            row.places
                .iter()
                .any(|place| home_parts(row, place).is_some())
        });
        if !has_home {
            return None;
        }
        let owners = Owners::new(schema, frame);
        let places = owners.keyed_places();
        let homes: Vec<Home<'a>> = places
            .iter()
            .enumerate()
            .filter_map(|(at, keyed)| {
                let (written, source) = home_parts(keyed.row, keyed.place)?;
                Some(Home {
                    at,
                    kind: keyed.row.name.as_str(),
                    written,
                    source,
                    whole: false,
                    read: false,
                    refused: false,
                    extensions: BTreeSet::new(),
                })
            })
            .collect();
        let plain_prefixes = homes
            .iter()
            .filter_map(|home| owners.plain_prefix(&places[home.at].key))
            .collect();
        Some(Self {
            owners,
            places,
            homes,
            plain_prefixes,
        })
    }

    /// The walk is starting at the directory `root`. An editor scoped to a path in a project
    /// that declares `[workspace]` walks that path alone (§AR-lsp.5.1.1), and a walk of part of
    /// a folder can answer for its files but not for the folder, so only a home some walked
    /// root is at or above gets a verdict.
    pub(super) fn walking(&mut self, root: &Path) {
        let root = self.owners.key(root);
        for home in &mut self.homes {
            home.whole |= self.places[home.at].key.starts_with(&root);
        }
    }

    /// Whether some home could hold `file`: false only for a file under a plain relative path
    /// that starts with no home's prefix (§AR-scanner.1.6). This is only a necessary condition,
    /// and [`KeyedPlace::holds`] decides whatever it lets through.
    fn may_hold(&self, file: &Path) -> bool {
        let Some(relative) = self.owners.relative(file) else {
            return true;
        };
        self.plain_prefixes
            .iter()
            .any(|prefix| relative.starts_with(prefix))
            || !relative
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
    }

    /// Credit `answer`, the predicate's answer about `file`, to the rule folder homes it counts
    /// for, whichever root reached it (§FS-scan.checks.unread-scan-root.rule-folder). No answer
    /// is a read and an extension is a refusal; a hidden name, or any other reason, counts as
    /// neither (§AR-scanner.1.13). A refusal stays with the deepest holder, and a read reaches
    /// every folder of that holder's row that holds the file.
    pub(super) fn count(&mut self, file: &Path, answer: Option<Unread>) {
        let read = match answer {
            None => true,
            Some(Unread::Extension) => false,
            Some(_) => return,
        };
        if !self.may_hold(file) {
            return;
        }
        let key = self.owners.key(file);
        let deepest = KeyedPlace::deepest_holders(&self.places, &key);
        for home in &mut self.homes {
            let own = &self.places[home.at];
            if read {
                home.read |= own.holds(&key)
                    && deepest
                        .iter()
                        .any(|holder| std::ptr::eq(holder.row, own.row));
                continue;
            }
            if deepest
                .iter()
                .any(|holder| std::ptr::eq(holder.place, own.place))
            {
                home.refused = true;
                if let Some(extension) = spelled_extension(file) {
                    home.extensions.insert(extension.to_string());
                }
            }
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
