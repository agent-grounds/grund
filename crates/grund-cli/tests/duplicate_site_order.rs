//! Black-box: one duplicate's sites have one order whichever command lists them
//! (§FS-errors.4.1.1, §REQ-one-answer.identity.declarations). Each shape compares `check`'s
//! `sites` with `show`'s and `refs`', byte for byte, and with the order itself, because two
//! commands wrong alike agree.
#![cfg(unix)]

#[path = "support/two_names.rs"]
#[allow(dead_code)]
mod two_names;

use std::process::Command;
use two_names::Tree;

const SPEC: &str = "grund_config_version = 2\nproject_name = \"demo\"\n\
    [schema]\nid_format = \"{kind}-{slug}\"\n\
    [schema.kinds.SPEC]\nfolders = [\"docs\"]\nindex = false\n";
const CASES: &str = "grund_config_version = 1\n\n[reference]\nstrict = false\n\n\
    [id]\nformat = \"{kind}-{slug}\"\n\n\
    [[kinds]]\nkind = \"E2E\"\nfolder = \"e2e/cases\"\ntitle = \"End-to-end test\"\n";
const STUB: &str = "grund_config_version = 1\n\n[reference]\nstrict = true\n\n\
    [id]\nformat = \"{kind}-{slug}\"\n\n\
    [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n\n\
    [scan]\ninclude = [\"docs\"]\n";
const DECL: &str = "# SPEC-login: Login\n\nBody.\n";

/// The raw `sites` array of the one record of `code` in `text`, its bytes as printed.
fn raw_sites(text: &str, code: &str) -> String {
    let needle = format!("\"code\":\"{code}\"");
    let line = text
        .lines()
        .find(|line| line.contains(&needle))
        .unwrap_or_else(|| panic!("no {code} record in: {text}"));
    let start = line.find("\"sites\":").expect("a sites field");
    let end = line.find(",\"authority\"").expect("an authority field");
    line[start + "\"sites\":".len()..end].to_string()
}

/// `check`'s, `show`'s and `refs`' JSON `sites` for `id`, each run from `cwd` under `tree`.
fn three_sites(tree: &Tree, cwd: &str, id: &str, extra: &[&str]) -> [String; 3] {
    let run = |verb: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(verb)
            .args(["--format=json"])
            .args(extra)
            .current_dir(tree.root.join(cwd))
            .env("NO_COLOR", "1")
            .output()
            .expect("run grund");
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    };
    [
        raw_sites(&run(&["check"]), "duplicate"),
        raw_sites(&run(&["show", id]), "ambiguous"),
        raw_sites(&run(&["refs", id]), "ambiguous"),
    ]
}

/// All three commands list exactly `want`.
fn assert_one_order(sites: [String; 3], want: &str) {
    let [check, show, refs] = sites;
    assert_eq!(check, want, "check's order moved");
    assert_eq!(
        show, check,
        "show lists the sites in another order than check"
    );
    assert_eq!(
        refs, check,
        "refs lists the sites in another order than check"
    );
}

#[test]
fn line_nine_comes_before_line_ten() {
    let tree = Tree::new("site-order-lines", SPEC);
    tree.put(
        "docs/one.md",
        "# SPEC-other: O\n\nBody.\n\nFive.\n\nSeven.\n\n# SPEC-login: First\n# SPEC-login: Second\n\nBody.\n",
    );
    assert_one_order(
        three_sites(&tree, ".", "SPEC-login", &[]),
        r#"[{"path":"docs/one.md","line":9},{"path":"docs/one.md","line":10}]"#,
    );
}

#[test]
fn a_path_that_prefixes_another_comes_first() {
    let tree = Tree::new("site-order-prefix", CASES);
    tree.put("e2e/cases/login/command.args", "check repo\n");
    tree.put("e2e/cases/login/expected.exit", "0\n");
    tree.put("e2e/cases/login.md", "# E2E-login: Login\n\nBody.\n");
    assert_one_order(
        three_sites(&tree, ".", "E2E-login", &[]),
        r#"[{"path":"e2e/cases/login","line":1},{"path":"e2e/cases/login.md","line":1}]"#,
    );
}

#[test]
fn a_run_from_a_subdirectory_keeps_the_order_of_the_project() {
    let tree = Tree::new("site-order-subdir", SPEC);
    tree.put("docs/sub/a.md", DECL);
    tree.put("docs/z.md", DECL);
    assert_one_order(
        three_sites(&tree, "docs/sub", "SPEC-login", &["--path-base=invocation"]),
        r#"[{"path":"a.md","line":1},{"path":"../z.md","line":1}]"#,
    );
}

/// §FS-declarations.stubs.pairing.3: a home reached through a stub is named at its target, and
/// its sites sort as `check` sorts them.
#[test]
fn a_stubs_target_declaring_the_id_twice_is_in_line_order() {
    let tree = Tree::new("site-order-stub", STUB);
    tree.put(
        "docs/second.md",
        "# FS-second: [../source.rs](../source.rs)\n",
    );
    tree.put(
        "source.rs",
        "/// FS-other: Other\npub fn other() {}\n\n\n\n/// FS-second: Second\npub fn second() {}\n\n\n\n/// FS-second: Again\npub fn again() {}\n",
    );
    assert_one_order(
        three_sites(&tree, ".", "FS-second", &[]),
        r#"[{"path":"source.rs","line":6},{"path":"source.rs","line":11}]"#,
    );
}

/// Regression pin, green before the fix: a catalog belongs to one member, so its sites are in
/// one frame from the member's subdirectory as from anywhere.
#[test]
fn a_member_run_from_a_subdirectory_lists_one_order() {
    let root = "grund_config_version = 2\nproject_name = \"root\"\n\
        [workspace]\nmembers = [\"hw\"]\n";
    let tree = Tree::new("site-order-member", root);
    tree.put("hw/grund.toml", SPEC);
    tree.put("hw/docs/sub/a.md", DECL);
    tree.put("hw/docs/z.md", DECL);
    let sites = three_sites(&tree, "hw/docs/sub", "SPEC-login", &[]);
    assert_one_order(
        sites,
        r#"[{"path":"docs/sub/a.md","line":1},{"path":"docs/z.md","line":1}]"#,
    );
}

/// Regression pin, green before the fix: `check` settles a site on its first name and then sorts,
/// so a run handed the second name keeps the finding where the whole run puts it
/// (§FS-scan.identity.findings.content). Pinned against literal output and not against `show`,
/// whose first names are #1207's.
#[test]
fn a_file_with_two_names_keeps_checks_located_site() {
    let config = "grund_config_version = 2\nproject_name = \"n\"\n\
        [schema.kinds.FS]\nfolders = [\"docs/fs\"]\nindex = false\n";
    let decl = "# FS-d: A\n\n\u{a7}FS-d\n";
    let tree = Tree::new("site-order-two-names", config);
    tree.put("docs/fs/a.md", decl);
    tree.put("docs/fs/b.md", decl);
    tree.link("notes/link.md", "../docs/fs/a.md");
    let want = "docs/fs/a.md:1: error: duplicate declaration of FS-d (also declared at \
                docs/fs/b.md:1) (also at notes/link.md:1)\n";
    assert_eq!(tree.grund(&["check"]).stdout, want);
    assert_eq!(tree.grund(&["check", "notes/link.md"]).stdout, want);
    let sites = tree
        .grund(&["check", "notes/link.md", "--format=json"])
        .sites("duplicate");
    assert_eq!(
        sites.join(" "),
        "docs/fs/a.md:1 notes/link.md:1 docs/fs/b.md:1"
    );
}
