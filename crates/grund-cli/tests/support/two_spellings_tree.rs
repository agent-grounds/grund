//! The trees behind `scan_failure_two_spellings`: a project reached through in-project links,
//! plain or as a workspace root, and the exact failure lines a run owes (§FS-check.2.4.2).
#![allow(dead_code)]

use serde_json::Value;
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::PathBuf;
use std::process::Command;

pub const REMEDY: &str = "make the path readable, or keep it out of the scan";
pub const ROOT_REMEDY: &str = "make the path readable, or keep it out of the scan by removing it \
    from the [schema.kinds] row that lists it, because exclude does not prune scan roots";
pub const PASSED_REMEDY: &str = "make the path readable, or keep it out of the scan by not passing \
    it, because exclude does not prune a path the command is given";
pub const OUTSIDE: &str = "outside the report scope, so a citation declared there may be reported as \
                       unresolved";
pub const SKIP: &str =
    "skipped: the mode does not bar this user, so nothing here can be unreadable";

/// The report's config: a non-citable `docs` and an `E2E` kind whose cases are at `cases`.
pub fn config(cases: &str) -> String {
    format!(
        "grund_config_version = 2\nproject_name = \"q\"\n\n[schema]\n\
         slug_pattern = \"[a-z][a-z-]*\"\n\n[schema.kinds.doc]\ncitable = false\n\
         folders = [\"docs\"]\n\n[schema.kinds.E2E]\nfolders = [\"{cases}\"]\nindex = false\n"
    )
}

/// Where the tree's project stands: a plain project, or the root project of a workspace.
#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Plain,
    WorkspaceRoot,
}

pub const SHAPES: [Shape; 2] = [Shape::Plain, Shape::WorkspaceRoot];

/// A project in a directory of its own, its barred entries given back on drop.
pub struct Tree {
    pub root: PathBuf,
    pub locked: Vec<PathBuf>,
}

pub struct Run {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Run {
    /// The scan failures a run reports, from either stream: an `error:` line naming no line
    /// number, or the caution of §FS-check.1.3.6.3. Every other warning says nothing about one.
    pub fn failure_lines(&self) -> Vec<&str> {
        self.stdout
            .lines()
            .chain(self.stderr.lines())
            .filter(|line| {
                line.starts_with("error: ")
                    || line.starts_with("warning: ") && line.contains(" — outside the report scope")
            })
            .collect()
    }

    /// `(severity, path)` of every `io` record of a `--format json` run.
    pub fn io(&self) -> Vec<(String, String)> {
        self.stdout
            .lines()
            .chain(self.stderr.lines())
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .filter(|record| record["code"] == "io")
            .map(|record| {
                let field = |name: &str| record[name].as_str().unwrap_or("(none)").to_string();
                (field("severity"), field("path"))
            })
            .collect()
    }

    pub fn shown(&self) -> String {
        format!("stdout:\n{}\nstderr:\n{}", self.stdout, self.stderr)
    }
}

impl Tree {
    pub fn new(test: &str, shape: Shape, config: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-1288-{test}-{shape:?}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs")).expect("create the home");
        fs::write(root.join("docs/notes.md"), "# Notes\n").expect("write the notes");
        let mut text = config.to_string();
        if matches!(shape, Shape::WorkspaceRoot) {
            text.push_str("\n[workspace]\nmembers = [\"m\"]\n");
            fs::create_dir_all(root.join("m/spec")).expect("create the member");
            fs::write(
                root.join("m/grund.toml"),
                "grund_config_version = 2\nproject_name = \"m\"\n\n[schema]\n\
                 slug_pattern = \"[a-z][a-z-]*\"\n\n[schema.kinds.FS]\nfolders = [\"spec\"]\n",
            )
            .expect("write the member's config");
        }
        fs::write(root.join("grund.toml"), text).expect("write the config");
        Self {
            root,
            locked: Vec::new(),
        }
    }

    pub fn dirs(&self, rels: &[&str]) {
        for rel in rels {
            fs::create_dir_all(self.root.join(rel)).expect("create the directory");
        }
    }

    /// Creates the link `link`, pointing at `target` as written relative to the link.
    pub fn link(&self, link: &str, target: &str) {
        symlink(target, self.root.join(link)).expect("create the link");
    }

    pub fn write(&self, rel: &str, text: &str) {
        fs::write(self.root.join(rel), text).expect("write the file");
    }

    /// Bars the directory or file `rel`, and answers whether that bars this user.
    pub fn bar(&mut self, rel: &str) -> bool {
        let path = self.root.join(rel);
        let is_file = path.is_file();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("set the mode");
        self.locked.push(path.clone());
        let barred = if is_file {
            fs::read(&path).is_err()
        } else {
            fs::read_dir(&path).is_err()
        };
        if !barred {
            eprintln!("{SKIP}");
        }
        barred
    }

    pub fn grund(&self, args: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .env("NO_COLOR", "1")
            .current_dir(&self.root)
            .output()
            .expect("run grund");
        Run {
            code: output.status.code().expect("exited with a status"),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    /// What each command of `wants` printed where it was not the exit and the exact failure
    /// lines wanted, so one assertion shows every surface.
    pub fn misreports(&self, wants: &[Want<'_>]) -> Vec<String> {
        wants
            .iter()
            .filter_map(|(args, code, lines)| {
                let run = self.grund(args);
                (run.code != *code || run.failure_lines() != *lines).then(|| {
                    format!(
                        "{}: grund {}: exit {}, wanted {code} and {lines:#?}\n{}",
                        self.root.display(),
                        args.join(" "),
                        run.code,
                        run.shown()
                    )
                })
            })
            .collect()
    }

    pub fn assert_prints(&self, wants: &[Want<'_>]) {
        assert_none(&self.misreports(wants));
    }

    /// `e2e -> realE`, `realE/cases` barred: the report's tree. Answers whether the bar holds.
    pub fn reports_tree(&mut self) -> bool {
        self.dirs(&["realE/cases"]);
        self.link("e2e", "realE");
        self.bar("realE/cases")
    }

    /// `realE/link -> ../<target>`, the target barred, `e2e -> realE`.
    pub fn link_under_two_names(&mut self, target: &str) -> bool {
        self.dirs(&["realE", "zz/cases", target]);
        self.link("realE/link", &format!("../{target}"));
        self.link("e2e", "realE");
        self.bar(target)
    }

    /// `aa -> zz`, `realE/lnk -> ../zz`, `zz/x` barred: the name a whole run keeps is under
    /// `realE` neither as written nor resolved.
    pub fn third_name(&mut self) -> bool {
        self.dirs(&["realE", "zz/x", "zz/cases"]);
        self.link("aa", "zz");
        self.link("realE/lnk", "../zz");
        self.bar("zz/x")
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

/// `(the command's arguments, its exit code, its exact failure lines in order)`.
pub type Want<'a> = (&'a [&'a str], i32, Vec<String>);

pub fn assert_none(misreports: &[String]) {
    assert!(
        misreports.is_empty(),
        "{} runs printed otherwise:\n\n{}",
        misreports.len(),
        misreports.join("\n")
    );
}

pub fn denied(path: &str) -> String {
    format!("error: {path}: permission denied; {REMEDY}")
}

/// At a scan root, which `exclude` does not prune (§FS-check.2.4.3).
pub fn denied_root(path: &str) -> String {
    format!("error: {path}: permission denied; {ROOT_REMEDY}")
}

/// At the path a run was handed, which `exclude` does not prune either (§FS-check.2.4.3).
pub fn denied_passed(path: &str) -> String {
    format!("error: {path}: permission denied; {PASSED_REMEDY}")
}

pub fn denied_link_passed(path: &str) -> String {
    format!("error: {path}: unreadable symlink: permission denied; {PASSED_REMEDY}")
}

pub fn denied_link(path: &str) -> String {
    format!("error: {path}: unreadable symlink: permission denied; {REMEDY}")
}

pub fn broken(path: &str) -> String {
    format!("error: {path}: broken symlink: the target does not exist; {REMEDY}")
}

pub fn caution(path: &str) -> String {
    format!("warning: {path}: permission denied — {OUTSIDE}")
}
