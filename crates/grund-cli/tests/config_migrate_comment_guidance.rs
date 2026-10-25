//! #1004: listed comments explain their mapping and how to preserve the prose
//! (§FS-config-migrate.2.2), without changing carrying or consent (§FS-config-migrate.4.4).

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const COMMENT: &str = "# Everything the walk reads besides the homes.";
const V1: &str = "grund_config_version = 1
project_name = \"guidance\" # carried project comment
[reference]
marker = \"§\"
conversation = \"link\"
inline_note_suggested_lines = 2
warn_on_suggested = true
[id]
format = \"{kind}-{slug}\"
named_sections = true
[[kinds]]
kind = \"FS\"
file = \"requirements.md\"
[scan]
include = [\"requirements.md\", \"docs\", \"src\"]
extensions = [\"md\", \"rs\"]
comment_prefixes = [\"//\", \"#\"]
";

struct Fixture(PathBuf);

impl Fixture {
    fn new(input: &str) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "grund-1004-guidance-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        for folder in ["docs", "src"] {
            fs::create_dir_all(root.join(folder)).unwrap();
        }
        fs::write(root.join("requirements.md"), "# FS-login: Login\n").unwrap();
        fs::write(root.join("grund.toml"), input).unwrap();
        let git = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(git.status.success(), "{git:?}");
        Self(root)
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(["config", "migrate", "--to", "2"])
            .args(args)
            .current_dir(&self.0)
            .output()
            .unwrap()
    }

    fn config(&self) -> String {
        fs::read_to_string(self.0.join("grund.toml")).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn above(needle: &str) -> String {
    assert_eq!(V1.matches(needle).count(), 1);
    V1.replacen(needle, &format!("{COMMENT}\n{needle}"), 1)
}

fn listing<'a>(stderr: &'a str, place: &str) -> &'a str {
    let entries: Vec<_> = stderr
        .lines()
        .filter(|line| line.starts_with("  grund.toml:") && line.contains(place))
        .collect();
    assert_eq!(
        entries.len(),
        1,
        "expected one entry for {place}:\n{stderr}"
    );
    entries[0]
}

fn contains_all(line: &str, expected: &[&str], failures: &mut Vec<String>) {
    for text in expected {
        if !line.contains(text) {
            failures.push(format!("entry lacks {text:?}: {line}"));
        }
    }
}

/// Exercise the four report-producing modes; compact success is already pinned
/// by config_migrate_comments::carried_comment_warning_keeps_split_comment_drop_policy.
fn guidance_failures(input: &str, place: &str, reason: &str, destinations: &[&str]) -> Vec<String> {
    let mut failures = Vec::new();
    let mut candidate = None;
    for args in [
        vec![],
        vec!["--write"],
        vec!["--drop-comments"],
        vec!["--write", "--drop-comments", "--full"],
    ] {
        let fixture = Fixture::new(input);
        let output = fixture.run(&args);
        let stderr = String::from_utf8(output.stderr).unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        let consent = args.contains(&"--drop-comments");
        let wrote = args.contains(&"--write") && consent;
        assert_eq!(
            output.status.code(),
            Some(if consent { 0 } else { 1 }),
            "{stderr}"
        );
        assert!(
            !stderr.contains("refused:"),
            "fixture must be translatable:\n{stderr}"
        );
        assert!(
            stderr.contains("compared: the records are equal"),
            "{stderr}"
        );
        if args == ["--write"] {
            assert!(stdout.is_empty(), "refused write has no candidate stdout");
        } else {
            assert!(stdout.contains("[schema.kinds.FS]"), "{stdout}");
            assert!(
                !stdout.contains(COMMENT),
                "listed comments must not be auto-carried"
            );
            assert!(stdout.contains("# carried project comment"), "{stdout}");
            if let Some(expected) = &candidate {
                assert_eq!(&stdout, expected, "consent must not alter candidate bytes");
            } else {
                candidate = Some(stdout.clone());
            }
        }
        assert_eq!(fixture.config(), if wrote { &stdout } else { input });
        let entry = listing(&stderr, place);
        // Inspect the entry, never a remote mapped: line that already had the answer.
        contains_all(entry, &[reason, "--write --drop-comments"], &mut failures);
        contains_all(entry, destinations, &mut failures);
        let lower = entry.to_lowercase();
        if !(lower.contains("retain") || lower.contains("save") || lower.contains("keep"))
            || !lower.contains("re-add")
            || !lower.contains("after")
        {
            failures.push(format!(
                "entry lacks save / authorize / re-add-after-write remedy: {entry}"
            ));
        }
    }
    failures
}

fn assert_guidance(input: &str, place: &str, reason: &str, destinations: &[&str]) {
    let failures = guidance_failures(input, place, reason, destinations);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn include_comment_names_mixed_destinations_and_preservation() {
    assert_guidance(
        &above("include ="),
        "above [scan] include",
        "split",
        &[
            "[schema.kinds.code]",
            "folders",
            "docs",
            "src",
            "[schema.kinds.FS]",
            "requirements.md",
        ],
    );
}

#[test]
fn scan_header_comment_names_actual_destinations_and_preservation() {
    assert_guidance(
        &above("[scan]"),
        "above [scan]",
        "split",
        &[
            "[schema.kinds.code]",
            "folders",
            "docs",
            "src",
            "[schema.kinds.FS]",
            "requirements.md",
            "[schema.sources]",
            "languages",
            "extensions",
        ],
    );
}

#[test]
fn source_key_comments_name_their_actual_definitions() {
    let mut failures = Vec::new();
    for (key, targets) in [
        (
            "extensions",
            vec![
                "[schema.sources]",
                "languages",
                "extensions",
                "v1_markdown",
                "v1_c_family",
            ],
        ),
        (
            "comment_prefixes",
            vec![
                "[schema.sources.definitions.v1_markdown]",
                "[schema.sources.definitions.v1_c_family]",
                "comment_prefixes",
            ],
        ),
    ] {
        failures.extend(guidance_failures(
            &above(&format!("{key} =")),
            &format!("above [scan] {key}"),
            "split",
            &targets,
        ));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn other_split_headers_name_only_actual_destinations() {
    let mut failures = Vec::new();
    for (header, targets, absent) in [
        (
            "reference",
            vec![
                "[schema]",
                "marker",
                "[presentation]",
                "conversation",
                "[schema.notes.lines]",
                "warn",
            ],
            "[schema.notes.layout]",
        ),
        ("id", vec!["[schema]", "id_format"], "named_sections"),
    ] {
        let input = above(&format!("[{header}]"));
        let fixture = Fixture::new(&input);
        let out = fixture.run(&[]);
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert!(
            !listing(&stderr, &format!("above [{header}]")).contains(absent),
            "{stderr}"
        );
        failures.extend(guidance_failures(
            &input,
            &format!("above [{header}]"),
            "split",
            &targets,
        ));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn merged_note_keys_name_the_shared_destination() {
    let mut failures = Vec::new();
    for key in ["inline_note_suggested_lines", "warn_on_suggested"] {
        failures.extend(guidance_failures(
            &above(&format!("{key} =")),
            &format!("above [reference] {key}"),
            "merg",
            &["[schema.notes.lines]", "warn"],
        ));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn no_destination_comments_get_truthful_manual_guidance() {
    let cases = [
        (format!("{V1}\n{COMMENT}\n\n"), "standing alone", "attach"),
        (
            format!("{V1}\n[output]\n{COMMENT}\nformat = \"text\"\n"),
            "above [output] format",
            "drop",
        ),
        (
            V1.replace(
                "file = \"requirements.md\"",
                &format!("file = \"requirements.md\"\n{COMMENT}\nvalues = false"),
            ),
            "above [[kinds]] values",
            "omit",
        ),
        (
            V1.replace(
                "[[kinds]]\nkind = \"FS\"",
                "[[kinds]] # first\nkind = \"FS\" # second",
            ),
            "inside [[kinds]] kind",
            "trailing",
        ),
    ];
    let mut failures = Vec::new();
    for (input, place, reason) in cases {
        let fixture = Fixture::new(&input);
        let out = fixture.run(&["--write"]);
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert_eq!(out.status.code(), Some(1), "{stderr}");
        assert!(!stderr.contains("refused:"), "{stderr}");
        assert_eq!(fixture.config(), input);
        let entry = listing(&stderr, place);
        contains_all(
            entry,
            &[reason, "manual", "--write --drop-comments"],
            &mut failures,
        );
        if place.contains("kind") && reason == "trailing" {
            contains_all(entry, &["[schema.kinds.FS]"], &mut failures);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn unavailable_candidate_does_not_invent_a_destination() {
    let input = above("include =").replace("\"docs\", \"src\"", "\"missing\"");
    let fixture = Fixture::new(&input);
    let mut failures = Vec::new();
    for args in [vec!["--write"], vec!["--write", "--drop-comments"]] {
        let out = fixture.run(&args);
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert_eq!(out.status.code(), Some(1), "{stderr}");
        assert!(out.stdout.is_empty());
        assert!(stderr.contains("refused:"), "{stderr}");
        assert!(stderr.contains("repair: create missing"), "{stderr}");
        assert_eq!(fixture.config(), input);
        let entry = listing(&stderr, "above [scan] include");
        contains_all(entry, &["unavailable", "repair", "preview"], &mut failures);
        assert!(
            !entry.contains("[schema.kinds.code]"),
            "unresolved path has no code destination: {entry}"
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn singular_and_plural_refusals_point_to_guidance() {
    let mut failures = Vec::new();
    for plural in [false, true] {
        let mut input = above("include =");
        if plural {
            input = input.replace("extensions =", &format!("{COMMENT}\nextensions ="));
        }
        let fixture = Fixture::new(&input);
        let out = fixture.run(&["--write"]);
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert_eq!(out.status.code(), Some(1), "{stderr}");
        assert_eq!(fixture.config(), input);
        let count = if plural {
            "2 comments have"
        } else {
            "1 comment has"
        };
        let expected = format!(
            "not written: {count} no single key to follow (follow the comment guidance above); grund.toml is unchanged"
        );
        if stderr.lines().last() != Some(expected.as_str()) {
            failures.push(format!(
                "expected closing refusal: {expected}\nactual: {}",
                stderr.lines().last().unwrap_or("<missing>")
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
