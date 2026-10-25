//! Refused-preview output for #1002 (§FS-config-migrate.1.3,
//! §FS-config-migrate.2.1, §FS-config-migrate.2.2): a refused preview closes
//! with the write's causes, and only `--full` brings back a candidate.

use super::*;

const COMMENT_CAUSE: &str =
    "1 comment has no single key to follow (follow the comment guidance above)";

/// The `<causes>` of a closing line, or `None` when no line closes that way.
fn causes<'a>(stderr: &'a str, prefix: &str, suffix: &str) -> Option<&'a str> {
    let last = stderr.lines().last()?;
    last.strip_prefix(prefix)?.strip_suffix(suffix)
}

/// §FS-config-migrate.1.3, §FS-config-migrate.2.1: a passing preview is the
/// same bytes with or without full; one only comments refuse prints its
/// candidate only under full, as --drop-comments previews it, plus the
/// closing line. PATH and both path-base values included.
#[test]
fn full_previews_preserve_both_streams_and_tree() {
    for base in ["project", "invocation"] {
        let fixture = Fixture::new(&format!("{V1}\n# standalone\n"));
        let before = snapshot(&fixture.0);
        let flags = ["src", "--path-base", base];
        let with = |extra: &[&'static str]| {
            let mut all = flags.to_vec();
            all.extend_from_slice(extra);
            fixture.migrate(&all)
        };
        let dropped = with(&["--drop-comments"]);
        status(&dropped, 0);
        let dropped_full = with(&["--drop-comments", "--full"]);
        status(&dropped_full, 0);
        assert_eq!(dropped_full.stdout, dropped.stdout);
        assert_eq!(dropped_full.stderr, dropped.stderr);

        let refused = with(&[]);
        status(&refused, 1);
        assert!(refused.stdout.is_empty(), "{}", text(&refused.stdout));
        let compact = text(&refused.stderr);
        assert!(
            !compact.lines().any(|l| l.starts_with("written: ")),
            "{compact}"
        );
        assert!(!compact.contains(PROJECT_MAP) && !compact.contains(REMINDER));
        assert_eq!(
            causes(&compact, "would not write: ", ""),
            Some(COMMENT_CAUSE)
        );

        let full = with(&["--full"]);
        status(&full, 1);
        assert_eq!(full.stdout, dropped.stdout);
        let mut expected = lines(&dropped.stderr);
        expected.push(format!("would not write: {COMMENT_CAUSE}"));
        assert_eq!(lines(&full.stderr), expected);
        assert_eq!(snapshot(&fixture.0), before);
    }
}

/// §FS-config-migrate.2.2 item 6: the preview's `would not write:` and the
/// write's `not written:` read one cause list, for every mix of setting and
/// generated-guidance refusals, listed comments and --drop-comments; a refused
/// preview with no candidate prints none, nor any `written:` line, under full
/// either. No black-box fixture reaches a differing gate; the renderer's unit
/// tests cover that cause.
#[test]
fn refused_previews_close_with_the_writes_causes() {
    let refusal = V1.replace("named_sections = true", "named_sections = false");
    let guidance = V1.replace(
        "file = \"requirements.md\"",
        "file = \"requirements.md\"\nvalues = true",
    );
    let comment = "\n# standalone\n";
    let inputs = [
        (refusal.clone(), false),
        (format!("{V1}{comment}"), false),
        (guidance.clone(), false),
        (format!("{refusal}{comment}"), false),
        (format!("{refusal}{comment}"), true),
        (format!("{guidance}{comment}"), false),
        (format!("{guidance}{comment}"), true),
    ];
    for (input, drop_comments) in inputs {
        let fixture = Fixture::new(&input);
        let before = snapshot(&fixture.0);
        let extra: &[&str] = if drop_comments {
            &["--drop-comments"]
        } else {
            &[]
        };
        let run = |more: &[&str]| fixture.migrate(&[extra, more].concat());
        let write = run(&["--write"]);
        status(&write, 1);
        let write_stderr = text(&write.stderr);
        let expected = causes(&write_stderr, "not written: ", "; grund.toml is unchanged")
            .unwrap_or_else(|| panic!("{write_stderr}"));
        for full in [false, true] {
            let preview = run(if full { &["--full"] } else { &[] });
            status(&preview, 1);
            let stderr = text(&preview.stderr);
            assert_eq!(
                causes(&stderr, "would not write: ", ""),
                Some(expected),
                "{stderr}"
            );
            let candidate = !preview.stdout.is_empty();
            assert_eq!(candidate, full && input.starts_with(V1), "{input}");
            let narrated = stderr.lines().any(|l| l.starts_with("written: "));
            assert_eq!(narrated, candidate, "{stderr}");
            assert_eq!(stderr.contains(PROJECT_MAP), candidate, "{stderr}");
        }
        assert_eq!(snapshot(&fixture.0), before);
    }
}
