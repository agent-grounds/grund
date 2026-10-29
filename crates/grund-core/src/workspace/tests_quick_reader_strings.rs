//! Test module: the readers that read one key of a config without loading it pass over a
//! string that closes early, which the full load refuses from grund 0.22.0, as they pass
//! over any value they cannot read, and record no finding of their own
//! (§FS-config-v2.reader.6.refusal).

use std::path::Path;

use super::members::ancestor_member_entries;
use crate::config::unread_project_name;
use crate::testing::{test_root, write};

/// The config no run loads has no name when its `project_name` closes early.
#[test]
#[ignore = "red until #1094's implementation; implement removes this"]
fn the_unread_project_name_passes_over_a_string_that_closes_early() {
    let text = "grund_config_version = 2\nproject_name = \"one\" \"two\"\n";
    assert_eq!(unread_project_name(text, Path::new("grund.toml")), None);
}

/// The ancestor climb's `members` cannot be obtained when an element closes early, as for
/// any value it cannot read (§FS-workspace.6.1.7.5).
#[test]
#[ignore = "red until #1094's implementation; implement removes this"]
fn the_ancestor_members_pass_over_an_element_that_closes_early() {
    let root = test_root("quick-reader-strings-members");
    let config = root.join("grund.toml");
    write(
        &config,
        "grund_config_version = 2\n\n[workspace]\nmembers = [\"a\" \"b\"]\n",
    );
    assert_eq!(
        ancestor_member_entries(&config),
        Err("`members` is not a list of strings".to_string())
    );
}

/// Guard, green today and kept: a `members` value the readers before refused is passed
/// over as before.
#[test]
fn the_ancestor_members_pass_over_a_value_refused_before() {
    let root = test_root("quick-reader-strings-members-refused-before");
    let config = root.join("grund.toml");
    write(
        &config,
        "grund_config_version = 2\n\n[workspace]\nmembers = [\"target\" \"a,b\"]\n",
    );
    assert_eq!(
        ancestor_member_entries(&config),
        Err("`members` is not a list of strings".to_string())
    );
}
