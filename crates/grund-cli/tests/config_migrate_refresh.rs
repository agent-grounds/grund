//! Black-box migration follow-up (§FS-config-migrate.6.5): port the #831
//! reproducer, keeping successful config-only writes separate from advice.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const V1: &str = "grund_config_version = 1
project_name = \"demo\"

[reference]
strict = true

[id]
format = \"{kind}-{slug}\"
named_sections = true

[scan]
include = [\"requirements.md\", \"src\"]

[[kinds]]
kind = \"FS\"
file = \"requirements.md\"
";

const REMINDER: &str = "note: existing AGENTS.md managed instructions may still describe version 1; run grund init in the migrated project to refresh them";

struct Fixture(PathBuf);

impl Fixture {
    fn new(name: &str) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("grund-831-{name}-{}-{serial}", std::process::id()));
        fs::create_dir_all(dir.join("src")).expect("create fixture");
        let git = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&dir)
            .output()
            .expect("initialize fixture Git repository");
        assert!(git.status.success(), "git init: {}", text(&git.stderr));
        fs::write(dir.join("grund.toml"), V1).unwrap();
        fs::write(dir.join("requirements.md"), "# FS-login: Login\n").unwrap();
        fs::write(dir.join("src/a.py"), "# Implements \u{a7}FS-login\n").unwrap();
        Self(dir)
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .expect("run grund")
    }

    fn migrate(&self, write: bool) -> Output {
        if write {
            self.run(&["config", "migrate", "--to", "2", "--write"])
        } else {
            self.run(&["config", "migrate", "--to", "2"])
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove fixture");
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, dir: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.file_name().unwrap() == ".git" {
                continue;
            }
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(root).unwrap().into(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

/// §FS-config-migrate.6.4, §FS-config-migrate.6.5: the config alone changes,
/// compact stdout is empty, disk matches preview, and the reminder is last.
fn assert_successful_write(fixture: &Fixture) {
    let before = snapshot(&fixture.0);
    let preview = fixture.migrate(false);
    assert_eq!(preview.status.code(), Some(0), "{}", text(&preview.stderr));
    assert!(preview.stdout.starts_with(b"grund_config_version = 2\n"));
    assert_eq!(snapshot(&fixture.0), before, "preview is read-only");
    let out = fixture.migrate(true);
    let stderr = text(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "migration must succeed: {stderr}"
    );
    assert!(out.stdout.is_empty(), "compact success omits the candidate");
    assert_eq!(
        fs::read(fixture.0.join("grund.toml")).unwrap(),
        preview.stdout
    );
    let mut expected = before;
    expected.insert("grund.toml".into(), preview.stdout);
    assert_eq!(snapshot(&fixture.0), expected, "only the config may change");
    assert!(
        stderr.contains("note: the next grund init adds a `code` row"),
        "keep the distinct Project map note: {stderr}"
    );
    assert!(stderr.contains("wrote grund.toml: version 1"), "{stderr}");
    assert!(
        stderr.contains(
            "to see this candidate again: restore version 1 and run grund config migrate --to 2\n"
        ),
        "keep the restore guidance: {stderr}"
    );
    assert_eq!(
        stderr.lines().filter(|line| *line == REMINDER).count(),
        1,
        "missing AGENTS.md managed-instructions reminder after successful write:\n{stderr}"
    );
    assert_eq!(stderr.lines().last(), Some(REMINDER), "reminder is last");
}

/// §FS-config-migrate.6.5: existing v1 instructions survive unchanged, so the
/// successful write explicitly tells the user to refresh them with `grund init`.
#[test]
fn existing_v1_managed_block_gets_refresh_reminder() {
    let fixture = Fixture::new("existing-block");
    fs::write(
        fixture.0.join("grund.toml"),
        V1.replace("strict = true", "strict = false"),
    )
    .unwrap();
    let init = fixture.run(&["init", "--agents-md"]);
    assert_eq!(init.status.code(), Some(0), "{}", text(&init.stderr));
    let agents = fs::read_to_string(fixture.0.join("AGENTS.md")).unwrap();
    assert!(agents.contains("[reference] strict = false"), "{agents}");
    // The user repairs v1 before migrating; the previously written block stays.
    fs::write(fixture.0.join("grund.toml"), V1).unwrap();
    assert_successful_write(&fixture);
}

/// §FS-config-migrate.6.5: the qualified reminder is also emitted without an
/// AGENTS.md; it must not claim to have inspected a file or create one.
#[test]
fn absent_managed_block_gets_qualified_refresh_reminder() {
    let fixture = Fixture::new("absent-block");
    assert!(!fixture.0.join("AGENTS.md").exists());
    assert_successful_write(&fixture);
}

/// §FS-config-migrate.6.5: only an actual successful v1-to-v2 write earns the
/// reminder; previews, refusal gates and already-v2 no-ops do not.
#[test]
fn refresh_reminder_is_absent_without_a_successful_write() {
    let preview = Fixture::new("preview");
    let before = snapshot(&preview.0);
    let out = preview.migrate(false);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(!text(&out.stderr).contains(REMINDER));
    assert_eq!(snapshot(&preview.0), before);

    for (name, config, diagnostic) in [
        (
            "refused-setting",
            V1.replace("named_sections = true", "named_sections = false"),
            "refused:",
        ),
        (
            "refused-comments",
            format!("{V1}\n# unattached comment\n"),
            "comments with no single key",
        ),
        (
            "refused-guidance",
            format!("{V1}values = true\n"),
            "Declaration shapes",
        ),
    ] {
        let fixture = Fixture::new(name);
        fs::write(fixture.0.join("grund.toml"), config).unwrap();
        let before = snapshot(&fixture.0);
        let out = fixture.migrate(true);
        let stderr = text(&out.stderr);
        assert_eq!(out.status.code(), Some(1), "{name}: {stderr}");
        assert!(stderr.contains(diagnostic), "{name}: {stderr}");
        assert!(!stderr.contains(REMINDER), "{name}: {stderr}");
        assert!(out.stdout.is_empty());
        assert_eq!(snapshot(&fixture.0), before);
    }

    let twice = Fixture::new("already-v2");
    let first = twice.migrate(true);
    assert_eq!(first.status.code(), Some(0), "{}", text(&first.stderr));
    let before = snapshot(&twice.0);
    for write in [false, true] {
        let out = twice.migrate(write);
        assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
        assert!(out.stdout.is_empty());
        assert_eq!(
            text(&out.stderr),
            "grund.toml is already version 2; nothing to migrate\n"
        );
        assert_eq!(snapshot(&twice.0), before);
    }
}
