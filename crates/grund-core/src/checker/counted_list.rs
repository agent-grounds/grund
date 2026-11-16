//! The list a count row names (§FS-rules.7.3.1): the engine hands over the citations a count read
//! (§AR-rules.5), and the checker, which holds the run's spellings and report base, writes
//! `; counted at <path>:<line> (<citation>), …` after the whole message and fills `sites` with
//! each distinct line, so a run narrowed to any of them reports the row (§FS-check.1.3.6.2).

use std::path::{Path, PathBuf};

use super::spellings_report::PlaceNames;
use crate::config::Frame;
use crate::model::{Diagnostic, Site};
use crate::rules::COUNTED_AT;
use crate::rules::engine::RuleFinding;
use crate::rules::facts::SiteMeta;

/// Each engine finding as the report holds it: a count above zero carries its list and `sites`,
/// and every other finding, a zero count included, is the engine's diagnostic unchanged.
pub(super) fn with_counted_lists(
    findings: Vec<RuleFinding>,
    spellings: &PlaceNames<'_>,
    frame: Frame<'_>,
) -> Vec<Diagnostic> {
    findings
        .into_iter()
        .map(|finding| with_counted_list(finding, spellings, frame))
        .collect()
}

fn with_counted_list(
    finding: RuleFinding,
    spellings: &PlaceNames<'_>,
    frame: Frame<'_>,
) -> Diagnostic {
    let (mut diagnostic, counted) = finding;
    if counted.is_empty() {
        return diagnostic;
    }
    // §AR-scanner.1.8: named as a whole run names them, before anything is rendered.
    let mut sites: Vec<Site> = counted
        .iter()
        .map(|citation| Site {
            path: PathBuf::from(&citation.anchor.path),
            line: citation.anchor.line,
        })
        .collect();
    spellings.settle(&mut sites);
    let entries = sites.into_iter().zip(counted).collect();
    let (clause, sites) = counted_clause(entries, |path| frame.display_path(path));
    diagnostic.message.push_str(&clause);
    diagnostic.sites = sites;
    diagnostic
}

/// The clause and the `sites` of `entries`, each a settled site beside the citation written there:
/// entries by path, then line, then column, with no cap, and each distinct line once
/// (§FS-rules.7.3.1). `display` names a path as the report names every path (§FS-errors.3.1), and
/// the citation is the one written there, without its `§`.
fn counted_clause(
    mut entries: Vec<(Site, SiteMeta)>,
    display: impl Fn(&Path) -> String,
) -> (String, Vec<Site>) {
    entries.sort_by(|(a, at), (b, bt)| {
        (&a.path, a.line, at.anchor.column).cmp(&(&b.path, b.line, bt.anchor.column))
    });
    let named: Vec<String> = entries
        .iter()
        .map(|(site, citation)| {
            format!(
                "{}:{} ({})",
                display(&site.path),
                site.line,
                citation
                    .label
                    .strip_prefix('\u{a7}')
                    .unwrap_or(&citation.label)
            )
        })
        .collect();
    let mut sites: Vec<Site> = Vec::new();
    for (site, _) in entries {
        if !sites
            .iter()
            .any(|held| held.path == site.path && held.line == site.line)
        {
            sites.push(site);
        }
    }
    (format!("{COUNTED_AT}{}", named.join(", ")), sites)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::RuleAnchor;

    fn entry(path: &str, line: usize, column: usize, written: &str) -> (Site, SiteMeta) {
        (
            Site {
                path: PathBuf::from(path),
                line,
            },
            SiteMeta {
                label: format!("\u{a7}{written}"),
                anchor: RuleAnchor {
                    path: path.into(),
                    line,
                    column: Some(column),
                },
            },
        )
    }

    fn render(entries: Vec<(Site, SiteMeta)>) -> (String, Vec<(String, usize)>) {
        let (clause, sites) = counted_clause(entries, |path| path.to_string_lossy().into_owned());
        let sites = sites
            .into_iter()
            .map(|site| (site.path.to_string_lossy().into_owned(), site.line))
            .collect();
        (clause, sites)
    }

    /// §FS-rules.7.3.1: by path, then line, then column; two citations on one line are two
    /// entries and one site.
    #[test]
    fn entries_sort_by_path_line_and_column_and_one_line_is_one_site() {
        let (clause, sites) = render(vec![
            entry("b.md", 3, 1, "X-b"),
            entry("a.md", 12, 30, "X-a.1"),
            entry("a.md", 12, 9, "X-c"),
            entry("a.md", 7, 4, "X-a"),
        ]);
        assert_eq!(
            clause,
            "; counted at a.md:7 (X-a), a.md:12 (X-c), a.md:12 (X-a.1), b.md:3 (X-b)"
        );
        assert_eq!(
            sites,
            vec![("a.md".into(), 7), ("a.md".into(), 12), ("b.md".into(), 3)]
        );
    }

    /// §FS-rules.7.3.1: no cap.
    #[test]
    fn twelve_citations_are_twelve_entries() {
        let entries = (1..=12).map(|line| entry("a.md", line, 1, "X-a")).collect();
        let (clause, sites) = render(entries);
        assert_eq!(clause.matches("(X-a)").count(), 12);
        assert_eq!(sites.len(), 12);
        assert!(clause.ends_with("a.md:12 (X-a)"));
    }

    /// The display spells the path and nothing else (§FS-errors.3.1).
    #[test]
    fn the_path_is_named_by_the_display() {
        let (clause, _) = counted_clause(vec![entry("docs/a.md", 2, 1, "X-a")], |path| {
            format!("../{}", path.display())
        });
        assert_eq!(clause, "; counted at ../docs/a.md:2 (X-a)");
    }
}
