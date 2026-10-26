//! Test module: the one judgement of a section lock that `check` reports from and
//! `fmt --write` writes from (§FS-section-lock.write.rules,
//! §FS-section-lock.checks.section-address-changed), over a scanned tree.

use std::collections::BTreeMap;

use super::section_lock::{EntryVerdict, judge_section_locks};
use crate::config::load_config;
use crate::testing::{scan_findings, test_root, write};

const CONFIG: &str = "grund_config_version = 1\n\n[reference]\nstrict = true\n\
require_grounding = false\n\n[id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\n\
[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n\n[scan]\ninclude = [\"docs\"]\n";

/// `FS-login` with `## format:` and `## parse:`, each holding one `### .1`
/// whose text is given, `docs/use.md` citing `cites`, and the lock holding
/// `lock`; then each entry's verdict and each unrecorded address.
fn judge(
    name: &str,
    format: &str,
    parse: &str,
    cites: &str,
    lock: &str,
) -> (Vec<(String, EntryVerdict)>, Vec<String>) {
    let (verdicts, unrecorded) = judge_in(name, [format, parse], cites, lock, &[]);
    (
        verdicts,
        unrecorded.into_iter().map(|(section, _)| section).collect(),
    )
}

/// `judge` over a tree that also holds `extra` as `(path, text)`, each unrecorded
/// address with the fingerprint `fmt --write` would enter for it.
fn judge_in(
    name: &str,
    [format, parse]: [&str; 2],
    cites: &str,
    lock: &str,
    extra: &[(&str, &str)],
) -> (Vec<(String, EntryVerdict)>, Vec<(String, Option<String>)>) {
    let root = test_root(name);
    for (path, text) in extra {
        write(&root.join(path), text);
    }
    write(&root.join("grund.toml"), CONFIG);
    write(
        &root.join("docs/FS-login.md"),
        &format!(
            "# FS-login: Login\n\n## format: Formatting\n\n### .1: First\n\n{format}\n\n\
             ## parse: Parsing\n\n### .1: Second\n\n{parse}\n"
        ),
    );
    write(&root.join("docs/use.md"), &format!("# Use\n\n{cites}\n"));
    if !lock.is_empty() {
        write(&root.join(".grund/sections/FS-login.lock"), lock);
    }
    let config = load_config(&root).expect("load config");
    let findings = scan_findings(&config, &root);
    let judgements =
        judge_section_locks(&findings, config.schema(), config.frame(), &BTreeMap::new());
    let verdicts = judgements
        .iter()
        .flat_map(|judgement| &judgement.verdicts)
        .map(|(entry, verdict)| (entry.section.clone(), verdict.clone()))
        .collect();
    let unrecorded = judgements
        .iter()
        .flat_map(|judgement| &judgement.unrecorded)
        .map(|(section, _, fingerprint)| (section.clone(), fingerprint.clone()))
        .collect();
    (verdicts, unrecorded)
}

#[test]
fn a_wrapped_citation_fingerprints_as_the_bare_one() {
    // c8dd830ca673266a is `See <§>FS-login.parse.1.`, written bare.
    let (verdicts, unrecorded) = judge(
        "section_lock_wrapped_citation",
        "See [\u{a7}FS-login.parse.1](FS-login.md#1-second).",
        "Split on commas.",
        "See \u{a7}FS-login.format.1.",
        "FS-login.format.1 c8dd830ca673266a\n",
    );
    assert_eq!(verdicts, [("format.1".to_string(), EntryVerdict::Keep)]);
    // The pointer-shaped citation is a citation like any other.
    assert_eq!(unrecorded, ["parse.1"]);
}

#[test]
fn a_cited_address_without_an_entry_is_unrecorded_and_an_uncited_one_drops() {
    let (verdicts, unrecorded) = judge(
        "section_lock_add_and_drop",
        "Trim surrounding whitespace.",
        "Split on commas.",
        "See \u{a7}FS-login.parse.1.",
        "FS-login.format.1 8f332b6fe87bb346\n",
    );
    assert_eq!(verdicts, [("format.1".to_string(), EntryVerdict::Drop)]);
    assert_eq!(unrecorded, ["parse.1"]);
}

#[test]
fn text_that_left_for_another_address_is_a_move_and_an_edit_a_refresh() {
    let (moved, _) = judge(
        "section_lock_swap",
        "Split on commas.",
        "Trim surrounding whitespace.",
        "See \u{a7}FS-login.format.1.",
        "FS-login.format.1 8f332b6fe87bb346\n",
    );
    assert!(
        matches!(&moved[..], [(section, EntryVerdict::Moved { to, .. })]
            if section == "format.1" && to == "parse.1")
    );
    let (edited, _) = judge(
        "section_lock_edit_in_place",
        "Trim all surrounding whitespace.",
        "Split on commas.",
        "See \u{a7}FS-login.format.1.",
        "FS-login.format.1 8f332b6fe87bb346\n",
    );
    assert!(matches!(edited[..], [(_, EntryVerdict::Refresh(Some(_)))]));
}

/// §FS-section-lock.checks.section-address-changed: a move carries the lines that
/// still cite the recorded address, one per line, sorted by path then line.
#[test]
fn a_move_carries_the_lines_still_citing_the_old_address() {
    let (moved, _) = judge_in(
        "section_lock_swap_sites",
        ["Split on commas.", "Trim surrounding whitespace."],
        "See \u{a7}FS-login.format.1 and \u{a7}FS-login.format.1.\n\nAgain \u{a7}FS-login.format.1.",
        "FS-login.format.1 8f332b6fe87bb346\n",
        &[("docs/a.md", "# A\n\nSee \u{a7}FS-login.format.1.\n")],
    );
    let [(_, EntryVerdict::Moved { sites, .. })] = &moved[..] else {
        panic!("expected one move, got {moved:?}");
    };
    let sites = sites
        .iter()
        .map(|(path, line)| {
            let name = path.file_name().expect("a file").to_string_lossy();
            (name.into_owned(), *line)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        sites,
        [
            ("a.md".to_string(), 3),
            ("use.md".to_string(), 3),
            ("use.md".to_string(), 5)
        ]
    );
}

#[test]
fn a_pointer_acknowledges_the_move() {
    let (verdicts, unrecorded) = judge(
        "section_lock_pointer",
        "See \u{a7}FS-login.parse.1.",
        "Trim surrounding whitespace.",
        "See \u{a7}FS-login.format.1.",
        "FS-login.format.1 8f332b6fe87bb346\n",
    );
    assert_eq!(
        verdicts,
        [(
            "format.1".to_string(),
            EntryVerdict::Refresh(Some("c8dd830ca673266a".to_string()))
        )]
    );
    assert_eq!(unrecorded, ["parse.1"]);
}

#[test]
fn of_a_coordinate_entered_twice_the_held_entry_stays() {
    let (verdicts, _) = judge(
        "section_lock_twice",
        "Trim surrounding whitespace.",
        "Split on commas.",
        "See \u{a7}FS-login.format.1.",
        "FS-login.format.1 be879e46c73e3d05\nFS-login.format.1 8f332b6fe87bb346\n",
    );
    assert_eq!(
        verdicts,
        [
            ("format.1".to_string(), EntryVerdict::Drop),
            ("format.1".to_string(), EntryVerdict::Keep),
        ]
    );
}

/// §FS-section-lock.holds.repeated: a text the declaration repeats is entered as
/// `-`, and the entry takes the fingerprint once the texts differ.
#[test]
fn a_repeated_text_is_written_as_a_dash_until_it_differs() {
    let cites = "See \u{a7}FS-login.format.1.";
    let (_, added) = judge_in("section_lock_repeated", ["Same.", "Same."], cites, "", &[]);
    assert_eq!(added, [("format.1".to_string(), None)]);
    let (differ, _) = judge(
        "section_lock_repeated_differ",
        "Trim surrounding whitespace.",
        "Split on commas.",
        cites,
        "FS-login.format.1 -\n",
    );
    assert_eq!(
        differ,
        [(
            "format.1".to_string(),
            EntryVerdict::Refresh(Some("8f332b6fe87bb346".to_string()))
        )]
    );
}

/// §FS-section-lock.limits.identical: while two sections hold one text, an entry
/// of it is kept whichever of them holds it, so swapping them is not seen.
#[test]
fn identical_texts_cannot_be_told_apart() {
    let (verdicts, _) = judge(
        "section_lock_identical",
        "Same.",
        "Same.",
        "See \u{a7}FS-login.format.1.",
        "FS-login.format.1 -\n",
    );
    assert_eq!(verdicts, [("format.1".to_string(), EntryVerdict::Keep)]);
}

/// §FS-section-lock.limits.gate: an edit `fmt --write` never recorded leaves the
/// old fingerprint, so moving the edited text away reads as an edit in place.
#[test]
fn an_unrecorded_edit_hides_a_later_move() {
    let (verdicts, _) = judge(
        "section_lock_unrefreshed",
        "Split on commas.",
        "Trim all surrounding whitespace.",
        "See \u{a7}FS-login.format.1.",
        "FS-login.format.1 8f332b6fe87bb346\n",
    );
    assert!(
        matches!(verdicts[..], [(_, EntryVerdict::Refresh(Some(_)))]),
        "{verdicts:?}"
    );
}

/// §FS-section-lock.limits.across: the recorded text now in another declaration
/// is not looked for there, so the entry reads as edited in place.
#[test]
fn a_text_moved_into_another_declaration_is_not_found() {
    let signup = "# FS-signup: Signup\n\n## form: Form\n\n### .1: Name\n\n\
                  Trim surrounding whitespace.\n";
    let (verdicts, _) = judge_in(
        "section_lock_across",
        ["Split on commas.", "Lower case."],
        "See \u{a7}FS-login.format.1.",
        "FS-login.format.1 8f332b6fe87bb346\n",
        &[("docs/FS-signup.md", signup)],
    );
    assert!(
        matches!(verdicts[..], [(_, EntryVerdict::Refresh(Some(_)))]),
        "{verdicts:?}"
    );
}
