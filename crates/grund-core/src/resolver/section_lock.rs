//! What each section lock should hold, and what it holds that the tree no longer
//! bears out (§FS-section-lock): one judgement of the scan's catalog that `check`
//! reports from and `fmt --write` writes from, so the two cannot disagree on an
//! entry. A function of the loaded findings, because the fingerprint is the text
//! `show` prints (§FS-section-lock.holds.fingerprint) and a citation's target is
//! the resolver's answer (§AR-system.2.10).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::citation_target::WorkspaceCheckTarget;
use super::point_body::point_body_pair;
use super::relative_fingerprint::relative_nested_headings;
use super::section_fingerprint::section_fingerprint;
use crate::config::{Frame, Schema};
use crate::grammar::{parse_id_arg, render_catalog_id, render_id};
use crate::model::{
    Catalog, Citation, Declaration, Id, LockEntry, LockLines, locked, paired_declarations,
    read_lock_lines, section_lock_path, sort_path_key,
};

/// What `fmt --write` does with one entry the lock holds (§FS-section-lock.write.rules).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EntryVerdict {
    /// It holds what the tree needs.
    Keep,
    /// Its section was edited in place, or a pointer acknowledged its move: the
    /// fingerprint to record, `None` for `-` (§FS-section-lock.move.pointer).
    Refresh(Option<String>),
    /// Nothing cites its address, it is no longer a locked address, or an earlier
    /// line enters the same coordinate.
    Drop,
    /// Its recorded text sits at `to`, another section of the declaration, and
    /// nothing acknowledged the move; `sites` are the lines that cite the recorded
    /// address, one per line, sorted by path then line
    /// (§FS-section-lock.checks.section-address-changed).
    Moved {
        to: String,
        sites: Vec<(PathBuf, usize)>,
    },
    /// No section holds its address any more, and its recorded text sits at `to`,
    /// another section of the declaration: dropped as `Drop` is, its coordinate no
    /// longer a locked address, and named on the `missing-section` finding of a
    /// citation of it (§FS-section-lock.checks.vanished).
    Gone { to: String },
}

/// One lock file judged against the tree: the one that exists, the one a locked
/// address needs, or both (§FS-section-lock.file.shard).
pub(crate) struct LockJudgement<'a> {
    /// The project its citations go into when that is another one
    /// (§FS-section-lock.file.members).
    pub(crate) alias: Option<&'a str>,
    /// The coordinate prefix: the ID as that project's `[id] format` renders it.
    pub(crate) rendered: String,
    /// What separates the ID from a section path there.
    pub(crate) separator: String,
    pub(crate) path: PathBuf,
    /// The ID it is of, `None` where the file's ID is declared nowhere.
    pub(crate) id: Option<&'a Id>,
    /// Its text as read, `None` where no file exists.
    pub(crate) text: Option<&'a str>,
    pub(crate) lines: LockLines,
    /// The declaration the ID pairs to, `None` where it pairs to none or several.
    pub(crate) decl: Option<&'a Declaration>,
    /// Each entry the file holds, in file order, with what becomes of it.
    pub(crate) verdicts: Vec<(LockEntry, EntryVerdict)>,
    /// Each locked address the file holds no entry for, with its citations and
    /// the fingerprint to record, `None` for `-`.
    pub(crate) unrecorded: Vec<(String, Vec<&'a Citation>, Option<String>)>,
}

impl LockJudgement<'_> {
    /// The coordinate of `section` as a line of the file spells it
    /// (§FS-section-lock.holds.entry).
    pub(crate) fn coordinate(&self, section: &str) -> String {
        format!("{}{}{section}", self.rendered, self.separator)
    }

    /// The coordinate as a finding names it: with the alias of the project it
    /// is in when that is another one (§FS-section-lock.file.members).
    pub(crate) fn cited(&self, section: &str) -> String {
        match self.alias {
            Some(alias) => format!("{alias}/{}", self.coordinate(section)),
            None => self.coordinate(section),
        }
    }

    /// The entries the file holds once `fmt --write` has run, as
    /// `(coordinate, fingerprint)` pairs (§FS-section-lock.write.rules).
    pub(crate) fn written_entries(&self) -> Vec<(String, Option<String>)> {
        let coordinate = |section: &str| self.coordinate(section);
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
                EntryVerdict::Drop | EntryVerdict::Gone { .. } => None,
            });
        let added = self
            .unrecorded
            .iter()
            .map(|(section, _, fingerprint)| (coordinate(section), fingerprint.clone()));
        kept.chain(added).collect()
    }
}

/// Where the text the lock of `id`, cited through `alias`, recorded for `section`
/// sits now, when no section holds `section` any more: the lock file, and the
/// address as a finding names it (§FS-section-lock.checks.vanished).
pub(crate) fn recorded_text_now_at<'j>(
    judgements: &'j [LockJudgement<'_>],
    alias: Option<&str>,
    id: &Id,
    section: &str,
) -> Option<(&'j Path, String)> {
    judgements
        .iter()
        .filter(|judgement| judgement.alias == alias && judgement.id == Some(id))
        .find_map(|judgement| {
            judgement
                .verdicts
                .iter()
                .find_map(|(entry, verdict)| match verdict {
                    EntryVerdict::Gone { to } if entry.section == section => {
                        Some((judgement.path.as_path(), judgement.cited(to)))
                    }
                    _ => None,
                })
        })
}

/// The project a lock file's entries are judged against: the citing project
/// itself, or the member or remote its alias names (§FS-section-lock.file.members).
#[derive(Clone, Copy)]
struct LockTarget<'a, 'b> {
    catalog: &'a Catalog,
    schema: &'b Schema,
    frame: Frame<'b>,
}

/// Judge every lock file the scan read and every one a locked address needs, in
/// path order (§FS-section-lock): a citation into another project
/// against `targets`, the projects the run loaded, by its alias, and a lock
/// file of an alias the run did not load not at all
/// (§FS-section-lock.file.members). Empty, and nothing sliced, in a tree with
/// neither a lock nor a cited locked address. Whether a citation's section is
/// locked is asked of the declaration it resolves to, through its stubs, so a
/// section in a stub's target outside `[scan] include` is locked as one inside it
/// (§FS-section-lock.holds.entry).
pub(crate) fn judge_section_locks<'a>(
    findings: &'a Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    targets: &BTreeMap<String, WorkspaceCheckTarget<'a>>,
) -> Vec<LockJudgement<'a>> {
    let local = LockTarget {
        catalog: findings,
        schema,
        frame,
    };
    let target_of = |alias: Option<&str>| match alias {
        None => Some(local),
        Some(alias) => targets.get(alias).map(|target| LockTarget {
            catalog: target.catalog,
            schema: target.schema,
            frame: target.frame(),
        }),
    };
    type Cited<'a> = BTreeMap<(Option<&'a str>, &'a Id), BTreeMap<&'a str, Vec<&'a Citation>>>;
    let mut cited: Cited<'a> = BTreeMap::new();
    for citation in &findings.citations {
        let Some(section) = &citation.section else {
            continue;
        };
        let alias = citation.namespace.as_deref();
        // §FS-section-lock.holds.entry: a relative heading's or a mixed path's address.
        if target_of(alias)
            .and_then(|target| sole_declaration(target.catalog, &citation.id))
            .and_then(|decl| decl.sections.get(section))
            .is_some_and(|info| locked(section, info))
        {
            cited
                .entry((alias, &citation.id))
                .or_default()
                .entry(section.as_str())
                .or_default()
                .push(citation);
        }
    }
    // §FS-section-lock: a tree that cites no locked address and has no lock reads none.
    if cited.is_empty() && findings.section_locks.is_empty() {
        return Vec::new();
    }
    let mut ids: BTreeMap<(Option<&'a str>, String), Option<&'a Id>> = BTreeMap::new();
    let mut texts = BTreeMap::new();
    for lock in &findings.section_locks {
        let alias = lock.alias.as_deref();
        let Some(target) = target_of(alias) else {
            continue;
        };
        texts.insert((alias, lock.id.as_str()), lock.text.as_str());
        ids.entry((alias, lock.id.clone())).or_insert_with(|| {
            // §FS-section-lock.write.never-a-move: an older renderer's numeric
            // spelling still owns its history (§FS-config.3.2.8).
            if let Ok((id, None)) = parse_id_arg(&lock.id, target.frame.grammar()) {
                return target
                    .catalog
                    .declarations
                    .get_key_value(&id)
                    .map(|(id, _)| id);
            }
            target
                .catalog
                .declarations
                .keys()
                .find(|id| render_id(target.frame.grammar(), id) == lock.id)
        });
    }
    let recorded: BTreeSet<_> = ids
        .iter()
        .filter_map(|((alias, _), id)| id.map(|id| (*alias, id)))
        .collect();
    for &(alias, id) in cited.keys() {
        // Keep every existing equivalent shard, including its spelling and
        // fingerprints; only an identity with no shard receives a new baseline.
        if recorded.contains(&(alias, id)) {
            continue;
        }
        let target = target_of(alias).expect("a cited project is loaded");
        let rendered = render_catalog_id(target.frame.grammar(), &target.catalog.declarations, id);
        ids.insert((alias, rendered), Some(id));
    }
    let mut judgements = Vec::new();
    for ((alias, rendered), id) in ids {
        let target = target_of(alias).expect("a judged project is loaded");
        let separator = target.schema.ids.section_separator.clone();
        let decl = id.and_then(|id| sole_declaration(target.catalog, id));
        let text = texts.get(&(alias, rendered.as_str())).copied();
        let lines = text.map_or_else(LockLines::default, |text| {
            read_lock_lines(text, &rendered, &separator)
        });
        let cited = id
            .and_then(|id| cited.get(&(alias, id)).cloned())
            .unwrap_or_default();
        let fingerprints = decl.map_or_else(Vec::new, |decl| {
            fingerprints(target, id.expect("a paired ID"), decl)
        });
        let (verdicts, unrecorded) =
            judge_entries(target.catalog, decl, &lines, cited, &fingerprints);
        judgements.push(LockJudgement {
            path: section_lock_path(frame.root(), alias, &rendered),
            id,
            alias,
            rendered,
            separator,
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

/// The one declaration `id`'s citations resolve to, as the pairing of its stubs
/// answers it: a locked address is a section of it (§FS-section-lock.terms).
fn sole_declaration<'a>(findings: &'a Catalog, id: &Id) -> Option<&'a Declaration> {
    let sole = paired_declarations(findings.declarations.get(id)?).sole()?;
    Some(sole.record)
}

/// Every section of `decl` with its line and fingerprint, in document order: the
/// one place a fingerprint is computed (§DF-section-lock-relative-fingerprint.decision.one-place).
fn fingerprints(
    target: LockTarget<'_, '_>,
    id: &Id,
    decl: &Declaration,
) -> Vec<(String, usize, String)> {
    let LockTarget {
        catalog,
        schema,
        frame,
    } = target;
    let mut sections = decl
        .sections
        .iter()
        .filter_map(|(path, info)| {
            let (_, full) =
                point_body_pair(&catalog.files, schema, frame, id, decl, Some((path, info)))
                    .ok()??;
            // §FS-section-lock.holds.fingerprint: the text below the heading line,
            // its nested headings written relative to the section.
            let below = full.split_once('\n').map_or("", |(_, below)| below);
            let below = relative_nested_headings(below, decl, path, info.line);
            Some((path.clone(), info.line, section_fingerprint(&below)))
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
    // §FS-section-lock.checks.section-address-changed, §FS-section-lock.checks.vanished:
    // the first other section, in document order, holding the recorded text now.
    let moved_to = |section: &str, recorded: &str| {
        fingerprints
            .iter()
            .find(|(path, _, fingerprint)| path != section && fingerprint == recorded)
            .map(|(path, _, _)| path.clone())
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
    let was_cited: BTreeMap<&str, Vec<&Citation>> = cited.clone();
    let mut verdicts = Vec::new();
    for (index, entry) in lines.entries.iter().enumerate() {
        let section = entry.section.as_str();
        let info = decl.and_then(|decl| decl.sections.get(section));
        let held = info.is_some_and(|info| locked(section, info));
        let verdict = match (current(section), held && was_cited.contains_key(section)) {
            _ if chosen.get(section) != Some(&index) => EntryVerdict::Drop,
            // A text no slice could read is no evidence of an edit or a move.
            (None, true) => EntryVerdict::Keep,
            (Some(now), true) => match entry.fingerprint.as_deref() {
                None => match written(now) {
                    Some(fingerprint) => EntryVerdict::Refresh(Some(fingerprint)),
                    None => EntryVerdict::Keep,
                },
                Some(recorded) if recorded == now => EntryVerdict::Keep,
                Some(recorded) => match moved_to(section, recorded) {
                    Some(to) if !points_at(findings, decl, section, &to) => EntryVerdict::Moved {
                        to,
                        sites: citing_sites(&was_cited[section]),
                    },
                    // Edited in place, or a move a pointer acknowledged.
                    _ => EntryVerdict::Refresh(written(now)),
                },
            },
            // §FS-section-lock.checks.vanished: an address no section holds.
            _ => match entry.fingerprint.as_deref() {
                Some(recorded) if info.is_none() => moved_to(section, recorded)
                    .map_or(EntryVerdict::Drop, |to| EntryVerdict::Gone { to }),
                _ => EntryVerdict::Drop,
            },
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

/// The lines `citations` sit on, two citations on one line being one site, sorted
/// by path then line as the duplicate check sorts its sites
/// (§FS-section-lock.checks.section-address-changed).
fn citing_sites(citations: &[&Citation]) -> Vec<(PathBuf, usize)> {
    let mut sites = citations
        .iter()
        .map(|citation| (citation.file.clone(), citation.line))
        .collect::<Vec<_>>();
    sites.sort_by(|a, b| (sort_path_key(&a.0), a.1).cmp(&(sort_path_key(&b.0), b.1)));
    sites.dedup();
    sites
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
