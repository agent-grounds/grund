//! The stub targets one run reads outside its walk, each at most once
//! (§FS-check.3.2.1, §AR-resolver.5). The records a target holds are a fact of the
//! tree the walk saw, so they live with the `Catalog` that walk produced: a
//! rescan, such as the next `--watch` run, starts from empty slots and reads every
//! target it needs again, through the input observation of §FS-check.6.1.1.
//!
//! Only the slots are kept here. Filling one is the scanner's own pass over the
//! target, which the resolver runs (§AR-system.4).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use super::records::{Declaration, Id, TextOverlays};

/// What one target file declares, by ID: the records of the scanner's pass over
/// it, stubs left out.
pub(crate) type TargetRecords = BTreeMap<Id, Vec<Declaration>>;

/// One slot per stub target, keyed by physical location. The key set is fixed the
/// first time any slot is asked for, and a slot is filled the first time its own
/// target is, so a target no reader asks about is never read and one many readers
/// ask about is read once. Both are `OnceLock`s rather than a locked map so a
/// filled slot can be borrowed for as long as the `Catalog` that owns it.
#[derive(Default)]
pub(crate) struct StubTargets {
    /// The editor's text the walk read (§FS-lsp.1.1), so a target read after it
    /// reads what a save would write, as the scan does.
    overlays: TextOverlays,
    slots: OnceLock<BTreeMap<PathBuf, OnceLock<TargetRecords>>>,
}

impl StubTargets {
    pub(crate) fn new(overlays: &TextOverlays) -> Self {
        Self {
            overlays: overlays.clone(),
            slots: OnceLock::new(),
        }
    }

    pub(crate) fn overlays(&self) -> &TextOverlays {
        &self.overlays
    }

    /// The records at `key`, filled by `read` on the first ask. `keys` names every
    /// target a slot is kept for, and is called once per run. A key outside it
    /// answers `None`.
    pub(crate) fn records(
        &self,
        key: &Path,
        keys: impl FnOnce() -> Vec<PathBuf>,
        read: impl FnOnce() -> TargetRecords,
    ) -> Option<&TargetRecords> {
        let slots = self.slots.get_or_init(|| {
            keys()
                .into_iter()
                .map(|key| (key, OnceLock::new()))
                .collect()
        });
        slots.get(key).map(|slot| slot.get_or_init(read))
    }

    /// How many targets this run has read: what a test counts reads by.
    #[cfg(test)]
    pub(crate) fn read_count(&self) -> usize {
        self.slots.get().map_or(0, |slots| {
            slots.values().filter(|slot| slot.get().is_some()).count()
        })
    }
}
