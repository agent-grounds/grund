//! Black-box: a finding about what a file holds is one line naming each spelling once, a line
//! names only what the plain run reaches, and a finding judged from the file's directory names
//! the spelling it was judged at (§FS-scan.identity.findings.content,
//! §FS-scan.identity.findings.judged, §FS-scan.identity.link-path, §FS-check.1.3.2,
//! §FS-check.1.3.6.2, §FS-errors.5.1).
#![cfg(unix)]

#[path = "support/two_names.rs"]
#[allow(dead_code)]
mod two_names;

use two_names::Tree;

const V2: &str = "grund_config_version = 2\nproject_name = \"c\"\n\
    [schema.kinds.FS]\nfolders = [\"docs/fs\"]\nindex = false\n";
const BODY: &str = "# FS-a: A\n\nBody.\n\n\u{a7}FS-gone and \u{a7}FS-a.\n";
const LINE: &str = "docs/fs/a.md:5: error: unknown reference FS-gone (also at notes/link.md:5)\n";

fn tree(case: &str) -> Tree {
    let tree = Tree::new(case, V2);
    tree.put("docs/fs/a.md", BODY);
    tree.link("notes/link.md", "../docs/fs/a.md");
    tree
}

#[test]
fn a_content_finding_is_one_line_naming_each_spelling() {
    let tree = tree("one-line");
    let plain = tree.grund(&["check"]);
    assert_eq!(plain.stdout, LINE);
    assert_eq!(plain.code, 1);
    assert_eq!(tree.grund(&["check", "--full"]).stdout, LINE);
}

/// The JSON row of §FS-scan.identity.findings.content, byte for byte: `sites` keeps its shape.
#[test]
fn its_json_carries_both_in_sites() {
    let json = tree("json").grund(&["check", "--format", "json"]);
    assert_eq!(
        json.stdout,
        "{\"severity\":\"error\",\"path\":\"docs/fs/a.md\",\"line\":5,\"code\":\"dangling\",\
         \"message\":\"unknown reference FS-gone (also at notes/link.md:5)\",\
         \"sites\":[{\"path\":\"docs/fs/a.md\",\"line\":5},{\"path\":\"notes/link.md\",\"line\":5}],\
         \"authority\":null}\n"
    );
}

/// A finding naming several sites is in scope at any and keeps its bytes, so a run naming the
/// link prints the whole run's line (§FS-scan.identity.link-path, §FS-check.1.3.6.2).
#[test]
fn a_run_naming_either_spelling_prints_the_whole_runs_line() {
    let tree = tree("scoped");
    for path in ["notes/link.md", "docs/fs/a.md"] {
        let check = tree.grund(&["check", path]);
        assert_eq!(check.stdout, LINE, "{path}");
        assert_eq!(check.code, 1, "{path}");
    }
}

fn v1_tree(case: &str, include: &str) -> Tree {
    let tree = Tree::new(
        case,
        &format!(
            "project_name = \"c\"\n[scan]\ninclude = [{include}]\n\n\
             [[kinds]]\nkind = \"FS\"\nfolder = \"spec\"\nindex = false\n"
        ),
    );
    tree.put("spec/a.md", "# FS-001-a: A\n\nCites \u{a7}FS-001-a.\n");
    tree.put("docs/x.md", "One.\n\nTwo.\n\n\u{a7}FS-001-gone here.\n");
    tree.link("notes/link.md", "../docs/x.md");
    tree
}

/// Where the plain run reaches both names, plain and `--full` print the same line, naming both.
#[test]
fn plain_and_full_agree_on_a_line_naming_both_reached_spellings() {
    let tree = v1_tree("reached-both", "\"docs\", \"notes\"");
    let want = "docs/x.md:5: error: unknown reference FS-001-gone (also at notes/link.md:5)";
    assert!(
        tree.grund(&["check"])
            .stdout
            .lines()
            .any(|line| line == want)
    );
    assert!(
        tree.grund(&["check", "--full"])
            .stdout
            .lines()
            .any(|line| line == want)
    );
}

/// Guard: a line names only the spellings the plain run reaches. With one root the plain run
/// never meets the other name, so `--full` must print the plain bytes (§FS-check.1.3.2).
#[test]
fn a_line_names_only_the_spellings_the_plain_run_reaches() {
    for (include, want) in [
        (
            "\"notes\"",
            "notes/link.md:5: error: unknown reference FS-001-gone",
        ),
        (
            "\"docs\"",
            "docs/x.md:5: error: unknown reference FS-001-gone",
        ),
    ] {
        let tree = v1_tree(&format!("reached-{}", include.trim_matches('"')), include);
        for args in [vec!["check"], vec!["check", "--full"]] {
            let check = tree.grund(&args);
            let lines: Vec<&str> = check.stdout.lines().collect();
            assert!(
                lines.contains(&want),
                "{include} {args:?}: {}",
                check.stdout
            );
            assert!(
                !check.stdout.contains("also at"),
                "{include} {args:?}: {}",
                check.stdout
            );
        }
    }
}

/// Guard: a stub's link resolves from the file's directory, so it can hold at one spelling and
/// fail at the other. It is judged at the name listed first and names only that one, whichever
/// way that goes (§FS-scan.identity.findings.judged, §FS-fmt.2.3.2.2).
#[test]
fn a_stub_finding_names_only_the_spelling_it_was_judged_at() {
    let tree = Tree::new("stub-holds", V2);
    tree.put("src/s.rs", "/// FS-s: S\nfn s() {}\n");
    tree.put(
        "docs/fs/s.md",
        "# FS-s: [s](../../src/s.rs)\n\nCites \u{a7}FS-s.\n",
    );
    tree.link("a/s.md", "../docs/fs/s.md");
    assert_eq!(
        tree.grund(&["check"]).stdout,
        "success\n",
        "it holds where it was judged"
    );

    let broken = Tree::new("stub-broken", V2);
    broken.put(
        "docs/fs/s.md",
        "# FS-s: [s](../../src/gone.rs)\n\nCites \u{a7}FS-s.\n",
    );
    broken.link("a/s.md", "../docs/fs/s.md");
    let check = broken.grund(&["check"]);
    let stub = check
        .stdout
        .lines()
        .find(|line| line.contains("broken") || line.contains("stub"));
    let stub = stub.unwrap_or_else(|| panic!("a stub finding in {}", check.stdout));
    assert!(stub.starts_with("docs/fs/s.md:1:"), "{stub}");
    assert!(!check.stdout.contains("a/s.md"), "{}", check.stdout);
    assert!(!check.stdout.contains("also at"), "{}", check.stdout);
}
