//! Test module: a locked section's nested headings are fingerprinted with their
//! paths relative to it (§FS-section-lock.holds.fingerprint), so both spellings
//! of a nested heading hash alike and a renumber leaves the fingerprint alone.

use std::collections::BTreeMap;

use super::section_fingerprint::section_fingerprint;
use super::section_lock::judge_section_locks;
use crate::config::load_config;
use crate::testing::{scan_findings, test_root, write};

const CONFIG: &str = "grund_config_version = 1\n\n[reference]\nstrict = true\n\
require_grounding = false\n\n[id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\n\
[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n\n[fmt.cross_refs]\n\
enabled = false\n\n[scan]\ninclude = [\"docs\"]\n";

/// The fingerprint `fmt --write` would enter for `FS-x.1.1`, a relative
/// `### .1. Goals` under `## 1. Overview` whose text is `goals`, cited and unlocked.
fn entered(name: &str, goals: &str) -> String {
    let root = test_root(name);
    write(&root.join("grund.toml"), CONFIG);
    write(
        &root.join("docs/FS-x.md"),
        &format!("# FS-x: Example\n\n## 1. Overview\n\n### .1. Goals\n\n{goals}\n"),
    );
    write(&root.join("docs/use.md"), "# Use\n\nSee \u{a7}FS-x.1.1.\n");
    let config = load_config(&root).expect("load config");
    let findings = scan_findings(&config, &root);
    let judgements =
        judge_section_locks(&findings, config.schema(), config.frame(), &BTreeMap::new());
    let unrecorded = judgements
        .iter()
        .flat_map(|judgement| &judgement.unrecorded)
        .map(|(section, _, fingerprint)| (section.clone(), fingerprint.clone()))
        .collect::<Vec<_>>();
    match unrecorded.as_slice() {
        [(section, Some(fingerprint))] if section == "1.1" => fingerprint.clone(),
        other => panic!("expected one unrecorded FS-x.1.1, got {other:?}"),
    }
}

#[test]
fn both_spellings_of_a_nested_heading_hash_alike() {
    let complete = entered(
        "fingerprint_complete_child",
        "Goals.\n\n#### 1.1.1. Latency\n\nLatency.",
    );
    let relative = entered(
        "fingerprint_relative_child",
        "Goals.\n\n#### .1. Latency\n\nLatency.",
    );
    assert_eq!(complete, relative);
}

#[test]
fn the_level_and_title_are_kept_and_the_path_is_relative() {
    let complete = entered(
        "fingerprint_kept_level_title",
        "Goals.\n\n#### 1.1.1. Latency\n\nLatency.",
    );
    assert_eq!(
        complete,
        section_fingerprint("Goals.\n\n#### .1. Latency\n\nLatency.")
    );
}

#[test]
fn a_deeper_relative_heading_hashes_with_its_path_below_the_section() {
    let deep = entered(
        "fingerprint_deeper_relative",
        "Goals.\n\n#### .1. Latency\n\nLatency.\n\n##### .1. P95\n\nP95.",
    );
    assert_eq!(
        deep,
        section_fingerprint("Goals.\n\n#### .1. Latency\n\nLatency.\n\n##### .1.1. P95\n\nP95.")
    );
}

#[test]
fn a_heading_shaped_line_the_map_does_not_record_hashes_as_written() {
    // A fenced line and a dotted line of prose are no headings of the section map.
    let text =
        "Goals.\n\n```markdown\n#### 1.1.1. Latency\n```\n\nThe text `#### 1.1.2. Other` stays.";
    assert_eq!(
        entered("fingerprint_fenced_heading", text),
        section_fingerprint(text)
    );
}
