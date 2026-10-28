//! Binary regressions for agent-grounds/grund#1079: a run that loaded no config,
//! over a tree holding a project config below its root, names that config. The
//! empty-scan warning stops advising `grund init --scaffold` (§FS-check.2.2.2.5),
//! and the invalid-ID hint stops naming a default format no project there uses
//! (§FS-show.3.5.1).
//!
//! These cases live outside the `tests/e2e/cases` corpus because every case there
//! sits inside this repository, whose own grund.toml would be discovered: the
//! condition under test is a tree with no config at or above it. Each fixture is
//! built under the system temp directory, so run with a `TMPDIR` that no
//! grund.toml encloses. `Tree::new` refuses one that does, naming the config, so a
//! failure here is about grund and never about where the fixture landed.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

/// The report's own project: version 2, kind FS in `specs/`, no index.
const SUITE_CONFIG: &str = "grund_config_version = 2\nproject_name = \"nested-suite\"\n\n\
                            [schema.kinds.FS]\nfolders = [\"specs\"]\nindex = false\n";

/// The report's one spec file, citing an ID it never declares, so a run that
/// does read the project exits 1 on it.
const LOGIN: &str = "# FS-login: users log in\n\nSee \u{a7}FS-logout.\n";

/// §FS-check.2.2.2.1's message, byte for byte as it reads today: a run with no
/// config and nothing below keeps it.
const ROOT_WARNING: &str = "warning: nothing to scan — grund looked under [scan] include = \
    [\"requirements.md\", \"docs\", \"e2e\", \"src\"] and found no files. Run `grund init --scaffold` \
    to scaffold the canonical docs/ and e2e/ trees, point `[scan] include` in `grund.toml` at your \
    sources, or pass a path explicitly (`grund check <dir>`).\n";

/// §FS-output-shapes.6.1.2's refusal, for the report's ID: kept where nothing lies below.
const DEFAULT_FORMAT_REFUSAL: &str = "invalid ID `FS-login`\nhint: this repo's [id] format is \
    `{kind}-{number}-{slug}` (run `grund config show`); `grund list` shows the IDs that exist\n";

/// §FS-output-shapes.6.1.2: the JSON refusal carries no hint, whatever lies below.
const JSON_REFUSAL: &str = "{\"severity\":\"error\",\"path\":null,\"line\":null,\"code\":\"invalid-id\",\
    \"message\":\"invalid ID `FS-login`\",\"sites\":null,\"authority\":null}\n";

/// §FS-check.2.2.2.5: the warning, from the names and the routes it offers.
fn warning(names: &str, routes: &str) -> String {
    format!(
        "warning: nothing to scan — no grund.toml applies here, so grund used its defaults and \
         found no files. {names}: {routes}.\n"
    )
}

/// §FS-show.3.5.1: the refusal of `FS-login`, whose hint names the configs below.
fn refusal(names: &str, routes: &str) -> String {
    format!(
        "invalid ID `FS-login`\nhint: no grund.toml applies here, so grund read `FS-login` against \
         its defaults. {names}: {routes}\n"
    )
}

/// The one-config sentences of §FS-check.2.2.2.5, for a config whose directory is `dir`.
fn one(config: &str) -> String {
    format!("`{config}` declares a project this run does not read")
}

fn check_routes_one(dir: &str) -> String {
    format!(
        "run grund from `{dir}/` (or `grund check {dir}`), or add a grund.toml here whose \
         [workspace] has members = [\"{dir}\"] and include_root = false"
    )
}

fn query_routes_one(dir: &str) -> String {
    format!(
        "run grund from `{dir}/`, or add a grund.toml here whose [workspace] has \
         members = [\"{dir}\"] and include_root = false"
    )
}

/// A tree whose root no config applies to: a `.git` marker, nothing else until a
/// case lays its projects.
struct Tree(PathBuf);

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Tree {
    fn new(what: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-1079-{what}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join(".git")).unwrap();
        let root = root.canonicalize().unwrap();
        if let Some(config) = enclosing_config(&root) {
            panic!(
                "the fixture root {} is enclosed by {}, so the run would load it: rerun with a \
                 TMPDIR no grund.toml encloses, such as TMPDIR=/var/tmp/grund-1079-tmp",
                root.display(),
                config.display()
            );
        }
        Self(root)
    }

    fn write(&self, path: &str, text: &str) -> &Self {
        let file = self.0.join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, text).unwrap();
        self
    }

    /// The report's project at `config` (either discovery form), with its spec
    /// file in `specs/` under the project's own directory.
    fn project(&self, config: &str) -> &Self {
        let dir = Path::new(config).parent().unwrap();
        let dir = if dir.ends_with(".agents") {
            dir.parent().unwrap()
        } else {
            dir
        };
        self.write(config, SUITE_CONFIG)
            .write(&format!("{}/specs/FS-login.md", dir.display()), LOGIN)
    }

    fn run_in(&self, dir: &str, args: &[&str]) -> Run {
        let out = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(self.0.join(dir))
            .output()
            .expect("run built grund");
        Run {
            code: out.status.code(),
            stdout: String::from_utf8(out.stdout).unwrap(),
            stderr: String::from_utf8(out.stderr).unwrap(),
        }
    }

    fn run(&self, args: &[&str]) -> Run {
        self.run_in(".", args)
    }

    /// `grund check` and `grund check .` from the root: exit 0, nothing on
    /// stdout, and exactly `expected` on stderr (§FS-check.2.2.3).
    fn assert_check(&self, expected: &str) {
        for args in [&["check"][..], &["check", "."]] {
            let run = self.run(args);
            assert!(
                expected.contains("grund init --scaffold")
                    || !run.stderr.contains("grund init --scaffold"),
                "grund {args:?} offers the scaffold above a project it did not read:\n{}",
                run.stderr
            );
            assert_eq!(run.stderr, expected, "grund {args:?}: stderr");
            assert_eq!(run.code, Some(0), "grund {args:?}: exit");
            assert_eq!(run.stdout, "", "grund {args:?}: stdout");
        }
    }

    /// The bare, `show` and `refs` forms of `FS-login`: exit 1, nothing on stdout,
    /// and exactly `expected` on stderr (§FS-show.3.5.1).
    fn assert_refusal(&self, expected: &str) {
        for args in [
            &["FS-login"][..],
            &["show", "FS-login"],
            &["refs", "FS-login"],
        ] {
            let run = self.run(args);
            assert!(
                expected.contains("{kind}-{number}-{slug}")
                    || !run.stderr.contains("{kind}-{number}-{slug}"),
                "grund {args:?} names a default format no project here uses:\n{}",
                run.stderr
            );
            assert_eq!(run.stderr, expected, "grund {args:?}: stderr");
            assert_eq!(run.code, Some(1), "grund {args:?}: exit");
            assert_eq!(run.stdout, "", "grund {args:?}: stdout");
        }
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The config discovery would load for `dir`, at it or above (§FS-config.1).
fn enclosing_config(dir: &Path) -> Option<PathBuf> {
    dir.ancestors()
        .flat_map(|dir| [dir.join("grund.toml"), dir.join(".agents/grund.toml")])
        .find(|config| config.is_file())
}

/// The fixture is the situation the report describes: from its own directory the
/// project is read, and the citation it carries is reported.
#[test]
fn the_project_is_read_from_its_own_directory() {
    let tree = Tree::new("control");
    tree.project("suite/grund.toml");
    let run = tree.run_in("suite", &["check"]);
    assert!(
        run.stdout
            .contains("specs/FS-login.md:3: error: unknown reference FS-logout"),
        "{}",
        run.stdout
    );
    assert_eq!(run.code, Some(1));
}

/// §FS-check.2.2.2.5: the report's tree, run from its root.
#[test]
fn check_names_the_config_below() {
    let tree = Tree::new("check");
    tree.project("suite/grund.toml");
    tree.assert_check(&warning(
        &one("suite/grund.toml"),
        &check_routes_one("suite"),
    ));
}

/// §FS-show.3.5.1: the report's lookup, in the bare, `show` and `refs` forms.
#[test]
fn the_refusal_names_the_config_below() {
    let tree = Tree::new("query");
    tree.project("suite/grund.toml");
    tree.assert_refusal(&refusal(
        &one("suite/grund.toml"),
        &query_routes_one("suite"),
    ));
}

/// §FS-check.2.2.3: the JSON warning keeps its code and shape, and carries the
/// text message.
#[test]
fn json_warning_carries_the_new_message() {
    let tree = Tree::new("json-check");
    tree.project("suite/grund.toml");
    let run = tree.run(&["check", "--format", "json"]);
    let text = warning(&one("suite/grund.toml"), &check_routes_one("suite"));
    let message = text.strip_prefix("warning: ").unwrap().trim_end();
    let object: serde_json::Value = serde_json::from_str(run.stderr.trim_end()).unwrap();
    assert_eq!(object["message"], message, "{}", run.stderr);
    assert_eq!(object["code"], "empty-scan");
    assert_eq!(object["severity"], "warning");
    assert!(object["path"].is_null() && object["line"].is_null() && object["sites"].is_null());
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stdout, "");
}

/// §FS-show.3.5.1, §FS-output-shapes.6.1.2: JSON carries no hint, so the
/// refusal there is the same object whatever lies below.
#[test]
fn json_refusal_stays_hint_free() {
    let tree = Tree::new("json-query");
    tree.project("suite/grund.toml");
    let run = tree.run(&["FS-login", "--format", "json"]);
    assert_eq!(run.stderr, JSON_REFUSAL);
    assert_eq!(run.code, Some(1));
    assert_eq!(run.stdout, "");
}

/// §FS-check.2.2.2.5.1: `.agents/grund.toml` is named as itself, and its
/// directory in the routes is the one holding `.agents/`.
#[test]
fn the_agents_form_is_named_as_itself() {
    let tree = Tree::new("agents");
    tree.project("suite/.agents/grund.toml");
    let names = one("suite/.agents/grund.toml");
    tree.assert_check(&warning(&names, &check_routes_one("suite")));
    tree.assert_refusal(&refusal(&names, &query_routes_one("suite")));
}

/// §FS-check.2.2.2.5.1: two configs, named in path order rather than the order
/// they were laid, with every directory in `members`.
#[test]
fn two_configs_are_named_in_path_order() {
    let tree = Tree::new("two");
    tree.project("suite/grund.toml").project("other/grund.toml");
    let names = "`other/grund.toml` and `suite/grund.toml` declare projects this run does not read";
    let members = "members = [\"other\", \"suite\"] and include_root = false";
    tree.assert_check(&warning(
        names,
        &format!(
            "run grund from one of their directories (or `grund check other`), or add a grund.toml \
             here whose [workspace] has {members}"
        ),
    ));
    tree.assert_refusal(&refusal(
        names,
        &format!(
            "run grund from one of their directories, or add a grund.toml here whose [workspace] \
             has {members}"
        ),
    ));
}

/// §FS-check.2.2.2.5.1: past three, the rest are counted and `members` is not
/// spelled out, since three entries would leave the fourth unread.
#[test]
fn past_three_the_rest_are_counted() {
    let tree = Tree::new("four");
    for dir in ["d", "c", "b", "a"] {
        tree.project(&format!("{dir}/grund.toml"));
    }
    let names = "`a/grund.toml`, `b/grund.toml`, `c/grund.toml` and 1 more declare projects this \
                 run does not read";
    let workspace = "add a grund.toml here whose [workspace] lists their directories in members \
                     and has include_root = false";
    tree.assert_check(&warning(
        names,
        &format!("run grund from one of their directories (or `grund check a`), or {workspace}"),
    ));
    tree.assert_refusal(&refusal(
        names,
        &format!("run grund from one of their directories, or {workspace}"),
    ));
}

/// §FS-check.2.2.2.5.2: three levels below the root is inside the search.
#[test]
fn a_config_three_levels_down_is_found() {
    let tree = Tree::new("deep");
    tree.project("a/b/c/grund.toml");
    tree.assert_check(&warning(
        &one("a/b/c/grund.toml"),
        &check_routes_one("a/b/c"),
    ));
}

/// §FS-check.2.2.2.5.2: the search stops at a config it found, so the project
/// below it is that project's own and is not named beside it.
#[test]
fn a_config_below_a_found_one_is_not_named() {
    let tree = Tree::new("inner");
    tree.project("suite/grund.toml")
        .project("suite/inner/grund.toml");
    tree.assert_check(&warning(
        &one("suite/grund.toml"),
        &check_routes_one("suite"),
    ));
}

/// §FS-check.2.2.2.1, §FS-output-shapes.6.1.2: with nothing below, both
/// messages keep today's bytes.
#[test]
fn nothing_below_keeps_both_messages() {
    let tree = Tree::new("nothing");
    tree.assert_check(ROOT_WARNING);
    tree.assert_refusal(DEFAULT_FORMAT_REFUSAL);
}

/// §FS-check.2.2.2.5.2: a config the scan would not reach is not named, and the
/// run keeps today's messages: under the defaults' `[scan] exclude`, under a
/// hidden directory, under an ignored one, and four levels down.
#[test]
fn a_config_the_search_does_not_reach_keeps_both_messages() {
    let tree = Tree::new("pruned");
    tree.project("target/x/grund.toml")
        .project(".hidden/grund.toml")
        .project("ignored/grund.toml")
        .write(".gitignore", "ignored/\n")
        .project("a/b/c/d/grund.toml");
    tree.assert_check(ROOT_WARNING);
    tree.assert_refusal(DEFAULT_FORMAT_REFUSAL);
}
