//! Black-box: how a file is read belongs to the file, not to the name that reached it
//! (§FS-scan.identity.reading, §FS-scan.requirements.spelling, §DF-one-file-one-reading).
//! The fused reproducer of #1248 is here: renaming a link changed what `fmt` wrote and could
//! drop a declaration on a green `check`. Git carries no link a fixture could keep portable.
#![cfg(unix)]

#[path = "support/two_names.rs"]
#[allow(dead_code)]
mod two_names;

use two_names::{TXT_IS_RUST, Tree};

const V1: &str = "[scan]\nextensions = [\"md\", \"txt\"]\n";

fn fmt_tree(case: &str, version: u8) -> (Tree, &'static str, &'static str) {
    if version == 1 {
        let tree = Tree::new(case, V1);
        tree.put(
            "docs/functional-spec/FS-001-a.md",
            "# FS-001-a: A\n\nBody.\n",
        );
        (tree, "src", "FS-001-a")
    } else {
        let tree = Tree::new(case, TXT_IS_RUST);
        tree.put("fs/h.md", "# FS-a: A\n\nBody.\n\nCites $$FS-a.\n");
        (tree, "fs", "FS-a")
    }
}

/// The bytes `fmt --write` writes for one file under each link name and each scope, which must
/// be those of the same tree with no link (§FS-scan.identity.reading.every-caller).
fn every_name_and_scope_writes_one_set_of_bytes(version: u8) {
    let (control, folder, id) = fmt_tree(&format!("fmt-v{version}-control"), version);
    let original = format!("Quoted \"$${id}\" here.\nOutside $${id} here.\n");
    let target = format!("{folder}/m-real.md");
    control.put(&target, &original);
    assert_eq!(control.grund(&["fmt", "--write"]).code, 0);
    let expected = control.read(&target);
    assert_ne!(expected, original, "the control rewrites something");

    let mut wrong = Vec::new();
    for alias in ["a-link.txt", "z-link.txt"] {
        let (tree, _, _) = fmt_tree(&format!("fmt-v{version}-{alias}"), version);
        let link = format!("{folder}/{alias}");
        tree.put(&target, &original);
        tree.link(&link, "m-real.md");
        for scope in [None, Some(target.as_str()), Some(link.as_str())] {
            tree.put(&target, &original);
            let mut args = vec!["fmt"];
            args.extend(scope);
            args.push("--write");
            let run = tree.grund(&args);
            let bytes = tree.read(&target);
            if bytes != expected || run.code != 0 {
                wrong.push(format!("{alias} {scope:?}: exit {}, {bytes:?}", run.code));
            }
        }
    }
    assert!(wrong.is_empty(), "want {expected:?}, got: {wrong:#?}");
}

#[test]
fn renaming_a_link_changes_nothing_fmt_writes_under_version_2() {
    every_name_and_scope_writes_one_set_of_bytes(2);
}

#[test]
fn renaming_a_link_changes_nothing_fmt_writes_under_version_1() {
    every_name_and_scope_writes_one_set_of_bytes(1);
}

/// #1248's green disappearance: `FS-b` is there under either link name, in every scope.
#[test]
fn renaming_a_link_changes_nothing_a_file_declares() {
    let mut wrong = Vec::new();
    for alias in ["a-link.txt", "z-link.txt"] {
        let tree = Tree::new(&format!("declares-{alias}"), TXT_IS_RUST);
        tree.put("fs/h.md", "# FS-a: A\n\nCites \u{a7}FS-a.\n");
        tree.put("fs/m-real.md", "# FS-b: B\n\nBody.\n");
        let link = format!("fs/{alias}");
        tree.link(&link, "m-real.md");
        for scope in [None, Some("fs/m-real.md"), Some(link.as_str())] {
            let mut args = vec!["list"];
            args.extend(scope);
            let run = tree.grund(&args);
            if !run.stdout.contains("FS-b") {
                wrong.push(format!(
                    "{alias} {scope:?}: list has no FS-b: {}",
                    run.stdout
                ));
            }
        }
        tree.put("fs/cites.md", "Cites \u{a7}FS-b.\n");
        let cited = tree.grund(&["check"]);
        if cited.code != 0 {
            wrong.push(format!(
                "{alias}: a citation to FS-b is red: {}",
                cited.stdout
            ));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// The other direction: `.txt` is Rust by its own name, so a link named `.md` reads it as Rust
/// and `fmt` leaves its string literal alone (§FS-config-v2.schema.sources.extensions).
#[test]
fn a_link_named_md_to_a_rust_named_file_reads_as_rust() {
    let tree = Tree::new("md-to-rust", TXT_IS_RUST);
    tree.put("fs/h.md", "# FS-a: A\n\nCites \u{a7}FS-a.\n");
    let source = "# FS-c: C\n\nlet x = \"$$FS-a\";\n";
    tree.put("fs/z.txt", source);
    tree.link("fs/a.md", "z.txt");
    assert!(
        !tree.grund(&["list"]).stdout.contains("FS-c"),
        "read as Markdown"
    );
    assert_eq!(tree.grund(&["fmt", "--write"]).code, 0);
    assert_eq!(
        tree.read("fs/z.txt"),
        source,
        "a Rust literal was rewritten"
    );
}

fn ignored_rust_target(case: &str) -> Tree {
    let tree = Tree::new(
        case,
        "grund_config_version = 2\nproject_name = \"gen\"\n\
         [schema.sources]\nlanguages = [\"markdown\", \"rust\"]\nexclude = [\"build/\"]\n\
         [schema.kinds.FS]\nfolders = [\"docs/fs\"]\nindex = false\n",
    );
    tree.put("docs/fs/a.md", "# FS-a: A\n\nCites \u{a7}FS-a.\n");
    tree.put("build/gen.rs", "# FS-g: G\n\nx = \"\u{a7}FS-a\"\n");
    tree.link("docs/fs/gen.md", "../../build/gen.rs");
    tree
}

/// The compatibility case: an in-root target whose own spelling is never scanned still claims by
/// its name, and the link earns the one warning (§FS-scan.identity.reading.claims,
/// §FS-scan.checks.reading-moved).
#[test]
fn an_excluded_targets_own_name_still_claims_a_reading() {
    let tree = ignored_rust_target("excluded-target");
    let check = tree.grund(&["check"]);
    assert_eq!(check.code, 0, "the verdict stays: {}", check.stdout);
    assert!(
        check.stdout.lines().any(|line| line
            == "docs/fs/gen.md:1: warning: read as rust, as its target's own name reads, \
                not as markdown (since 0.22.0); rename the target or copy the file"),
        "{}",
        check.stdout
    );
    assert!(
        !tree.grund(&["list"]).stdout.contains("FS-g"),
        "read as Markdown"
    );
    let source = tree.read("build/gen.rs");
    assert_eq!(tree.grund(&["fmt", "--write"]).code, 0);
    assert_eq!(
        tree.read("build/gen.rs"),
        source,
        "a Rust literal was rewritten"
    );

    tree.put("docs/fs/cites.md", "Cites \u{a7}FS-g.\n");
    let cited = tree.grund(&["check"]);
    assert_eq!(
        cited.code, 1,
        "a citation to the gone declaration: {}",
        cited.stdout
    );
    assert!(
        cited.stdout.contains("unknown reference FS-g"),
        "{}",
        cited.stdout
    );
}

/// "At each link whose reading moved": the name listed first decided the file before, so only it
/// earns the warning, and a link listed after its target earns none
/// (§FS-scan.checks.reading-moved.1).
#[test]
fn the_warning_stands_at_the_link_whose_reading_moved_and_nowhere_else() {
    for (alias, moved) in [("a-link.txt", true), ("z-link.txt", false)] {
        let tree = Tree::new(&format!("moved-{alias}"), TXT_IS_RUST);
        tree.put("fs/h.md", "# FS-a: A\n\nCites \u{a7}FS-a and \u{a7}FS-b.\n");
        tree.put("fs/m-real.md", "# FS-b: B\n\nBody.\n");
        tree.link(&format!("fs/{alias}"), "m-real.md");
        let run = tree.grund(&["check"]);
        assert_eq!(run.code, 0, "{}", run.stdout);
        let line = format!(
            "fs/{alias}:1: warning: read as markdown, as its target's own name reads, \
             not as rust (since 0.22.0); rename the target or copy the file\n"
        );
        let want = if moved { line } else { "success\n".to_string() };
        assert_eq!(run.stdout, want);
        if moved {
            assert_eq!(
                tree.grund(&["check", "--ignore", "reading-moved"]).stdout,
                "success\n"
            );
            let json = tree.grund(&["check", "--only", "reading-moved", "--format", "json"]);
            assert_eq!(json.findings()[0]["code"], "reading-moved");
            assert_eq!(json.code, 0);
        }
    }
}

fn license_under_v1(case: &str, extensions: &str) -> Tree {
    let tree = Tree::new(
        case,
        &format!(
            "project_name = \"v\"\n[scan]\nextensions = [{extensions}]\ninclude = [\"docs\"]\n\n\
             [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n"
        ),
    );
    tree.put("LICENSE", "# FS-001-l: L\n\nBody.\n");
    tree.link("docs/license.md", "../LICENSE");
    tree
}

/// The owner's addition: a v1 list holding `""` reads `LICENSE`, so there it claims a reading and
/// the link stops being Markdown (§FS-scan.identity.reading.claims,
/// §FS-scan.scope.extensions.empty).
#[test]
fn a_v1_empty_extension_makes_license_claim_a_reading() {
    let tree = license_under_v1("v1-empty-entry", "\"md\", \"\"");
    tree.put("docs/a.md", "# FS-001-a: A\n\nCites \u{a7}FS-001-a.\n");
    let source = "Quoted \"$$FS-001-a\" here.\n";
    tree.put("LICENSE", source);
    let check = tree.grund(&["check"]);
    assert_eq!(check.code, 0, "{}", check.stdout);
    assert!(
        check.stdout.lines().any(|line| line
            == "docs/license.md:1: warning: read as source with no extension, as its target's \
                own name reads, not as markdown (since 0.22.0); rename the target or copy the file"),
        "{}",
        check.stdout
    );
    assert_eq!(tree.grund(&["fmt", "--write"]).code, 0);
    assert_eq!(tree.read("LICENSE"), source, "read as Markdown");
}

/// The value reader asks the rule too: this file is YAML by its own name, and `paths[0]`, a link
/// named `.json`, no longer decides (§FS-scan.identity.reading.every-caller).
#[test]
fn the_value_reader_takes_its_format_from_the_files_own_name() {
    let tree = Tree::new(
        "value-format",
        "grund_config_version = 2\nproject_name = \"h\"\n\
         [schema.kinds.CONST]\nfolders = [\"values\"]\nform = \"value\"\nindex = false\n",
    );
    tree.put("values/limits.yaml", "CONST-a: [1, x]\n");
    tree.link("values/a.json", "limits.yaml");
    let check = tree.grund(&["check"]);
    assert!(!check.stderr.contains("invalid JSON"), "{}", check.stderr);
    assert_eq!(check.code, 0, "{}{}", check.stdout, check.stderr);
    assert!(tree.grund(&["list"]).stdout.contains("CONST-a"));
}

// Guards: these hold today, and hold what the fix must not break.

/// Guard: `docs/license.md -> ../LICENSE` and `src/gen.rs -> ../build/gen.rs.in` must stay read
/// by the link's name, since the target's own name claims nothing
/// (§FS-scan.identity.reading.claims).
#[test]
fn a_target_whose_own_name_claims_nothing_keeps_the_links_reading() {
    let tree = Tree::new(
        "claims-nothing",
        "grund_config_version = 2\nproject_name = \"c\"\n\
         [schema.sources]\nlanguages = [\"markdown\", \"rust\"]\n\
         [schema.kinds.FS]\nfolders = [\"docs/fs\"]\nindex = false\n",
    );
    tree.put("LICENSE", "# FS-l: L\n\nBody.\n");
    tree.put("build/gen.rs.in", "/// FS-y: Y\nfn y() {}\n");
    tree.link("docs/license.md", "../LICENSE");
    tree.link("src/gen.rs", "../build/gen.rs.in");
    let list = tree.grund(&["list"]);
    assert!(list.stdout.contains("FS-l"), "{}", list.stdout);
    assert!(list.stdout.contains("FS-y"), "{}", list.stdout);
}

/// Guard: under v1 with no `""`, `LICENSE` claims nothing, so the link stays Markdown.
#[test]
fn without_the_empty_extension_license_claims_nothing() {
    let tree = license_under_v1("v1-no-entry", "\"md\"");
    assert!(tree.grund(&["list"]).stdout.contains("FS-001-l"));
}

/// Guard: an outside name has no say, so a Markdown link to `/elsewhere/spec.txt` keeps its
/// declarations with no second name in sight (§FS-scan.identity.reading.outside).
#[test]
fn a_target_outside_the_root_keeps_the_links_reading() {
    let outside = Tree::new("outside-target", "");
    outside.put("spec.txt", "# FS-o: O\n\nBody.\n");
    let tree = Tree::new("outside-root", TXT_IS_RUST);
    tree.link("fs/spec.md", outside.root.join("spec.txt"));
    let list = tree.grund(&["list"]);
    assert!(
        list.stdout.contains("FS-o  fs/spec.md:1"),
        "{}",
        list.stdout
    );
}

/// Guard: whether a spelling enters the scan is judged by the name met
/// (§FS-scan.identity.reading.entry).
#[test]
fn a_spelling_the_scan_does_not_admit_stays_unread() {
    let tree = Tree::new(
        "entry",
        "grund_config_version = 2\nproject_name = \"e\"\n\
         [schema.kinds.FS]\nfolders = [\"fs\"]\nindex = false\n",
    );
    tree.put(
        "fs/spec.md",
        "# FS-s: S\n\nCites \u{a7}FS-s and \u{a7}FS-gone.\n",
    );
    tree.link("fs/notes.txt", "spec.md");
    tree.link("fs/.hidden.md", "spec.md");
    let check = tree.grund(&["check"]);
    assert_eq!(
        check.stdout,
        "fs/spec.md:3: error: unknown reference FS-gone\n"
    );
}

/// Guard: hard links are two files in the walk and the value scan alike, and stay a duplicate
/// (§FS-scan.identity.read-once).
#[test]
fn hard_links_stay_two_files() {
    let tree = Tree::new("hard", TXT_IS_RUST);
    tree.put("fs/a.md", "# FS-x: X\n\nCites \u{a7}FS-x.\n");
    std::fs::hard_link(tree.root.join("fs/a.md"), tree.root.join("fs/b.md")).unwrap();
    let check = tree.grund(&["check"]);
    assert_eq!(
        check.stdout,
        "fs/a.md:1: error: duplicate declaration of FS-x (also declared at fs/b.md:1)\n"
    );
}
