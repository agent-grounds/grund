//! Black-box: a file's names are judged over the names the plain run reaches, when it reaches any.
//! A name only `--full` or a handed path reaches decides nothing, so `--full` stays additive and a
//! run naming a path agrees with the whole run (§FS-scan.identity.reading.plain,
//! §FS-scan.checks.reading-conflict, §FS-scan.checks.reading-moved, §FS-check.1.3.2).
#![cfg(unix)]

#[path = "support/two_names.rs"]
#[allow(dead_code)]
mod two_names;

use two_names::Tree;

/// `docs/` is the plain run's only root, so a name under `other/` is reached by `--full` alone.
const V1: &str = "grund_config_version = 1\n[id]\nformat = \"{kind}-{slug}\"\n\
    [[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\nindex = false\n\
    [scan]\ninclude = [\"docs\"]\nextensions = [\"md\", \"rs\"]\n";
/// The plain run walks the whole root and passes over `build/`, so a handed path alone reaches it.
const V1_EXCLUDED: &str = "grund_config_version = 1\n[id]\nformat = \"{kind}-{slug}\"\n\
    [[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\nindex = false\n\
    [scan]\nexclude = [\"build\"]\nextensions = [\"md\", \"rs\"]\n";

/// `LICENSE` claims no reading, `docs/license.md` is its one name the plain run reaches, and
/// `other/license.rs` would read it another way.
fn license(case: &str, text: &str) -> Tree {
    let tree = Tree::new(case, V1);
    tree.put(
        "docs/fs/a.md",
        "# FS-a: A\n\nCites \u{a7}FS-lic and \u{a7}FS-a.\n",
    );
    tree.put("LICENSE", text);
    tree.link("docs/license.md", "../LICENSE");
    tree.link("other/license.rs", "../LICENSE");
    tree
}

/// The stdout and exit code of `args`, with the version 1 notice on stderr left out.
fn ran(tree: &Tree, args: &[&str]) -> (i32, String) {
    let run = tree.grund(args);
    (run.code, run.stdout)
}

#[test]
fn a_name_only_full_reaches_refuses_nothing() {
    let tree = license("plain-green", "# FS-lic: L\n\n\u{a7}FS-lic\n");
    for args in [
        &["check"][..],
        &["check", "--full"],
        &["check", "other/license.rs"],
        &["check", "other"],
        &["check", "docs/license.md"],
    ] {
        let (code, stdout) = ran(&tree, args);
        assert_eq!(
            (code, stdout.contains("reads two ways")),
            (0, false),
            "{args:?}: {stdout}"
        );
    }
}

/// `--full` adds to the plain run's lines and loses none of them.
#[test]
fn a_finding_in_such_a_file_is_the_plain_runs_line_from_every_scope() {
    let tree = license("plain-red", "# FS-lic: L\n\n\u{a7}FS-lic \u{a7}FS-gone\n");
    let want = "docs/license.md:3: error: unknown reference FS-gone\n";
    for args in [
        &["check"][..],
        &["check", "--full"],
        &["check", "other/license.rs"],
        &["check", "docs/license.md"],
    ] {
        assert_eq!(ran(&tree, args), (1, want.to_string()), "{args:?}");
    }
}

/// `list` and `fmt` handed the name no plain run reaches read the file as that run does: Markdown,
/// where the handed `.rs` name alone would have read a Rust file holding no declaration.
#[test]
fn a_handed_name_reads_as_the_plain_run_reads() {
    let tree = license("plain-read", "# FS-lic: L\n\nx = \"\u{a7}FS-a\"\n");
    let listed = tree.grund(&["list", "other/license.rs"]);
    assert_eq!(
        listed.stdout, "FS-lic  other/license.rs:1  L\n",
        "{}",
        listed.stderr
    );

    let by_plain = tree.grund(&["fmt", "docs/license.md", "--check"]);
    let by_handed = tree.grund(&["fmt", "other/license.rs", "--check"]);
    assert_eq!(by_handed.code, by_plain.code, "{}", by_handed.stderr);
    assert_eq!(by_handed.code, 1);
    assert!(
        by_handed.stdout.contains(":3: markdown link"),
        "{}",
        by_handed.stdout
    );
    assert!(
        !by_handed.stderr.contains("read this file differently"),
        "{}",
        by_handed.stderr
    );
}

/// A root the configuration passes over is not a root by being handed: `build/` is excluded.
#[test]
fn a_handed_name_under_an_excluded_directory_decides_nothing() {
    let tree = Tree::new("plain-excluded", V1_EXCLUDED);
    tree.put("docs/fs/a.md", "# FS-a: A\n\nCites \u{a7}FS-lic.\n");
    tree.put("LICENSE", "# FS-lic: L\n\nCites \u{a7}FS-a.\n");
    tree.link("docs/license.md", "../LICENSE");
    tree.link("build/license.rs", "../LICENSE");
    for args in [
        &["check"][..],
        &["check", "--full"],
        &["check", "build/license.rs"],
        &["check", "docs/license.md"],
    ] {
        let run = tree.grund(args);
        assert_eq!(
            (run.code, run.stdout.as_str()),
            (0, ""),
            "{args:?}: {}",
            run.stderr
        );
    }
    // `fmt` handed that name finds in the file what it finds under the plain one, and refuses nothing.
    let by_handed = tree.grund(&["fmt", "build/license.rs", "--check"]);
    assert_eq!(by_handed.stdout, "build/license.rs:3: markdown link\n");
    assert!(
        !by_handed.stderr.contains("differently"),
        "{}",
        by_handed.stderr
    );
    let by_plain = tree.grund(&["fmt", "docs/license.md", "--check"]);
    assert_eq!(by_plain.stdout, "docs/license.md:3: markdown link\n");
}

/// Where the plain run reaches no name there is nothing for it to agree with, so the names the run
/// met are judged, and the line stands in the out-of-scope tier a `--full` line belongs to.
#[test]
fn a_file_no_plain_name_reaches_is_judged_out_of_scope() {
    let tree = Tree::new("plain-none", V1);
    tree.put("docs/fs/a.md", "# FS-a: A\n\nCites \u{a7}FS-a.\n");
    tree.put("LICENSE", "# FS-x: X\n");
    tree.link("other/a.md", "../LICENSE");
    tree.link("other/a.rs", "../LICENSE");
    let want = "other/a.md:1: error: outside [scan] include: other/a.md and other/a.rs are one \
                file that reads two ways, as markdown and as rs; keep one name, or give the file \
                a name that claims a reading\n";
    assert_eq!(ran(&tree, &["check", "--full"]), (1, want.to_string()));
    assert_eq!(ran(&tree, &["check", "other"]), (1, want.to_string()));
    assert_eq!(
        ran(&tree, &["check"]).0,
        0,
        "the plain run reaches neither name"
    );
}

/// A move no plain name is part of is the same tier's line, from `--full` and from the handed name.
#[test]
fn a_move_no_plain_name_reaches_is_judged_out_of_scope() {
    let tree = Tree::new("plain-moved-none", V1);
    tree.put("docs/fs/a.md", "# FS-a: A\n\nCites \u{a7}FS-a.\n");
    tree.put("build/gen.rs", "// text\n");
    tree.link("aaa/gen.md", "../build/gen.rs");
    let want = "aaa/gen.md:1: warning: outside [scan] include: read as rs, as its target's own \
                name reads, not as markdown (since 0.22.0); rename the target or copy the file\n";
    assert_eq!(ran(&tree, &["check"]), (0, String::new()));
    assert_eq!(ran(&tree, &["check", "--full"]), (0, want.to_string()));
    assert_eq!(ran(&tree, &["check", "aaa/gen.md"]), (0, want.to_string()));
}

/// A move names the first name the plain run reaches, here under `--full` too, though `aaa/gen.md`
/// sorts ahead of it there. Handed, a name no plain run reaches decides nothing and earns nothing.
/// Passes either way: a plain name was already listed first. It holds the choice to the one rule.
#[test]
fn a_moved_reading_goes_by_the_plain_names() {
    let tree = Tree::new("plain-moved", V1);
    tree.put("docs/fs/a.md", "# FS-a: A\n\nCites \u{a7}FS-a.\n");
    tree.put("build/gen.rs", "// text\n");
    tree.link("docs/gen.md", "../build/gen.rs");
    tree.link("aaa/gen.md", "../build/gen.rs");
    let want = "docs/gen.md:1: warning: read as rs, as its target's own name reads, not as \
                markdown (since 0.22.0); rename the target or copy the file\n";
    assert_eq!(ran(&tree, &["check"]), (0, want.to_string()));
    assert_eq!(ran(&tree, &["check", "--full"]), (0, want.to_string()));
    assert_eq!(ran(&tree, &["check", "aaa/gen.md"]), (0, String::new()));
}
