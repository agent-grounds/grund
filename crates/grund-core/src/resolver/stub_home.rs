//! A stub's home outside the walk (§AR-resolver.5): the declarations of a stub's
//! ID in a target `[scan] include` does not reach, as the scanner's own pass over
//! that file records them. `show` reads a stub's body and sections out of this
//! reading (§FS-show.2.3.7), and every reader in `check` asks it for a section the
//! recorded declarations do not hold (§FS-check.3.2.1), so the two commands count
//! one section set (§FS-show.2.2.2.2). `list --size` measures the home it finds, and
//! that home's sections (§FS-list.3.4.6).

use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::model::{
    Declaration, Findings, Id, TargetRecords, TextOverlays, is_stub_for_inline_decl,
    paths_same_location, physical_path_key, resolve_stub_target,
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

/// The declaration of `id` that `stub` pairs with in a target the walk recorded
/// nothing of `id` at (§FS-check.3.2.1): read on the first ask, at most once per
/// target per run, under the overlays the walk read (`Findings::stub_targets`).
///
/// `None` for anything but a stub whose target is another scannable file. A target
/// holding a recorded declaration of `id` answers from that record, which is
/// already among `findings`; a stub that links to its own file pairs with nothing
/// in it (§AR-scanner.4.6). A broken stub's target declares no home of `id`
/// (§FS-declarations.checks.broken-stub), so it is `None` too: what is read is the
/// target's declaration of the ID, never the file's headings. So is a target that
/// declares `id` twice, and every target of an ID with more than one home, which
/// `show` refuses as ambiguous rather than read (§FS-show.2.3.7, §FS-show.2.2.1).
pub(crate) fn unscanned_stub_home<'a>(
    findings: &'a Findings,
    config: &Config,
    id: &Id,
    stub: &Declaration,
) -> Option<&'a Declaration> {
    let target = stub.defined_in.as_ref().filter(|_| stub.is_stub)?;
    let resolved = resolve_stub_target(&config.root, &stub.file, target);
    let recorded = findings.declarations.get(id).map_or(&[][..], Vec::as_slice);
    if paths_same_location(&config.root.join(&stub.file), &resolved)
        || recorded
            .iter()
            .any(|decl| paths_same_location(&decl.file, &resolved))
    {
        return None;
    }
    // §FS-check.3.2.1: an ambiguous ID lends no section, as `show` reads none.
    let mut homes = recorded
        .iter()
        .filter(|decl| !is_stub_for_inline_decl(&config.root, decl, recorded));
    if homes.next().is_some() && homes.next().is_some() {
        return None;
    }
    // §AR-checker.2.5: the broken-stub rule's own reading, scannable files only.
    if !resolved.is_file() || !is_scannable(&resolved, config) {
        return None;
    }
    match findings
        .stub_targets
        .records(
            &physical_path_key(&resolved),
            || stub_target_keys(findings, config),
            || target_records(&resolved, config, findings.stub_targets.overlays()),
        )?
        .get(id)?
        .as_slice()
    {
        [home] => Some(home),
        _ => None,
    }
}

/// `home` itself, or where it is a stub with a home outside the walk, the
/// declaration of `id` there: the record a reader in `check` reads a value or a
/// value's authority from, as it would were the target scanned (§FS-check.3.2.1),
/// and the home the size catalog measures (§FS-list.3.4.6).
pub(crate) fn home_as_scanned<'a>(
    findings: &'a Findings,
    config: &Config,
    id: &Id,
    home: &'a Declaration,
) -> &'a Declaration {
    unscanned_stub_home(findings, config, id, home).unwrap_or(home)
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
