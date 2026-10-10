//! The one function that maps a citation to the project it resolves against
//! (§AR-system.2.10, §AR-resolver.1): the target's `Catalog` and the `Schema`
//! and `Compiled` grammar its ID is parsed and rendered with, or `None` when the alias is unknown
//! (§FS-workspace.1.2, §FS-workspace.8).
//!
//! Every consumer of citations goes through here rather than matching on
//! `citation.namespace` itself, which is what keeps `check`, the editor jump and
//! every query command agreeing on what "qualified" means (§AR-resolver.1). A
//! `None` return is never a silent skip: the calling rule turns it into a
//! located diagnostic (§AR-resolver.2).
//!
//! It sat in `checker/support.rs` with the rules that call it, which had the
//! grammar's shorthand resolution reading the target record upward out of the
//! checker (§AR-system.4). The record is a pair of borrows and the function a
//! map lookup — neither is a rule — so both belong to the component that holds
//! the loaded set they are borrowed from.

use std::collections::BTreeMap;

use crate::config::{Compiled, Display, Frame, Run, Schema};
use crate::model::{Catalog, Citation, Declaration, Id, id_homes};

/// One project a citation can resolve against, as a rule needs it: the catalog
/// the ID is looked up in and the schema and grammar that spell it, because a
/// workspace may mix `[id] format`s (§FS-workspace.1.2, §AR-workspace.2), with
/// the run that roots its stubs — records, never the façade (§AR-config.5).
#[derive(Clone, Copy)]
pub(crate) struct WorkspaceCheckTarget<'a> {
    pub(crate) catalog: &'a Catalog,
    pub(crate) schema: &'a Schema,
    pub(crate) compiled: &'a Compiled,
    pub(crate) run: &'a Run,
}

impl<'a> WorkspaceCheckTarget<'a> {
    /// The target a project's `catalog` makes, in the frame it was loaded in.
    pub(crate) fn of(catalog: &'a Catalog, schema: &'a Schema, frame: Frame<'a>) -> Self {
        Self {
            catalog,
            schema,
            compiled: frame.compiled,
            run: frame.run,
        }
    }

    /// The frame a read of this project runs in, spelled from its own root: a
    /// finding about it is spelled through the citing project's frame instead
    /// (§FS-workspace.8.1).
    pub(crate) fn frame(&self) -> Frame<'a> {
        Frame {
            run: self.run,
            compiled: self.compiled,
            name: None,
            alias: None,
            display: Display {
                root: &self.run.root,
                cli_base: &self.run.cli_base,
                from_root: true,
            },
        }
    }
}

/// §AR-resolver.1: the resolver itself. An unqualified citation resolves against
/// `local`, a qualified one against the alias-selected project, and an unknown
/// alias returns `None` for the caller to locate.
pub(crate) fn target_for_citation<'a>(
    cite: &Citation,
    local: &'a Catalog,
    local_schema: &'a Schema,
    local_frame: Frame<'a>,
    workspace: &'a BTreeMap<String, WorkspaceCheckTarget<'a>>,
) -> Option<WorkspaceCheckTarget<'a>> {
    match cite.namespace.as_deref() {
        Some(namespace) => workspace.get(namespace).copied(),
        None => Some(WorkspaceCheckTarget::of(local, local_schema, local_frame)),
    }
}

/// Whether a citation's target declares the ID — the same resolution above,
/// asked as a yes/no by the rules that only need that (§FS-check.3.1).
pub(crate) fn citation_resolves(
    cite: &Citation,
    local: &Catalog,
    local_schema: &Schema,
    local_frame: Frame<'_>,
    workspace: &BTreeMap<String, WorkspaceCheckTarget<'_>>,
) -> bool {
    target_for_citation(cite, local, local_schema, local_frame, workspace)
        .map(|target| target.catalog.declarations.contains_key(&cite.id))
        .unwrap_or(false)
}

/// Whether some declaration of `id` records a heading at section path `section`:
/// the one lookup §FS-check.3.2 reports a missing section from, §FS-fmt.2.4.6
/// declines a declaration-local rewrite on, and §FS-check.3.17.4 admits a bare
/// index entry by, so the finding and the refusals cannot disagree. An undeclared
/// `id` has no sections, and an owner with no numbered headings, a wholly absent
/// path and a partially resolving one all answer `false`. It is `section_home`,
/// asked as a yes/no.
pub(crate) fn section_resolves(findings: &Catalog, id: &Id, section: &str) -> bool {
    section_home(findings, id, section).is_some()
}

/// Which record holds a section a citation resolves to (§FS-check.3.2.1).
pub(crate) enum SectionHome<'a> {
    /// A declaration the walk recorded.
    Recorded,
    /// The one declaration of the ID in a stub's target, the record the scan kept
    /// on the stub because the catalog holds no record of it.
    Unscanned(&'a Declaration),
}

/// Where `section` of `id` resolves, or `None` where it does not. A stub's sections
/// are its target's, scanned or not (§FS-check.3.2.1): where no recorded declaration
/// holds the path, the ID's one home answers from the record the scan kept on the
/// stub that stands for it (§AR-scanner.4.6); an ID `show` refuses as ambiguous, and
/// a broken stub, lend no section there. A rule's `cites` fact asks here rather than
/// `section_resolves`, because a citation into a home outside the walk counts for
/// that home's chapter (§FS-rules.5.1).
pub(crate) fn section_home<'a>(
    findings: &'a Catalog,
    id: &Id,
    section: &str,
) -> Option<SectionHome<'a>> {
    let decls = findings.declarations.get(id)?;
    if decls.iter().any(|decl| decl.sections.contains_key(section)) {
        return Some(SectionHome::Recorded);
    }
    let home = id_homes(decls).sole()?;
    (!std::ptr::eq(home.record, home.stand_in) && home.record.sections.contains_key(section))
        .then_some(SectionHome::Unscanned(home.record))
}
