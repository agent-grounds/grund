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
use crate::config::{Frame, Schema};
use crate::grammar::{
    PythonDocstringScanState, STUB_LINK_HEADING, declaration_id_on_line, markdown_fence_delimiter,
    source_scan_line,
};
use crate::model::{
    Catalog, Declaration, Id, StubHome, TextOverlays, paths_same_location, physical_path_key,
    resolve_stub_target,
};

/// Whether the scan reads `path`, a stub's target, at all: a file whose own name
/// does not begin with `.` and whose extension `[scan] extensions` lists, wherever it
/// lies (§FS-declarations.checks.broken-stub.3). A target it does not read declares
/// nothing, whatever it holds, so every reader of a stub's target asks this before
/// it reads: `file_declares_inline_home` for the broken-stub rule and `show`'s test,
/// the count of homes below, and the resolver's reading of a target the walk did not
/// reach (§AR-resolver.5), which `show`'s body and `refs`' refusals take.
pub(crate) fn scan_reads_target(path: &Path, schema: &Schema) -> bool {
    path.is_file() && is_scannable(path, schema)
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
    schema: &Schema,
    frame: Frame<'_>,
    overlays: &TextOverlays,
) -> Result<bool> {
    if !scan_reads_target(path, schema) {
        return Ok(false);
    }
    let text = target_text(path, overlays)?;
    Ok(inline_home_lines(&text, path, id, schema, frame)
        .next()
        .is_some())
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

/// Every line of `text`, the contents of `path`, that declares `id` and is not
/// itself a stub link, in file order (§FS-declarations.checks.broken-stub,
/// §AR-scanner.4.6): each is a home, so a target that declares the ID twice is two
/// (§FS-declarations.checks.duplicate.1). The broken-stub rule asks only for the
/// first. A Markdown `text` is read the way the scan reads it: fence delimiter
/// lines and every line inside a fence are skipped first (§AR-scanner.2.3.3), so a
/// heading shown there as an example declares nothing
/// (§FS-declarations.checks.broken-stub.2).
fn inline_home_lines<'a>(
    text: &'a str,
    path: &Path,
    id: &'a Id,
    schema: &'a Schema,
    frame: Frame<'a>,
) -> impl Iterator<Item = usize> + 'a {
    let is_md = path.extension().and_then(|e| e.to_str()) == Some("md");
    let is_py = path.extension().and_then(|e| e.to_str()) == Some("py");
    let mut py_docstring = PythonDocstringScanState::default();
    let mut markdown_fence = None;
    text.lines().enumerate().filter_map(move |(index, line)| {
        if is_md && markdown_fence_delimiter(&mut markdown_fence, line) {
            return None;
        }
        if markdown_fence.is_some() {
            return None;
        }
        let scan = source_scan_line(
            line,
            is_py,
            schema.sources.docstring_python,
            &mut py_docstring,
        );
        let scan_line = scan.text.as_ref();
        let (found, token_end) =
            declaration_id_on_line(frame.grammar(), scan_line, scan.in_py_docstring, is_md)?;
        (&found == id && !STUB_LINK_HEADING.is_match(&scan_line[token_end..])).then_some(index + 1)
    })
}

/// Record on every stub, a lone one too, every line its home declares the ID on
/// (§AR-scanner.4.6): the records of the ID at the stub's target where the walk
/// holds them, else the target read as a save would write it, the editor's text in
/// `overlays` where it is open and the disk otherwise, once per target however many
/// stubs and IDs name it. A lone stub is read because its target may declare the ID
/// twice, two homes whether or not the walk reached it
/// (§FS-declarations.checks.duplicate.1); an ID no stub declares is passed over.
pub(super) fn record_stub_homes(
    schema: &Schema,
    frame: Frame<'_>,
    overlays: &TextOverlays,
    findings: &mut Catalog,
) {
    let mut targets = TargetTexts::new(overlays);
    for (id, decls) in &mut findings.declarations {
        if !decls.iter().any(|decl| decl.is_stub) {
            continue;
        }
        let homes: Vec<Option<StubHome>> = decls
            .iter()
            .map(|decl| stub_home(schema, frame, id, decl, decls.as_slice(), &mut targets))
            .collect();
        for (decl, home) in decls.iter_mut().zip(homes) {
            decl.stub_home = home;
        }
    }
}

/// Where `decl`, if it is a stub, finds the home it points at (§AR-scanner.4.6).
fn stub_home(
    schema: &Schema,
    frame: Frame<'_>,
    id: &Id,
    decl: &Declaration,
    decls: &[Declaration],
    targets: &mut TargetTexts<'_>,
) -> Option<StubHome> {
    if !decl.is_stub {
        return None;
    }
    let target = decl.defined_in.as_ref()?;
    let resolved = resolve_stub_target(frame.root(), &decl.file, target);
    // A stub that links to its own file pairs with nothing in it, as the predicate
    // has it, so its sites stay its own line and its file's (§AR-scanner.4.6).
    if paths_same_location(&frame.root().join(&decl.file), &resolved) {
        return None;
    }
    // Kept where the walk reached the target, so a scope narrowed after it still
    // pairs the stub (§AR-checker.2.13), at each of its records there.
    let at_target = |other: &&Declaration| {
        paths_same_location(&other.file, &resolved) && other.file != decl.file
    };
    if let Some(record) = decls.iter().find(at_target) {
        let mut lines: Vec<usize> = decls
            .iter()
            .filter(at_target)
            .map(|other| other.line)
            .collect();
        lines.sort_unstable();
        return Some(StubHome {
            path: record.file.clone(),
            lines,
        });
    }
    // §FS-declarations.checks.broken-stub.3: the rule's own gate, a file the scan reads.
    if !scan_reads_target(&resolved, schema) {
        return None;
    }
    // §FS-declarations.checks.duplicate.1: the rule's text, the editor's before the
    // disk, whether or not the walk reached it (§FS-declarations.checks.broken-stub.1),
    // and every line of it that declares the ID, never the first alone.
    let lines: Vec<usize> =
        inline_home_lines(targets.read(&resolved)?, &resolved, id, schema, frame).collect();
    if lines.is_empty() {
        return None;
    }
    Some(StubHome {
        path: resolved,
        lines,
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
