//! A stub's home, recorded once after the walk (§AR-scanner.4.6): where the file a
//! stub points at declares the stub's ID, read the way the broken-stub rule reads
//! it (§FS-declarations.checks.broken-stub), through the one reader both take, as a
//! save would write it (§FS-declarations.checks.broken-stub.1). So the count of
//! homes and the stub's health agree whether or not the walk reached the target,
//! and whether or not an edit to it is saved (§FS-declarations.checks.duplicate.1).

use anyhow::Result;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::tree::overlay_text;
use super::walk_boundaries::is_scannable;
use crate::config::Config;
use crate::grammar::{
    PythonDocstringScanState, STUB_LINK_HEADING, declaration_id_on_line, markdown_fence_delimiter,
    source_scan_line,
};
use crate::model::{
    Declaration, Findings, Id, StubHome, TextOverlays, paths_same_location, physical_path_key,
    resolve_stub_target,
};

/// Whether the scan reads `path`, a stub's target, at all: a file whose own name
/// does not begin with `.` and whose extension `[scan] extensions` lists, wherever it
/// lies (§FS-declarations.checks.broken-stub.3). A target it does not read declares
/// nothing, whatever it holds, so every reader of a stub's target asks this before
/// it reads: `file_declares_inline_home` for the broken-stub rule and `show`'s test,
/// the count of homes below, and the resolver's reading of a target the walk did not
/// reach (§AR-resolver.5), which `show`'s body and `refs`' refusals take.
pub(crate) fn scan_reads_target(path: &Path, config: &Config) -> bool {
    path.is_file() && is_scannable(path, config)
}

/// Whether `path` contains a real (non-stub) inline declaration of `id` —
/// the check that a stub's link target actually carries the inline home it claims
/// (§FS-declarations.checks.broken-stub, §AR-checker.2.5, §AR-scanner.4). A target
/// the scan does not read contains none (§FS-declarations.checks.broken-stub.3). One
/// it does is read as a save would write it (§FS-declarations.checks.broken-stub.1):
/// the editor's overlay where there is one, the disk otherwise, as the scan reads it.
pub(crate) fn file_declares_inline_home(
    path: &Path,
    id: &Id,
    config: &Config,
    overlays: &TextOverlays,
) -> Result<bool> {
    if !scan_reads_target(path, config) {
        return Ok(false);
    }
    let text = target_text(path, overlays)?;
    Ok(inline_home_line(&text, path, id, config).is_some())
}

/// The text of `path`, a stub's target, as a save would write it
/// (§FS-declarations.checks.broken-stub.1): the editor's overlay where the target is
/// open in one, the disk only where it is not. The broken-stub rule and the count of
/// homes both read a target here, so the stub the one accepts is the stub the other
/// pairs with its target (§FS-declarations.checks.duplicate.1).
fn target_text<'a>(path: &Path, overlays: &'a TextOverlays) -> std::io::Result<Cow<'a, str>> {
    Ok(match overlay_text(overlays, path) {
        Some(text) => Cow::Borrowed(text),
        // §FS-check.6.1.1: cover this effective input before its shared read.
        None => Cow::Owned(crate::config::input_read_to_string(path)?),
    })
}

/// The first line of `text`, the contents of `path`, that declares `id` and is not
/// itself a stub link (§FS-declarations.checks.broken-stub, §AR-scanner.4.6).
/// A Markdown `text` is read the way the scan reads it: fence delimiter lines and
/// every line inside a fence are skipped first (§AR-scanner.2.3.3), so a heading
/// shown there as an example declares nothing (§FS-declarations.checks.broken-stub.2).
fn inline_home_line(text: &str, path: &Path, id: &Id, config: &Config) -> Option<usize> {
    let is_md = path.extension().and_then(|e| e.to_str()) == Some("md");
    let is_py = path.extension().and_then(|e| e.to_str()) == Some("py");
    let mut py_docstring = PythonDocstringScanState::default();
    let mut markdown_fence = None;
    for (index, line) in text.lines().enumerate() {
        if is_md && markdown_fence_delimiter(&mut markdown_fence, line) {
            continue;
        }
        if markdown_fence.is_some() {
            continue;
        }
        let scan = source_scan_line(line, is_py, config.docstring_python, &mut py_docstring);
        let scan_line = scan.text.as_ref();
        if let Some((found, token_end)) =
            declaration_id_on_line(&config.grammar, scan_line, scan.in_py_docstring, is_md)
            && &found == id
        {
            let tail = &scan_line[token_end..];
            if STUB_LINK_HEADING.is_match(tail) {
                continue;
            }
            return Some(index + 1);
        }
    }
    None
}

/// Record on every stub of an ID declared more than once where its home declares
/// the ID (§AR-scanner.4.6): the record of the ID at the stub's target where the walk
/// holds one, else the target read as a save would write it, the editor's text in
/// `overlays` where it is open and the disk otherwise, once per target however many
/// stubs and IDs name it. An ID declared once is passed over unresolved, so a lone
/// stub costs nothing here and stays its own home (§FS-declarations.checks.duplicate.1).
pub(super) fn record_stub_homes(config: &Config, overlays: &TextOverlays, findings: &mut Findings) {
    let mut targets = TargetTexts::new(overlays);
    for (id, decls) in &mut findings.declarations {
        if decls.len() < 2 || !decls.iter().any(|decl| decl.is_stub) {
            continue;
        }
        let homes: Vec<Option<StubHome>> = decls
            .iter()
            .map(|decl| stub_home(config, id, decl, decls.as_slice(), &mut targets))
            .collect();
        for (decl, home) in decls.iter_mut().zip(homes) {
            decl.stub_home = home;
        }
    }
}

/// Where `decl`, if it is a stub, finds the home it points at (§AR-scanner.4.6).
fn stub_home(
    config: &Config,
    id: &Id,
    decl: &Declaration,
    decls: &[Declaration],
    targets: &mut TargetTexts<'_>,
) -> Option<StubHome> {
    if !decl.is_stub {
        return None;
    }
    let target = decl.defined_in.as_ref()?;
    let resolved = resolve_stub_target(&config.root, &decl.file, target);
    // A stub that links to its own file pairs with nothing in it, as the predicate
    // has it, so its sites stay its own line and its file's (§AR-scanner.4.6).
    if paths_same_location(&config.root.join(&decl.file), &resolved) {
        return None;
    }
    // Kept where the walk reached the target, so a scope narrowed after it still
    // pairs the stub (§AR-checker.2.13).
    if let Some(record) = decls
        .iter()
        .find(|other| paths_same_location(&other.file, &resolved) && other.file != decl.file)
    {
        return Some(StubHome {
            path: record.file.clone(),
            line: record.line,
        });
    }
    // §FS-declarations.checks.broken-stub.3: the rule's own gate, a file the scan reads.
    if !scan_reads_target(&resolved, config) {
        return None;
    }
    // §FS-declarations.checks.duplicate.1: the rule's text, the editor's before the
    // disk, whether or not the walk reached it (§FS-declarations.checks.broken-stub.1).
    let line = inline_home_line(targets.read(&resolved)?, &resolved, id, config)?;
    Some(StubHome {
        path: resolved,
        line,
    })
}

/// The targets one pass has read, by physical location, so each is read once
/// (§AR-scanner.4.6), through the broken-stub rule's reader `target_text`. An
/// unreadable target is remembered as such.
struct TargetTexts<'a> {
    overlays: &'a TextOverlays,
    texts: BTreeMap<PathBuf, Option<Cow<'a, str>>>,
}

impl<'a> TargetTexts<'a> {
    fn new(overlays: &'a TextOverlays) -> Self {
        Self {
            overlays,
            texts: BTreeMap::new(),
        }
    }

    fn read(&mut self, path: &Path) -> Option<&str> {
        let overlays = self.overlays;
        self.texts
            .entry(physical_path_key(path))
            .or_insert_with(|| target_text(path, overlays).ok())
            .as_deref()
    }
}
