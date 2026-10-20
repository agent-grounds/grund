//! Test module: a code its catalog cell turns on more than one way is armed by
//! any of them (§FS-check.1.4.1) — the value codes by a declared value chapter
//! as by a value kind, and by a value tag the scan found with no switch at all,
//! and `optional-member-absent` by a nested block's list as by the root's.
//! Each source is pinned off and on, as `tests_armed_switches` pins the rest.

use std::path::{Path, PathBuf};

use super::tests_armed_switches::{armed, pinned, v2};
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
