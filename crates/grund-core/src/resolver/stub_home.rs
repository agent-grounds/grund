//! A stub's home outside the walk (§AR-resolver.5): the declarations of a stub's
//! ID in a target `[scan] include` does not reach, as the scanner's own pass over
//! that file records them. `show` reads a stub's body and sections out of this
//! reading (§FS-show.2.3.7), and every reader in `check` asks it for a section the
//! recorded declarations do not hold (§FS-check.3.2.1), so the two commands count
//! one section set (§FS-show.2.2.2.2).

use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::model::{
    Declaration, Findings, Id, TargetRecords, TextOverlays, paths_same_location, physical_path_key,
    resolve_stub_target,
};
use crate::scanner::{is_scannable, scan_unwalked_file};

/// What the scanner's own pass over `file` declares, by ID, stubs left out: the
/// records of a file the walk did not reach, read from the editor's overlay where
/// there is one and from the disk through the input observation of §FS-check.6.1.1
/// otherwise (§FS-show.2.3.7). A file that cannot be read declares nothing.
pub(crate) fn target_records(
    file: &Path,
    config: &Config,
    overlays: &TextOverlays,
) -> TargetRecords {
    let Ok(findings) = scan_unwalked_file(file, config, overlays) else {
        return TargetRecords::new();
    };
    findings
        .declarations
        .into_iter()
        .filter_map(|(id, decls)| {
            let homes: Vec<Declaration> = decls.into_iter().filter(|decl| !decl.is_stub).collect();
            (!homes.is_empty()).then_some((id, homes))
        })
        .collect()
}

/// The declarations of `id` that `stub` pairs with in a target the walk recorded
/// nothing of `id` at (§FS-check.3.2.1): read on the first ask, at most once per
/// target per run, under the overlays the walk read (`Findings::stub_targets`).
///
/// Empty for anything but a stub whose target is another scannable file. A target
/// holding a recorded declaration of `id` answers from that record, which is
/// already among `findings`; a stub that links to its own file pairs with nothing
/// in it (§AR-scanner.4.6). A broken stub's target declares no home of `id`
/// (§FS-declarations.checks.broken-stub), so it is empty too: what is read is the
/// target's declaration of the ID, never the file's headings.
pub(crate) fn unscanned_stub_homes<'a>(
    findings: &'a Findings,
    config: &Config,
    id: &Id,
    stub: &Declaration,
) -> &'a [Declaration] {
    let Some(target) = stub.defined_in.as_ref().filter(|_| stub.is_stub) else {
        return &[];
    };
    let resolved = resolve_stub_target(&config.root, &stub.file, target);
    let recorded = findings.declarations.get(id).map_or(&[][..], Vec::as_slice);
    if paths_same_location(&config.root.join(&stub.file), &resolved)
        || recorded
            .iter()
            .any(|decl| paths_same_location(&decl.file, &resolved))
    {
        return &[];
    }
    // §AR-checker.2.5: the broken-stub rule's own reading, scannable files only.
    if !resolved.is_file() || !is_scannable(&resolved, config) {
        return &[];
    }
    findings
        .stub_targets
        .records(
            &physical_path_key(&resolved),
            || stub_target_keys(findings, config),
            || target_records(&resolved, config, findings.stub_targets.overlays()),
        )
        .and_then(|records| records.get(id))
        .map_or(&[], Vec::as_slice)
}

/// `home` itself, or where it is a stub with a home outside the walk, the
/// declarations of `id` there: the records a reader in `check` compares against,
/// as it would were the target scanned (§FS-check.3.2.1).
pub(crate) fn homes_as_scanned<'a>(
    findings: &'a Findings,
    config: &Config,
    id: &Id,
    home: &'a Declaration,
) -> &'a [Declaration] {
    match unscanned_stub_homes(findings, config, id, home) {
        [] => std::slice::from_ref(home),
        records => records,
    }
}

/// Every stub's target, by physical location: the slots one run keeps, resolved
/// once, on the first section a recorded declaration does not hold.
fn stub_target_keys(findings: &Findings, config: &Config) -> Vec<PathBuf> {
    findings
        .declarations
        .values()
        .flatten()
        .filter(|decl| decl.is_stub)
        .filter_map(|decl| {
            let target = decl.defined_in.as_ref()?;
            let resolved = resolve_stub_target(&config.root, &decl.file, target);
            Some(physical_path_key(&resolved))
        })
        .collect()
}
