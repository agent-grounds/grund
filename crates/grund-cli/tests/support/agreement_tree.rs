//! The fixture trees and the command runs `command_agreement.rs` compares
//! (§REQ-one-answer.proof): one tree per question, each command through its own
//! entrypoint, and the answers read off as exit codes, stderr lines and JSON rows.

use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[path = "agreement_fixture_root.rs"]
pub(super) mod fixture_root;
use fixture_root::FixtureRoot;

#[path = "agreement_fixture_checks.rs"]
pub(super) mod fixture_checks;
#[path = "agreement_tree_ownership.rs"]
mod ownership;

/// One invariant asked of one tree: `Err` carries the two answers that differ.
pub type Verdict = Result<(), String>;

/// `{kind}-{slug}` with one kind, `FS`, at `docs/fs.md`.
pub const CONFIG_SLUG: &str = "grund_config_version = 1\nproject_name = \"t\"\n\
    [reference]\nmarker = \"\u{a7}\"\nstrict = true\n[id]\nformat = \"{kind}-{slug}\"\n\
    [[kinds]]\nkind = \"FS\"\nfile = \"docs/fs.md\"\n";
/// `{kind}-{number}-{slug}` with one kind, `FS`, at `docs/fs.md`.
pub const CONFIG_NUMBER: &str = "grund_config_version = 1\nproject_name = \"t\"\n\
    [reference]\nmarker = \"\u{a7}\"\nstrict = true\n[id]\nformat = \"{kind}-{number}-{slug}\"\n\
    [[kinds]]\nkind = \"FS\"\nfile = \"docs/fs.md\"\n";

/// One fixture tree under the worktree's own `target/`.
pub struct Tree(pub PathBuf, pub(super) FixtureRoot);

impl Tree {
    pub fn new(name: &str, files: &[(&str, &str)]) -> Self {
        // §AR-ci.10.1, §AR-ci.10.2: claim even an empty fixture before writing.
        let parent =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/command-agreement");
        let owner = FixtureRoot::new(&parent, name).expect("claim fixture root");
        let root = owner.path();
        for (path, body) in files {
            let path = root.join(path);
            fs::create_dir_all(path.parent().expect("parent")).expect("create fixture parent");
            fs::write(path, body).expect("write fixture");
        }
        Self(root.to_path_buf(), owner)
    }

    pub fn grund(&self, args: &[&str]) -> Output {
        grund(self.1.path(), args)
    }

    /// The exit code of one command, `-1` where it was killed.
    pub fn code(&self, args: &[&str]) -> i32 {
        self.grund(args).status.code().unwrap_or(-1)
    }

    /// The first `error:` line one command prints, or `""`.
    pub fn error(&self, args: &[&str]) -> String {
        first_error(&self.grund(args))
    }

    /// The JSON rows one command prints on stdout.
    pub fn json(&self, args: &[&str]) -> Vec<Value> {
        json_lines(&stdout(&self.grund(args)))
    }
}

pub fn grund(root: &Path, args: &[&str]) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(root)
        .output()
        .unwrap_or_else(|error| panic!("run grund {args:?}: {error}"));
    super::agreement_evidence::cli(root, args, &output);
    output
}

pub fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

pub fn first_error(output: &Output) -> String {
    stderr(output)
        .lines()
        .find(|line| line.starts_with("error:"))
        .unwrap_or("")
        .to_string()
}

pub fn json_lines(text: &str) -> Vec<Value> {
    text.lines()
        .filter(|line| line.starts_with('{'))
        .map(|line| serde_json::from_str(line).unwrap_or_else(|error| panic!("{error}: {line}")))
        .collect()
}

/// `Err` naming both answers unless they agree.
pub fn agree<T: PartialEq + std::fmt::Debug>(what: &str, one: T, other: T) -> Verdict {
    if one == other {
        Ok(())
    } else {
        Err(format!("{what}: {one:?} against {other:?}"))
    }
}

/// The steps of #576 a deferred row or a widening may name: the ids of
/// `tests/integration/one_answer_steps.toml`, the one list of the live steps
/// (§REQ-one-answer.proof), read off its `id = "…"` lines.
pub fn listed_steps() -> BTreeSet<String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/integration/one_answer_steps.toml");
    let text = fs::read_to_string(path).expect("read one_answer_steps.toml");
    text.lines().filter_map(step_id).collect()
}

/// The id an `id = "…"` line of one_answer_steps.toml sets, whatever its indent,
/// quote or trailing `# comment`; an id itself may start with `#`.
fn step_id(line: &str) -> Option<String> {
    let rest = line
        .trim()
        .strip_prefix("id")?
        .trim_start()
        .strip_prefix('=')?
        .trim_start();
    let quote = rest.chars().next().filter(|c| *c == '"' || *c == '\'')?;
    let (id, tail) = rest[1..].split_once(quote)?;
    let tail = tail.trim();
    (tail.is_empty() || tail.starts_with('#')).then(|| id.to_string())
}
