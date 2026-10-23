//! Numeric identity and authored spelling agree at the editor and host report
//! boundaries (§FS-config.3.2.8, §FS-lsp.1.2.5, §FS-lsp.1.3.1).

#[path = "support/numeric_spelling.rs"]
mod support;

use grund_core::{
    LspSnapshotOpts, RefsOpts, lsp_snapshot, node_embedding_call, node_request, refs,
};
use serde_json::json;
use support::{NOTES, Repo};

fn snapshot(repo: &Repo) -> grund_core::LspSnapshot {
    lsp_snapshot(LspSnapshotOpts {
        path: repo.0.clone(),
        path_provided: true,
        open_documents: Default::default(),
    })
    .unwrap()
}

#[test]
fn editor_keys_join_equivalent_citations_and_preserve_source_ranges() {
    for version in [1, 2] {
        for per_kind in [false, true] {
            let repo = Repo::new(version, per_kind);
            let s = snapshot(&repo);
            let decl = &s.declarations[0];
            assert_eq!(decl.query_id, "ADR-0001");
            assert_eq!(decl.text, "ADR-0001: Decision");
            let usage = s.title_usage(&decl.query_id, &decl.section_separator);
            assert_eq!((usage.sites, usage.files), (3, 1));
            let mut opts = RefsOpts::default();
            opts.path = repo.0.clone();
            opts.path_provided = true;
            opts.id = "ADR-01".into();
            let references = refs(opts).unwrap();
            assert_eq!(references.hits.len(), usage.sites);
            assert_eq!(s.citations.len(), 3);
            for (cite, (written, query, target_line)) in s.citations.iter().zip([
                ("§ADR-1", "ADR-0001", 1),
                ("§ADR-001.terms", "ADR-0001.terms", 3),
                ("§ADR-0001", "ADR-0001", 1),
            ]) {
                assert_eq!(cite.text, written);
                assert_eq!(cite.column, NOTES.find(written).unwrap() + 1);
                assert_eq!(cite.line, 1);
                assert_eq!(cite.query_id, query);
                assert_eq!(cite.declaration_query_id, decl.query_id);
                assert_eq!(
                    cite.target_path,
                    Some(repo.0.join("docs/adr/decision.md").canonicalize().unwrap())
                );
                assert_eq!(cite.target_line, Some(target_line));
            }
            let terms = s
                .sections
                .iter()
                .find(|row| row.query_id == "ADR-0001.terms")
                .unwrap();
            assert_eq!(
                s.title_citations(&terms.query_id, &terms.section_separator)
                    .len(),
                1
            );

            repo.write(
                "docs/notes.md",
                &format!("{NOTES}§ADR-01.terms.detail and §ADR-001.other.terms and §ADR-0007.\n"),
            );
            let s = snapshot(&repo);
            assert_eq!(s.title_citations("ADR-0001.terms", ".").len(), 2);
            assert_eq!(s.title_citations("ADR-0001.other.terms", ".").len(), 1);
            let missing = s.citations.iter().find(|c| c.text == "§ADR-0007").unwrap();
            assert_eq!(missing.query_id, "ADR-0007");
            assert_eq!(missing.declaration_query_id, "ADR-0007");
            assert!(missing.target_path.is_none() && missing.target_line.is_none());
            assert_eq!(s.title_usage("ADR-0001", ".").sites, 5);
        }
    }
}

#[test]
fn node_scan_retains_escaped_source_digits_and_existing_snapshot_shape() {
    for version in [1, 2] {
        for per_kind in [false, true] {
            let repo = Repo::new(version, per_kind);
            repo.write(
                "docs/notes.md",
                "<§>ADR-0001 and <§>ADR-01 and <§>ADR-00001.terms and <§>ADR-0007.\n",
            );
            let output = node_embedding_call(node_request("scan", vec![json!(repo.0)], &repo.0));
            assert!(output["failure"].is_null(), "{output}");
            let s = &output["result"]["snapshot"];
            let escaped = s["escaped_citations"].as_array().unwrap();
            assert_eq!(escaped.len(), 4);
            for (row, (id, section)) in escaped.iter().zip([
                ("ADR-0001", None),
                ("ADR-01", None),
                ("ADR-00001", Some("terms")),
                ("ADR-0007", None),
            ]) {
                assert_eq!(row["id"], id);
                assert_eq!(row["section"].as_str(), section);
                let suffix = section.map_or(String::new(), |s| format!(".{s}"));
                assert_eq!(row["text"], format!("<§>{id}{suffix}"));
                assert_eq!(row["marker"], false);
            }
            assert!(s["citations"].as_array().unwrap().is_empty());
            assert_eq!(s["declarations"][0]["id"], "ADR-0001");
            assert_eq!(s["declarations"][0]["number"], 1);
            assert!(s["declarations"][0]["number"].is_u64());
        }
    }
}

#[test]
fn missing_query_host_records_retain_input_digits_without_shape_changes() {
    for version in [1, 2] {
        for per_kind in [false, true] {
            let repo = Repo::new(version, per_kind);
            for id in ["ADR-0007", "ADR-07"] {
                let args = vec![json!(id), json!({"root":repo.0})];
                let show = node_embedding_call(node_request("show", args.clone(), &repo.0));
                assert!(show["result"].is_null(), "{show}");
                assert_eq!(show["failure"]["code"], "not-found");
                assert!(
                    show["failure"]["message"]
                        .as_str()
                        .unwrap()
                        .contains(&format!("ID not found: {id}"))
                );
                let refs = node_embedding_call(node_request("refs", args, &repo.0));
                assert!(refs["failure"].is_null(), "{refs}");
                assert!(refs["result"]["hits"].as_array().unwrap().is_empty());
                assert!(
                    refs["result"]["note"]
                        .as_str()
                        .unwrap()
                        .starts_with(&format!("{id} is neither declared nor cited"))
                );
                assert_eq!(refs["result"]["totals"], json!({"sites":0,"files":0}));
            }
        }
    }
}
