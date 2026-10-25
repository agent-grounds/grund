//! Black-box migration reporting: §FS-config-migrate.4.1.1. Preserve carried
//! comments and explain vocabulary translations without changing write/drop policy.

use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Case {
    section: &'static str,
    setting: &'static str,
    source: &'static str,
    target: &'static str,
    candidate_line: &'static str,
    comment: &'static str,
    translated: bool,
}

const CASES: &[Case] = &[
    Case {
        section: "reference",
        setting: "inline_style = \"citation-with-note\"",
        source: "[reference] inline_style",
        target: "[schema.notes.text] must",
        candidate_line: "must = true",
        comment: "citation-with-note | citation-only",
        translated: true,
    },
    Case {
        section: "reference",
        setting: "inline_style = \"citation-only\"",
        source: "[reference] inline_style",
        target: "[schema.notes.text] must",
        candidate_line: "must = false",
        comment: "editor preference",
        translated: true,
    },
    Case {
        section: "id",
        setting: "section_heading_levels = \"strict\"",
        source: "[id] section_heading_levels",
        target: "[schema] heading_depth",
        candidate_line: "heading_depth = \"must\"",
        comment: "strict | warn | loose",
        translated: true,
    },
    Case {
        section: "id",
        setting: "section_heading_levels = \"warn\"",
        source: "[id] section_heading_levels",
        target: "[schema] heading_depth",
        candidate_line: "heading_depth = \"warn\"",
        comment: "strict | warn | loose",
        translated: true,
    },
    Case {
        section: "id",
        setting: "section_heading_levels = \"loose\"",
        source: "[id] section_heading_levels",
        target: "[schema] heading_depth",
        candidate_line: "heading_depth = \"may\"",
        comment: "strict | warn | loose",
        translated: true,
    },
    Case {
        section: "kinds",
        setting: "resolve = \"should\"",
        source: "[[kinds]] resolve",
        target: "[rules.resolution] FS",
        candidate_line: "FS = \"warn\"",
        comment: "must | should",
        translated: true,
    },
    Case {
        section: "kinds",
        setting: "rules = true",
        source: "[[kinds]] rules",
        target: "[schema.kinds.FS] form",
        candidate_line: "form = \"rule\"",
        comment: "true | false",
        translated: true,
    },
    Case {
        section: "fmt.cross_refs",
        setting: "enabled = true",
        source: "[fmt.cross_refs] enabled",
        target: "[presentation.fmt] links",
        candidate_line: "links = \"index\"",
        comment: "true | false",
        translated: true,
    },
    Case {
        section: "scan",
        setting: "exclude = [\"vendor\"]",
        source: "[scan] exclude",
        target: "[schema.sources] exclude",
        candidate_line: "exclude = [\"vendor/\"]",
        comment: "directory names only",
        translated: true,
    },
    Case {
        section: "fmt.cross_refs",
        setting: "anchor_format = \"github\"",
        source: "[fmt.cross_refs] anchor_format",
        target: "[presentation.fmt] anchors",
        candidate_line: "anchors = \"github\"",
        comment: "github | gitlab | ...",
        translated: false,
    },
    Case {
        section: "reference",
        setting: "lead_size_warning = { max = 120, unit = \"words\" }",
        source: "[reference] lead_size_warning",
        target: "[schema.leads.words] warn",
        candidate_line: "warn = 120",
        comment: "max and unit: lines | words | bytes",
        translated: true,
    },
    Case {
        section: "kinds",
        setting: "file = \"requirements.md\"",
        source: "[[kinds]] file",
        target: "[schema.kinds.FS] files",
        candidate_line: "files = [\"requirements.md\"]",
        comment: "a path",
        translated: false,
    },
    Case {
        section: "citations",
        setting: "default = \"must\"",
        source: "[citations] default",
        target: "[rules.citations] default",
        candidate_line: "default = \"may\"",
        comment: "must | should | may",
        translated: false,
    },
];

fn config(case: &Case, attachment: Option<bool>) -> String {
    let setting = match attachment {
        Some(true) => format!("# {}\n{}\n", case.comment, case.setting),
        Some(false) => format!("{} # {}\n", case.setting, case.comment),
        None => format!("{}\n", case.setting),
    };
    let at = |section| {
        if case.section == section {
            setting.as_str()
        } else {
            ""
        }
    };
    let home = if case.section == "kinds" && case.setting.starts_with("file =") {
        ""
    } else {
        "folder = \"docs\"\nindex = false\n"
    };
    let fetch = if case.setting.starts_with("resolve =") {
        "fetch = \"scripts/fetch\"\n"
    } else {
        ""
    };
    format!(
        "grund_config_version = 1\n\
         [reference]\n{}\
         [id]\nnamed_sections = true\n{}\
         [[kinds]]\nkind = \"FS\"\n{home}{fetch}{}\
         [scan]\ninclude = [\"docs\", \"requirements.md\"]\nextensions = [\"md\"]\n{}\
         [fmt.cross_refs]\n{}\
         [citations]\n{}",
        at("reference"),
        at("id"),
        at("kinds"),
        at("scan"),
        at("fmt.cross_refs"),
        at("citations"),
    )
}

/// Run all modes even while warnings are missing, so a failure records the
/// entire translation/control matrix rather than only its first cell.
/// §FS-config-migrate.6.5: compact writes omit stdout but keep carried warnings.
fn check_carried_comments(above: bool) {
    static SERIAL: AtomicUsize = AtomicUsize::new(0);
    let mut missing = Vec::new();
    let mut first_report = None;
    for case in CASES {
        for attached in [true, false] {
            let input = config(case, attached.then_some(above));
            let key_line = input
                .lines()
                .position(|l| l.starts_with(case.setting))
                .unwrap()
                + 1;
            let mut preview = None;
            for drop_comments in [false, true] {
                for write in [false, true] {
                    let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
                    let root = std::env::temp_dir().join(format!(
                        "grund-783-comments-{}-{serial}",
                        std::process::id()
                    ));
                    fs::create_dir_all(root.join("docs")).unwrap();
                    fs::write(root.join("requirements.md"), "").unwrap();
                    fs::write(root.join("grund.toml"), &input).unwrap();
                    let mut cmd = Command::new(env!("CARGO_BIN_EXE_grund"));
                    cmd.current_dir(&root)
                        .args(["config", "migrate", "--to", "2"]);
                    if write {
                        cmd.arg("--write");
                    }
                    if drop_comments {
                        cmd.arg("--drop-comments");
                    }
                    let out = cmd.output().expect("run grund");
                    let after = fs::read_to_string(root.join("grund.toml")).unwrap();
                    fs::remove_dir_all(&root).unwrap();
                    let stderr = String::from_utf8(out.stderr).unwrap();
                    let stdout = String::from_utf8(out.stdout).unwrap();
                    let candidate = if write {
                        assert!(stdout.is_empty(), "compact success omits the candidate");
                        after.clone()
                    } else {
                        stdout
                    };
                    let context = format!(
                        "{}; above={above}, attached={attached}, write={write}, drop={drop_comments}",
                        case.source
                    );
                    assert_eq!(out.status.code(), Some(0), "{context}\n{stderr}");
                    assert!(
                        candidate.starts_with("grund_config_version = 2\n"),
                        "{context}"
                    );
                    assert!(
                        candidate.contains(case.candidate_line),
                        "{context}\n{candidate}"
                    );
                    assert_eq!(
                        after,
                        if write { &candidate } else { &input }.as_str(),
                        "{context}"
                    );
                    assert!(
                        !stderr.contains("comments with no single key to follow"),
                        "{context}\n{stderr}"
                    );
                    if attached {
                        let carried = if above {
                            format!("# {}\n{}", case.comment, case.candidate_line)
                        } else {
                            format!("{} # {}", case.candidate_line, case.comment)
                        };
                        assert!(candidate.contains(&carried), "{context}\n{candidate}");
                    } else {
                        assert!(
                            !candidate.contains(&format!("# {}", case.comment)),
                            "{context}"
                        );
                    }
                    if let Some(expected) = &preview {
                        assert_eq!(&candidate, expected, "flags changed candidate: {context}");
                    } else {
                        preview = Some(candidate);
                    }
                    // Match the warning's meaning, not its prefix, punctuation,
                    // or a new public report field. A mapped line alone is insufficient.
                    let warnings: Vec<_> = stderr
                        .lines()
                        .filter(|line| {
                            let lower = line.to_lowercase();
                            lower.contains("comment")
                                && (lower.contains("check") || lower.contains("review"))
                                && (lower.contains("translat") || lower.contains("vocab"))
                        })
                        .collect();
                    if case.translated && attached {
                        if warnings.len() != 1
                            || !warnings.iter().any(|line| {
                                line.contains(case.source)
                                    && line.contains(case.target)
                                    && line.contains(&format!("grund.toml:{key_line}:"))
                            })
                        {
                            missing
                                .push(format!("expected one carried-comment warning: {context}"));
                            first_report.get_or_insert(stderr);
                        }
                    } else {
                        assert!(
                            warnings.is_empty(),
                            "unexpected warning: {context}\n{stderr}"
                        );
                    }
                }
            }
        }
    }
    assert!(
        missing.is_empty(),
        "{} missing warnings:\n{}\nfirst report:\n{}",
        missing.len(),
        missing.join("\n"),
        first_report.unwrap_or_default()
    );
}

/// §FS-config-migrate.4.1.1: the two real trailing value-set comments and the
/// other carryable translations warn; harmless renames and absent comments do not.
#[test]
fn trailing_carried_comments_report_translated_vocabulary() {
    check_carried_comments(false);
}

/// §FS-config-migrate.4.1.1: a leading attached comment has the same risk,
/// regardless of its prose, the selected old value, or preview/write/drop flags.
#[test]
fn leading_carried_comments_report_translated_vocabulary() {
    check_carried_comments(true);
}

/// §FS-config-migrate.4.1.1 and §FS-config-migrate.4.4: warning about a carried
/// comment does not change which unrelated split-key comments refuse or drop.
/// §FS-config-migrate.6.5: only compact success omits the pre-write listing.
#[test]
fn carried_comment_warning_keeps_split_comment_drop_policy() {
    let input = config(&CASES[0], Some(false)).replace(
        "include = [\"docs\", \"requirements.md\"]",
        "include = [\"docs\", \"requirements.md\"] # split-key comment",
    );
    let mut missing = Vec::new();
    let mut preview = None;
    let split_line = input
        .lines()
        .position(|line| line.contains("# split-key comment"))
        .unwrap()
        + 1;
    for drop_comments in [false, true] {
        for write in [false, true] {
            let root = std::env::temp_dir().join(format!(
                "grund-783-split-comments-{}-{write}-{drop_comments}",
                std::process::id()
            ));
            fs::create_dir_all(root.join("docs")).unwrap();
            fs::write(root.join("requirements.md"), "").unwrap();
            fs::write(root.join("grund.toml"), &input).unwrap();
            let mut cmd = Command::new(env!("CARGO_BIN_EXE_grund"));
            cmd.current_dir(&root)
                .args(["config", "migrate", "--to", "2"]);
            if write {
                cmd.arg("--write");
            } else {
                // §FS-config-migrate.2.1: a comment-refused preview prints its
                // candidate only under --full; a passing one is the same either way.
                cmd.arg("--full");
            }
            if drop_comments {
                cmd.arg("--drop-comments");
            }
            let out = cmd.output().unwrap();
            let after = fs::read_to_string(root.join("grund.toml")).unwrap();
            fs::remove_dir_all(root).unwrap();
            let stderr = String::from_utf8(out.stderr).unwrap();
            let stdout = String::from_utf8(out.stdout).unwrap();
            assert_eq!(
                out.status.code(),
                Some(if drop_comments { 0 } else { 1 }),
                "{stderr}"
            );
            let compact = write && drop_comments;
            assert_eq!(
                stderr.contains("comments with no single key to follow"),
                !compact,
                "{stderr}"
            );
            assert_eq!(
                stderr.contains("inside [scan] include"),
                !compact,
                "{stderr}"
            );
            assert!(!stdout.contains("split-key comment"));
            if write {
                assert!(stdout.is_empty());
            } else {
                assert!(stdout.contains("must = true # citation-with-note | citation-only"));
                if let Some(expected) = &preview {
                    assert_eq!(
                        &stdout, expected,
                        "authorization does not change candidate bytes"
                    );
                } else {
                    preview = Some(stdout.clone());
                }
            }
            if compact {
                assert_eq!(&after, preview.as_ref().expect("preview precedes writes"));
                assert!(!after.contains("split-key comment"));
                assert!(after.contains("must = true # citation-with-note | citation-only"));
                let dropped =
                    format!("dropped the comments at grund.toml:{split_line} (version 1 lines)");
                assert_eq!(
                    stderr.lines().filter(|line| *line == dropped).count(),
                    1,
                    "{stderr}"
                );
            } else {
                assert_eq!(after, input);
            }
            let warnings: Vec<_> = stderr
                .lines()
                .filter(|line| {
                    line.contains("comment")
                        && (line.contains("check") || line.contains("review"))
                        && (line.contains("translat") || line.contains("vocab"))
                        && line.contains("grund.toml:3:")
                        && line.contains("[reference] inline_style")
                        && line.contains("[schema.notes.text] must")
                })
                .collect();
            if warnings.len() != 1 {
                missing.push(format!(
                    "expected one carried-comment warning: write={write}, drop={drop_comments}\n{stderr}"
                ));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "{} missing warnings:\n{}",
        missing.len(),
        missing.join("\n")
    );
}
