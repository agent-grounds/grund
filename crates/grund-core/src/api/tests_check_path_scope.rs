//! Test module: the path scope layer — a run given an explicit path reads what
//! its project's ordinary run reads and reports only the path
//! (§FS-check.1.3.6.1, §DF-path-scope-resolves-project-wide). The `--full`
//! scope layer beside it is `tests_check_full_scope.rs`, and the two must not
//! be confused: that one narrows the scan before any rule runs, this one
//! narrows the report after every rule has run (§AR-resolver.3.3).

use std::path::PathBuf;

use crate::testing::{check_run, codes, findings, test_root, write};

const CONFIG: &str = "grund_config_version = 1\n\
    project_name = \"fixture\"\n\n\
    [id]\n\
    format = \"{kind}-{slug}\"\n\
    slug_pattern = \"[a-z][a-z0-9-]*\"\n\n\
    [[kinds]]\n\
    kind = \"FS\"\n\
    folder = \"docs/functional-spec\"\n\
    index = false\n\n\
    [scan]\n\
    include = [\"docs\", \"src\"]\n\
    extensions = [\"md\", \"rs\"]\n";

/// The minimal shape of the defect: `FS-widget` declared under its configured
/// kind home, cited once from `src/lib.rs`, and nothing else wrong. A
/// whole-tree run over this is `success`, which is what makes a finding from a
/// narrower run a disagreement rather than a discovery.
fn widget_repo(name: &str) -> PathBuf {
    let root = test_root(name);
    write(&root.join("grund.toml"), CONFIG);
    write(
        &root.join("docs/functional-spec/FS-widget.md"),
        "# FS-widget: The widget\n\nThe widget does a thing.\n\n## 1. It starts\n\nIt starts when asked.\n",
    );
    write(
        &root.join("src/lib.rs"),
        "//! \u{a7}FS-widget.1 \u{2014} the widget's start path.\npub fn start() {}\n",
    );
    root
}

/// §FS-check.1.3.6.1: a run given an explicit path resolves against the
/// project, so the one citation in the path resolves and the run is silent —
/// the headline of the defect this layer corrects.
#[test]
fn a_path_scoped_run_resolves_a_declaration_outside_the_path() {
    let root = widget_repo("a_path_scoped_run_resolves_a_declaration_outside_the_path");

    assert_eq!(
        findings(&check_run(&root, false)),
        Vec::<String>::new(),
        "the fixture is clean whole-tree, or a narrow finding would prove nothing"
    );
    assert_eq!(
        findings(&check_run(&root.join("src/lib.rs"), false)),
        Vec::<String>::new(),
        "§FS-check.3.1 fires under neither scope: the ID resolves project-wide"
    );
}

/// §FS-check.4.1.4: the declaring side of the same cause. A declaration cited
/// only from outside the path is cited, so the `declared but never cited`
/// warnings of a path-scoped run are exactly the whole-project run's.
#[test]
fn a_citation_outside_the_path_retires_the_unused_declaration_warning() {
    let root = widget_repo("a_citation_outside_the_path_retires_the_unused_declaration_warning");

    assert_eq!(
        findings(&check_run(&root.join("docs"), false)),
        Vec::<String>::new(),
        "§FS-check.4.1.4: a citation counts wherever the run read it"
    );
}

/// §DF-path-scope-resolves-project-wide.2.2: the declaration set is the one
/// `grund check .` reads and not the configured kind homes alone. `AR-widget`
/// lives in a source doc-comment and enrolls through its canonical link in the
/// architecture index, which is the shape a homes-only resolution misses — and
/// the shape this repository's own `AR-checker` is written in.
#[test]
fn a_path_scoped_run_resolves_a_declaration_written_inline_in_source() {
    let root = test_root("a_path_scoped_run_resolves_a_declaration_written_inline_in_source");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\
         project_name = \"fixture\"\n\n\
         [id]\n\
         format = \"{kind}-{slug}\"\n\
         slug_pattern = \"[a-z][a-z0-9-]*\"\n\n\
         [[kinds]]\n\
         kind = \"AR\"\n\
         folder = \"docs/architecture\"\n\n\
         [scan]\n\
         include = [\"docs\", \"src\"]\n\
         extensions = [\"md\", \"rs\"]\n",
    );
    write(
        &root.join("docs/architecture/README.md"),
        "# How: structure\n\n- [\u{a7}AR-widget](../../src/widget.rs): The widget's checker\n",
    );
    write(
        &root.join("src/widget.rs"),
        "//! AR-widget: The widget's checker\n//!\n//! ## 1. Rules\n//!\n//! It checks what the widget did.\npub fn check() {}\n",
    );
    write(
        &root.join("src/lib.rs"),
        "//! \u{a7}AR-widget.1 \u{2014} the widget's rules, applied.\npub fn start() {}\n",
    );

    assert_eq!(
        findings(&check_run(&root, false)),
        Vec::<String>::new(),
        "the fixture is clean whole-tree"
    );
    assert_eq!(
        findings(&check_run(&root.join("src/lib.rs"), false)),
        Vec::<String>::new(),
        "§DF-path-scope-resolves-project-wide.2.2: a declaration may live in a doc-comment, so the resolution scope is every scanned file"
    );
}

/// §FS-check.1.3.6.1: the one direction this layer *adds* a finding. A
/// duplicate declaration whose twin lies outside the path was invisible and is
/// reported, so a path-scoped run that was silent exits 1
/// (§FS-declarations.checks.duplicate).
#[test]
fn a_duplicate_whose_twin_lies_outside_the_path_is_reported() {
    let root = widget_repo("a_duplicate_whose_twin_lies_outside_the_path_is_reported");
    write(
        &root.join("docs/functional-spec/FS-widget2.md"),
        "# FS-widget: The widget\n\nThe twin declaration, in a file the path does not name.\n",
    );

    let whole_tree = findings(&check_run(&root, false));
    let narrow = findings(&check_run(
        &root.join("docs/functional-spec/FS-widget.md"),
        false,
    ));
    assert_eq!(
        narrow, whole_tree,
        "§FS-check.1.3.6.1: the path-scoped run and the whole-tree run stop disagreeing"
    );
    assert_eq!(
        codes(&check_run(
            &root.join("docs/functional-spec/FS-widget.md"),
            false
        )),
        vec!["duplicate".to_string()],
        "the warning that was false is gone and the error that was real is reported"
    );
}

/// §FS-check.1.3.6: the report scope is still exactly the path. Guards against
/// the change rather than pinning it — a finding in a sibling file the widened
/// resolution now reads must be dropped before the report is written, or this
/// layer has traded a false positive for a wider report nobody asked for.
#[test]
fn a_finding_in_a_sibling_file_inside_the_resolution_scope_is_not_reported() {
    let root =
        widget_repo("a_finding_in_a_sibling_file_inside_the_resolution_scope_is_not_reported");
    write(
        &root.join("src/here.rs"),
        "//! \u{a7}FS-missing-alpha \u{2014} declared nowhere.\npub fn here() {}\n",
    );
    write(
        &root.join("src/there.rs"),
        "//! \u{a7}FS-absent-beta \u{2014} declared nowhere.\npub fn there() {}\n",
    );

    assert_eq!(
        findings(&check_run(&root.join("src/here.rs"), false)),
        vec!["src/here.rs:1: unknown reference FS-missing-alpha".to_string()],
        "§FS-check.1.3.6: an explicit path still narrows — the report is the path"
    );
}

/// §FS-check.2.2: the scope cautions are computed against the **report**
/// scope, not the resolution scope. Guards against the change: a path holding
/// no scannable file must still earn its empty-scan caution however much the
/// widened walk read, which a caution computed over the wrong scope would
/// silently delete. (The nothing-recognized caution of §FS-check.4.5 is
/// withheld from a narrowed run either way, by §FS-check.4.5.4.)
#[test]
fn an_empty_path_still_earns_its_caution_however_much_the_walk_read() {
    let root = widget_repo("an_empty_path_still_earns_its_caution_however_much_the_walk_read");
    write(
        &root.join("src/empty/notes.txt"),
        "Not a scanned extension.\n",
    );

    let run = check_run(&root.join("src/empty"), false);
    assert!(
        run.report
            .warnings
            .iter()
            .any(|diagnostic| diagnostic.code == "empty-scan"),
        "§FS-check.2.2: the caution is about what the report scope read, got {:?}",
        codes(&run)
    );
}

/// §FS-check.1.3.6.1: the agent-entrypoint check is a probe over the project
/// root rather than a finding about a scanned file, and it is the one
/// path-anchored diagnostic the report filter exempts. Guards against the
/// change: a blanket filter on the path would drop it, because `AGENTS.md`
/// lies outside every path but the root.
#[test]
fn the_agent_entrypoint_finding_survives_the_report_filter() {
    let root = widget_repo("the_agent_entrypoint_finding_survives_the_report_filter");
    write(
        &root.join("AGENTS.md"),
        "<!-- BEGIN GRUND MANAGED BLOCK -->\n## Grounding with grund (v12)\n\ncurrent managed block\n",
    );

    let run = check_run(&root.join("src/lib.rs"), false);
    assert!(
        run.report
            .errors
            .iter()
            .any(|diagnostic| diagnostic.code == "agents-init"),
        "§FS-check.3.5 is a probe over the project root, not a finding about a scanned file, got {:?}",
        codes(&run)
    );
}

/// §FS-check.1.3.6.1: a run-level finding carries no path, so the report
/// filter never sees one. Guards against the change: a blanket path filter
/// would drop the config findings, the scope cautions and the workspace run
/// warnings along with the diagnostics it was written for.
#[test]
fn a_run_level_finding_survives_the_report_filter() {
    let root = widget_repo("a_run_level_finding_survives_the_report_filter");
    std::fs::remove_file(root.join("grund.toml")).expect("move the config aside");
    write(&root.join(".agents/grund.toml"), CONFIG);

    let run = check_run(&root.join("src/lib.rs"), false);
    assert!(
        run.report
            .warnings
            .iter()
            .any(|diagnostic| diagnostic.code == "deprecated-config-location"
                && diagnostic.path.is_none()),
        "§FS-check.1.3.6.1: a finding that is not about a scanned file is not filtered, got {:?}",
        codes(&run)
    );
}

/// §FS-check.1.3.6.1 and §FS-workspace.5: the resolution scope is the
/// enclosing project's and never a sibling member's. A path inside a member
/// resolves against that member's own `include` and homes, and the boundary
/// stays where §AR-workspace.6 puts it.
#[test]
fn in_a_workspace_a_path_inside_a_member_resolves_against_that_member() {
    let root = test_root("in_a_workspace_a_path_inside_a_member_resolves_against_that_member");
    write(
        &root.join("grund.toml"),
        &format!("{CONFIG}\n[workspace]\nmembers = [\"packages/sub\"]\n"),
    );
    write(
        &root.join("docs/functional-spec/FS-widget.md"),
        "# FS-widget: The widget\n\nThe root project's declaration.\n\n## 1. It starts\n\nIt starts when asked.\n",
    );
    write(
        &root.join("src/lib.rs"),
        "//! \u{a7}FS-widget.1 \u{2014} the widget's start path.\npub fn start() {}\n",
    );
    write(
        &root.join("packages/sub/grund.toml"),
        &CONFIG.replace("project_name = \"fixture\"", "project_name = \"sub\""),
    );
    write(
        &root.join("packages/sub/docs/functional-spec/FS-gadget.md"),
        "# FS-gadget: The gadget\n\nThe member's declaration.\n\n## 1. It runs\n\nIt runs when asked.\n",
    );
    write(
        &root.join("packages/sub/src/lib.rs"),
        "//! \u{a7}FS-gadget.1 \u{2014} the gadget's run path.\npub fn run() {}\n",
    );

    assert_eq!(
        findings(&check_run(&root.join("packages/sub/src/lib.rs"), false)),
        Vec::<String>::new(),
        "§FS-workspace.5: the member's own declaration resolves for a path inside the member"
    );
}
