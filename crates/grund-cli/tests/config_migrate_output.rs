//! Black-box output contract for #907 (§FS-config-migrate.2.1,
//! §FS-config-migrate.2.2, §FS-config-migrate.6.5). The fixture and tree checks
//! port triage's run.sh; assertions use the CLI, without a second renderer.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

#[path = "config_migrate_output/outcomes.rs"]
mod outcomes;

const V1: &str = "grund_config_version = 1
project_name = \"demo\"

[reference]
strict = true

[id]
format = \"{kind}-{slug}\"
named_sections = true

[[kinds]]
kind = \"FS\"
file = \"requirements.md\"

[scan]
include = [\"requirements.md\", \"src\"]
";
const REMINDER: &str = "note: existing AGENTS.md managed instructions may still describe version 1; run grund init in the migrated project to refresh them";
const PROJECT_MAP: &str = "note: the next grund init adds a `code` row to the AGENTS.md Project map, because the complement's places are now listed; grund check does not compare that table";
const REPREVIEW: &str =
    "to see this candidate again: restore version 1 and run grund config migrate --to 2";

struct Fixture(PathBuf);

impl Fixture {
    fn new(input: &str) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "grund-907-output-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(dir.join("src")).unwrap();
        let git = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&dir)
            .output()
            .unwrap();
        assert!(git.status.success(), "{}", text(&git.stderr));
        fs::write(dir.join("grund.toml"), input).unwrap();
        fs::write(dir.join("requirements.md"), "# FS-login: Login\n").unwrap();
        fs::write(dir.join("src/auth.py"), "# Implements \u{a7}FS-login\n").unwrap();
        Self(dir)
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .unwrap()
    }

    fn migrate(&self, flags: &[&str]) -> Output {
        let mut args = vec!["config", "migrate", "--to", "2"];
        args.extend_from_slice(flags);
        self.run(&args)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap()
}

fn status(out: &Output, code: i32) {
    assert_eq!(out.status.code(), Some(code), "{}", text(&out.stderr));
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

fn outcome(input: &str, config: &str) -> String {
    format!(
        "wrote {config}: version 1 ({} lines) replaced by version 2",
        input.lines().count()
    )
}

fn advice(config: &str) -> Vec<String> {
    vec![
        format!("to get version 1 back: it is in version control, e.g. git restore {config}"),
        REPREVIEW.into(),
        REMINDER.into(),
    ]
}

fn lines(bytes: &[u8]) -> Vec<String> {
    text(bytes).lines().map(str::to_owned).collect()
}

fn annotated_v1() -> String {
    V1.replace(
        "strict = true",
        "strict = true\ninline_style = \"citation-only\" # citation-with-note | citation-only",
    ) + "\n# standalone one\n# standalone two\n\n[output]\nformat = \"json\"\nrelative_paths = false\ncolor = \"always\"\n\n# final standalone\n"
}

fn dropped_ranges(input: &str) -> String {
    let first = input.lines().position(|l| l == "# standalone one").unwrap() + 1;
    format!(
        "dropped the comments at grund.toml:{first}-{} and grund.toml:{} (version 1 lines)",
        first + 1,
        input.lines().count()
    )
}

fn assert_complete_report(preview: &Output) {
    let stderr = text(&preview.stderr);
    for prefix in ["mapped:", "written:", "compared:"] {
        assert!(
            stderr.lines().any(|line| line.starts_with(prefix)),
            "missing {prefix}: {stderr}"
        );
    }
}

/// §FS-config-migrate.2.1, §FS-config-migrate.6.4, §FS-config-migrate.6.5:
/// issue fixture, including existing managed instructions, preview and tree.
#[test]
fn compact_write_has_no_candidate_and_only_changes_config() {
    for init in [true, false] {
        let fixture = Fixture::new(V1);
        if init {
            status(&fixture.run(&["init"]), 0);
        }
        let before = snapshot(&fixture.0);
        let preview = fixture.migrate(&[]);
        status(&preview, 0);
        assert_complete_report(&preview);
        assert!(preview.stdout.starts_with(b"grund_config_version = 2\n"));
        assert_eq!(snapshot(&fixture.0), before, "preview is read-only");
        let write = fixture.migrate(&["--write"]);
        status(&write, 0);
        assert_eq!(
            fs::read(fixture.0.join("grund.toml")).unwrap(),
            preview.stdout
        );
        let mut expected = before;
        expected.insert("grund.toml".into(), preview.stdout.clone());
        assert_eq!(
            snapshot(&fixture.0),
            expected,
            "only config changes, no init"
        );
        assert!(
            write.stdout.is_empty(),
            "successful write replayed {} candidate bytes ({} lines); preview equality: {}",
            write.stdout.len(),
            text(&write.stdout).lines().count(),
            write.stdout == preview.stdout
        );
    }
}

/// §FS-config-migrate.6.5: exact five-line report independently pins stderr,
/// so candidate suppression alone cannot satisfy the issue.
#[test]
fn compact_write_report_is_exactly_five_lines() {
    let fixture = Fixture::new(V1);
    let write = fixture.migrate(&["--write"]);
    status(&write, 0);
    let actual = lines(&write.stderr);
    let replayed: Vec<_> = actual
        .iter()
        .filter(|line| {
            ["mapped:", "written:", "compared:"]
                .iter()
                .any(|prefix| line.starts_with(prefix))
        })
        .collect();
    assert!(
        replayed.is_empty(),
        "successful write replayed {} detail lines:\n{}",
        replayed.len(),
        text(&write.stderr)
    );
    let mut expected = vec![outcome(V1, "grund.toml"), PROJECT_MAP.into()];
    expected.extend(advice("grund.toml"));
    assert_eq!(actual, expected, "compact stderr order and exact contents");
    assert_eq!(actual.iter().filter(|line| *line == REMINDER).count(), 1);
    assert_eq!(actual.last().map(String::as_str), Some(REMINDER));
}

/// §FS-config-migrate.6.5, §FS-config-migrate.4.1.1: retain every notice and
/// carried-comment warning, but list dropped original ranges only once.
#[test]
fn compact_write_orders_multiple_drops_notices_and_carried_warning() {
    let input = annotated_v1();
    let fixture = Fixture::new(&input);
    let before = snapshot(&fixture.0);
    let preview = fixture.migrate(&["--drop-comments"]);
    status(&preview, 0);
    let preview_lines = lines(&preview.stderr);
    let notices: Vec<_> = preview_lines
        .iter()
        .filter(|l| l.starts_with("changed:") || l.starts_with("dropped:"))
        .cloned()
        .collect();
    assert_eq!(
        notices.len(),
        4,
        "three output notices and carried-comment warning: {}",
        text(&preview.stderr)
    );
    assert!(
        notices
            .iter()
            .any(|l| l.contains("[schema.notes.text] must") && l.contains("comment"))
    );
    let write = fixture.migrate(&["--write", "--drop-comments"]);
    status(&write, 0);
    let mut expected = vec![outcome(&input, "grund.toml"), dropped_ranges(&input)];
    expected.extend(notices);
    expected.push(PROJECT_MAP.into());
    expected.extend(advice("grund.toml"));
    assert_eq!(lines(&write.stderr), expected);
    assert!(write.stdout.is_empty());
    let mut after = before;
    after.insert("grund.toml".into(), preview.stdout);
    assert_eq!(snapshot(&fixture.0), after);
    assert!(
        text(&fs::read(fixture.0.join("grund.toml")).unwrap())
            .contains("# citation-with-note | citation-only")
    );
}

/// §FS-config-migrate.8, §FS-config-migrate.6.5: member commands survive
/// compact rendering between notices and the Project map note, without writes.
#[test]
fn compact_workspace_keeps_member_commands_in_order() {
    for full in [false, true] {
        let input = format!("{V1}\n[workspace]\nmembers = [\"member\"]\n");
        let fixture = Fixture::new(&input);
        fs::create_dir_all(fixture.0.join("member/src")).unwrap();
        fs::write(
            fixture.0.join("member/grund.toml"),
            V1.replace("project_name = \"demo\"", "project_name = \"member\""),
        )
        .unwrap();
        fs::write(
            fixture.0.join("member/requirements.md"),
            "# FS-other: Other\n",
        )
        .unwrap();
        let before = snapshot(&fixture.0);
        let preview = fixture.migrate(&[]);
        status(&preview, 0);
        let flags = if full {
            vec!["--write", "--full"]
        } else {
            vec!["--write"]
        };
        let write = fixture.migrate(&flags);
        status(&write, 0);
        let mut expected = vec![outcome(&input, "grund.toml"), "member: member/grund.toml is version 1; migrate it with grund config migrate --to 2 member".into(), PROJECT_MAP.into()];
        expected.extend(advice("grund.toml"));
        if full {
            expected = lines(&preview.stderr);
            expected[0] = "grund.toml: version 1 → 2, --write".into();
            expected.push(outcome(&input, "grund.toml"));
            expected.extend(advice("grund.toml"));
        }
        assert_eq!(lines(&write.stderr), expected);
        if full {
            assert_eq!(write.stdout, preview.stdout);
        } else {
            assert!(write.stdout.is_empty());
        }
        let mut after = before;
        after.insert("grund.toml".into(), preview.stdout);
        assert_eq!(snapshot(&fixture.0), after);
    }
}

/// §FS-config-migrate.1.3, §FS-config-migrate.2.2: full writes restore the
/// entire legacy report in its order, not just the suppressed stdout.
#[test]
fn full_write_restores_candidate_and_complete_report_with_path_and_comments() {
    for base in ["project", "invocation"] {
        let input = annotated_v1();
        let fixture = Fixture::new(&input);
        let before = snapshot(&fixture.0);
        let flags = ["src", "--drop-comments", "--path-base", base];
        let preview = fixture.migrate(&flags);
        status(&preview, 0);
        assert_complete_report(&preview);
        let mut flags = flags.to_vec();
        flags.extend(["--write", "--full"]);
        let write = fixture.migrate(&flags);
        status(&write, 0);
        assert_eq!(write.stdout, preview.stdout);
        let mut expected = lines(&preview.stderr);
        expected[0] = "grund.toml: version 1 → 2, --write".into();
        expected.push(outcome(&input, "grund.toml"));
        expected.push(dropped_ranges(&input));
        expected.extend(advice("grund.toml"));
        assert_eq!(
            lines(&write.stderr),
            expected,
            "complete legacy stderr and order"
        );
        let mut after = before;
        after.insert("grund.toml".into(), preview.stdout);
        assert_eq!(snapshot(&fixture.0), after);
    }
}

/// §FS-config-migrate.1.3: full is inert on successful and comment-refused
/// previews, including PATH, --drop-comments and both path-base values.
#[test]
fn full_previews_preserve_both_streams_and_tree() {
    for base in ["project", "invocation"] {
        for drop in [false, true] {
            let fixture = Fixture::new(&format!("{V1}\n# standalone\n"));
            let before = snapshot(&fixture.0);
            let mut flags = vec!["src", "--path-base", base];
            if drop {
                flags.push("--drop-comments");
            }
            let preview = fixture.migrate(&flags);
            status(&preview, if drop { 0 } else { 1 });
            flags.push("--full");
            let full = fixture.migrate(&flags);
            status(&full, if drop { 0 } else { 1 });
            assert_eq!(full.stdout, preview.stdout);
            assert_eq!(full.stderr, preview.stderr);
            assert!(!text(&full.stderr).contains(REMINDER));
            assert_eq!(snapshot(&fixture.0), before);
        }
    }
}

/// §FS-config-migrate.2.2, §FS-config-migrate.5.3: compact selection must
/// not abridge failed setting, comment or generated-guidance gate diagnostics.
#[test]
fn full_refused_writes_keep_complete_diagnostics_and_original_tree() {
    for (input, diagnostic) in [
        (
            V1.replace("named_sections = true", "named_sections = false"),
            "repair:",
        ),
        (
            format!("{V1}\n# standalone\n"),
            "comments with no single key",
        ),
        (
            V1.replace(
                "file = \"requirements.md\"",
                "file = \"requirements.md\"\nvalues = true",
            ),
            "Declaration shapes",
        ),
    ] {
        let fixture = Fixture::new(&input);
        let before = snapshot(&fixture.0);
        let normal = fixture.migrate(&["--write"]);
        status(&normal, 1);
        assert!(
            text(&normal.stderr).contains(diagnostic),
            "{}",
            text(&normal.stderr)
        );
        let full = fixture.migrate(&["--write", "--full"]);
        status(&full, 1);
        assert!(normal.stdout.is_empty() && full.stdout.is_empty());
        assert_eq!(full.stderr, normal.stderr);
        assert!(!text(&full.stderr).contains(REMINDER));
        assert_eq!(snapshot(&fixture.0), before);
    }
}

/// §FS-config-migrate.1.2, §FS-config-migrate.1.3: accepting full must not
/// bypass required target validation or make --format a supported flag.
#[test]
fn full_keeps_to_required_and_format_rejected() {
    let fixture = Fixture::new(V1);
    let before = snapshot(&fixture.0);
    for (args, diagnostic) in [
        (
            vec!["config", "migrate", "--full"],
            "config migrate needs --to 2",
        ),
        (
            vec!["config", "migrate", "--full", "--to", "1"],
            "config migrate converts to version 2 only",
        ),
        (
            vec![
                "config", "migrate", "--full", "--to", "2", "--format", "json",
            ],
            "unknown flag `--format`",
        ),
    ] {
        let out = fixture.run(&args);
        status(&out, 2);
        assert!(
            text(&out.stderr).contains(diagnostic),
            "{}",
            text(&out.stderr)
        );
        assert!(out.stdout.is_empty());
        assert_eq!(snapshot(&fixture.0), before);
    }
}
