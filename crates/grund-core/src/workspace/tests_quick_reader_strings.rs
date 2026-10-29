//! Test module: the readers that read one key of a config without loading it read a
//! string that closes early as the load reads it through grund 0.21.x, and record no
//! finding, having no run to report into (§FS-config-v2.reader.6.refusal).

use std::path::Path;

use super::members::ancestor_member_entries;
use crate::config::unread_project_name;
use crate::testing::{test_root, write};

/// Guard, green today and kept: the name of a config no run loads is the one string
/// the load reads, every quote kept.
#[test]
fn the_unread_project_name_reads_a_string_that_closes_early_as_the_load_does() {
    let text = "grund_config_version = 2\nproject_name = \"one\" \"two\"\n";
    assert_eq!(
        unread_project_name(text, Path::new("grund.toml")).as_deref(),
        Some("one\" \"two")
    );
}

/// Guard, green today and kept: the ancestor climb's `members` reads an element that
/// closes early as the one element the load reads.
#[test]
fn the_ancestor_members_read_an_element_that_closes_early_as_the_load_does() {
    let root = test_root("quick-reader-strings-members");
    let config = root.join("grund.toml");
    write(
        &config,
        "grund_config_version = 2\n\n[workspace]\nmembers = [\"a\" \"b\"]\n",
    );
    assert_eq!(
        ancestor_member_entries(&config),
        Ok(vec!["a\" \"b".to_string()])
    );
}
