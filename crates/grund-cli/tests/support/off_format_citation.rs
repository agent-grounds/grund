//! Test-only repositories for the off-format citation tests: the #1073 fixture, the configs its
//! siblings read in, and the message of §FS-check.checks.off-format-citation. Every marker a
//! fixture writes is spelled `\u{a7}` here, so this file cites nothing it illustrates.

#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The issue's config: `FS` and `REQ` under the default `{kind}-{slug}` format.
pub const CONFIG: &str = "grund_config_version = 2
project_name = \"fixture\"

[schema.kinds.FS]
folders = [\"docs/FS\"]
index = false

[schema.kinds.REQ]
folders = [\"docs/REQ\"]
index = false
";

/// A schema-wide numbered format, `{kind}-{number}-{slug}`.
pub const NUMBERED: &str = "grund_config_version = 2
project_name = \"numbered\"

[schema]
id_format = \"{kind}-{number}-{slug}\"

[schema.kinds.FS]
folders = [\"docs/FS\"]
index = false
";

/// A kind with its own format: `TICKET` IDs are `{kind}-{number}`.
pub const TICKETS: &str = "grund_config_version = 2
project_name = \"tickets\"

[schema.kinds.FS]
folders = [\"docs/FS\"]
index = false

[schema.kinds.TICKET]
folders = [\"docs/TICKET\"]
index = false
id_format = \"{kind}-{number}\"
";

/// The issue's page: two tokens whose slug opens with a digit, and one in-format dangling one.
pub const ISSUE: &str = "# FS-a: A\n\nSee \u{a7}REQ-999 and \u{a7}REQ-12a and \u{a7}REQ-a12.\n";

/// The page every fixture of [`Fixture::source`] declares, cited from its source file.
pub const PAGE: &str = "docs/FS/FS-a.md";

pub const CODE: &str = "off-format-citation";

/// What a `REQ` token is held to under [`CONFIG`].
pub const REQ_FORMAT: &str =
    "REQ IDs follow [schema] id_format = \"{kind}-{slug}\" with slug_pattern = \"[a-z][a-z0-9-]*\"";

/// What an `FS` token is held to under [`NUMBERED`]: both components, in the template's order.
pub const FS_NUMBERED: &str = "FS IDs follow [schema] id_format = \"{kind}-{number}-{slug}\" \
                               with number_pattern = \"\\d+\" and slug_pattern = \
                               \"[a-z][a-z0-9-]*\"";

/// What a `TICKET` token is held to under [`TICKETS`]: its own key, and only `{number}`.
pub const TICKET_FORMAT: &str = "TICKET IDs follow [schema.kinds.TICKET] id_format = \
                                 \"{kind}-{number}\" with number_pattern = \"\\d+\"";

pub const UNUSED_FS_A: &str =
    "docs/FS/FS-a.md:1: warning: declared but never cited: FS-a; cite it\n";

/// The message of §FS-check.checks.off-format-citation for `token`, whose kind `follows`, closing
/// with the release attribution of §FS-check.checks.off-format-citation.1.
pub fn off_format(token: &str, follows: &str) -> String {
    // Releases as arguments: the release guard (§FS-distribution.4.2.5) reads the clause from the
    // code that ships it, not from this test ahead of it.
    let (unchecked, error) = ("0.20.0", "0.21.0");
    format!(
        "off-format citation \u{a7}{token}: {follows}, and nothing declares this one; cite an ID \
         in that format, or write <\u{a7}>{token} for an illustration \
         — unchecked in grund {unchecked}, an error in {error}"
    )
}

/// One text-output finding, `<path>:<line>: <severity>: <message>`.
pub fn text(path: &str, line: usize, severity: &str, message: &str) -> String {
    format!("{path}:{line}: {severity}: {message}\n")
}

/// One finding as `check --format json` prints it, with the message's `"` and `\` escaped.
pub fn json(severity: &str, path: &str, line: usize, code: &str, message: &str) -> String {
    let message = message.replace('\\', "\\\\").replace('"', "\\\"");
    format!(
        "{{\"severity\":\"{severity}\",\"path\":\"{path}\",\"line\":{line},\"code\":\"{code}\",\
         \"message\":\"{message}\",\"sites\":null,\"authority\":null}}\n"
    )
}

pub struct Fixture(pub PathBuf);

pub struct Run {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl Fixture {
    pub fn with(files: &[(&str, &str)]) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("grund-1073-{}-{serial}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        for (path, contents) in files {
            let path = dir.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, contents).unwrap();
        }
        Self(dir)
    }

    /// The issue's repository: no requirement declared, and [`ISSUE`] as its one page.
    pub fn issue() -> Self {
        Self::page(ISSUE)
    }

    /// The issue's config and empty `REQ` folder, with `text` as `docs/FS/FS-a.md`.
    pub fn page(text: &str) -> Self {
        Self::with(&[("grund.toml", CONFIG), ("docs/REQ/.keep", ""), (PAGE, text)])
    }

    /// The issue's config, a page that cites nothing, and `text` as the source file at `path`.
    pub fn source(path: &str, text: &str) -> Self {
        Self::with(&[
            ("grund.toml", CONFIG),
            ("docs/REQ/.keep", ""),
            (PAGE, "# FS-a: A\n\nText.\n"),
            (path, text),
        ])
    }

    pub fn read(&self, path: &str) -> String {
        fs::read_to_string(self.0.join(path)).unwrap()
    }

    pub fn run(&self, args: &[&str]) -> Run {
        let out = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .expect("run grund");
        Run {
            code: out.status.code(),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        }
    }

    /// Run `grund args…`, require exit `code`, and return the run.
    pub fn expect(&self, code: i32, args: &[&str]) -> Run {
        let run = self.run(args);
        assert_eq!(
            run.code,
            Some(code),
            "grund {} exited {:?}: stdout:\n{}\nstderr:\n{}",
            args.join(" "),
            run.code,
            run.stdout,
            run.stderr
        );
        run
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
