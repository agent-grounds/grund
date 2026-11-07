//! Black-box: a value home that cannot be listed is one failure, so every surface reports it once
//! (§FS-check.2.4.2). The ordinary walk and the value pass both list the home, and the pass
//! publishes its listing failure only where the scan has not already reported it, which it
//! consults rather than assumes (§AR-scanner.1.relisted). Git carries no mode a fixture could keep
//! unreadable, so the e2e corpus cannot hold these trees. Every test that bars a directory skips
//! where the mode does not bar this user, as it does not bar root.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const REMEDY: &str = "make the path readable, or keep it out of the scan";
const SKIP: &str = "skipped: the mode does not bar this user, so nothing here can be unreadable";
const CONFIG: &str = "grund_config_version = 2\nproject_name = \"vals\"\n\n\
    [schema.kinds.CONST]\nfolders = [\"values\"]\nform = \"value\"\n\n\
    [schema.kinds.page]\ncitable = false\nfiles = [\"docs/offer.md\"]\n";

/// A project whose `CONST` kind keeps its values in `values`, its barred directory given back on
/// drop.
struct Scratch {
    root: PathBuf,
    locked: Vec<PathBuf>,
}

impl Scratch {
    /// The project, its value home `values` holding one source at `home`, which is `values`
    /// itself or the directory `values` links to.
    fn new(name: &str, home: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1262-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(home)).expect("create the value home");
        fs::create_dir_all(root.join("docs")).expect("create the docs");
        fs::write(root.join("grund.toml"), CONFIG).expect("write the config");
        fs::write(root.join(home).join("prices.json"), "{\"CONST-a\": 1}\n")
            .expect("write the source");
        fs::write(root.join("docs/offer.md"), "# Offer\n\nThe price is `1`.\n")
            .expect("write the page");
        if home != "values" {
            std::os::unix::fs::symlink(home, root.join("values")).expect("link the home");
        }
        Self {
            root,
            locked: Vec::new(),
        }
    }

    /// Takes the directory's mode, and answers whether that bars this user.
    fn lock(&mut self, relative: &str) -> bool {
        let path = self.root.join(relative);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("take the mode");
        self.locked.push(path.clone());
        fs::read_dir(&path).is_err()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        for path in &self.locked {
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o755));
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

fn grund(dir: &Path, args: &[&str]) -> Run {
    let output = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(dir)
        .env("NO_COLOR", "1")
        .output()
        .expect("run grund");
    Run {
        code: output.status.code().expect("exited with a status"),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

impl Run {
    /// The scan failures a run reports, from either stream: a located finding reads
    /// `path:line: error:`, so a line that starts `error: ` is a failure of the run
    /// (§FS-errors.2.2).
    fn failure_lines(&self) -> Vec<&str> {
        self.stdout
            .lines()
            .chain(self.stderr.lines())
            .filter(|line| line.starts_with("error: "))
            .collect()
    }

    /// The `io` records of a `--format json` run, from either stream.
    fn io_records(&self) -> Vec<serde_json::Value> {
        self.stdout
            .lines()
            .chain(self.stderr.lines())
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .filter(|record| record["code"] == "io")
            .collect()
    }

    fn shown(&self) -> String {
        format!("stdout:\n{}\nstderr:\n{}", self.stdout, self.stderr)
    }
}

/// The commands that do not exit `2` with failure lines `accept` takes, each with what it printed,
/// so that one assertion shows every surface.
fn misreports(dir: &Path, commands: &[&[&str]], accept: impl Fn(&[&str]) -> bool) -> Vec<String> {
    commands
        .iter()
        .filter_map(|args| {
            let run = grund(dir, args);
            (run.code != 2 || !accept(&run.failure_lines())).then(|| {
                format!(
                    "grund {}: exit {}\n{}",
                    args.join(" "),
                    run.code,
                    run.shown()
                )
            })
        })
        .collect()
}

fn assert_none(misreports: &[String], runs: usize) {
    assert!(
        misreports.is_empty(),
        "one value home that failed to list once is one line; {} of {runs} runs printed otherwise:\n\n{}",
        misreports.len(),
        misreports.join("\n")
    );
}

/// The walk and the value pass both list a home at mode `000`, and the run prints the one line,
/// the walk's, as the only failure it has.
#[test]
fn an_unlistable_value_home_is_one_line_from_check_and_list() {
    let mut scratch = Scratch::new("value-home", "values");
    if !scratch.lock("values") {
        eprintln!("{SKIP}");
        return;
    }
    let expected = format!("error: values: permission denied; {REMEDY}");
    let commands: &[&[&str]] = &[&["check"], &["list"]];
    assert_none(
        &misreports(&scratch.root, commands, |lines| {
            lines == [expected.as_str()]
        }),
        commands.len(),
    );
}

#[test]
fn an_unlistable_value_home_is_one_io_entry_in_check_json() {
    let mut scratch = Scratch::new("value-home-json", "values");
    if !scratch.lock("values") {
        eprintln!("{SKIP}");
        return;
    }
    let run = grund(&scratch.root, &["check", "--format", "json"]);
    assert_eq!(run.code, 2, "{}", run.shown());
    let records = run.io_records();
    assert_eq!(
        records.len(),
        1,
        "one failure is one io entry\n{}",
        run.shown()
    );
    assert_eq!(records[0]["path"], "values", "{}", run.shown());
    assert_eq!(records[0]["severity"], "error", "{}", run.shown());
    assert_eq!(
        records[0]["message"],
        format!("permission denied; {REMEDY}"),
        "{}",
        run.shown()
    );
}

/// The home links to a hidden directory the walk never descends into. The walk still reports the
/// link at the home's own path, in its own words (§FS-scan.scope.links.unresolvable.unopenable),
/// and the value pass's listing of that home is the same failure, whatever words each writes
/// around the reason, so the one line is the walk's (§FS-check.2.4.2). Both config versions refuse
/// a home linking out of the project, so no layout leaves a home to the pass alone.
#[test]
fn a_value_home_that_links_to_a_barred_target_is_the_walks_one_line() {
    let mut scratch = Scratch::new("value-home-linked", ".store/real");
    if !scratch.lock(".store/real") {
        eprintln!("{SKIP}");
        return;
    }
    let expected = format!("error: values: unreadable symlink: permission denied; {REMEDY}");
    let commands: &[&[&str]] = &[&["check"], &["list"], &["check", "--full"]];
    assert_none(
        &misreports(&scratch.root, commands, |lines| {
            lines == [expected.as_str()]
        }),
        commands.len(),
    );
    let run = grund(&scratch.root, &["check", "--format", "json"]);
    assert_eq!(run.code, 2, "{}", run.shown());
    let records = run.io_records();
    assert_eq!(records.len(), 1, "one io entry\n{}", run.shown());
    assert_eq!(records[0]["path"], "values", "{}", run.shown());
}
