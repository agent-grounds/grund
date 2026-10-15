//! Stub filters match the walk's link spelling (§FS-declarations.stubs.target.3,
//! §FS-scan.scope.links.names); rejected aliases lend no body or sections
//! (§FS-declarations.stubs.target.5).

#![cfg(unix)]

use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

const CONFIG: &str = "grund_config_version = 1\n\
    [reference]\nstrict = true\nrequire_grounding = false\n\
    [id]\nformat = \"{kind}-{slug}\"\n\
    [[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\nindex = false\n\
    [scan]\ninclude = [\"docs\", \"notes\"]\n";

const TARGET: &str = "# FS-x: X\n\nLead.\n\n## 1. Detail\n\nAlias section.\n";

fn write(root: &Path, path: &str, body: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

fn grund(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(root)
        .env("GIT_CONFIG_GLOBAL", root.join("xdg/git/config"))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("XDG_CONFIG_HOME", root.join("xdg"))
        .output()
        .expect("run grund")
}

fn json_lines(output: &Output) -> Vec<Value> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).expect("JSON line"))
        .collect()
}

fn tree(name: &str, alias: &str, physical: &str, exclude: bool) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stub-target-aliases")
        .join(format!("{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("notes")).unwrap();
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(&root)
            .status()
            .unwrap()
            .success()
    );
    write(
        &root,
        "grund.toml",
        &format!(
            "{CONFIG}exclude = [{}]\n",
            if exclude { "\"gen\"" } else { "" }
        ),
    );
    write(&root, "docs/fs/refs.md", "Uses \u{a7}FS-x.\n");
    write(&root, &format!("{physical}/a.md"), TARGET);
    symlink(format!("../{physical}"), root.join("notes").join(alias)).unwrap();
    root
}

fn stub(root: &Path, id: &str, alias: &str) {
    write(
        root,
        &format!("docs/fs/{id}.md"),
        &format!("# FS-{id}: [{id}](../../notes/{alias}/a.md)\n"),
    );
}

/// Compare the walk without a stub to check/show after adding it, including the
/// false-refusal case where a readable alias must pair without a duplicate.
fn single(root: &Path, alias: &str, broken: bool) {
    let walked = grund(root, &["list", "--format", "json"]);
    assert!(walked.status.success());
    let rows = json_lines(&walked);
    assert_eq!(rows.len(), usize::from(!broken));
    if !broken {
        assert_eq!(rows[0]["path"], format!("notes/{alias}/a.md"));
    }
    stub(root, "x", alias);
    let checked = grund(root, &["check", "--format", "json"]);
    let findings = json_lines(&checked);
    assert_eq!(
        checked.status.code(),
        Some(i32::from(broken)),
        "{checked:?}"
    );
    assert_eq!(findings.len(), usize::from(broken), "{findings:?}");
    if broken {
        assert_eq!(findings[0]["code"], "broken-stub");
        assert_eq!(findings[0]["severity"], "error");
        assert_eq!(findings[0]["path"], "docs/fs/x.md");
    }
    for (id, body) in [("FS-x", "Lead."), ("FS-x.1", "Alias section.")] {
        let shown = grund(root, &["show", id]);
        assert_eq!(shown.status.code(), Some(i32::from(broken)), "{shown:?}");
        assert_eq!(
            String::from_utf8_lossy(&shown.stdout).contains(body),
            !broken,
            "{shown:?}"
        );
        if broken {
            assert!(String::from_utf8_lossy(&shown.stderr).contains("broken stub: FS-x"));
        }
    }
    let rows = json_lines(&grund(root, &["list", "--format", "json"]));
    assert_eq!(
        rows.len(),
        1,
        "a readable alias pairs with its stub: {rows:?}"
    );
    assert_eq!(rows[0]["duplicate"], false);
}

#[test]
fn an_excluded_alias_cannot_bypass_the_walk() {
    let root = tree("excluded-alias", "gen", "storage", true);
    single(&root, "gen", true);
}

#[test]
fn an_ignored_directory_alias_cannot_bypass_the_walk() {
    for file in [".gitignore", ".ignore"] {
        let root = tree(&format!("ignored-alias-{file}"), "gen", "storage", false);
        write(&root, file, "notes/gen/\n");
        single(&root, "gen", true);
    }
}

#[test]
fn an_excluded_physical_name_does_not_refuse_a_readable_alias() {
    let root = tree("excluded-physical", "readable", "gen", true);
    single(&root, "readable", false);
}

#[test]
fn an_ignored_physical_name_does_not_refuse_a_readable_alias() {
    for file in [".gitignore", ".ignore"] {
        let root = tree(
            &format!("ignored-physical-{file}"),
            "readable",
            "gen",
            false,
        );
        write(&root, file, "gen/\n");
        single(&root, "readable", false);
    }
}

#[test]
fn an_unrelated_ignore_keeps_the_alias_readable() {
    let root = tree("unrelated-ignore", "gen", "storage", false);
    write(&root, ".gitignore", "unrelated/\n");
    single(&root, "gen", false);
}

/// §FS-config-v2.schema.sources.exclude.stubs: config-relative patterns also
/// match the lexical alias, not the physical directory behind it.
#[test]
fn v2_patterns_filter_the_alias_spelling() {
    for (alias, physical, broken) in [("gen", "storage", true), ("readable", "gen", false)] {
        let root = tree(&format!("v2-alias-{alias}"), alias, physical, false);
        write(
            &root,
            "grund.toml",
            "grund_config_version = 2\nproject_name = \"aliases\"\n\
            [schema.sources]\nlanguages = [\"markdown\"]\nexclude = [\"/notes/gen/\", \"/gen/\"]\n\
            [schema.kinds.FS]\nfolders = [\"docs/fs\"]\nindex = false\n\
            [schema.kinds.code]\nfolders = [\"notes\"]\n",
        );
        single(&root, alias, broken);
    }
}

/// A shared physical target has two IDs, so BTreeMap ID order controls which
/// alias asks the cache first. Exercise both permission and refusal first.
#[test]
fn alias_eligibility_is_isolated_in_both_processing_orders() {
    for ignore in [false, true] {
        for refused in ["a", "b"] {
            let allowed = if refused == "a" { "b" } else { "a" };
            let root = tree(
                &format!("cache-{ignore}-{refused}"),
                "gen",
                "storage",
                !ignore,
            );
            write(
                &root,
                "grund.toml",
                &format!(
                    "{}exclude = [{}]\n",
                    CONFIG.replace("[\"docs\", \"notes\"]", "[\"docs\"]"),
                    if ignore { "" } else { "\"gen\"" }
                ),
            );
            if ignore {
                write(&root, ".gitignore", "notes/gen/\n");
            }
            symlink("../storage", root.join("notes/readable")).unwrap();
            write(
                &root,
                "storage/a.md",
                "# FS-a: A\n\nBody a.\n\n## 1. Detail\n\nSection a.\n\n# FS-b: B\n\nBody b.\n\n## 1. Detail\n\nSection b.\n",
            );
            write(
                &root,
                "docs/fs/refs.md",
                "Uses \u{a7}FS-a and \u{a7}FS-b.\n",
            );
            stub(&root, refused, "gen");
            stub(&root, allowed, "readable");
            let checked = grund(&root, &["check", "--format", "json"]);
            assert_eq!(checked.status.code(), Some(1), "{checked:?}");
            let findings = json_lines(&checked);
            assert_eq!(findings.len(), 1, "{findings:?}");
            assert_eq!(findings[0]["code"], "broken-stub");
            assert_eq!(findings[0]["path"], format!("docs/fs/{refused}.md"));
            for (id, broken) in [(refused, true), (allowed, false)] {
                for section in ["", ".1"] {
                    let shown = grund(&root, &["show", &format!("FS-{id}{section}")]);
                    assert_eq!(shown.status.code(), Some(i32::from(broken)), "{shown:?}");
                    assert_eq!(shown.stdout.is_empty(), broken, "{shown:?}");
                }
            }
        }
    }
}

#[test]
fn an_explicit_alias_root_outruns_exclude_and_ignore() {
    for ignore in [false, true] {
        let root = tree(&format!("alias-root-{ignore}"), "gen", "storage", !ignore);
        let config = fs::read_to_string(root.join("grund.toml")).unwrap();
        write(
            &root,
            "grund.toml",
            &config.replace("\"notes\"", "\"notes/gen\""),
        );
        if ignore {
            write(&root, ".gitignore", "notes/gen/\n");
        }
        single(&root, "gen", false);
    }
}
