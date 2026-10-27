//! Test module: the run's one kind table (§FS-scan.projects.kinds) — a verdict
//! per scope, unverifiable where the run holds no workspace, named sections read
//! under the project that configures the kind, the non-citable reason in both
//! quotings (§FS-errors.2.5.1), the first of §FS-errors.3.8.1's final templates.

use std::collections::BTreeSet;

use super::*;
use crate::config::{Config, load_config};
use crate::testing::{test_root, write};

/// A project whose `kinds` rows are `rows`, with named sections set to `named`.
fn project(name: &str, rows: &str, named: bool) -> Config {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        &format!("grund_config_version = 1\n\n[id]\nnamed_sections = {named}\n\n{rows}"),
    );
    load_config(&root).expect("load config")
}

/// One `[[kinds]]` row. `index` is written on a citable row alone: config
/// refuses it on a kind that declares nothing to index (§FS-config.3.4.2).
fn row(kind: &str, folder: &str, citable: bool) -> String {
    let index = if citable { "index = false\n" } else { "" };
    format!("[[kinds]]\nkind = \"{kind}\"\nfolder = \"{folder}\"\n{index}citable = {citable}\n\n")
}

/// `api` configures `FS` and a non-citable `skill`; `web` configures `AR`,
/// and `skill` as citable, with named sections on.
fn workspace() -> (Config, Config) {
    let api = project(
        "kinds_workspace_api",
        &(row("FS", "docs/specs", true) + &row("skill", "skills", false)),
        false,
    );
    let web = project(
        "kinds_workspace_web",
        &(row("AR", "docs/architecture", true) + &row("skill", "skills", true)),
        true,
    );
    (api, web)
}

/// §FS-scan.projects.kinds: one name, one verdict per scope — and citable wins
/// where two projects configure it differently.
#[test]
fn a_kind_has_one_verdict_per_scope() {
    let (api, web) = workspace();
    let table = KindTable::build([("api", api.schema(), 1), ("web", web.schema(), 1)]);
    let api_only = BTreeSet::from(["api".to_string()]);

    assert_eq!(
        table.verdict(KindScope::Project("api"), "FS"),
        KindVerdict::Citable
    );
    assert_eq!(
        table.verdict(KindScope::Project("web"), "FS"),
        KindVerdict::Unknown
    );
    assert!(matches!(
        table.verdict(KindScope::Project("api"), "skill"),
        KindVerdict::NonCitable(entry) if entry.home.as_deref() == Some("skills/")
    ));
    assert_eq!(
        table.verdict(KindScope::Project("web"), "skill"),
        KindVerdict::Citable
    );
    assert!(matches!(
        table.verdict(KindScope::Selected(&api_only), "skill"),
        KindVerdict::NonCitable(_)
    ));
    assert_eq!(
        table.verdict(KindScope::Selected(&BTreeSet::new()), "skill"),
        KindVerdict::Citable
    );
    assert_eq!(
        table.verdict(KindScope::Pinned("web"), "AR"),
        KindVerdict::Citable
    );
    assert_eq!(
        table.verdict(KindScope::Pinned("api"), "AR"),
        KindVerdict::Unknown
    );
    assert_eq!(
        table.verdict(KindScope::Pinned("mobile"), "AR"),
        KindVerdict::UnknownAlias
    );
    assert_eq!(table.verdict(KindScope::Any, "skill"), KindVerdict::Citable);
    assert_eq!(table.verdict(KindScope::Any, "DOC"), KindVerdict::Unknown);
    assert_eq!(
        table.known_kinds_line(KindScope::Any),
        "known kinds: FS, AR, skill"
    );
    // `skill` is citable in `web`; only the homeless kind's default name, which
    // neither project declares, is not citable (§FS-config.3.9.2).
    let non_citable: Vec<&str> = table
        .non_citable(KindScope::Any)
        .into_iter()
        .map(|entry| entry.name.as_str())
        .collect();
    assert_eq!(non_citable, ["code"], "skill is citable in `web`");
}

/// §FS-rules.4.1: with no workspace, a scope naming other projects cannot be
/// judged beyond the citable answer the one project gives.
#[test]
fn a_run_without_a_workspace_cannot_judge_other_projects() {
    let api = project(
        "kinds_standalone",
        &(row("FS", "docs/specs", true) + &row("skill", "skills", false)),
        false,
    );
    let table = KindTable::standalone(api.schema(), 1);

    assert_eq!(
        table.verdict(KindScope::Project("anything"), "FS"),
        KindVerdict::Citable
    );
    assert_eq!(table.verdict(KindScope::Any, "FS"), KindVerdict::Citable);
    assert_eq!(
        table.verdict(KindScope::Any, "skill"),
        KindVerdict::Unverifiable
    );
    assert_eq!(
        table.verdict(KindScope::Any, "DOC"),
        KindVerdict::Unverifiable
    );
    assert_eq!(
        table.verdict(KindScope::Pinned("api"), "FS"),
        KindVerdict::Unverifiable
    );
    assert!(matches!(
        table.verdict(KindScope::Project(""), "skill"),
        KindVerdict::NonCitable(_)
    ));
}

/// §FS-scan.projects.kinds: a named section of a kind follows the setting of the
/// project that configures the kind, whatever another project sets.
#[test]
fn named_sections_follow_the_configuring_project() {
    let (api, web) = workspace();
    let table = KindTable::build([("api", api.schema(), 1), ("web", web.schema(), 1)]);

    assert!(!table.named_sections(KindScope::Any));
    assert!(table.named_sections(KindScope::Project("web")));
    assert_eq!(
        table.named_section_kinds(KindScope::Any),
        BTreeSet::from(["AR", "skill"])
    );
    assert!(
        table
            .named_section_kinds(KindScope::Project("api"))
            .is_empty()
    );
}

/// §FS-errors.2.5.1: one renderer, in each surface's quoting. The double-quoted
/// form is the first of §FS-errors.3.8.1's final templates, which `0.19.0`
/// prints alone.
#[test]
fn the_non_citable_reason_renders_in_both_quotings() {
    let entry = KindEntry {
        name: "skill".to_string(),
        citable: false,
        home: Some("skills/".to_string()),
        fields: Vec::new(),
    };
    assert_eq!(
        entry.non_citable_reason(Quote::Backtick),
        "kind `skill` declares no IDs — skills/ is not a citable home"
    );
    assert_eq!(
        entry.non_citable_reason(Quote::Double),
        "kind \"skill\" declares no IDs — skills/ is not a citable home"
    );
}
