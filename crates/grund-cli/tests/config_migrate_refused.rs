//! #939's reproducer, ported to the CLI harness: refused writes explain every
//! blocking cause without completed-write claims (§FS-config-migrate.2.2).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const COMMENT: &str = "1 comment has no single key to follow (follow the comment guidance above)";

struct Fixture(PathBuf);

impl Fixture {
    fn new(comment_only: bool) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "grund-939-refused-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        copy_tree(&example().join("repo"), &root);
        let git = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(git.status.success(), "{git:?}");
        if comment_only {
            let config = root.join("grund.toml");
            let input = fs::read_to_string(&config).unwrap();
            assert!(input.contains("named_sections = false"));
            fs::write(
                config,
                input.replace("named_sections = false", "named_sections = true"),
            )
            .unwrap();
            fs::create_dir(root.join("e2e")).unwrap();
        }
        Self(root)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn example() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/migrate/migrate-1-refused")
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let destination = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &destination);
        } else {
            fs::copy(entry.path(), destination).unwrap();
        }
    }
}

fn assert_refused_write(comment_only: bool, full: bool) {
    let fixture = Fixture::new(comment_only);
    let config = fixture.0.join("grund.toml");
    let before = fs::read(&config).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_grund"));
    command
        .args(["config", "migrate", "--to", "2", "--write"])
        .current_dir(&fixture.0);
    if full {
        command.arg("--full");
    }
    let output = command.output().unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert_eq!(
        fs::read(config).unwrap(),
        before,
        "config must be byte-identical"
    );
    assert!(
        output.stdout.is_empty(),
        "refused write must have empty stdout"
    );
    assert!(
        stderr.contains("grund.toml:28  above [scan] include"),
        "{stderr}"
    );
    if comment_only {
        assert!(
            !stderr.contains("refused:"),
            "settings must be repaired: {stderr}"
        );
        assert!(
            stderr.contains("compared: the records are equal"),
            "{stderr}"
        );
    } else {
        assert_eq!(
            stderr.lines().filter(|l| l.starts_with("refused:")).count(),
            2
        );
        assert!(stderr.contains("repair: set [id] named_sections = true"));
        assert!(stderr.contains("repair: create e2e, or remove it from include"));
    }

    // Collect both presentation failures after proving the refusal setup,
    // so the red evidence shows the footer and completed-write claims together.
    let expected = if comment_only {
        format!("not written: {COMMENT}; grund.toml is unchanged")
    } else {
        format!(
            "not written: 2 refusals (follow the repairs above); {COMMENT}; grund.toml is unchanged"
        )
    };
    let mut failures = Vec::new();
    if stderr.lines().last() != Some(expected.as_str()) {
        failures.push(format!("expected closing line: {expected}"));
    }
    for line in stderr.lines() {
        if line.starts_with("written:") || line.starts_with("note: the next grund init") {
            failures.push(format!("completed-write claim on refused write: {line}"));
        }
    }
    if !comment_only {
        let recorded = fs::read_to_string(example().join("expected.write.stderr")).unwrap();
        if stderr != recorded {
            failures.push("report differs from the example's expected.write.stderr".into());
        }
    }
    assert!(
        failures.is_empty(),
        "{}\nactual stderr:\n{stderr}",
        failures.join("\n")
    );
}

#[test]
fn refused_example_write_explains_settings_and_comment() {
    assert_refused_write(false, false);
}

#[test]
fn refused_example_full_write_explains_settings_and_comment() {
    assert_refused_write(false, true);
}

#[test]
fn refused_comment_only_write_explains_comment_without_completed_write_claims() {
    assert_refused_write(true, false);
}

#[test]
fn refused_comment_only_full_write_explains_comment_without_completed_write_claims() {
    assert_refused_write(true, true);
}
