//! Test module: whether a rule kind's `file` home is Markdown is the reader's
//! question (§FS-config-v2.schema.sources.rules) — its name in version 1, the language
//! `[schema.sources]` maps it to in version 2 — refused at `form` with the
//! message of §FS-config.3.4.12 when the answer is no.

use super::migrate::{GateResult, Located, Prepared, translate};
use super::*;
use crate::testing::{test_root, write};

const REFUSAL: &str = "kind `RULE` sets `rules = true` but rule kinds must be citable, scanned Markdown kinds with a `file` or `folder` home and without `values` or `fetch`";

/// A v2 file whose `form = "rule"` kind has the one `file` home `home`, under
/// `sources` (the `[schema.sources]` body); `form` is its last line.
fn v2(sources: &str, home: &str) -> String {
    format!(
        "grund_config_version = 2\nproject_name = \"g179\"\n[schema.sources]\n{sources}\
         [schema.kinds.RULE]\nfiles = [\"{home}\"]\nform = \"rule\"\n"
    )
}

fn load(name: &str, toml: &str, home: &str) -> anyhow::Result<Config> {
    let root = test_root(name);
    write(&root.join("grund.toml"), toml);
    write(&root.join(home), "");
    load_config(&root)
}

fn assert_refused_at_form(name: &str, toml: &str, home: &str) {
    let err = match load(name, toml, home) {
        Ok(_) => panic!("{name}: the rule kind should be refused"),
        Err(err) => format!("{err:#}"),
    };
    let form = toml.lines().count();
    let at = format!("grund.toml:{form}: {REFUSAL}");
    assert!(err.contains(&at), "{name}: expected `{at}`, got: {err}");
}

fn assert_rule_kind_loads(name: &str, toml: &str, home: &str) {
    let config = match load(name, toml, home) {
        Ok(config) => config,
        Err(err) => panic!("{name}: the rule kind should load: {err:#}"),
    };
    assert!(
        config
            .kinds
            .iter()
            .any(|kind| kind.kind == "RULE" && kind.rules),
        "{name}: RULE should load as a rule kind"
    );
}

/// §FS-config-v2.checks.config-rule-file-not-markdown: the row loads, and `home` is warned about
/// at its `files` line, the one before `form`.
fn assert_rule_home_warns(name: &str, toml: &str, home: &str) {
    assert_rule_kind_loads(name, toml, home);
    let config = load(name, toml, home).expect("loaded above");
    let files = toml.lines().count() - 1;
    let warned = config.project().load_warnings.0.iter().any(|warning| {
        warning.code == crate::model::Code::CONFIG_RULE_FILE_NOT_MARKDOWN
            && warning.at.line == files
            && warning
                .message
                .contains(&format!("`{home}` is read as `rust`"))
    });
    assert!(
        warned,
        "{name}: expected config-rule-file-not-markdown at line {files}"
    );
}

const MD_AS_JSON: &str =
    "languages = [\"markdown\", \"json\"]\n[schema.sources.extensions]\nmd = \"json\"\n";

/// §FS-config-v2.schema.sources.rules: the `.json` control — read as JSON, refused at `form`.
#[test]
fn v2_rule_home_read_as_json_is_refused() {
    let toml = v2(MD_AS_JSON, "rules/rules.json");
    assert_refused_at_form("v2_rule_home_json", &toml, "rules/rules.json");
}

/// §FS-config-v2.schema.sources.rules: issue #1149 — a `.md` home `md = "json"` reads as JSON
/// is refused exactly like the `.json` control.
#[test]
fn v2_rule_home_md_read_as_json_is_refused() {
    let toml = v2(MD_AS_JSON, "rules/rules.md");
    assert_refused_at_form("v2_rule_home_md_as_json", &toml, "rules/rules.md");
}

/// §FS-config-v2.schema.sources.rules: a `.md` home no admitted language reads is refused.
#[test]
fn v2_rule_home_md_read_by_no_language_is_refused() {
    let toml = v2("languages = [\"json\"]\n", "rules/rules.md");
    assert_refused_at_form("v2_rule_home_md_unread", &toml, "rules/rules.md");
}

/// §FS-config-v2.schema.sources.rules: the converse — a `.txt` home `txt = "markdown"` reads as
/// Markdown is a rule kind.
#[test]
fn v2_rule_home_txt_read_as_markdown_loads() {
    let sources = "languages = [\"markdown\"]\n[schema.sources.extensions]\ntxt = \"markdown\"\n";
    let toml = v2(sources, "rules/rules.txt");
    assert_rule_kind_loads("v2_rule_home_txt_as_markdown", &toml, "rules/rules.txt");
}

/// §FS-config-v2.schema.sources.rules: a plain v2 `.md` home is a rule kind.
#[test]
fn v2_rule_home_md_read_as_markdown_loads() {
    let toml = v2("languages = [\"markdown\"]\n", "rules/rules.md");
    assert_rule_kind_loads("v2_rule_home_md", &toml, "rules/rules.md");
}

/// §FS-config-v2.schema.sources.rules: a `document` definition takes the Markdown reader, so a
/// home it reads is a rule kind though `v1_markdown` is no built-in language.
#[test]
fn v2_rule_home_read_by_a_document_definition_loads() {
    let sources = "languages = [\"v1_markdown\"]\n\
                   [schema.sources.extensions]\nmd = \"v1_markdown\"\n\
                   [schema.sources.definitions.v1_markdown]\nclass = \"document\"\n";
    let toml = v2(sources, "rules/rules.md");
    assert_rule_kind_loads("v2_rule_home_document_definition", &toml, "rules/rules.md");
}

/// §FS-config-v2.schema.sources.rules: each entry of `files` is asked, not only the first, so a
/// second entry read as JSON is refused at `form` though the first is Markdown.
#[test]
fn v2_rule_home_second_file_read_as_json_is_refused() {
    let toml = "grund_config_version = 2\nproject_name = \"g179\"\n\
                [schema.sources]\nlanguages = [\"markdown\", \"json\"]\n\
                [schema.kinds.RULE]\nfiles = [\"rules/rules.md\", \"rules/other.json\"]\n\
                form = \"rule\"\n";
    assert_refused_at_form("v2_rule_home_second_file", toml, "rules/other.json");
}

/// §FS-config-v2.schema.sources.rules: a `source` language's entry is judged by its reading,
/// never its name, so a `.rs` home alone loads with `config-rule-file-not-markdown`.
#[test]
fn v2_rule_home_rs_alone_warns() {
    let toml = v2("languages = [\"markdown\", \"rust\"]\n", "rules/rules.rs");
    assert_rule_home_warns("v2_rule_home_rs", &toml, "rules/rules.rs");
}

/// §FS-config-v2.schema.sources.rules: a `.md` home `md = "rust"` reads as source keeps loading:
/// a rule in a block eligible to declare is in force there.
#[test]
fn v2_rule_home_md_read_as_rust_loads() {
    let sources = "languages = [\"markdown\", \"rust\"]\n\
                   [schema.sources.extensions]\nmd = \"rust\"\n";
    let toml = v2(sources, "rules/rules.md");
    assert_rule_kind_loads("v2_rule_home_md_as_rust", &toml, "rules/rules.md");
}

/// §FS-config-v2.schema.sources.rules: a later entry a `source` language reads keeps loading
/// after a `.md` first entry, with `[schema.sources]` written or left out.
#[test]
fn v2_rule_home_second_file_read_as_rust_loads() {
    let row = "[schema.kinds.RULE]\nfiles = [\"rules/rules.md\", \"rules/more.rs\"]\n\
               form = \"rule\"\n";
    let head = "grund_config_version = 2\nproject_name = \"g179\"\n";
    let listed = format!("{head}[schema.sources]\nlanguages = [\"markdown\", \"rust\"]\n{row}");
    assert_rule_kind_loads("v2_rule_home_second_rs", &listed, "rules/more.rs");
    let unwritten = format!("{head}{row}");
    assert_rule_kind_loads("v2_rule_home_second_rs_bare", &unwritten, "rules/more.rs");
}

/// §FS-config-v2.schema.sources.rules, §AR-config.1.6: no entry is judged by the first one's
/// name, so a later `.rs` after a `.txt` first entry `txt = "markdown"` reads loads and warns.
#[test]
fn v2_rule_home_second_file_read_as_rust_after_txt_warns() {
    let toml = "grund_config_version = 2\nproject_name = \"g179\"\n\
                [schema.sources]\nlanguages = [\"markdown\", \"rust\"]\n\
                [schema.sources.extensions]\ntxt = \"markdown\"\n\
                [schema.kinds.RULE]\nfiles = [\"rules/rules.txt\", \"rules/more.rs\"]\n\
                form = \"rule\"\n";
    assert_rule_home_warns("v2_rule_home_txt_then_rs", toml, "rules/more.rs");
}

/// §FS-config-v2.schema.sources.rules, §FS-config-v2.schema.sources.migration: what the real
/// migration writes for a v1 rule kind's `.md` `file` home, `md = "v1_markdown"`, stays one.
#[test]
fn v1_rule_home_carried_through_migration_loads() {
    let home = "rules/rules.md";
    let root = test_root("v1_rule_home_migrating");
    write(&root.join(home), "");
    write(&root.join("docs/a.md"), "");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n[[kinds]]\nkind = \"RULE\"\nfile = \"rules/rules.md\"\n\
         rules = true\n[scan]\ninclude = [\"docs\"]\n[id]\nnamed_sections = true\n",
    );
    let located = Located {
        file: root.join("grund.toml"),
        root: root.clone(),
        config: "grund.toml".into(),
    };
    let Prepared::Translated { report, .. } =
        translate(&located, false, false, &|| true, &|_| Ok(Vec::new()))
            .expect("the v1 file reads")
    else {
        panic!("read as version 2");
    };
    assert_eq!(report.gates, Some(GateResult::Equal), "{report:#?}");
    let candidate = report.candidate.expect("a candidate");
    assert!(
        candidate.contains("md = \"v1_markdown\""),
        "the migration should map `.md` to its definition: {candidate}"
    );
    assert_rule_kind_loads("v1_rule_home_migrated", &candidate, home);
}

/// §FS-config-v2.schema.sources.rules: version 1 keeps the name test — `.md` loads, `.json`
/// and `.txt` are refused.
#[test]
fn v1_rule_home_is_judged_by_its_name() {
    let v1 = |home: &str| {
        format!(
            "grund_config_version = 1\n\n[[kinds]]\nkind = \"RULE\"\nfile = \"{home}\"\nrules = true\n"
        )
    };
    assert_rule_kind_loads("v1_rule_home_md", &v1("rules/rules.md"), "rules/rules.md");
    for home in ["rules/rules.json", "rules/rules.txt"] {
        let err = match load("v1_rule_home_other", &v1(home), home) {
            Ok(_) => panic!("v1 rule home {home} should be refused"),
            Err(err) => format!("{err:#}"),
        };
        assert!(err.contains(REFUSAL), "{home}: unexpected error: {err}");
    }
}
