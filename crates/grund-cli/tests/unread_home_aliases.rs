//! Folder coverage joins eligible spellings to actual reads by either pass, without changing
//! entry gates or the file's reading (§FS-scan.checks.unread-scan-root.folder,
//! §FS-scan.identity.read-once, §FS-scan.identity.reading.entry).
#![cfg(unix)]

#[path = "support/two_names.rs"]
#[allow(dead_code)]
mod two_names;

use two_names::{Run, Tree};

const TEXT: &str = "grund_config_version = 2\n[schema.sources]\nlanguages = [\"markdown\"]\n\
    [schema.kinds.FS]\nfolders = [\"aa\", \"bb\"]\nindex = false\n";
const VALUE: &str = "grund_config_version = 2\nproject_name = \"v\"\n\
    [schema.sources]\nlanguages = [\"markdown\", \"rust\"]\n\
    [schema.sources.extensions]\ntxt = \"rust\"\n\
    [schema.kinds.CONST]\nfolders = [\"values\"]\nform = \"value\"\nindex = false\n";
const MOVED: &str = "values/a.json:1: warning: read as rust, as its target's own name reads, \
    not as json (since 0.22.0); rename the target or copy the file\n";
const LIMIT: &str = "{\"CONST-room-limit\": [40, \"rooms\"]}\n";

fn exact(run: Run, code: i32, stdout: &str) {
    assert_eq!(run.code, code, "{}{}", run.stdout, run.stderr);
    assert_eq!(run.stdout, stdout);
    assert_eq!(run.stderr, "");
}

/// The single alias need not appear in content-report names, and multiple value names are
/// published after the text pass. Neither timing may leave its home falsely unread.
#[test]
fn value_aliases_share_the_actual_rust_read_without_becoming_json() {
    for count in [1, 2] {
        let tree = Tree::new(&format!("1206-value-{count}"), VALUE);
        tree.put("data/x.txt", LIMIT);
        for name in ["a", "b"].into_iter().take(count) {
            tree.link(&format!("values/{name}.json"), "../data/x.txt");
        }
        exact(tree.grund(&["check"]), 0, MOVED);
        exact(tree.grund(&["check", "values/a.json"]), 0, MOVED);
        exact(tree.grund(&["list"]), 0, "");

        tree.put("data/x.txt", "//! Cites \u{a7}CONST-gone.\n");
        exact(
            tree.grund(&["check"]),
            1,
            &format!("data/x.txt:1: error: unknown reference CONST-gone\n{MOVED}"),
        );
        exact(tree.grund(&["list"]), 0, "");
    }
}

#[test]
fn a_read_text_identity_satisfies_both_admitted_homes() {
    let tree = Tree::new("1206-text", TEXT);
    tree.put("aa/a.md", "# FS-a: A\n\n\u{a7}FS-a.\n");
    tree.put("bb/nothing.txt", "not admitted\n");
    tree.link("bb/link.md", "../aa/a.md");
    exact(tree.grund(&["check"]), 0, "success\n");
    exact(tree.grund(&["list"]), 0, "FS-a  aa/a.md:1  A\n");

    tree.put("aa/a.md", "# FS-a: A\n\n\u{a7}FS-gone and \u{a7}FS-a.\n");
    exact(
        tree.grund(&["check"]),
        1,
        "aa/a.md:3: error: unknown reference FS-gone (also at bb/link.md:3)\n",
    );
}

#[test]
fn a_parsed_value_identity_satisfies_both_homes_and_keeps_its_report_names() {
    let config = TEXT
        .replace("FS", "CONST")
        .replace("index = false", "form = \"value\"\nindex = false");
    let tree = Tree::new("1206-two-value-homes", &config);
    tree.put("data/x.json", LIMIT);
    tree.put("bb/nothing.dat", "unread\n");
    tree.link("aa/a.json", "../data/x.json");
    tree.link("bb/b.json", "../data/x.json");
    exact(
        tree.grund(&["check"]),
        0,
        "aa/a.json:1: warning: declared but never cited: CONST-room-limit; cite it (also at bb/b.json:1)\n",
    );
    exact(tree.grund(&["list"]), 0, "CONST-room-limit  aa/a.json:1\n");
}

/// A readable target does not admit an unlisted, hidden, excluded or ignored text spelling.
#[test]
fn refused_or_filtered_text_names_do_not_cover_their_home() {
    for gate in ["unadmitted", "hidden", "exclude", "ignore"] {
        let config = if gate == "exclude" {
            TEXT.replace(
                "[schema.sources]\n",
                "[schema.sources]\nexclude = [\"bb/link.md\"]\n",
            )
        } else {
            TEXT.to_string()
        };
        let tree = Tree::new(&format!("1206-text-{gate}"), &config);
        tree.put("aa/a.md", "# FS-a: A\n\n\u{a7}FS-a.\n");
        tree.put("bb/nothing.txt", "not admitted\n");
        let link = match gate {
            "unadmitted" => "bb/link.txt",
            "hidden" => "bb/.link.md",
            _ => "bb/link.md",
        };
        tree.link(link, "../aa/a.md");
        if gate == "ignore" {
            tree.put("bb/.ignore", "link.md\n");
        }
        let line = if gate == "exclude" { 6 } else { 5 };
        exact(
            tree.grund(&["check"]),
            0,
            &format!(
                "grund.toml:{line}: warning: [schema.kinds.FS] folder `bb` is not read — no language in [schema.sources] languages reads `.txt`, so nothing in it is checked. Give the files in it an extension a listed language reads.\n"
            ),
        );
        exact(tree.grund(&["list"]), 0, "FS-a  aa/a.md:1  A\n");
    }
}

/// Value discovery admits the alias, but its Rust target is excluded and neither pass reads it.
/// The real warning's offered rename still repairs that home without admitting the target path.
#[test]
fn a_value_candidate_is_no_proof_of_a_successful_read() {
    let config = VALUE.replace(
        "[schema.sources]\n",
        "[schema.sources]\nexclude = [\"data\"]\n",
    );
    let tree = Tree::new("1206-value-no-read", &config);
    tree.put("data/x.txt", "//! Cites \u{a7}CONST-gone.\n");
    tree.link("values/a.json", "../data/x.txt");
    exact(
        tree.grund(&["check"]),
        0,
        &format!(
            "grund.toml:9: warning: [schema.kinds.CONST] folder `values` is not read — no language in [schema.sources] languages reads `.json`, so nothing in it is checked. Give the files in it an extension a listed language reads.\n{MOVED}"
        ),
    );
    exact(tree.grund(&["list"]), 0, "");
    std::fs::rename(
        tree.root.join("values/a.json"),
        tree.root.join("values/a.rs"),
    )
    .unwrap();
    exact(
        tree.grund(&["check"]),
        1,
        "values/a.rs:1: error: unknown reference CONST-gone\n",
    );
    exact(tree.grund(&["list"]), 0, "");
}

/// The text walk's ignore gate does not erase value discovery's existing home exception.
#[test]
fn value_home_discovery_keeps_its_ignore_exception() {
    let tree = Tree::new("1206-value-ignore", VALUE);
    tree.put("data/x.txt", LIMIT);
    tree.put("values/.ignore", "a.json\n");
    tree.put("values/nothing.dat", "unread\n");
    tree.link("values/a.json", "../data/x.txt");
    exact(tree.grund(&["check"]), 0, MOVED);
    exact(tree.grund(&["list"]), 0, "");
}

/// A hidden value alias cannot satisfy its home, even though another name is actually read.
#[test]
fn a_hidden_value_name_does_not_cover_its_home() {
    let tree = Tree::new("1206-value-hidden", VALUE);
    tree.put("data/x.txt", LIMIT);
    tree.put("values/nothing.dat", "unread\n");
    tree.link("values/.a.json", "../data/x.txt");
    let check = tree.grund(&["check"]);
    assert_eq!(check.code, 0, "{}{}", check.stdout, check.stderr);
    assert_eq!(check.stderr, "");
    assert!(
        check.stdout.contains("folder `values` is not read"),
        "{}",
        check.stdout
    );
    exact(tree.grund(&["list"]), 0, "");
}
