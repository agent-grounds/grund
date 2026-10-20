//! Test module: a code its catalog cell turns on more than one way is armed by
//! any of them (§FS-check.1.4.1) — the value codes by a declared value chapter
//! as by a value kind, and by a value tag the scan found with no switch at all,
//! and `optional-member-absent` by a nested block's list as by the root's — and
//! one whose cell needs two things only by both, as `missing-snapshot` needs a
//! fetched kind and its `should` resolution, `empty-citation-obligation` an
//! obligation and a folder home, and `ungrounded` a default and a place it
//! reaches. Each source is pinned off and on, as `tests_armed_switches` pins the
//! rest.

use std::path::{Path, PathBuf};

use super::tests_armed_switches::{FOLDER, armed, pinned, rule_project, run, v2};
use crate::model::Switch;
use crate::testing::{test_root, write};

/// A version-1 project with named sections, its one `AR` row carrying `extra`.
fn v1(name: &str, extra: &str) -> PathBuf {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 1\n\n[reference]\nstrict = true\n\n\
             [id]\nformat = \"{{kind}}-{{slug}}\"\nnamed_sections = true\n\n\
             [[kinds]]\nkind = \"AR\"\nfolder = \"docs\"\nindex = false\n{extra}\n"
        ),
    );
    write(&root.join("docs/AR-a.md"), "# AR-a: A\n\nText.\n");
    root
}

/// §FS-check.1.4.1, §FS-values.2.5: a declared value chapter arms the value
/// codes under either version, and the advice names it beside the value kind.
#[test]
fn a_value_chapter_arms_the_value_codes_in_either_version() {
    let v1_off = armed(v1("armed_chapter_v1_off", ""));
    let advice = v1_off.off_advice("value-mismatch").expect("advised off");
    assert!(advice.contains("value_chapter"), "{advice}");
    let v1_on = armed(v1("armed_chapter_v1_on", "value_chapter = \"values\""));
    pinned(Switch::Values, "value-mismatch", v1_off, v1_on);

    let v2_off = armed(v2("armed_chapter_v2_off", "", ""));
    let advice = v2_off.off_advice("value-mismatch").expect("advised off");
    assert!(advice.contains("content = \"values\""), "{advice}");
    let field = "[schema.kinds.FS.fields.values]\ncontent = \"values\"\n";
    let v2_on = armed(v2("armed_chapter_v2_on", "", field));
    for code in ["invalid-value-binding", "invalid-value-declaration"] {
        assert_eq!(v2_on.off_advice(code), None, "{code}");
    }
    pinned(Switch::Values, "value-mismatch", v2_off, v2_on);
}

/// §FS-check.1.4.1, §FS-values.2.4: a value tag needs no switch, so the scan
/// finding one arms the value codes, and so does a misplaced one, which the
/// family reports (§FS-values.2.4.4).
#[test]
fn a_value_tag_arms_the_value_codes_with_no_switch() {
    let off = armed(v2("armed_tag_off", "", ""));
    let marked = v2("armed_tag_on", "", "");
    write(
        &marked.join("docs/fs/FS-a.md"),
        "# FS-a: A\n\n## 1. Floor <!-- grund:value -->\n\n### 1.1. 45\n",
    );
    pinned(Switch::Values, "value-mismatch", off, armed(marked));

    let misplaced = v2("armed_tag_misplaced", "", "");
    write(
        &misplaced.join("docs/fs/FS-a.md"),
        "# FS-a: A\n\n## floor: Floor <!-- grund:value -->\n\nText.\n",
    );
    assert_eq!(
        armed(misplaced).off_advice("invalid-value-declaration"),
        None
    );
}

/// A fetched `TICKET` kind, the table a version-2 project adds beside its `FS`.
const TICKET_V2: &str = "[schema.kinds.TICKET]\nfiles = [\"docs/tickets.md\"]\n\
                         id_format = \"{kind}-{number}\"\nfetch = \"scripts/fetch-ticket\"\n";

/// The same kind as a version-1 row, `resolve` its last line.
fn ticket_v1(resolve: &str) -> String {
    format!(
        "\n[[kinds]]\nkind = \"TICKET\"\nfile = \"docs/tickets.md\"\n\
         format = \"{{kind}}-{{number}}\"\nfetch = \"scripts/fetch-ticket\"\n{resolve}"
    )
}

/// §FS-check.1.4.1, §FS-config.3.4.10.3: `missing-snapshot` is armed by a fetched
/// kind whose resolution is `should` (v2's `warn`), and by neither part alone:
/// `fetch` without it resolves `must`, which selects the `dangling` error. The
/// advice names the resolution, in the version the project reads.
#[test]
fn only_a_fetched_kind_that_warns_arms_missing_snapshot() {
    let code = "missing-snapshot";
    let warn = format!("{TICKET_V2}\n[rules.resolution]\nTICKET = \"warn\"\n");
    let v2_on = armed(v2("armed_snapshot_v2_warn", "", &warn));
    let v2_alone = armed(v2("armed_snapshot_v2_alone", "", TICKET_V2));
    assert_eq!(
        v2_alone.off_advice(code).as_deref(),
        Some(
            "the check is off until [rules.resolution] <KIND> = \"warn\" is set for a fetched kind"
        )
    );
    for off in [armed(v2("armed_snapshot_v2_none", "", "")), v2_alone] {
        pinned(Switch::SnapshotWarning, code, off, v2_on.clone());
    }

    let v1_on = armed(v1(
        "armed_snapshot_v1_should",
        &ticket_v1("resolve = \"should\"\n"),
    ));
    let v1_alone = armed(v1("armed_snapshot_v1_alone", &ticket_v1("")));
    assert_eq!(
        v1_alone.off_advice(code).as_deref(),
        Some("the check is off until resolve = \"should\" is set on a fetched [[kinds]] row")
    );
    for off in [armed(v1("armed_snapshot_v1_none", "")), v1_alone] {
        pinned(Switch::SnapshotWarning, code, off, v1_on.clone());
    }
}

/// §FS-check.1.4.1, §FS-check.2.2.1.1: an obligation arms `empty-citation-obligation`
/// only on a scanned kind with a folder home, the only kind it warns about, so an
/// entry's or a rule's on a file home, and an entry's on the homeless kind, leave
/// it off. The advice names the folder home, in the version the project reads.
#[test]
fn only_an_obligation_that_can_apply_to_nothing_arms_its_line() {
    let code = "empty-citation-obligation";
    let e2e = |name: &str, home: &str| {
        let tail =
            format!("[schema.kinds.e2e]\n{home}\n\n[rules.citations.e2e]\nmust = [\"FS\"]\n");
        armed(v2(name, "", &tail))
    };
    let on = e2e("armed_obligation_folder", FOLDER);
    let file = e2e("armed_obligation_file", "files = [\"tests/e2e/login.md\"]");
    assert_eq!(
        file.off_advice(code).as_deref(),
        Some(
            "the check is off until [rules.citations.<kind>] must or should, or a must or should rule, is set for a scanned kind with a folder home"
        )
    );
    let homeless = "[rules.citations.code]\nmust = [\"FS\"]\n";
    let homeless = armed(v2("armed_obligation_homeless", "", homeless));
    for off in [file, homeless] {
        pinned(Switch::Obligation, code, off, on.clone());
    }
    let goal =
        "\n[[kinds]]\nkind = \"GOAL\"\nfile = \"goals.md\"\n\n[citations.GOAL]\nmust = [\"AR\"]\n";
    let v1_file = armed(v1("armed_obligation_v1_file", goal));
    assert_eq!(
        v1_file.off_advice(code).as_deref(),
        Some(
            "the check is off until [citations.<kind>] must or should, or a must or should rule, is set for a scanned kind with a folder home"
        )
    );
    let v1_folder = armed(v1(
        "armed_obligation_v1_folder",
        "\n[citations.AR]\nmust = [\"AR\"]\n",
    ));
    pinned(Switch::Obligation, code, v1_file, v1_folder);

    let rule_file = rule_project(
        "armed_rule_file_home",
        "files = [\"tests/e2e/login.md\"]",
        &[],
    );
    let rule_folder = rule_project("armed_rule_folder_home", FOLDER, &[]);
    pinned(
        Switch::Obligation,
        code,
        armed(rule_file),
        armed(rule_folder),
    );
}

/// A version-1 project whose `[reference]` and `code` row set `require_grounding`
/// to `default` and `code`, and whose `AR` row sets it `false`.
fn grounding(name: &str, default: bool, code: bool) -> PathBuf {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 1\n\n[reference]\nstrict = true\nrequire_grounding = {default}\n\n\
             [id]\nformat = \"{{kind}}-{{slug}}\"\n\n\
             [[kinds]]\nkind = \"AR\"\nfolder = \"docs\"\nindex = false\nrequire_grounding = false\n\n\
             [[kinds]]\nkind = \"code\"\ncitable = false\nrequire_grounding = {code}\n"
        ),
    );
    write(&root.join("docs/AR-a.md"), "# AR-a: A\n\nText.\n");
    root
}

/// §FS-check.1.4.1, §FS-config.3.4.8.3: `ungrounded` is armed by a place grounding
/// reaches, and a row's own `false` wins over the default, so a default that every
/// row refuses, the key's or the flag's, leaves it off. The advice says so in either
/// version.
#[test]
fn a_grounding_default_every_row_refuses_arms_nothing() {
    let code = "ungrounded";
    let on = armed(grounding("armed_grounding_reaches", true, true));
    let key = armed(grounding("armed_grounding_key_refused", true, false));
    assert_eq!(
        key.off_advice(code).as_deref(),
        Some(
            "the check is off until require_grounding = true is set on a [[kinds]] row, or in [reference] or by --require-grounding for a kind whose row does not set it false"
        )
    );
    let flag = grounding("armed_grounding_flag_refused", false, false);
    let flag = run(flag, |opts| opts.require_grounding = true).1;
    for off in [key, flag] {
        pinned(Switch::Grounding, code, off, on.clone());
    }

    let refused = "[rules.citations.grounding]\nmust = \"file\"\n\n\
                   [rules.citations.FS.grounding]\nmay = \"file\"\n\n\
                   [rules.citations.REQ.grounding]\nmay = \"file\"\n\n\
                   [rules.citations.code.grounding]\nmay = \"file\"\n";
    let v2_off = armed(v2("armed_grounding_v2_refused", "", refused));
    assert_eq!(
        v2_off.off_advice(code).as_deref(),
        Some(
            "the check is off until a must rung is set in [rules.citations.<kind>.grounding], or in [rules.citations.grounding] or by --require-grounding for a kind with no ladder of its own"
        )
    );
    let reaches = "[rules.citations.grounding]\nmust = \"file\"\n";
    let v2_on = armed(v2("armed_grounding_v2_reaches", "", reaches));
    pinned(Switch::Grounding, code, v2_off, v2_on);
}

/// A version-2 project with one `FS` declaration at `root`.
fn member(root: &Path, name: &str) {
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 2\nproject_name = \"{name}\"\n\n\
             [schema.sources]\nlanguages = [\"markdown\"]\n\n\
             [schema.kinds.FS]\nfolders = [\"docs/fs\"]\nindex = false\n"
        ),
    );
    write(&root.join("docs/fs/FS-a.md"), "# FS-a: A\n\nText.\n");
}

/// A workspace whose one member, `sub`, is a block and no project, its own
/// `[workspace]` table carrying `lists`; each name in `present` is a project
/// under it.
fn nested_block(name: &str, lists: &str, present: &[&str]) -> PathBuf {
    let root = v2(name, "", "[workspace]\nmembers = [\"sub\"]\n");
    write(
        &root.join("sub/grund.toml"),
        &format!(
            "grund_config_version = 2\nproject_name = \"sub\"\n\n\
             [workspace]\ninclude_root = false\n{lists}\n"
        ),
    );
    for project in present {
        member(&root.join("sub").join(project), project);
    }
    root
}

/// §FS-check.1.4.1, §FS-check.4.9: a nested block's `optional_members` arms the
/// absence check as the root's does, whether the member is absent or present.
#[test]
fn a_nested_block_listing_an_optional_member_arms_its_absence() {
    let off = || {
        armed(nested_block(
            "armed_nested_off",
            "members = [\"inner\"]",
            &["inner"],
        ))
    };
    let optional = "optional_members = [\"vendored\"]";
    let absent = armed(nested_block("armed_nested_absent", optional, &[]));
    pinned(
        Switch::OptionalMember,
        "optional-member-absent",
        off(),
        absent,
    );
    let present = armed(nested_block(
        "armed_nested_present",
        optional,
        &["vendored"],
    ));
    pinned(
        Switch::OptionalMember,
        "optional-member-absent",
        off(),
        present,
    );
}
