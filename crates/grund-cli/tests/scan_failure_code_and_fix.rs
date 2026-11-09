//! A scan failure carries its own code and a fix that works at its path (§FS-check.2.4.3): a file
//! read and not parsed is `unparsable-file`, exit `2`, and an unread path's `io` fix is the case
//! of its place (§FS-errors.5.5). Acceptance cases of #1272 and the reports fused into it, #1251
//! and #1263; `ignore` marks are removed by the implementation.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const V2: &str = "grund_config_version = 2\nproject_name = \"p\"\n\n[schema]\n\
    slug_pattern = \"[a-z][a-z-]*\"\n\n[schema.kinds.FS]\nfolders = [\"docs/fs\"]\n";
const V2_VALUES: &str = "grund_config_version = 2\nproject_name = \"v\"\n\n\
    [schema.kinds.CONST]\nfolders = [\"values\"]\nform = \"value\"\n";
const V2_E2E: &str = "grund_config_version = 2\nproject_name = \"p\"\n\n[schema]\n\
    slug_pattern = \"[a-z][a-z-]*\"\n\n[schema.kinds.E2E]\nfolders = [\"e2e/cases\"]\n";
const ENTRY: &str = "make the path readable, or keep it out of the scan";
const ROOT_V2: &str = "make the path readable, or keep it out of the scan by removing it from \
    the [schema.kinds] row that lists it, because exclude does not prune scan roots";
const ROOT_V1: &str = "make the path readable, or keep it out of the scan by removing it from \
    the [scan] include or [[kinds]] row that lists it, because exclude does not prune scan roots";
const PASSED: &str = "make the path readable, or keep it out of the scan by not passing it, \
    because exclude does not prune a path the command is given";
const VALUE: &str = "make the path readable, or keep it out of the scan by moving it out of the \
    value home, because exclude does not reach a value source";
const EXIT: &str = "write the case's expected exit status in the file as one integer";
const IGNORE: &str = "write the line it names as a valid ignore pattern";

/// A project under the temp directory; what it locked is unlocked again on drop.
struct Tree {
    root: PathBuf,
    locked: Vec<PathBuf>,
}

struct Run {
    code: Option<i32>,
    stderr: String,
}

impl Tree {
    fn new(name: &str, config: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1272-{name}-{}", std::process::id()));
        if root.exists() {
            let _ = Command::new("chmod")
                .arg("-R")
                .arg("u+rwx")
                .arg(&root)
                .status();
            fs::remove_dir_all(&root).expect("clear the tree");
        }
        fs::create_dir_all(&root).expect("create the tree");
        fs::write(root.join("grund.toml"), config).expect("write the config");
        Self {
            root,
            locked: Vec::new(),
        }
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("below the tree")).expect("create the parent");
        fs::write(path, content).expect("write the file");
    }

    fn append_config(&self, tail: &str) {
        let config = fs::read_to_string(self.root.join("grund.toml")).expect("read the config");
        fs::write(self.root.join("grund.toml"), config + tail).expect("write the config");
    }

    fn lock(&mut self, relative: &str) {
        let path = self.root.join(relative);
        let mode = fs::metadata(&path)
            .expect("the path exists")
            .permissions()
            .mode();
        assert!(mode & 0o777 != 0, "{relative} is already locked");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("lock");
        assert!(
            is_denied(&path),
            "mode 000 must deny access; run as an unprivileged user"
        );
        self.locked.push(path);
    }

    fn grund(&self, args: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.root)
            .env("NO_COLOR", "1")
            .output()
            .expect("run grund");
        Run {
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    /// Asserts both surfaces of one scan failure at `path`: the `error:` line ends with `fix`, and
    /// the JSON record of it wears `code`. Exit `2` on each (§FS-check.2.4).
    fn assert_failure(&self, args: &[&str], path: &str, fix: &str, code: &str) {
        let text = self.grund(args);
        assert_eq!(text.code, Some(2), "{args:?}: {}", text.stderr);
        let line = failure_line(&text.stderr, "error", path);
        assert!(line.ends_with(&format!("; {fix}")), "{line}");
        let mut json_args = args.to_vec();
        json_args.push("--format=json");
        let json = self.grund(&json_args);
        assert_eq!(json.code, Some(2), "{json_args:?}: {}", json.stderr);
        assert_eq!(
            json_code(&json.stderr, path).as_deref(),
            Some(code),
            "{}",
            json.stderr
        );
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        for path in &self.locked {
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o755));
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn is_denied(path: &Path) -> bool {
    let read = if path.is_dir() {
        fs::read_dir(path).map(|_| ()).err()
    } else {
        fs::read(path).map(|_| ()).err()
    };
    read.map(|e| e.kind()) == Some(std::io::ErrorKind::PermissionDenied)
}

/// The one `<level>: <path>: …` line of a run; a failure is one line (§FS-check.2.4.2).
fn failure_line<'a>(stderr: &'a str, level: &str, path: &str) -> &'a str {
    let prefix = format!("{level}: {path}: ");
    let lines: Vec<&str> = stderr.lines().filter(|l| l.starts_with(&prefix)).collect();
    assert_eq!(lines.len(), 1, "one line for {path}, in:\n{stderr}");
    lines[0]
}

/// The code of the NDJSON record at `path`, without parsing a message.
fn json_code(stderr: &str, path: &str) -> Option<String> {
    let line = stderr
        .lines()
        .map(|l| l.replace("\": ", "\":"))
        .find(|l| l.contains(&format!("\"path\":\"{path}\"")))?;
    let tail = line.split("\"code\":\"").nth(1)?;
    Some(tail.split('"').next()?.to_owned())
}

fn exit_case(name: &str) -> Tree {
    let tree = Tree::new(name, V2_E2E);
    tree.write("e2e/cases/a/expected.exit", "not-an-exit\n");
    tree
}

/// #1251: a readable `expected.exit` that is no integer.
#[test]
#[ignore = "red until #1272 records a parse failure as unparsable-file; implement removes this"]
fn a_malformed_expected_exit_is_unparsable_and_says_what_it_should_hold() {
    let tree = exit_case("exit");
    tree.assert_failure(
        &["check"],
        "e2e/cases/a/expected.exit",
        EXIT,
        "unparsable-file",
    );
}

#[test]
#[ignore = "red until #1272 records a parse failure as unparsable-file; implement removes this"]
fn a_json_value_source_that_does_not_parse_is_unparsable() {
    let tree = Tree::new("json", V2_VALUES);
    tree.write("values/a.json", "{ not json\n");
    tree.assert_failure(
        &["check"],
        "values/a.json",
        "make the file valid JSON",
        "unparsable-file",
    );
}

#[test]
#[ignore = "red until #1272 records a parse failure as unparsable-file; implement removes this"]
fn a_yaml_value_source_that_does_not_parse_is_unparsable() {
    let tree = Tree::new("yaml", V2_VALUES);
    tree.write("values/a.yaml", "a: [1, 2\nb: }\n");
    tree.assert_failure(
        &["check"],
        "values/a.yaml",
        "make the file valid YAML",
        "unparsable-file",
    );
}

#[test]
#[ignore = "red until #1272 records a parse failure as unparsable-file; implement removes this"]
fn an_ignore_file_with_a_bad_line_is_unparsable() {
    let tree = Tree::new("ignore", V2);
    tree.write("docs/fs/x.md", "# Notes\n");
    tree.write(".ignore", "{alpha\n");
    tree.assert_failure(&["check"], ".ignore", IGNORE, "unparsable-file");
}

/// §FS-check.1.4: `--ignore unparsable-file` is accepted and hides nothing.
#[test]
#[ignore = "red until #1272 makes unparsable-file a catalog code; implement removes this"]
fn ignoring_unparsable_file_is_accepted_and_hides_nothing() {
    let tree = exit_case("ignore-flag");
    let run = tree.grund(&["check", "--ignore", "unparsable-file"]);
    assert_eq!(run.code, Some(2), "{}", run.stderr);
    let line = failure_line(&run.stderr, "error", "e2e/cases/a/expected.exit");
    assert!(line.ends_with(&format!("; {EXIT}")), "{line}");
}

/// §FS-check.1.3.6.3: the caution of a narrowed run wears the failure's own code.
#[test]
#[ignore = "red until #1272 gives the caution the record's code; implement removes this"]
fn a_parse_failure_outside_a_narrowed_run_cautions_as_unparsable() {
    let tree = Tree::new(
        "narrowed",
        &(V2.to_owned() + "\n[schema.kinds.CONST]\nfolders = [\"values\"]\nform = \"value\"\n"),
    );
    tree.write("docs/fs/FS-a.md", "# FS-a: A\n\nLead.\n");
    tree.write("values/a.json", "{ not json\n");
    let run = tree.grund(&["check", "docs/fs/FS-a.md", "--format=json"]);
    assert_ne!(
        run.code,
        Some(2),
        "a caution leaves the exit alone: {}",
        run.stderr
    );
    assert_eq!(
        json_code(&run.stderr, "values/a.json").as_deref(),
        Some("unparsable-file"),
        "{}",
        run.stderr
    );
}

/// #1263: `exclude` cannot prune a root, so the fix says what can, before and after trying it.
#[test]
#[ignore = "red until #1272 picks io's fix by place; implement removes this"]
fn an_unreadable_scan_root_names_its_row_before_and_after_exclude_names_it() {
    let mut tree = Tree::new("root", V2);
    fs::create_dir_all(tree.root.join("docs/fs")).expect("create the root");
    tree.lock("docs/fs");
    tree.assert_failure(&["check"], "docs/fs", ROOT_V2, "io");
    tree.append_config("\n[schema.sources]\nexclude = [\"docs/fs\"]\n");
    tree.assert_failure(&["check"], "docs/fs", ROOT_V2, "io");
}

#[test]
#[ignore = "red until #1272 picks io's fix by place; implement removes this"]
fn a_nested_root_names_its_row() {
    let mut nested = Tree::new(
        "nested",
        "grund_config_version = 2\nproject_name = \"p\"\n\n[schema]\n\
         slug_pattern = \"[a-z][a-z-]*\"\n\n[schema.kinds.FS]\nfolders = [\"docs\"]\n\n\
         [schema.kinds.AR]\nfolders = [\"docs/ar\"]\n",
    );
    fs::create_dir_all(nested.root.join("docs/ar")).expect("create the nested root");
    nested.lock("docs/ar");
    nested.assert_failure(&["check"], "docs/ar", ROOT_V2, "io");
}

#[test]
#[ignore = "red until #1272 picks io's fix by place; implement removes this"]
fn a_single_file_home_names_its_row() {
    let mut file = Tree::new(
        "file-home",
        "grund_config_version = 2\nproject_name = \"p\"\n\n[schema]\n\
         slug_pattern = \"[a-z][a-z-]*\"\n\n[schema.kinds.FS]\nfiles = [\"spec.md\"]\n",
    );
    file.write("spec.md", "# FS-a: A\n");
    file.lock("spec.md");
    file.assert_failure(&["check"], "spec.md", ROOT_V2, "io");
}

/// A version-1 config names its own row (§FS-errors.5.5.1).
#[test]
#[ignore = "red until #1272 picks io's fix by place; implement removes this"]
fn a_version_1_scan_root_names_its_own_row() {
    let mut tree = Tree::new(
        "v1-root",
        "grund_config_version = 1\n\n[scan]\ninclude = [\"src\"]\n",
    );
    fs::create_dir_all(tree.root.join("src")).expect("create the root");
    tree.lock("src");
    tree.assert_failure(&["check"], "src", ROOT_V1, "io");
}

#[test]
#[ignore = "red until #1272 picks io's fix by place; implement removes this"]
fn a_path_the_command_was_given_says_to_stop_passing_it() {
    let mut tree = Tree::new("passed", V2);
    tree.write("docs/fs/FS-a.md", "# FS-a: A\n");
    tree.write("notes/x.md", "text\n");
    tree.lock("notes/x.md");
    tree.assert_failure(&["check", "notes/x.md"], "notes/x.md", PASSED, "io");
}

#[test]
#[ignore = "red until #1272 picks io's fix by place; implement removes this"]
fn a_value_source_says_the_value_home_reaches_it() {
    let mut source = Tree::new("value", V2_VALUES);
    source.write("values/a.json", "{}\n");
    source.lock("values/a.json");
    source.assert_failure(&["check"], "values/a.json", VALUE, "io");
}

#[test]
#[ignore = "red until #1272 picks io's fix by place; implement removes this"]
fn a_value_home_names_its_row() {
    let mut home = Tree::new("value-home", V2_VALUES);
    fs::create_dir_all(home.root.join("values")).expect("create the home");
    home.lock("values");
    home.assert_failure(&["check"], "values", ROOT_V2, "io");
}

#[test]
#[ignore = "red until #1272 picks io's fix by place; implement removes this"]
fn a_section_lock_is_only_told_to_be_readable() {
    let mut tree = Tree::new("lock", V2);
    tree.write("docs/fs/FS-a.md", "# FS-a: A\n");
    tree.write(".grund/sections/FS-a.lock", "");
    tree.lock(".grund/sections/FS-a.lock");
    tree.assert_failure(
        &["check"],
        ".grund/sections/FS-a.lock",
        "make the path readable",
        "io",
    );
}

/// The control: what `exclude` reaches keeps today's bytes and `io`.
#[test]
fn an_unreadable_file_below_a_root_keeps_todays_fix_and_io() {
    let mut tree = Tree::new("control", V2);
    tree.write("docs/fs/x.md", "# FS-a: A\n");
    tree.lock("docs/fs/x.md");
    let run = tree.grund(&["check"]);
    assert_eq!(run.code, Some(2), "{}", run.stderr);
    assert!(
        run.stderr.contains(&format!(
            "error: docs/fs/x.md: permission denied; {ENTRY}\n"
        )),
        "{}",
        run.stderr
    );
    let json = tree.grund(&["check", "--format=json"]);
    assert_eq!(
        json_code(&json.stderr, "docs/fs/x.md").as_deref(),
        Some("io"),
        "{}",
        json.stderr
    );
}
