//! The membership half of the kind-index invariant (§AR-checker.2.16), split
//! from `index.rs` so adding external enrollment does not push that rule
//! family past the core-source file budget (§AR-core-module-layout.3).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::index::{KindIndexTarget, declarations_under_folder, kind_index_targets};
use crate::config::{Frame, Owners, Schema};
use crate::grammar::{
    CitationLink, LineLinks, MarkdownLinks, citation_link, markdown_never_rewrite_context,
};
use crate::model::{
    Catalog, Citation, Declaration, HeadingKind, Id, paired_declarations,
    scanned_decl_relative_path, scanned_path_key,
};
use crate::sources::{Sources, physical_path_key};

/// The IDs each kind index owes an entry for, keyed by the index's
/// config-root-relative path (§FS-check.3.18). `folder_owed` preserves the
/// original folder-membership rule; `owed` additionally holds external inline
/// declarations enrolled by a canonical link. Their exact sites stay separate
/// so another citation of the same external ID on the page remains real use
/// (§DF-index-not-an-inbound-citation.2.2). `binding_sites` holds the citation
/// sites of the recognized value bindings on each index, which are use rather
/// than the entry even for a folder-owned ID (§FS-check.4.1.2.1). `headings`
/// holds each index's declaration heading lines, whose citations are the entry
/// of no ID (§FS-check.4.1.2.1, §FS-citations.recognized.title-lines).
pub(crate) struct KindIndexEntries {
    configured_root: PathBuf,
    physical_root: PathBuf,
    owed: BTreeMap<PathBuf, BTreeSet<Id>>,
    folder_owed: BTreeMap<PathBuf, BTreeSet<Id>>,
    external_sites: BTreeMap<PathBuf, BTreeSet<(usize, usize)>>,
    binding_sites: BTreeMap<PathBuf, BTreeSet<(usize, usize)>>,
    headings: IndexHeadings,
}

impl KindIndexEntries {
    /// `index_targets` is `Expected`'s canonical link per (index file, ID), the
    /// destination an external enrollment is compared against (§AR-checker.2.16).
    pub(crate) fn new(
        findings: &Catalog,
        schema: &Schema,
        frame: Frame<'_>,
        index_targets: &BTreeMap<(PathBuf, Id), String>,
    ) -> Self {
        Self::build(findings, schema, frame, index_targets, None)
    }

    /// The same classifier, owing entries only for `ids`: `is_index_entry`
    /// answers a citation of one of them exactly as `new`'s would, without
    /// walking every declaration of the kind (§GOAL-fast-feedback).
    pub(crate) fn restricted(
        findings: &Catalog,
        schema: &Schema,
        frame: Frame<'_>,
        index_targets: &BTreeMap<(PathBuf, Id), String>,
        ids: &BTreeSet<Id>,
    ) -> Self {
        Self::build(findings, schema, frame, index_targets, Some(ids))
    }

    fn build(
        findings: &Catalog,
        schema: &Schema,
        frame: Frame<'_>,
        index_targets: &BTreeMap<(PathBuf, Id), String>,
        only: Option<&BTreeSet<Id>>,
    ) -> Self {
        let configured_root = scanned_path_key(frame.root());
        let physical_root = physical_path_key(frame.root());
        let targets = kind_index_targets(schema, frame);
        let owners = Owners::new(schema, frame);
        let mut folder_owed: BTreeMap<PathBuf, BTreeSet<Id>> = BTreeMap::new();
        for target in &targets {
            for (id, decls) in &findings.declarations {
                if id.kind == target.kind
                    && only.is_none_or(|ids| ids.contains(id))
                    && declarations_under_folder(decls, target.folder, &owners)
                {
                    folder_owed
                        .entry(target.index_key.clone())
                        .or_default()
                        .insert(id.clone());
                }
            }
        }
        let mut owed = folder_owed.clone();
        let headings = IndexHeadings::new(findings, &targets, &configured_root, &physical_root);
        let external_sites = enroll_external_inline_declarations(
            findings,
            &targets,
            &owners,
            &configured_root,
            &physical_root,
            index_targets,
            frame.sources(),
            only,
            &headings,
            &mut owed,
        );
        let binding_sites =
            index_value_binding_sites(findings, &folder_owed, &configured_root, &physical_root);
        Self {
            configured_root,
            physical_root,
            owed,
            folder_owed,
            external_sites,
            binding_sites,
            headings,
        }
    }

    /// The IDs this index owes an entry for, or `None` when `path` is not a
    /// configured index (§FS-fmt.6.1.2). What the always-linkify carve-out wraps:
    /// declarations under the folder plus an external inline ID only after its
    /// canonical enrollment link exists (§FS-check.3.18).
    pub(crate) fn entries_in(&self, path: &Path) -> Option<&BTreeSet<Id>> {
        if self.owed.is_empty() {
            return None;
        }
        let relative =
            scanned_decl_relative_path(path, &self.configured_root, &self.physical_root)?;
        self.owed.get(relative.as_ref())
    }

    /// §FS-check.4.1.2 / §DF-index-not-an-inbound-citation.2.2: folder-owned IDs
    /// keep PR #134's accounting. For an external ID only the canonical link site
    /// is navigation; a second same-ID citation in the index is ordinary use.
    /// §FS-check.4.1.2.1: a folder-owned ID's entry is a citation of the
    /// declaration itself, so a section citation, or the citation inside a
    /// recognized value binding (a root-aimed one with no section included), is use.
    /// So is a citation on a declaration heading, for either branch: it is a
    /// citation from the declaration that heading declares, and it enrolled nothing
    /// (§FS-citations.recognized.title-lines, §FS-check.3.18.3).
    pub(crate) fn is_index_entry(&self, citation: &Citation) -> bool {
        if citation.namespace.is_some() || self.owed.is_empty() {
            return false;
        }
        let Some(relative) =
            scanned_decl_relative_path(&citation.file, &self.configured_root, &self.physical_root)
        else {
            return false;
        };
        if self.headings.holds(relative.as_ref(), citation.line) {
            return false;
        }
        let site = (citation.line, citation.column);
        (citation.section.is_none()
            && self
                .folder_owed
                .get(relative.as_ref())
                .is_some_and(|ids| ids.contains(&citation.id))
            && !self
                .binding_sites
                .get(relative.as_ref())
                .is_some_and(|sites| sites.contains(&site)))
            || self
                .external_sites
                .get(relative.as_ref())
                .is_some_and(|sites| sites.contains(&site))
    }

    /// Whether line `line` of the kind index at `path` is a declaration heading:
    /// the one answer the entry rule, its mention look, the unused carve-out, `list
    /// --unused` and the `refs` label all read (§AR-checker.2.16). A citation on one
    /// is the entry of no ID (§FS-check.4.1.2.1), and the line declares an ID rather
    /// than mentioning one (§FS-check.3.18.5.1).
    pub(super) fn on_declaration_heading(&self, path: &Path, line: usize) -> bool {
        scanned_decl_relative_path(path, &self.configured_root, &self.physical_root)
            .is_some_and(|relative| self.headings.holds(relative.as_ref(), line))
    }
}

/// The lines of each kind index the scan read as a declaration heading, keyed by
/// the index's config-root-relative path (§AR-checker.2.16). They are read off the
/// index's file record, the record of the text every look at the index reads
/// (§AR-scanner.2.8.5), so the line is not parsed again. The scan records no
/// heading in a fence or in a raw-text HTML block (§FS-check.1.1.5.1), so a
/// recorded line needs no second test of its own.
struct IndexHeadings(BTreeMap<PathBuf, BTreeSet<usize>>);

impl IndexHeadings {
    fn new(
        findings: &Catalog,
        targets: &[KindIndexTarget<'_>],
        configured_root: &Path,
        physical_root: &Path,
    ) -> Self {
        let mut lines: BTreeMap<PathBuf, BTreeSet<usize>> = BTreeMap::new();
        if targets.is_empty() {
            return Self(lines);
        }
        let keys: BTreeSet<&Path> = targets
            .iter()
            .map(|target| target.index_key.as_path())
            .collect();
        for (file, record) in &findings.files {
            let Some(relative) = scanned_decl_relative_path(file, configured_root, physical_root)
            else {
                continue;
            };
            if !keys.contains(relative.as_ref()) {
                continue;
            }
            lines.entry(relative.into_owned()).or_default().extend(
                record
                    .headings
                    .iter()
                    .filter(|heading| heading.kind == HeadingKind::Declaration)
                    .map(|heading| heading.line),
            );
        }
        Self(lines)
    }

    /// Whether line `line` of the index at config-root-relative `index` is one.
    fn holds(&self, index: &Path, line: usize) -> bool {
        self.0.get(index).is_some_and(|lines| lines.contains(&line))
    }
}

/// The citation sites of the scanner's recognized value bindings on each kind
/// index (§FS-check.4.1.2.1, §FS-values.3.2), read off the catalog's binding
/// records rather than re-parsed.
fn index_value_binding_sites(
    findings: &Catalog,
    folder_owed: &BTreeMap<PathBuf, BTreeSet<Id>>,
    configured_root: &Path,
    physical_root: &Path,
) -> BTreeMap<PathBuf, BTreeSet<(usize, usize)>> {
    let mut sites: BTreeMap<PathBuf, BTreeSet<(usize, usize)>> = BTreeMap::new();
    for binding in &findings.value_bindings {
        if binding.namespace.is_some() {
            continue;
        }
        let Some(relative) =
            scanned_decl_relative_path(&binding.file, configured_root, physical_root)
        else {
            continue;
        };
        if folder_owed.contains_key(relative.as_ref()) {
            sites
                .entry(relative.into_owned())
                .or_default()
                .insert((binding.line, binding.citation_column));
        }
    }
    sites
}

/// Add every canonical external-inline enrollment to `owed`, returning the
/// exact citation sites that are navigation rather than use (§FS-check.3.18.3).
/// Citations are already scanner records; index text is read once per target
/// solely to inspect the persisted Markdown wrapper and destination.
fn enroll_external_inline_declarations(
    findings: &Catalog,
    targets: &[KindIndexTarget<'_>],
    owners: &Owners<'_>,
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
        // §FS-check.3.18.3: the link that enrolls is the entry that satisfies, and a
        // citation on a declaration heading is no entry (§FS-check.4.1.2.1).
        if headings.holds(relative.as_ref(), citation.line) {
            continue;
        }
        let Some(target) = targets_by_index.get(relative.as_ref()).and_then(|targets| {
            targets
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
        if external_inline_home(decls, target.folder, owners, &markdown).is_none() {
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
    markdown: &dyn Fn(&Path) -> bool,
) -> Option<&'a Declaration> {
    if declarations_under_folder(decls, folder, owners) {
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
