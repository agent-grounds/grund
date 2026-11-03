//! Test module: whether a rule kind's `file` home is Markdown is the reader's
//! question (§FS-config-v2.schema.sources.rules) — its name in version 1, the language
//! `[schema.sources]` maps it to in version 2 — refused at `form` with the
//! message of §FS-config.3.4.12 when the answer is no.

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
#[ignore = "red until #1149 asks [schema.sources] whether a v2 rule home is Markdown; implement removes this"]
fn v2_rule_home_md_read_as_json_is_refused() {
    let toml = v2(MD_AS_JSON, "rules/rules.md");
    assert_refused_at_form("v2_rule_home_md_as_json", &toml, "rules/rules.md");
}

/// §FS-config-v2.schema.sources.rules: a `.md` home no admitted language reads is refused.
#[test]
#[ignore = "red until #1149 asks [schema.sources] whether a v2 rule home is Markdown; implement removes this"]
fn v2_rule_home_md_read_by_no_language_is_refused() {
    let toml = v2("languages = [\"json\"]\n", "rules/rules.md");
    assert_refused_at_form("v2_rule_home_md_unread", &toml, "rules/rules.md");
}

/// §FS-config-v2.schema.sources.rules: the converse — a `.txt` home `txt = "markdown"` reads as
/// Markdown is a rule kind.
#[test]
#[ignore = "red until #1149 asks [schema.sources] whether a v2 rule home is Markdown; implement removes this"]
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
