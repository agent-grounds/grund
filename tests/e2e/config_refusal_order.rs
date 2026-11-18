//! Which refusal a version-2 config holding several is refused for (§FS-config-v2.reader.4.order):
//! the reading, then the whole file, then the meanings both versions share with the citation
//! tables last, then the pinned targets the run judges, and only then the six asked last, at the
//! earliest line and, at one line, the entry written first. A citation-table refusal names its
//! line (§FS-config-v2.rules.citations). Each tree is built in the temp directory, and each
//! refusal is asserted on `check` (exit 2) and `config validate` (exit 1).

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tells apart the trees tests running in parallel build from one case.
static TREES: AtomicUsize = AtomicUsize::new(0);

/// The `md` refusal every tree that maps `md = "json"` at line 4 would print if it came first.
const MD_AT_4: &str = "grund.toml:4: [schema.sources.extensions] maps `md` to `json`";

/// The head of a config that maps `md` away from Markdown at line 4 and configures `FS` at 6-7.
const MD_JSON: &str = "grund_config_version = 2\n\n[schema.sources.extensions]\nmd = \"json\"\n\n\
                       [schema.kinds.FS]\nfolders = [\"docs\"]\n";

/// A project tree under the temp directory holding each `(path, text)`, removed on drop.
struct Tree(PathBuf);

impl Tree {
    fn new(name: &str, files: &[(&str, &str)]) -> Self {
        let n = TREES.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "grund-refusal-order-{name}-{}-{n}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        for (path, text) in files {
            let path = root.join(path);
            fs::create_dir_all(path.parent().expect("a parent")).expect("create the parent");
            fs::write(path, text).expect("write a file");
        }
        Self(root)
    }

    /// The stderr of `check` and of `config validate`, each with its exit status asserted.
    fn refusals(&self) -> [String; 2] {
        [(&["check"][..], 2), (&["config", "validate"][..], 1)].map(|(args, code)| {
            let output = Command::new(env!("CARGO_BIN_EXE_grund"))
                .args(args)
                .current_dir(&self.0)
                .env("NO_COLOR", "1")
                .output()
                .expect("run grund");
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            assert_eq!(output.status.code(), Some(code), "{args:?}: {stderr}");
            stderr
        })
    }

    /// Both commands print exactly `refusal`.
    fn refused_for(&self, refusal: &str) {
        for stderr in self.refusals() {
            assert_eq!(stderr, format!("error: {refusal}\n"));
        }
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// An unknown target in a citation table wins over the `md` mapping on an earlier line.
#[test]
fn an_unknown_citation_target_wins_over_the_six() {
    let config = format!("{MD_JSON}\n[rules.citations.FS]\nmust = [\"NOPE\"]\n");
    Tree::new("target", &[("grund.toml", &config)]).refused_for(
        "grund.toml:10: [rules.citations.FS] must names an unknown target kind `NOPE`",
    );
}

/// An unknown citing kind is named at its table's header, and wins over the six.
#[test]
fn an_unknown_citing_kind_wins_over_the_six() {
    let config = format!("{MD_JSON}\n[rules.citations.NOPE]\nmust = [\"FS\"]\n");
    Tree::new("citing", &[("grund.toml", &config)])
        .refused_for("grund.toml:9: [rules.citations.NOPE] names an unknown kind `NOPE`");
}

/// A rule home no admitted language reads as Markdown waits for the citation tables.
#[test]
fn a_rule_home_refusal_waits_for_the_citation_tables() {
    let config = "grund_config_version = 2\n\n[schema.sources]\nlanguages = [\"rust\"]\n\n\
                  [schema.kinds.RULE]\nform = \"rule\"\nfolders = [\"rules\"]\n\n\
                  [rules.citations.RULE]\nmust = [\"NOPE\"]\n";
    Tree::new("rule-home", &[("grund.toml", config), ("rules/.keep", "")]).refused_for(
        "grund.toml:11: [rules.citations.RULE] must names an unknown target kind `NOPE`",
    );
}

/// A kind collision, judged with the kind table, wins over the six.
#[test]
fn a_kind_collision_wins_over_the_six() {
    let config = format!("{MD_JSON}\n[schema.kinds.FS_X]\nfolders = [\"d2\"]\n");
    for stderr in Tree::new("collision", &[("grund.toml", &config)]).refusals() {
        assert!(stderr.contains("kinds `FS` and `FS_X` collide"), "{stderr}");
        assert!(!stderr.contains(MD_AT_4), "{stderr}");
    }
}

/// A non-citable target is named at the line of the list that names it.
#[test]
fn a_non_citable_target_is_refused_at_its_list_line() {
    let config = "grund_config_version = 2\n\n[schema.kinds.T]\nfolders = [\"a\"]\ncitable = false\n\n\
                  [schema.kinds.FS]\nfolders = [\"docs\"]\n\n[rules.citations.FS]\nmust = [\"FS\"]\n\
                  warn = [\"T\", \"NOPE\"]\n";
    Tree::new("non-citable", &[("grund.toml", config)]).refused_for(
        "grund.toml:12: [rules.citations.FS] warn names a non-citable target kind `T`",
    );
}

/// An unwalked citing kind is named at its table's header.
#[test]
fn an_unwalked_citing_kind_is_refused_at_its_header() {
    let config = "grund_config_version = 2\n\n[schema.kinds.T]\nfolders = [\"a\"]\ncitable = false\n\
                  scan = false\n\n[schema.kinds.FS]\nfolders = [\"docs\"]\n\n[rules.citations.T]\n\
                  must = [\"FS\"]\n";
    Tree::new("unwalked", &[("grund.toml", config)]).refused_for(
        "grund.toml:11: [rules.citations.T] names an unwalked kind `T` (its home is `scan = false`, \
         so no file in it is checked and the rule could never fire)",
    );
}

/// The whole file's step comes before the shared meanings: a place overlap at line 10 wins over
/// an unknown target at line 4.
#[test]
fn the_whole_file_step_wins_over_a_meaning_on_an_earlier_line() {
    let config = "grund_config_version = 2\n\n[rules.citations.FS]\nmust = [\"NOPE\"]\n\n\
                  [schema.kinds.GOAL]\nfolders = [\"a\"]\n\n[schema.kinds.AR]\nfolders = [\"a\"]\n";
    Tree::new("whole-file", &[("grund.toml", config)]).refused_for(
        "grund.toml:10: place `a` in `[schema.kinds.AR] folders` is already a place of \
         `[schema.kinds.GOAL]` (line 7); a place belongs to one row",
    );
}

/// The reading comes before the whole file: an unknown key at line 11 wins over a place overlap.
#[test]
fn the_reading_wins_over_the_whole_file() {
    let config = "grund_config_version = 2\n\n[schema.kinds.GOAL]\nfolders = [\"a\"]\n\n\
                  [schema.kinds.AR]\nfolders = [\"a\"]\n\n[schema.kinds.FS]\nfolders = [\"docs\"]\n\
                  bogus = 1\n";
    Tree::new("reading", &[("grund.toml", config)]).refused_for(
        "grund.toml:11: unknown key `bogus` in [schema.kinds.FS] (expected files, folders, \
         citable, scan, index, id_format, fetch, form, or closed)",
    );
}

/// At one line of the six, the entry written first is refused, whichever refusal each earns.
#[test]
fn at_one_line_the_entry_written_first_is_refused() {
    let rules = |files: &str| {
        format!(
            "grund_config_version = 2\n\n[schema.kinds.RULE]\nform = \"rule\"\nfiles = [{files}]\n"
        )
    };
    let read_as_source = |entry: &str| {
        format!(
            "grund.toml:5: [schema.kinds.RULE] files entry `{entry}` is read as `rust`, which does \
             not read Markdown, and every home of a rule kind must be; move the rules into a file \
             a `document` language reads; this became an error in grund 1.0.0"
        )
    };
    let homes = [
        ("rules/rules.md", "# Rules\n"),
        ("rules/z.rs", ""),
        ("rules/more.rs", ""),
    ];
    for (files, first) in [
        (
            "\"rules/rules.md\", \"rules/z.rs\", \"rules/more.rs\"",
            "rules/z.rs",
        ),
        (
            "\"rules/rules.md\", \"rules/more.rs\", \"rules/z.rs\"",
            "rules/more.rs",
        ),
    ] {
        let config = rules(files);
        let mut tree = vec![("grund.toml", config.as_str())];
        tree.extend(homes);
        Tree::new("rule-tie", &tree).refused_for(&read_as_source(first));
    }
    let values = |files: &str| {
        format!(
            "grund_config_version = 2\n\n[schema.kinds.CONST]\nform = \"value\"\nfiles = [{files}]\n"
        )
    };
    for (files, refusal) in [
        (
            "\"values/a.md\", \"values/dir.md\", \"values/absent.md\"",
            "grund.toml:5: value home for kind `CONST` must be an existing file: values/dir.md; \
             make it an existing file, or drop the entry; this became an error in grund 1.0.0",
        ),
        (
            "\"values/a.md\", \"values/absent.md\", \"values/dir.md\"",
            "grund.toml:5: value home for kind `CONST` does not exist: values/absent.md; create \
             the file, or drop the entry; this became an error in grund 1.0.0",
        ),
    ] {
        let config = values(files);
        let files = [
            ("grund.toml", config.as_str()),
            ("values/a.md", "# CONST-a: A\n"),
            ("values/dir.md/.keep", ""),
        ];
        Tree::new("value-tie", &files).refused_for(refusal);
    }
}

/// A pinned target a run with no workspace cannot judge wins over the six.
#[test]
fn a_pinned_target_wins_over_the_six() {
    let config = format!("{MD_JSON}\n[rules.citations.FS]\nmust = [\"web/ZED\"]\n");
    for stderr in Tree::new("pinned", &[("grund.toml", &config)]).refusals() {
        assert!(
            stderr.contains("[rules.citations.FS] must names `web/ZED`"),
            "{stderr}"
        );
        assert!(!stderr.contains(MD_AT_4), "{stderr}");
    }
}

/// In a workspace, a pinned target the expansion refuses wins over the six, whichever project
/// holds them.
#[test]
fn a_workspace_pinned_target_wins_over_the_six() {
    let member = "grund_config_version = 2\n\n[schema.kinds.API]\nfolders = [\"docs\"]\n";
    let root = "grund_config_version = 2\n\n[schema.kinds.FS]\nfolders = [\"docs\"]\n\n\
                [rules.citations.FS]\nmust = [\"api/NOPE\"]\n\n[workspace]\nmembers = [\"api\"]\n";
    let md = "[schema.sources.extensions]\nmd = \"json\"\n";
    let root_md = root.replacen("\n\n", &format!("\n\n{md}\n"), 1);
    let member_md = member.replacen("\n\n", &format!("\n\n{md}\n"), 1);
    let pinned = "grund.toml: [rules.citations.FS] must names an unknown target kind `NOPE` in \
                  namespace `api`; this became an error in grund 0.19.0";
    for (name, root, member) in [
        ("ws-root", root_md.as_str(), member),
        ("ws-member", root, member_md.as_str()),
    ] {
        Tree::new(name, &[("grund.toml", root), ("api/grund.toml", member)]).refused_for(pinned);
    }
}
