//! Test module: a prohibition's targets join with "or" (§FS-init.2.3.5.4,
//! §DF-prohibition-targets-join-with-or). Each entry of a `must-not` or `should-not` list is
//! banned on its own, so "and" would read as forbidding only the pair; the case
//! is this repository's own `README.md` row (agent-grounds/grund#151).

use super::*;
use crate::config::load_config;
use crate::testing::{test_root, write};

/// A v2 config with five citable kinds, the single-file place `readme` at
/// `README.md` and the folder place `guide` at `docs/guide/`, and `rules`.
fn config(rules: &str) -> String {
    format!(
        r#"grund_config_version = 2
project_name = "demo"

[schema.kinds.FS]
folders = ["docs/fs"]

[schema.kinds.AR]
folders = ["docs/ar"]

[schema.kinds.RM]
files = ["docs/roadmap.md"]

[schema.kinds.GOAL]
files = ["docs/goals.md"]

[schema.kinds.DA]
folders = ["docs/da"]

[schema.kinds.readme]
files = ["README.md"]
citable = false

[schema.kinds.guide]
folders = ["docs/guide"]
citable = false

{rules}"#
    )
}

/// Render the whole section, every row at the root.
fn render(name: &str, rules: &str) -> String {
    let root = test_root(name);
    write(&root.join("grund.toml"), &config(rules));
    let config = load_config(&root).expect("load config");
    citation_directions_section(config.project(), config.run()).expect("the config declares rules")
}

/// The single-file subject of this repository's `README.md` row:
/// `must-not = ["FS", "AR"]` forbids a citation to either.
#[test]
fn a_single_file_prohibition_joins_its_targets_with_or() {
    let section = render(
        "prohibition_single_file",
        "[rules.citations.readme]\nmust-not = [\"FS\", \"AR\"]\n",
    );
    assert!(
        section.contains("- The file **README.md**") && section.contains("must not cite FS or AR."),
        "{section}"
    );
    assert!(!section.contains("FS and AR"), "{section}");
}

/// The folder subject, at both prohibition strengths, with the Oxford comma
/// from three targets on.
#[test]
fn a_folder_prohibition_joins_its_targets_with_or() {
    let section = render(
        "prohibition_folder",
        "[rules.citations.guide]\nshould-not = [\"FS\", \"AR\", \"RM\"]\n",
    );
    assert!(
        section.contains("- Each file in **docs/guide/**")
            && section.contains("should not cite FS, AR, or RM."),
        "{section}"
    );
    assert!(!section.contains("AR, and RM"), "{section}");
}

/// A prohibition after an obligation keeps the short form, and its entries
/// join as alternatives while the obligation's still conjoin; an entry's own
/// alternatives join the same list, unparenthesised.
#[test]
fn a_following_prohibition_joins_its_entries_as_alternatives() {
    let section = render(
        "prohibition_following",
        "[rules.citations.guide]\nmust = [\"FS\", \"GOAL\"]\nmust-not = [\"AR|RM\", \"DA\"]\n",
    );
    assert!(
        section.contains("must cite FS and GOAL; never cite AR, RM, or DA."),
        "{section}"
    );
}
