//! Test module: the one predicate that says a row is closed to every citation
//! (§FS-config.3.9.4.1), read from the lowered rules in either config version.
//! The Citation directions bullet and the closed-citation-grounding warning of
//! agent-grounds/grund#1460 both read it, so it is pinned here, apart from both.

use super::grounding::row_closed_to_every_citation;
use super::*;
use crate::testing::{test_root, write};

/// Whether the row `row` is closed, under the global default `global` and the
/// row's own `rule`, in config `version`. Every config declares `FS` and the
/// non-citable `ce` home; `rule` is `None` where the row has no table at all.
fn closed(version: u32, global: Option<&str>, row: &str, rule: Option<&str>) -> bool {
    let body = if version == 1 {
        let global = global.map_or_else(String::new, |level| format!("default = \"{level}\"\n"));
        let rule = rule.map_or_else(String::new, |rule| format!("[citations.{row}]\n{rule}"));
        format!(
            "grund_config_version = 1\n\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\n\n\
             [[kinds]]\nkind = \"ce\"\nfolder = \"ce\"\ncitable = false\n\n\
             [citations]\n{global}{rule}"
        )
    } else {
        let global = global.map_or_else(String::new, |level| {
            format!("[rules.citations]\ndefault = \"{level}\"\n")
        });
        let rule = rule.map_or_else(String::new, |rule| {
            format!("[rules.citations.{row}]\n{rule}")
        });
        let declared = if global.is_empty() && rule.is_empty() {
            "[rules.citations]\n"
        } else {
            ""
        };
        format!(
            "grund_config_version = 2\nproject_name = \"demo\"\n\n[schema.kinds.FS]\n\
             folders = [\"docs\"]\n\n[schema.kinds.ce]\nfolders = [\"ce\"]\ncitable = false\n\n\
             {declared}{global}{rule}"
        )
    };
    let root = test_root("closed_row");
    write(&root.join("grund.toml"), &body);
    let config = load_config(&root).unwrap_or_else(|error| panic!("{error:#}\n{body}"));
    row_closed_to_every_citation(config.rules(), row)
}

/// The closed shapes, `(global, rule)`, each true in both versions.
const CLOSED: &[(Option<&str>, Option<&str>)] = &[
    (None, Some("default = \"must-not\"\n")),
    (None, Some("default = \"must-not\"\nmust-not = [\"FS\"]\n")),
    (Some("must-not"), Some("must-not = [\"FS\"]\n")),
    (Some("must-not"), None),
    (Some("may"), Some("default = \"must-not\"\n")),
];

/// The open shapes, `(version, global, rule)`: a `must-not` default with any
/// list but `must-not` beside it, a discouraged default, an own `may` default
/// under a global `must-not`, no default at all, and an inherited `must-not`
/// beside a `may` list. Version 2's `warn-not` rungs are its own.
const OPEN: &[(u32, Option<&str>, Option<&str>)] = &[
    (1, None, Some("default = \"must-not\"\nmay = [\"FS\"]\n")),
    (
        1,
        None,
        Some("default = \"must-not\"\nshould-not = [\"FS\"]\n"),
    ),
    (1, None, Some("default = \"must-not\"\nmust = [\"FS\"]\n")),
    (1, None, Some("default = \"should-not\"\n")),
    (1, Some("must-not"), Some("default = \"may\"\n")),
    (1, None, Some("must-not = [\"FS\"]\n")),
    (1, None, None),
    (1, Some("must-not"), Some("may = [\"FS\"]\n")),
    (
        2,
        None,
        Some("default = \"must-not\"\nwarn-not = [\"FS\"]\n"),
    ),
    (2, None, Some("default = \"warn-not\"\n")),
    (2, None, Some("default = \"must-not\"\nshould = [\"FS\"]\n")),
    (2, Some("must-not"), Some("default = \"may\"\n")),
    (2, None, None),
];

/// §FS-config.3.9.4.1: a `must-not` fallback, own or inherited, with no list
/// but `must-not` closes the row, and v1 and v2 spellings of it agree.
#[test]
fn a_must_not_fallback_with_only_must_not_lists_closes_the_row() {
    let mut open = Vec::new();
    for version in [1, 2] {
        for &(global, rule) in CLOSED {
            if !closed(version, global, "ce", rule) {
                open.push(format!("v{version}: global {global:?}, row {rule:?}"));
            }
        }
    }
    assert!(
        open.is_empty(),
        "read as open, expected closed:\n{}",
        open.join("\n")
    );
}

/// §FS-config.3.9.4.1: the predicate is the row's, not its place's, so a citable
/// row closed the same way is closed too.
#[test]
fn a_citable_row_is_closed_the_same_way() {
    let rule = Some("default = \"must-not\"\n");
    let open: Vec<u32> = [1, 2]
        .into_iter()
        .filter(|&version| !closed(version, None, "FS", rule))
        .collect();
    assert!(
        open.is_empty(),
        "FS with {rule:?} read as open in versions {open:?}"
    );
}

/// §FS-config.3.9.4.1: any permitted or merely discouraged target, or a fallback
/// that is not `must-not`, leaves the row open. A guard: it holds today.
#[test]
fn a_row_with_a_permitted_target_or_a_weaker_fallback_is_open() {
    for (version, global, rule) in OPEN {
        let shape = format!("v{version}: global {global:?}, row {rule:?}");
        assert!(
            !closed(*version, *global, "ce", *rule),
            "{shape} read as closed"
        );
    }
}
