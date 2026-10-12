//! A stub's verdict, reached once after the walk (§AR-scanner.4.6): whether the
//! file a stub points at is there, whether the scan reads it, and which of its
//! records declare the stub's ID, read through the walk's own pass on the text a
//! save would write (§FS-declarations.stubs.target.1). Every command reads
//! the verdict recorded on the stub, so the stub's health and the count of the ID's
//! declarations agree whether or not the walk reached the target, and whether or not
//! an edit to it is saved (§FS-declarations.stubs.verdict).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::read_predicate::{excluding_rule, name_refusal};
use super::tree::scan_unwalked_file;
use crate::config::{Frame, Run, Schema};
use crate::grammar::{Grammar, resolve_shorthands};
use crate::model::{
    Catalog, Citation, Declaration, ExcludedStubTargetNote, ExcludingRule, FileRecord, Id,
    StubResolution, paired_declarations,
};
use crate::resolver::section_resolves;
use crate::sources::{paths_same_location, physical_path_key};

/// Reach every stub's verdict once, after the walk, and record it on the stub
/// (§AR-scanner.4.6): every command reads it from there and none reaches its own
/// (§FS-declarations.stubs.verdict). A target the walk did not record the ID
/// in is read through the walk's own pass over one file, on the text a save would
/// write (§FS-declarations.stubs.target.1), once per target however many
/// stubs and IDs name it. An ID no stub declares is passed over. A stub that pairs
/// with a target the walk from `roots` would not read, for an `exclude` entry or an
/// ignore file, is noted for the warning (§FS-declarations.checks.excluded-stub-target).
/// Once every verdict is in, such a target lends the declaration its citations
/// (§FS-declarations.stubs.target.5).
pub(super) fn resolve_stubs(
    schema: &Schema,
    frame: Frame<'_>,
    roots: &[PathBuf],
    findings: &mut Catalog,
) {
    let mut targets = UnwalkedTargets::new(schema, frame, roots);
    let mut notes = Vec::new();
    for (id, decls) in &mut findings.declarations {
        if !decls.iter().any(|decl| decl.is_stub) {
            continue;
        }
        let verdicts: Vec<Option<StubResolution>> = decls
            .iter()
            .map(|decl| resolve_stub(frame, id, decl, decls.as_slice(), &mut targets, &mut notes))
            .collect();
        for (decl, verdict) in decls.iter_mut().zip(verdicts) {
            decl.stub_resolution = verdict;
        }
    }
    // §AR-scanner.2.8: a target read here is sliced from its record like any scanned file.
    findings.files.extend(targets.files);
    findings.excluded_stub_target_notes.extend(notes);
    lend_citations(frame.grammar(), findings, targets.lendable);
}

/// Keep, of `candidates`, each citation a stub's target lends the declaration the
/// stub pairs with, resolved as a walked citation is, in the catalog's lent set
/// (§FS-declarations.stubs.target.5, §AR-scanner.4.6). A candidate is lent when the
/// pass over its file assigned it to a declaration whose record a stub stands for,
/// in that record's file, where the catalog holds no record of it. One that does
/// not resolve counts for nothing and is reported nowhere, so it is dropped here: an
/// unqualified ID nobody declares, a shorthand that names none or several, a section
/// the cited declaration lacks. A qualified one resolves against its namespace where
/// every edge reader already asks it. A `--full` run that narrows a walked target
/// away lends through here too.
pub(crate) fn lend_citations(
    grammar: &Grammar,
    findings: &mut Catalog,
    mut candidates: Vec<Citation>,
) {
    if candidates.is_empty() {
        return;
    }
    candidates.retain(|cite| stands_for_owner(findings, cite));
    if candidates.is_empty() {
        return;
    }
    let declared: Vec<Id> = findings.declarations.keys().cloned().collect();
    resolve_shorthands(grammar, &declared, candidates.iter_mut());
    let lent: Vec<Citation> = candidates
        .into_iter()
        .filter(|cite| resolves(findings, cite))
        .collect();
    findings.lent_citations.extend(lent);
}

/// Whether a stub stands for the record that encloses `cite`, one the catalog holds
/// no record of (§FS-declarations.stubs.target.5).
fn stands_for_owner(findings: &Catalog, cite: &Citation) -> bool {
    let Some(decls) = cite
        .enclosing_declaration
        .as_ref()
        .and_then(|owner| findings.declarations.get(owner))
    else {
        return false;
    };
    paired_declarations(decls).iter().any(|paired| {
        !std::ptr::eq(paired.record, paired.stand_in)
            && paths_same_location(&paired.record.file, &cite.file)
    })
}

/// Whether a lent citation is an edge: as in the scanned tree, an unknown ID or an
/// absent section never is (§FS-rules.5.1).
fn resolves(findings: &Catalog, cite: &Citation) -> bool {
    cite.namespace.is_some()
        || (findings.declarations.contains_key(&cite.id)
            && cite
                .section
                .as_deref()
                .is_none_or(|section| section_resolves(findings, &cite.id, section)))
}

/// The verdict on `stub`, asked in the order of §AR-scanner.4.6; `None` for a
/// declaration that is not a stub.
fn resolve_stub(
    frame: Frame<'_>,
    id: &Id,
    stub: &Declaration,
    decls: &[Declaration],
    targets: &mut UnwalkedTargets<'_>,
    notes: &mut Vec<ExcludedStubTargetNote>,
) -> Option<StubResolution> {
    let target = stub.defined_in.as_ref().filter(|_| stub.is_stub)?;
    // §FS-scan.text.buffer-only: a buffer with no file behind it is there.
    let sources = frame.sources();
    let resolved = sources.stub_target(frame.root(), &stub.file, target);
    if !sources.exists(&resolved) {
        return Some(StubResolution::Missing);
    }
    // §FS-declarations.stubs.target.3, §AR-scanner.1.13: only the name and
    // extension reasons, judged by the name the stub wrote before any record is
    // paired with it, so a symlink under a hidden or unlisted name never pairs.
    if !sources.is_file(&resolved) || name_refusal(&resolved, targets.schema).is_some() {
        return Some(StubResolution::NotRead);
    }
    if paths_same_location(&frame.root().join(&stub.file), &resolved) {
        let declared = decls
            .iter()
            .any(|decl| !decl.is_stub && decl.file == stub.file);
        return Some(if declared {
            StubResolution::OwnFile
        } else {
            StubResolution::LacksId
        });
    }
    let mut walked: Vec<Declaration> = decls
        .iter()
        .filter(|decl| !decl.is_stub && paths_same_location(&decl.file, &resolved))
        .cloned()
        .collect();
    if walked.is_empty() {
        walked = match targets.records(&resolved, id) {
            Some(records) => records,
            None => return Some(StubResolution::NotRead),
        };
        // §FS-declarations.stubs.target.5: the records' citations, once per target.
        targets.lend(&resolved, id);
        // §FS-declarations.stubs.target.3: a target the walk did not read keeps its
        // verdict until 0.19.0, and warns where it pairs.
        if !walked.is_empty()
            && let Some(rule) = targets.excluding_rule(&resolved)
        {
            notes.push(ExcludedStubTargetNote {
                file: stub.file.clone(),
                line: stub.line,
                id: id.clone(),
                target: target.clone(),
                rule,
            });
        }
    }
    walked.sort_by_key(|decl| decl.line);
    Some(if walked.is_empty() {
        StubResolution::LacksId
    } else {
        StubResolution::Declares(walked)
    })
}

/// The records of the targets the walk did not record a stub's ID in, by physical
/// location, each read once per scan through `scan_unwalked_file` (§AR-scanner.4.6).
/// A target that cannot be read is remembered as such; one that can keeps its file
/// record (§AR-scanner.2.8), which is how `show` reads a stub's target, and the
/// citations its pass assigns to a declaration, which it lends once per target and
/// ID (§FS-declarations.stubs.target.5).
struct UnwalkedTargets<'a> {
    schema: &'a Schema,
    frame: Frame<'a>,
    roots: &'a [PathBuf],
    read: BTreeMap<PathBuf, Option<TargetRead>>,
    excluded: BTreeMap<PathBuf, Option<ExcludingRule>>,
    files: BTreeMap<PathBuf, Arc<FileRecord>>,
    lendable: Vec<Citation>,
}

/// One target's pass: its declarations, and its citations by the declaration that
/// encloses each.
struct TargetRead {
    declarations: BTreeMap<Id, Vec<Declaration>>,
    citations: BTreeMap<Id, Vec<Citation>>,
}

impl<'a> UnwalkedTargets<'a> {
    fn new(schema: &'a Schema, frame: Frame<'a>, roots: &'a [PathBuf]) -> Self {
        Self {
            schema,
            frame,
            roots,
            read: BTreeMap::new(),
            excluded: BTreeMap::new(),
            files: BTreeMap::new(),
            lendable: Vec::new(),
        }
    }

    /// The `exclude` entry or ignore-file rule that keeps the walk from `path`, asked
    /// once per target (§FS-declarations.checks.excluded-stub-target).
    fn excluding_rule(&mut self, path: &Path) -> Option<ExcludingRule> {
        let (schema, frame, roots) = (self.schema, self.frame, self.roots);
        self.excluded
            .entry(physical_path_key(path))
            .or_insert_with(|| excluding_rule(schema, frame, roots, path))
            .clone()
    }

    /// The declarations of `id` in `path` that are not stubs, or `None` where the
    /// file cannot be read.
    fn records(&mut self, path: &Path, id: &Id) -> Option<Vec<Declaration>> {
        let (schema, frame, files) = (self.schema, self.frame, &mut self.files);
        let read = self
            .read
            .entry(physical_path_key(path))
            .or_insert_with(|| read_target(path, schema, frame, files))
            .as_ref()?;
        Some(
            read.declarations
                .get(id)
                .into_iter()
                .flatten()
                .filter(|decl| !decl.is_stub)
                .cloned()
                .collect(),
        )
    }

    /// Set the citations `path`'s pass assigns to a declaration of `id` aside to be
    /// lent, the first time a stub asks and never again, however many stubs name
    /// the target (§FS-declarations.stubs.target.5).
    fn lend(&mut self, path: &Path, id: &Id) {
        if let Some(Some(read)) = self.read.get_mut(&physical_path_key(path))
            && let Some(citations) = read.citations.remove(id)
        {
            self.lendable.extend(citations);
        }
    }
}

/// One pass over a target the walk did not reach, its file record kept in `files`.
/// The citations are those of a pass that assigns each to its enclosing
/// declaration, which a run that classifies no citing side asks for here alone, so
/// `list` counts what `check` counts (§FS-refs.2, §AR-scanner.2.4.2).
fn read_target(
    path: &Path,
    schema: &Schema,
    frame: Frame<'_>,
    files: &mut BTreeMap<PathBuf, Arc<FileRecord>>,
) -> Option<TargetRead> {
    let mut findings = scan_unwalked_file(path, schema, frame).ok()?;
    files.extend(std::mem::take(&mut findings.files));
    let citations = if frame.run.scope.classify_citation_sources {
        findings.citations
    } else {
        let mut run: Run = frame.run.clone();
        run.scope.classify_citation_sources = true;
        let classifying = Frame { run: &run, ..frame };
        scan_unwalked_file(path, schema, classifying)
            .ok()?
            .citations
    };
    let mut by_owner: BTreeMap<Id, Vec<Citation>> = BTreeMap::new();
    for citation in citations {
        if let Some(owner) = citation.enclosing_declaration.clone() {
            by_owner.entry(owner).or_default().push(citation);
        }
    }
    Some(TargetRead {
        declarations: findings.declarations,
        citations: by_owner,
    })
}
