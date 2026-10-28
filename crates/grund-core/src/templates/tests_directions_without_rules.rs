//! Test module: the `### Citation directions` section of a config that declares
//! no citation rule (§FS-init.2.3.4.10). There is no rule to state, so there is
//! no section, whatever kinds the config declares and with `kinds = []` alike;
//! where grounding lands, the section is the heading and the grounding sentence
//! alone, at the root and in a scoped block.

use super::citation_directions::{root_citation_directions, scoped_citation_directions};
use super::*;
use crate::config::load_config;
use crate::testing::{test_root, write};

/// Render the whole section, every row at the root, for a config written into a
/// fresh test root; `None` where it states nothing.
fn try_render(name: &str, config: &str) -> Option<String> {
    let root = test_root(name);
    write(&root.join("grund.toml"), config);
    let config = load_config(&root).expect("load config");
    citation_directions_section(config.project(), config.run())
}

/// [`try_render`] for a config that renders a section.
fn render(name: &str, config: &str) -> String {
    try_render(name, config).expect("a Citation directions section")
}

/// §FS-init.2.3.4.10: whether a file must ground at all is not a direction
/// rule, so the sentence does not wait on `[citations]` being declared — and
/// with no rule declared it is the section's only content: no legend, no static
/// sentence, no closing line.
#[test]
fn grounding_renders_without_a_citations_section() {
    let section = render(
        "grounding_renders_without_a_citations_section",
        "[reference]\nrequire_grounding = true\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs/functional-spec\"\n",
    );
    assert_eq!(
        section,
        "### Citation directions\n\nEvery source file must cite a declared ID or declare one inline."
    );
}

/// §FS-init.2.3.4.10: a config that declares kinds and no rule states no
/// direction, in either config version, so there is no section to render.
#[test]
fn kinds_without_rules_render_no_section() {
    let v1 = "[[kinds]]\nkind = \"FS\"\nfolder = \"docs/functional-spec\"\n";
    assert_eq!(try_render("kinds_without_rules_v1", v1), None);
    let v2 = "grund_config_version = 2\n[schema.kinds.FS]\nfolders = [\"docs/functional-spec\"]\n";
    assert_eq!(try_render("kinds_without_rules_v2", v2), None);
}

/// §FS-init.2.3.4.4.2: `kinds = []` declares no rule either, so it renders what
/// every such config renders, and no "no kind is declared yet" paragraph.
#[test]
fn no_kinds_render_no_section() {
    assert_eq!(
        try_render("no_kinds_render_no_section", "kinds = []\n"),
        None
    );
}

/// §FS-init.2.3.5.12.1: a scoped block of a config without rules holds its
/// destination's grounding sentence alone, as the root does, and a destination
/// no grounding clause lands at has no section.
#[test]
fn a_scoped_block_without_rules_holds_the_grounding_sentence_alone() {
    let root = test_root("a_scoped_block_without_rules_holds_the_grounding_sentence_alone");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\n[schema.kinds.FS]\nfolders = [\"docs/fs\"]\n\
         [schema.kinds.e2e]\nfolders = [\"tests/e2e\"]\ncitable = false\n\
         [rules.citations.e2e.grounding]\nmust = \"file\"\n",
    );
    let config = load_config(&root).expect("load config");
    let (project, run) = (config.project(), config.run());
    let in_e2e = |row: &str| row == "e2e";
    assert_eq!(
        scoped_citation_directions(project, run, &in_e2e).as_deref(),
        Some("### Citation directions\n\nEvery file under tests/e2e/ must cite one.")
    );
    let at_root = |row: &str| row != "e2e";
    assert_eq!(root_citation_directions(project, run, &at_root), None);
}
