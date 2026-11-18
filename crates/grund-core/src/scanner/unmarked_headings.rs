use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::embedded_value_context::immediate_numeric_component;
use super::extents::DeclarationBodies;
use super::relative_sections::usable_section;
use super::values::value_declaration_is_in_home;
use crate::config::{Frame, Schema};
use crate::grammar::{Grammar, path_fault, relative_heading_hint};
use crate::model::{
    Catalog, Declaration, FileRecord, PathFaultHint, RelativeHint, SectionInfo, UnmarkedHeading,
    UnmarkedHeadingCandidate, named_section_path,
};

/// Retain only Markdown headings owned by a declaration body, choose the
/// nearest nested owner, and assign deterministic unused section paths
/// (§AR-scanner.2.2.7, §AR-scanner.2.4.1, §FS-declarations.checks.unmarked-heading).
pub(super) fn assign_unmarked_heading_owners(
    findings: &mut Catalog,
    mut candidates: Vec<UnmarkedHeadingCandidate>,
    record: &FileRecord,
    grammar: &Grammar,
    value_homes: Option<(&Schema, Frame<'_>, &Path)>,
) {
    #[derive(Clone)]
    struct KnownPath {
        line: usize,
        path: String,
    }

    let owners = DeclarationBodies::of(record, findings);
    let bodies = findings
        .declarations
        .values()
        .flatten()
        .map(|decl| {
            (
                decl.line,
                (
                    decl.heading_level,
                    decl.id.clone(),
                    decl.sections
                        .iter()
                        .map(|(path, info)| KnownPath {
                            line: info.line,
                            path: path.clone(),
                        })
                        .collect::<Vec<_>>(),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    candidates.sort_by_key(|candidate| candidate.line);
    // This pass is per file, so the inclusive body span identifies a concrete
    // declaration site even when another site declares the same logical ID.
    let mut suggested_by_body: BTreeMap<(usize, usize), Vec<KnownPath>> = BTreeMap::new();
    // §FS-declarations.checks.unmarked-heading.2.3: a repair is shown once per body.
    let mut repaired_by_body: BTreeMap<(usize, usize), BTreeSet<String>> = BTreeMap::new();

    for candidate in candidates {
        let Some((body_start, body_end)) = owners.owner(candidate.line, candidate.heading_level)
        else {
            continue;
        };
        let Some((declaration_level, owner, existing)) = bodies.get(&body_start) else {
            continue;
        };
        let target_depth = candidate.heading_level - declaration_level;
        let prior_suggestions = suggested_by_body.entry((body_start, body_end)).or_default();
        let mut paths = existing.clone();
        paths.extend(prior_suggestions.iter().cloned());
        let suggested_path = suggested_section_path(&paths, candidate.line, target_depth);
        prior_suggestions.push(KnownPath {
            line: candidate.line,
            path: suggested_path.clone(),
        });
        // §FS-declarations.checks.unmarked-heading.2.1: a dotted heading is shown the
        // relative form, where the suggested coordinate's parent could hold one.
        let relative_hint = candidate
            .title
            .starts_with('.')
            .then_some(())
            .and_then(|()| {
                let parent = suggested_path.rsplit_once('.').map(|(parent, _)| parent);
                let decl = findings
                    .declarations
                    .values()
                    .flatten()
                    .find(|decl| decl.line == body_start)?;
                parent
                    .is_none_or(|parent| usable_section(decl, body_end, parent))
                    .then(|| RelativeHint {
                        parent: parent.unwrap_or_default().to_string(),
                        heading: relative_heading_hint(
                            candidate.heading_level,
                            &suggested_path,
                            &candidate.title,
                        ),
                    })
            });
        let fault_hint = findings
            .declarations
            .values()
            .flatten()
            .find(|decl| decl.line == body_start)
            .and_then(|decl| {
                let value = value_homes.is_some_and(|(schema, frame, path)| {
                    value_declaration_is_in_home(schema, frame, path, &decl.id)
                });
                fault_hint(&candidate, decl, body_end, grammar.named_sections, value)
            })
            .filter(|hint| {
                repaired_by_body
                    .entry((body_start, body_end))
                    .or_default()
                    .insert(hint.corrected.clone())
            });
        findings.unmarked_headings.push(UnmarkedHeading {
            file: candidate.file,
            line: candidate.line,
            column: candidate.column,
            heading: candidate.heading,
            heading_level: candidate.heading_level,
            title: candidate.title,
            owner: owner.clone(),
            // §FS-declarations.checks.unmarked-heading.2.2: retain the concrete owner site.
            owner_line: body_start,
            suggested_path,
            relative_hint,
            fault_hint,
        });
    }

    fn suggested_section_path(paths: &[KnownPath], line: usize, target_depth: usize) -> String {
        let parent = paths
            .iter()
            .filter(|known| known.line < line && path_depth(&known.path) < target_depth)
            .max_by_key(|known| known.line)
            .map(|known| known.path.clone());

        let mut parts = parent
            .as_deref()
            .map(|path| path.split('.').map(str::to_string).collect::<Vec<_>>())
            .unwrap_or_default();
        if parts.is_empty() {
            parts.push(next_numeric_child(paths, &[]));
        }
        while parts.len() + 1 < target_depth {
            parts.push("1".to_string());
        }
        if parts.len() < target_depth {
            let next = next_numeric_child(paths, &parts);
            parts.push(next);
        }
        parts.join(".")
    }

    fn path_depth(path: &str) -> usize {
        path.split('.').count()
    }

    fn next_numeric_child(paths: &[KnownPath], parent: &[String]) -> String {
        paths
            .iter()
            .filter_map(|known| {
                let parts = known.path.split('.').collect::<Vec<_>>();
                if parts.len() != parent.len() + 1
                    || !parts[..parent.len()]
                        .iter()
                        .zip(parent)
                        .all(|(left, right)| *left == right)
                {
                    return None;
                }
                let child = *parts.last()?;
                child
                    .bytes()
                    .all(|byte| byte.is_ascii_digit())
                    .then_some(child)
            })
            .max_by(|left, right| compare_decimal(left, right))
            .map(increment_decimal)
            .unwrap_or_else(|| "1".to_string())
    }

    // Section coordinates accept an unbounded digit run. Compare and increment those runs as
    // decimal text so repair guidance stays above every authored sibling without imposing an
    // integer-width ceiling (§FS-declarations.checks.unmarked-heading.3).
    fn compare_decimal(left: &str, right: &str) -> std::cmp::Ordering {
        let left = normalize_decimal(left);
        let right = normalize_decimal(right);
        left.len().cmp(&right.len()).then_with(|| left.cmp(right))
    }

    fn normalize_decimal(value: &str) -> &str {
        let normalized = value.trim_start_matches('0');
        if normalized.is_empty() {
            "0"
        } else {
            normalized
        }
    }

    fn increment_decimal(value: &str) -> String {
        let normalized = normalize_decimal(value);
        let mut digits = normalized.as_bytes().to_vec();
        for digit in digits.iter_mut().rev() {
            if *digit == b'9' {
                *digit = b'0';
            } else {
                *digit += 1;
                return String::from_utf8(digits).expect("decimal digits stay UTF-8");
            }
        }
        format!("1{}", "0".repeat(digits.len()))
    }
}

/// The fault of a heading whose complete path is already in place, where writing
/// the repairing heading instead would declare a recognized section of `decl`,
/// whose body ends on `end` (§FS-declarations.checks.unmarked-heading.2.3): named
/// sections on for a name-bearing path, one component per ATX level below the
/// declaration, a parent [`usable_section`] accepts, and a path not already
/// declared. Where the path lies in a value run — the body of a value declaration
/// (`value`, §FS-values.2.1), or a value root inside `decl` (§FS-values.2.4,
/// §FS-values.2.5) — it must also be that run's next component, after every one
/// the run declares.
fn fault_hint(
    candidate: &UnmarkedHeadingCandidate,
    decl: &Declaration,
    end: usize,
    named_sections: bool,
    value: bool,
) -> Option<PathFaultHint> {
    let fault = path_fault(&candidate.title)?;
    let corrected = fault.corrected.as_str();
    let declares = (named_sections || !named_section_path(corrected))
        && corrected.split('.').count() + decl.heading_level == candidate.heading_level
        && corrected
            .rsplit_once('.')
            .is_none_or(|(parent, _)| usable_section(decl, end, parent))
        && !decl.sections.contains_key(corrected)
        && value_run(decl, corrected, value)
            .is_none_or(|run| next_value_component(decl, end, candidate.line, run, corrected));
    declares.then(|| PathFaultHint {
        path: fault.path.to_string(),
        missing_title: fault.title.is_none(),
        underscore: fault.underscore,
        colon: fault.colon,
        heading: fault.shown_heading(candidate.heading_level),
        corrected: corrected.to_string(),
    })
}

/// The root path of the value run `corrected` would join in `decl`, if any
/// (§FS-declarations.checks.unmarked-heading.2.3): the empty path, the whole body,
/// in a value declaration (`value`, §FS-values.2.1); otherwise the outermost proper
/// prefix of `corrected` that is a value root of `decl`, embedded (§FS-values.2.4)
/// or by a declared chapter (§FS-values.2.5). Outermost, because a root nested in
/// another is refused anyway (§FS-values.2.4.4) and the outer one is the stricter.
fn value_run<'a>(decl: &Declaration, corrected: &'a str, value: bool) -> Option<&'a str> {
    if value {
        return Some("");
    }
    corrected
        .match_indices('.')
        .map(|(at, _)| &corrected[..at])
        .find(|prefix| {
            decl.sections
                .get(*prefix)
                .is_some_and(|info| info.value_root.is_some())
        })
}

/// Whether `corrected`, written at `line`, appends the next component of the value
/// run rooted at `root` in `decl`, whose body ends on `end`: `root` and one number,
/// one more than the run's components in the body, all of them above `line`
/// (§FS-declarations.checks.unmarked-heading.2.3). This is the run's contiguity and
/// physical order, so the count is the one its own check keeps: every section the
/// body declares in a value declaration (`root` empty, §FS-values.2.1); under a root,
/// every heading of an immediate numeric component, a duplicate's too, and neither a
/// grandchild nor a named child, which the root's cursor skips (§FS-values.2.4.3).
fn next_value_component(
    decl: &Declaration,
    end: usize,
    line: usize,
    root: &str,
    corrected: &str,
) -> bool {
    let in_body = |info: &&SectionInfo| (decl.body_start..=end).contains(&info.line);
    let run = if root.is_empty() {
        decl.sections.values().filter(in_body).collect::<Vec<_>>()
    } else {
        let duplicates = decl
            .duplicate_sections
            .iter()
            .map(|(path, info)| (path, info));
        decl.sections
            .iter()
            .chain(duplicates)
            .filter(|(path, _)| immediate_numeric_component(root, path))
            .map(|(_, info)| info)
            .filter(in_body)
            .collect()
    };
    let next = run.len() + 1;
    let expected = if root.is_empty() {
        next.to_string()
    } else {
        format!("{root}.{next}")
    };
    run.iter().all(|info| info.line < line) && corrected == expected
}
