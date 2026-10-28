//! §FS-examples.2 (issue #1078): the downstream-citers recipe is taught by a
//! guide section and a multi-run example (§FS-examples.5.5). The behaviour it
//! teaches already ships (§FS-cover.2.1, §FS-check.4.1.4), so what these tests
//! pin is the teaching: the example's three runs, the guide section with the
//! regeneration step and its four limits, and the links that lead to them.
use std::fs;
use std::path::PathBuf;

const SECTION: &str = "### Citers in a repository you cannot read";
const ANCHOR: &str = "repository-options.md#citers-in-a-repository-you-cannot-read";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(relative: &str) -> String {
    fs::read_to_string(root().join(relative)).unwrap_or_else(|_| panic!("missing {relative}"))
}

/// The body under `heading` up to the next heading of the same or a higher level.
fn section<'a>(text: &'a str, heading: &str) -> &'a str {
    let start = text
        .find(heading)
        .unwrap_or_else(|| panic!("no heading {heading:?}"));
    let level = heading.chars().take_while(|c| *c == '#').count();
    let body = &text[start + heading.len()..];
    let end = body
        .match_indices("\n#")
        .find(|(at, _)| {
            let hashes = body[at + 1..].chars().take_while(|c| *c == '#').count();
            hashes <= level && body[at + 1 + hashes..].starts_with(' ')
        })
        .map_or(body.len(), |(at, _)| at);
    &body[..end]
}

fn collapsed(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
#[ignore = "red until #1078 adds the guide section and examples/downstream-citers/; implement removes this"]
fn downstream_citers_example_pins_its_three_runs() {
    let example = root().join("examples/downstream-citers");
    assert!(
        example.join("README.md").is_file(),
        "missing example README"
    );
    assert!(
        !example.join("expected.exit").exists(),
        "a multi-run example has no top-level run"
    );
    for (run, command, exit) in [
        ("1-cover", "cover", "0"),
        ("2-check", "check", "0"),
        ("3-deleted", "check", "1"),
    ] {
        let dir = example.join(run);
        assert!(dir.join("repo").is_dir(), "{run}: missing repo/");
        let args = fs::read_to_string(dir.join("command.args")).unwrap_or_default();
        assert!(
            args.starts_with(command),
            "{run}: command.args runs {command}: {args:?}"
        );
        let expected = fs::read_to_string(dir.join("expected.exit"))
            .unwrap_or_else(|_| panic!("{run}: missing expected.exit"));
        assert_eq!(expected.trim(), exit, "{run}: exit");
    }
    let cover = read("examples/downstream-citers/1-cover/command.args");
    assert!(
        cover.contains("--format json"),
        "1-cover pins cover --format json"
    );
    let stdout = read("examples/downstream-citers/1-cover/expected.stdout");
    assert!(
        stdout.contains("\"id\":\"substratevm/"),
        "1-cover keeps the unresolved alias"
    );
    let deleted = read("examples/downstream-citers/3-deleted/expected.stdout")
        + &read("examples/downstream-citers/3-deleted/expected.stderr");
    assert!(
        deleted.contains("unknown reference"),
        "3-deleted is refused at the page"
    );
    let readme = read("examples/downstream-citers/README.md");
    assert!(
        readme.contains("jq"),
        "the README shows the filter the runner cannot run"
    );
}

#[test]
#[ignore = "red until #1078 adds the guide section and examples/downstream-citers/; implement removes this"]
fn repository_options_teaches_the_recipe_and_its_limits() {
    let guide = read("docs/user-facing/repository-options.md");
    let workspaces = section(&guide, "## Workspaces and sub-projects");
    assert!(
        workspaces.contains(SECTION),
        "the recipe sits under Workspaces and sub-projects"
    );
    let recipe = collapsed(section(&guide, SECTION));
    for phrase in [
        "grund cover --format json",
        "jq",
        "grund fmt --write",
        "unknown reference",
        // The four limits: presence only, staleness, the closed default, the alias.
        "only records that",
        "stale",
        "regenerate",
        "must-not",
        "unknown project alias",
    ] {
        assert!(recipe.contains(phrase), "the recipe is missing {phrase:?}");
    }
}

#[test]
#[ignore = "red until #1078 adds the guide section and examples/downstream-citers/; implement removes this"]
fn unused_teaching_links_to_the_recipe() {
    let querying = read("docs/user-facing/querying.md");
    let unused = section(&querying, "## What nothing cites yet: `list --unused`");
    assert!(
        unused.contains(ANCHOR),
        "querying.md's list --unused section links the recipe"
    );
    assert!(
        read("docs/user-facing/checking.md").contains(ANCHOR),
        "checking.md links the recipe"
    );
}

#[test]
#[ignore = "red until #1078 adds the guide section and examples/downstream-citers/; implement removes this"]
fn indexes_list_the_example_and_the_guide_section() {
    assert!(
        read("examples/README.md").contains("(downstream-citers/)"),
        "examples index row"
    );
    let guides = read("docs/user-facing/README.md");
    assert!(
        guides.contains("examples/downstream-citers/"),
        "guide index names the example"
    );
    assert!(guides.contains(ANCHOR), "guide index names the section");
}
