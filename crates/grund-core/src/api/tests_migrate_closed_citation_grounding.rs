//! §FS-config-migrate.5.2: equal facts compare equal; changed or missing warnings never do.
use super::config_findings::config_diagnostics;
use super::config_migrate::report_gate;
use crate::config::migrate::{Located, Prepared, translate};
use crate::config::{Project, load_config};
use crate::testing::{test_root, write};

const V1: &str = "grund_config_version = 1\n[id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\
    [[kinds]]\nkind = \"FS\"\nfolder = \"specification\"\nindex = false\n\
    [[kinds]]\nkind = \"ce\"\nfolder = \"ce\"\ncitable = false\n\
    require_grounding = true\ngrounding_level = 2\n\
    [[kinds]]\nkind = \"other\"\nfolder = \"other\"\ncitable = false\n\
    [scan]\ninclude = [\"specification\", \"ce\", \"other\"]\n\
    [citations]\ndefault = \"must-not\"\n";

fn candidate(row: &str, rung: &str) -> String {
    format!(
        "grund_config_version = 2\n[schema.kinds.FS]\n\
        folders = [\"specification\"]\nindex = false\n\
        [schema.kinds.ce]\nfolders = [\"ce\"]\ncitable = false\n\
        [schema.kinds.other]\nfolders = [\"other\"]\ncitable = false\n\
        [rules.citations]\ndefault = \"must-not\"\n\
        [rules.citations.{row}.grounding]\n{rung}\n"
    )
}

#[test]
fn closed_citation_grounding_report_gate_compares_facts_and_preserves_differences() {
    let root = test_root("grund-1460-report-facts");
    write(&root.join("grund.toml"), V1);
    write(
        &root.join("specification/FS-login.md"),
        "# FS-login: Login\n\nText.\n",
    );
    for place in ["ce", "other"] {
        std::fs::create_dir_all(root.join(place)).unwrap();
    }
    let config = load_config(&root).unwrap();
    let warnings: Vec<_> = config_diagnostics(&config, config.display())
        .filter(|d| d.code == "closed-citation-grounding")
        .collect();
    let located = Located::new(
        root.to_path_buf(),
        root.join("grund.toml"),
        root.to_path_buf(),
    );
    let Prepared::Translated { v1, line_map, .. } =
        translate(&located, None, false, false, &|| true, &|_| Ok(Vec::new())).unwrap()
    else {
        panic!("v1 tree");
    };
    let v1: Project = *v1;
    let equal = report_gate(
        &located,
        &v1,
        &candidate("ce", "must = \"h2\""),
        &line_map,
        false,
    );
    assert!(
        equal.is_empty(),
        "version-specific wording compares equal: {equal:#?}"
    );
    let mut missing = Vec::new();
    if warnings.len() != 1 {
        missing.push("missing v1 closed-citation-grounding warning".to_string());
    }
    for (name, changed, marks) in [
        ("row", candidate("other", "must = \"h2\""), vec!["- ", "+ "]),
        (
            "strength",
            candidate("ce", "warn = \"h2\""),
            vec!["- ", "+ "],
        ),
        ("unit", candidate("ce", "must = \"h3\""), vec!["- ", "+ "]),
        ("missing", candidate("ce", ""), vec!["- "]),
    ] {
        let lines = report_gate(&located, &v1, &changed, &line_map, false);
        for mark in marks {
            if !lines
                .iter()
                .any(|l| l.starts_with(mark) && l.contains("closed-citation-grounding"))
            {
                missing.push(format!(
                    "{name}: missing {mark}closed-citation-grounding difference: {lines:#?}"
                ));
            }
        }
        if name == "missing" {
            assert!(
                !lines
                    .iter()
                    .any(|l| l.starts_with("+ ") && l.contains("closed-citation-grounding"))
            );
        }
    }
    write(
        &root.join("grund.toml"),
        &V1.replace(
            "require_grounding = true\ngrounding_level = 2\n",
            "require_grounding = false\n",
        ),
    );
    let off = load_config(&root).expect("the grounding-off source loads");
    let added = report_gate(
        &located,
        off.project(),
        &candidate("ce", "must = \"h2\""),
        &line_map,
        false,
    );
    if !added
        .iter()
        .any(|line| line.starts_with("+ ") && line.contains("closed-citation-grounding"))
    {
        missing.push(format!(
            "added: missing + closed-citation-grounding difference: {added:#?}"
        ));
    }
    assert!(missing.is_empty(), "{}", missing.join("\n"));
}

#[test]
fn closed_citation_grounding_report_gate_keeps_config_paths_distinct() {
    let root = test_root("grund-1460-report-paths");
    let text = V1.replace(
        "require_grounding = true\ngrounding_level = 2\n",
        "require_grounding = false\n",
    );
    write(
        &root.join("grund.toml"),
        &format!("{text}\n[workspace]\nmembers = [\"a\"]\n"),
    );
    write(
        &root.join("specification/FS-login.md"),
        "# FS-login: Login\n\nAs \u{a7}FS-login says.\n",
    );
    for place in ["ce", "other"] {
        std::fs::create_dir_all(root.join(place)).unwrap();
    }
    for member in ["a", "b"] {
        write(
            &root.join(member).join("grund.toml"),
            &candidate("ce", "must = \"h2\""),
        );
        write(
            &root.join(member).join("specification/FS-login.md"),
            "# FS-login: Login\n\nAs \u{a7}FS-login says.\n",
        );
        for place in ["ce", "other"] {
            std::fs::create_dir_all(root.join(member).join(place)).unwrap();
        }
    }
    let located = Located::new(
        root.to_path_buf(),
        root.join("grund.toml"),
        root.to_path_buf(),
    );
    let Prepared::Translated {
        v1,
        line_map,
        report,
        ..
    } = translate(&located, None, false, false, &|| true, &|_| Ok(Vec::new())).unwrap()
    else {
        panic!("v1 tree");
    };
    let changed = report
        .candidate
        .unwrap()
        .replace("members = [\"a\"]", "members = [\"b\"]");
    let lines = report_gate(&located, &v1, &changed, &line_map, false);
    for (mark, path) in [("- ", "a/grund.toml"), ("+ ", "b/grund.toml")] {
        assert!(
            lines.iter().any(|line| line.starts_with(mark)
                && line.contains("closed-citation-grounding")
                && line.contains(path)),
            "missing {mark}closed-citation-grounding at {path}: {lines:#?}"
        );
    }
}
