//! The account of one change `fmt` makes (§FS-fmt.3.5, §FS-fmt.3.7): which
//! kinds of rewrite fired on a line and how often, or which lock entry a row
//! names and what the write does to its file. The row text both modes print is
//! rendered from that one record, and the `--write` totals sum its counts, so the
//! preview and the write cannot name different kinds (§FS-fmt.7.3).

use std::ops::AddAssign;
use std::path::PathBuf;

/// How many rewrites of each kind one change makes (§FS-fmt.3.7,
/// §FS-distribution.3.2.2), in §FS-fmt.3.5's label order with the lock last.
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
    /// (§FS-distribution.3.2.2), rendered against `render`.
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

/// What one line's rewrites did (§FS-fmt.3.5): the counts, the text each
/// expansion wrote (§FS-fmt.3.6), a link's repeated-anchor detail
/// (§DF-repeated-heading-anchor.decision.row), and each link it wrote, before
/// and after (§FS-fmt.3.6).
#[derive(Default)]
pub(super) struct LineRewrite {
    pub(super) counts: FmtCounts,
    /// `written → canonical` for each expansion, in source order (§FS-fmt.3.6).
    pub(super) expansions: Vec<String>,
    pub(super) anchor_detail: Option<String>,
    /// `replaced → written` for each link, in source order (§FS-fmt.3.6).
    pub(super) links: Vec<String>,
}

const LINK_LABEL: &str = "markdown link";

impl LineRewrite {
    /// The row text (§FS-fmt.3.5): the first kind that fired with the expansion
    /// detail, then every other kind that fired, in label order, as one `, `
    /// list that follows the label with `; ` when it has detail and `, ` when not.
    /// A `markdown link` keeps its own detail wherever it stands; being last, it
    /// ends the row. Empty when nothing fired.
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
            .map(|(_, kind)| {
                if *kind == LINK_LABEL {
                    self.link_label()
                } else {
                    kind.to_string()
                }
            });
        let Some(mut row) = fired.next() else {
            return String::new();
        };
        // §FS-fmt.3.6: the leading label carries the expansions' detail; a link's
        // own detail rides on its label, leading or trailing (§FS-fmt.3.6).
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

    /// `markdown link`, then `: ` and its detail where it has any: the
    /// repeated-anchor note, then `; ` and the items where both are present
    /// (§FS-fmt.3.6). Both are appended, so the label alone, and the label with
    /// the note, stay prefixes of it (§FS-fmt.3.5).
    fn link_label(&self) -> String {
        let mut label = LINK_LABEL.to_string();
        let items = (!self.links.is_empty()).then(|| self.links.join(", "));
        let detail = [self.anchor_detail.clone(), items];
        let detail = detail.into_iter().flatten().collect::<Vec<_>>();
        if !detail.is_empty() {
            label.push_str(": ");
            label.push_str(&detail.join("; "));
        }
        label
    }
}

#[cfg(test)]
mod tests {
    use super::{FmtCounts, LineRewrite};

    fn line(
        counts: [usize; 4],
        expansions: &[&str],
        anchor: Option<&str>,
        links: &[&str],
    ) -> LineRewrite {
        LineRewrite {
            counts: FmtCounts {
                triggers_marked: counts[0],
                citations_marked: counts[1],
                local_sections_expanded: counts[2],
                shorthands_expanded: counts[3],
                links_written: links.len(),
                section_lock_entries: 0,
            },
            expansions: expansions.iter().map(|it| it.to_string()).collect(),
            anchor_detail: anchor.map(str::to_string),
            links: links.iter().map(|it| it.to_string()).collect(),
        }
    }

    fn rewrite(
        counts: [usize; 4],
        expansions: &[&str],
        anchor: Option<&str>,
        links: &[&str],
    ) -> String {
        line(counts, expansions, anchor, links).label()
    }

    const NOTE: &str = "repeated heading anchor #1-overview \u{2192} #1-overview-1, \
                        numbered since grund 0.18";
    const ALPHA: &str = "[\u{a7}SRC-501-alpha](SRC-501-alpha.md) \u{2192} \
                         [\u{a7}SRC-501-alpha](SRC-501-alpha.md#src-501-alpha-alpha-source-report)";
    const BETA: &str = "\u{a7}SRC-502-beta \u{2192} \
                        [\u{a7}SRC-502-beta](SRC-502-beta.md#src-502-beta-beta-source-report)";
    const B1: &str = "[\u{a7}FS-b.1](functional-spec/FS-ab.md#1-overview) \u{2192} \
                      [\u{a7}FS-b.1](functional-spec/FS-ab.md#1-overview-1)";

    // §FS-fmt.3.5.1: every kind is named once, in label order.
    #[test]
    fn all_five_kinds_on_one_line_are_named_once_each() {
        assert_eq!(
            rewrite(
                [2, 1, 1, 3],
                &["\u{a7}1 \u{2192} \u{a7}FS-a.1"],
                None,
                &["l"]
            ),
            "trigger \u{2192} marker: \u{a7}1 \u{2192} \u{a7}FS-a.1; bare \u{2192} marker, \
             local section \u{2192} canonical, shorthand \u{2192} canonical, markdown link: l"
        );
        assert_eq!(
            rewrite([0, 3, 0, 0], &[], None, &["a", "b"]),
            "bare \u{2192} marker, markdown link: a, b"
        );
        assert_eq!(rewrite([0; 4], &[], None, &[]), "");
    }

    // §FS-fmt.3.6.2.3: re-pointed and fresh, each alone and in source order.
    #[test]
    fn a_link_row_lists_each_link_before_and_after() {
        assert_eq!(
            rewrite([0; 4], &[], None, &[ALPHA]),
            format!("markdown link: {ALPHA}")
        );
        assert_eq!(
            rewrite([0; 4], &[], None, &[BETA]),
            format!("markdown link: {BETA}")
        );
        assert_eq!(
            rewrite([0; 4], &[], None, &[ALPHA, BETA]),
            format!("markdown link: {ALPHA}, {BETA}")
        );
    }

    // §FS-fmt.3.6.2.2, §FS-fmt.3.6.2.3: a dropped title holding `, ` is one item.
    #[test]
    fn a_title_the_write_drops_stays_inside_its_item() {
        let login = "[\u{a7}FS-login](functional-spec/FS-login.md \"Login, again\") \u{2192} \
                     [\u{a7}FS-login](functional-spec/FS-login.md#fs-login-login)";
        assert_eq!(
            rewrite([0; 4], &[], None, &[login, BETA]),
            format!("markdown link: {login}, {BETA}")
        );
    }

    // §FS-fmt.3.6.2.3: trailing a marking, a trigger, and an expansion's detail.
    #[test]
    fn a_trailing_link_names_its_items_after_its_label() {
        assert_eq!(
            rewrite([0, 1, 0, 0], &[], None, &[BETA]),
            format!("bare \u{2192} marker, markdown link: {BETA}")
        );
        assert_eq!(
            rewrite([1, 0, 0, 0], &[], None, &[BETA]),
            format!("trigger \u{2192} marker, markdown link: {BETA}")
        );
        assert_eq!(
            rewrite(
                [0, 0, 0, 1],
                &["\u{a7}SRC-504 \u{2192} \u{a7}SRC-504-delta"],
                None,
                &[BETA]
            ),
            format!(
                "shorthand \u{2192} canonical: \u{a7}SRC-504 \u{2192} \u{a7}SRC-504-delta; \
                 markdown link: {BETA}"
            )
        );
        assert_eq!(
            rewrite(
                [2, 1, 0, 2],
                &[
                    "\u{a7}SRC-501 \u{2192} \u{a7}SRC-501-alpha",
                    "\u{a7}SRC-503 \u{2192} \u{a7}SRC-503-gamma"
                ],
                None,
                &[ALPHA, BETA]
            ),
            format!(
                "trigger \u{2192} marker: \u{a7}SRC-501 \u{2192} \u{a7}SRC-501-alpha, \
                 \u{a7}SRC-503 \u{2192} \u{a7}SRC-503-gamma; bare \u{2192} marker, \
                 shorthand \u{2192} canonical, markdown link: {ALPHA}, {BETA}"
            )
        );
    }

    // §FS-fmt.3.6.2: the repeated-anchor note comes first, then `; ` and the items,
    // leading or trailing.
    #[test]
    fn a_link_keeps_its_anchor_note_before_its_items() {
        assert_eq!(
            rewrite([0; 4], &[], Some(NOTE), &[B1]),
            format!("markdown link: {NOTE}; {B1}")
        );
        assert_eq!(
            rewrite([0, 1, 0, 0], &[], Some(NOTE), &[BETA, B1]),
            format!("bare \u{2192} marker, markdown link: {NOTE}; {BETA}, {B1}")
        );
    }

    // §FS-fmt.3.5.1, §FS-errors.3: the row an earlier release printed, which named
    // no item, is a prefix of the row that names them.
    #[test]
    fn every_old_row_is_a_prefix_of_its_new_row() {
        let cases = [
            line([0; 4], &[], None, &[ALPHA]),
            line([0; 4], &[], None, &[ALPHA, BETA]),
            line([0, 1, 0, 0], &[], None, &[BETA]),
            line([1, 0, 0, 0], &[], None, &[BETA]),
            line(
                [0, 0, 0, 1],
                &["\u{a7}SRC-504 \u{2192} \u{a7}SRC-504-delta"],
                None,
                &[BETA],
            ),
            line([0; 4], &[], Some(NOTE), &[B1]),
            line([0, 1, 0, 0], &[], Some(NOTE), &[BETA, B1]),
        ];
        for mut rewrite in cases {
            let new = rewrite.label();
            rewrite.links.clear();
            let old = rewrite.label();
            assert!(new.starts_with(&old), "{old:?} is not a prefix of {new:?}");
            assert_ne!(new, old);
        }
    }
}
