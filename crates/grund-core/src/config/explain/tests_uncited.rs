//! §FS-config-v2.rules.citations.uncited, §FS-config.3.9.1.4: the obligation
//! keys' answers say that a file of a place or the complement that cites
//! nothing is not judged, and name the setting that makes it a finding
//! (agent-grounds/grund#151).

use super::*;

fn meaning(version: u32, target: &str) -> String {
    match answer(version, &ExplainAsk::Target(target.to_string())) {
        ExplainAnswer::Key(key) => key.meaning,
        other => panic!("not a key: {other:?}"),
    }
}

/// The second sentence every version-2 obligation key carries.
const V2_UNCITED: &str = " A file of a place or the complement that cites nothing is not checked by this key; `[rules.citations.<ROW>.grounding]` makes it a finding.";

/// The second sentence every version-1 obligation key carries.
const V1_UNCITED: &str = " A file of a place or the complement that cites nothing is not checked by this key; `require_grounding` makes it a finding.";

#[test]
#[ignore = "red until #151 the obligation keys name the uncited file; implement removes this"]
fn a_v2_obligation_key_says_an_uncited_file_is_not_checked() {
    for (key, first) in [
        ("must", "Kinds this one must cite; a miss is an error."),
        ("warn", "Kinds this one must cite; a miss is a warning."),
        (
            "should",
            "Kinds this one should cite; a miss is a suggestion.",
        ),
    ] {
        assert_eq!(
            meaning(2, &format!("[rules.citations.e2e] {key}")),
            format!("{first}{V2_UNCITED}"),
            "{key}"
        );
    }
}

#[test]
#[ignore = "red until #151 the obligation keys name the uncited file; implement removes this"]
fn a_v1_obligation_key_says_an_uncited_file_is_not_checked() {
    for (key, first) in [
        (
            "must",
            "Kinds each declaration (or file, for a place) must cite; a miss is an error.",
        ),
        ("should", "Kinds it should cite; a miss is a suggestion."),
    ] {
        assert_eq!(
            meaning(1, &format!("[citations.e2e] {key}")),
            format!("{first}{V1_UNCITED}"),
            "{key}"
        );
    }
}
