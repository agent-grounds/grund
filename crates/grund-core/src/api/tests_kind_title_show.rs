//! Target-kind metadata on every successful show form (§FS-config.3.4.3,
//! §FS-output-shapes.4, §FS-output-shapes.4.1, §FS-show.2.4).

use super::*;
use crate::model::json_escape;
use crate::queries::{BatchShowQuery, ShowFormat, ShowMode, ShowOpts, show_batch_with_scope};
use crate::testing::{test_root, write};
use std::path::Path;

fn config(title: Option<&str>) -> String {
    format!(
        "grund_config_version = 1\n[id]\nformat = \"{{kind}}-{{slug}}\"\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n{}",
        title
            .map(|title| format!("title = \"{}\"\n", json_escape(title)))
            .unwrap_or_default()
    )
}

fn read(root: &Path, id: &str, mode: ShowMode, format: ShowFormat) -> crate::model::ShowOutput {
    show(
        id,
        ShowOpts {
            path: root.to_path_buf(),
            section: None,
            mode,
            format,
        },
    )
    .unwrap()
}

#[test]
fn kind_title_show_modes_preserve_bodies_and_append_metadata_last() {
    let root = test_root("kind_title_show_modes");
    let source = "# FS-authored: Authored title\n\nLead.\n\n## 1. Detail\n\nDetail body.\n";
    write(&root.join("docs/FS-authored.md"), source);
    for title in [None, Some("Product \"contracts\""), Some("")] {
        write(&root.join("grund.toml"), &config(title));
        let metadata = title
            .map(|title| format!(",\"kind_title\":\"{}\"", json_escape(title)))
            .unwrap_or_default();
        for (mode, body, sections) in [
            (ShowMode::Lead, "Lead.\n", ""),
            (
                ShowMode::Brief,
                "# FS-authored: Authored title\n\nLead.\n",
                "",
            ),
            (
                ShowMode::Full,
                "Lead.\n\n## 1. Detail\n\nDetail body.\n",
                "",
            ),
            (
                ShowMode::Toc,
                "Lead.\n\n## 1. Detail\n",
                ",\"sections\":[{\"path\":\"1\",\"title\":\"Detail\",\"depth\":1}]",
            ),
        ] {
            let shown = read(&root, "FS-authored", mode, ShowFormat::Json);
            assert_eq!(shown.body, body);
            assert_eq!(
                shown.json.unwrap(),
                format!(
                    "{{\"id\":\"FS-authored\",\"section\":null,\"body\":\"{}\",\"path\":\"docs/FS-authored.md\",\"line\":1{sections}{metadata}}}",
                    json_escape(body)
                )
            );
            let section = read(&root, "FS-authored.1", mode, ShowFormat::Json);
            let section_body = "## 1. Detail\n\nDetail body.\n";
            let section_map = if mode == ShowMode::Toc {
                ",\"sections\":[]"
            } else {
                ""
            };
            assert_eq!(
                section.json.unwrap(),
                format!(
                    "{{\"id\":\"FS-authored\",\"section\":\"1\",\"body\":\"{}\",\"path\":\"docs/FS-authored.md\",\"line\":5{section_map}{metadata}}}",
                    json_escape(section_body)
                )
            );
        }
        assert_eq!(
            read(&root, "FS-authored", ShowMode::Full, ShowFormat::Markdown).body,
            source
        );
    }
}

#[test]
fn kind_title_show_uses_effective_defaults() {
    let root = test_root("kind_title_show_defaults");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n[id]\nformat = \"{kind}-{slug}\"\n",
    );
    write(
        &root.join("requirements.md"),
        "# FS-authored: Authored\n\nLead.\n",
    );
    assert_eq!(
        read(&root, "FS-authored", ShowMode::Lead, ShowFormat::Json)
            .json
            .unwrap(),
        "{\"id\":\"FS-authored\",\"section\":null,\"body\":\"Lead.\\n\",\"path\":\"requirements.md\",\"line\":1,\"kind_title\":\"What: behavior, requirements, and constraints\"}"
    );
}

#[test]
fn kind_title_show_e2e_and_json_values_keep_their_alternate_shapes() {
    let root = test_root("kind_title_show_alternate_shapes");
    for title in [None, Some("Target metadata"), Some("")] {
        let title_config = title
            .map(|title| format!("title = \"{title}\"\n"))
            .unwrap_or_default();
        write(
            &root.join("grund.toml"),
            &format!(
                "grund_config_version = 1\n[id]\nformat = \"{{kind}}-{{slug}}\"\n\
             [[kinds]]\nkind = \"CONST\"\nfolder = \"values\"\nindex = false\nvalues = true\n{title_config}\
             [[kinds]]\nkind = \"E2E\"\nfolder = \"cases\"\nindex = false\n{title_config}"
            ),
        );
        write(
            &root.join("values/data.json"),
            "{\n  \"CONST-price\": [12, \"USD\"]\n}\n",
        );
        write(
            &root.join("cases/login/command.args"),
            "check --format json repo\n",
        );
        write(&root.join("cases/login/expected.exit"), "0\n");
        let metadata = title
            .map(|title| format!(",\"kind_title\":\"{title}\""))
            .unwrap_or_default();
        for mode in [
            ShowMode::Lead,
            ShowMode::Brief,
            ShowMode::Full,
            ShowMode::Toc,
        ] {
            let e2e = read(&root, "E2E-login", mode, ShowFormat::Json);
            let records = show_batch_with_scope(
                Some(
                    ["E2E-login", "CONST-price", "CONST-price.2"]
                        .into_iter()
                        .map(|id| BatchShowQuery {
                            id: id.into(),
                            section: None,
                        })
                        .collect(),
                ),
                ShowOpts {
                    path: root.clone(),
                    mode,
                    ..ShowOpts::default()
                },
                true,
            )
            .unwrap();
            for (record, id) in records
                .iter()
                .zip(["E2E-login", "CONST-price", "CONST-price.2"])
            {
                assert_eq!(record.query.id, id);
                assert_eq!(
                    record.result.as_ref().unwrap(),
                    &read(&root, id, mode, ShowFormat::Json).json.unwrap()
                );
            }
            assert_eq!(
                e2e.json.unwrap(),
                format!(
                    "{{\"id\":\"E2E-login\",\"kind\":\"E2E\",\"path\":\"cases/login\",\"args\":[\"check\",\"--format\",\"json\",\"repo\"],\"expected_exit\":0,\"fixtures\":[\"command.args\",\"expected.exit\"]{metadata}}}"
                )
            );
            for (id, section, body) in [
                ("CONST-price", "null", "\"CONST-price\": [12, \"USD\"]"),
                ("CONST-price.2", "\"2\"", "\"USD\""),
            ] {
                let shown = read(&root, id, mode, ShowFormat::Json);
                let map = if mode == ShowMode::Toc {
                    ",\"sections\":[]"
                } else {
                    ""
                };
                assert_eq!(shown.body, body);
                assert_eq!(
                    shown.json.unwrap(),
                    format!(
                        "{{\"id\":\"CONST-price\",\"section\":{section},\"body\":\"{}\",\"path\":\"values/data.json\",\"line\":2{map}{metadata}}}",
                        json_escape(body)
                    )
                );
            }
        }
    }
}

#[test]
fn kind_title_show_workspace_and_batch_select_each_targets_title() {
    let root = test_root("kind_title_show_workspace_batch");
    write(
        &root.join("grund.toml"),
        &(config(Some("Caller title")) + "\n[workspace]\nmembers = [\"target\", \"untitled\"]\n"),
    );
    write(
        &root.join("target/grund.toml"),
        &config(Some("Target title")),
    );
    write(&root.join("untitled/grund.toml"), &config(None));
    for member in ["", "target", "untitled"] {
        write(
            &root.join(member).join("docs/FS-authored.md"),
            "# FS-authored: Authored\n\nLead.\n",
        );
    }
    let queries = [
        "FS-authored",
        "target/FS-authored",
        "untitled/FS-authored",
        "target/FS-missing",
    ];
    let records = show_batch_with_scope(
        Some(
            queries
                .iter()
                .map(|id| BatchShowQuery {
                    id: id.to_string(),
                    section: None,
                })
                .collect(),
        ),
        ShowOpts {
            path: root.clone(),
            ..ShowOpts::default()
        },
        true,
    )
    .unwrap();
    assert_eq!(records.len(), 4);
    for (index, (prefix, title)) in [
        ("", Some("Caller title")),
        ("target/", Some("Target title")),
        ("untitled/", None),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(records[index].query.id, queries[index]);
        assert_eq!(records[index].query.section, None);
        let metadata = title
            .map(|title| format!(",\"kind_title\":\"{title}\""))
            .unwrap_or_default();
        let expected = format!(
            "{{\"id\":\"FS-authored\",\"section\":null,\"body\":\"Lead.\\n\",\"path\":\"{prefix}docs/FS-authored.md\",\"line\":1{metadata}}}"
        );
        assert_eq!(records[index].result.as_ref().unwrap(), &expected);
        assert_eq!(
            read(&root, queries[index], ShowMode::Lead, ShowFormat::Json)
                .json
                .unwrap(),
            expected
        );
    }
    let failure = records[3].result.as_ref().unwrap_err();
    assert_eq!(failure.code, "not-found");
    assert_eq!(failure.message, "ID not found: FS-missing");
    assert!(failure.sites.is_empty());
}
