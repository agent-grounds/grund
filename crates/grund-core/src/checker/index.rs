//! The kind-index invariant, in a file of its own beside the section and
//! reference families (§AR-checker.2.16, §AR-core-module-layout.1): a kind with a
//! `folder` and an `index` (§FS-config.3.4) promises that the index names every
//! declaration in that folder, and may enroll an external inline declaration by
//! canonical source link, as full Markdown links. §FS-check.3.18 is the coverage
//! half and §FS-check.3.17 the link half; this module owns both, plus
//! the set of citations they make navigational rather than referential
//! (§FS-check.4.1.2, §DF-index-not-an-inbound-citation).

use crate::model::remedies;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::index_entries::KindIndexEntries;
use super::index_form::{
    IndexCitationForm, IndexEntryState, index_citation_form, index_citation_form_at,
};
use super::index_mention::index_mentions;
use crate::config::{Frame, Owners, Schema};
use crate::grammar::{MarkdownLinks, front_matter_extent, render_catalog_id};
use crate::model::{
    Catalog, CheckReport, Citation, Code, Declaration, Diagnostic, Id, paired_declarations,
    scanned_decl_relative_path, scanned_path_key, section_resolves,
};
use crate::sources::{path_is_dir, physical_path_key};

/// One kind's index obligation, resolved against the config root
/// (§FS-config.3.4). `folder_key` and `index_key` are config-root-relative and
/// lexically normalized, so they compare against the paths the scanner recorded;
/// `index_file` is the readable path.
pub(super) struct KindIndexTarget<'a> {
    pub(super) kind: &'a str,
    /// The folder place the index lists, as written in the config.
    pub(super) folder: &'a str,
    pub(super) index_key: PathBuf,
    pub(super) index_file: PathBuf,
}

/// Every folder of a `[[kinds]]` entry that carries an enabled `index`
/// (§FS-config.3.4), one target per folder (§FS-config-v2.schema.places.ownership).
/// A kind with no folder, or with `index = false`, is absent from this list and is
/// therefore invisible to every rule below.
///
/// Why a non-`.md` `index` is dropped rather than reported: every rule below is
/// stated in terms of what `grund fmt --cross-refs` would write, and that pass
/// runs on `.md` files only (§FS-fmt.6.1). A non-Markdown index is a file the
/// formatter can never repair, and the honest report about one is no report at
/// all rather than a finding with no fix.
pub(super) fn kind_index_targets<'a>(
    schema: &'a Schema,
    frame: Frame<'_>,
) -> Vec<KindIndexTarget<'a>> {
    schema
        .rows
        .iter()
        .flat_map(|row| {
            row.index_paths()
                .into_iter()
                .map(move |(folder, index)| (row, folder, index))
        })
        .filter_map(|(row, folder, index)| {
            // §FS-config.3.4 rejects an `index` that does not name a `.md` file,
            // so this filter never fires on a config that loaded; it states the
            // Markdown assumption every rule below rests on.
            if index.extension().and_then(|ext| ext.to_str()) != Some("md") {
                return None;
            }
            Some(KindIndexTarget {
                kind: row.name.as_str(),
                folder,
                index_key: scanned_path_key(&index),
                index_file: frame.root().join(&index),
            })
        })
        .collect()
}

/// §FS-check.3.12.2: actual marked links owed by an index, rather than every
/// citation on that page. Membership is §AR-checker.2.16's shared derivation;
/// unlike unused accounting, a linked section can satisfy the entry rule too.
/// Form is read from the scan's file records, without another text read.
pub(super) fn marked_index_entries<'a>(
    findings: &'a Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    index_targets: &BTreeMap<(PathBuf, Id), String>,
) -> Vec<&'a Citation> {
    let entries = KindIndexEntries::new(findings, schema, frame, index_targets);
    let mut lines_by_file = BTreeMap::new();
    let mut links_by_file = BTreeMap::new();
    findings
        .edges()
        .filter(|cite| {
            // §FS-check.4.1.2.1: a citation on a declaration heading is the entry of no ID,
            // a linked section included, by the one answer (§AR-checker.2.16).
            if !cite.has_marker
                || cite.namespace.is_some()
                || entries.on_declaration_heading(&cite.file, cite.line)
                || !(entries.is_index_entry(cite)
                    || cite.section.is_some()
                        && entries
                            .entries_in(&cite.file)
                            .is_some_and(|ids| ids.contains(&cite.id)))
            {
                return false;
            }
            let Some(record) = findings.files.get(&cite.file) else {
                return false;
            };
            let lines = lines_by_file
                .entry(&cite.file)
                .or_insert_with(|| record.text().lines().collect::<Vec<_>>());
            let Some(line) = lines.get(cite.line.saturating_sub(1)) else {
                return false;
            };
            let links = links_by_file
                .entry(&cite.file)
                .or_insert_with(|| MarkdownLinks::new(record.text()));
            let Some(start) = cite.column.checked_sub(1) else {
                return false;
            };
            let Some(end) = start.checked_add(cite.text.len()) else {
                return false;
            };
            line.get(start..end) == Some(cite.text.as_str())
                && index_citation_form_at(
                    line,
                    links.line(cite.line - 1),
                    start,
                    end,
                    &schema.citation.marker,
                ) == IndexCitationForm::Link
        })
        .collect()
}

/// The declaration a finding about `id` points at — the same home `grund list`
/// and the unused warning pick, so a collapsed stub-and-inline pair is named at
/// the body rather than twice (§AR-scanner.4.6, §DF-index-entry-form.2.5), and a
/// stubbed declaration at its target whether or not the target is scanned
/// (§FS-declarations.stubs.pairing.4).
fn index_home_declaration(decls: &[Declaration]) -> Option<&Declaration> {
    paired_declarations(decls).records().next()
}

/// Whether any of `decls` sits in `folder` — the recursive membership test of
/// §FS-check.3.18.2: a stub in the folder is what puts an inline-homed ID in it,
/// and a folder's whole subtree counts, not its top level, short of a nested
/// place that owns the file (§FS-config-v2.schema.places.ownership).
pub(super) fn declarations_under_folder(
    decls: &[Declaration],
    folder: &str,
    owners: &Owners<'_>,
) -> bool {
    decls.iter().any(|decl| {
        owners
            .relative(&decl.file)
            .is_some_and(|relative| owners.schema().folder_owns(folder, &relative))
    })
}

/// The two releases §FS-check.3.17.3's message names
/// (§DF-index-compatibility-ramp.2.3) — the pair
/// §REQ-backwards-compatibility.3 asks of a verdict that flipped in one
/// release. Both are literals in message text, so both are part of the release
/// process: bumping the workspace version is also the moment to ask whether
/// they still say what they mean (§FS-distribution.4.5), and
/// `index_rule_releases_are_ordered_and_behind_us` below holds them to it.
///
/// §FS-check.3.18.9's message names a third, the release its own ramp ended in.
/// That one is written into the message text rather than kept here, the way the
/// `[[kinds]] prefix` removal writes its own (§FS-config.3.4.6): a landed
/// release is read back out of the line a user sees, by the release gate that
/// refuses a version contradicting it (§FS-distribution.4.2.2).

/// The last release before `check` knew anything about a kind's index — the
/// "from" half of the pair §REQ-backwards-compatibility.3 requires a
/// verdict-flipping finding to name.
pub(crate) const INDEX_RULE_PRIOR_RELEASE: &str = "0.11.0";

/// The release the kind-index rules arrive in, and in which §FS-check.3.17.3 is an
/// error on arrival — the "to" half of that pair.
pub(crate) const INDEX_RULE_RELEASE: &str = "0.12.0";

/// §AR-checker.2.16 — the kind-index rule (§FS-check.3.18, §FS-check.3.17). One
/// pass per configured index: read the file once, classify the citations the
/// scanner already recorded in it, then judge each declaration the index owns.
/// It re-reads the index file, and a stub's target the walk did not reach where an
/// entry cites a section no recorded declaration holds (§AR-checker.placement).
///
/// Why the run's own scan decides which indexes are judged: a run that cannot see
/// the index does not get to judge it — a narrowed `grund check <one-file>`, or an
/// index the `[scan]` set excludes, is not evidence about that index. An index file
/// that does not exist is a different fact, and is still reported.
///
/// Why the three ways an index fails to read are named apart: "does not exist",
/// said about a directory that plainly does, is a diagnosis a reader has to argue
/// with.
///
/// Why only the bare form is gated on the cited section resolving and the index
/// target staying inside the physical config root: §FS-check.3.17.4 is the finding
/// that names `grund fmt --write`, so it may only reach an occurrence the pass
/// would in fact rewrite, while a link already written stands whatever `fmt`
/// would do with it (§DF-index-entry-form.2.4). The tree is already red for the
/// unresolved section itself (§FS-check.3.2); an external file-symlink target is
/// outside the formatter's write boundary (§FS-fmt.2.3.2). In either case the ID
/// falls to §FS-check.3.18's error, whose fix is an edit.
///
/// Why the unlinked-entry message names `grund fmt --write`: that command is only
/// ever named on a site the pass will in fact rewrite, which is what
/// `IndexCitationForm::Bare` is narrowed to mean.
pub(super) fn check_kind_indexes(
    findings: &Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    index_targets: &BTreeMap<(PathBuf, Id), String>,
    report: &mut CheckReport,
) {
    let targets = kind_index_targets(schema, frame);
    if targets.is_empty() {
        return;
    }
    let configured_root = scanned_path_key(frame.root());
    let physical_root = physical_path_key(frame.root());
    // §FS-check.3.18: folder declarations plus the external inline declarations
    // their canonical index links enroll. `KindIndexEntries` is also what `fmt`
    // and the unused-accounting surfaces read, so membership has one derivation.
    let index_entries = KindIndexEntries::new(findings, schema, frame, index_targets);

    // One pass over the citations, bucketed by index file, so the per-kind loop
    // below is a lookup rather than another walk of the whole citation list
    // (§GOAL-fast-feedback).
    let index_keys: BTreeSet<&Path> = targets
        .iter()
        .map(|target| target.index_key.as_path())
        .collect();
    let mut cited_in_index: BTreeMap<&Path, Vec<&Citation>> = BTreeMap::new();
    for citation in &findings.citations {
        if citation.namespace.is_some() {
            continue;
        }
        let Some(relative) =
            scanned_decl_relative_path(&citation.file, &configured_root, &physical_root)
        else {
            continue;
        };
        if let Some(key) = index_keys.get(relative.as_ref()) {
            cited_in_index.entry(key).or_default().push(citation);
        }
    }
    // §FS-check.3.18.8: which index files *this run* read. The entries come from the
    // scan and the form from the index's text, so an index the run never scanned
    // would otherwise look empty and report every declaration in the folder as unlisted.
    let index_scanned: BTreeSet<&Path> = findings
        .scanned_files
        .iter()
        .filter_map(|file| {
            let relative = scanned_decl_relative_path(file, &configured_root, &physical_root)?;
            index_keys.get(relative.as_ref()).copied()
        })
        .collect();

    // §AR-system.2.13: the index as a save would write it.
    let sources = frame.sources();
    for target in &targets {
        // `is_file`, not `exists`: a path that is not a readable file is not an index
        // this run failed to read, it is an index that is not there — a missing one or
        // a directory wearing the name — which is §FS-check.3.18.7's finding, not silence.
        if !index_scanned.contains(target.index_key.as_path())
            && sources.is_file(&target.index_file)
        {
            continue;
        }
        let Some(owed) = index_entries.entries_in(&target.index_file) else {
            continue;
        };
        let covered: Vec<(&Id, &Declaration)> = findings
            .declarations
            .iter()
            .filter(|(id, _)| id.kind == target.kind)
            .filter(|(id, _)| owed.contains(*id))
            .filter_map(|(id, decls)| Some((id, index_home_declaration(decls)?)))
            .collect();
        if covered.is_empty() {
            continue;
        }
        let index_display = frame.display_path(&target.index_file);
        // §FS-check.3.18.7: an absent index is reported once per declaration;
        // distinguish missing files, directories and unreadable files.
        // §FS-check.6.1.1: cover this effective input before its shared read.
        let text = sources.text(&target.index_file).ok();
        let absent = match &text {
            Some(_) => "",
            None if path_is_dir(&target.index_file) => " (the index file is a directory)",
            None if sources.exists(&target.index_file) => " (the index file could not be read)",
            None => " (the index file does not exist)",
        };
        let lines: Vec<&str> = text
            .as_deref()
            .map(|text| text.lines().collect())
            .unwrap_or_default();
        // §FS-citations.wrap: the index is the document a reference is read in.
        let links = MarkdownLinks::new(text.as_deref().unwrap_or_default());
        // §FS-check.3.17.4: `fmt` writes no link in front matter (§FS-check.1.1.5.2).
        let front_matter = front_matter_extent(lines.iter().copied());
        // §FS-check.3.17.4: a bare citation is an entry only where `fmt --write`
        // may wrap it. A link through this index path to a target outside the
        // physical config root is readable but not writable by the formatter.
        let bare_repairable = physical_path_key(&target.index_file).starts_with(&physical_root);
        let mut entries: BTreeMap<&Id, IndexEntryState> = BTreeMap::new();
        for citation in cited_in_index
            .get(target.index_key.as_path())
            .map(Vec::as_slice)
            .unwrap_or_default()
        {
            let Some(line) = lines.get(citation.line.saturating_sub(1)) else {
                continue;
            };
            // §FS-fmt.6.4: `fmt` leaves a declaration heading alone, and a citation on one
            // is the entry of no ID (§FS-check.4.1.2.1), by the answer the unused carve-out
            // reads too (§AR-checker.2.16). A fence records no citation.
            if index_entries.on_declaration_heading(&citation.file, citation.line) {
                continue;
            }
            // An `Ignored` form creates no entry: a citation `fmt` will not wrap
            // neither satisfies the rule nor triggers §FS-check.3.17.5
            // (§DF-index-entry-form.2.3), so the ID is reported as unlisted.
            let form = index_citation_form(
                line,
                links.line(citation.line.saturating_sub(1)),
                &citation.text,
                &schema.citation.marker,
            );
            // §FS-check.3.2.1: §FS-check.3.2's own test, so a stub's section in a
            // target outside the walk admits the entry.
            let section_known = citation
                .section
                .as_deref()
                .is_none_or(|section| section_resolves(findings, &citation.id, section));
            // §FS-fmt.6.4.1: `fmt` skips a section citation with no link target and
            // reports `rewrote 0 lines`. The physical-root predicate above is the
            // other §FS-check.3.17.4 gate; only the bare form is gated.
            let in_front_matter = front_matter.is_some_and(|last| citation.line <= last);
            let form = if form == IndexCitationForm::Bare
                && (!bare_repairable || !section_known || in_front_matter)
            {
                IndexCitationForm::Ignored
            } else {
                form
            };
            if form == IndexCitationForm::Ignored {
                continue;
            }
            entries
                .entry(&citation.id)
                .or_default()
                .record(form, citation.line, citation.column);
        }

        // §FS-check.3.18.5.1: one look at the index text, only where a finding is
        // owed. An unreadable file leaves `lines` empty, so nothing appears in it
        // and §FS-check.3.18.7's parenthesis stands alone.
        let mut unentered: Vec<(&Id, &Declaration)> = Vec::new();
        for (id, decl) in &covered {
            if !entries.contains_key(id) {
                unentered.push((*id, *decl));
            }
        }
        let mentioned = if unentered.is_empty() {
            BTreeSet::new()
        } else {
            index_mentions(
                frame,
                &schema.citation.marker,
                |line| index_entries.on_declaration_heading(&target.index_file, line),
                &lines,
                &unentered,
            )
        };

        for (id, decl) in covered {
            match entries.get(id) {
                // §FS-check.3.17: an entry that exists and is not a link, at the
                // line `grund fmt --write` rewrites.
                Some(state) if !state.linked => {
                    let (line, column) = state.first_bare.unwrap_or((1, 1));
                    report.errors.push(Diagnostic {
                        code: Code::UNLINKED_INDEX_ENTRY,
                        path: Some(target.index_file.clone()),
                        line: Some(line),
                        column: Some(column),
                        // §REQ-backwards-compatibility.3 wants all three: the
                        // versions the verdict moved between, one command the
                        // tool ships, and a release note. The first two are here.
                        message: format!(
                            "index entry {}{} is not a link; unchecked in grund {INDEX_RULE_PRIOR_RELEASE}, an error in {INDEX_RULE_RELEASE} — {}",
                            schema.citation.marker,
                            render_catalog_id(frame.grammar(), &findings.declarations, id),
                            remedies::unlinked_index_entry()
                        ),
                        sites: Vec::new(),
                    authority: Vec::new(),});
                }
                Some(_) => {}
                // §FS-check.3.18.7: no entry at all, anchored at the declaration's
                // own heading — the one line that exists whether or not the
                // index file does (§DF-index-entry-form.2.6).
                None => {
                    // §FS-check.3.18.5.1: "is not listed", said about a line the
                    // reader is looking at, is a diagnosis the reader argues with.
                    // Both forms keep §FS-distribution.4.2.3's past-tense release.
                    let rendered = render_catalog_id(frame.grammar(), &findings.declarations, id);
                    let form = entry_form(schema);
                    let message = if mentioned.contains(id) {
                        format!(
                            "{rendered} appears in {index_display} but not as an entry: {form} — became an error in grund 0.13.0"
                        )
                    } else {
                        // §FS-check.3.18.5.2: the form, then the entry `fmt` would
                        // write here, or the form alone where `fmt` derives no link.
                        let entry = index_targets
                            .get(&(target.index_file.clone(), id.clone()))
                            .map(|link| {
                                format!(
                                    "; {}",
                                    remedies::missing_index_entry(
                                        format!("[{}{rendered}]({link})", schema.citation.marker),
                                        &index_display
                                    )
                                )
                            })
                            .unwrap_or_default();
                        format!(
                            "{rendered} is not listed in {index_display}{absent}: {form}{entry} — became an error in grund 0.13.0"
                        )
                    };
                    report.errors.push(Diagnostic {
                        code: Code::MISSING_INDEX_ENTRY,
                        path: Some(decl.file.clone()),
                        line: Some(decl.line),
                        column: None,
                        message,
                        sites: Vec::new(),
                        authority: Vec::new(),
                    });
                }
            }
        }
    }
}

/// The form an index entry takes, the one clause both wordings of
/// `missing-index-entry` name (§FS-check.3.18.5, §FS-check.3.18.5.2). It follows
/// `[reference] strict`, so it is true of the index it is said about.
fn entry_form(schema: &Schema) -> String {
    let marker = &schema.citation.marker;
    if schema.citation.strict {
        format!("an entry is a `{marker}`-marked Markdown link to the declaration")
    } else {
        format!(
            "an entry is a Markdown link to the declaration whose text is the ID, with or without the `{marker}` marker"
        )
    }
}

/// The configured index files, as a path-membership test (§FS-config.3.4).
/// `grund fmt` reads it to keep the cross-reference pass running over an index
/// whatever `[fmt.cross_refs] enabled` says (§FS-fmt.6.1.1,
/// §DF-index-always-linkified) — the one region the formatter always writes,
/// mirroring §FS-fmt.2.3's regions it never writes.
pub(crate) struct KindIndexFiles {
    configured_root: PathBuf,
    physical_root: PathBuf,
    keys: BTreeSet<PathBuf>,
}

impl KindIndexFiles {
    pub(crate) fn new(schema: &Schema, frame: Frame<'_>) -> Self {
        Self {
            configured_root: scanned_path_key(frame.root()),
            physical_root: physical_path_key(frame.root()),
            keys: kind_index_targets(schema, frame)
                .into_iter()
                .map(|target| target.index_key)
                .collect(),
        }
    }

    /// The set a run that already linkifies everything needs — no config read,
    /// no allocation, and `contains` short-circuits on it.
    pub(crate) fn empty() -> Self {
        Self {
            configured_root: PathBuf::new(),
            physical_root: PathBuf::new(),
            keys: BTreeSet::new(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub(crate) fn contains(&self, path: &Path) -> bool {
        if self.keys.is_empty() {
            return false;
        }
        scanned_decl_relative_path(path, &self.configured_root, &self.physical_root)
            .is_some_and(|relative| self.keys.contains(relative.as_ref()))
    }
}
