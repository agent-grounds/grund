//! Test module: a citation inside a link its author wrote, read as a kind-index
//! entry (§FS-citations.wrap.enclosing, §FS-check.3.18.5). The entry is a wrap or
//! a citation in the text of an enclosing link whose destination is not empty;
//! a bare citation `fmt` would wrap is still §FS-check.3.17's, and anything else
//! still falls to §FS-check.3.18. Each case is one side of the three-way reading:
//! which brackets enclose (§FS-citations.wrap.enclosing.shape), which never do
//! (§FS-citations.wrap.enclosing.images), and the nested line an older `fmt`
//! wrote (§FS-citations.wrap.enclosing.wrap-wins).

use crate::api::CheckRun;
use crate::testing::{check_run, codes, findings, kind_index_repo, kind_index_repo_loose, write};

/// The clause §FS-check.3.18.5 names under `strict = true`, in the bytes it has
/// shipped in: one form that is enough, not the definition of an entry.
const MENTION_CLAUSE: &str =
    "but not as an entry: an entry is a `§`-marked Markdown link to the declaration";

fn has(run: &CheckRun, code: &str) -> bool {
    codes(run).iter().any(|fired| fired == code)
}

/// The index of `kind_index_repo` holding `index`, checked.
fn checked_index(name: &str, index: &str) -> CheckRun {
    let root = kind_index_repo(name);
    write(&root.join("docs/specs/README.md"), index);
    check_run(&root, false)
}

/// Asserts the index holds an entry for every covered ID and reports none of them.
fn assert_entry(run: &CheckRun, why: &str) {
    assert!(
        !has(run, "unlinked-index-entry") && !has(run, "missing-index-entry"),
        "{why}: {:?}",
        findings(run)
    );
}

/// §FS-citations.wrap.enclosing / §FS-check.3.18.5: an index row whose link the
/// author wrote, its text the citation and a title, is the entry. `fmt` leaves it,
/// so `check` may not name `fmt` (§FS-check.3.17.4), and it is a link a reader can
/// follow, so the ID is not missing either.
#[test]
fn an_authored_label_is_an_entry() {
    let run = checked_index(
        "an_authored_label_is_an_entry",
        "# Specs\n\n- [§FS-001-login: A user logs in](FS-001-login.md)\n",
    );
    assert_entry(&run, "the author's link is the entry");
}

/// §FS-check.3.17.1 / §FS-check.3.18.5: the three readings side by side — a wrap
/// is an entry, a citation in an enclosing link is an entry, and a bare citation
/// is the one `unlinked-index-entry` reports, at its own line.
#[test]
fn a_wrap_and_an_authored_label_are_entries_and_a_bare_citation_is_not_a_link() {
    let root = kind_index_repo(
        "a_wrap_and_an_authored_label_are_entries_and_a_bare_citation_is_not_a_link",
    );
    write(
        &root.join("docs/specs/FS-002-logout.md"),
        "# FS-002-logout: A user logs out\n\nBody.\n",
    );
    write(
        &root.join("docs/specs/FS-003-reset.md"),
        "# FS-003-reset: A user resets a password\n\nBody.\n",
    );
    write(
        &root.join("docs/specs/README.md"),
        "# Specs\n\n\
         - [§FS-001-login](FS-001-login.md#fs-001-login-a-user-logs-in): A user logs in\n\
         - [§FS-002-logout: A user logs out](FS-002-logout.md)\n\
         - §FS-003-reset: A user resets a password\n",
    );

    let run = check_run(&root, false);
    let unlinked: Vec<_> = findings(&run)
        .into_iter()
        .filter(|line| line.contains("is not a link"))
        .collect();
    assert_eq!(
        unlinked.len(),
        1,
        "only the bare citation is not a link: {:?}",
        findings(&run)
    );
    assert!(
        unlinked[0].starts_with("docs/specs/README.md:5:") && unlinked[0].contains("FS-003-reset"),
        "the finding is the bare row's: {unlinked:?}"
    );
    assert!(!has(&run, "missing-index-entry"), "{:?}", findings(&run));
}

/// §FS-citations.wrap.enclosing: each citation in the text counts for its own ID,
/// and a citation repeated in one text is one entry, not two findings.
#[test]
fn each_citation_in_one_label_counts_for_its_own_id() {
    let root = kind_index_repo("each_citation_in_one_label_counts_for_its_own_id");
    write(
        &root.join("docs/specs/FS-002-logout.md"),
        "# FS-002-logout: A user logs out\n\nBody.\n",
    );
    write(
        &root.join("docs/specs/README.md"),
        "# Specs\n\n\
         - [§FS-001-login, then §FS-002-logout, then §FS-001-login again](session.md)\n",
    );

    let run = check_run(&root, false);
    assert_entry(&run, "one link is the entry of both IDs it holds");
}

/// §FS-check.3.17.5 / §FS-citations.wrap.enclosing: off strict mode an unmarked
/// token is a citation, and a link its author wrote around it with a title is an
/// entry, as a hand-written `[FS-x](…)` already is.
#[test]
fn off_strict_an_unmarked_citation_in_an_authored_label_is_an_entry() {
    let root =
        kind_index_repo_loose("off_strict_an_unmarked_citation_in_an_authored_label_is_an_entry");
    write(
        &root.join("docs/specs/README.md"),
        "# Specs\n\n- [FS-001-login: A user logs in](FS-001-login.md)\n",
    );

    let run = check_run(&root, false);
    assert_entry(
        &run,
        "an unmarked citation in the author's link is the entry",
    );
}

/// §FS-citations.wrap.enclosing.shape: every shape the reading admits — an escaped
/// `]` and balanced brackets in the text, a code span in it, an escaped `)` and
/// balanced parentheses in the destination — is the entry.
#[test]
fn every_enclosing_shape_is_an_entry() {
    let rows = [
        (
            "escaped_bracket",
            r"- [§FS-001-login: logs in \] for good](FS-001-login.md)",
        ),
        (
            "balanced_brackets",
            "- [§FS-001-login: logs in [draft]](FS-001-login.md)",
        ),
        (
            "code_span",
            "- [§FS-001-login: run `grund check` then `]`](FS-001-login.md)",
        ),
        (
            "escaped_paren",
            r"- [§FS-001-login: A user logs in](FS-001-login\).md)",
        ),
        (
            "balanced_parens",
            "- [§FS-001-login: A user logs in](FS-001-login(1).md)",
        ),
        (
            "words_before",
            "- [the §FS-001-login record](FS-001-login.md)",
        ),
    ];
    for (shape, row) in rows {
        let run = checked_index(
            &format!("every_enclosing_shape_is_an_entry_{shape}"),
            &format!("# Specs\n\n{row}\n"),
        );
        assert_entry(&run, &format!("{shape}: `{row}` encloses its citation"));
    }
}

/// §FS-citations.wrap.enclosing / §FS-check.3.18.5: an enclosing link with an
/// empty destination goes nowhere, so it is no entry and the ID falls to
/// §FS-check.3.18 in the bytes that finding has shipped in — and never to
/// §FS-check.3.17, whose command now leaves the link alone.
#[test]
fn an_empty_destination_is_no_entry() {
    let run = checked_index(
        "an_empty_destination_is_no_entry",
        "# Specs\n\n- [§FS-001-login: A user logs in]()\n",
    );
    assert!(!has(&run, "unlinked-index-entry"), "{:?}", findings(&run));
    let missing: Vec<_> = findings(&run)
        .into_iter()
        .filter(|line| line.contains(MENTION_CLAUSE))
        .collect();
    assert_eq!(missing.len(), 1, "{:?}", findings(&run));
}

/// §FS-citations.wrap.enclosing.shape: a backslash-escaped `[` opens nothing, so
/// the citation is bare and `fmt` wraps it — §FS-check.3.17 still reports it.
#[test]
fn an_escaped_opener_encloses_nothing() {
    let run = checked_index(
        "an_escaped_opener_encloses_nothing",
        "# Specs\n\n- \\[§FS-001-login: A user logs in](FS-001-login.md)\n",
    );
    assert!(has(&run, "unlinked-index-entry"), "{:?}", findings(&run));
}

/// §FS-citations.wrap.enclosing.shape: a `[` inside an inline code span opens
/// nothing either.
#[test]
fn a_bracket_in_a_code_span_encloses_nothing() {
    let run = checked_index(
        "a_bracket_in_a_code_span_encloses_nothing",
        "# Specs\n\n- `[` §FS-001-login: A user logs in](FS-001-login.md)\n",
    );
    assert!(has(&run, "unlinked-index-entry"), "{:?}", findings(&run));
}

/// §FS-citations.wrap.enclosing.shape: a destination that never closes on its
/// line makes no link, so the citation before it is bare.
#[test]
fn an_unclosed_destination_makes_no_link() {
    let run = checked_index(
        "an_unclosed_destination_makes_no_link",
        "# Specs\n\n- [§FS-001-login: A user logs in](FS-001-login.md\n",
    );
    assert!(has(&run, "unlinked-index-entry"), "{:?}", findings(&run));
}

/// §FS-citations.wrap.enclosing.images: an image's description never encloses,
/// so a citation in one is bare, never an entry.
#[test]
fn an_image_is_never_an_entry() {
    let run = checked_index(
        "an_image_is_never_an_entry",
        "# Specs\n\n- ![a diagram of §FS-001-login](diagram.png)\n",
    );
    assert!(has(&run, "unlinked-index-entry"), "{:?}", findings(&run));
}

/// §FS-citations.checks.image-index-entry: a citation that is an image's whole
/// description is no link to the declaration (§FS-citations.wrap.enclosing.images),
/// and from grund 0.20.0 it keeps no credit. The index mentions the ID and holds no
/// entry for it, so the ID is §FS-check.3.18's, in the "appears" words of
/// §FS-check.3.18.5, never §FS-check.3.17's, and the spent code never fires.
#[test]
fn a_citation_only_image_is_no_entry() {
    let run = checked_index(
        "a_citation_only_image_is_no_entry",
        "# Specs\n\n- ![§FS-001-login](diagram.png)\n",
    );
    assert!(!has(&run, "image-index-entry"), "{:?}", findings(&run));
    assert!(!has(&run, "unlinked-index-entry"), "{:?}", findings(&run));
    assert!(has(&run, "missing-index-entry"), "{:?}", findings(&run));
    let missing: Vec<_> = findings(&run)
        .into_iter()
        .filter(|line| line.contains(MENTION_CLAUSE))
        .collect();
    assert_eq!(missing.len(), 1, "{:?}", findings(&run));
}

/// §FS-citations.checks.image-index-entry: an image beside another entry for the
/// same ID changes nothing: the wrap is the entry, and nothing is reported.
#[test]
fn a_citation_only_image_beside_an_entry_draws_no_warning() {
    let run = checked_index(
        "a_citation_only_image_beside_an_entry_draws_no_warning",
        "# Specs\n\n- ![§FS-001-login](diagram.png)\n\
         - [§FS-001-login](FS-001-login.md#fs-001-login-a-user-logs-in): A user logs in\n",
    );
    assert_entry(&run, "the wrap is the entry");
    assert!(!has(&run, "image-index-entry"), "{:?}", findings(&run));
}

/// §FS-citations.wrap.enclosing.wrap-wins: on a line an older `fmt` nested, the
/// inner wrap is the entry and the outer brackets are text.
#[test]
fn an_already_nested_line_is_an_entry() {
    let run = checked_index(
        "an_already_nested_line_is_an_entry",
        "# Specs\n\n\
         - [[§FS-001-login](FS-001-login.md#fs-001-login-a-user-logs-in): A user logs in](FS-001-login.md)\n",
    );
    assert_entry(&run, "the inner wrap is the entry");
}

/// §FS-citations.wrap.enclosing: the citation in the text is still checked, so a
/// dangling one keeps its finding, and so does one naming a missing section.
#[test]
fn a_dangling_or_missing_section_citation_in_a_label_keeps_its_finding() {
    let run = checked_index(
        "a_dangling_or_missing_section_citation_in_a_label_keeps_its_finding",
        "# Specs\n\n\
         - [§FS-001-login](FS-001-login.md#fs-001-login-a-user-logs-in): A user logs in\n\
         - [§FS-009-gone: A spec that is gone](FS-009-gone.md)\n\
         - [§FS-001-login.9: The ninth section](FS-001-login.md)\n",
    );
    let lines = findings(&run);
    assert!(
        lines.iter().any(|line| line.starts_with("docs/specs/README.md:4:") && line.contains("FS-009-gone")),
        "the dangling citation is reported at its line: {lines:?}"
    );
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("docs/specs/README.md:5:")
                && line.contains("FS-001-login.9")),
        "the missing section is reported at its line: {lines:?}"
    );
}
