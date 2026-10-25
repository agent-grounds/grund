use std::collections::BTreeMap;

use super::extents::DeclarationBodies;
use super::relative_sections::usable_section;
use crate::grammar::relative_heading_hint;
use crate::model::{Catalog, FileRecord, RelativeHint, UnmarkedHeading, UnmarkedHeadingCandidate};

/// Retain only Markdown headings owned by a declaration body, choose the
/// nearest nested owner, and assign deterministic unused section paths
/// (§AR-scanner.2.2.7, §AR-scanner.2.4.1, §FS-declarations.checks.unmarked-heading).
pub(super) fn assign_unmarked_heading_owners(
    findings: &mut Catalog,
    mut candidates: Vec<UnmarkedHeadingCandidate>,
    record: &FileRecord,
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
