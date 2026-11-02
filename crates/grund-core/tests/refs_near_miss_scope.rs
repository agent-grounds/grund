//! §FS-scan.path.refs: operand resolution uses the selected project's catalog;
//! the Rust API and Python/Node transports retain only the selected sites.

#[path = "support/refs_near_miss_scope.rs"]
mod support;

use grund_core::{
    EmbeddingRequest, RefsOpts, RefsOutcome, embedding_call, node_embedding_call, node_request,
    refs_outcome,
};
use serde_json::{Value, json};
use support::{CONFIG, Repo};

fn query(repo: &Repo, path: Option<&str>, id: &str) -> RefsOutcome {
    let result = refs_outcome(options(repo, path, id)).unwrap();
    assert!(result.output.scan_errors.is_empty(), "{result:?}");
    result
}

fn options(repo: &Repo, path: Option<&str>, id: &str) -> RefsOpts {
    let mut opts = RefsOpts::default();
    opts.path = path.map_or_else(|| repo.0.clone(), |path| repo.0.join(path));
    opts.path_provided = path.is_some();
    opts.id = id.into();
    opts
}

fn expected(path: Option<&str>) -> Vec<(&'static str, Option<&'static str>)> {
    match path {
        None | Some("hits" | "hits/citer.md") => {
            vec![("hits/citer.md", None), ("hits/citer.md", Some("1"))]
        }
        _ => vec![],
    }
}

const PATHS: [Option<&str>; 7] = [
    None,
    Some("src"),
    Some("hits/citer.md"),
    Some("hits"),
    Some("docs/FS-001-alpha.md"),
    Some("docs"),
    Some("empty"),
];

#[test]
fn refs_near_miss_scope_api_accepts_and_filters_sites() {
    let repo = Repo::new();
    let mut failures = Vec::new();
    for path in PATHS {
        let result = query(&repo, path, "FS-BadHaskell");
        if let Some(failure) = result.query_failure {
            failures.push(format!("{path:?}: {failure:?}"));
            continue;
        }
        assert!(result.output.scan_errors.is_empty(), "{result:?}");
        let sites = result
            .output
            .hits
            .iter()
            .map(|hit| (hit.path.as_str(), hit.section.as_deref()))
            .collect::<Vec<_>>();
        assert_eq!(sites, expected(path), "{path:?}");
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn host_call(repo: &Repo, path: Option<&str>, python: bool) -> Value {
    let root = path.map_or_else(|| repo.0.clone(), |path| repo.0.join(path));
    if python {
        embedding_call(EmbeddingRequest {
            operation: "refs".into(),
            root,
            explicit: path.is_some(),
            args: vec![json!("FS-BadHaskell")],
            options: json!({}),
        })
    } else {
        let options = path.map_or_else(|| json!({}), |_| json!({"root": root}));
        node_embedding_call(node_request(
            "refs",
            vec![json!("FS-BadHaskell"), options],
            &repo.0,
        ))
    }
}

fn host_contract(python: bool) {
    let repo = Repo::new();
    let mut failures = Vec::new();
    for path in PATHS {
        let call = host_call(&repo, path, python);
        if !call["failure"].is_null() {
            failures.push(format!("{path:?}: {call}"));
            continue;
        }
        let hits = call["result"]["hits"].as_array().unwrap();
        assert_eq!(call["result"]["scan_errors"], json!([]), "{call}");
        let sites = hits
            .iter()
            .map(|hit| (hit["path"].as_str().unwrap(), hit["section"].as_str()))
            .collect::<Vec<_>>();
        assert_eq!(sites, expected(path), "{path:?}: {call}");
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn refs_near_miss_scope_python_transport() {
    host_contract(true);
}

#[test]
fn refs_near_miss_scope_node_transport() {
    host_contract(false);
}

/// §FS-config.3.2.6: an inline section keeps its interpretation outside the path.
#[test]
fn refs_near_miss_scope_inline_section() {
    let repo = Repo::new();
    let unscoped = query(&repo, None, "FS-BadHaskell.1");
    assert!(unscoped.query_failure.is_none(), "{unscoped:?}");
    assert_eq!(unscoped.output.hits.len(), 1);
    let section = query(&repo, Some("hits"), "FS-BadHaskell.1");
    assert!(section.query_failure.is_none(), "{section:?}");
    assert_eq!(section.output.hits.len(), 1);
    assert_eq!(section.output.hits[0].section.as_deref(), Some("1"));
}

/// §FS-refs.4: duplicate project homes outside the path still refuse as ambiguous.
#[test]
fn refs_near_miss_scope_duplicate_homes() {
    let repo = Repo::new();
    repo.write("docs/duplicate.md", "# FS-BadHaskell: Another home\n");
    let unscoped = refs_outcome(options(&repo, None, "FS-BadHaskell"))
        .expect_err("fixture must have two homes");
    assert!(
        format!("{unscoped:#}").contains("ambiguous"),
        "{unscoped:#}"
    );
    let error = refs_outcome(options(&repo, Some("hits"), "FS-BadHaskell"))
        .expect_err("two project homes must refuse even outside the report path");
    assert!(format!("{error:#}").contains("ambiguous"), "{error:#}");
}

/// These controls pass before the fix: compatibility needs a local declaration,
/// while an ordinary conforming ID can resolve outside the report path.
#[test]
fn refs_near_miss_scope_controls() {
    let repo = Repo::new();
    let control = query(&repo, Some("src"), "FS-001-alpha");
    assert!(control.query_failure.is_none(), "{control:?}");
    assert_eq!(control.output.hits.len(), 1);
    assert_eq!(control.output.hits[0].path, "src/A.hs");
    let missing = query(&repo, Some("hits"), "FS-Undeclared");
    assert_eq!(missing.query_failure.unwrap().kind.code(), "invalid-id");

    // Another loaded project contains the compatible token; the selected root
    // does not. Its catalog alone must never authorize the unqualified operand.
    repo.write("src/A.hs", "-- No declaration.\n");
    repo.write("other/grund.toml", CONFIG);
    repo.write("other/src/A.hs", "-- | FS-BadHaskell: Other project\n");
    repo.write(
        "grund.toml",
        &format!("{CONFIG}\n[workspace]\nmembers = [\"other\"]\ninclude_root = true\n"),
    );
    let isolated = query(&repo, None, "FS-BadHaskell");
    assert!(
        isolated.output.workspace,
        "fixture must load the other project"
    );
    assert_eq!(isolated.query_failure.unwrap().kind.code(), "invalid-id");
    let qualified = query(&repo, None, "other/FS-BadHaskell");
    assert!(qualified.query_failure.is_none(), "{qualified:?}");
}
