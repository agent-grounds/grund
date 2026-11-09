//! A `fetch` key that names no executable, under both config versions (§FS-fetch.2.1): version 2
//! warns at the key's line through grund 0.22.x (§FS-config-v2.checks.config-fetch-names-nothing),
//! version 1 loads it unchanged (§FS-config.3.4.10.2), `grund fetch` refuses it before any spawn,
//! and `config migrate` refuses it with the repair (§FS-config-migrate.3.1.10). Each tree is built
//! in the temp directory, because a file named `" "` cannot be a committed case fixture on every
//! platform the suite runs on.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const EMPTY: &str = "";
const BLANK: &str = " ";
const V2_REPAIR: &str = "name the program, or if this kind is not fetched remove the key and any \
                         `[rules.resolution] AR`, and its missing IDs are then errors";
const V1_REPAIR: &str = "name the program, or if this kind is not fetched remove the key and any \
                         `resolve`, and its missing IDs are then errors";

#[derive(Clone, Copy, PartialEq)]
enum Version {
    One,
    Two,
}

use Version::{One, Two};

impl Version {
    fn id(self) -> &'static str {
        match self {
            One => "AR-001-power",
            Two => "AR-power",
        }
    }

    fn absent(self) -> &'static str {
        match self {
            One => "AR-002-other",
            Two => "AR-other",
        }
    }

    /// The key's name in a message, and the repair that goes with it (§FS-fetch.2.1).
    fn key(self) -> &'static str {
        match self {
            One => "[[kinds]] `AR` fetch",
            Two => "`[schema.kinds.AR] fetch`",
        }
    }

    fn repair(self) -> &'static str {
        match self {
            One => V1_REPAIR,
            Two => V2_REPAIR,
        }
    }

    /// The refusal `grund fetch` prints for a key at line 6 (§FS-fetch.2.1).
    fn fetch_refusal(self, value: &str) -> String {
        format!(
            "error: cannot run fetch integration `{value}` for {}: grund.toml:6: {} names no \
             executable; {}\n",
            self.id(),
            self.key(),
            self.repair()
        )
    }
}

const WINDOW: &str = "this warning becomes an error in grund 0.23.0";

/// The version 2 warning for a key at line 6 (§FS-config-v2.checks.config-fetch-names-nothing).
fn warning() -> String {
    format!(
        "warning: grund.toml:6: `[schema.kinds.AR] fetch` names no executable, read for now as a \
         kind whose fetch cannot run; {V2_REPAIR}; {WINDOW}"
    )
}

/// One project tree under the temp directory, removed on drop.
struct Tree(PathBuf);

impl Tree {
    /// The report's tree with `fetch` at line 6 when `value` is `Some`, a working fetcher on disk
    /// whichever value the key holds, and `settings` appended to the row's table.
    fn new(name: &str, version: Version, value: Option<&str>, settings: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1233-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs")).expect("create docs");
        fs::create_dir_all(root.join("scripts")).expect("create scripts");
        let key = value.map_or(String::from("# no fetch key\n"), |value| {
            format!("fetch = \"{value}\"\n")
        });
        let config = match version {
            One => format!(
                "project_name = \"g233\"\n\n[[kinds]]\nkind = \"AR\"\nfile = \"docs/arch.md\"\n\
                 {key}{settings}\n[id]\nnamed_sections = true\n\n[scan]\ninclude = [\"docs\"]\n"
            ),
            Two => format!(
                "grund_config_version = 2\nproject_name = \"g233\"\n\n[schema.kinds.AR]\n\
                 files = [\"docs/arch.md\"]\n{key}{settings}"
            ),
        };
        fs::write(root.join("grund.toml"), config).expect("write the config");
        fs::write(
            root.join("docs/arch.md"),
            format!(
                "# Architecture\n\n## {}: Power\n\nHand-written.\n",
                version.id()
            ),
        )
        .expect("write the home");
        let tree = Self(root);
        tree.write_fetcher("scripts/fetch");
        tree
    }

    /// An executable that prints a declaration of the ID it is given.
    fn write_fetcher(&self, relative: &str) {
        let path = self.0.join(relative);
        fs::write(
            &path,
            "#!/bin/sh\nprintf '## %s: Power\\n\\nFetched body.\\n' \"$1\"\n",
        )
        .expect("write the fetcher");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("set the mode");
        }
    }

    /// Adds a citation of an ID the home does not hold, so the kind has a missing snapshot.
    fn cite_absent(&self, version: Version) {
        let path = self.0.join("docs/arch.md");
        let text = fs::read_to_string(&path).expect("read the home");
        fs::write(path, format!("{text}\nSee §{}.\n", version.absent())).expect("write the home");
    }

    fn run(&self, args: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .env("NO_COLOR", "1")
            .output()
            .expect("run grund");
        Run {
            code: output.status.code().expect("exited with a status"),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    fn snapshot(&self) -> BTreeMap<PathBuf, Vec<u8>> {
        fn walk(dir: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
            for entry in fs::read_dir(dir).expect("read a directory") {
                let path = entry.expect("read an entry").path();
                if path.is_dir() {
                    walk(&path, out);
                } else {
                    out.insert(path.clone(), fs::read(&path).expect("read a file"));
                }
            }
        }
        let mut out = BTreeMap::new();
        walk(&self.0, &mut out);
        out
    }
}

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

impl Run {
    fn shown(&self) -> String {
        format!(
            "exit {}\nstdout:\n{}\nstderr:\n{}",
            self.code, self.stdout, self.stderr
        )
    }

    fn has_line(&self, line: &str) -> usize {
        self.stderr.lines().filter(|seen| *seen == line).count()
    }
}

fn each(mut body: impl FnMut(Version, &str, &str)) {
    for (version, tag) in [(One, "v1"), (Two, "v2")] {
        for (value, spelled) in [(EMPTY, "empty"), (BLANK, "blank")] {
            body(version, value, &format!("{tag}-{spelled}"));
        }
    }
}

/// §FS-config-v2.checks.config-fetch-names-nothing: one warning at the key's line, exit 0.
#[test]
fn v2_check_warns_at_the_key_line() {
    for value in [EMPTY, BLANK] {
        let tree = Tree::new(&format!("check-{}", value.len()), Two, Some(value), "");
        let run = tree.run(&["check"]);
        assert_eq!(run.code, 0, "{}", run.shown());
        assert_eq!(run.has_line(&warning()), 1, "{}", run.shown());
    }
}

/// §FS-config.3.4.10.2: a version 1 row keeps loading, with no new finding, and
/// §FS-config-v2.checks.config-fetch-names-nothing never fires for version 1.
#[test]
fn v1_check_is_unchanged() {
    for value in [EMPTY, BLANK] {
        let tree = Tree::new(&format!("check-v1-{}", value.len()), One, Some(value), "");
        let run = tree.run(&["check"]);
        assert_eq!(run.code, 0, "{}", run.shown());
        assert!(
            !run.stderr.contains("names no executable"),
            "{}",
            run.shown()
        );
    }
}

/// §FS-fetch.2.1: exit 2, the old prefix and the key's line, with nothing spawned or written.
#[test]
fn fetch_refuses_before_any_spawn() {
    each(|version, value, name| {
        let tree = Tree::new(&format!("fetch-{name}"), version, Some(value), "");
        let before = tree.snapshot();
        let run = tree.run(&["fetch", version.id()]);
        assert_eq!(run.code, 2, "{}", run.shown());
        assert_eq!(run.stderr, version.fetch_refusal(value), "{}", run.shown());
        assert_eq!(tree.snapshot(), before, "the tree must stay byte-identical");
    });
}

/// §FS-config-migrate.3.1.10: refused at the key's line with the repair, the file untouched.
#[test]
fn migrate_refuses_the_key() {
    for value in [EMPTY, BLANK] {
        let tree = Tree::new(&format!("migrate-{}", value.len()), One, Some(value), "");
        let before = tree.snapshot();
        let run = tree.run(&["config", "migrate", "--to", "2", "--write", "."]);
        assert_eq!(run.code, 1, "{}", run.shown());
        let block = format!(
            "refused: grund.toml:6: [[kinds]] `AR` fetch names no executable\n  repair: under \
             version 1, {V1_REPAIR}; grund check shows its effect\n"
        );
        assert!(run.stderr.contains(&block), "{}", run.shown());
        assert_eq!(tree.snapshot(), before, "a refusal writes nothing");
    }
}

/// §FS-config-migrate.3.1.10: one block per key, ordered by line, 3.1.7.1's words unchanged.
#[test]
fn migrate_orders_the_fetch_block_and_the_value_chapter_block_by_line() {
    const CHAPTER: &str = "refused: grund.toml:7: [[kinds]] `AR` value_chapter has no version 2 \
                           spelling on a fetched kind";
    let tree = Tree::new("chapter", One, Some(EMPTY), "value_chapter = \"values\"\n");
    let run = tree.run(&["config", "migrate", "--to", "2", "--write", "."]);
    assert_eq!(run.code, 1, "{}", run.shown());
    let fetch = run
        .stderr
        .find("refused: grund.toml:6:")
        .expect(&run.shown());
    let chapter = run.stderr.find(CHAPTER).expect(&run.shown());
    assert!(fetch < chapter, "{}", run.shown());
    assert!(
        run.stderr.contains("not written: 2 refusals"),
        "{}",
        run.shown()
    );
}

/// The tree of a blank key that a resolution setting leans on (§FS-fetch.2.1).
fn leaning_tree(name: &str, version: Version, value: Option<&str>, setting: bool) -> Tree {
    let settings = match (version, setting) {
        (One, true) => "resolve = \"should\"\n",
        (Two, true) => "\n[rules.resolution]\nAR = \"warn\"\n",
        (_, false) => "",
    };
    let tree = Tree::new(name, version, value, settings);
    tree.cite_absent(version);
    tree
}

/// §FS-config-v2.checks.config-fetch-names-nothing: the key still lets the setting load.
#[test]
#[ignore = "red until #1233 the v2 reader warns; implement removes this"]
fn leaning_tree_warns_under_version_2_and_repairs_truthfully() {
    let tree = leaning_tree("lean-v2", Two, Some(EMPTY), true);
    let run = tree.run(&["check"]);
    assert_eq!(run.code, 0, "{}", run.shown());
    assert_eq!(run.has_line(&warning()), 1, "{}", run.shown());
    assert!(
        run.stderr.contains("run grund fetch AR-other"),
        "{}",
        run.shown()
    );
}

/// §FS-fetch.2.1: the repair names the setting, under both versions, at `grund fetch`.
#[test]
fn leaning_tree_fetch_refusal_carries_the_repair() {
    for version in [One, Two] {
        let name = format!("lean-fetch-{}", version.id());
        let tree = leaning_tree(&name, version, Some(EMPTY), true);
        let run = tree.run(&["fetch", version.absent()]);
        assert_eq!(run.code, 2, "{}", run.shown());
        assert!(run.stderr.contains(version.repair()), "{}", run.shown());
    }
}

/// §FS-config-migrate.3.1.10: a version 1 `resolve` row is refused with the same repair.
#[test]
fn leaning_tree_migrate_refusal_names_resolve() {
    let tree = leaning_tree("lean-migrate", One, Some(EMPTY), true);
    let run = tree.run(&["config", "migrate", "--to", "2", "--write", "."]);
    assert_eq!(run.code, 1, "{}", run.shown());
    assert!(run.stderr.contains(V1_REPAIR), "{}", run.shown());
}

/// §FS-fetch.2.1: why the repair says what it says — the three-way behaviour of the tree.
#[test]
fn leaning_tree_guard_the_repair_is_true_of_it() {
    for version in [One, Two] {
        let name = format!("guard-{}", version.id());
        let held = leaning_tree(&name, version, Some(EMPTY), true).run(&["check"]);
        assert_eq!(
            held.code,
            0,
            "the blank key lets the setting load\n{}",
            held.shown()
        );
        let name = format!("guard-key-{}", version.id());
        let key_gone = leaning_tree(&name, version, None, true).run(&["check"]);
        assert_eq!(
            key_gone.code,
            2,
            "the setting needs the key\n{}",
            key_gone.shown()
        );
        let name = format!("guard-both-{}", version.id());
        let both_gone = leaning_tree(&name, version, None, false).run(&["check"]);
        assert_eq!(
            both_gone.code,
            1,
            "missing IDs are then errors\n{}",
            both_gone.shown()
        );
    }
}

/// §FS-config-v2.checks.config-fetch-names-nothing: the surfaces and selectors of the window.
#[test]
#[ignore = "red until #1233 the v2 reader warns; implement removes this"]
fn the_warning_reaches_its_surfaces_and_selectors() {
    let tree = Tree::new("surfaces", Two, Some(EMPTY), "");
    for args in [&["config", "validate"][..], &["config", "show"]] {
        let run = tree.run(args);
        assert_eq!(run.code, 0, "{args:?}\n{}", run.shown());
        assert_eq!(run.has_line(&warning()), 1, "{args:?}\n{}", run.shown());
    }
    let shown = tree.run(&["config", "show"]);
    assert!(shown.stdout.contains("fetch = \"\"\n"), "{}", shown.shown());
    let ignored = tree.run(&["check", "--ignore", "config-fetch-names-nothing"]);
    assert_eq!(ignored.has_line(&warning()), 0, "{}", ignored.shown());
    let only = tree.run(&["check", "--only", "config-fetch-names-nothing"]);
    assert_eq!(only.code, 0, "{}", only.shown());
    assert_eq!(only.has_line(&warning()), 1, "{}", only.shown());
    let json = tree.run(&["check", "--format=json"]);
    for field in [
        "\"code\":\"config-fetch-names-nothing\"",
        "\"path\":\"grund.toml\"",
        "\"line\":6",
    ] {
        assert!(
            json.stdout.replace(": ", ":").contains(field),
            "{field}\n{}",
            json.shown()
        );
    }
}

/// §FS-distribution.4.2.1: the pending half — the warning names 0.23.0, and this tree is below it.
#[test]
fn the_window_is_pending_below_0_23_0() {
    let parts: Vec<u64> = env!("CARGO_PKG_VERSION")
        .split(['.', '-'])
        .take(3)
        .map(|part| part.parse().expect("a numeric version"))
        .collect();
    assert!(
        (parts[0], parts[1]) < (0, 23),
        "grund {} has reached the release the warning names: land the refusal \
         (§FS-config-v2.checks.config-fetch-names-nothing)",
        env!("CARGO_PKG_VERSION")
    );
    let tree = Tree::new("pending", Two, Some(EMPTY), "");
    let run = tree.run(&["check"]);
    assert!(run.stderr.contains(WINDOW), "{}", run.shown());
}

/// §FS-fetch.2: a named program still fetches, under both versions.
#[test]
#[cfg(unix)]
fn control_a_named_program_still_fetches() {
    for version in [One, Two] {
        let name = format!("control-{}", version.id());
        let tree = Tree::new(&name, version, Some("scripts/fetch"), "");
        let run = tree.run(&["fetch", version.id()]);
        assert_eq!(run.code, 0, "{}", run.shown());
        let home = fs::read_to_string(tree.0.join("docs/arch.md")).expect("read the home");
        assert!(home.contains("Fetched body."), "{home}");
    }
}

/// §FS-config-migrate.3.1.10: a value with any other character is mapped as written.
#[test]
fn control_a_named_program_still_migrates() {
    let tree = Tree::new("migrate-named", One, Some(" scripts/fetch "), "");
    let run = tree.run(&["config", "migrate", "--to", "2", "--write", "."]);
    assert_eq!(run.code, 0, "{}", run.shown());
    let config = fs::read_to_string(tree.0.join("grund.toml")).expect("read the config");
    assert!(config.contains("fetch = \" scripts/fetch \"\n"), "{config}");
}

/// §FS-fetch.2.1: a real name with stray spaces is taken literally and fails as a spawn failure.
#[test]
fn control_a_name_with_stray_spaces_is_still_a_spawn_failure() {
    for version in [One, Two] {
        let name = format!("spaces-{}", version.id());
        let tree = Tree::new(&name, version, Some(" scripts/fetch "), "");
        let check = tree.run(&["check"]);
        assert_eq!(check.code, 0, "{}", check.shown());
        assert!(
            !check.stderr.contains("names no executable"),
            "{}",
            check.shown()
        );
        let run = tree.run(&["fetch", version.id()]);
        assert_eq!(run.code, 2, "{}", run.shown());
        assert!(
            run.stderr
                .contains("cannot run fetch integration ` scripts/fetch `"),
            "{}",
            run.shown()
        );
        assert!(
            !run.stderr.contains("names no executable"),
            "{}",
            run.shown()
        );
    }
}

/// §FS-fetch.2.1: a file literally named `" "` still runs, under version 1 and the v2 window.
#[test]
#[cfg(unix)]
fn control_a_file_named_blank_still_runs() {
    for version in [One, Two] {
        let name = format!("named-blank-{}", version.id());
        let tree = Tree::new(&name, version, Some(BLANK), "");
        tree.write_fetcher(BLANK);
        let run = tree.run(&["fetch", version.id()]);
        assert_eq!(run.code, 0, "{}", run.shown());
        let home = fs::read_to_string(tree.0.join("docs/arch.md")).expect("read the home");
        assert!(home.contains("Fetched body."), "{home}");
    }
}
