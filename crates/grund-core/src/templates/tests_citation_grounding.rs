//! Test module: the Citation directions section claims only the grounding
//! `grund check` enforces (§FS-init.2.3.5.2.2, §FS-init.2.3.5.7). A non-citable
//! home whose row grounds nothing narrows its bullet and names the setting that
//! closes the hole; a home whose own row grounds keeps the unnarrowed bullet and
//! is named in the grounding sentence. The configs are agent-grounds/grund#908's.

use super::citation_directions::CITATION_LEVEL_LEGEND;
use super::*;
use crate::config::load_config;
use crate::testing::{test_root, write};

/// The v2 config of agent-grounds/grund#908: one citable kind and the
/// non-citable `e2e` home whose files must cite it.
const V2: &str = r#"grund_config_version = 2
project_name = "demo"

[schema.kinds.FS]
files = ["requirements.md"]

[schema.kinds.e2e]
folders = ["tests/e2e"]
citable = false

[rules.citations.e2e]
must = ["FS"]
"#;

/// The same config in version 1, with `e2e_row` appended to the `e2e` row and
/// `reference` written as the `[reference]` table.
fn v1(e2e_row: &str, reference: &str) -> String {
    format!(
        "{reference}[[kinds]]\nkind = \"FS\"\nfile = \"requirements.md\"\n[[kinds]]\nkind = \"e2e\"\nfolder = \"tests/e2e\"\ncitable = false\n{e2e_row}[citations]\n[citations.e2e]\nmust = [\"FS\"]\n"
    )
}

/// Render the whole section, every row at the root, for a config written into a
/// fresh test root.
fn render(name: &str, config: &str) -> String {
    let root = test_root(name);
    write(&root.join("grund.toml"), config);
    let config = load_config(&root).expect("load config");
    citation_directions_section(config.project(), config.run()).expect("the config declares rules")
}

/// The opening paragraph when only the `tests/e2e/` home grounds: the homeless
/// kind's row grounds nothing, so the source-file clause is absent.
fn e2e_grounding_only() -> String {
    format!("{CITATION_LEVEL_LEGEND} Every file under tests/e2e/ must cite one.\n")
}

/// §FS-init.2.3.5.2.2: the ticket's config — a `must` rule and no grounding —
/// passes a file in `tests/e2e/` that cites nothing, so the bullet says so and
/// names the v2 ladder that makes it a finding; no grounding sentence renders.
#[test]
fn a_v2_home_that_grounds_nothing_narrows_its_bullet_and_names_the_ladder() {
    let section = render("v2_home_grounds_nothing", V2);
    assert!(
        section.contains(
            "- Each file in **tests/e2e/** that cites anything must cite FS. A file there that cites nothing is not checked; `[rules.citations.e2e.grounding] must = \"file\"` makes it a finding.\n"
        ),
        "{section}"
    );
    assert!(
        !section.contains("- Each file in **tests/e2e/** must cite FS."),
        "{section}"
    );
    assert!(!section.contains("must cite one"), "{section}");
}

/// §FS-init.2.3.5.7: a row's own `[rules.citations.<row>.grounding]` ladder is
/// its effective grounding, so the bullet stays unnarrowed and the sentence
/// names the home — and only the home, since the homeless kind grounds nothing.
#[test]
fn a_v2_grounding_ladder_on_the_row_keeps_the_bullet_and_names_the_home() {
    let section = render(
        "v2_home_grounding_ladder",
        &format!("{V2}\n[rules.citations.e2e.grounding]\nmust = \"file\"\n"),
    );
    assert!(section.contains(&e2e_grounding_only()), "{section}");
    assert!(
        section.contains("- Each file in **tests/e2e/** must cite FS.\n"),
        "{section}"
    );
    assert!(!section.contains("Every source file"), "{section}");
}

/// §FS-init.2.3.5.7: the version-1 row key is the same switch as the v2 ladder.
#[test]
fn a_v1_row_require_grounding_keeps_the_bullet_and_names_the_home() {
    let section = render(
        "v1_home_require_grounding",
        &v1("require_grounding = true\n", ""),
    );
    assert!(section.contains(&e2e_grounding_only()), "{section}");
    assert!(
        section.contains("- Each file in **tests/e2e/** must cite FS.\n"),
        "{section}"
    );
    assert!(!section.contains("Every source file"), "{section}");
}

/// §FS-init.2.3.5.2.2: the setting is spelled in the live config's version.
#[test]
fn a_v1_home_that_grounds_nothing_names_the_v1_key() {
    let section = render("v1_home_grounds_nothing", &v1("", ""));
    assert!(
        section.contains(
            "- Each file in **tests/e2e/** that cites anything must cite FS. A file there that cites nothing is not checked; `require_grounding = true` on its `[[kinds]]` row makes it a finding.\n"
        ),
        "{section}"
    );
}

/// §FS-init.2.3.5.7, §FS-config.3.4.8.3: the row wins over the global default, so
/// a row that turns grounding off under a global `true` is left out of the
/// sentence and narrows its bullet, while the source-file clause stays.
#[test]
fn a_row_that_turns_grounding_off_under_a_global_default_narrows_its_bullet() {
    let section = render(
        "v1_home_grounding_off_under_global",
        &v1(
            "require_grounding = false\n",
            "[reference]\nrequire_grounding = true\n",
        ),
    );
    assert!(
        section.contains(&format!(
            "{CITATION_LEVEL_LEGEND} Every source file must cite a declared ID or declare one inline.\n"
        )),
        "{section}"
    );
    assert!(
        section.contains("- Each file in **tests/e2e/** that cites anything must cite FS. A file there that cites nothing is not checked;"),
        "{section}"
    );
}

/// §FS-init.2.3.5.2: the single-file and several-file homes take their own
/// grounding-off subjects and second sentences.
#[test]
fn file_homes_that_ground_nothing_take_their_own_forms() {
    let section = render(
        "v2_file_homes_ground_nothing",
        r#"grund_config_version = 2
project_name = "demo"

[schema.kinds.FS]
files = ["requirements.md"]

[schema.kinds.runbook]
files = ["docs/runbook.md"]
citable = false

[schema.kinds.guide]
files = ["docs/a.md", "docs/b.md"]
citable = false

[rules.citations.runbook]
must = ["FS"]

[rules.citations.guide]
must = ["FS"]
"#,
    );
    assert!(
        section.contains(
            "- The file **docs/runbook.md**, if it cites anything, must cite FS. If it cites nothing, it is not checked; `[rules.citations.runbook.grounding] must = \"file\"` makes it a finding.\n"
        ),
        "{section}"
    );
    assert!(
        section.contains(
            "- Each of the files **docs/a.md, docs/b.md** that cites anything must cite FS. One that cites nothing is not checked; `[rules.citations.guide.grounding] must = \"file\"` makes it a finding.\n"
        ),
        "{section}"
    );
}

/// §FS-init.2.3.5.2.2: a `warn` rung reaches the home's files, so the row does
/// not ground nothing and its bullet is not narrowed. A guard: it holds today.
#[test]
fn a_soft_grounding_rung_keeps_the_bullet_unnarrowed() {
    let section = render(
        "v2_home_warn_ladder",
        &format!("{V2}\n[rules.citations.e2e.grounding]\nwarn = \"file\"\n"),
    );
    assert!(
        section.contains("- Each file in **tests/e2e/** must cite FS.\n"),
        "{section}"
    );
    assert!(!section.contains("is not checked"), "{section}");
}
