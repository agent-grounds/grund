//! Which files are configured indexes, the membership test `fmt` and `refs` ask without the
//! kind-index pass (§FS-config.3.4, §AR-checker.2.16).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::index::kind_index_targets;
use super::index_entries::IndexNames;
use crate::config::{Frame, Schema};
use crate::model::{scanned_decl_relative_path, scanned_path_key};
use crate::sources::physical_path_key;

/// The configured index files, as a path-membership test (§FS-config.3.4).
/// `grund fmt` reads it to keep the cross-reference pass running over an index
/// whatever `[fmt.cross_refs] enabled` says (§FS-fmt.6.1.1,
/// §DF-index-always-linkified) — the one region the formatter always writes,
/// mirroring §FS-fmt.2.3's regions it never writes.
pub(crate) struct KindIndexFiles {
    configured_root: PathBuf,
    physical_root: PathBuf,
    keys: BTreeSet<PathBuf>,
}

impl KindIndexFiles {
    pub(crate) fn new(schema: &Schema, frame: Frame<'_>) -> Self {
        let configured_root = scanned_path_key(frame.root());
        let physical_root = physical_path_key(frame.root());
        let targets = kind_index_targets(schema, frame);
        // §AR-scanner.1.8: `fmt` meets an index under the name listed first,
        // and a finding that names the rewrite holds whichever name that is.
        let names = IndexNames::new(&targets, frame, &configured_root, &physical_root);
        let others = names.relative_names();
        Self {
            keys: targets
                .iter()
                .map(|target| target.index_key.clone())
                .chain(others)
                .collect(),
            configured_root,
            physical_root,
        }
    }

    /// The set a run that already linkifies everything needs — no config read,
    /// no allocation, and `contains` short-circuits on it.
    pub(crate) fn empty() -> Self {
        Self {
            configured_root: PathBuf::new(),
            physical_root: PathBuf::new(),
            keys: BTreeSet::new(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub(crate) fn contains(&self, path: &Path) -> bool {
        if self.keys.is_empty() {
            return false;
        }
        scanned_decl_relative_path(path, &self.configured_root, &self.physical_root)
            .is_some_and(|relative| self.keys.contains(relative.as_ref()))
    }
}
