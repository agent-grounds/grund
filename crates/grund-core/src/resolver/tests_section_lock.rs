//! Test module: the one judgement of a section lock that `check` reports from and
//! `fmt --write` writes from (§FS-section-lock.write.rules,
//! §FS-section-lock.checks.section-address-changed), over a scanned tree.

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
    let root = test_root(name);
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
    let judgements = judge_section_locks(&findings, config.schema(), config.frame());
    let verdicts = judgements
        .iter()
        .flat_map(|judgement| &judgement.verdicts)
        .map(|(entry, verdict)| (entry.section.clone(), verdict.clone()))
        .collect();
    let unrecorded = judgements
        .iter()
        .flat_map(|judgement| &judgement.unrecorded)
        .map(|(section, _, _)| section.clone())
        .collect();
    (verdicts, unrecorded)
}

#[test]
fn a_wrapped_citation_fingerprints_as_the_bare_one() {
    // b2405f19c414fb23 is `See <§>FS-login.parse.1.`, written bare.
    let (verdicts, unrecorded) = judge(
        "section_lock_wrapped_citation",
        "See [\u{a7}FS-login.parse.1](FS-login.md#1-second).",
        "Split on commas.",
        "See \u{a7}FS-login.format.1.",
        "FS-login.format.1 b2405f19c414fb23\n",
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
    assert_eq!(
        moved,
        [(
            "format.1".to_string(),
            EntryVerdict::Moved {
                to: "parse.1".to_string()
            }
        )]
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
            EntryVerdict::Refresh(Some("b2405f19c414fb23".to_string()))
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

#[test]
fn a_repeated_text_is_written_as_a_dash() {
    let (verdicts, _) = judge(
        "section_lock_repeated",
        "Same.",
        "Same.",
        "See \u{a7}FS-login.format.1.",
        "FS-login.format.1 -\n",
    );
    assert_eq!(verdicts, [("format.1".to_string(), EntryVerdict::Keep)]);
}
