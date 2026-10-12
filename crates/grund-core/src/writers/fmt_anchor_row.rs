//! The dry-run row of a link that moved to a repeated heading's numbered anchor
//! (§DF-repeated-heading-anchor.decision.row): a verdict an older grund passed, so
//! the row names the anchor it moves to and the release that numbers it, on the
//! route of a loud, mechanical migration (§REQ-backwards-compatibility.3).

use std::borrow::Cow;

use once_cell::sync::Lazy;
use regex::Regex;

/// The release whose `fmt --cross-refs` first numbers a repeated heading's anchor.
const NUMBERED_SINCE: &str = "0.18";

/// A Markdown inline link's destination, `#` fragment and all.
static LINK_TARGET: Lazy<Regex> = Lazy::new(|| Regex::new(r"\]\(([^)\s]*)\)").unwrap());

/// The row a line's link rewrite names (§FS-fmt.3.5.1): none when the links left
/// `before` as it was, the repeated-anchor row when a repeat's numbering is all
/// that moved them, and the plain `markdown link` otherwise.
pub(super) fn link_row(before: &str, after: &str) -> Option<Cow<'static, str>> {
    (before != after).then(|| {
        repeated_anchor_row(before, after).map_or(Cow::Borrowed("markdown link"), Cow::Owned)
    })
}

/// The row for a line whose only change is links whose old anchor is the
/// unnumbered slug of the anchor they now take — `#1-overview` → `#1-overview-1`,
/// or `_1` under mkdocs — or `None` when anything else on the line changed.
fn repeated_anchor_row(before: &str, after: &str) -> Option<String> {
    let old = LINK_TARGET.captures_iter(before).collect::<Vec<_>>();
    let new = LINK_TARGET.captures_iter(after).collect::<Vec<_>>();
    if old.len() != new.len() || LINK_TARGET.split(before).ne(LINK_TARGET.split(after)) {
        return None;
    }
    let mut moved = Vec::new();
    for (old, new) in old.iter().zip(&new) {
        let (old, new) = (&old[1], &new[1]);
        if old == new {
            continue;
        }
        let (old_file, old_anchor) = old.split_once('#')?;
        let (new_file, new_anchor) = new.split_once('#')?;
        let suffix = new_anchor.strip_prefix(old_anchor)?;
        let number = suffix.strip_prefix(['-', '_'])?;
        if old_file != new_file || number.is_empty() || !number.bytes().all(|b| b.is_ascii_digit())
        {
            return None;
        }
        moved.push(format!("#{old_anchor} \u{2192} #{new_anchor}"));
    }
    (!moved.is_empty()).then(|| {
        format!(
            "markdown link: repeated heading anchor {}, numbered since grund {NUMBERED_SINCE}",
            moved.join(", ")
        )
    })
}

#[cfg(test)]
mod tests {
    use super::repeated_anchor_row;

    #[test]
    fn a_link_moved_to_the_numbered_anchor_names_both() {
        assert_eq!(
            repeated_anchor_row(
                "See [\u{a7}FS-b.1](a.md#1-overview).",
                "See [\u{a7}FS-b.1](a.md#1-overview-1)."
            )
            .as_deref(),
            Some(
                "markdown link: repeated heading anchor #1-overview \u{2192} #1-overview-1, \
                 numbered since grund 0.18"
            )
        );
    }

    #[test]
    fn any_other_change_keeps_the_plain_row() {
        assert_eq!(
            repeated_anchor_row(
                "See \u{a7}FS-b.1.",
                "See [\u{a7}FS-b.1](a.md#1-overview-1)."
            ),
            None
        );
        assert_eq!(
            repeated_anchor_row(
                "See [\u{a7}FS-b.1](a.md#one).",
                "See [\u{a7}FS-b.1](a.md#two)."
            ),
            None
        );
    }
}
