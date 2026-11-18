//! #1452's reproducer, ported to the CLI harness: the re-preview a successful
//! write prints is a command that shows the written candidate again, so after a
//! write that dropped comments it carries `--drop-comments` (§FS-config-migrate.6.5).
//! Each test follows the line word for word: write, `git restore`, run what it says.

#[path = "support/fixture_cleanup.rs"]
mod fixture_cleanup;
use fixture_cleanup::remove_fixture_tree;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const REPREVIEW: &str = "to see this candidate again: restore version 1 and run ";
/// The one comment in migrate-3-write's config with no single key to follow.
const LISTED: &str = "# What the walk reads besides the homes.\n";

/// A committed copy of `examples/migrate/migrate-3-write/repo`, so the advice's
/// `git restore grund.toml` has a version 1 to bring back.
struct Fixture(PathBuf);

impl Fixture {
    /// `listed` keeps the comment at grund.toml:28; without it nothing is listed.
    fn new(listed: bool) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "grund-1452-repreview-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        copy_tree(&example(), &root);
        let fixture = Self(root);
        if !listed {
            let config = fixture.config();
            let input = fs::read_to_string(&config).unwrap();
            assert_eq!(input.matches(LISTED).count(), 1, "{input}");
            fs::write(config, input.replace(LISTED, "")).unwrap();
        }
        fixture.git(&["init", "-q"]);
        fixture.git(&["add", "-A"]);
        fixture.git(&["commit", "-q", "-m", "version 1"]);
        fixture
    }

    fn config(&self) -> PathBuf {
        self.0.join("grund.toml")
    }

    fn grund(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .expect("run grund")
    }

    fn git(&self, args: &[&str]) {
        let out = Command::new("git")
            .args([
                "-c",
                "user.name=grund-test",
                "-c",
                "user.email=test@example.invalid",
            ])
            .args([
                "-c",
                "commit.gpgsign=false",
                "-c",
                "core.hooksPath=/dev/null",
            ])
            .args(args)
            .current_dir(&self.0)
            .output()
            .expect("run git");
        assert!(out.status.success(), "git {args:?}: {}", text(&out.stderr));
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        remove_fixture_tree(&self.0, "remove fixture");
    }
}

fn example() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/migrate/migrate-3-write/repo")
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

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// Writes with `flags`, restores version 1 the way the advice says, and runs the
/// command the re-preview line prints, word for word with this binary as `grund`.
/// That run must exit 0 and print exactly the bytes the write left on disk.
/// Returns the write's stderr and the printed command.
fn round_trip(fixture: &Fixture, flags: &[&str]) -> (String, String) {
    let version_1 = fs::read(fixture.config()).unwrap();
    let mut args = vec!["config", "migrate", "--to", "2", "--write"];
    args.extend_from_slice(flags);
    let write = fixture.grund(&args);
    let stderr = text(&write.stderr);
    assert_eq!(
        write.status.code(),
        Some(0),
        "the write must succeed: {stderr}"
    );
    let written = fs::read(fixture.config()).unwrap();
    assert_ne!(
        written, version_1,
        "the write must replace version 1: {stderr}"
    );

    let printed: Vec<&str> = stderr
        .lines()
        .filter_map(|line| line.strip_prefix(REPREVIEW))
        .collect();
    let [command] = printed[..] else {
        panic!("expected exactly one re-preview line: {stderr}");
    };
    let command = command.to_owned();
    let words: Vec<&str> = command.split(' ').collect();
    assert_eq!(
        words[0], "grund",
        "the re-preview names a grund command: {command}"
    );

    fixture.git(&["restore", "grund.toml"]);
    assert_eq!(fs::read(fixture.config()).unwrap(), version_1, "restored");
    let again = fixture.grund(&words[1..]);
    let again_stderr = text(&again.stderr);
    assert_eq!(
        again.status.code(),
        Some(0),
        "`{command}`, run after git restore grund.toml as the write said, must show the \
         candidate again, not refuse; its stdout was {} bytes and its stderr:\n{again_stderr}",
        again.stdout.len()
    );
    assert!(
        again.stdout == written,
        "`{command}` must print byte for byte the candidate that was written\n\
         --- written grund.toml:\n{}\n--- stdout of `{command}`:\n{}",
        text(&written),
        text(&again.stdout)
    );
    (stderr, command)
}

/// §FS-config-migrate.6.5, §FS-config-migrate.1.3: the plain preview of this config
/// refuses on its listed comment and withholds the candidate, so the re-preview after
/// a write that dropped it must name `--drop-comments` to show what was written.
#[test]
#[ignore = "red until #1452 makes the re-preview carry --drop-comments after a write that dropped comments; implement removes this"]
fn repreview_after_dropping_comments_shows_the_written_candidate() {
    let fixture = Fixture::new(true);
    let preview = fixture.grund(&["config", "migrate", "--to", "2"]);
    let preview_stderr = text(&preview.stderr);
    assert_eq!(preview.status.code(), Some(1), "{preview_stderr}");
    assert!(
        preview.stdout.is_empty(),
        "a comment-refused preview withholds the candidate"
    );
    assert!(
        preview_stderr.contains("comments with no single key to follow"),
        "{preview_stderr}"
    );

    let (stderr, command) = round_trip(&fixture, &["--drop-comments"]);
    assert!(
        stderr.contains("dropped the comments at grund.toml:28 (version 1 lines)\n"),
        "{stderr}"
    );
    assert_eq!(command, "grund config migrate --to 2 --drop-comments");
}

/// §FS-config-migrate.6.5: `--full` never follows into the re-preview, because a
/// `--drop-comments` preview already prints the candidate (§FS-config-migrate.1.3).
#[test]
#[ignore = "red until #1452 makes the re-preview carry --drop-comments after a write that dropped comments; implement removes this"]
fn repreview_after_full_write_dropping_comments_names_drop_comments_but_not_full() {
    let fixture = Fixture::new(true);
    let (stderr, command) = round_trip(&fixture, &["--full", "--drop-comments"]);
    assert!(
        stderr.contains("dropped the comments at grund.toml:28 (version 1 lines)\n"),
        "{stderr}"
    );
    assert_eq!(command, "grund config migrate --to 2 --drop-comments");
}

/// §FS-config-migrate.6.5: a write that dropped nothing keeps the plain re-preview,
/// given `--drop-comments` or not, because the plain preview shows its candidate.
#[test]
fn repreview_after_dropping_nothing_stays_plain() {
    for flags in [&[][..], &["--drop-comments"][..]] {
        let fixture = Fixture::new(false);
        let (stderr, command) = round_trip(&fixture, flags);
        assert!(
            !stderr.contains("dropped the comments"),
            "{flags:?}: {stderr}"
        );
        assert_eq!(command, "grund config migrate --to 2", "{flags:?}");
    }
}
