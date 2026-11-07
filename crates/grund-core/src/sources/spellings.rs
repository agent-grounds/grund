//! The one table of a file's names (§AR-scanner.1.8, §FS-scan.identity.read-once): every in-tree
//! name that reached a physical file, published by the walk and the value scan on the run's
//! sources. It knows no configuration; what a name means is `config`'s (§AR-config.1.6).
//!
//! A tree with no link holds an empty table, and a lookup there is one miss in an empty map, so
//! that tree pays no new `realpath`. A caller with no walk behind it, such as an editor keystroke,
//! resolves the one path it asks about.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::identity::physical_path_key;

/// §AR-scanner.1.8: the run's table, shared by every clone of its `Run`.
#[derive(Clone, Default)]
pub(crate) struct Spellings {
    table: Arc<RwLock<Table>>,
}

#[derive(Default)]
struct Table {
    /// Where each spelling that can wear a second name physically is.
    physical: HashMap<PathBuf, PathBuf>,
    /// Each physical file's names in listing order: the first is `decl.file` and `cite.file`.
    names: BTreeMap<PathBuf, Vec<PathBuf>>,
    /// The physical files read under neither name (§FS-scan.checks.reading-conflict).
    refused: BTreeSet<PathBuf>,
    /// Whether a walk has published, so that a miss means the spelling is the file's own.
    walked: bool,
    /// The config root's physical path, resolved the first time a link asks.
    root: Option<PathBuf>,
}

impl Spellings {
    fn read(&self) -> RwLockReadGuard<'_, Table> {
        self.table
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn write(&self) -> RwLockWriteGuard<'_, Table> {
        self.table
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// One walk's names, physical file by physical file in listing order. A walk of the whole
    /// project replaces what the table held; a narrow walk adds to it (§AR-scanner.1.8).
    pub(crate) fn publish_walk(&self, groups: &[(PathBuf, Vec<PathBuf>)], whole_project: bool) {
        let mut table = self.write();
        if whole_project {
            table.physical.clear();
            table.names.clear();
            table.refused.clear();
        }
        table.walked = true;
        for (physical, names) in groups {
            for name in names {
                table.physical.insert(name.clone(), physical.clone());
            }
            let listed = table.names.entry(physical.clone()).or_default();
            for name in names {
                if !listed.contains(name) {
                    listed.push(name.clone());
                }
            }
        }
    }

    /// The value scan's names of one physical file, listed first because `decl.file` holds the
    /// first of them (§FS-values.2.3).
    pub(crate) fn publish_value_names(&self, physical: &Path, names: &[PathBuf]) {
        let mut table = self.write();
        let listed = table.names.entry(physical.to_path_buf()).or_default();
        let walked = std::mem::take(listed);
        listed.extend_from_slice(names);
        listed.extend(walked.into_iter().filter(|name| !names.contains(name)));
    }

    /// Where `spelling` physically is. After a walk a miss is a file reached by its own name, and
    /// costs nothing; with no walk behind the caller the one path is resolved, and not kept.
    pub(crate) fn physical(&self, spelling: &Path) -> Option<PathBuf> {
        {
            let table = self.read();
            if let Some(physical) = table.physical.get(spelling) {
                return Some(physical.clone());
            }
            if table.walked {
                return None;
            }
        }
        Some(physical_path_key(spelling))
    }

    /// `physical`, for a path the walk may not have met: a value source, a buffer or a stub's
    /// target. Resolved once a run, and kept so the rule reads it as this caller did.
    pub(crate) fn resolve(&self, spelling: &Path) -> PathBuf {
        if let Some(physical) = self.read().physical.get(spelling) {
            return physical.clone();
        }
        let physical = physical_path_key(spelling);
        self.write()
            .physical
            .insert(spelling.to_path_buf(), physical.clone());
        physical
    }

    /// The config root's physical path, resolved once however many links ask.
    pub(crate) fn physical_root(&self, root: &Path) -> PathBuf {
        if let Some(physical) = &self.read().root {
            return physical.clone();
        }
        let physical = physical_path_key(root);
        self.write().root = Some(physical.clone());
        physical
    }

    /// Every name of the file `spelling` reached, in listing order, and empty where the table
    /// knows no other (§FS-scan.identity.findings).
    pub(crate) fn names_of(&self, spelling: &Path) -> Vec<PathBuf> {
        let table = self.read();
        table
            .physical
            .get(spelling)
            .and_then(|physical| table.names.get(physical))
            .filter(|names| names.len() > 1)
            .cloned()
            .unwrap_or_default()
    }

    /// The names of one physical file, as the table lists them.
    pub(crate) fn names_at(&self, physical: &Path) -> Vec<PathBuf> {
        self.read().names.get(physical).cloned().unwrap_or_default()
    }

    /// Whether any file has more than one name, which is what every reader of the names asks
    /// before it does anything else (§AR-scanner.1.8).
    pub(crate) fn has_second_names(&self) -> bool {
        self.read().names.values().any(|names| names.len() > 1)
    }

    /// Mark a file read under neither name (§FS-scan.checks.reading-conflict).
    pub(crate) fn refuse(&self, physical: &Path) {
        self.write().refused.insert(physical.to_path_buf());
    }

    /// Whether the run holds a refusal for the file `spelling` reached (§FS-scan.identity.buffers).
    pub(crate) fn is_refused(&self, spelling: &Path) -> bool {
        let table = self.read();
        !table.refused.is_empty()
            && table
                .physical
                .get(spelling)
                .is_some_and(|physical| table.refused.contains(physical))
    }
}
