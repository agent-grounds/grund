//! The docs-and-src tree behind the `refs` note contracts (§FS-refs.2.1): the
//! scoped typo note of issue #862 and the `list --unused` pointer of #936.
//! `FS-api-audit` is declared and uncited, `FS-login` is cited from `src`.
#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

pub const V1: &str = "grund_config_version = 1\n\
    [reference]\nstrict = true\n\
    [id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\
    [scan]\ninclude = [\"docs\", \"src\"]\n\
    [[kinds]]\nkind = \"FS\"\nfolder = \"docs/functional-spec\"\nindex = false\n";

pub const V2: &str = "grund_config_version = 2\n\
    [schema]\nid_format = \"{kind}-{slug}\"\n\
    [schema.kinds.FS]\nfolders = [\"docs/functional-spec\"]\nindex = false\n\
    [schema.kinds.code]\nfolders = [\"src\"]\n";

/// Every rendering of an empty answer, with its stdout: text and JSON, each
/// as rows, per file, totalled, and both, where `--total` wins (§FS-refs.3.4).
pub const RENDERINGS: [(&[&str], &str); 8] = [
    (&[], ""),
    (&["--summary"], ""),
    (&["--total"], "not cited\n"),
    (&["--total", "--summary"], "not cited\n"),
    (&["--format", "json"], ""),
    (&["--format", "json", "--summary"], ""),
    (
        &["--format", "json", "--total"],
        "{\"sites\":0,\"files\":0}\n",
    ),
    (
        &["--format", "json", "--total", "--summary"],
        "{\"sites\":0,\"files\":0}\n",
    ),
];

/// The stderr §FS-refs.2.1.1 gives for an uncited `id`, naming `list`.
pub fn unused_note(id: &str, list: &str) -> String {
    format!("note: {id} is declared but nothing cites it; `{list}` lists every such declaration\n")
}

pub struct Fixture(pub PathBuf);

impl Fixture {
    /// One project, or with `workspace` a root aliased `beta` beside the
    /// member `packages/other`, under either configuration version.
    pub fn new(v2: bool, workspace: bool) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-862-refs-note-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("docs/functional-spec")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        let config = if v2 { V2 } else { V1 };
        let config = if workspace {
            fs::create_dir_all(root.join("packages/other/docs/functional-spec")).unwrap();
            fs::write(root.join("packages/other/grund.toml"), V1).unwrap();
            format!(
                "project_name = \"beta\"\n{config}\n\
                 [workspace]\nmembers = [\"packages/other\"]\n"
            )
        } else {
            config.to_owned()
        };
        fs::write(root.join("grund.toml"), config).unwrap();
        for (slug, title) in [("api-audit", "API audit"), ("login", "Login")] {
            fs::write(
                root.join(format!("docs/functional-spec/FS-{slug}.md")),
                format!("# FS-{slug}: {title}\n\nText.\n"),
            )
            .unwrap();
        }
        fs::write(
            root.join("src/lib.rs"),
            "// See \u{a7}FS-login.\npub fn f() {}\n",
        )
        .unwrap();
        Self(root)
    }

    /// Add an uncited `E2E-login` case and an uncited custom `API-thing` to the
    /// root project, whose configuration version `v2` names.
    pub fn add_kinds(&self, v2: bool) {
        self.append(
            "grund.toml",
            if v2 {
                "[schema.kinds.E2E]\nfolders = [\"docs/e2e\"]\nindex = false\n\
                 [schema.kinds.API]\nfolders = [\"docs/api\"]\nindex = false\n"
            } else {
                "[[kinds]]\nkind = \"E2E\"\nfolder = \"docs/e2e\"\n\
                 [[kinds]]\nkind = \"API\"\nfolder = \"docs/api\"\nindex = false\n"
            },
        );
        self.write("docs/e2e/login/command.args", "check\n");
        self.write("docs/e2e/login/expected.exit", "0\n");
        self.write("docs/api/API-thing.md", "# API-thing: Thing\n\nText.\n");
    }

    /// Run the `grund ...` command line a note names from `cwd`, and require
    /// that it succeeds and lists `row`, so the advice holds the target.
    pub fn check_lists(&self, cwd: &str, list: &str, row: &str, failures: &mut Vec<String>) {
        let args: Vec<&str> = list.split(' ').skip(1).collect();
        let output = self.run(cwd, &args);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let listed = stdout
            .lines()
            .any(|line| line.split_whitespace().next() == Some(row));
        if !output.status.success() || !listed {
            failures.push(format!(
                "(cd {cwd}) {list}: expected a {row} row, got {:?}\n{stdout}",
                output.status
            ));
        }
    }

    /// Write `body` at `path` under the fixture, its directories made.
    pub fn write(&self, path: &str, body: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    /// Append `text` to the file at `path`, such as kinds to `grund.toml`.
    pub fn append(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        let body = fs::read_to_string(&path).unwrap();
        fs::write(path, body + text).unwrap();
    }

    pub fn run(&self, cwd: &str, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .current_dir(self.0.join(cwd))
            .args(args)
            .output()
            .expect("run grund")
    }

    /// Run `args` from `cwd` and require exit `0` and both streams.
    pub fn check(
        &self,
        cwd: &str,
        args: &[&str],
        expected_stdout: &str,
        expected_stderr: &str,
        failures: &mut Vec<String>,
    ) {
        self.check_exit(cwd, args, 0, expected_stdout, expected_stderr, failures);
    }

    /// Run `args` from `cwd` and require `code` and both streams, each judged
    /// on its own so a wording mismatch cannot hide result drift.
    pub fn check_exit(
        &self,
        cwd: &str,
        args: &[&str],
        code: i32,
        expected_stdout: &str,
        expected_stderr: &str,
        failures: &mut Vec<String>,
    ) {
        let output = self.run(cwd, args);
        let label = format!("(cd {cwd}) grund {}", args.join(" "));
        if output.status.code() != Some(code) {
            failures.push(format!(
                "{label}: expected exit {code}, got {:?}",
                output.status
            ));
        }
        for (stream, actual, expected) in [
            ("stdout", &output.stdout, expected_stdout),
            ("stderr", &output.stderr, expected_stderr),
        ] {
            let actual = String::from_utf8_lossy(actual).replace("\r\n", "\n");
            if actual != expected {
                failures.push(format!(
                    "{label}: {stream}\nexpected: {expected:?}\nactual: {actual:?}"
                ));
            }
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
