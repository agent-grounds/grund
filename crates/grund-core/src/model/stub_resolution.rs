//! A stub's verdict and the homes of an ID (§AR-scanner.4.6): what the scan found
//! at each stub's target, reached once per scan and recorded on the stub, and the
//! one derivation of an ID's homes from those verdicts. Every command reads both
//! from here and decides nothing about a stub itself
//! (§FS-declarations.checks.broken-stub.4).

use std::cmp::Ordering;

use super::paths::sort_path_key;
use super::records::Declaration;

/// What the scan found at a stub's target, in the order the scanner asks
/// (§AR-scanner.4.6). `Missing`, `NotRead` and `LacksId` are a broken stub
/// (§FS-declarations.checks.broken-stub).
#[derive(Debug, Clone)]
pub(crate) enum StubResolution {
    /// Nothing is at the path the stub links to.
    Missing,
    /// The scan does not read the target: not a file, a name that begins with `.`,
    /// an extension `[scan] extensions` does not list, or a file it cannot read
    /// (§FS-declarations.checks.broken-stub.3). Judged on the name the stub wrote.
    NotRead,
    /// The target was read, as the scan reads it on the text a save would write,
    /// and declares no home of the ID (§FS-declarations.checks.broken-stub.1,
    /// §FS-declarations.checks.broken-stub.2).
    LacksId,
    /// The stub links to the file it sits in, which declares the ID elsewhere. The
    /// stub pairs with nothing there, so it is a home of its own.
    OwnFile,
    /// The target's records of the ID that are not themselves stubs, ascending by
    /// line and never empty: each is a home (§FS-declarations.checks.duplicate.1).
    /// Kept whole, sections and body span included, so a reader answers from the
    /// record whether or not the walk reached the target.
    Homes(Vec<Declaration>),
}

impl StubResolution {
    /// Whether `check` reports the stub broken (§FS-declarations.checks.broken-stub).
    pub(crate) fn is_broken(&self) -> bool {
        matches!(self, Self::Missing | Self::NotRead | Self::LacksId)
    }
}

/// One home of an ID: the record it is, and the declaration of the catalog that
/// stands for it — the record itself, or the stub that points at a target the
/// catalog holds no record of.
#[derive(Clone, Copy)]
pub(crate) struct Home<'a> {
    /// The declaration a body, a section, an anchor or a site is read from.
    pub(crate) record: &'a Declaration,
    /// The declaration in the catalog that stands for the home.
    pub(crate) stand_in: &'a Declaration,
}

impl Home<'_> {
    /// Whether this home is a stub `check` reports broken.
    pub(crate) fn is_broken_stub(&self) -> bool {
        self.record
            .stub_resolution
            .as_ref()
            .is_some_and(StubResolution::is_broken)
    }
}

/// The homes of one ID, in `path:line` order of their records.
pub(crate) struct IdHomes<'a> {
    decls: &'a [Declaration],
    homes: Vec<Home<'a>>,
}

impl<'a> IdHomes<'a> {
    pub(crate) fn len(&self) -> usize {
        self.homes.len()
    }

    /// Every home, in `path:line` order of its record.
    pub(crate) fn iter(&self) -> impl Iterator<Item = Home<'a>> + '_ {
        self.homes.iter().copied()
    }

    /// The one home of an ID that has exactly one: what a link, a body, a section
    /// and a value read. An ID with more than one is a duplicate, reported rather
    /// than ranked, so none of them is its home here.
    pub(crate) fn sole(&self) -> Option<Home<'a>> {
        match self.homes.as_slice() {
            [home] => Some(*home),
            _ => None,
        }
    }

    /// The declarations of the catalog that stand for a home, each once, in the
    /// catalog's order. A stub that stands for none is the pointer to a home.
    pub(crate) fn stand_ins(&self) -> impl Iterator<Item = &'a Declaration> + '_ {
        self.decls.iter().filter(|decl| {
            self.homes
                .iter()
                .any(|home| std::ptr::eq(home.stand_in, *decl))
        })
    }
}

/// The homes of the ID `decls` declares (§AR-scanner.4.6). A declaration that is
/// not a stub is a home of its own. A stub whose verdict is `Homes` stands for each
/// record there that no declaration, and no stub before it in `path:line` order,
/// already stands for, so stubs to one target are one home between them and a
/// target declaring the ID twice is two (§FS-declarations.checks.duplicate.2). Any
/// other stub is a home of its own: a broken one, and one that links to its own
/// file.
pub(crate) fn id_homes(decls: &[Declaration]) -> IdHomes<'_> {
    let mut homes: Vec<Home<'_>> = decls
        .iter()
        .filter(|decl| !decl.is_stub)
        .map(|decl| Home {
            record: decl,
            stand_in: decl,
        })
        .collect();
    let mut stubs: Vec<&Declaration> = decls.iter().filter(|decl| decl.is_stub).collect();
    stubs.sort_by(|a, b| site_order(a, b));
    for stub in stubs {
        let Some(StubResolution::Homes(records)) = &stub.stub_resolution else {
            homes.push(Home {
                record: stub,
                stand_in: stub,
            });
            continue;
        };
        for record in records {
            // The verdict holds clones of the walk's records, or one reading of a
            // target per scan, so one home is one `file` and `line` here.
            if !homes
                .iter()
                .any(|home| home.record.file == record.file && home.record.line == record.line)
            {
                homes.push(Home {
                    record,
                    stand_in: stub,
                });
            }
        }
    }
    homes.sort_by(|a, b| site_order(a.record, b.record));
    IdHomes { decls, homes }
}

fn site_order(a: &Declaration, b: &Declaration) -> Ordering {
    (sort_path_key(&a.file), a.line).cmp(&(sort_path_key(&b.file), b.line))
}
