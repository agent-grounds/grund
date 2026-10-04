//! Binary-level contract for the rule-site unknown-alias reasons: the two final
//! templates §FS-errors.3.7.1 fixes, which the wording migration of
//! §FS-errors.3.7 ended in.
//!
//! Its own target for the reason `agents_init_message_ramp.rs` is one: this is
//! about the text of one code across a release boundary, and it fails on a
//! different day from anything that is about selecting or producing the finding.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A single project — no `[workspace]` anywhere — with a rule kind, so every
/// namespace-qualified object kind in it is §FS-rules.4.1's unverifiable case.
fn fixture(name: &str, object: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/rule-alias-message-ramp")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    write(
        root.join("grund.toml"),
        concat!(
            "grund_config_version = 1\n\n",
            "[reference]\nmarker = \"\u{a7}\"\nstrict = true\n\n",
            "[id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\n",
            "[scan]\ninclude = [\"docs\"]\n\n",
            "[[kinds]]\nkind = \"GOAL\"\nfolder = \"docs/goals\"\nindex = false\n\n",
            "[[kinds]]\nkind = \"SEG\"\nfolder = \"docs/segments\"\nindex = false\n\n",
            "[[kinds]]\nkind = \"RULE\"\nfolder = \"docs/rules\"\nindex = false\nrules = true\n",
        ),
    );
    write(
        root.join("docs/goals/GOAL-build.md"),
        "# GOAL-build: Every segment names how it is built.\n\nA segment nobody can build is a drawing.\n",
    );
    write(
        root.join("docs/segments/SEG-drive.md"),
        "# SEG-drive: The drive segment.\n\nThe drive segment turns the wheels for \u{a7}GOAL-build.\n\n## operations: Operations\n\nBuilt somehow.\n",
    );
    write(
        root.join("docs/rules/RULE-operations.md"),
        &format!(
            "# RULE-operations: The operations chapter of each SEG must cite at least one {object}.\n\nThe operations chapter carries the build evidence for \u{a7}GOAL-build.\n"
        ),
    );
    root
}

fn write(path: impl AsRef<Path>, text: &str) {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create fixture parent");
    }
    fs::write(path, text).expect("write fixture file");
}

/// The one `invalid-rule` reason a `--only invalid-rule` run printed, without
/// its `<path>:<line>: error: <ID> is not a valid rule: ` prefix.
fn invalid_rule_reason(name: &str, object: &str) -> String {
    let root = fixture(name, object);
    let output = Command::new(env!("CARGO_BIN_EXE_grund"))
        .arg("check")
        .arg(&root)
        .args(["--only", "invalid-rule"])
        .output()
        .expect("run grund check");
    let stdout = std::str::from_utf8(&output.stdout).expect("stdout is UTF-8");
    let mut lines = stdout.lines();
    let line = lines
        .next()
        .unwrap_or_else(|| panic!("no invalid-rule finding; stdout {stdout:?}"));
    assert_eq!(lines.next(), None, "one finding per fixture: {stdout:?}");
    line.split_once(" is not a valid rule: ")
        .unwrap_or_else(|| panic!("not a rule finding: {line:?}"))
        .1
        .to_string()
}

/// §FS-errors.3.7.1: the two final reasons, exactly — no legacy `unknown kind`
/// prefix and no suffix naming the release the wording changes in.
#[test]
fn the_rule_alias_reasons_are_the_two_final_templates() {
    assert_eq!(
        invalid_rule_reason("pinned", "workshop/OP"),
        "unknown project alias workshop; no workspace is in scope here, so the alias cannot be resolved \u{2014} check from the workspace root"
    );
    assert_eq!(
        invalid_rule_reason("any", "*/OP"),
        "no workspace is in scope here, so no namespace can be searched for OP \u{2014} check from the workspace root"
    );
}
