//! Test module: the link of a `.<section>` citation names a section the scan
//! records, so a numbered heading after the heading that closes the declaration's
//! body gives no link, in a scanned home and in a stub's target the walk does not
//! reach (§FS-fmt.6.4.1). The link is the one `fmt --cross-refs` writes
//! (§FS-fmt.6.2.1).
//!
//! Each case asks `markdown_link_target`, the derivation `fmt` calls, rather than
//! the anchor beneath it, so how the home's record reaches the anchor stays the
//! implementation's choice. Each states its premise first: what the walk records
//! of `FS-a`.

use super::markdown_link_target;
use crate::config::load_config;
use crate::model::{Findings, Id};
use crate::testing::{scan_findings, test_root, write};

const CONFIG: &str = "grund_config_version = 1\n\n[reference]\nstrict = true\n\
require_grounding = false\n\n[id]\nformat = \"{kind}-{slug}\"\n\n[[kinds]]\n\
kind = \"FS\"\nfolder = \"docs\"\nindex = false\n\n[scan]\ninclude = [\"docs\"]\n";

/// grund.92's home: a plain heading closes `FS-a`'s body, and `## 1.` follows it.
const CLOSED: &str = "# FS-a: A\n\nLead.\n\n# Other heading\n\n## 1. After close\n\nbody\n";

/// The same home with its section inside the body.
const DECLARED: &str = "# FS-a: A\n\nLead.\n\n## 1. Real section\n\nbody\n";

/// A stub in the walk to a home under `notes/`, which `[scan] include` leaves out.
const STUB: &str = "# FS-a: [../notes/a.md](../notes/a.md)\n";

fn id_a() -> Id {
    Id {
        kind: "FS".to_string(),
        num: None,
        slug: Some("a".to_string()),
    }
}

/// The tree `files` writes, scanned; then what the walk records, and the link
/// `docs/uses.md` takes for a citation of `FS-a.1`.
fn link_of_a_1(name: &str, files: &[(&str, &str)]) -> (Findings, Option<String>) {
    let root = test_root(name);
    write(&root.join("grund.toml"), CONFIG);
    write(&root.join("docs/uses.md"), "Uses \u{a7}FS-a.1.\n");
    for (path, text) in files {
        write(&root.join(path), text);
    }
    let config = load_config(&root).expect("load config");
    let findings = scan_findings(&config, &root);
    let link = markdown_link_target(
        &root.join("docs/uses.md"),
        &id_a(),
        Some("1"),
        &config,
        &findings,
    );
    (findings, link)
}

/// Premise of the stub cases: the walk's only record of `FS-a` is the stub, so
/// the target's sections are read from outside it (§FS-check.3.2.1).
fn assert_only_the_stub_is_recorded(findings: &Findings) {
    let decls = findings
        .declarations
        .get(&id_a())
        .expect("the walk records FS-a");
    assert!(
        decls.len() == 1 && decls[0].is_stub,
        "premise: the walk records FS-a only as the stub: {decls:?}"
    );
}

/// §FS-fmt.6.4.1: the reported tree. The scan ends `FS-a`'s body at
/// `# Other heading`, so the `## 1.` after it is no section of `FS-a`.
#[test]
fn a_section_heading_after_the_body_closes_gives_no_link() {
    let (findings, link) = link_of_a_1(
        "a_section_heading_after_the_body_closes_gives_no_link",
        &[("docs/a.md", CLOSED)],
    );
    let decls = findings
        .declarations
        .get(&id_a())
        .expect("the scan records FS-a");
    assert!(
        decls.iter().all(|decl| !decl.sections.contains_key("1")),
        "premise: the scan records no section 1 of FS-a: {decls:?}"
    );
    assert_eq!(
        link, None,
        "a section the scan does not record gives FS-a.1 no link"
    );
}

/// §FS-fmt.6.4.1: the same home behind a stub, outside the walk. The target is
/// read by the same scan, so its body ends where the scanned home's does.
#[test]
fn a_section_heading_after_the_body_closes_in_an_unscanned_stub_target_gives_no_link() {
    let (findings, link) = link_of_a_1(
        "a_section_heading_after_the_body_closes_in_an_unscanned_stub_target_gives_no_link",
        &[("docs/a.md", STUB), ("notes/a.md", CLOSED)],
    );
    assert_only_the_stub_is_recorded(&findings);
    assert_eq!(
        link, None,
        "a section the target's scan does not record gives FS-a.1 no link"
    );
}

/// Guard, passing today: a section inside the body of a stub's target outside
/// the walk is the section `check` finds there (§FS-check.3.2.1), and keeps its
/// link to the target's heading (§FS-fmt.6.2.1).
#[test]
fn a_real_section_in_an_unscanned_stub_target_keeps_its_link() {
    let (findings, link) = link_of_a_1(
        "a_real_section_in_an_unscanned_stub_target_keeps_its_link",
        &[("docs/a.md", STUB), ("notes/a.md", DECLARED)],
    );
    assert_only_the_stub_is_recorded(&findings);
    assert_eq!(link.as_deref(), Some("../notes/a.md#1-real-section"));
}
