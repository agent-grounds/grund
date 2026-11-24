use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::embedded_value_context::{
    ValueLines, immediate_numeric_component, relative_heading_paths, root_cursor,
};
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
    lines: &ValueLines<'_>,
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
    // §FS-declarations.checks.unmarked-heading: a repair is shown once per body.
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
        let decl = findings
            .declarations
            .values()
            .flatten()
            .find(|decl| decl.line == body_start);
        let value = decl.is_some_and(|decl| {
            value_homes.is_some_and(|(schema, frame, path)| {
                value_declaration_is_in_home(schema, frame, path, &decl.id)
            })
        });
        // §FS-declarations.checks.unmarked-heading: a dotted heading is shown the relative
        // form where the suggested coordinate's parent could hold one and a value run would
        // take it there (§FS-declarations.checks.unmarked-heading).
        let relative_hint = decl
            .filter(|_| candidate.title.starts_with('.'))
            .and_then(|decl| {
                let parent = suggested_path.rsplit_once('.').map(|(parent, _)| parent);
                let line = candidate.line;
                let next =
                    |run| next_value_component(decl, body_end, line, run, &suggested_path, lines);
                (parent.is_none_or(|parent| usable_section(decl, body_end, parent))
                    && value_run(decl, &suggested_path, value).is_none_or(next))
                .then(|| RelativeHint {
                    parent: parent.unwrap_or_default().to_string(),
                    heading: relative_heading_hint(
                        candidate.heading_level,
                        &suggested_path,
                        &candidate.title,
                    ),
                })
            });
        let fault_hint = decl
            .and_then(|decl| {
                fault_hint(
                    &candidate,
                    decl,
                    body_end,
                    grammar.named_sections,
                    value,
                    lines,
                )
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
            // §FS-declarations.checks.unmarked-heading: retain the concrete owner site.
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
    // integer-width ceiling (§FS-declarations.checks.unmarked-heading).
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
/// whose body ends on `end` (§FS-declarations.checks.unmarked-heading): named
/// sections on for a name-bearing path, one component per ATX level below the
/// declaration, a parent [`usable_section`] accepts, and a path not already
/// declared. Where the path lies in a value run — the body of a value declaration
/// (`value`, §FS-values.2.1), or a value root inside `decl` (§FS-values.2.4,
/// §FS-values.2.5) — it must also be the component that run expects at the heading,
/// as [`next_value_component`] counts it over the file's `lines`.
fn fault_hint(
    candidate: &UnmarkedHeadingCandidate,
    decl: &Declaration,
    end: usize,
    named_sections: bool,
    value: bool,
    lines: &ValueLines<'_>,
) -> Option<PathFaultHint> {
    let fault = path_fault(&candidate.title)?;
    let corrected = fault.corrected.as_str();
    let declares = (named_sections || !named_section_path(corrected))
        && corrected.split('.').count() + decl.heading_level == candidate.heading_level
        && corrected
            .rsplit_once('.')
            .is_none_or(|(parent, _)| usable_section(decl, end, parent))
        && !decl.sections.contains_key(corrected)
        && value_run(decl, corrected, value).is_none_or(|run| {
            next_value_component(decl, end, candidate.line, run, corrected, lines)
        });
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
/// (§FS-declarations.checks.unmarked-heading): the empty path, the whole body,
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

/// Whether `corrected`, written at `line`, is the component the value run rooted at
/// `root` in `decl`, whose body ends on `end`, expects there
/// (§FS-declarations.checks.unmarked-heading). In a value declaration (`root` empty,
/// §FS-values.2.1) that is one number, one more than the sections the body declares, all
/// of them above `line`. Under a root it is `root` and one number, one more than the
/// lines above `line` that the root's own cursor advances on (§FS-values.2.4.3): every
/// heading read at an immediate numeric component of `root`, recorded, a duplicate or
/// one with no title alike, and neither a grandchild nor a named child. The lines are
/// read through that cursor, [`root_cursor`], so the repair names the coordinate the
/// root reports as expected on the same line, and a number taken below is left to the
/// caller's own check that the path is not declared yet.
fn next_value_component(
    decl: &Declaration,
    end: usize,
    line: usize,
    root: &str,
    corrected: &str,
    lines: &ValueLines<'_>,
) -> bool {
    if root.is_empty() {
        let in_body = |info: &&SectionInfo| (decl.body_start..=end).contains(&info.line);
        let run = decl.sections.values().filter(in_body).collect::<Vec<_>>();
        let expected = (run.len() + 1).to_string();
        return run.iter().all(|info| info.line < line) && corrected == expected;
    }
    let Some(info) = decl.sections.get(root) else {
        return false;
    };
    let relative = relative_heading_paths(decl);
    let (is_md, lexical) = (lines.reading.markdown, lines.reading.lexical);
    let above = root_cursor(
        lines.lines(),
        info,
        decl.body_end,
        &relative,
        is_md,
        lexical,
    )
    .take_while(|(at, _, _)| *at < line)
    .filter(|(_, _, path)| {
        path.as_deref()
            .is_some_and(|path| immediate_numeric_component(root, path))
    })
    .count();
    corrected == format!("{root}.{}", above + 1)
}
