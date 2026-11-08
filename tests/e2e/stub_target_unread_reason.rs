//! Why a name is not read is said in one wording, §FS-scan.checks.unread-scan-root.message, and a
//! broken stub's target is told that wording (§FS-declarations.stubs.target.3). Every verdict
//! here stays `broken-stub` and exit 1: only the reason moves. The trees are built in the temp
//! directory so one fixture covers each target name under both configuration versions.

use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

const STUB: &str = "docs/fs/FS-001-a.md";

/// The version 1 sentence for a name with no extension.
const NONE_V1: &str =
    "it has no extension, and [scan] extensions reads a file only by its extension";
/// The version 2 sentence for a name with no extension.
const NONE_V2: &str =
    "it has no extension, and [schema.sources] languages reads a file only by its extension";

const V1: &str = "[scan]\nextensions = [\"md\"]\ninclude = [\"docs\", \"src\"]\n\n[[kinds]]\n\
                  kind = \"FS\"\nfolder = \"docs/fs\"\nindex = false\n";
const V2: &str = "grund_config_version = 2\n\n[schema]\nmarker = \"§\"\n\
                  id_format = \"{kind}-{number}-{slug}\"\n\n[schema.sources]\n\
                  languages = [\"markdown\"]\n\n[schema.kinds.FS]\nfolders = [\"docs/fs\"]\n\
                  index = false\n\n[schema.kinds.code]\nfolders = [\"docs\", \"src\"]\n";

/// A scratch project removed on drop; `name` keeps concurrent tests apart.
struct Tree(PathBuf);

impl Tree {
    fn empty(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1275-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create the tree");
        Self(root)
    }

    /// A project whose one stub points at `src/<target>`, which holds the declaration.
    fn stub(name: &str, config: &str, target: &str) -> Self {
        let tree = Self::empty(name);
        tree.write("grund.toml", config);
        tree.write(STUB, &format!("# FS-001-a: [FACTS](../../src/{target})\n"));
        tree.write(&format!("src/{target}"), "# FS-001-a: Alpha\n");
        tree.write("docs/a.md", "See \u{a7}FS-001-a.\n");
        tree
    }

    fn write(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().expect("a parent")).expect("create the parent");
        fs::write(path, text).expect("write the fixture");
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .env("NO_COLOR", "1")
            .output()
            .expect("run grund")
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// `check` keeps the verdict and prints exactly this reason for `src/<target>`.
fn assert_reason(name: &str, config: &str, target: &str, reason: &str) {
    let tree = Tree::stub(name, config, target);
    let output = tree.run(&["check"]);
    let text = stdout(&output);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let expected =
        format!("{STUB}:1: error: stub link target ../../src/{target} is not scanned: {reason}");
    let line = text.lines().find(|l| l.contains("stub link target"));
    assert_eq!(line, Some(expected.as_str()), "{text}");
}

/// `--format=json` carries `reason` in the `broken-stub` finding's `message`.
fn assert_json_reason(name: &str, config: &str, target: &str, reason: &str) {
    let tree = Tree::stub(name, config, target);
    let output = tree.run(&["check", "--format=json"]);
    let text = stdout(&output);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let finding: Value = text
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).expect("a JSON finding"))
        .find(|f| f["code"] == "broken-stub")
        .expect("a broken-stub finding");
    assert_eq!(
        finding["message"].as_str(),
        Some(format!("stub link target ../../src/{target} is not scanned: {reason}").as_str()),
        "{text}"
    );
}

#[test]
fn a_target_with_no_extension_is_told_so_in_version_1() {
    assert_reason("none-v1", V1, "FACTS", NONE_V1);
}

#[test]
fn a_target_with_no_extension_is_told_so_in_version_2() {
    assert_reason("none-v2", V2, "FACTS", NONE_V2);
}

#[test]
fn a_target_ending_in_a_dot_has_no_extension_in_version_1() {
    assert_reason("dot-v1", V1, "FACTS.", NONE_V1);
}

#[test]
fn a_target_ending_in_a_dot_has_no_extension_in_version_2() {
    assert_reason("dot-v2", V2, "FACTS.", NONE_V2);
}

#[test]
fn version_2_names_its_languages_for_an_extension_nothing_maps() {
    let reason = "no language in [schema.sources] languages reads `.zz`";
    assert_reason("zz-v2", V2, "FACTS.zz", reason);
}

#[test]
fn version_2_names_its_languages_for_a_known_language_it_does_not_list() {
    let reason = "no language in [schema.sources] languages reads `.rs`";
    assert_reason("rs-v2", V2, "FACTS.rs", reason);
}

#[test]
fn the_json_message_carries_the_same_reason() {
    assert_json_reason("none-json", V1, "FACTS", NONE_V1);
    let reason = "no language in [schema.sources] languages reads `.zz`";
    assert_json_reason("zz-json", V2, "FACTS.zz", reason);
}

#[test]
fn the_warning_says_the_version_1_reason_the_stub_line_says() {
    let tree = Tree::empty("warning-v1");
    tree.write(
        "grund.toml",
        "grund_config_version = 1\n\n[[kinds]]\nkind = \"TXT\"\nfile = \"notes.txt\"\n\n\
         [scan]\ninclude = [\"docs\"]\nextensions = [\"md\"]\n",
    );
    tree.write("notes.txt", "notes\n");
    tree.write("docs/a.md", "# Notes\n");
    let text = stdout(&tree.run(&["check"]));
    assert!(
        text.contains("`notes.txt` is not read — extension txt is not in [scan] extensions, so"),
        "{text}"
    );
}

#[test]
fn a_hidden_and_unlisted_warning_says_the_same_version_1_reason() {
    let tree = Tree::empty("warning-hidden-v1");
    tree.write(
        "grund.toml",
        "grund_config_version = 1\n\n[[kinds]]\nkind = \"TXT\"\nfile = \".notes.txt\"\n\n\
         [scan]\ninclude = [\"docs\"]\nextensions = [\"md\"]\n",
    );
    tree.write(".notes.txt", "notes\n");
    tree.write("docs/a.md", "# Notes\n");
    let text = stdout(&tree.run(&["check"]));
    assert!(
        text.contains("its name begins with `.`, and extension txt is not in [scan] extensions"),
        "{text}"
    );
}

/// The control: a version 1 target with an extension keeps its line word for word.
#[test]
fn a_version_1_target_with_an_extension_keeps_its_line() {
    let reason = "extension zz is not in [scan] extensions";
    assert_reason("zz-v1", V1, "FACTS.zz", reason);
}

/// The control: a target the configuration reads pairs with its stub, under either version.
#[test]
fn a_read_target_makes_the_stub_healthy() {
    for (name, config) in [("md-v1", V1), ("md-v2", V2)] {
        let tree = Tree::stub(name, config, "FACTS.md");
        let output = tree.run(&["check"]);
        assert_eq!(output.status.code(), Some(0), "{output:?}");
    }
}
