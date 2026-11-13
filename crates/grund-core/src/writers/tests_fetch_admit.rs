//! Focused tests for fetch's admission (§FS-fetch.admission): each clause refuses what the
//! selected project's own scan reads back as something other than what was asked for, the
//! candidate is spliced where the scan put the requested declaration (§FS-fetch.4,
//! §FS-fetch.5), and a finding the home already held is no reason to refuse.

use super::tests_fetch_readback::{At, admit_during};
use crate::config::load_config;
use crate::testing::{scan_findings, test_root, write};

pub(super) fn file_toml(named_sections: bool) -> String {
    format!(
        "grund_config_version = 1\n\n[id]\nnamed_sections = {named_sections}\n\n\
         [[kinds]]\nkind = \"AR\"\nfile = \"docs/arch.md\"\n\n\
         [scan]\ninclude = [\"docs\"]\nrespect_gitignore = false\n"
    )
}

pub(super) const FOLDER_TOML: &str = "grund_config_version = 1\n\n[id]\nnamed_sections = true\n\n\
     [[kinds]]\nkind = \"AR\"\nfolder = \"docs/ar\"\n\n\
     [scan]\ninclude = [\"docs\"]\nrespect_gitignore = false\n";

/// What admission makes of `output` for `local` in a tree of `files`: the bytes it would
/// install, or the refusal's line.
fn admit_into(
    name: &str,
    toml: &str,
    at: At,
    files: &[(&str, &str)],
    local: &str,
    output: &str,
) -> Result<String, String> {
    admit_during(name, toml, at, files, &[], local, output)
}

/// A file home holding `home`, admitting `output` for `AR-001-thing`.
pub(super) fn into_file(name: &str, home: &str, output: &str) -> Result<String, String> {
    admit_into(
        name,
        &file_toml(true),
        At::File("docs/arch.md"),
        &[("docs/arch.md", home)],
        "AR-001-thing",
        output,
    )
}

/// An empty file home, admitting `output` for `AR-001-thing`.
fn alone(name: &str, output: &str) -> Result<String, String> {
    into_file(name, "", output)
}

// §FS-fetch.admission.1: one declaration, the requested one, at the home's level.

#[test]
fn a_nested_declaration_is_refused_by_its_id_not_its_depth() {
    let refusal = alone(
        "admit_nested_two_below",
        "## AR-001-thing: Thing\n\n#### AR-002-other: Other\n",
    )
    .expect_err("a nested declaration must be refused");
    assert_eq!(
        refusal,
        "fetch output nests declaration `AR-002-other` inside requested `AR-001-thing`; \
         fetch output must hold only the requested declaration"
    );
}

#[test]
fn the_requested_id_nested_again_is_refused() {
    let refusal = alone(
        "admit_nested_requested",
        "## AR-001-thing: Thing\n\n### AR-001-thing: Again\n",
    )
    .expect_err("the requested ID nested again must be refused");
    assert!(
        refusal.contains("nests declaration `AR-001-thing`"),
        "{refusal}"
    );
}

#[test]
fn a_nested_declaration_after_a_tab_or_on_a_comment_line_is_refused() {
    for (index, line) in ["###\tAR-002-other: Other", "// AR-002-other: Other"]
        .into_iter()
        .enumerate()
    {
        let refusal = alone(
            &format!("admit_nested_spelling_{index}"),
            &format!("## AR-001-thing: Thing\n\n{line}\n"),
        )
        .expect_err("the scan reads this line as declaring");
        assert!(
            refusal.contains("nests declaration `AR-002-other`"),
            "{line:?}: {refusal}"
        );
    }
}

#[test]
fn nested_near_misses_are_refused() {
    for (index, (heading, token)) in [
        ("### AR-002-Other: Other", "AR-002-Other"),
        ("### AR-002: Other", "AR-002"),
    ]
    .into_iter()
    .enumerate()
    {
        let refusal = alone(
            &format!("admit_nested_near_miss_{index}"),
            &format!("## AR-001-thing: Thing\n\n{heading}\n"),
        )
        .expect_err("the scan records a near miss as declaring");
        assert!(
            refusal.contains(&format!("nests declaration `{token}`")),
            "{heading}: {refusal}"
        );
    }
}

#[test]
fn a_sibling_after_a_tab_is_a_second_declaration() {
    let refusal = alone(
        "admit_sibling_after_tab",
        "## AR-001-thing: Thing\n\n##\tAR-002-other: Other\n",
    )
    .expect_err("a sibling is a second declaration");
    assert_eq!(
        refusal,
        "fetch output must contain exactly one declaration and no leading content"
    );
}

#[test]
fn the_wrong_id_depth_or_nothing_at_all_is_refused() {
    for (index, (output, refusal)) in [
        (
            "",
            "fetch output must contain exactly one Markdown declaration",
        ),
        (
            "prose\n",
            "fetch output must begin with one Markdown declaration",
        ),
        (
            "\n## AR-001-thing: Thing\n",
            "fetch output must contain exactly one declaration and no leading content",
        ),
        (
            "### AR-001-thing: Thing\n",
            "fetch output heading depth 3 is invalid for a depth-2 declaration",
        ),
        (
            "## AR-002-other: Other\n",
            "fetch output declares `AR-002-other` instead of requested `AR-001-thing`",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(
            alone(&format!("admit_clause_one_{index}"), output)
                .as_deref()
                .map_err(String::as_str),
            Err(refusal),
            "{output:?}"
        );
    }
}

#[test]
fn numbered_and_fenced_subsections_are_admitted() {
    for (index, body) in [
        "### 1. Notes\n\nprose\n",
        "```\n### AR-002-other: Other\n```\n",
    ]
    .into_iter()
    .enumerate()
    {
        let output = format!("## AR-001-thing: Thing\n\n{body}");
        assert_eq!(
            alone(&format!("admit_subsection_{index}"), &output),
            Ok(output.clone()),
            "{body:?}"
        );
    }
}

// §FS-fetch.admission.2: every other declaration reads as it did.

#[test]
fn replacing_a_range_that_holds_another_declaration_is_refused() {
    let refusal = into_file(
        "admit_keeps_others",
        "## AR-001-thing: Thing\n\n### AR-009-inner: Inner\n\nprose\n",
        "## AR-001-thing: Thing\n\nnew\n",
    )
    .expect_err("the replacement would delete AR-009-inner");
    assert!(refusal.contains("would delete it"), "{refusal}");
    assert!(refusal.contains("AR-009-inner"), "{refusal}");
}

// §FS-fetch.admission.3: no finding the home did not already hold.

#[test]
fn a_heading_the_scan_reads_as_unmarked_is_refused_at_its_output_line() {
    let refusal = alone(
        "admit_unmarked_line",
        "## AR-001-thing: Thing\n\nprose\n\n### Other  \n",
    )
    .expect_err("an unmarked heading is a new finding");
    assert_eq!(
        refusal,
        "fetch output line 5: heading `### Other` is not a section of `AR-001-thing`; \
         number it, fence it, or make it a bold label"
    );
}

#[test]
fn the_refusal_agrees_with_the_next_scan() {
    for (index, (named_sections, body, refused)) in [
        (true, "### 1. Notes", false),
        (true, "### notes: Notes", false),
        (false, "### 1. Notes", false),
        (false, "### notes: Notes", true),
        (true, "### .notes: Notes", false),
        (false, "### .notes: Notes", true),
        (true, "### .1. Notes", false),
        (false, "### .1. Notes", false),
        (true, "### .1: Notes", true),
        (true, "### Other", true),
        (true, "### XX-002-other: Other", true),
        (true, "### <§>AR-002-other: Other", true),
        (true, "###", true),
        (true, "### ", true),
        (true, "    ### Other", false),
        (true, "```\n### Other\n```", false),
        (true, "<pre>\n### Other\n</pre>", false),
        (true, "**Other**", false),
    ]
    .into_iter()
    .enumerate()
    {
        let name = format!("admit_agreement_{index}");
        let output = format!("## AR-001-thing: Thing\n\n{body}\n\nprose\n");
        let verdict = admit_into(
            &name,
            &file_toml(named_sections),
            At::File("docs/arch.md"),
            &[("docs/arch.md", "")],
            "AR-001-thing",
            &output,
        );
        assert_eq!(
            verdict.is_err(),
            refused,
            "{body:?} with named_sections = {named_sections}: {verdict:?}"
        );
        // The next scan of the bytes as the home finds the heading exactly when fetch refused.
        let root = test_root(&name);
        write(&root.join("grund.toml"), &file_toml(named_sections));
        write(&root.join("docs/arch.md"), &output);
        let config = load_config(&root).expect("load config");
        assert_eq!(
            scan_findings(&config, &root).unmarked_headings.len(),
            usize::from(refused),
            "the next scan of {body:?} with named_sections = {named_sections}"
        );
    }
}

#[test]
fn a_finding_the_home_already_held_is_no_reason_to_refuse_however_far_it_moves() {
    let home = "## AR-001-thing: Thing\n\nold\n\n## AR-002-other: Other\n\n### Stray\n";
    let output = "## AR-001-thing: Thing\n\nnew\nlonger\nbody\n";
    assert_eq!(
        into_file("admit_held_finding", home, output),
        Ok(format!("{output}\n## AR-002-other: Other\n\n### Stray\n"))
    );
}

// §FS-fetch.admission.4: no block left open.

#[test]
fn a_fence_the_output_does_not_close_is_refused() {
    assert_eq!(
        alone("admit_open_fence", "## AR-001-thing: Thing\n\n```\ncode\n"),
        Err(
            "fetch output line 3 opens a fenced block it does not close; close it, and fetch \
             again"
                .to_string()
        )
    );
}

// §FS-fetch.admission.5 and §FS-fetch.4: the splice, and a second fetch that changes nothing.

#[test]
fn the_output_replaces_the_range_the_scan_records_at_every_site() {
    let home = "# Arch\n\n## AR-001-thing: One\n\nold\n\n## AR-002-thing: Two\n\nold\n\n\
                ## AR-003-thing: Three\n\nold\n";
    for (index, local) in ["AR-001-thing", "AR-002-thing", "AR-003-thing"]
        .into_iter()
        .enumerate()
    {
        let output = format!("## {local}: New\n\nnew\n");
        let installed = admit_into(
            &format!("admit_site_{index}"),
            &file_toml(true),
            At::File("docs/arch.md"),
            &[("docs/arch.md", home)],
            local,
            &output,
        )
        .expect("admitted");
        let replaced = home.replacen(
            &format!("## {local}: {}\n\nold\n", ["One", "Two", "Three"][index]),
            &output,
            1,
        );
        assert_eq!(installed, replaced, "{local}");
    }
}

#[test]
fn a_new_declaration_goes_in_id_order_parted_by_one_blank_line() {
    let home = "## AR-001-thing: One\n\none\n\n## AR-003-thing: Three\n\nthree\n";
    let installed = admit_into(
        "admit_insert_order",
        &file_toml(true),
        At::File("docs/arch.md"),
        &[("docs/arch.md", home)],
        "AR-002-thing",
        "## AR-002-thing: Two\n\ntwo\n",
    )
    .expect("admitted");
    assert_eq!(
        installed,
        "## AR-001-thing: One\n\none\n\n## AR-002-thing: Two\n\ntwo\n\n\
         ## AR-003-thing: Three\n\nthree\n"
    );
}

#[test]
fn a_crlf_home_keeps_its_line_endings_around_an_insertion() {
    let home = "## AR-001-thing: One\r\n\r\none\r\n";
    let installed = admit_into(
        "admit_crlf",
        &file_toml(true),
        At::File("docs/arch.md"),
        &[("docs/arch.md", home)],
        "AR-002-thing",
        "## AR-002-thing: Two\r\n\r\ntwo\r\n",
    )
    .expect("admitted");
    assert_eq!(
        installed,
        "## AR-001-thing: One\r\n\r\none\r\n\r\n## AR-002-thing: Two\r\n\r\ntwo\r\n"
    );
}

#[test]
fn a_second_fetch_of_the_same_output_changes_nothing() {
    let home = "## AR-001-thing: Thing\n\nold\n\n## AR-002-other: Other\n\nprose\n";
    let output = "## AR-001-thing: Thing\n\nnew\n\n";
    let first = into_file("admit_settled_first", home, output).expect("admitted");
    let second = into_file("admit_settled_second", &first, output).expect("admitted again");
    assert_eq!(first, second);
}

#[test]
fn a_folder_home_takes_the_output_as_the_whole_file() {
    let output = "# AR-001-thing: Thing\n\nbody\n";
    for (index, existing) in [None, Some(""), Some(" \n"), Some("# AR-001-thing: Old\n")]
        .into_iter()
        .enumerate()
    {
        let files: Vec<(&str, &str)> = existing
            .map(|text| ("docs/ar/AR-001-thing.md", text))
            .into_iter()
            .chain([("docs/ar/AR-002-other.md", "# AR-002-other: Other\n")])
            .collect();
        let installed = admit_into(
            &format!("admit_folder_{index}"),
            FOLDER_TOML,
            At::Folder("docs/ar"),
            &files,
            "AR-001-thing",
            output,
        );
        assert_eq!(installed.as_deref(), Ok(output), "{existing:?}");
    }
}

#[test]
fn a_folder_file_the_scan_reads_no_id_in_is_refused() {
    let refusal = admit_into(
        "admit_folder_no_id",
        FOLDER_TOML,
        At::Folder("docs/ar"),
        &[("docs/ar/AR-001-thing.md", "prose\n")],
        "AR-001-thing",
        "# AR-001-thing: Thing\n",
    )
    .expect_err("an `<ID>.md` with text and no ID is not fetch's to fill");
    assert_eq!(
        refusal,
        "snapshot file docs/ar/AR-001-thing.md already exists and the scan reads no ID in it; \
         move or rename it, and fetch again"
    );
}
