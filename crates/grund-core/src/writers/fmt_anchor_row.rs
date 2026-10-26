//! The detail a link row carries when a link moved to a repeated heading's
//! numbered anchor (§DF-repeated-heading-anchor.decision.row): a verdict an older
//! grund passed, so the row names the anchor it moves to and the release that
//! numbers it, on the route of a loud, mechanical migration
//! (§REQ-backwards-compatibility.3).

/// The release whose `fmt --cross-refs` first numbers a repeated heading's anchor.
const NUMBERED_SINCE: &str = "0.18";

/// The detail of a line's `markdown link` (§FS-fmt.3.6), from the destinations
/// its link pass re-pointed, `(old, new)`: the repeated-anchor note when every
/// one of them only gained a repeat's number — `#1-overview` → `#1-overview-1`,
/// or `_1` under mkdocs — and `None` when there is none or any other moved. A
/// wrap the pass made fresh has no old anchor and does not decide it, so the note
/// stands whether the link leads its row or trails another kind
/// (§FS-fmt.3.5.1).
pub(super) fn anchor_detail(repointed: &[(String, String)]) -> Option<String> {
    let moved = repointed
        .iter()
        .map(|(old, new)| numbered_move(old, new))
        .collect::<Option<Vec<_>>>()?;
    (!moved.is_empty()).then(|| {
        format!(
            "repeated heading anchor {}, numbered since grund {NUMBERED_SINCE}",
            moved.join(", ")
        )
    })
}

/// `#old → #new` where `new` is `old`'s anchor in the same file with a repeat's
/// number appended, and `None` for any other move.
fn numbered_move(old: &str, new: &str) -> Option<String> {
    let (old_file, old_anchor) = old.split_once('#')?;
    let (new_file, new_anchor) = new.split_once('#')?;
    let suffix = new_anchor.strip_prefix(old_anchor)?;
    let number = suffix.strip_prefix(['-', '_'])?;
    if old_file != new_file || number.is_empty() || !number.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(format!("#{old_anchor} \u{2192} #{new_anchor}"))
}

#[cfg(test)]
mod tests {
    use super::anchor_detail;

    fn moved(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        let owned = pairs
            .iter()
            .map(|(old, new)| (old.to_string(), new.to_string()));
        owned.collect()
    }

    #[test]
    fn a_link_moved_to_the_numbered_anchor_names_both() {
        assert_eq!(
            anchor_detail(&moved(&[("a.md#1-overview", "a.md#1-overview-1")])).as_deref(),
            Some(
                "repeated heading anchor #1-overview \u{2192} #1-overview-1, \
                 numbered since grund 0.18"
            )
        );
    }

    #[test]
    fn any_other_move_keeps_the_plain_row() {
        assert_eq!(anchor_detail(&[]), None);
        assert_eq!(anchor_detail(&moved(&[("a.md#one", "a.md#two")])), None);
        assert_eq!(
            anchor_detail(&moved(&[
                ("a.md#1-overview", "a.md#1-overview-1"),
                ("a.md#one", "b.md#one")
            ])),
            None
        );
    }
}
