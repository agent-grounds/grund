//! Black-box permission regressions for §FS-check.2.4.1. Port the intake's
//! file/directory comparisons across commands; modes require runtime fixtures.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

const CONFIG: &str = "grund_config_version = 2\nproject_name = \"p\"\n\n\
    [schema]\nslug_pattern = \"[a-z][a-z-]*\"\n\n\
    [schema.kinds.FS]\nfolders = [\"docs\"]\n";
const REASON: &str = "permission denied";
const MESSAGE: &str = "permission denied; make the path readable, or keep it out of the scan";

struct Fixture {
    root: PathBuf,
    locked: Vec<PathBuf>,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1223-{name}-{}", std::process::id()));
        fs::create_dir(&root).expect("create fresh fixture");
        fs::create_dir(root.join("docs")).unwrap();
        fs::write(root.join("grund.toml"), CONFIG).unwrap();
        fs::write(root.join("docs/FS-b.md"), "# FS-b: B\n\nPlain text.\n").unwrap();
        // Keep index diagnostics from obscuring the permission regression.
        fs::write(
            root.join("docs/README.md"),
            "[\u{a7}FS-b](FS-b.md#fs-b-b)\n",
        )
        .unwrap();
        Self {
            root,
            locked: Vec::new(),
        }
    }

    fn lock(&mut self, path: &str, directory: bool) {
        let path = self.root.join(path);
        if directory {
            fs::create_dir(&path).unwrap();
        } else {
            fs::write(&path, "# FS-a: A\n\nPlain text.\n").unwrap();
        }
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
        self.locked.push(path.clone());
        let error = if directory {
            fs::read_dir(&path).err()
        } else {
            fs::read(&path).err()
        };
        assert_eq!(
            error
                .expect("mode 000 must deny access; run as an unprivileged user")
                .kind(),
            std::io::ErrorKind::PermissionDenied
        );
    }

    fn run(&self, args: &[&str]) -> String {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.root)
            .output()
            .expect("run grund");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert_eq!(output.status.code(), Some(2), "grund {args:?}: {stderr}");
        stderr
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        for path in &self.locked {
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o755));
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn reason<'a>(stderr: &'a str, path: &str) -> &'a str {
    let prefix = format!("{path}: ");
    let message = stderr
        .lines()
        .filter_map(|line| line.strip_prefix("error: "))
        .map(|line| line.strip_prefix("nothing was rewritten: ").unwrap_or(line))
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap_or_else(|| panic!("missing scan failure for {path}:\n{stderr}"));
    message.split("; ").next().unwrap()
}

fn io_records(stderr: &str) -> Vec<serde_json::Value> {
    stderr
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("NDJSON record"))
        .filter(|record| record["code"] == "io")
        .collect()
}

#[test]
fn check_text_gives_file_and_directory_one_permission_reason() {
    let mut fixture = Fixture::new("check-text");
    fixture.lock("docs/FS-a.md", false);
    fixture.lock("docs/locked", true);
    let stderr = fixture.run(&["check"]);
    let file = reason(&stderr, "docs/FS-a.md");
    let directory = reason(&stderr, "docs/locked");
    assert_eq!((file, directory), (REASON, REASON), "{stderr}");
    for path in ["docs/FS-a.md", "docs/locked"] {
        assert!(
            stderr
                .lines()
                .any(|line| line == format!("error: {path}: {MESSAGE}"))
        );
    }
}

#[test]
fn check_json_gives_file_and_directory_one_permission_reason() {
    let mut fixture = Fixture::new("check-json");
    fixture.lock("docs/FS-a.md", false);
    fixture.lock("docs/locked", true);
    let stderr = fixture.run(&["check", "--format", "json"]);
    let records = io_records(&stderr);
    assert_eq!(records.len(), 2, "{stderr}");
    for path in ["docs/FS-a.md", "docs/locked"] {
        let record = records
            .iter()
            .find(|record| record["path"] == path)
            .unwrap();
        assert_eq!(record["severity"], "error");
        assert!(record["line"].is_null());
        assert_eq!(record["message"], MESSAGE, "{stderr}");
    }
}

// Point queries stop at the first failure: use the intake's separate trees.
macro_rules! command_regression {
    ($name:ident, $args:expr) => {
        #[test]
        fn $name() {
            let mut file = Fixture::new(concat!(stringify!($name), "-file"));
            file.lock("docs/FS-a.md", false);
            let mut directory = Fixture::new(concat!(stringify!($name), "-dir"));
            directory.lock("docs/locked", true);
            let file_stderr = file.run($args);
            let dir_stderr = directory.run($args);
            assert_eq!(
                (
                    reason(&file_stderr, "docs/FS-a.md"),
                    reason(&dir_stderr, "docs/locked")
                ),
                (REASON, REASON),
                "file:\n{file_stderr}directory:\n{dir_stderr}"
            );
        }
    };
}

command_regression!(list_uses_the_shared_permission_reason, &["list"]);
command_regression!(refs_uses_the_shared_permission_reason, &["refs", "FS-b"]);
command_regression!(cover_uses_the_shared_permission_reason, &["cover"]);
command_regression!(show_uses_the_shared_permission_reason, &["show", "FS-b"]);
command_regression!(fmt_uses_the_shared_permission_reason, &["fmt", "--check"]);
command_regression!(
    id_uses_the_shared_permission_reason,
    &["id", "FS", "new-thing"]
);

/// The home links to a hidden directory the walk never descends into, so the value pass is what
/// reports it and its line keeps the home's context. Where the walk reports the home too, the one
/// line is the walk's (§FS-check.2.4.2). This layout leaves the home to the pass only while the
/// walk reports a link whose target it cannot open at its walk root rather than at the link
/// (#1266); no layout does so by design. Once that is fixed the one line here is the walk's and
/// this test goes, the fetch test below keeping the pin of the home's context.
#[test]
fn value_home_read_retains_context_with_the_shared_permission_reason() {
    let mut fixture = Fixture::new("value-home");
    fs::write(
        fixture.root.join("grund.toml"),
        format!("{CONFIG}\n[schema.kinds.CONST]\nfolders = [\"values\"]\nform = \"value\"\n"),
    )
    .unwrap();
    fs::create_dir(fixture.root.join(".store")).unwrap();
    fixture.lock(".store/home", true);
    std::os::unix::fs::symlink(".store/home", fixture.root.join("values")).unwrap();
    let stderr = fixture.run(&["check"]);
    let expected =
        "read value home: permission denied; make the path readable, or keep it out of the scan";
    assert!(
        stderr
            .lines()
            .any(|line| line == format!("error: values: {expected}")),
        "{stderr}"
    );
    let stderr = fixture.run(&["check", "--format", "json"]);
    assert!(
        io_records(&stderr)
            .iter()
            .any(|record| record["path"] == "values" && record["message"] == expected),
        "{stderr}"
    );
}

/// Fetch's selected-home discovery runs no walk, so its failure to list the home is its own and
/// keeps the home's context around the shared reason (§FS-check.2.4.1, §FS-fetch.5.1). No walk
/// report can take this line over, so it is the durable pin of that context.
#[test]
fn fetch_of_an_unreadable_value_home_retains_context_with_the_shared_permission_reason() {
    let mut fixture = Fixture::new("fetch-value-home");
    fs::write(
        fixture.root.join("grund.toml"),
        format!(
            "{CONFIG}\n[schema.kinds.CONST]\nfolders = [\"values\"]\nform = \"value\"\n\
             fetch = \"scripts/fetch-const\"\n"
        ),
    )
    .unwrap();
    let script = fixture.root.join("scripts/fetch-const");
    fs::create_dir(fixture.root.join("scripts")).unwrap();
    fs::write(
        &script,
        "#!/bin/sh\nprintf '%b' \"# $1: Fetched\\n\\n## 1. 99\\n\"\n",
    )
    .unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    fixture.lock("values", true);
    let stderr = fixture.run(&["fetch", "CONST-room-limit"]);
    let failures: Vec<_> = stderr
        .lines()
        .filter(|line| line.starts_with("error: "))
        .collect();
    assert_eq!(failures.len(), 1, "{stderr}");
    assert!(
        failures[0].starts_with("error: cannot read snapshot folder ")
            && failures[0].ends_with(&format!("values: read value home: {REASON}")),
        "{stderr}"
    );
}
