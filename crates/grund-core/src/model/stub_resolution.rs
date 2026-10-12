//! A stub's verdict and the declarations of an ID once its stubs are paired
//! (§AR-scanner.4.6): what the scan found at each stub's target, reached once per
//! scan and recorded on the stub, and the one pairing of an ID's declarations from
//! those verdicts (§FS-declarations.stubs.verdict). Every command reads both from
//! here and decides nothing about a stub itself.

use std::cmp::Ordering;

use super::paths::sort_path_key;
use super::records::{Catalog, Declaration, Id};

/// What the scan found at a stub's target, in the order the scanner asks
/// (§AR-scanner.4.6). `Missing`, `NotRead` and `LacksId` are a broken stub
/// (§FS-declarations.checks.broken-stub).
#[derive(Debug, Clone)]
pub(crate) enum StubResolution {
    /// Nothing is at the path the stub links to.
    Missing,
    /// The scan does not read the target: not a file, a name that begins with `.`,
    /// an extension `[scan] extensions` does not list, or a file it cannot read
    /// (§FS-declarations.stubs.target.3). Judged on the name the stub wrote.
    NotRead,
    /// The target was read, as the scan reads it on the text a save would write,
    /// and does not declare the ID (§AR-scanner.4.6,
    /// §FS-declarations.stubs.target.2).
    LacksId,
    /// The stub links to the file it sits in, which declares the ID elsewhere. The
    /// stub pairs with nothing there, so it is a declaration of its own
    /// (§FS-declarations.stubs.verdict).
    OwnFile,
    /// The target's records of the ID that are not themselves stubs, ascending by
    /// line and never empty: the stub points at each
    /// (§FS-declarations.stubs.pairing.1). Kept whole, sections and body span
    /// included, so a reader answers from the record whether or not the walk
    /// reached the target.
    Declares(Vec<Declaration>),
}

impl StubResolution {
    /// Whether `check` reports the stub broken (§FS-declarations.checks.broken-stub).
    pub(crate) fn is_broken(&self) -> bool {
        matches!(self, Self::Missing | Self::NotRead | Self::LacksId)
    }
}

/// One declaration of an ID once its stubs are paired: the record it is, and the
/// declaration of the catalog that stands for it — the record itself, or the stub
/// that points at a target the catalog holds no record of.
#[derive(Clone, Copy)]
pub(crate) struct Paired<'a> {
    /// The declaration a body, a section, an anchor or a site is read from.
    pub(crate) record: &'a Declaration,
    /// The declaration in the catalog that stands for this one.
    pub(crate) stand_in: &'a Declaration,
}

impl Paired<'_> {
    /// Whether this declaration is a stub `check` reports broken.
    pub(crate) fn is_broken_stub(&self) -> bool {
        self.record
            .stub_resolution
            .as_ref()
            .is_some_and(StubResolution::is_broken)
    }
}

/// The declarations of one ID once its stubs are paired, in `path:line` order of
/// their records.
pub(crate) struct PairedDeclarations<'a> {
    decls: &'a [Declaration],
    paired: Vec<Paired<'a>>,
}

impl<'a> PairedDeclarations<'a> {
    pub(crate) fn len(&self) -> usize {
        self.paired.len()
    }

    /// Every declaration, in `path:line` order of its record.
    pub(crate) fn iter(&self) -> impl Iterator<Item = Paired<'a>> + '_ {
        self.paired.iter().copied()
    }

    /// The one declaration of an ID that has exactly one: what a link, a body, a
    /// section and a value read. An ID with more than one is a duplicate, reported
    /// rather than ranked, so none of them answers here.
    pub(crate) fn sole(&self) -> Option<Paired<'a>> {
        match self.paired.as_slice() {
            [paired] => Some(*paired),
            _ => None,
        }
    }

    /// The declarations of the catalog that stand for one, each once, in the
    /// catalog's order. A stub that stands for none is the pointer to one.
    pub(crate) fn stand_ins(&self) -> impl Iterator<Item = &'a Declaration> + '_ {
        self.decls.iter().filter(|decl| {
            self.paired
                .iter()
                .any(|paired| std::ptr::eq(paired.stand_in, *decl))
        })
    }
}

/// The declarations of the ID `decls` declares, once its stubs are paired by their
/// verdicts (§FS-declarations.stubs.verdict, §AR-scanner.4.6). A declaration that
/// is not a stub is one of its own. A stub whose verdict is `Declares` stands for
/// each record there that no declaration, and no stub before it in `path:line`
/// order, already stands for, so stubs to one target stand for its declarations
/// once between them and a target declaring the ID twice is two
/// (§FS-declarations.stubs.pairing.2). Any other stub is a declaration of its
/// own: a broken one, and one that links to its own file.
pub(crate) fn paired_declarations(decls: &[Declaration]) -> PairedDeclarations<'_> {
    let mut paired: Vec<Paired<'_>> = decls
        .iter()
        .filter(|decl| !decl.is_stub)
        .map(|decl| Paired {
            record: decl,
            stand_in: decl,
        })
        .collect();
    let mut stubs: Vec<&Declaration> = decls.iter().filter(|decl| decl.is_stub).collect();
    stubs.sort_by(|a, b| site_order(a, b));
    for stub in stubs {
        let Some(StubResolution::Declares(records)) = &stub.stub_resolution else {
            paired.push(Paired {
                record: stub,
                stand_in: stub,
            });
            continue;
        };
        for record in records {
            // The verdict holds clones of the walk's records, or one reading of a
            // target per scan, so one declaration is one `file` and `line` here.
            if !paired
                .iter()
                .any(|seen| seen.record.file == record.file && seen.record.line == record.line)
            {
                paired.push(Paired {
                    record,
                    stand_in: stub,
                });
            }
        }
    }
    paired.sort_by(|a, b| site_order(a.record, b.record));
    PairedDeclarations { decls, paired }
}

/// Whether some declaration of `id` records a heading at section path `section`:
/// the one lookup §FS-check.3.2 reports a missing section from, §FS-fmt.2.4.6
/// declines a declaration-local rewrite on, §FS-check.3.17.4 admits a bare
/// index entry by, and a rule's `cites` fact and a lent citation resolve by
/// (§FS-rules.5.1, §FS-declarations.stubs.target.5), so the finding and the refusals
/// cannot disagree. An undeclared `id` has no sections, and an owner with no
/// numbered headings, a wholly absent path and a partially resolving one all answer
/// `false`. A stub's sections are those of the target's record its verdict holds
/// (§AR-scanner.4.6), scanned or not, as `check` reports them (§FS-check.3.2.1):
/// where no recorded declaration holds the path, the ID's one declaration answers
/// from the record the scan kept on the stub that stands for it; an ID `show`
/// refuses as ambiguous, and a broken stub, lend no section there.
pub(crate) fn section_resolves(findings: &Catalog, id: &Id, section: &str) -> bool {
    let Some(decls) = findings.declarations.get(id) else {
        return false;
    };
    if decls.iter().any(|decl| decl.sections.contains_key(section)) {
        return true;
    }
    paired_declarations(decls).sole().is_some_and(|sole| {
        !std::ptr::eq(sole.record, sole.stand_in) && sole.record.sections.contains_key(section)
    })
}

fn site_order(a: &Declaration, b: &Declaration) -> Ordering {
    (sort_path_key(&a.file), a.line).cmp(&(sort_path_key(&b.file), b.line))
}
