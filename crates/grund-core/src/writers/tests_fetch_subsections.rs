//! Focused tests for the subsection refusal of fetch's output validator (§FS-fetch.3.2): a
//! heading is refused exactly when the next scan reads it as an unmarked heading
//! (§FS-declarations.checks.unmarked-heading), so each case asks the validator and the
//! scanner the same question over the same configuration, and the two cannot drift.

use super::fetch::validate_fetched_declaration;
use crate::config::load_config;
use crate::testing::{numbered_config, scan_findings, test_root, write};

/// What `body` below a file home's declaration gets from the validator, and how many
/// unmarked headings the next scan finds once those bytes are the home.
fn verdicts(name: &str, named_sections: bool, body: &str) -> (Result<(), String>, usize) {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 1\n\n[id]\nnamed_sections = {named_sections}\n\n\
             [[kinds]]\nkind = \"AR\"\nfile = \"docs/arch.md\"\n\n\
             [scan]\ninclude = [\"docs\"]\nrespect_gitignore = false\n"
        ),
    );
    let output = format!("## AR-001-thing: Thing\n\n{body}\n\nprose\n");
    write(&root.join("docs/arch.md"), &output);
    let config = load_config(&root).expect("load config");
    let verdict = validate_fetched_declaration(&output, "AR-001-thing", 2, &config.grammar)
        .map_err(|failure| failure.message);
    let unmarked = scan_findings(&config, &root).unmarked_headings.len();
    (verdict, unmarked)
}

#[test]
fn the_refusal_agrees_with_the_next_scan() {
    for (index, (named_sections, body, refused)) in [
        // A complete section path, read with the project's configuration.
        (true, "### 1. Notes", false),
        (true, "### notes: Notes", false),
        (false, "### 1. Notes", false),
        (false, "### notes: Notes", true),
        // A relative heading: the title syntax its canonical path takes, nothing looser.
        (true, "### .notes: Notes", false),
        (false, "### .notes: Notes", true),
        (true, "### .1. Notes", false),
        (false, "### .1. Notes", false),
        (true, "### .1: Notes", true),
        // No section path at all.
        (true, "### Other", true),
        (true, "### XX-002-other: Other", true),
        (true, "### <§>AR-002-other: Other", true),
        // A heading to the scan though the validator's own test once skipped it.
        (true, "###", true),
        (true, "### ", true),
        // No heading to the scan though it looks like one.
        (true, "    ### Other", false),
        (true, "```\n### Other\n```", false),
        (true, "<pre>\n### Other\n</pre>", false),
        (true, "**Other**", false),
    ]
    .into_iter()
    .enumerate()
    {
        let name = format!("fetch_subsection_agreement_{index}");
        let (verdict, unmarked) = verdicts(&name, named_sections, body);
        assert_eq!(
            verdict.is_err(),
            refused,
            "{body:?} with named_sections = {named_sections}: {verdict:?}"
        );
        assert_eq!(
            unmarked,
            usize::from(refused),
            "the next scan of {body:?} with named_sections = {named_sections}"
        );
    }
}

#[test]
fn the_refusal_names_the_output_line_and_the_heading_as_written() {
    let config = numbered_config(test_root("fetch_subsection_refusal_message"));
    let refusal = validate_fetched_declaration(
        "# AR-001-thing: Thing\n\nprose\n\n  ## Other  \n",
        "AR-001-thing",
        1,
        &config.grammar,
    )
    .expect_err("a folder home holds its subsections to the same reading");
    assert_eq!(
        refusal.message,
        "fetch output line 5: heading `  ## Other` is not a section of `AR-001-thing`; \
         number it, fence it, or make it a bold label"
    );
}
