//! Black-box: a finding about what the file holds names every spelling whether or not it came with
//! sites of its own, and a run naming either spelling keeps it (§FS-scan.identity.findings.content,
//! §FS-check.1.3.6.2, §FS-check.1.3.2). Which code stands on which side is one table, tested over
//! the whole catalog beside it.
#![cfg(unix)]

#[path = "support/two_names.rs"]
#[allow(dead_code)]
mod two_names;

use two_names::Tree;

const CONFIG: &str = "grund_config_version = 1\nproject_name = \"s\"\n\
    [reference]\nstrict = true\n[id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\
    [[kinds]]\nkind = \"BENCH\"\nfolder = \"docs/bench\"\nindex = false\n\
    [[kinds]]\nkind = \"AR\"\nfolder = \"docs/ar\"\nindex = false\n\
    [[kinds]]\nkind = \"CONST\"\nfolder = \"values\"\nindex = false\nvalues = true\n\
    [[kinds]]\nkind = \"RULE\"\nfolder = \"docs/rules\"\nindex = false\nrules = true\n\
    [scan]\ninclude = [\"docs\", \"notes\"]\nextensions = [\"md\"]\n";

fn tree(case: &str) -> Tree {
    let tree = Tree::new(case, CONFIG);
    tree.put(
        "docs/rules/RULE-shape.md",
        "# RULE-shape: Each BENCH must have exactly one goal chapter.\n\nA bench says what it is \
         for, \u{a7}RULE-shape.\n",
    );
    tree.put(
        "docs/rules/RULE-cites.md",
        "# RULE-cites: Each BENCH must cite at least 2 AR.\n\nA bench says what it measures, \
         \u{a7}RULE-cites.\n",
    );
    tree.put(
        "docs/bench/b.md",
        "# BENCH-b: B\n\nThe lead, \u{a7}BENCH-b.\n",
    );
    tree.link("notes/b-link.md", "../docs/bench/b.md");
    tree.put("values/price.md", "# CONST-price: Price\n\n## 1. 1200\n");
    tree.put(
        "docs/ar/v.md",
        "# AR-v: V\n\nCites \u{a7}AR-v.\n\n`1250` (\u{a7}CONST-price.1)\n\n## 1. One\n\nFirst.\n\n\
         ## 1. Again\n\nSecond.\n",
    );
    tree.link("notes/v-link.md", "../docs/ar/v.md");
    tree
}

/// The line of `stdout` holding `code`'s words.
fn line_of<'a>(stdout: &'a str, words: &str) -> &'a str {
    stdout
        .lines()
        .find(|line| line.contains(words))
        .unwrap_or_else(|| panic!("no line with {words:?} in {stdout}"))
}

#[test]
fn a_rule_finding_names_both_spellings() {
    let check = tree("rules").grund(&["check"]);
    let chapter = line_of(&check.stdout, "goal chapter");
    assert!(
        chapter.starts_with("docs/bench/b.md:1: error:"),
        "{chapter}"
    );
    assert!(
        chapter.ends_with("(also at notes/b-link.md:1)"),
        "{chapter}"
    );
    let cites = line_of(&check.stdout, "RULE-cites");
    assert!(cites.starts_with("docs/bench/b.md:1: error:"), "{cites}");
    assert!(cites.ends_with("(also at notes/b-link.md:1)"), "{cites}");
}

/// Sites of its own are other lines of the same text: each gets its other name too, and none
/// comes twice.
#[test]
fn a_finding_with_sites_of_its_own_names_both_spellings_of_each_once() {
    let tree = tree("own-sites");
    let check = tree.grund(&["check"]);
    let section = line_of(&check.stdout, "duplicate section AR-v.1");
    assert!(section.starts_with("docs/ar/v.md:7: error:"), "{section}");
    assert!(
        section.ends_with("(also at notes/v-link.md:7)"),
        "{section}"
    );
    let mismatch = line_of(&check.stdout, "value mismatch");
    assert!(mismatch.starts_with("docs/ar/v.md:5: error:"), "{mismatch}");
    assert!(
        mismatch.ends_with("(also at notes/v-link.md:5)"),
        "{mismatch}"
    );

    let json = tree.grund(&["check", "--format", "json"]);
    assert_eq!(
        json.sites("duplicate-section"),
        [
            "docs/ar/v.md:7",
            "notes/v-link.md:7",
            "docs/ar/v.md:11",
            "notes/v-link.md:11"
        ]
    );
    assert_eq!(
        json.sites("value-mismatch"),
        ["docs/ar/v.md:5", "notes/v-link.md:5", "values/price.md:3"]
    );
}

/// The added sites are what keep such a finding in scope at the name no site held before, and the
/// line does not change with the name handed.
#[test]
fn a_run_naming_either_spelling_prints_the_whole_runs_lines() {
    let tree = tree("scopes");
    let whole = tree.grund(&["check"]);
    for (scope, file) in [
        ("notes/b-link.md", "docs/bench/b.md"),
        ("docs/bench/b.md", "docs/bench/b.md"),
        ("notes/v-link.md", "docs/ar/v.md"),
        ("docs/ar/v.md", "docs/ar/v.md"),
    ] {
        let lines: Vec<&str> = whole
            .stdout
            .lines()
            .filter(|line| line.starts_with(file))
            .collect();
        assert!(!lines.is_empty(), "{file}");
        let narrow = tree.grund(&["check", scope]);
        assert_eq!(narrow.stdout.lines().collect::<Vec<_>>(), lines, "{scope}");
    }
}

/// `--full` reads a superset and adds nothing in scope (§FS-check.1.3.2).
#[test]
fn plain_and_full_agree() {
    let tree = tree("full");
    assert_eq!(
        tree.grund(&["check"]).stdout,
        tree.grund(&["check", "--full"]).stdout
    );
}

/// Guard: a duplicate that already names every spelling gets no clause, and no site comes twice.
#[test]
fn a_finding_that_already_names_each_spelling_is_left_as_it_is() {
    let config = "grund_config_version = 2\nproject_name = \"s\"\n\
        [schema.kinds.CONST]\nfolders = [\"a\", \"z\"]\nform = \"value\"\nindex = false\n";
    let tree = Tree::new("named", config);
    tree.put("a/limits.json", "{\"CONST-limit\": [40, \"rooms\"]}\n");
    tree.link("z/limits.json", "../a/limits.json");
    let check = tree.grund(&["check"]);
    assert!(!check.stdout.contains("duplicate"), "{}", check.stdout);
    assert!(
        !check.stdout.contains("(also at z/limits.json:1) (also"),
        "{}",
        check.stdout
    );
}
