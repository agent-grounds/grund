//! Test module: the Citation directions section claims only the grounding
//! `grund check` enforces (§FS-init.2.3.5.2.2, §FS-init.2.3.5.7). A non-citable
//! home whose row grounds nothing narrows its bullet and names the setting that
//! closes the hole; a home whose own row grounds keeps the unnarrowed bullet and
//! is named in the grounding sentence. The configs are agent-grounds/grund#908's;
//! a row closed to every citation, agent-grounds/grund#1456's, is neither
//! narrowed nor hinted (§FS-config.3.9.4.1).

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

/// Each non-citable place shape: its name, version, schema rows, row name and
/// the subject its bullet opens with. A several-file or several-place home is a
/// version-2 row, so no version-1 config renders one.
///
/// The cases below pin agent-grounds/grund#1456: a row closed to every citation
/// (§FS-config.3.9.4.1) is never narrowed, never carries the grounding sentence,
/// and renders the one clause `must not cite anything` (§FS-init.2.3.5.2.2,
/// §FS-init.2.3.5.5), in every place shape and both versions.
const SHAPES: &[Shape] = &[
    (
        "v1_folder",
        1,
        "[[kinds]]\nkind = \"ce\"\nfolder = \"ce\"\ncitable = false\n",
        "ce",
        "Each file in **ce/**",
    ),
    (
        "v1_file",
        1,
        "[[kinds]]\nkind = \"runbook\"\nfile = \"docs/runbook.md\"\ncitable = false\n",
        "runbook",
        "The file **docs/runbook.md**",
    ),
    (
        "v2_folder",
        2,
        "[schema.kinds.ce]\nfolders = [\"ce\"]\ncitable = false\n",
        "ce",
        "Each file in **ce/**",
    ),
    (
        "v2_file",
        2,
        "[schema.kinds.runbook]\nfiles = [\"docs/runbook.md\"]\ncitable = false\n",
        "runbook",
        "The file **docs/runbook.md**",
    ),
    (
        "v2_files",
        2,
        "[schema.kinds.guide]\nfiles = [\"docs/a.md\", \"docs/b.md\"]\ncitable = false\n",
        "guide",
        "Each of the files **docs/a.md, docs/b.md**",
    ),
    (
        "v2_places",
        2,
        "[schema.kinds.ce]\nfolders = [\"ce\", \"cf\"]\nfiles = [\"docs/a.md\"]\ncitable = false\n",
        "ce",
        "Each file in **docs/a.md, ce/, cf/**",
    ),
];

/// The homeless kind, with and without a `title`, as [`SHAPES`] lays a shape out.
const HOMELESS: &[Shape] = &[
    (
        "v1_code",
        1,
        "[[kinds]]\nkind = \"code\"\ncitable = false\n",
        "code",
        "Each source file outside the Project map (**code**)",
    ),
    (
        "v1_code_title",
        1,
        "[[kinds]]\nkind = \"code\"\ncitable = false\ntitle = \"Gradle build and workflows\"\n",
        "code",
        "Each source file outside the Project map (**code**: Gradle build and workflows)",
    ),
    (
        "v2_code",
        2,
        "[schema.kinds.code]\ncitable = false\n",
        "code",
        "Each source file outside the Project map (**code**)",
    ),
    (
        "v2_code_title",
        2,
        "[schema.kinds.code]\ncitable = false\n[presentation.kinds.code]\ntitle = \"Gradle build and workflows\"\n",
        "code",
        "Each source file outside the Project map (**code**: Gradle build and workflows)",
    ),
];

/// A config of `version` declaring `FS`, then `rows`, the row `row` ruled by
/// `rule`, the global default `global` where one is given, and the row's
/// grounding on where `grounded`: version 1's row key, version 2's ladder.
fn ruled(
    version: u32,
    rows: &str,
    row: &str,
    rule: &str,
    global: Option<&str>,
    grounded: bool,
) -> String {
    if version == 1 {
        let grounding = if grounded {
            "require_grounding = true\n"
        } else {
            ""
        };
        let global = global.map_or_else(String::new, |level| format!("default = \"{level}\"\n"));
        format!(
            "[[kinds]]\nkind = \"FS\"\nfile = \"requirements.md\"\n{rows}{grounding}\
             [citations]\n{global}[citations.{row}]\n{rule}"
        )
    } else {
        let global = global.map_or_else(String::new, |level| {
            format!("[rules.citations]\ndefault = \"{level}\"\n")
        });
        let ladder = if grounded {
            format!("[rules.citations.{row}.grounding]\nmust = \"file\"\n")
        } else {
            String::new()
        };
        format!(
            "grund_config_version = 2\nproject_name = \"demo\"\n\n[schema.kinds.FS]\n\
             files = [\"requirements.md\"]\n{rows}{global}[rules.citations.{row}]\n{rule}{ladder}"
        )
    }
}

/// The bullet `config` renders for `subject`, or `None` where it renders none.
fn bullet(name: &str, config: &str, subject: &str) -> Option<String> {
    let section = render(name, config);
    let opening = format!("- {subject}");
    section
        .lines()
        .find(|line| line.starts_with(&opening))
        .map(str::to_string)
}

/// One place shape: its name, version, schema rows, row name and subject.
type Shape<'a> = (&'a str, u32, &'a str, &'a str, &'a str);

/// Every shape, under `rule` and `global` with each grounding in `grounding`,
/// whose bullet is not `- <subject> <clauses>.`, with the bullet it renders.
fn wrong_bullets(
    shapes: &[Shape],
    rule: &str,
    global: Option<&str>,
    grounding: &[bool],
    clauses: &str,
) -> Vec<String> {
    let mut wrong = Vec::new();
    for &grounded in grounding {
        for (name, version, rows, row, subject) in shapes {
            let config = ruled(*version, rows, row, rule, global, grounded);
            let observed = bullet(&format!("closed_{name}"), &config, subject);
            if observed != Some(format!("- {subject} {clauses}.")) {
                wrong.push(format!("{name} (grounding {grounded}): {observed:?}"));
            }
        }
    }
    wrong
}

/// Asserts [`wrong_bullets`] finds none, naming every shape it does find.
fn assert_bullets(
    shapes: &[Shape],
    rule: &str,
    global: Option<&str>,
    grounding: &[bool],
    clauses: &str,
) {
    let wrong = wrong_bullets(shapes, rule, global, grounding, clauses);
    assert!(
        wrong.is_empty(),
        "expected `... {clauses}.`, rendered:\n{}",
        wrong.join("\n")
    );
}

/// The closed rule agent-grounds/grund#1456 writes.
const CLOSED: &str = "default = \"must-not\"\n";

/// §FS-init.2.3.5.2.2, §FS-config.3.9.4.1: a closed home that grounds nothing
/// takes no narrowed subject and no sentence naming a grounding setting, in
/// every place shape and both versions. Today each reads `… that cites anything
/// must not cite anything. A file there that cites nothing is not checked; …`.
#[test]
#[ignore = "red until #1456 renders a closed row as must not cite anything; implement removes this"]
fn a_closed_home_that_grounds_nothing_renders_one_unnarrowed_sentence() {
    assert_bullets(SHAPES, CLOSED, None, &[false], "must not cite anything");
}

/// §FS-init.2.3.5.2.2: with its grounding on, a closed home whose only rule is
/// its `must-not` default keeps the bullet it has, the same sentence.
#[test]
fn a_closed_home_that_grounds_keeps_its_bullet() {
    assert_bullets(SHAPES, CLOSED, None, &[true], "must not cite anything");
}

/// §FS-init.2.3.5.2.1: the homeless kind closed to every citation is not
/// narrowed, with or without a `title`, grounding off or on. Today it reads
/// `Each source file outside the Project map (**code**) that cites anything …`.
#[test]
#[ignore = "red until #1456 renders a closed row as must not cite anything; implement removes this"]
fn the_closed_homeless_kind_is_never_narrowed() {
    let clause = "must not cite anything";
    let subject = "Each source file outside the Project map (**code**)";
    let undeclared = [
        ("v1_undeclared", 1, "", "code", subject),
        ("v2_undeclared", 2, "", "code", subject),
    ];
    let mut wrong = wrong_bullets(HOMELESS, CLOSED, None, &[false, true], clause);
    wrong.extend(wrong_bullets(&undeclared, CLOSED, None, &[false], clause));
    assert!(
        wrong.is_empty(),
        "expected `... {clause}.`, rendered:\n{}",
        wrong.join("\n")
    );
}

/// §FS-init.2.3.5.5: a closed row's own `must-not` list folds into its default's
/// clause, grounding off or on. Today it reads `must not cite FS; never cite
/// anything else`, narrowed where the row grounds nothing.
#[test]
#[ignore = "red until #1456 renders a closed row as must not cite anything; implement removes this"]
fn a_closed_row_folds_its_own_must_not_list() {
    let rule = "default = \"must-not\"\nmust-not = [\"FS\"]\n";
    assert_bullets(SHAPES, rule, None, &[false, true], "must not cite anything");
}

/// §FS-config.3.9.4.1, §FS-init.2.3.5.5: a row whose own rules are `must-not`
/// lists alone is closed by a `must-not` global default it inherits, and folds.
/// Today it reads `must not cite FS`, narrowed where the row grounds nothing.
#[test]
#[ignore = "red until #1456 renders a closed row as must not cite anything; implement removes this"]
fn a_row_closed_by_an_inherited_default_folds_its_list() {
    let rule = "must-not = [\"FS\"]\n";
    assert_bullets(
        SHAPES,
        rule,
        Some("must-not"),
        &[false, true],
        "must not cite anything",
    );
}

/// The citable kind `AR`, in each version, as [`SHAPES`] lays a shape out.
const CITABLE: &[Shape] = &[
    (
        "v1_citable",
        1,
        "[[kinds]]\nkind = \"AR\"\nfolder = \"docs/ar\"\n",
        "AR",
        "Each **AR** declaration",
    ),
    (
        "v2_citable",
        2,
        "[schema.kinds.AR]\nfolders = [\"docs/ar\"]\n",
        "AR",
        "Each **AR** declaration",
    ),
];

/// §FS-init.2.3.5.5: the fold is the row's, not its place's, so a citable row
/// closed with a `must-not` list of its own folds too. Today it reads `Each **AR**
/// declaration must not cite FS; never cite anything else.`
#[test]
#[ignore = "red until #1456 renders a closed row as must not cite anything; implement removes this"]
fn a_closed_citable_row_folds_its_list() {
    let rule = "default = \"must-not\"\nmust-not = [\"FS\"]\n";
    assert_bullets(CITABLE, rule, None, &[false], "must not cite anything");
}

/// §FS-init.2.3.5.5: a citable row closed by its default alone keeps its bullet.
#[test]
fn a_closed_citable_row_with_its_default_alone_keeps_its_bullet() {
    assert_bullets(CITABLE, CLOSED, None, &[false], "must not cite anything");
}

/// §FS-init.2.3.5.5, §FS-init.2.3.5.1: the fold gives no row a bullet. A row with no
/// rule of its own, closed by the global default alone, renders none, and the
/// closing line states the default.
#[test]
fn a_row_closed_by_the_global_default_alone_has_no_bullet() {
    for version in [1, 2] {
        let rows = SHAPES
            .iter()
            .find(|shape| shape.1 == version)
            .map(|shape| shape.2)
            .expect("a folder shape");
        let config = ruled(
            version,
            rows,
            "FS",
            "may = [\"FS\"]\n",
            Some("must-not"),
            false,
        );
        let section = render(&format!("closed_by_global_v{version}"), &config);
        assert!(!section.contains("- Each file in **ce/**"), "{section}");
        assert!(
            section.ends_with("Any citation not listed above is forbidden."),
            "{section}"
        );
    }
}

/// §FS-init.2.3.5.5, §FS-init.2.3.5.2.2: rows that are not closed keep their
/// bullets byte for byte — a discouraged default, a closed default with a `may`,
/// `should-not` or `warn-not` exception, a `warn-not` default, an open `should`
/// and agent-grounds/grund#908's obligation — narrowed, with the sentence that
/// names the grounding setting, where they ground nothing.
#[test]
fn rows_that_are_not_closed_keep_their_bullets() {
    let cases: &[(u32, &str, &str)] = &[
        (1, "default = \"should-not\"\n", "should not cite anything"),
        (
            1,
            "default = \"must-not\"\nmay = [\"FS\"]\n",
            "may cite only FS",
        ),
        (
            1,
            "default = \"must-not\"\nshould-not = [\"FS\"]\n",
            "should not cite FS; never cite anything else",
        ),
        (1, "should = [\"FS\"]\n", "should cite FS"),
        (1, "must = [\"FS\"]\n", "must cite FS"),
        (2, "default = \"should-not\"\n", "should not cite anything"),
        (
            2,
            "default = \"must-not\"\nmay = [\"FS\"]\n",
            "may cite only FS",
        ),
        (
            2,
            "default = \"must-not\"\nwarn-not = [\"FS\"]\n",
            "must not cite FS (a warning); never cite anything else",
        ),
        (
            2,
            "default = \"warn-not\"\n",
            "must not cite anything (a warning)",
        ),
        (2, "must = [\"FS\"]\n", "must cite FS"),
    ];
    let setting = |version: u32| {
        if version == 1 {
            "`require_grounding = true` on its `[[kinds]]` row".to_string()
        } else {
            "`[rules.citations.ce.grounding] must = \"file\"`".to_string()
        }
    };
    for (index, (version, rule, clauses)) in cases.iter().enumerate() {
        let rows = SHAPES
            .iter()
            .find(|shape| shape.1 == *version)
            .map(|shape| shape.2)
            .expect("a folder shape");
        let config = ruled(*version, rows, "ce", rule, None, false);
        assert_eq!(
            bullet(&format!("open_{index}"), &config, "Each file in **ce/**"),
            Some(format!(
                "- Each file in **ce/** that cites anything {clauses}. A file there that cites \
                 nothing is not checked; {} makes it a finding.",
                setting(*version)
            )),
            "{config}"
        );
    }
}
