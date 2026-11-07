//! External enrollment, the half of the kind-index membership a canonical link adds to what the
//! folder owes (§AR-checker.2.16, §FS-check.3.18.3), split from `index_entries.rs` along that seam
//! (§AR-core-module-layout.1).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::index::{KindIndexTarget, declarations_under_folder};
use super::index_entries::{IndexHeadings, IndexNames};
use super::spellings_report::PlaceNames;
use crate::config::Owners;
use crate::grammar::{
    CitationLink, LineLinks, MarkdownLinks, citation_link, markdown_never_rewrite_context,
};
use crate::model::{
    Catalog, Citation, Declaration, Id, paired_declarations, scanned_decl_relative_path,
};
use crate::sources::Sources;

/// Add every canonical external-inline enrollment to `owed`, returning the
/// exact citation sites that are navigation rather than use (§FS-check.3.18.3).
/// Citations are already scanner records; index text is read once per target
/// solely to inspect the persisted Markdown wrapper and destination.
pub(super) fn enroll_external_inline_declarations(
    findings: &Catalog,
    targets: &[KindIndexTarget<'_>],
    owners: &Owners<'_>,
    spellings: &PlaceNames<'_>,
    names: &IndexNames,
    configured_root: &Path,
    physical_root: &Path,
    index_targets: &BTreeMap<(PathBuf, Id), String>,
    sources: Sources<'_>,
    only: Option<&BTreeSet<Id>>,
    headings: &IndexHeadings,
    owed: &mut BTreeMap<PathBuf, BTreeSet<Id>>,
) -> BTreeMap<PathBuf, BTreeSet<(usize, usize)>> {
    let mut targets_by_index: BTreeMap<&Path, Vec<&KindIndexTarget<'_>>> = BTreeMap::new();
    for target in targets {
        targets_by_index
            .entry(target.index_key.as_path())
            .or_default()
            .push(target);
    }
    let text_by_index: BTreeMap<&Path, String> = targets
        .iter()
        .filter_map(|target| {
            // §FS-check.6.1.1: cover this effective input before its shared read,
            // as a save would write it (§AR-system.2.13).
            let text = sources.text(&target.index_file).ok()?;
            Some((target.index_key.as_path(), text.to_string()))
        })
        .collect();
    // §FS-citations.wrap: each index is the document its references are read in.
    let lines_by_index: BTreeMap<&Path, (Vec<&str>, MarkdownLinks<'_>)> = text_by_index
        .iter()
        .map(|(key, text)| (*key, (text.lines().collect(), MarkdownLinks::new(text))))
        .collect();
    let mut sites: BTreeMap<PathBuf, BTreeSet<(usize, usize)>> = BTreeMap::new();

    for citation in &findings.citations {
        if citation.namespace.is_some()
            || citation.section.is_some()
            || !citation.has_marker
            || citation.shorthand
            || only.is_some_and(|ids| !ids.contains(&citation.id))
        {
            continue;
        }
        let Some(relative) =
            scanned_decl_relative_path(&citation.file, configured_root, physical_root)
        else {
            continue;
        };
        // §FS-check.3.18.3: a citation on a declaration heading never enrolls, so it is
        // not the entry that satisfies either.
        if names
            .keys(relative.as_ref())
            .any(|key| headings.holds(key, citation.line))
        {
            continue;
        }
        let Some(target) = names.keys(relative.as_ref()).find_map(|key| {
            targets_by_index
                .get(key)?
                .iter()
                .copied()
                .find(|target| target.kind == citation.id.kind)
        }) else {
            continue;
        };
        let Some(decls) = findings.declarations.get(&citation.id) else {
            continue;
        };
        // The scan read each file as its language reads it, and recorded that.
        let markdown = |file: &Path| findings.files.get(file).is_some_and(|file| file.markdown);
        if external_inline_home(decls, target.folder, owners, spellings, &markdown).is_none() {
            continue;
        }
        let Some((lines, links)) = lines_by_index.get(target.index_key.as_path()) else {
            continue;
        };
        let index = citation.line.saturating_sub(1);
        let Some(line) = lines.get(index) else {
            continue;
        };
        let Some(written_target) = link_target_at_citation(line, links.line(index), citation)
        else {
            continue;
        };
        // §AR-checker.2.16: the destination is `Expected`'s; this pass only looks it up.
        let Some(canonical_target) =
            index_targets.get(&(target.index_file.clone(), citation.id.clone()))
        else {
            continue;
        };
        if written_target != canonical_target {
            continue;
        }
        owed.entry(target.index_key.clone())
            .or_default()
            .insert(citation.id.clone());
        sites
            .entry(target.index_key.clone())
            .or_default()
            .insert((citation.line, citation.column));
    }
    sites
}

/// The one logical, non-Markdown home an enrollment candidate would add. A
/// declaration already under the folder is covered by the ordinary rule, and
/// multiple independent homes remain the duplicate error rather than becoming
/// an index membership `grund` guessed (§REQ-no-wrong-citation.1). The ID has one
/// only where it has one declaration once its stubs are paired by their verdicts
/// (§AR-scanner.4.6). `markdown` says whether a file reads as Markdown, which its
/// language decides (§FS-config-v2.schema.sources.extensions).
fn external_inline_home<'a>(
    decls: &'a [Declaration],
    folder: &str,
    owners: &Owners<'_>,
    spellings: &PlaceNames<'_>,
    markdown: &dyn Fn(&Path) -> bool,
) -> Option<&'a Declaration> {
    if declarations_under_folder(decls, folder, owners, spellings) {
        return None;
    }
    let home = paired_declarations(decls).sole()?.stand_in;
    if home.is_stub || home.e2e_case.is_some() || markdown(&home.file) {
        return None;
    }
    Some(home)
}

/// Return the destination of the Markdown wrapper at this scanner-recorded
/// citation site. Unlike `index_citation_form`, enrollment must inspect this
/// exact occurrence: another linked occurrence of the same text on the line
/// cannot lend it structural meaning (§DF-index-entry-form.2.7).
fn link_target_at_citation<'a>(
    line: &'a str,
    links: LineLinks<'_>,
    citation: &Citation,
) -> Option<&'a str> {
    let start = citation.column.checked_sub(1)?;
    let end = start.checked_add(citation.text.len())?;
    if line.get(start..end)? != citation.text {
        return None;
    }
    // §FS-citations.wrap.recognized: a wrap enrolls; an authored link never does
    // (§FS-check.3.18.3), nor a citation-only image (§FS-citations.checks.image-index-entry).
    let CitationLink::Wrap(wrap) = citation_link(links, line, start, end) else {
        return None;
    };
    // §FS-fmt.2.3: a wrap whose `[` sits in a zone `fmt` never writes in is not one.
    if markdown_never_rewrite_context(links, line, start - 1) {
        return None;
    }
    (wrap.close > wrap.url_start).then(|| &line[wrap.url_start..wrap.close])
}
