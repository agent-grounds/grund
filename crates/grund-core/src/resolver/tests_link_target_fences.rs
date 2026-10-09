//! Test module: the anchor of a `.<section>` citation names a section the scan
//! records, so a heading inside a fence of the declaration's Markdown home gives
//! no anchor (§FS-fmt.6.4.1). The anchor is the one `fmt --cross-refs` writes and
//! `show --format=json` reports (§FS-fmt.6.2.1).
//!
//! Each case states its premise first: whether the scan records the cited
//! section. A section it does not record gives no anchor, and that is what these
//! cases pin; a case the scan reads differently is about the scanner, not about
//! the anchor.

use super::heading_anchor;
use crate::config::load_config;
use crate::model::Id;
use crate::testing::{scan_findings, test_root, write};

const CONFIG: &str = "grund_config_version = 1\n\n[reference]\nstrict = true\n\
require_grounding = false\n\n[id]\nformat = \"{kind}-{slug}\"\n\n[[kinds]]\n\
kind = \"FS\"\nfolder = \"docs\"\nindex = false\n\n[scan]\ninclude = [\"docs\"]\n";

fn id(slug: &str) -> Id {
    Id {
        kind: "FS".to_string(),
        num: None,
        slug: Some(slug.to_string()),
    }
}

/// `docs/a.md` holding `home`, scanned; then the anchor a citation of `FS-a.1`
/// takes there, and whether the scan recorded section `1` of `FS-a`.
fn anchor_of_a_1(name: &str, home: &str) -> (bool, Option<String>) {
    let root = test_root(name);
    write(&root.join("grund.toml"), CONFIG);
    write(&root.join("docs/a.md"), home);
    let config = load_config(&root).expect("load config");
    let findings = scan_findings(&config, &root);
    let id = id("a");
    let decls = findings
        .declarations
        .get(&id)
        .expect("the scan records FS-a");
    assert_eq!(decls.len(), 1, "one declaration of FS-a: {decls:?}");
    let decl = &decls[0];
    let recorded = decl.sections.contains_key("1");
    (recorded, heading_anchor(decl, Some("1"), &config))
}

fn assert_no_anchor(name: &str, home: &str) {
    let (recorded, anchor) = anchor_of_a_1(name, home);
    assert!(
        !recorded,
        "premise: the scan records no section 1 of FS-a in {home:?}"
    );
    assert_eq!(
        anchor, None,
        "a section the scan does not record gives FS-a.1 no anchor"
    );
}

/// §FS-fmt.6.4.1: the reported tree, a section heading only inside a backtick
/// fence of the declaration's body.
#[test]
fn a_section_heading_only_inside_a_backtick_fence_gives_no_anchor() {
    assert_no_anchor(
        "a_section_heading_only_inside_a_backtick_fence_gives_no_anchor",
        "# FS-a: A\n\nLead.\n\n```markdown\n## 1. Fenced section\n\nbody\n```\n",
    );
}

/// §FS-fmt.6.4.1: the same with a tilde fence, which the scan reads alike.
#[test]
fn a_section_heading_only_inside_a_tilde_fence_gives_no_anchor() {
    assert_no_anchor(
        "a_section_heading_only_inside_a_tilde_fence_gives_no_anchor",
        "# FS-a: A\n\nLead.\n\n~~~markdown\n## 1. Fenced section\n\nbody\n~~~\n",
    );
}

/// §FS-fmt.6.4.1: a fence left open runs to the end of the file, so a section
/// heading after its opener is still inside it.
#[test]
fn a_section_heading_inside_an_unclosed_fence_gives_no_anchor() {
    assert_no_anchor(
        "a_section_heading_inside_an_unclosed_fence_gives_no_anchor",
        "# FS-a: A\n\nLead.\n\n```markdown\n## 1. Fenced section\n\nbody\n",
    );
}

/// §FS-fmt.6.4.1: a fenced copy of `# FS-a: A` inside `FS-b`'s body opens
/// nothing, so `FS-b`'s own `## 1.` after it is no section of `FS-a`.
#[test]
fn a_fenced_copy_of_the_cited_heading_does_not_open_its_declaration() {
    assert_no_anchor(
        "a_fenced_copy_of_the_cited_heading_does_not_open_its_declaration",
        "# FS-b: B\n\nB lead.\n\n```markdown\n# FS-a: A\n```\n\n\
         ## 1. B section\n\nB body.\n\n# FS-a: A\n\nA lead.\n",
    );
}

/// Guard, passing today: a real section after a fenced example of it is the
/// section the scan records, and keeps its anchor (§FS-fmt.6.2.1).
#[test]
fn a_real_section_after_a_fenced_example_keeps_its_anchor() {
    let home = "# FS-a: A\n\nLead.\n\n```markdown\n## 1. Fenced section\n```\n\n\
                ## 1. Real section\n\nbody\n";
    let (recorded, anchor) = anchor_of_a_1(
        "a_real_section_after_a_fenced_example_keeps_its_anchor",
        home,
    );
    assert!(recorded, "premise: the scan records section 1 of FS-a");
    assert_eq!(anchor.as_deref(), Some("1-real-section"));
}
