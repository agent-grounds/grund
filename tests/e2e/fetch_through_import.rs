//! Where the fetch of a qualified ID runs, and whether it can, has one answer, and `grund fetch`
//! and `grund check` both print it (§FS-imports.writes.answer, §FS-check.3.1.3, §FS-check.4.12).
//! Each scenario builds real import trees in the temp directory and, where a command is named,
//! follows it word for word: one command must be enough.
#![cfg(unix)]

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

const DASH: &str = "\u{2014}";
const VALUE_SOURCE: &str = "a value source holds no Markdown; move the kind to a Markdown file or \
                            a folder without it, or remove its `fetch`";

static NEXT: AtomicUsize = AtomicUsize::new(0);

/// One tree of projects under the temp directory, removed on drop.
struct Tree(PathBuf);

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

impl Tree {
    fn new() -> Self {
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("grund-1273-{}-{n}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create the tree");
        Self(root)
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().expect("a path below the tree")).expect("create the dir");
        fs::write(path, content).expect("write the file");
    }

    /// A version-2 project at `dir`, with `extra` after its name.
    fn project(&self, dir: &str, name: &str, extra: &str) {
        self.write(
            &format!("{dir}/grund.toml"),
            &format!("grund_config_version = 2\nproject_name = \"{name}\"\n\n{extra}\n"),
        );
    }

    /// A project whose `FS` kind fetches into `docs/fs.md` through an integration that succeeds.
    fn fetcher(&self, dir: &str, name: &str, extra: &str) {
        self.project(dir, name, &format!("{FS_FETCH}\n{extra}"));
        self.write(
            &format!("{dir}/docs/fs.md"),
            "# FS-unrelated-long-slug: u\n",
        );
        self.script(
            &format!("{dir}/scripts/fetch"),
            "#!/bin/sh\nprintf '## %s: Fetched\\n\\nBody.\\n' \"$1\"\n",
        );
    }

    fn script(&self, relative: &str, body: &str) {
        self.write(relative, body);
        let path = self.0.join(relative);
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("make it executable");
    }

    /// A project whose `CONST` kind fetches into a value source, a home `fetch` refuses.
    fn refusing(&self, dir: &str, name: &str) {
        self.project(
            dir,
            name,
            "[schema.kinds.CONST]\nfiles = [\"values/limits.json\"]\nform = \"value\"\n\
             fetch = \"scripts/fetch\"\n",
        );
        self.write(
            &format!("{dir}/values/limits.json"),
            "{\n  \"CONST-room-limit\": [40, \"rooms\"]\n}\n",
        );
        self.script(&format!("{dir}/scripts/fetch"), "#!/bin/sh\nexit 0\n");
    }

    fn consumer(&self, target: &str, citation: &str) {
        self.project("consumer", "consumer", &import("lb", target, "lib"));
        self.write("consumer/README.md", &format!("See §{citation}.\n"));
    }

    fn grund(&self, dir: &str, args: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(self.0.join(dir))
            .env("NO_COLOR", "1")
            .output()
            .expect("run grund");
        Run {
            code: output.status.code().expect("exited with a status"),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    /// The refusal `grund show --batch` reports for `id` in its one-item envelope.
    fn batch_refusal(&self, dir: &str, id: &str) -> String {
        let mut child = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(["show", "--batch", "--format=json"])
            .current_dir(self.0.join(dir))
            .env("NO_COLOR", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("run grund");
        let query = serde_json::json!({ "id": id }).to_string();
        let mut stdin = child.stdin.take().expect("a piped stdin");
        writeln!(stdin, "{query}").expect("write the query");
        drop(stdin);
        let output = child.wait_with_output().expect("wait for grund");
        assert_eq!(output.status.code(), Some(1), "a failed batch item exits 1");
        let line: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("one JSON envelope");
        line["error"]["message"]
            .as_str()
            .expect("a refusal")
            .to_owned()
    }

    /// Runs the `run grund fetch <id>[ in <dir>]` named in `line` from `from`: one try must do.
    fn follow(&self, from: &str, line: &str) {
        let (id, place) = named(line).unwrap_or_else(|| panic!("names no command: {line}"));
        let dir = match place {
            Some(place) => normal(&format!("{from}/{place}")),
            None => from.to_owned(),
        };
        let run = self.grund(&dir, &["fetch", &id]);
        assert_eq!(
            run.code, 0,
            "the command named by `{line}`, run in {dir}, was refused: {}",
            run.stderr
        );
    }

    /// The first line `grund check` in `dir` prints about `id`.
    fn finding(&self, dir: &str, id: &str) -> String {
        let run = self.grund(dir, &["check"]);
        run.stdout
            .lines()
            .chain(run.stderr.lines())
            .find(|line| line.contains(id))
            .unwrap_or_else(|| panic!("check says nothing of {id}: {}{}", run.stdout, run.stderr))
            .to_owned()
    }

    fn assert_resolved(&self, dir: &str, id: &str) {
        let run = self.grund(dir, &["check"]);
        let all = format!("{}{}", run.stdout, run.stderr);
        assert!(!all.contains(id), "{id} is still unresolved: {all}");
    }
}

const FS_FETCH: &str = "[schema.kinds.FS]\nfiles = [\"docs/fs.md\"]\nfetch = \"scripts/fetch\"\n";

fn import(alias: &str, path: &str, project: &str) -> String {
    format!("[workspace.imports.{alias}]\npath = \"{path}\"\nproject = \"{project}\"\n")
}

/// Collapses `a/b/../c`, since the named place is relative to the directory the run started in.
fn normal(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            ".." => {
                parts.pop();
            }
            "." | "" => {}
            part => parts.push(part),
        }
    }
    if parts.is_empty() {
        ".".to_owned()
    } else {
        parts.join("/")
    }
}

/// The ID and place of the first `run grund fetch <id>[ in <dir>]` in `line`.
fn named(line: &str) -> Option<(String, Option<String>)> {
    let tail = line.split("run grund fetch ").nth(1)?;
    let mut words = tail.split_whitespace();
    let id = words.next()?.to_owned();
    let place = match words.next() {
        Some("in") => words.next().map(str::to_owned),
        _ => None,
    };
    Some((id, place))
}

fn assert_no_command(text: &str) {
    assert!(!text.contains("run grund fetch"), "names a command: {text}");
    assert!(!text.contains("read-only"), "calls it read-only: {text}");
}

/// A workspace member's ID keeps the qualified command, which works from the root. Passes today:
/// it is the behaviour the answer must not move.
#[test]
fn a_members_id_keeps_the_qualified_command_from_the_root() {
    let tree = Tree::new();
    tree.project(
        "root",
        "root",
        "[workspace]\nmembers = [\"alpha\", \"beta\"]\ninclude_root = false\n",
    );
    tree.fetcher("root/alpha", "alpha", "");
    tree.project("root/beta", "beta", "");
    tree.write("root/beta/README.md", "See §alpha/FS-a.\n");

    let line = tree.finding("root", "alpha/FS-a");
    assert!(
        line.ends_with(&format!("{DASH} run grund fetch alpha/FS-a")),
        "{line}"
    );
    tree.follow("root", &line);
    tree.assert_resolved("root", "alpha/FS-a");
}

/// A fetch of a member's ID reads no import it does not name, so one that cannot be read does
/// not fail it. Passes today: a fix that loads every edge up front would break it.
#[test]
fn a_members_fetch_does_not_read_an_import_it_does_not_name() {
    let tree = Tree::new();
    tree.project(
        "root",
        "root",
        &format!(
            "[workspace]\nmembers = [\"alpha\"]\ninclude_root = false\n\n{}{}",
            import("gone", "../missing", "missing"),
            import("broken", "../broken", "broken"),
        ),
    );
    tree.write("broken/grund.toml", "this is = = not toml\n");
    tree.fetcher("root/alpha", "alpha", "");

    let run = tree.grund("root", &["fetch", "alpha/FS-a"]);
    assert_eq!(run.code, 0, "{}", run.stderr);
}

/// Near-ID hints keep their precedence over the tail through an import (§FS-check.3.1.3).
/// Passes today: the fix replaces the tail, and this holds the hint in front of it.
#[test]
fn a_near_id_hint_takes_the_tails_place_through_an_import() {
    let tree = Tree::new();
    tree.consumer("../lib", "lb/FS-unrelated-long-slux");
    tree.fetcher("lib", "lib", "");

    let line = tree.finding("consumer", "lb/FS-unrelated-long-slux");
    assert!(line.contains("did you mean"), "{line}");
    assert!(!line.contains("run grund fetch"), "{line}");
}

fn check_and_fetch_name_one_command(lib_extra: &str, finding: &str) {
    let tree = Tree::new();
    tree.consumer("../lib", "lb/FS-a");
    tree.fetcher("lib", "lib", lib_extra);

    let check = tree.finding("consumer", "lb/FS-a");
    assert_eq!(
        check,
        format!("{finding} {DASH} run grund fetch FS-a in ../lib")
    );
    let fetch = tree.grund("consumer", &["fetch", "lb/FS-a"]);
    assert_eq!(fetch.code, 2, "{}", fetch.stderr);
    assert!(
        fetch.stderr.contains(&format!(
            "`lb/FS-a` is in import `lb`, read-only here {DASH} run grund fetch FS-a in ../lib"
        )),
        "{}",
        fetch.stderr
    );
    tree.follow("consumer", &check);
    tree.assert_resolved("consumer", "lb/FS-a");
}

#[test]
fn check_and_fetch_name_one_command_for_a_dangling_citation() {
    check_and_fetch_name_one_command(
        "",
        "README.md:1: error: unknown reference lb/FS-a; no snapshot in ../lib/docs/fs.md",
    );
}

#[test]
fn check_and_fetch_name_one_command_for_a_missing_snapshot() {
    check_and_fetch_name_one_command(
        "\n[rules.resolution]\nFS = \"warn\"\n",
        "README.md:1: warning: no snapshot for lb/FS-a in ../lib/docs/fs.md",
    );
}

/// A nested import costs one command, in the last import's target (§FS-imports.graph.transitive).
fn nested(id: &str, below: &str, inner: impl FnOnce(&Tree)) {
    let tree = Tree::new();
    tree.consumer("../lib", id);
    tree.project("lib", "lib", &import("in", "../inner", "inner"));
    inner(&tree);

    let fetch = tree.grund("consumer", &["fetch", id]);
    assert_eq!(fetch.code, 2, "{}", fetch.stderr);
    let remedy = format!("run grund fetch {below} in ../inner");
    assert!(
        fetch.stderr.contains(&format!(
            "`{id}` is in import `lb/in`, read-only here {DASH} {remedy}"
        )),
        "{}",
        fetch.stderr
    );
    let check = tree.finding("consumer", id);
    assert!(check.ends_with(&format!("{DASH} {remedy}")), "{check}");
    tree.follow("consumer", &check);
    tree.assert_resolved("consumer", id);
}

#[test]
fn a_nested_import_takes_one_command() {
    nested("lb/in/FS-a", "FS-a", |tree| {
        tree.fetcher("inner", "inner", "");
    });
}

#[test]
fn a_member_below_a_nested_import_takes_one_command() {
    nested("lb/in/alpha/FS-a", "alpha/FS-a", |tree| {
        tree.project(
            "inner",
            "inner",
            "[workspace]\nmembers = [\"alpha\"]\ninclude_root = false\n",
        );
        tree.fetcher("inner/alpha", "alpha", "");
    });
}

/// An import a member declares is reached from the root, under the member's alias.
#[test]
fn an_import_a_member_declares_takes_one_command_from_the_root() {
    let tree = Tree::new();
    tree.project(
        "root",
        "root",
        "[workspace]\nmembers = [\"alpha\"]\ninclude_root = false\n",
    );
    tree.fetcher("root/alpha", "alpha", &import("lb", "../../lib", "lib"));
    tree.write("root/alpha/README.md", "See §alpha/lb/FS-a.\n");
    tree.fetcher("lib", "lib", "");

    let check = tree.finding("root", "alpha/lb/FS-a");
    assert!(
        check.ends_with(&format!("{DASH} run grund fetch FS-a in ../lib")),
        "{check}"
    );
    let fetch = tree.grund("root", &["fetch", "alpha/lb/FS-a"]);
    assert_eq!(fetch.code, 2, "{}", fetch.stderr);
    assert!(
        fetch.stderr.contains("run grund fetch FS-a in ../lib"),
        "{}",
        fetch.stderr
    );
    tree.follow("root", &check);
    tree.assert_resolved("root", "alpha/lb/FS-a");
}

/// A home `fetch` refuses gets its own remedy and no command, from fetch and from check.
fn refused(id: &str, home: &str, setup: impl FnOnce(&Tree)) {
    let tree = Tree::new();
    tree.consumer("../lib", id);
    setup(&tree);

    let fetch = tree.grund("consumer", &["fetch", id]);
    assert_eq!(fetch.code, 2, "{}", fetch.stderr);
    assert_no_command(&fetch.stderr);
    assert!(
        fetch.stderr.contains(&format!(
            "kind `CONST` cannot fetch into {home}: {VALUE_SOURCE}"
        )),
        "{}",
        fetch.stderr
    );
    let check = tree.finding("consumer", id);
    assert!(
        check.ends_with(&format!("{DASH} declare it in that value source")),
        "{check}"
    );
}

#[test]
fn a_refused_home_through_an_import_names_no_command() {
    refused("lb/CONST-a", "../lib/values/limits.json", |tree| {
        tree.refusing("lib", "lib");
    });
}

#[test]
fn a_refused_home_through_a_nested_import_names_no_command() {
    refused("lb/in/CONST-a", "../inner/values/limits.json", |tree| {
        tree.project("lib", "lib", &import("in", "../inner", "inner"));
        tree.refusing("inner", "inner");
    });
}

#[test]
fn a_kind_with_no_fetch_through_an_import_names_no_command() {
    let tree = Tree::new();
    tree.consumer("../lib", "lb/FS-a");
    tree.project(
        "lib",
        "lib",
        "[schema.kinds.FS]\nfiles = [\"docs/fs.md\"]\n",
    );
    tree.write("lib/docs/fs.md", "# FS-other-thing: o\n");

    let fetch = tree.grund("consumer", &["fetch", "lb/FS-a"]);
    assert_eq!(fetch.code, 2, "{}", fetch.stderr);
    assert_no_command(&fetch.stderr);
    assert!(
        fetch.stderr.contains("kind `FS` has no fetch integration"),
        "{}",
        fetch.stderr
    );
}

#[test]
fn a_kind_the_target_lacks_is_what_the_target_says() {
    let tree = Tree::new();
    tree.consumer("../lib", "lb/FS-a");
    tree.fetcher("lib", "lib", "");

    let fetch = tree.grund("consumer", &["fetch", "lb/BOGUS-a"]);
    assert_eq!(fetch.code, 1, "{}", fetch.stderr);
    assert_no_command(&fetch.stderr);
    assert!(
        fetch.stderr.contains("invalid ID `BOGUS-a`"),
        "{}",
        fetch.stderr
    );
}

/// An unknown alias or a missing import is refused in the words `show` prints, naming no command,
/// by `fetch`, `refs` and each item of `show --batch` alike.
fn in_shows_words(tree: &Tree, id: &str, words: &str) {
    let show = tree.grund("consumer", &["show", id]);
    assert_eq!(show.code, 2, "{}", show.stderr);
    assert!(show.stderr.contains(words), "{}", show.stderr);
    let fetch = tree.grund("consumer", &["fetch", id]);
    assert_eq!(fetch.code, 2, "{}", fetch.stderr);
    assert_no_command(&fetch.stderr);
    assert_eq!(fetch.stderr, show.stderr);
    let refs = tree.grund("consumer", &["refs", id]);
    assert_eq!(refs.code, 2, "{}", refs.stderr);
    assert_eq!(refs.stderr, show.stderr);
    let refusal = show.stderr.strip_prefix("error: ").expect("an error line");
    assert_eq!(tree.batch_refusal("consumer", id), refusal.trim_end());
}

#[test]
fn an_unknown_alias_below_an_import_is_unknown() {
    let tree = Tree::new();
    tree.consumer("../lib", "lb/FS-a");
    tree.project("lib", "lib", "");
    in_shows_words(&tree, "lb/nope/FS-a", "unknown project alias `lb/nope`");
}

#[test]
fn an_unknown_alias_below_a_nested_import_is_unknown() {
    let tree = Tree::new();
    tree.consumer("../lib", "lb/FS-a");
    tree.project("lib", "lib", &import("in", "../inner", "inner"));
    tree.project("inner", "inner", "");
    in_shows_words(
        &tree,
        "lb/in/nope/FS-a",
        "unknown project alias `lb/in/nope`",
    );
}

#[test]
fn a_missing_import_is_named_missing() {
    let tree = Tree::new();
    tree.consumer("../gone", "lb/FS-a");
    in_shows_words(&tree, "lb/FS-a", "import `lb` is missing");
}

#[test]
fn a_missing_nested_import_is_named_missing() {
    let tree = Tree::new();
    tree.consumer("../lib", "lb/FS-a");
    tree.project("lib", "lib", &import("in", "../gone", "gone"));
    in_shows_words(&tree, "lb/in/FS-a", "import `lb/in` is missing");
}
