//! §FS-refs.2.1.1 at the library boundary, from issue #936: the engine decides
//! the `list --unused` note once, as the `note` of the `refs` result, so the
//! Rust API, the Python call and the Node call carry one value, in the
//! declaration's catalog spelling (§FS-distribution.3.2.2.2).

#[path = "support/numeric_spelling.rs"]
mod support;

use grund_core::{
    EmbeddingRequest, RefsOpts, embedding_call, node_embedding_call, node_request, refs,
};
use serde_json::{Value, json};
use support::Repo;

const NOTE: &str =
    "ADR-0003 is declared but nothing cites it; `grund list --unused` lists every such declaration";

/// The numeric fixture of #819 under each version and format, with
/// `ADR-0003` declared and cited nowhere beside the cited `ADR-0001`.
fn repos() -> impl Iterator<Item = Repo> {
    [1, 2].into_iter().flat_map(|version| {
        [false, true].into_iter().map(move |per_kind| {
            let repo = Repo::new(version, per_kind);
            repo.write("docs/adr/ADR-0003.md", "# ADR-0003: Unused\n\nBody.\n");
            repo
        })
    })
}

fn note(repo: &Repo, path_provided: bool, id: &str, section: Option<&str>) -> Option<String> {
    let output = refs(RefsOpts {
        path: repo.0.clone(),
        path_provided,
        id: id.into(),
        section: section.map(str::to_owned),
        descendants: false,
    })
    .unwrap();
    assert!(output.hits.is_empty() || id.starts_with("ADR-1"), "{id}");
    output.note
}

/// The `refs` result of the Python and the Node call, with no root or with
/// the root given, which is the bindings' explicit path.
fn host_results(repo: &Repo, id: &str, root: bool) -> [Value; 2] {
    let options = if root {
        json!({"root": repo.0})
    } else {
        json!({})
    };
    let python = embedding_call(EmbeddingRequest {
        operation: "refs".into(),
        root: repo.0.clone(),
        explicit: root,
        args: vec![json!(id)],
        options: json!({}),
    });
    let node = node_embedding_call(node_request("refs", vec![json!(id), options], &repo.0));
    [python, node].map(|call| {
        assert!(call["failure"].is_null(), "{call}");
        call["result"].clone()
    })
}

/// §FS-refs.2.1.1: a no-path whole-ID query of the uncited declaration
/// carries the note, in catalog spelling whatever digits the operand used.
#[test]
#[ignore = "red until #936 returns the list --unused note from refs; implement removes this"]
fn api_note_names_the_catalog_spelling() {
    for repo in repos() {
        for id in ["ADR-3", "ADR-03", "ADR-0003"] {
            assert_eq!(note(&repo, false, id, None).as_deref(), Some(NOTE), "{id}");
        }
    }
}

/// §FS-refs.2.1.1: an explicit path, a section and a cited ID each withhold
/// the note, and the result is what it was before the point.
#[test]
fn api_note_withheld() {
    for repo in repos() {
        assert_eq!(note(&repo, true, "ADR-3", None), None);
        assert_eq!(note(&repo, false, "ADR-3", Some("terms")), None);
        assert_eq!(note(&repo, false, "ADR-1", None), None);
        for result in host_results(&repo, "ADR-3", true) {
            assert!(result["note"].is_null(), "{result}");
            assert!(result["hits"].as_array().unwrap().is_empty(), "{result}");
        }
    }
}

/// §FS-refs.2.1.1, §FS-distribution.3.2.2.2: both host transports carry the
/// engine's one note unchanged, with no new field.
#[test]
#[ignore = "red until #936 returns the list --unused note from refs; implement removes this"]
fn host_note_matches_the_api() {
    for repo in repos() {
        for result in host_results(&repo, "ADR-03", false) {
            assert_eq!(result["note"], json!(NOTE), "{result}");
            assert!(result["hits"].as_array().unwrap().is_empty(), "{result}");
        }
    }
}
