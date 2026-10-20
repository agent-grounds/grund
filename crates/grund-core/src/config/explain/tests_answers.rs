//! §FS-config-explain.1.1, §FS-config-explain.2, §FS-config-explain.5: what a
//! target names in each version, and which version a file is read as.

use std::path::Path;

use super::*;

fn ask(version: u32, target: &str) -> ExplainAnswer {
    answer(version, &ExplainAsk::Target(target.to_string()))
}

fn key_of(answer: ExplainAnswer) -> (String, String) {
    match answer {
        ExplainAnswer::Key(key) => (key.table, key.key),
        other => panic!("not a key: {other:?}"),
    }
}

fn table_of(answer: ExplainAnswer) -> String {
    match answer {
        ExplainAnswer::Table(table) => table.header,
        other => panic!("not a table: {other:?}"),
    }
}

fn unknown_of(answer: ExplainAnswer) -> ExplainUnknown {
    match answer {
        ExplainAnswer::Unknown(unknown) => unknown,
        other => panic!("known: {other:?}"),
    }
}

/// §FS-config-explain.1.1: a name stands for the placeholder of a table keyed
/// by names, and a placeholder for any placeholder.
#[test]
fn a_named_table_is_answered_as_its_placeholder() {
    for target in [
        "[schema.kinds.FS]",
        "[schema.kinds.<NAME>]",
        "[schema.kinds.<KIND>]",
    ] {
        assert_eq!(
            table_of(ask(2, target)),
            "[schema.kinds.<NAME>]",
            "{target}"
        );
    }
    assert_eq!(
        table_of(ask(2, "[rules.citations.grounding]")),
        "[rules.citations.grounding]"
    );
    assert_eq!(
        table_of(ask(2, "[rules.citations.FS]")),
        "[rules.citations.<KIND>]"
    );
    assert_eq!(table_of(ask(1, "[citations.FS]")), "[citations.<KIND>]");
    assert_eq!(table_of(ask(1, "[[kinds]]")), "[[kinds]]");
}

/// §FS-config-explain.1.1: a key follows its table after a space or a dot; a
/// member follows its inline table's key; any name is a placeholder key.
#[test]
fn keys_members_and_placeholder_keys_resolve() {
    let warn = ("[schema.leads.words]".to_string(), "warn".to_string());
    assert_eq!(key_of(ask(2, "[schema.leads.words] warn")), warn);
    assert_eq!(key_of(ask(2, "[schema.leads.words].warn")), warn);
    assert_eq!(key_of(ask(1, "[[kinds]].fetch")).1, "fetch");
    assert_eq!(
        key_of(ask(1, "[reference] lead_size_warning.max")).1,
        "lead_size_warning.max"
    );
    assert_eq!(key_of(ask(2, "[rules.resolution] TICKET")).1, "<KIND>");
    assert_eq!(
        key_of(ask(2, "[schema.sources.extensions] mdx")).1,
        "<suffix>"
    );
    assert_eq!(
        key_of(ask(1, "grund_config_version")),
        (String::new(), "grund_config_version".into())
    );
    match ask(1, "[reference] lead_size_warning") {
        ExplainAnswer::Key(key) => assert_eq!(
            key.members,
            ["lead_size_warning.max", "lead_size_warning.unit"]
        ),
        other => panic!("not a key: {other:?}"),
    }
}

/// §FS-config-explain.5: each version says why it has no answer.
#[test]
fn an_unknown_target_says_why_in_its_version() {
    match unknown_of(ask(2, "[scan]")) {
        ExplainUnknown::VersionOneTable { table, homes } => {
            assert_eq!(table, "[scan]");
            assert!(homes.contains(&"[schema.sources]".to_string()), "{homes:?}");
        }
        other => panic!("{other:?}"),
    }
    let v2 = ExplainUnknown::VersionTwoTable {
        table: "[schema.sources]".into(),
    };
    assert_eq!(unknown_of(ask(1, "[schema.sources]")), v2);
    match unknown_of(ask(2, "[schema.sources] include")) {
        ExplainUnknown::Key {
            table,
            key,
            admitted,
        } => {
            assert_eq!(
                (table.as_str(), key.as_str()),
                ("[schema.sources]", "include")
            );
            assert!(admitted.contains(&"languages".to_string()), "{admitted:?}");
        }
        other => panic!("{other:?}"),
    }
    for target in ["oversized-leed", "remote-missing", "[scna]", "[kinds]"] {
        let other = ExplainUnknown::Other {
            target: target.into(),
        };
        assert_eq!(unknown_of(ask(1, target)), other, "{target}");
    }
}

/// §FS-config-explain.4: a code a version never turns on is still answered there.
#[test]
fn a_code_is_answered_in_both_versions_and_listed_where_reported() {
    for version in [1, 2] {
        assert!(matches!(
            ask(version, "missing-field"),
            ExplainAnswer::Code(_)
        ));
    }
    let listed = |version| match answer(version, &ExplainAsk::Codes) {
        ExplainAnswer::Codes(codes) => codes.into_iter().map(|code| code.code).collect::<Vec<_>>(),
        other => panic!("{other:?}"),
    };
    assert!(listed(2).contains(&"missing-field".to_string()));
    assert!(!listed(1).contains(&"missing-field".to_string()));
    match ask(2, "oversized-lead") {
        ExplainAnswer::Code(code) => assert_eq!(code.tables, ["[schema.leads.words]"]),
        other => panic!("{other:?}"),
    }
}

/// §FS-config-explain.2: the version is read and nothing else; a version this
/// grund does not read, and text that is not TOML, are the refusal every
/// command gives.
#[test]
fn the_version_is_read_and_nothing_else() {
    let file = Path::new("grund.toml");
    assert_eq!(
        version_of("grund_config_version = 2\n[scan]\nnope = 1\n", file).unwrap(),
        2
    );
    assert_eq!(
        version_of("project_name = \"x\" # one\n\n[scna]\n", file).unwrap(),
        1
    );
    assert_eq!(version_of("grund_config_version = 1\n", file).unwrap(), 1);
    // A value spread over lines is refused at its key, before the first table.
    let spread = "grund_config_version = 1\nkinds = [\n]\n[reference]\nnope = 1\n";
    assert_eq!(version_of(spread, file).unwrap(), 1);
    let refusal = |text| format!("{:#}", version_of(text, file).unwrap_err());
    for (text, refused) in [
        (
            "grund_config_version = 3\n",
            "unsupported config version `3`",
        ),
        (
            "grund_config_version = \"2\"\n",
            "unsupported config version `\"2\"`",
        ),
        (
            "kinds = [\n]\ngrund_config_version = 7\n",
            "grund.toml:1: `kinds` takes only",
        ),
        ("this is not toml\n", "grund.toml:1: expected `key = value`"),
        (
            "[reference]\nthis is not toml\n",
            "grund.toml:2: expected `key = value`",
        ),
    ] {
        assert!(
            refusal(text).contains(refused),
            "{text:?}: {}",
            refusal(text)
        );
    }
}

/// §FS-errors.4: the path a call was given comes back spelled with `/`, so the
/// `Next:` line that repeats it is one string on every OS.
#[test]
fn the_path_asked_about_is_spelled_with_forward_slashes() {
    let asked = Path::new(r"examples\config-explain\repo");
    let explained = explain(asked, &ExplainAsk::Index).unwrap();
    assert_eq!(explained.path, "examples/config-explain/repo");
}
