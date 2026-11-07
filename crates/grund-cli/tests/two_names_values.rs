//! Black-box: a value spelling whose reading moved is a located finding, not a declaration gone
//! from a green tree (§FS-scan.checks.reading-moved, §REQ-backwards-compatibility.5). The entry
//! stays on the spelling: `.json` is what let the value home take the link at all
//! (§FS-scan.identity.reading.entry).
#![cfg(unix)]

#[path = "support/two_names.rs"]
#[allow(dead_code)]
mod two_names;

use two_names::Tree;

const CONFIG: &str = "grund_config_version = 2\nproject_name = \"v\"\n\
    [schema.sources]\nlanguages = [\"markdown\", \"rust\"]\n\
    [schema.sources.extensions]\ntxt = \"rust\"\n\
    [schema.kinds.CONST]\nfolders = [\"values\"]\nform = \"value\"\nindex = false\n";
const LIMIT: &str = "{\"CONST-room-limit\": [40, \"rooms\"]}\n";
const AS_RUST: &str = "values/a.json:1: warning: read as rust, as its target's own name reads, \
    not as json (since 0.22.0); rename the target or copy the file\n";

/// A value home took the link by its own `.json`, and the file's own name reads it as Rust: the
/// declaration is gone, which the warning says where it happened, on either scope.
#[test]
fn a_link_the_rule_reads_as_no_value_source_earns_the_warning() {
    let tree = Tree::new("alone", CONFIG);
    tree.put("data/x.txt", LIMIT);
    tree.link("values/a.json", "../data/x.txt");
    for args in [&["check"][..], &["check", "values/a.json"][..]] {
        let check = tree.grund(args);
        assert_eq!(check.stdout, AS_RUST, "{args:?}: {}", check.stderr);
        assert_eq!(check.code, 0, "{args:?}");
    }
    assert_eq!(tree.grund(&["list"]).stdout, "");
}

/// The target stays where its own name put it: it is a Rust file, read by the text scan.
#[test]
fn its_target_stays_read_by_its_own_name() {
    let tree = Tree::new("target", CONFIG);
    tree.put("data/x.txt", "//! Cites \u{a7}CONST-gone.\n");
    tree.link("values/a.json", "../data/x.txt");
    let check = tree.grund(&["check"]);
    assert!(
        check.stdout.contains("data/x.txt:1:") && check.stdout.contains("CONST-gone"),
        "{}",
        check.stdout
    );
    assert!(check.stdout.contains(AS_RUST), "{}", check.stdout);
}

/// Two names in the home are one file, and the warning stands at the name listed first.
#[test]
fn two_names_earn_one_warning_at_the_first() {
    let tree = Tree::new("two", CONFIG);
    tree.put("data/x.txt", LIMIT);
    tree.link("values/a.json", "../data/x.txt");
    tree.link("values/b.json", "../data/x.txt");
    let check = tree.grund(&["check"]);
    assert_eq!(check.stdout, AS_RUST, "{}", check.stderr);
    assert_eq!(check.code, 0);
    assert_eq!(tree.grund(&["list"]).stdout, "");
}

/// A citation of what dropped out still resolves to nothing, and the warning says why.
#[test]
fn a_citation_of_the_lost_declaration_sits_beside_the_warning() {
    let tree = Tree::new("cited", CONFIG);
    tree.put("data/x.txt", LIMIT);
    tree.link("values/a.json", "../data/x.txt");
    tree.put("docs/offer.md", "# Offer\n\n\u{a7}CONST-room-limit\n");
    let check = tree.grund(&["check"]);
    assert_eq!(check.code, 1);
    assert!(check.stdout.contains(AS_RUST), "{}", check.stdout);
    assert!(
        check
            .stdout
            .contains("docs/offer.md:3: error: unknown reference CONST-room-limit"),
        "{}",
        check.stdout
    );
}

/// A single `.json` link to a YAML file is read as YAML, its declaration kept, and its reading
/// moved all the same: it was invalid JSON before.
#[test]
fn a_link_read_as_another_value_format_earns_the_warning_and_keeps_the_declaration() {
    let tree = Tree::new("yaml", CONFIG);
    tree.put("data/limits.yaml", "CONST-room-limit: [40, rooms]\n");
    tree.link("values/a.json", "../data/limits.yaml");
    let check = tree.grund(&["check"]);
    assert!(
        check.stdout.contains(
            "values/a.json:1: warning: read as yaml, as its target's own name reads, not as json \
             (since 0.22.0); rename the target or copy the file\n"
        ),
        "{}{}",
        check.stdout,
        check.stderr
    );
    assert_eq!(check.code, 0, "{}{}", check.stdout, check.stderr);
    assert!(tree.grund(&["list"]).stdout.contains("CONST-room-limit"));
}

/// Guard: a value source with no link says nothing of readings.
#[test]
fn a_plain_value_source_earns_nothing() {
    let tree = Tree::new("plain", CONFIG);
    tree.put("values/a.json", LIMIT);
    let check = tree.grund(&["check"]);
    assert!(!check.stdout.contains("read as"), "{}", check.stdout);
    assert!(tree.grund(&["list"]).stdout.contains("CONST-room-limit"));
}
