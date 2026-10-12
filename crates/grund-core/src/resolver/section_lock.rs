//! What each section lock should hold, and what it holds that the tree no longer
//! bears out (§FS-section-lock): one judgement of the scan's catalog that `check`
//! reports from and `fmt --write` writes from, so the two cannot disagree on an
//! entry. A function of the loaded findings, because the fingerprint is the text
//! `show` prints (§FS-section-lock.holds.fingerprint) and a citation's target is
//! the resolver's answer (§AR-system.2.10).

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use super::point_body::point_body_pair;
use crate::config::{Frame, Schema};
use crate::grammar::render_id;
use crate::model::{
    Catalog, Citation, Declaration, Id, LockEntry, LockLines, paired_declarations, read_lock_lines,
    section_lock_path,
};
use crate::sources::sha256_hex;

/// The fingerprint of a section's text (§FS-section-lock.holds.fingerprint): the
/// first sixteen lowercase hex digits of the SHA-256 of `text` with LF line
/// endings, each line's trailing spaces and tabs removed, leading and trailing
/// empty lines dropped, and the lines joined with LF and no final LF.
pub(crate) fn section_fingerprint(text: &str) -> String {
    let lines = text
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .map(|line| line.trim_end_matches([' ', '\t']))
        .collect::<Vec<_>>();
    let first = lines.iter().position(|line| !line.is_empty());
    let last = lines.iter().rposition(|line| !line.is_empty());
    let normalized = match (first, last) {
        (Some(first), Some(last)) => lines[first..=last].join("\n"),
        _ => String::new(),
    };
    sha256_hex(normalized.as_bytes())[..16].to_string()
}

/// What `fmt --write` does with one entry the lock holds (§FS-section-lock.write.rules).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EntryVerdict {
    /// It holds what the tree needs.
    Keep,
    /// Its section was edited in place, or a pointer acknowledged its move: the
    /// fingerprint to record, `None` for `-` (§FS-section-lock.move.pointer).
    Refresh(Option<String>),
    /// Nothing cites its address, a relative heading no longer holds it, or an
    /// earlier line enters the same coordinate.
    Drop,
    /// Its recorded text sits at `to`, another section of the declaration, and
    /// nothing acknowledged the move (§FS-section-lock.checks.section-address-changed).
    Moved { to: String },
}

/// One lock file judged against the tree: the one that exists, the one a cited
/// relative address needs, or both (§FS-section-lock.file.shard).
pub(crate) struct LockJudgement<'a> {
    /// The coordinate prefix: the ID as `[id] format` renders it.
    pub(crate) rendered: String,
    pub(crate) path: PathBuf,
    /// Its text as read, `None` where no file exists.
    pub(crate) text: Option<&'a str>,
    pub(crate) lines: LockLines,
    /// The declaration the ID pairs to, `None` where it pairs to none or several.
    pub(crate) decl: Option<&'a Declaration>,
    /// Each entry the file holds, in file order, with what becomes of it.
    pub(crate) verdicts: Vec<(LockEntry, EntryVerdict)>,
    /// Each cited relative address the file holds no entry for, with its
    /// citations and the fingerprint to record, `None` for `-`.
    pub(crate) unrecorded: Vec<(String, Vec<&'a Citation>, Option<String>)>,
}

impl LockJudgement<'_> {
    /// The entries the file holds once `fmt --write` has run, as
    /// `(coordinate, fingerprint)` pairs (§FS-section-lock.write.rules).
    pub(crate) fn written_entries(&self, separator: &str) -> Vec<(String, Option<String>)> {
        let coordinate = |section: &str| format!("{}{separator}{section}", self.rendered);
        let kept = self
            .verdicts
            .iter()
            .filter_map(|(entry, verdict)| match verdict {
                EntryVerdict::Keep | EntryVerdict::Moved { .. } => {
                    Some((coordinate(&entry.section), entry.fingerprint.clone()))
                }
                EntryVerdict::Refresh(fingerprint) => {
                    Some((coordinate(&entry.section), fingerprint.clone()))
                }
                EntryVerdict::Drop => None,
            });
        let added = self
            .unrecorded
            .iter()
            .map(|(section, _, fingerprint)| (coordinate(section), fingerprint.clone()));
        kept.chain(added).collect()
    }
}

/// Judge every lock file the scan read and every one a cited relative address
/// needs, in path order (§FS-section-lock). A citation into another project is
/// not judged here. Empty, and nothing sliced, in a tree with neither.
pub(crate) fn judge_section_locks<'a>(
    findings: &'a Catalog,
    schema: &Schema,
    frame: Frame<'_>,
) -> Vec<LockJudgement<'a>> {
    // §FS-section-lock: a tree with no lock and no relative heading reads none.
    let relative = |decl: &Declaration| decl.sections.values().any(|info| info.relative);
    if findings.section_locks.is_empty() && !findings.declarations.values().flatten().any(relative)
    {
        return Vec::new();
    }
    let grammar = frame.grammar();
    let separator = schema.ids.section_separator.as_str();
    let mut cited: BTreeMap<&Id, BTreeMap<&str, Vec<&Citation>>> = BTreeMap::new();
    for citation in &findings.citations {
        let (None, Some(section)) = (&citation.namespace, &citation.section) else {
            continue;
        };
        if sole_declaration(findings, &citation.id)
            .and_then(|decl| decl.sections.get(section))
            .is_some_and(|info| info.relative)
        {
            cited
                .entry(&citation.id)
                .or_default()
                .entry(section.as_str())
                .or_default()
                .push(citation);
        }
    }
    let mut ids: BTreeMap<String, Option<&Id>> = cited
        .keys()
        .map(|id| (render_id(grammar, id), Some(*id)))
        .collect();
    let mut texts = BTreeMap::new();
    for lock in &findings.section_locks {
        texts.insert(lock.id.as_str(), lock.text.as_str());
        ids.entry(lock.id.clone()).or_insert_with(|| {
            findings
                .declarations
                .keys()
                .find(|id| render_id(grammar, id) == lock.id)
        });
    }
    let mut judgements = Vec::new();
    for (rendered, id) in ids {
        let decl = id.and_then(|id| sole_declaration(findings, id));
        let text = texts.get(rendered.as_str()).copied();
        let lines = text.map_or_else(LockLines::default, |text| {
            read_lock_lines(text, &rendered, separator)
        });
        let cited = id.and_then(|id| cited.remove(id)).unwrap_or_default();
        let fingerprints = decl.map_or_else(Vec::new, |decl| {
            fingerprints(findings, schema, frame, id.expect("a paired ID"), decl)
        });
        let (verdicts, unrecorded) = judge_entries(findings, decl, &lines, cited, &fingerprints);
        judgements.push(LockJudgement {
            path: section_lock_path(frame.root(), &rendered),
            rendered,
            text,
            lines,
            decl,
            verdicts,
            unrecorded,
        });
    }
    judgements.sort_by(|a, b| a.path.cmp(&b.path));
    judgements
}

/// The one declaration `id` pairs to, which is the one its citations resolve to
/// (§FS-declarations.stubs.verdict).
fn sole_declaration<'a>(findings: &'a Catalog, id: &Id) -> Option<&'a Declaration> {
    let sole = paired_declarations(findings.declarations.get(id)?).sole()?;
    Some(sole.record)
}

/// Every section of `decl` with its line and fingerprint, in document order.
fn fingerprints(
    findings: &Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    id: &Id,
    decl: &Declaration,
) -> Vec<(String, usize, String)> {
    let mut sections = decl
        .sections
        .iter()
        .filter_map(|(path, info)| {
            let (_, full) =
                point_body_pair(&findings.files, schema, frame, id, decl, Some((path, info)))
                    .ok()??;
            // §FS-section-lock.holds.fingerprint: the text below the heading line.
            let below = full.split_once('\n').map_or("", |(_, below)| below);
            Some((path.clone(), info.line, section_fingerprint(below)))
        })
        .collect::<Vec<_>>();
    sections.sort_by_key(|(_, line, _)| *line);
    sections
}

type Judged<'a> = (
    Vec<(LockEntry, EntryVerdict)>,
    Vec<(String, Vec<&'a Citation>, Option<String>)>,
);

/// Each entry's verdict, and each cited address left without one
/// (§FS-section-lock.write.rules, §FS-section-lock.write.never-a-move).
fn judge_entries<'a>(
    findings: &Catalog,
    decl: Option<&Declaration>,
    lines: &LockLines,
    mut cited: BTreeMap<&str, Vec<&'a Citation>>,
    fingerprints: &[(String, usize, String)],
) -> Judged<'a> {
    let current = |section: &str| {
        fingerprints
            .iter()
            .find(|(path, _, _)| path == section)
            .map(|(_, _, fingerprint)| fingerprint.as_str())
    };
    // §FS-section-lock.holds.repeated: a text the declaration repeats is `-`.
    let written = |fingerprint: &str| {
        let holders = fingerprints
            .iter()
            .filter(|(_, _, other)| other == fingerprint);
        (holders.count() == 1).then(|| fingerprint.to_string())
    };
    // §FS-section-lock.write.rebuild: of a coordinate entered twice, the entry
    // the section holds now, or else the first.
    let holds = |entry: &LockEntry| {
        entry.fingerprint.is_some() && entry.fingerprint.as_deref() == current(&entry.section)
    };
    let mut chosen: BTreeMap<&str, usize> = BTreeMap::new();
    for (index, entry) in lines.entries.iter().enumerate() {
        let first = *chosen.entry(entry.section.as_str()).or_insert(index);
        if !holds(&lines.entries[first]) && holds(entry) {
            chosen.insert(entry.section.as_str(), index);
        }
    }
    // Read before the loop takes entries out of `cited`: a second line of one
    // coordinate is still a line of a cited one.
    let was_cited: BTreeSet<&str> = cited.keys().copied().collect();
    let mut verdicts = Vec::new();
    for (index, entry) in lines.entries.iter().enumerate() {
        let section = entry.section.as_str();
        let held = decl
            .and_then(|decl| decl.sections.get(section))
            .is_some_and(|info| info.relative);
        let verdict = match (current(section), held && was_cited.contains(section)) {
            _ if chosen.get(section) != Some(&index) => EntryVerdict::Drop,
            // A text no slice could read is no evidence of an edit or a move.
            (None, true) => EntryVerdict::Keep,
            (Some(now), true) => match entry.fingerprint.as_deref() {
                None => match written(now) {
                    Some(fingerprint) => EntryVerdict::Refresh(Some(fingerprint)),
                    None => EntryVerdict::Keep,
                },
                Some(recorded) if recorded == now => EntryVerdict::Keep,
                Some(recorded) => {
                    let moved_to = fingerprints
                        .iter()
                        .find(|(path, _, fingerprint)| path != section && fingerprint == recorded)
                        .map(|(path, _, _)| path.clone());
                    match moved_to {
                        Some(to) if !points_at(findings, decl, section, &to) => {
                            EntryVerdict::Moved { to }
                        }
                        // Edited in place, or a move a pointer acknowledged.
                        _ => EntryVerdict::Refresh(written(now)),
                    }
                }
            },
            _ => EntryVerdict::Drop,
        };
        cited.remove(section);
        verdicts.push((entry.clone(), verdict));
    }
    let unrecorded = cited
        .into_iter()
        .map(|(section, citations)| {
            let fingerprint = current(section).and_then(written);
            (section.to_string(), citations, fingerprint)
        })
        .collect();
    (verdicts, unrecorded)
}

/// Whether the section at `section` cites `to`: a pointer that acknowledges the
/// move (§FS-section-lock.move.pointer).
fn points_at(findings: &Catalog, decl: Option<&Declaration>, section: &str, to: &str) -> bool {
    let Some(decl) = decl else {
        return false;
    };
    let Some(info) = decl.sections.get(section) else {
        return false;
    };
    let end = findings
        .files
        .get(&decl.file)
        .and_then(|record| record.heading_at(info.line))
        .map_or(info.line, |heading| heading.end);
    findings.citations.iter().any(|citation| {
        citation.namespace.is_none()
            && citation.id == decl.id
            && citation.section.as_deref() == Some(to)
            && citation.file == decl.file
            && info.line < citation.line
            && citation.line <= end
    })
}

#[cfg(test)]
mod tests {
    use super::section_fingerprint;

    #[test]
    fn the_fingerprint_is_the_digest_of_the_normalized_text() {
        assert_eq!(
            section_fingerprint("Trim surrounding whitespace."),
            "8f332b6fe87bb346"
        );
        assert_eq!(section_fingerprint("Split on commas."), "be879e46c73e3d05");
    }

    #[test]
    fn line_endings_and_trailing_blanks_do_not_change_it() {
        let plain = section_fingerprint("One.\nTwo.");
        assert_eq!(section_fingerprint("\r\nOne.  \r\nTwo.\t\r\n\r\n"), plain);
        assert_eq!(section_fingerprint("\n\nOne.\nTwo.\n"), plain);
        assert_ne!(section_fingerprint("One.\n\nTwo."), plain);
    }
}
