//! The index carve-out's one entry per owed ID (§FS-fmt.6.1). Which citation of
//! an owed ID the link pass may write where the ordinary pass is off depends on
//! the whole file, a link the ordinary pass writes later on the page included, so
//! a first walk of the file only surveys and a second writes what it decided.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use crate::model::Id;

/// A citation's place in its file: the line, and the marker's byte offset in that
/// line as the link pass reads it, which both walks read alike.
pub(super) type Site = (usize, usize);

/// The IDs a kind's index owes an entry for (§FS-check.3.18), and which of their
/// citations is each one's entry.
pub(crate) struct IndexCarveOut<'a> {
    owed: &'a BTreeSet<Id>,
    plan: RefCell<Plan>,
}

enum Plan {
    /// The first walk: each owed ID's first link in the file, and whether it lies
    /// where the carve-out writes, and its first citation there the pass can wrap.
    Survey {
        first_link: BTreeMap<Id, (Site, bool)>,
        first_bare: BTreeMap<Id, Site>,
    },
    /// The second walk: the one site per owed ID the carve-out writes.
    Entries(BTreeSet<Site>),
}

/// How the link pass meets the carve-out on one line of an index.
#[derive(Clone, Copy)]
pub(crate) enum CarveOutPass<'a> {
    /// The ordinary pass runs on this line; the carve-out only notes the links
    /// of owed IDs it finds or writes.
    Watch(&'a IndexCarveOut<'a>),
    /// The ordinary pass is off here, so only an owed ID's entry is written.
    Entries(&'a IndexCarveOut<'a>),
}

impl CarveOutPass<'_> {
    pub(super) fn carve_out(&self) -> &IndexCarveOut<'_> {
        match self {
            Self::Watch(carve_out) | Self::Entries(carve_out) => carve_out,
        }
    }
}

impl<'a> IndexCarveOut<'a> {
    /// A carve-out ready for its surveying walk.
    pub(crate) fn survey(owed: &'a BTreeSet<Id>) -> Self {
        Self {
            owed,
            plan: RefCell::new(Plan::Survey {
                first_link: BTreeMap::new(),
                first_bare: BTreeMap::new(),
            }),
        }
    }

    pub(super) fn owes(&self, id: &Id) -> bool {
        self.owed.contains(id)
    }

    pub(super) fn surveying(&self) -> bool {
        matches!(*self.plan.borrow(), Plan::Survey { .. })
    }

    /// A link of `id` at `site`: a wrap, a link its author wrote, or one the
    /// ordinary pass writes. `in_scope` says whether the carve-out writes there.
    pub(super) fn saw_link(&self, id: &Id, site: Site, in_scope: bool) {
        if let Plan::Survey { first_link, .. } = &mut *self.plan.borrow_mut() {
            first_link.entry(id.clone()).or_insert((site, in_scope));
        }
    }

    /// A bare citation of `id` at `site`, in the scope, that the pass could wrap.
    pub(super) fn saw_bare(&self, id: &Id, site: Site) {
        if let Plan::Survey { first_bare, .. } = &mut *self.plan.borrow_mut() {
            first_bare.entry(id.clone()).or_insert(site);
        }
    }

    /// Settle each owed ID's entry from the survey (§FS-fmt.6.1): its first link
    /// when that lies in the scope, nothing when it lies outside, and otherwise
    /// its first citation in the scope the pass can wrap.
    pub(crate) fn decide(&self) {
        let mut plan = self.plan.borrow_mut();
        let Plan::Survey {
            first_link,
            first_bare,
        } = &*plan
        else {
            return;
        };
        let entries = self
            .owed
            .iter()
            .filter_map(|id| match first_link.get(id) {
                Some((site, in_scope)) => in_scope.then_some(*site),
                None => first_bare.get(id).copied(),
            })
            .collect();
        *plan = Plan::Entries(entries);
    }

    /// Whether the citation at `site` is the one the carve-out writes.
    pub(super) fn is_entry(&self, site: Site) -> bool {
        matches!(&*self.plan.borrow(), Plan::Entries(entries) if entries.contains(&site))
    }
}
