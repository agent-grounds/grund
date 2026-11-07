//! Black-box: an index is owed by the folder a spelling sits in, its finding stands at that
//! spelling, and it is the same file under every name that reaches it
//! (§FS-scan.identity.findings.place, §FS-check.3.18, §DF-one-file-one-reading). `fmt --write`
//! never adds a missing entry (§FS-fmt.6.1), so the repair is the maintainer's and the finding
//! names the link to write.
#![cfg(unix)]

#[path = "support/two_names.rs"]
#[allow(dead_code)]
mod two_names;

use two_names::Tree;

const V1_ALIAS: &str = "grund_config_version = 1\n\n[scan]\ninclude = [\"docslink\"]\n";
const FS: &str = "# FS-001-login: A user can log in\n\nSee \u{a7}FS-001-login.\n";
const LINK: &str = "FS-001-login.md#fs-001-login-a-user-can-log-in";

fn alias(case: &str) -> Tree {
    let tree = Tree::new(case, V1_ALIAS);
    tree.put("docs/functional-spec/FS-001-login.md", FS);
    tree.link("docslink", "docs");
    tree
}

/// The line stands at the spelling in the home whichever root is listed first, and the link it
/// offers is the one `fmt` keeps once it is written.
#[test]
fn an_aliased_home_owes_its_index_and_says_where() {
    let tree = alias("alias-absent");
    let check = tree.grund(&["check"]);
    let want = format!(
        "docs/functional-spec/FS-001-login.md:1: error: FS-001-login is not listed in \
         docs/functional-spec/README.md (the index file does not exist): an entry is a \
         `\u{a7}`-marked Markdown link to the declaration; write `[\u{a7}FS-001-login]({LINK})` \
         in docs/functional-spec/README.md \u{2014} became an error in grund 0.13.0\n"
    );
    assert_eq!(check.stdout, want);
    assert_eq!(check.code, 1);

    let entry = format!("# Index\n\n- [\u{a7}FS-001-login]({LINK})\n");
    tree.put("docs/functional-spec/README.md", &entry);
    let written = tree.grund(&["check"]);
    assert_eq!(
        (written.code, written.stdout.as_str()),
        (0, ""),
        "{}",
        written.stderr
    );
    assert_eq!(tree.grund(&["fmt", "--write"]).code, 0);
    assert_eq!(tree.read("docs/functional-spec/README.md"), entry);
}

/// A README with no entry is as red as a missing one, though it was read under the other name.
#[test]
fn an_aliased_readme_with_no_entry_is_red() {
    let tree = alias("alias-empty");
    tree.put("docs/functional-spec/README.md", "# Index\n");
    let check = tree.grund(&["check"]);
    assert!(
        check.stdout.starts_with(
            "docs/functional-spec/FS-001-login.md:1: error: FS-001-login is not listed in \
             docs/functional-spec/README.md: "
        ),
        "{}",
        check.stdout
    );
    assert_eq!(check.stdout.lines().count(), 1, "{}", check.stdout);
    assert_eq!(check.code, 1);
}

/// `fmt` wraps a bare entry in the index it meets under the alias, as `check` says it will.
#[test]
fn a_bare_entry_is_linked_under_the_name_fmt_meets_it_by() {
    let tree = alias("alias-bare");
    tree.put(
        "docs/functional-spec/README.md",
        "# Index\n\n- \u{a7}FS-001-login\n",
    );
    let check = tree.grund(&["check"]);
    assert!(check.stdout.contains("is not a link"), "{}", check.stdout);
    assert_eq!(tree.grund(&["fmt", "--write"]).code, 0);
    let index = tree.read("docs/functional-spec/README.md");
    assert!(
        index.contains(&format!("[\u{a7}FS-001-login]({LINK})")),
        "{index}"
    );
    let fixed = tree.grund(&["check"]);
    assert_eq!(
        (fixed.code, fixed.stdout.as_str()),
        (0, ""),
        "{}",
        fixed.stderr
    );
}

/// A link in a second folder of the row owes that folder's index, and the finding stands at the
/// link rather than at the file's other name. The maintainer writes the entry, as it says.
#[test]
fn a_link_in_a_second_folder_is_located_at_the_link() {
    let config = "grund_config_version = 2\nproject_name = \"i\"\n[schema.sources]\n\
        languages = [\"markdown\"]\n[schema.kinds.FS]\nfolders = [\"aa\", \"zz\"]\n";
    let tree = Tree::new("second", config);
    tree.put("aa/x.md", "# FS-x: X\n\nCites \u{a7}FS-x.\n");
    tree.link("zz/x.md", "../aa/x.md");
    tree.put("aa/README.md", "# Index\n\n- [\u{a7}FS-x](x.md#fs-x-x)\n");
    tree.put("zz/README.md", "# Index\n");
    let check = tree.grund(&["check"]);
    assert!(
        check.stdout.starts_with(
            "zz/x.md:1: error: FS-x is not listed in zz/README.md: an entry is a \
             `\u{a7}`-marked Markdown link to the declaration; write \
             `[\u{a7}FS-x](../aa/x.md#fs-x-x)` in zz/README.md"
        ),
        "{}",
        check.stdout
    );
    assert_eq!(check.stdout.lines().count(), 1, "{}", check.stdout);
    // `fmt --write` adds no entry: the finding stays until someone writes the one it names.
    assert_eq!(tree.grund(&["fmt", "--write"]).code, 0);
    assert_eq!(tree.read("zz/README.md"), "# Index\n");
    tree.put(
        "zz/README.md",
        "# Index\n\n- [\u{a7}FS-x](../aa/x.md#fs-x-x)\n",
    );
    assert_eq!(tree.grund(&["check"]).stdout, "success\n");
}
