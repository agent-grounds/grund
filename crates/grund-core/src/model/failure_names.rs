//! The names a scan failure lost when the walk kept one line for its directory entry
//! (§AR-scanner.1.entry, §FS-check.2.4.2), for the one question a narrowed run asks of them
//! (§FS-check.1.3.6.3). Carried from the walk the way `walked_dirs` and `unread_roots` are.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Each surviving path's other names, in the order the walk met them. Keyed by the surviving
/// path and never by a resolved one, which would make a link and its target one thing. A name
/// is held under its reason: one path can carry two failures, and a name one of them lost says
/// nothing about the other, whose line and exit stay what they were.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct FailureNames {
    kept: BTreeMap<PathBuf, Vec<(String, PathBuf)>>,
}

impl FailureNames {
    /// Records that the failure at `survivor` for `reason` was also reported as `name`. Roots
    /// that overlap meet one name more than once, and that name is no second one.
    pub(crate) fn keep(&mut self, survivor: &Path, reason: &str, name: &Path) {
        if name == survivor {
            return;
        }
        let kept = self.kept.entry(survivor.to_path_buf()).or_default();
        if !kept.iter().any(|(r, n)| r == reason && n == name) {
            kept.push((reason.to_string(), name.to_path_buf()));
        }
    }

    /// Every name of the failure at `path` for `reason`: the one the whole run prints, then the
    /// others as the walk met them. A failure that lost none has just its own.
    pub(crate) fn of<'a>(
        &'a self,
        path: &'a Path,
        reason: &'a str,
    ) -> impl Iterator<Item = &'a Path> + Clone {
        let kept = self.kept.get(path).map_or(&[][..], Vec::as_slice);
        std::iter::once(path).chain(
            kept.iter()
                .filter(move |(r, _)| r == reason)
                .map(|(_, name)| name.as_path()),
        )
    }
}
