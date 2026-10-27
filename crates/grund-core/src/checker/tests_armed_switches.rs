//! Test module: each gate records the switch it found set (§FS-check.1.4.1) —
//! one fixture per switch with it off, which arms none of its codes, and one
//! with it on, which arms all of them — and a rule's empty subject reuses the
//! obligation's conditions (§FS-check.2.2.1.4).

use std::path::PathBuf;

use crate::api::{CheckOpts, check_with_run_warnings};
use crate::model::{Finding, Switch};
use crate::testing::{test_root, write};

use super::ArmedChecks;

/// A version-2 project with an `FS` and a `REQ` kind, `extra` added to the `FS`
/// table and `tail` after both.
pub(super) fn v2(name: &str, extra: &str, tail: &str) -> PathBuf {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 2\nproject_name = \"armed\"\n\n\
             [schema.sources]\nlanguages = [\"markdown\"]\n\n\
             [schema.kinds.FS]\nfolders = [\"docs/fs\"]\nindex = false\n{extra}\n\
             [schema.kinds.REQ]\nfolders = [\"docs/req\"]\nindex = false\n\n{tail}"
        ),
    );
    write(&root.join("docs/fs/FS-a.md"), "# FS-a: A\n\nText.\n");
    root
}

pub(super) fn run(
    root: PathBuf,
    flags: impl FnOnce(&mut CheckOpts),
) -> (Vec<Finding>, ArmedChecks) {
    let mut opts = CheckOpts {
        path: root,
        path_provided: true,
        ..CheckOpts::default()
    };
    flags(&mut opts);
    let (_, result) = check_with_run_warnings(opts);
    let (output, armed) = result.expect("the fixture checks");
    (output.report.warnings, armed)
}

pub(super) fn armed(root: PathBuf) -> ArmedChecks {
    run(root, |_| {}).1
}

/// Asserts `switch` is off in `off` and on in `on`, and that `code`, which
/// needs only it, is advised in the first and not the second.
pub(super) fn pinned(switch: Switch, code: &str, off: ArmedChecks, on: ArmedChecks) {
    assert!(!off.armed.contains(switch), "{switch:?} armed with it off");
    assert!(off.off_advice(code).is_some(), "{code} not advised off");
    assert!(on.armed.contains(switch), "{switch:?} not armed with it on");
    assert_eq!(on.off_advice(code), None, "{code} advised on");
}

#[test]
fn lead_words_arm_oversized_lead() {
    pinned(
        Switch::LeadWords,
        "oversized-lead",
        armed(v2("armed_leads_off", "", "")),
        armed(v2(
            "armed_leads_on",
            "",
            "[schema.leads.words]\nwarn = 60\n",
        )),
    );
}

#[test]
fn a_rule_kind_arms_the_rule_codes() {
    let rule = "[schema.kinds.RULE]\nfolders = [\"docs/rules\"]\nindex = false\nform = \"rule\"\n";
    let on = armed(v2("armed_rules_on", "", rule));
    let off = armed(v2("armed_rules_off", "", ""));
    for code in ["invalid-rule", "chapter-cardinality", "uncited-unit"] {
        assert!(off.off_advice(code).is_some() && on.off_advice(code).is_none());
    }
    pinned(Switch::Rules, "invalid-rule", off, on);
}

#[test]
fn each_direction_arms_its_codes() {
    let off = armed(v2("armed_directions_off", "", ""));
    for (level, switch, code) in [
        ("must", Switch::MustDirection, "missing-citation"),
        ("must-not", Switch::MustNotDirection, "forbidden-citation"),
        ("should", Switch::ShouldDirection, "suggested-citation"),
        (
            "should-not",
            Switch::ShouldNotDirection,
            "discouraged-citation",
        ),
    ] {
        let tail = format!("[rules.citations.FS]\n{level} = [\"REQ\"]\n");
        let on = armed(v2(&format!("armed_direction_{switch:?}"), "", &tail));
        assert!(
            !off.armed.contains(switch) && on.armed.contains(switch),
            "{level}"
        );
        assert!(off.off_advice(code).is_some(), "{code}");
        if matches!(level, "must" | "should") {
            assert!(on.armed.contains(Switch::Obligation), "{level}");
        }
    }
    assert!(!off.armed.contains(Switch::Obligation));
}

#[test]
fn the_flags_arm_their_channels() {
    let (_, off) = run(v2("armed_flags_off", "", ""), |_| {});
    let (_, on) = run(v2("armed_flags_on", "", ""), |opts| {
        opts.include_suggestions = true;
        opts.full = true;
        opts.require_grounding = true;
    });
    for (switch, code) in [
        (Switch::Suggestions, "escaped-citation-resolves"),
        (Switch::Full, "out-of-scope-dangling"),
        (Switch::Grounding, "ungrounded"),
    ] {
        pinned(switch, code, off.clone(), on.clone());
    }
}

#[test]
fn the_v2_field_switches_arm_their_codes() {
    let off = armed(v2("armed_fields_off", "", ""));
    for (field, switch, code) in [
        ("presence = \"must\"", Switch::Field, "missing-field"),
        (
            "one_of = [\"Yes\", \"No\"]",
            Switch::FieldOneOf,
            "invalid-field-value",
        ),
        (
            "position = \"first\"",
            Switch::FieldPlacement,
            "misplaced-field",
        ),
        ("title = \"Status\"", Switch::FieldTitle, "mistitled-field"),
    ] {
        let tail = format!("[schema.kinds.FS.fields.status]\n{field}\n");
        let on = armed(v2(&format!("armed_field_{switch:?}"), "", &tail));
        pinned(switch, code, off.clone(), on);
    }
    let closed = armed(v2("armed_closed_on", "closed = \"must\"", ""));
    pinned(Switch::ClosedKind, "undeclared-chapter", off, closed);
}

#[test]
fn heading_depth_arms_section_heading_level() {
    pinned(
        Switch::HeadingDepth,
        "section-heading-level",
        armed(v2(
            "armed_depth_off",
            "",
            "[schema]\nheading_depth = \"may\"\n",
        )),
        armed(v2(
            "armed_depth_on",
            "",
            "[schema]\nheading_depth = \"must\"\n",
        )),
    );
}

#[test]
fn version_two_always_arms_named_sections() {
    let on = armed(v2("armed_named_v2", "", ""));
    assert!(on.armed.contains(Switch::NamedSections));
    assert_eq!(on.off_advice("orphan-section"), None);
}

#[test]
fn a_value_kind_arms_the_value_codes() {
    let value = "[schema.kinds.CONST]\nfolders = [\"values\"]\nindex = false\nform = \"value\"\n";
    let off = armed(v2("armed_values_off", "", ""));
    let root = v2("armed_values_on", "", value);
    write(&root.join("values/.keep"), "");
    let on = armed(root);
    for code in ["invalid-value-binding", "invalid-value-declaration"] {
        assert!(off.off_advice(code).is_some() && on.off_advice(code).is_none());
    }
    pinned(Switch::Values, "value-mismatch", off, on);
}

/// §FS-values.2.2.4: a whole-value kind arms the YAML window, whether or not its
/// home holds a YAML source yet.
#[test]
fn a_value_kind_arms_the_yaml_window() {
    pinned(
        Switch::ValueHome,
        "yaml-value-source-deferred",
        armed(v2("armed_value_home_off", "", "")),
        armed(v2(
            "armed_value_home_on",
            "",
            "[schema.kinds.CONST]\nfolders = [\"values\"]\nform = \"value\"\nindex = false\n",
        )),
    );
}

#[test]
fn a_kind_home_arms_misplaced_declaration() {
    let root = test_root("armed_home_off");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\nproject_name = \"armed\"\n\n\
         [schema.sources]\nlanguages = [\"markdown\"]\n\n\
         [schema.kinds.code]\ncitable = false\n",
    );
    write(&root.join("notes.md"), "Text.\n");
    pinned(
        Switch::KindHome,
        "misplaced-declaration",
        armed(root),
        armed(v2("armed_home_on", "", "")),
    );
}

#[test]
fn an_optional_member_arms_its_absence() {
    let root = test_root("armed_optional_on");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\nproject_name = \"acme\"\n\n\
         [workspace]\noptional_members = [\"vendored\"]\n\n\
         [reference]\nstrict = true\n\n[id]\nformat = \"{kind}-{slug}\"\n\n\
         [[kinds]]\nkind = \"SPEC\"\nfile = \"docs/spec.md\"\n\n\
         [scan]\ninclude = [\"docs\"]\n",
    );
    write(&root.join("docs/spec.md"), "# SPEC-a: A\n\nText.\n");
    pinned(
        Switch::OptionalMember,
        "optional-member-absent",
        armed(v2("armed_optional_off", "", "")),
        armed(root),
    );
}

/// The rule twin of the e2e `check-rule-applies-to-nothing-declared` project:
/// a `RULE-e2e` whose subject kind is homed by `e2e_home`, holding `files`
/// (§FS-check.2.2.1.4).
pub(super) fn rule_project(name: &str, e2e_home: &str, files: &[(&str, &str)]) -> PathBuf {
    let rule = format!(
        "[schema.kinds.e2e]\n{e2e_home}\n\
         [schema.kinds.RULE]\nfolders = [\"docs/rules\"]\nindex = false\nform = \"rule\"\n"
    );
    let root = v2(name, "", &rule);
    write(
        &root.join("docs/rules/RULE-e2e.md"),
        "# RULE-e2e: Each e2e must cite at least one FS.\n\nTests cite specs.\n",
    );
    for (path, text) in files {
        write(&root.join(path), text);
    }
    root
}

pub(super) const FOLDER: &str = "folders = [\"tests/e2e\"]\nindex = false";

fn applies_to_nothing(warnings: &[Finding]) -> Vec<&str> {
    warnings
        .iter()
        .filter(|finding| finding.code == "empty-citation-obligation")
        .map(|finding| finding.message.as_str())
        .collect()
}

/// §FS-check.2.2.1.4: a rule whose subject kind declares nothing in a home that
/// holds files says so once, authored by the rule, with no `citable` tail.
#[test]
fn a_rule_with_an_empty_subject_says_it_applies_to_nothing() {
    let root = rule_project(
        "rule_empty_subject",
        FOLDER,
        &[("tests/e2e/login.md", "Login.\n")],
    );
    let (warnings, _) = run(root, |_| {});
    assert_eq!(
        applies_to_nothing(&warnings),
        ["RULE-e2e applies to nothing — tests/e2e/ declares no e2e ID"]
    );
    let line = warnings
        .iter()
        .find(|finding| finding.code == "empty-citation-obligation")
        .expect("the line");
    assert_eq!(line.authority, ["RULE-e2e"]);
}

/// §FS-check.2.2.1.4: the obligation's conditions hold for a rule — a folder
/// whose only file is its entry file, and a file home, stay silent. A citable
/// kind cannot be `scan = false`, so condition 2 reaches a rule as the file home.
#[test]
fn a_rule_reuses_the_obligation_conditions() {
    let entry = rule_project(
        "rule_entry_only",
        "folders = [\"tests/e2e\"]",
        &[("tests/e2e/README.md", "# Tests\n")],
    );
    assert!(applies_to_nothing(&run(entry, |_| {}).0).is_empty());
    let file_home = rule_project(
        "rule_file_home",
        "files = [\"tests/e2e/login.md\"]",
        &[("tests/e2e/login.md", "Login.\n")],
    );
    assert!(applies_to_nothing(&run(file_home, |_| {}).0).is_empty());
}
