//! What one answer costs (§FS-lsp.responsiveness.1, §FS-lsp.responsiveness.2).
//!
//! These cases count path resolutions rather than milliseconds. The spec point
//! they pin is a complexity and not a deadline, and a wall-clock assertion would
//! be flaky on a shared runner while saying less: a resolution reads the
//! filesystem once per path component, so the count *is* the syscall cost, and
//! it is the same number on every machine.
//!
//! What is counted is every resolution the server performs while answering —
//! `normalize_path` is the one door (§FS-lsp.responsiveness.2). Discovery
//! resolves a workspace folder and a project root outside that door, once per
//! folder at startup rather than once per answer, and is deliberately not
//! counted here.

use super::*;

/// The fixture shape the report was measured on, shrunk to a unit test: enough
/// declarations that a linear scan over the snapshot is visibly not linear in
/// the findings, and cheap enough to build twice.
const DECLS_PER_SPEC_FILE: usize = 10;
const IDS_PER_SOURCE_FILE: usize = 20;

/// A finding is anchored by looking at a handful of records, not at the
/// snapshot. The budget is per finding and generous: the defect this pins costs
/// two resolutions per snapshot record per finding, which is hundreds.
const RESOLUTIONS_PER_FINDING: usize = 8;

fn config() -> String {
    "grund_config_version = 1\n\
     project_name = \"fixture\"\n\
     [reference]\nmarker = \"\u{a7}\"\nstrict = true\n\
     [id]\nformat = \"{kind}-{slug}\"\nsection_separator = \".\"\n\
     slug_pattern = \"[a-z][a-z0-9-]*\"\n\
     [[kinds]]\nkind = \"FS\"\nfolder = \"docs/functional-spec\"\ntitle = \"What\"\n\
     [scan]\ninclude = [\"docs\", \"src\"]\nextensions = [\"md\", \"rs\"]\n"
        .to_string()
}

/// A workspace with `spec_files × DECLS_PER_SPEC_FILE` declarations, each
/// carrying two numbered sections, and source files citing them. No spec index,
/// so every declaration raises one `missing-index-entry` and every source file
/// one dangling citation: findings *and* snapshot records, which is what the
/// defect multiplies.
fn fixture(root: &Path, spec_files: usize) -> usize {
    fs::write(root.join("grund.toml"), config()).expect("write config");
    let mut ids = Vec::new();
    for file in 0..spec_files {
        let mut text = format!("# Spec file {file}\n\n");
        for offset in 0..DECLS_PER_SPEC_FILE {
            let id = format!("FS-point-p{:04}", file * DECLS_PER_SPEC_FILE + offset);
            text.push_str(&format!(
                "# {id}: a point\n\nThe lead.\n\n## 1. First\n\nBody.\n\n## 2. Second\n\nMore.\n\n"
            ));
            ids.push(id);
        }
        write(
            &root.join(format!("docs/functional-spec/spec-{file:03}.md")),
            &text,
        );
    }
    for (chunk, group) in ids.chunks(IDS_PER_SOURCE_FILE).enumerate() {
        let mut text = String::new();
        for (index, id) in group.iter().enumerate() {
            text.push_str(&format!(
                "/// Realizes \u{a7}{id}.1 and \u{a7}{id}.2.\npub fn f{chunk}_{index}() {{}}\n\n"
            ));
        }
        text.push_str(&format!(
            "/// Dangling \u{a7}FS-absent-p{chunk:04}.1.\npub fn g{chunk}() {{}}\n"
        ));
        write(&root.join(format!("src/mod-{chunk:03}.rs")), &text);
    }
    ids.len()
}

fn server_over(root: &Path) -> (Server, Connection) {
    let (connection, client) = Connection::memory();
    let server = Server::new(
        connection,
        [(Url::from_directory_path(root).unwrap(), root.to_path_buf())]
            .into_iter()
            .collect(),
        true,
    )
    .unwrap();
    (server, client)
}

/// The findings one project reports, and the path resolutions publishing them
/// costs. The snapshot is already built when the count starts, so this is the
/// work done *after* the scan — the part §FS-lsp.responsiveness.1 bounds.
fn publishing_cost(root: &Path) -> (usize, usize) {
    let (mut server, _client) = server_over(root);
    let snapshot = &server.projects.first().expect("one project").snapshot;
    let findings =
        snapshot.report.errors.len() + snapshot.report.warnings.len() + snapshot.run_warnings.len();
    assert!(findings > 0, "the fixture must have findings to publish");
    reset_path_resolutions();
    server.publish_diagnostics().expect("publish diagnostics");
    (findings, path_resolutions())
}

/// §FS-lsp.responsiveness.1: anchoring one finding does an amount of work that
/// does not grow with how many records the snapshot holds.
#[test]
fn publishing_a_diagnostic_set_resolves_a_bounded_number_of_paths_per_finding() {
    let root = test_root("publishing_cost_per_finding");
    let ids = fixture(&root, 6);
    let (findings, resolutions) = publishing_cost(&root);

    assert!(
        resolutions <= findings * RESOLUTIONS_PER_FINDING,
        "publishing {findings} findings over {ids} declarations resolved {resolutions} paths, \
         over the {} this budgets ({RESOLUTIONS_PER_FINDING} per finding): a lookup is \
         resolving a path per snapshot record rather than once per finding \
         (\u{a7}FS-lsp.responsiveness.2)",
        findings * RESOLUTIONS_PER_FINDING
    );
}

/// §FS-lsp.responsiveness.1 as a complexity: double the workspace and the cost
/// of publishing doubles. `GROWTH_CEILING` sits between the linear answer (2x)
/// and the quadratic one (4x), so neither reading can be mistaken for the other.
#[test]
fn publishing_a_diagnostic_set_does_not_grow_with_the_snapshot() {
    const GROWTH_NUMERATOR: usize = 5;
    const GROWTH_DENOMINATOR: usize = 2;

    let small_root = test_root("publishing_cost_small");
    fixture(&small_root, 3);
    let (small_findings, small) = publishing_cost(&small_root);

    let large_root = test_root("publishing_cost_large");
    fixture(&large_root, 6);
    let (large_findings, large) = publishing_cost(&large_root);

    assert!(
        large * GROWTH_DENOMINATOR <= small * GROWTH_NUMERATOR,
        "doubling the workspace took publishing from {small} path resolutions over \
         {small_findings} findings to {large} over {large_findings}: more than the \
         {GROWTH_NUMERATOR}/{GROWTH_DENOMINATOR} a linear cost allows, so the cost grows \
         with findings x snapshot records rather than with findings \
         (\u{a7}FS-lsp.responsiveness.1)"
    );
}

/// §FS-lsp.responsiveness.2 binds every answer, not only a published
/// diagnostic: a request finds its document by one resolution of its own path.
/// The hover here lands on a declaration title, so the lookup has walked past
/// every citation in the snapshot before it answers.
#[test]
fn answering_one_request_resolves_a_bounded_number_of_paths() {
    const PER_REQUEST_BUDGET: usize = 16;

    let root = test_root("request_cost");
    fixture(&root, 6);
    let (mut server, client) = server_over(&root);
    let spec = root.join("docs/functional-spec/spec-005.md");

    reset_path_resolutions();
    server
        .handle_request(Request::new(
            1.into(),
            "textDocument/hover".into(),
            json!({
                "textDocument": {"uri": Url::from_file_path(&spec).unwrap()},
                "position": {"line": 2, "character": 4}
            }),
        ))
        .expect("hover");
    let resolutions = path_resolutions();

    let Message::Response(response) = client.receiver.recv().unwrap() else {
        panic!("response")
    };
    assert!(response.error.is_none(), "{:?}", response.error);
    assert!(
        response.result.as_ref().is_some_and(|body| !body.is_null()),
        "the hover must answer, or the count below is the cost of answering nothing"
    );
    assert!(
        resolutions <= PER_REQUEST_BUDGET,
        "one hover resolved {resolutions} paths, over the {PER_REQUEST_BUDGET} a request is \
         budgeted: the lookup is resolving a path per snapshot record \
         (\u{a7}FS-lsp.responsiveness.2)"
    );
}

/// The claim §FS-lsp.responsiveness.2 rests on, held where it could rot: a
/// snapshot record's path is already resolved, so comparing it against a
/// resolved incoming path by `==` answers exactly what re-resolving both sides
/// answers. This case passes today — it is the guard that lets the cost above
/// be removed without changing a single published range (§FS-lsp.4).
///
/// The overlay is the one input that could have been an exception, because an
/// open document that has never been saved has no file to resolve against.
#[test]
fn a_snapshot_records_path_is_already_resolved() {
    let root = test_root("snapshot_paths_are_resolved");
    fixture(&root, 2);
    // An unmarked heading inside a declaration body keeps its authored span for
    // the diagnostic alone, which is what puts a record in `finding_ranges`
    // (§FS-lsp.1.1.1).
    write(
        &root.join("docs/functional-spec/FS-loose.md"),
        "# FS-loose: a point\n\nLead.\n\n## Not a section\n\nBody.\n",
    );
    // A stub reaching an inline source declaration puts a record in `stubs`.
    write(
        &root.join("src/inline.rs"),
        "/// FS-inline: an inline point\n/// Its body.\npub fn item() {}\n",
    );
    write(
        &root.join("docs/functional-spec/FS-inline.md"),
        "# FS-inline: [source](../../src/inline.rs)\n",
    );

    let (mut server, _client) = server_over(&root);
    // An open document that is not on disk: the overlay case.
    let unsaved = root.join("docs/functional-spec/unsaved.md");
    let unsaved_uri = Url::from_file_path(&unsaved).unwrap();
    server.open_docs.insert(
        unsaved_uri.clone(),
        "# FS-unsaved: a point\n\nLead \u{a7}FS-point-p0000.\n".to_string(),
    );
    server.refresh();

    let snapshot = &server.projects.first().expect("one project").snapshot;
    let collections: [(&str, Vec<&Path>); 5] = [
        (
            "citations",
            snapshot
                .citations
                .iter()
                .map(|c| c.path.as_path())
                .collect(),
        ),
        (
            "finding_ranges",
            snapshot
                .finding_ranges
                .iter()
                .map(|r| r.path.as_path())
                .collect(),
        ),
        (
            "declarations",
            snapshot
                .declarations
                .iter()
                .map(|d| d.path.as_path())
                .collect(),
        ),
        (
            "sections",
            snapshot.sections.iter().map(|s| s.path.as_path()).collect(),
        ),
        (
            "stubs",
            snapshot.stubs.iter().map(|s| s.path.as_path()).collect(),
        ),
    ];
    let probes = [
        unsaved.clone(),
        root.join("docs/functional-spec/spec-000.md"),
        root.join("src/mod-000.rs"),
        root.join("docs/functional-spec/./spec-001.md"),
        root.join("src/../docs/functional-spec/FS-loose.md"),
    ];

    let mut unsaved_seen = false;
    for (name, paths) in &collections {
        assert!(!paths.is_empty(), "{name} must hold a record to prove");
        for path in paths {
            assert_eq!(
                *path,
                normalize_path(path),
                "{name} carries a path that is not already resolved: {path:?}"
            );
            unsaved_seen |= *path == unsaved.as_path();
            for probe in &probes {
                assert_eq!(
                    same_path(path, probe),
                    *path == normalize_path(probe),
                    "{name}: comparing {path:?} against {probe:?} by equality on the resolved \
                     value does not answer what re-resolving both sides answers"
                );
            }
        }
    }
    assert!(
        unsaved_seen,
        "the overlay must reach the snapshot, or its path was never the exception under test"
    );
}
