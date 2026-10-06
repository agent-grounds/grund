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

/// §FS-check.2.2.3.1: the unread-source caution of §FS-check.1.3.6.3 is about a
/// file the *resolution* scope could not read, so it is not a finding about the
/// report scope and must not suppress that scope's own caution. Guards against
/// the change: a caution appended before the emptiness question is asked makes
/// one unreadable file anywhere delete the empty-scan line, which is the
/// mistyped-path case — `grund check src/typo` would stop saying nothing
/// matched and talk about a file elsewhere instead.
#[test]
fn an_unread_file_outside_the_path_does_not_suppress_the_empty_scan_caution() {
    let root =
        widget_repo("an_unread_file_outside_the_path_does_not_suppress_the_empty_scan_caution");
    write(
        &root.join("src/empty/notes.txt"),
        "Not a scanned extension.\n",
    );
    std::fs::write(
        root.join("docs/functional-spec/undecodable.md"),
        [0xff, 0xfe],
    )
    .expect("write an undecodable declaration source");

    let run = check_run(&root.join("src/empty"), false);
    assert!(
        run.report
            .warnings
            .iter()
            .any(|diagnostic| diagnostic.code == "empty-scan"),
        "§FS-check.2.2.3.1: the unread file outside the path does not suppress \
         the empty-scan caution, got {:?}",
        codes(&run)
    );
    assert!(
        run.report
            .warnings
            .iter()
            .any(|diagnostic| diagnostic.code == "io"
                && diagnostic.message.contains("outside the report scope")),
        "§FS-check.1.3.6.3: and the unread file still earns its own caution, got {:?}",
        codes(&run)
    );
}

/// §FS-check.1.3.6.1: the agent-entrypoint check is a probe over the project
/// root rather than a finding about a scanned file, and its finding is exempted
/// by its code rather than by its path. Guards against the
/// change: a blanket filter on the path would drop it, because `AGENTS.md`
/// lies outside every path but the root.
#[test]
fn the_agent_entrypoint_finding_survives_the_report_filter() {
    let root = widget_repo("the_agent_entrypoint_finding_survives_the_report_filter");
    write(
        &root.join("AGENTS.md"),
        "<!-- BEGIN GRUND MANAGED BLOCK -->\n## Grounding with grund (v14)\n\ncurrent managed block\n",
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

/// §FS-check.1.3.6.1: the entrypoint exemption is by finding and not by file.
/// An ordinary dangling citation in `AGENTS.md` is a finding about a scanned
/// file like any other, so a run about `src/lib.rs` must not report it — while
/// the §FS-check.3.5 probe's own finding still survives. An entrypoint inside
/// `[scan] include` is the ordinary configuration, which this repository's own
/// `grund.toml` writes.
#[test]
fn an_ordinary_finding_about_the_agent_entrypoint_is_dropped_by_the_report_filter() {
    let root = test_root("an_ordinary_finding_about_the_agent_entrypoint_is_dropped");
    write(
        &root.join("grund.toml"),
        &CONFIG.replace(
            "include = [\"docs\", \"src\"]",
            "include = [\"docs\", \"src\", \"AGENTS.md\"]",
        ),
    );
    write(
        &root.join("docs/functional-spec/FS-widget.md"),
        "# FS-widget: The widget\n\nThe widget does a thing.\n\n## 1. It starts\n\nIt starts when asked.\n",
    );
    write(
        &root.join("src/lib.rs"),
        "//! \u{a7}FS-widget.1 \u{2014} the widget's start path.\npub fn start() {}\n",
    );
    write(
        &root.join("AGENTS.md"),
        "# Agents\n\nEvery change follows \u{a7}FS-nowhere.\n",
    );

    let whole_tree = check_run(&root, false);
    assert!(
        whole_tree
            .report
            .errors
            .iter()
            .any(|diagnostic| diagnostic.code == "dangling"),
        "the whole-tree run reports the entrypoint's dangling citation, got {:?}",
        codes(&whole_tree)
    );

    let narrow = check_run(&root.join("src/lib.rs"), false);
    assert!(
        !narrow
            .report
            .errors
            .iter()
            .any(|diagnostic| diagnostic.code == "dangling"),
        "§FS-check.1.3.6.1: the report is still exactly the path, got {:?}",
        codes(&narrow)
    );
    assert!(
        narrow
            .report
            .errors
            .iter()
            .any(|diagnostic| diagnostic.code == "agents-init"),
        "§FS-check.3.5's own finding is the one the filter exempts, got {:?}",
        codes(&narrow)
    );
}

/// §FS-check.1.3.6.1 and §FS-config.3.5.8: the resolution scope is `[scan]
/// include` **plus every walked kind home**, so a citable `E2E` kind whose home
/// `include` does not name is read under a path scope exactly as the ordinary
/// walk reads it. Asking `include` alone made a `§E2E-…` citation dangle under
/// the narrow run that `grund check .` resolves — this ticket's own defect.
#[test]
fn a_path_scoped_run_resolves_an_e2e_case_whose_home_lies_outside_include() {
    let root = test_root("a_path_scoped_run_resolves_an_e2e_case_outside_include");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\
         project_name = \"fixture\"\n\n\
         [id]\n\
         format = \"{kind}-{slug}\"\n\
         slug_pattern = \"[a-z][a-z0-9-]*\"\n\n\
         [[kinds]]\n\
         kind = \"FS\"\n\
         folder = \"docs/functional-spec\"\n\
         index = false\n\n\
         [[kinds]]\n\
         kind = \"E2E\"\n\
         folder = \"e2e/cases\"\n\
         index = false\n\n\
         [scan]\n\
         include = [\"docs\", \"src\"]\n\
         extensions = [\"md\", \"rs\"]\n",
    );
    write(
        &root.join("docs/functional-spec/FS-widget.md"),
        "# FS-widget: The widget\n\nThe widget does a thing.\n\n## 1. It starts\n\nIt starts when asked.\n",
    );
    write(&root.join("e2e/cases/widget-starts/expected.exit"), "0\n");
    write(
        &root.join("src/lib.rs"),
        "//! \u{a7}FS-widget.1 \u{2014} the widget's start path, proved by \u{a7}E2E-widget-starts.\npub fn start() {}\n",
    );

    assert_eq!(
        findings(&check_run(&root, false)),
        Vec::<String>::new(),
        "the fixture is clean whole-tree, or a narrow finding would prove nothing"
    );
    assert_eq!(
        findings(&check_run(&root.join("src/lib.rs"), false)),
        Vec::<String>::new(),
        "§FS-config.3.5.8: the cases root is a walk root whether or not `include` names it"
    );
}

/// §FS-check.1.3.6.2: a finding that spans several sites is in scope at any of
/// them. The duplicate message is located at the lexicographically-first site,
/// so a run over the *other* twin has to report it from a site rather than from
/// the anchor — and the diagnostic is kept whole, with the same anchor and the
/// same bytes the whole-tree run prints.
#[test]
fn a_duplicate_is_reported_from_the_twin_it_is_not_anchored_at() {
    let root = widget_repo("a_duplicate_is_reported_from_the_twin_it_is_not_anchored_at");
    write(
        &root.join("docs/functional-spec/FS-widget2.md"),
        "# FS-widget: The widget\n\nThe twin declaration, in a file the path does not name.\n",
    );

    let whole_tree = findings(&check_run(&root, false));
    let twin = check_run(&root.join("docs/functional-spec/FS-widget2.md"), false);
    assert_eq!(
        findings(&twin),
        whole_tree,
        "§FS-check.1.3.6.2: the anchor is the other twin, and the run still reports it"
    );
    assert_eq!(
        codes(&twin),
        vec!["duplicate".to_string()],
        "a run over the twin exits 1 rather than printing `success`"
    );
}

/// §FS-check.1.3.6.2: the same rule, a second code. §FS-values.5.2's value
/// mismatch is anchored at the binding and names the declaration as its site,
/// so a run over the declaring file reports a mismatch it was silent about.
#[test]
fn a_value_mismatch_is_reported_from_the_declaring_file() {
    let root = test_root("a_value_mismatch_is_reported_from_the_declaring_file");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n\
         [reference]\nstrict = true\n\n\
         [id]\nformat = \"{kind}-{slug}\"\nslug_pattern = \"[a-z][a-z0-9-]*\"\n\n\
         [[kinds]]\nkind = \"CONST\"\nfolder = \"values\"\nindex = false\nvalues = true\n\n\
         [scan]\ninclude = [\"docs\"]\nextensions = [\"md\"]\n",
    );
    write(
        &root.join("values/field-price.md"),
        "# CONST-field-price: Reference field price\n## 1. 1200\n",
    );
    write(
        &root.join("docs/offer.md"),
        "The offer stands at `1250` (\u{a7}CONST-field-price.1).\n",
    );

    let whole_tree = check_run(&root, false);
    assert_eq!(
        codes(&whole_tree),
        vec!["value-mismatch".to_string()],
        "the whole-tree run reports the mismatch, got {:?}",
        findings(&whole_tree)
    );

    let declaring = check_run(&root.join("values/field-price.md"), false);
    assert_eq!(
        findings(&declaring),
        findings(&whole_tree),
        "§FS-check.1.3.6.2: the declaration is a site of the finding anchored at the binding"
    );
}

/// §FS-check.1.3.6.3: a file the resolution scope could not read leaves the
/// errors and the exit code — the narrow run is complete about its path — but
/// it is not passed over in silence, because the citation whose declaration lay
/// in it is reported as unresolved and §REQ-no-wrong-citation.2 asks that a
/// false alarm stay legible as one.
#[test]
fn a_file_the_resolution_scope_could_not_read_earns_a_caution() {
    let root = widget_repo("a_file_the_resolution_scope_could_not_read_earns_a_caution");
    std::fs::write(
        root.join("docs/functional-spec/undecodable.md"),
        [0xff, 0xfe],
    )
    .expect("write an undecodable declaration source");

    let whole_tree = check_run(&root, false);
    assert!(
        whole_tree.had_scan_errors,
        "the whole-tree run cannot read it either, or this fixture proves nothing"
    );

    let narrow = check_run(&root.join("src/lib.rs"), false);
    assert!(
        !narrow.had_scan_errors,
        "§FS-check.1.3.6.3: the exit code stays the narrow run's, got {:?}",
        codes(&narrow)
    );
    let caution = narrow
        .report
        .warnings
        .iter()
        .find(|diagnostic| diagnostic.code == "io")
        .unwrap_or_else(|| {
            panic!(
                "expected the unread-source caution, got {:?}",
                codes(&narrow)
            )
        });
    assert!(
        caution.message.contains("outside the report scope"),
        "§FS-check.1.3.6.3: the caution says where the file stands, got {:?}",
        caution.message
    );
    assert!(
        caution.line.is_none(),
        "§FS-check.2.1.1: a CLI-level message goes to stderr, which is what a `None` line selects"
    );
}
