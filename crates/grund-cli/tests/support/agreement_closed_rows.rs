//! Each closed row is an expectation of agreement (§REQ-one-answer.proof.evidence.closed):
//! the comparator is handed the two answers the row once ruled, under the very
//! condition that ruled them, and must report a divergence rather than receipt
//! a known difference. R06a to R06d hold no exception left to drop: their trees
//! in `agreement_buffers` must agree (§REQ-one-answer.text.parity).

use super::agreement_evidence::{Call, capture};
use super::agreement_tree::{CONFIG_SLUG, Tree};
use grund_core::LspSnapshotOpts;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// One kind, `FS`, anywhere under `docs/`, and only `docs/` scanned.
const DOCS: &str = "[id]\nformat = \"{kind}-{slug}\"\n[[kinds]]\nkind = \"FS\"\n\
    folder = \"docs\"\nindex = false\n[scan]\ninclude = [\"docs\"]\n";

/// Whether a known-difference receipt holds `row` as an exception.
fn held(calls: &[Call], row: &str) -> bool {
    calls
        .iter()
        .any(|call| call.surface == "known-difference" && call.result["row"] == row)
}

/// A stub of `FS-alpha` at `docs/home.md` whose target, `src/<target>`, is
/// outside the include (§REQ-one-answer.stubs.outside-include), and the real
/// editor snapshot of the tree. Since 0.19.0 the snapshot lists no such stub:
/// its target stands in for it (§FS-declarations.stubs.pairing.4). Placement,
/// U03a and U03c ruled on the snapshot the editor gave before, which listed the
/// stub beside an unscanned target, so that record is put back; the target
/// staying unscanned is asserted on the real snapshot, not assumed.
fn outside_target(name: &str, target: &str, body: &str) -> (Tree, Value) {
    let home = format!("# FS-alpha: [{target}](../src/{target})\n");
    let at = format!("src/{target}");
    let tree = Tree::new(
        name,
        &[("grund.toml", DOCS), ("docs/home.md", &home), (&at, body)],
    );
    let loaded = grund_core::lsp_snapshot_with_completion(LspSnapshotOpts {
        path: tree.0.clone(),
        path_provided: true,
        ..Default::default()
    })
    .expect("editor snapshot");
    assert!(
        !loaded
            .metadata
            .snapshot
            .scanned_files
            .contains(&tree.0.join(&at)),
        "the stub's target must lie outside the scan"
    );
    let mut facts = super::agreement_snapshot::facts(&loaded);
    facts["snapshot"]["stubs"] = json!([{"project":null,"path":tree.0.join("docs/home.md"),
        "display_path":"docs/home.md","line":1,"column":3,"text":home.trim_end(),
        "query_id":"FS-alpha","section_separator":".","target_path":tree.0.join(&at),
        "target_line":1}]);
    (tree, facts)
}

fn listed(target: &str) -> Value {
    json!({"result":{"entries":[{"id":"FS-alpha","project":null,"path":"docs/home.md",
        "line":1,"stub":true,"defines":format!("src/{target}"),"duplicate":false}]}})
}

#[test]
fn placement_a_declaration_listed_at_its_stub_is_a_divergence() {
    let (tree, loaded) = outside_target(
        "closed-placement",
        "target.rs",
        "/// FS-alpha: Alpha\n///\n/// Target body.\npub fn alpha() {}\n",
    );
    let context = json!({"saved":tree.0,"base":tree.0,"reportBase":tree.0});
    let mut answers = BTreeMap::new();
    answers.insert("list", listed("target.rs"));
    answers.insert(
        "list-size",
        json!({"result":{"entries":[{"id":"FS-alpha","project":null,
        "path":"src/target.rs","line":1,"stub":false,"duplicate":false}]}}),
    );
    let mut errors = Vec::new();
    let calls = capture(|| {
        super::agreement_facts::declarations(&context, &answers, Some(&loaded), &mut errors)
    });
    assert!(
        !held(&calls, "Placement"),
        "Placement closed in 0.19.0 (\u{a7}DF-stubs-by-home-and-at-the-target) yet is held as an exception"
    );
    assert!(!errors.is_empty(), "list and list --size differ unreported");
}

/// `show FS-alpha` at the outside target, as every surface answers it since
/// §DF-stubs-by-home-and-at-the-target, with the section `1` it declares.
fn shown() -> Value {
    json!({"result":{"json":json!({"path":"src/target.md","section":null,
        "sections":[{"path":"1","title":"One","depth":2}]}).to_string()}})
}

fn batch(sections: &[&str]) -> Value {
    let record = |section: Option<&str>| {
        json!({"query":{"id":"FS-alpha","section":section},
            "result":{"json":json!({"path":"src/target.md"}).to_string()}})
    };
    let mut records = vec![record(None)];
    records.extend(sections.iter().map(|section| record(Some(section))));
    json!({"result":{"records":records}})
}

const SECTIONED: &str = "# FS-alpha: Alpha\n\n## 1. One\n\nBody.\n";

#[test]
fn u03a_a_batch_omitting_an_outside_targets_section_is_a_divergence() {
    let (tree, loaded) = outside_target("closed-u03a", "target.md", SECTIONED);
    let context = json!({"saved":tree.0,"base":tree.0,"reportBase":tree.0});
    let mut errors = Vec::new();
    let calls = capture(|| {
        super::agreement_facts::sections(
            &context,
            "FS-alpha",
            &shown(),
            &batch(&[]),
            &listed("target.md"),
            Some(&loaded),
            &mut errors,
        )
    });
    assert!(
        !held(&calls, "U03a"),
        "U03a closed in 0.19.0 yet is held as an exception"
    );
    assert!(
        errors.iter().any(|error| error.contains("FS-alpha.1")),
        "the batch omits FS-alpha.1 unreported: {errors:?}"
    );
}

#[test]
fn u03c_an_editor_omitting_an_outside_targets_section_is_a_divergence() {
    let (tree, mut loaded) = outside_target("closed-u03c", "target.md", SECTIONED);
    // The editor once left the target's sections out; hand the comparator that answer.
    loaded["snapshot"]["sections"] = json!([]);
    let context = json!({"saved":tree.0,"base":tree.0,"reportBase":tree.0});
    let mut errors = Vec::new();
    let calls = capture(|| {
        super::agreement_facts::sections(
            &context,
            "FS-alpha",
            &shown(),
            &batch(&["1"]),
            &listed("target.md"),
            Some(&loaded),
            &mut errors,
        )
    });
    assert!(
        !held(&calls, "U03c"),
        "U03c closed in 0.19.0 yet is held as an exception"
    );
    assert!(
        errors.iter().any(|error| error.contains("FS-alpha.1")),
        "the editor omits FS-alpha.1 unreported: {errors:?}"
    );
}

/// R01's condition: a path argument below the declaration's home, the command
/// line refusing, and the editor answering from outside the selection.
#[test]
fn r01_an_editor_answer_a_narrowed_show_refuses_is_a_divergence() {
    let tree = Tree::new(
        "closed-r01",
        &[
            ("grund.toml", DOCS),
            ("docs/a/use.md", "See \u{a7}FS-alpha.\n"),
            ("docs/b/fs.md", "# FS-alpha: Alpha\n\nBody.\n"),
        ],
    );
    let scope = json!({"projectRoot":tree.0,"reportBase":tree.0});
    let context = json!({"base":tree.0,"saved":tree.0,"reportBase":tree.0,
        "path":tree.0.join("docs/a"),"cwd":tree.0,"editorPath":tree.0,"explicit":true,
        "variant":"path-argument","overlays":{},"scopes":{"editor":scope,"cli":scope}});
    let mut worker = super::agreement_editor_worker::Worker::new();
    let mut loading = Vec::new();
    super::agreement_editor::snapshot(&mut worker, &context, &mut loading)
        .expect("the editor loads the tree");
    let mut errors = Vec::new();
    let calls = capture(|| {
        super::agreement_editor::show(&mut worker, &context, "FS-alpha", &[], true, &mut errors)
    });
    worker.release();
    assert!(
        !held(&calls, "R01"),
        "R01 closed in 0.19.0 (\u{a7}DF-path-narrows-the-report-in-0-19) yet is held as an exception"
    );
    assert!(
        errors.iter().any(|error| error.contains("FS-alpha")),
        "the editor answers what the command line refuses, unreported: {errors:?}"
    );
}

/// R02's condition: a narrowed workspace run whose `check` refuses an alias
/// citation as an unknown project while the editor resolves it.
#[test]
fn r02_an_alias_the_editor_resolves_and_check_refuses_is_a_divergence() {
    let tree = Tree::new("closed-r02", &[("grund.toml", CONFIG_SLUG)]);
    let context = json!({"base":tree.0,"saved":tree.0,"reportBase":tree.0,
        "path":tree.0.join("docs"),"variant":"narrowed",
        "scopes":{"editor":{"reportBase":tree.0}}});
    let loaded = json!({"snapshot":{"workspace":true,"run_warnings":[],
        "report":{"errors":[],"warnings":[]},
        "citations":[{"path":tree.0.join("docs/use.md"),"line":1,
            "target_path":tree.0.join("member/docs/fs.md"),"query_id":"member/FS-alpha"}]}});
    let host = json!({"failure":null,"run_cautions":[],"result":{"selected_report":{
        "errors":[{"severity":"error","code":"unknown-project","path":"docs/use.md","line":1,
            "message":"unknown project alias `member` in member/FS-alpha"}],"warnings":[]}}});
    let mut errors = Vec::new();
    let calls = capture(|| super::agreement_editor::check(&context, &loaded, &host, &mut errors));
    assert!(
        !held(&calls, "R02"),
        "R02 closed in 0.19.0 (\u{a7}DF-path-narrows-the-report-in-0-19) yet is held as an exception"
    );
    assert!(
        !errors.is_empty(),
        "check refuses what the editor resolves, unreported"
    );
}
