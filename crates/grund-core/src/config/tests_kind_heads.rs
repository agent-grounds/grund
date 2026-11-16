//! Format-aware admission, shared by both readers (§FS-config.3.4.5.1,
//! §FS-config.3.2.9). Each matrix runs every row before reporting failures.

use super::load_config;
use crate::testing::{test_root, write};

fn pair(version: u8, default: &str, rows: &[(&str, Option<&str>)]) -> String {
    let mut text = if version == 1 {
        format!("grund_config_version = 1\n[id]\nformat = {default:?}\n")
    } else {
        format!("grund_config_version = 2\n[schema]\nid_format = {default:?}\n")
    };
    for (i, (kind, format)) in rows.iter().enumerate() {
        if version == 1 {
            text += &format!("[[kinds]]\nkind = {kind:?}\nfile = \"docs/{i}.md\"\n");
        } else {
            text += &format!("[schema.kinds.{kind}]\nfiles = [\"docs/{i}.md\"]\n");
        }
        if let Some(format) = format {
            let key = if version == 1 { "format" } else { "id_format" };
            text += &format!("{key} = {format:?}\n");
        }
    }
    text
}

/// ASCII upper/lower letters and digits are boundaries that preserve unique
/// malformed-token attribution; effective formats may be inherited or overridden.
#[test]
#[ignore = "red until #1357 format-aware admission; implement removes this"]
fn kind_heads_admit_distinct_heads_in_both_readers_and_orders() {
    let root = test_root("kind_heads_admit");
    let mut failures = Vec::new();
    for version in [1, 2] {
        for (short, long, default, short_format, long_format) in [
            (
                "GR",
                "GRUND",
                "{kind}-{slug}",
                Some("{kind}-{number}"),
                None,
            ),
            ("DA", "DAT", "{kind}-{slug}", None, None),
            ("GR", "GRu", "{kind}-{slug}", None, None),
            ("GR", "GR9", "{kind}-{slug}", None, None),
            (
                "GR",
                "GRUND",
                "{kind}_{number}",
                None,
                Some("{kind}-{slug}"),
            ),
            (
                "GR",
                "GRUND",
                "{kind}-{slug}",
                Some("{kind}:{number}"),
                None,
            ),
            ("GR", "GRUND", "{kind}-{slug}", None, Some("{kind}{number}")),
        ] {
            for reverse in [false, true] {
                let mut rows = [(short, short_format), (long, long_format)];
                if reverse {
                    rows.reverse();
                }
                write(&root.join("grund.toml"), &pair(version, default, &rows));
                if let Err(error) = load_config(&root) {
                    failures.push(format!("v{version} {rows:?}: {error:#}"));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Refusals distinguish remedies while keeping the whole old diagnostic as a
/// prefix. Different separators do not repair an unsafe name boundary.
#[test]
#[ignore = "red until #1357 appended collision reasons; implement removes this"]
fn kind_heads_collision_errors_preserve_prefix_and_explain_remedy() {
    let root = test_root("kind_heads_reasons");
    let mut failures = Vec::new();
    for version in [1, 2] {
        for (long, short_format, long_format, remedy) in [
            ("GR-X", "{kind}_{number}", "{kind}-{slug}", "rename"),
            ("GR_X", "{kind}-{number}", "{kind}_{slug}", "rename"),
            ("GRé", "{kind}-{number}", "{kind}-{slug}", "rename"),
            ("GRUND", "{kind}{number}", "{kind}-{slug}", "format"),
            ("GRUND", "{kind}UND{number}", "{kind}-{slug}", "format"),
            ("GRUND", "{kind}UND-{number}", "{kind}-{slug}", "format"),
            (
                "GRUND",
                "{kind}UND-extra-{number}",
                "{kind}-{slug}",
                "format",
            ),
            ("GRUND", "{number}-{kind}", "{kind}-{slug}", "format"),
            ("GRUND", "{kind}-{number}", "{slug}-{kind}", "format"),
        ] {
            // V2's spelling gate admits only bare ASCII header names; v1 can
            // reach the Unicode boundary, which is deliberately not ASCII.
            if version == 2 && !long.is_ascii() {
                continue;
            }
            write(
                &root.join("grund.toml"),
                &pair(
                    version,
                    "{kind}-{slug}",
                    &[("GR", Some(short_format)), (long, Some(long_format))],
                ),
            );
            let error = match load_config(&root) {
                Ok(_) => {
                    failures.push(format!("v{version} GR/{long} unexpectedly loaded"));
                    continue;
                }
                Err(error) => format!("{error:#}"),
            };
            let old = format!(
                "grund.toml: kinds \u{60}GR\u{60} and \u{60}{long}\u{60} collide (one is a prefix of the other)"
            );
            let suffix = error.strip_prefix(&old);
            if !suffix.is_some_and(|suffix| suffix.contains(remedy)) {
                failures.push(format!(
                    "v{version}: expected old prefix plus {remedy:?}: {error}"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// These controls already pass; unknown heads only constrain prefix pairs.
#[test]
fn kind_heads_keep_non_prefix_non_leading_and_non_citable_configs() {
    let root = test_root("kind_heads_controls");
    for version in [1, 2] {
        write(
            &root.join("grund.toml"),
            &pair(version, "{number}-{kind}", &[("GR", None), ("FS", None)]),
        );
        load_config(&root).unwrap_or_else(|e| panic!("v{version}: {e:#}"));
    }
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n[[kinds]]\nkind = \"GR\"\nfile = \"gr.md\"\n\
         [[kinds]]\nkind = \"GR-X\"\ncitable = false\nfolder = \"notes\"\n",
    );
    load_config(&root).expect("non-citable names do not tokenize");
}

/// Safety verdicts already pass before the new explanatory suffix exists.
#[test]
fn kind_heads_unsafe_pairs_and_duplicates_remain_refused() {
    let root = test_root("kind_heads_refusal_controls");
    for version in [1, 2] {
        for (long, short_format) in [
            ("GR-X", "{kind}_{number}"),
            ("GRUND", "{kind}{number}"),
            ("GRUND", "{kind}UND{number}"),
            ("GRUND", "{number}-{kind}"),
            ("GR", "{kind}-{number}"),
        ] {
            write(
                &root.join("grund.toml"),
                &pair(
                    version,
                    "{kind}-{slug}",
                    &[("GR", Some(short_format)), (long, None)],
                ),
            );
            assert!(
                load_config(&root).is_err(),
                "v{version} GR/{long} must fail"
            );
        }
    }
}
