//! A stub's verdict, reached once after the walk (§AR-scanner.4.6): whether the
//! file a stub points at is there, whether the scan reads it, and which of its
//! records declare the stub's ID, read through the walk's own pass on the text a
//! save would write (§FS-declarations.checks.broken-stub.1). Every command reads
//! the verdict recorded on the stub, so the stub's health and the count of homes
//! agree whether or not the walk reached the target, and whether or not an edit to
//! it is saved (§FS-declarations.checks.broken-stub.4).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::tree::scan_unwalked_file;
use super::walk_boundaries::is_scannable;
use crate::config::{Frame, Schema};
use crate::model::{
    Catalog, Declaration, Id, StubResolution, TextOverlays, paths_same_location, physical_path_key,
    resolve_stub_target,
};

/// Reach every stub's verdict once, after the walk, and record it on the stub
/// (§AR-scanner.4.6): every command reads it from there and none reaches its own
/// (§FS-declarations.checks.broken-stub.4). A target the walk did not record the ID
/// in is read through the walk's own pass over one file, on the text a save would
/// write (§FS-declarations.checks.broken-stub.1), once per target however many
/// stubs and IDs name it. An ID no stub declares is passed over.
pub(super) fn resolve_stubs(
    schema: &Schema,
    frame: Frame<'_>,
    overlays: &TextOverlays,
    findings: &mut Catalog,
) {
    let mut targets = UnwalkedTargets::new(schema, frame, overlays);
    for (id, decls) in &mut findings.declarations {
        if !decls.iter().any(|decl| decl.is_stub) {
            continue;
        }
        let verdicts: Vec<Option<StubResolution>> = decls
            .iter()
            .map(|decl| resolve_stub(frame, id, decl, decls.as_slice(), &mut targets))
            .collect();
        for (decl, verdict) in decls.iter_mut().zip(verdicts) {
            decl.stub_resolution = verdict;
        }
    }
}

/// The verdict on `stub`, asked in the order of §AR-scanner.4.6; `None` for a
/// declaration that is not a stub.
fn resolve_stub(
    frame: Frame<'_>,
    id: &Id,
    stub: &Declaration,
    decls: &[Declaration],
    targets: &mut UnwalkedTargets<'_>,
) -> Option<StubResolution> {
    let target = stub.defined_in.as_ref().filter(|_| stub.is_stub)?;
    let resolved = resolve_stub_target(frame.root(), &stub.file, target);
    if !resolved.exists() {
        return Some(StubResolution::Missing);
    }
    // §FS-declarations.checks.broken-stub.3: a file the scan would read wherever it
    // lay, judged by the name the stub wrote before any record is paired with it,
    // so a symlink under a hidden or unlisted name never pairs with its file.
    if !(resolved.is_file() && is_scannable(&resolved, targets.schema)) {
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
    }
    walked.sort_by_key(|decl| decl.line);
    Some(if walked.is_empty() {
        StubResolution::LacksId
    } else {
        StubResolution::Homes(walked)
    })
}

/// The records of the targets the walk did not record a stub's ID in, by physical
/// location, each read once per scan through `scan_unwalked_file` (§AR-scanner.4.6).
/// A target that cannot be read is remembered as such.
struct UnwalkedTargets<'a> {
    schema: &'a Schema,
    frame: Frame<'a>,
    overlays: &'a TextOverlays,
    read: BTreeMap<PathBuf, Option<BTreeMap<Id, Vec<Declaration>>>>,
}

impl<'a> UnwalkedTargets<'a> {
    fn new(schema: &'a Schema, frame: Frame<'a>, overlays: &'a TextOverlays) -> Self {
        Self {
            schema,
            frame,
            overlays,
            read: BTreeMap::new(),
        }
    }

    /// The declarations of `id` in `path` that are not stubs, or `None` where the
    /// file cannot be read.
    fn records(&mut self, path: &Path, id: &Id) -> Option<Vec<Declaration>> {
        let (schema, frame, overlays) = (self.schema, self.frame, self.overlays);
        let read = self
            .read
            .entry(physical_path_key(path))
            .or_insert_with(|| {
                let findings = scan_unwalked_file(path, schema, frame, overlays).ok()?;
                Some(findings.declarations)
            })
            .as_ref()?;
        Some(
            read.get(id)
                .into_iter()
                .flatten()
                .filter(|decl| !decl.is_stub)
                .cloned()
                .collect(),
        )
    }
}
