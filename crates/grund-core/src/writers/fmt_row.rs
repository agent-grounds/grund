//! The account of one change `fmt` makes (§FS-fmt.3.5.1, §FS-fmt.3.7): which
//! kinds of rewrite fired on a line and how often, or which lock entry a row
//! names and what the write does to its file. The row text both modes print is
//! rendered from that one record, and the `--write` totals sum its counts, so the
//! preview and the write cannot name different kinds (§FS-fmt.7.3).

use std::ops::AddAssign;
use std::path::PathBuf;

/// How many rewrites of each kind one change makes (§FS-fmt.3.7,
/// §FS-distribution.3.2.2.2), in §FS-fmt.3.5.1's label order with the lock last.
/// A link counts where it is new or its destination changed; one already right
/// changes no byte and counts nothing.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct FmtCounts {
    pub triggers_marked: usize,
    pub citations_marked: usize,
    pub local_sections_expanded: usize,
    pub shorthands_expanded: usize,
    pub links_written: usize,
    pub section_lock_entries: usize,
}

impl AddAssign for FmtCounts {
    fn add_assign(&mut self, other: Self) {
        self.triggers_marked += other.triggers_marked;
        self.citations_marked += other.citations_marked;
        self.local_sections_expanded += other.local_sections_expanded;
        self.shorthands_expanded += other.shorthands_expanded;
        self.links_written += other.links_written;
        self.section_lock_entries += other.section_lock_entries;
    }
}

/// What a write does to a whole lock file, where it does more than change its
/// lines (§FS-section-lock.write.report): the file is not there and the write
/// leaves entries in it, or it is there and the write leaves none.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FmtLockFile {
    Created,
    Deleted,
}

impl FmtLockFile {
    /// The word the `--write` line and the host field carry.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Deleted => "deleted",
        }
    }

    /// §FS-section-lock.write.report: the text a lock file's first dry-run row ends with.
    fn row_suffix(self) -> &'static str {
        match self {
            Self::Created => "; creates the file",
            Self::Deleted => "; deletes the file",
        }
    }

    /// The test `write_lock` acts on (§FS-section-lock.file.shard), so what the
    /// report says is what the write does.
    pub(super) fn of(exists: bool, text: &str) -> Option<Self> {
        match (exists, text.is_empty()) {
            (false, false) => Some(Self::Created),
            (true, true) => Some(Self::Deleted),
            _ => None,
        }
    }
}

/// One changed line or lock row, as the run reports it (§FS-fmt.3.5).
#[derive(Clone, Debug)]
pub(crate) struct FmtRow {
    pub(crate) path: PathBuf,
    pub(crate) line: usize,
    /// The row text after `path:line: `.
    pub(crate) label: String,
    pub(crate) counts: FmtCounts,
    /// Set on every row of a lock file the write creates or deletes.
    pub(crate) lock_file: Option<FmtLockFile>,
}

impl FmtRow {
    /// The rows of one lock file, sorted by line then text, with `lock_file`'s
    /// suffix on the first once sorted (§FS-section-lock.write.report).
    pub(super) fn lock_rows(
        path: &std::path::Path,
        rows: Vec<(usize, String)>,
        lock_file: Option<FmtLockFile>,
    ) -> impl Iterator<Item = Self> + '_ {
        let counts = FmtCounts {
            section_lock_entries: 1,
            ..FmtCounts::default()
        };
        rows.into_iter()
            .enumerate()
            .map(move |(index, (line, row))| {
                let suffix = lock_file
                    .filter(|_| index == 0)
                    .map(FmtLockFile::row_suffix);
                Self {
                    path: path.to_path_buf(),
                    line,
                    label: format!("{row}{}", suffix.unwrap_or_default()),
                    counts,
                    lock_file,
                }
            })
    }

    /// The JSON a partial run's diagnostic carries for this row
    /// (§FS-distribution.3.2.2.1), rendered against `render`.
    pub(super) fn partial_json(&self, render: &crate::config::Config) -> serde_json::Value {
        let counts = &self.counts;
        serde_json::json!({
            "path": crate::config::display_path(render, &self.path),
            "line": self.line,
            "label": self.label,
            "counts": {
                "triggers_marked": counts.triggers_marked,
                "citations_marked": counts.citations_marked,
                "local_sections_expanded": counts.local_sections_expanded,
                "shorthands_expanded": counts.shorthands_expanded,
                "links_written": counts.links_written,
                "section_lock_entries": counts.section_lock_entries,
            },
            "lock_file": self.lock_file.map(FmtLockFile::as_str),
        })
    }
}

/// What one line's rewrites did (§FS-fmt.3.5.1): the counts, the text each
/// expansion wrote (§FS-fmt.3.6), and a link's repeated-anchor detail
/// (§DF-repeated-heading-anchor.decision.row).
#[derive(Default)]
pub(super) struct LineRewrite {
    pub(super) counts: FmtCounts,
    /// `written → canonical` for each expansion, in source order (§FS-fmt.3.6.1).
    pub(super) expansions: Vec<String>,
    pub(super) anchor_detail: Option<String>,
}

const LINK_LABEL: &str = "markdown link";

impl LineRewrite {
    /// The row text (§FS-fmt.3.5.1): the first kind that fired with the expansion
    /// detail, then every other kind that fired, in label order, as one `, `
    /// list that follows the label with `; ` when it has detail and `, ` when not.
    /// A `markdown link` keeps its repeated-anchor detail wherever it stands; being
    /// last, it ends the row. Empty when nothing fired.
    pub(super) fn label(&self) -> String {
        let counts = &self.counts;
        let kinds = [
            (counts.triggers_marked, "trigger \u{2192} marker"),
            (counts.citations_marked, "bare \u{2192} marker"),
            (
                counts.local_sections_expanded,
                "local section \u{2192} canonical",
            ),
            (counts.shorthands_expanded, "shorthand \u{2192} canonical"),
            (counts.links_written, LINK_LABEL),
        ];
        let mut fired = kinds
            .iter()
            .filter(|(count, _)| *count > 0)
            .map(
                |(_, kind)| match (&self.anchor_detail, *kind == LINK_LABEL) {
                    (Some(detail), true) => format!("{LINK_LABEL}: {detail}"),
                    _ => kind.to_string(),
                },
            );
        let Some(mut row) = fired.next() else {
            return String::new();
        };
        // §FS-fmt.3.6: only an expansion writes into the ID, so only it has detail here.
        let detailed = !self.expansions.is_empty();
        if detailed {
            row.push_str(": ");
            row.push_str(&self.expansions.join(", "));
        }
        let rest = fired.collect::<Vec<_>>();
        if !rest.is_empty() {
            row.push_str(if detailed { "; " } else { ", " });
            row.push_str(&rest.join(", "));
        }
        row
    }
}

#[cfg(test)]
mod tests {
    use super::{FmtCounts, LineRewrite};

    fn rewrite(counts: [usize; 5], expansions: &[&str], anchor: Option<&str>) -> String {
        LineRewrite {
            counts: FmtCounts {
                triggers_marked: counts[0],
                citations_marked: counts[1],
                local_sections_expanded: counts[2],
                shorthands_expanded: counts[3],
                links_written: counts[4],
                section_lock_entries: 0,
            },
            expansions: expansions.iter().map(|it| it.to_string()).collect(),
            anchor_detail: anchor.map(str::to_string),
        }
        .label()
    }

    // §FS-fmt.3.5.1: every kind is named once, in label order.
    #[test]
    fn all_five_kinds_on_one_line_are_named_once_each() {
        assert_eq!(
            rewrite([2, 1, 1, 3, 4], &["\u{a7}1 \u{2192} \u{a7}FS-a.1"], None),
            "trigger \u{2192} marker: \u{a7}1 \u{2192} \u{a7}FS-a.1; bare \u{2192} marker, \
             local section \u{2192} canonical, shorthand \u{2192} canonical, markdown link"
        );
        assert_eq!(
            rewrite([0, 3, 0, 0, 2], &[], None),
            "bare \u{2192} marker, markdown link"
        );
        assert_eq!(rewrite([0; 5], &[], None), "");
    }

    // §FS-fmt.3.6: the repeated-anchor note stays on the link, leading or trailing.
    #[test]
    fn a_link_keeps_its_anchor_detail_leading_or_trailing() {
        assert_eq!(rewrite([0, 0, 0, 0, 1], &[], Some("d")), "markdown link: d");
        assert_eq!(
            rewrite([1, 0, 0, 0, 2], &[], Some("d")),
            "trigger \u{2192} marker, markdown link: d"
        );
    }
}
