//! Test module: a `.<section>` citation of a section its declaration does not
//! have gives no link where the link would take no heading anchor - a source-file
//! home behind a stub, walked or not, and any home under the `none` profile - as it
//! gives none where the link would carry one (§FS-fmt.6.4.1). On the same homes the
//! section the declaration has and the bare ID keep their file link with no
//! fragment (§FS-fmt.6.2, §FS-fmt.6.7.1).
//!
//! Each case asks `markdown_link_target`, the derivation `fmt` calls, or its
//! workspace form, so where the existence question sits stays the implementation's
//! choice. Each states its premise first: what the walk records of `FS-a`.

use super::{load_workspace_context, markdown_link_target, markdown_link_target_with_root};
use crate::config::load_config;
use crate::model::{Findings, Id};
use crate::testing::{scan_findings, test_root, write};

const CONFIG: &str = "grund_config_version = 1\n\n[reference]\nstrict = true\n\
require_grounding = false\n\n[id]\nformat = \"{kind}-{slug}\"\n\n[[kinds]]\n\
kind = \"FS\"\nfolder = \"docs\"\nindex = false\n\n[scan]\ninclude = [\"docs\"]\n";

/// What `CONFIG` gains to put the project on the `none` profile.
const NONE_PROFILE: &str = "\n[fmt.cross_refs]\nanchor_format = \"none\"\n";

/// A Markdown home with a section 1 and no section 2.
const MARKDOWN: &str = "# FS-a: A\n\nLead.\n\n## 1. Detail\n\nbody\n";

/// grund#546's home: a plain heading closes `FS-a`'s body, and `## 1.` follows it.
const CLOSED: &str = "# FS-a: A\n\nLead.\n\n# Other heading\n\n## 1. After close\n\nbody\n";

/// A source-file home whose doc-comment declares a section 1 and no section 2.
const SOURCE: &str = "/// FS-a: A\n///\n/// Lead.\n///\n/// ## 1. Detail\n///\n/// body\n\
pub fn a() {}\n";

/// A stub in the walk to the source-file home.
const STUB: &str = "# FS-a: [../src/a.rs](../src/a.rs)\n";

fn id_a() -> Id {
    Id {
        kind: "FS".to_string(),
        num: None,
        slug: Some("a".to_string()),
    }
}

/// The links a tree's `docs/uses.md` takes for `FS-a.2`, `FS-a.1` and `FS-a`,
/// in that order.
struct Links {
    missing: Option<String>,
    existing: Option<String>,
    bare: Option<String>,
}

/// The tree `config` and `files` write, scanned; then what the walk records, and
/// the links `docs/uses.md` takes.
fn links_of_a(name: &str, config: &str, files: &[(&str, &str)]) -> (Findings, Links) {
    let root = test_root(name);
    write(&root.join("grund.toml"), config);
    write(&root.join("docs/uses.md"), "Uses \u{a7}FS-a.2.\n");
    for (path, text) in files {
        write(&root.join(path), text);
    }
    let config = load_config(&root).expect("load config");
    let findings = scan_findings(&config, &root);
    let from = root.join("docs/uses.md");
    let link = |section| markdown_link_target(&from, &id_a(), section, &config, &findings);
    let links = Links {
        missing: link(Some("2")),
        existing: link(Some("1")),
        bare: link(None),
    };
    (findings, links)
}

/// Premise: the walk records `FS-a`'s home with a section 1 and no section 2.
fn assert_home_has_only_section_1(findings: &Findings) {
    let decls = findings
        .declarations
        .get(&id_a())
        .expect("the walk records FS-a");
    let home = decls
        .iter()
        .find(|decl| !decl.is_stub)
        .expect("the walk records FS-a's home");
    assert!(
        home.sections.contains_key("1") && !home.sections.contains_key("2"),
        "premise: the scan records section 1 of FS-a and no section 2: {decls:?}"
    );
}

/// Premise of the unwalked stub case: the walk's only record of `FS-a` is the
/// stub, so the target's sections are read from outside it (§FS-check.3.2.1).
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

/// §FS-fmt.6.4.1: a source-file home behind a stub, outside `[scan] include`.
/// `check` reads its sections from the target (§FS-check.3.2.1) and reports
/// `FS-a.2` missing, so `fmt` gives it no link.
#[test]
fn a_missing_section_of_an_unwalked_source_home_gives_no_link() {
    let (findings, links) = links_of_a(
        "a_missing_section_of_an_unwalked_source_home_gives_no_link",
        CONFIG,
        &[("docs/a.md", STUB), ("src/a.rs", SOURCE)],
    );
    assert_only_the_stub_is_recorded(&findings);
    assert_eq!(
        links.missing, None,
        "a section the source home does not have gives FS-a.2 no link"
    );
}

/// §FS-fmt.6.4.1: the same home with `src` inside `[scan] include`, so the walk
/// records it.
#[test]
fn a_missing_section_of_a_walked_source_home_gives_no_link() {
    let config = CONFIG.replace("[\"docs\"]", "[\"docs\", \"src\"]");
    let (findings, links) = links_of_a(
        "a_missing_section_of_a_walked_source_home_gives_no_link",
        &config,
        &[("docs/a.md", STUB), ("src/a.rs", SOURCE)],
    );
    assert_home_has_only_section_1(&findings);
    assert_eq!(
        links.missing, None,
        "a section the source home does not have gives FS-a.2 no link"
    );
}

/// §FS-fmt.6.4.1: a Markdown home under the `none` profile, whose link would carry
/// no fragment (§FS-fmt.6.7.1).
#[test]
fn a_missing_section_under_the_none_profile_gives_no_link() {
    let (findings, links) = links_of_a(
        "a_missing_section_under_the_none_profile_gives_no_link",
        &format!("{CONFIG}{NONE_PROFILE}"),
        &[("docs/a.md", MARKDOWN)],
    );
    assert_home_has_only_section_1(&findings);
    assert_eq!(
        links.missing, None,
        "a section the Markdown home does not have gives FS-a.2 no link under `none`"
    );
}

/// §FS-fmt.6.4.1: grund#546's closed-body home under the `none` profile. The scan
/// ends the body at `# Other heading`, so the `## 1.` after it is no section of
/// `FS-a`, and no anchor is asked for to find that out.
#[test]
fn a_section_heading_after_the_body_closes_gives_no_link_under_the_none_profile() {
    let root = test_root("a_section_heading_after_the_body_closes_gives_no_link_under_none");
    write(&root.join("grund.toml"), &format!("{CONFIG}{NONE_PROFILE}"));
    write(&root.join("docs/a.md"), CLOSED);
    let config = load_config(&root).expect("load config");
    let findings = scan_findings(&config, &root);
    let decls = findings
        .declarations
        .get(&id_a())
        .expect("the scan records FS-a");
    assert!(
        decls.iter().all(|decl| !decl.sections.contains_key("1")),
        "premise: the scan records no section 1 of FS-a: {decls:?}"
    );
    let link = markdown_link_target(
        &root.join("docs/uses.md"),
        &id_a(),
        Some("1"),
        &config,
        &findings,
    );
    assert_eq!(
        link, None,
        "a section the scan does not record gives FS-a.1 no link under `none`"
    );
}

/// §FS-fmt.6.4.1, §FS-workspace.8.5: a qualified citation across projects, into a
/// member on the `none` profile, takes its path from the workspace root and its
/// sections from the member's scan, so the missing one gives no link there either.
#[test]
fn a_missing_section_across_projects_under_the_none_profile_gives_no_link() {
    let root = test_root("a_missing_section_across_projects_under_the_none_profile");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n[id]\nformat = \"{kind}-{slug}\"\n\n\
[workspace]\nmembers = [\"apps/api\"]\n",
    );
    write(&root.join("docs/uses.md"), "Uses \u{a7}api/FS-a.2.\n");
    write(
        &root.join("apps/api/grund.toml"),
        &format!("{CONFIG}{NONE_PROFILE}"),
    );
    write(&root.join("apps/api/docs/a.md"), MARKDOWN);
    let context = load_workspace_context(&root, true).expect("load workspace context");
    let api = context.project_by_alias("api").expect("the api member");
    assert_home_has_only_section_1(&api.findings);
    let link = |section| {
        markdown_link_target_with_root(
            &root.join("docs/uses.md"),
            &id_a(),
            section,
            &api.config,
            &api.findings,
            Some(&context.render_root),
        )
    };
    assert_eq!(
        link(Some("1")).as_deref(),
        Some("../apps/api/docs/a.md"),
        "guard: the section the member's home has keeps its file link"
    );
    assert_eq!(
        link(Some("2")),
        None,
        "a section the member's home does not have gives api/FS-a.2 no link"
    );
}

/// Guard, passing today: on each no-anchor home the section the declaration has
/// and the bare ID still link to the file, with no fragment (§FS-fmt.6.2,
/// §FS-fmt.6.7.1).
#[test]
fn an_existing_section_and_the_bare_id_keep_their_file_link() {
    let walked: &str = &CONFIG.replace("[\"docs\"]", "[\"docs\", \"src\"]");
    let none: &str = &format!("{CONFIG}{NONE_PROFILE}");
    let source: &[(&str, &str)] = &[("docs/a.md", STUB), ("src/a.rs", SOURCE)];
    let markdown: &[(&str, &str)] = &[("docs/a.md", MARKDOWN)];
    let homes = [
        ("guard_unwalked_source", CONFIG, source, "../src/a.rs"),
        ("guard_walked_source", walked, source, "../src/a.rs"),
        ("guard_none_profile", none, markdown, "a.md"),
    ];
    for (name, config, files, file_link) in homes {
        let (_, links) = links_of_a(name, config, files);
        assert_eq!(links.existing.as_deref(), Some(file_link), "{name}: FS-a.1");
        assert_eq!(links.bare.as_deref(), Some(file_link), "{name}: FS-a");
    }
}
