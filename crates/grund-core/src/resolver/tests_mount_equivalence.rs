//! Test module: the mount layer answers as the mounted tree's own standalone run
//! does (§REQ-remote-equivalence.4, §FS-remote-projects.mount.rule), with its ignore
//! inputs isolated (§FS-remote-projects.ignore). The consumer's edge is set on its
//! run directly, so the proof holds the mount layer apart from the import table.
//! These are the catalog-level rows; the command rows, one per command the binary
//! lists, are `crates/grund-cli/tests/mount_operations.rs`, where the commands are.

use std::path::{Path, PathBuf};

use super::context::{WorkspaceProject, load_workspace_projects};
use crate::config::{MountEdge, ProjectRecords, load_config_at};
use crate::testing::{canonical_test_path, test_root, write};

const KINDS: &str = "[schema.kinds.FS]\nfolders = [\"docs/fs\"]\n";

/// One operation of the mount layer's inventory: what a reader downstream of the
/// load reads off a project (§REQ-remote-equivalence.4). The match in [`answer`]
/// has no default arm, so an operation added here without a comparison does not
/// compile, and [`Operation::ALL`] is checked against the same match.
#[derive(Clone, Copy, Debug)]
enum Operation {
    Declarations,
    Sections,
    ScannedFiles,
    ScanErrors,
    CitationTargets,
    ValueBindings,
}

impl Operation {
    const ALL: [Self; 6] = [
        Self::Declarations,
        Self::Sections,
        Self::ScannedFiles,
        Self::ScanErrors,
        Self::CitationTargets,
        Self::ValueBindings,
    ];
}

/// `project`'s answer to `operation`, with the one declared relocation field, the
/// namespace a target is named by, passed through `namespace` (§REQ-remote-equivalence.4:
/// only declared relocation fields are normalized, never arbitrary text).
fn answer(
    operation: Operation,
    project: &WorkspaceProject,
    namespace: &dyn Fn(Option<&str>) -> Option<String>,
) -> Vec<String> {
    let findings = &project.findings;
    let mut rows: Vec<String> = match operation {
        Operation::Declarations => findings
            .declarations
            .iter()
            .flat_map(|(id, decls)| {
                decls
                    .iter()
                    .map(move |decl| format!("{id:?}|{}|{}", decl.file.display(), decl.line))
            })
            .collect(),
        Operation::Sections => findings
            .declarations
            .iter()
            .flat_map(|(id, decls)| {
                decls.iter().flat_map(move |decl| {
                    decl.sections
                        .keys()
                        .map(move |path| format!("{id:?}|{path}"))
                })
            })
            .collect(),
        Operation::ScannedFiles => findings
            .scanned_files
            .iter()
            .map(|file| file.display().to_string())
            .collect(),
        Operation::ScanErrors => project
            .scan_errors
            .iter()
            .map(|(file, message)| format!("{}|{message}", file.display()))
            .collect(),
        Operation::CitationTargets => findings
            .edges()
            .chain(&findings.escaped_citations)
            .map(|cite| {
                format!(
                    "{}|{}|{}|{:?}|{:?}|{:?}",
                    cite.file.display(),
                    cite.line,
                    cite.column,
                    namespace(cite.namespace.as_deref()),
                    cite.id,
                    cite.section
                )
            })
            .collect(),
        Operation::ValueBindings => findings
            .value_bindings
            .iter()
            .map(|binding| {
                format!(
                    "{}|{}|{:?}|{:?}|{:?}",
                    binding.file.display(),
                    binding.line,
                    namespace(binding.namespace.as_deref()),
                    binding.id,
                    binding.section
                )
            })
            .collect(),
    };
    rows.sort();
    rows
}

/// `text` with each `@` the citation marker, so the fixture's citations are not this
/// file's own.
fn cites(text: &str) -> String {
    text.replace('@', "§")
}

/// A mounted tree inside a git checkout whose ignore files above it, and whose
/// `info/exclude`, would each hide a declaration of its own run in place: `lib`,
/// a workspace with member `core`, under `outer`, and the consumer `app` beside it.
fn fixture(name: &str) -> (PathBuf, PathBuf) {
    let base = canonical_test_path(&test_root(name));
    let outer = base.join("outer");
    let lib = outer.join("lib");
    write(&outer.join(".git/info/exclude"), "FS-excluded.md\n");
    write(&outer.join(".gitignore"), "FS-above.md\nFS-core-above.md\n");
    write(
        &lib.join(".gitignore"),
        "FS-own.md\nFS-core-own.md\n/core/docs/fs/FS-core-anchored.md\n",
    );
    write(
        &lib.join("grund.toml"),
        &format!(
            "grund_config_version = 2\nproject_name = \"lib\"\n\n{KINDS}\n[workspace]\nmembers = [\"core\"]\n"
        ),
    );
    write(
        &lib.join("docs/fs/FS-x.md"),
        &cites("# FS-x: X\n\n## 1. One\n\nCites @core/FS-y, @FS-z, @lib/FS-z and @FS-x.1.\n"),
    );
    write(&lib.join("docs/fs/FS-z.md"), "# FS-z: Z\n");
    write(&lib.join("docs/fs/FS-above.md"), "# FS-above: Above\n");
    write(
        &lib.join("docs/fs/FS-excluded.md"),
        "# FS-excluded: Excluded\n",
    );
    write(&lib.join("docs/fs/FS-own.md"), "# FS-own: Own\n");
    write(
        &lib.join("core/grund.toml"),
        &format!("grund_config_version = 2\nproject_name = \"core\"\n\n{KINDS}"),
    );
    write(
        &lib.join("core/docs/fs/FS-core-above.md"),
        "# FS-core-above: Above\n",
    );
    write(
        &lib.join("core/docs/fs/FS-core-own.md"),
        "# FS-core-own: Own\n",
    );
    write(
        &lib.join("core/docs/fs/FS-core-anchored.md"),
        "# FS-core-anchored: Anchored\n",
    );
    write(
        &lib.join("core/docs/fs/FS-y.md"),
        &cites("# FS-y: Y\n\nCites @FS-y and @lib/FS-x.\n"),
    );
    let app = base.join("app");
    write(
        &app.join("grund.toml"),
        &format!("grund_config_version = 2\nproject_name = \"app\"\n\n{KINDS}"),
    );
    write(
        &app.join("docs/fs/FS-a.md"),
        &cites("# FS-a: A\n\nCites @dep/FS-x.1 and @dep/core/FS-y.\n"),
    );
    (canonical_test_path(&lib), canonical_test_path(&app))
}

/// `lib` as its own standalone run reads it in the committed portable scan context
/// (§REQ-remote-equivalence.4, §FS-remote-projects.ignore).
fn standalone(lib: &Path) -> Vec<WorkspaceProject> {
    let mut config = load_config_at(lib, lib).expect("lib loads");
    config.set_mounted_root(lib.to_path_buf());
    load_workspace_projects(&mut ProjectRecords::of(config)).expect("lib's own run")
}

/// The consumer's run with `lib` mounted as `dep`.
fn mounted(lib: &Path, app: &Path) -> Vec<WorkspaceProject> {
    let mut config = load_config_at(app, app).expect("app loads");
    config.set_mount_edges(vec![MountEdge {
        alias: "dep".into(),
        segment: "dep".into(),
        written: "../outer/lib".into(),
        root: lib.to_path_buf(),
        source: None,
        missing: false,
        project: "lib".into(),
        project_source: None,
    }]);
    load_workspace_projects(&mut ProjectRecords::of(config)).expect("app's run")
}

/// §FS-remote-projects.mount.rule: the native root alias is the mount alias and
/// every other alias path gains it in front.
fn shown(native: &str) -> String {
    match native {
        "lib" => "dep".to_string(),
        other => format!("dep/{other}"),
    }
}

/// §REQ-remote-equivalence.4: source against projection, operation by operation.
#[test]
fn a_mounted_tree_answers_every_inventoried_operation_as_its_standalone_run() {
    let (lib, app) = fixture("a_mounted_tree_answers_every_inventoried_operation");
    let native = standalone(&lib);
    let run = mounted(&lib, &app);
    assert_eq!(
        run.iter().map(|p| p.alias.as_str()).collect::<Vec<_>>(),
        ["app", "dep", "dep/core"],
        "the mount is grafted after the consumer's own projects, in alias order"
    );
    for source in &native {
        let projected = run
            .iter()
            .find(|project| project.alias == shown(&source.alias))
            .unwrap_or_else(|| panic!("`{}` is in the consumer run", source.alias));
        assert!(
            !projected.reports_findings(),
            "a mounted project's findings are suppressed"
        );
        for operation in Operation::ALL {
            let native_namespace = |ns: Option<&str>| ns.map(shown);
            let as_is = |ns: Option<&str>| ns.map(str::to_string);
            assert_eq!(
                answer(operation, projected, &as_is),
                answer(operation, source, &native_namespace),
                "{operation:?} of `{}` differs between its own run and the mount",
                source.alias
            );
        }
    }
    let consumer = &run[0];
    assert!(consumer.reports_findings() && consumer.mount.is_none());
    let targets = consumer
        .findings
        .citations
        .iter()
        .map(|cite| cite.namespace.clone())
        .collect::<Vec<_>>();
    assert_eq!(targets, [Some("dep".into()), Some("dep/core".into())]);
}

/// §FS-remote-projects.ignore, §FS-remote-projects.differences.ignore: a mounted root
/// reads its own ignore files and none above it, `.git/info/exclude` included, where
/// its in-place run reads all three.
#[test]
fn a_mounted_root_reads_only_the_ignore_files_at_and_below_it() {
    let (lib, app) = fixture("a_mounted_root_reads_only_the_ignore_files_at_and_below_it");
    let ids = |projects: &[WorkspaceProject]| {
        let mut ids = projects
            .iter()
            .filter(|project| project.config.root.starts_with(&lib))
            .flat_map(|project| project.findings.declarations.keys())
            .map(|id| format!("{id:?}"))
            .collect::<Vec<_>>();
        ids.sort();
        ids
    };
    let in_place = load_config_at(&lib, &lib).expect("lib loads");
    let in_place = load_workspace_projects(&mut ProjectRecords::of(in_place)).expect("in place");
    let in_place = ids(&in_place);
    let mounted = ids(&mounted(&lib, &app));
    for hidden_above in ["\"above", "core-above", "excluded"] {
        assert!(
            !in_place.iter().any(|id| id.contains(hidden_above)),
            "the fixture's checkout hides FS-{hidden_above} from lib's in-place run"
        );
        assert!(
            mounted.iter().any(|id| id.contains(hidden_above)),
            "no ignore file above the mounted root hides FS-{hidden_above}"
        );
    }
    // An anchored line is matched from the directory of the file that holds it.
    for own in ["\"own", "core-own", "core-anchored"] {
        assert!(
            !mounted.iter().any(|id| id.contains(own)),
            "the mounted root's own `.gitignore` still hides FS-{own}, in a member too"
        );
    }
}

/// §FS-imports.declaration.table, §FS-imports.mount: the edge an import table
/// declares is the one the proof above sets by hand, so the consumer's run through its
/// own config answers every inventoried operation as the run through the bare edge.
#[test]
fn an_import_table_mounts_the_tree_as_the_bare_edge_does() {
    let (lib, app) = fixture("an_import_table_mounts_the_tree_as_the_bare_edge_does");
    let by_edge = mounted(&lib, &app);
    write(
        &app.join("grund.toml"),
        &format!(
            "grund_config_version = 2\nproject_name = \"app\"\n\n{KINDS}\n\
             [workspace.imports.dep]\npath = \"../outer/lib\"\nproject = \"lib\"\n"
        ),
    );
    let config = load_config_at(&app, &app).expect("app loads its import table");
    let by_table = load_workspace_projects(&mut ProjectRecords::of(config)).expect("app's run");
    let aliases = |run: &[WorkspaceProject]| {
        run.iter()
            .map(|p| (p.alias.clone(), p.mount.is_some()))
            .collect::<Vec<_>>()
    };
    assert_eq!(aliases(&by_table), aliases(&by_edge));
    let as_is = |ns: Option<&str>| ns.map(str::to_string);
    for (table, edge) in by_table.iter().zip(&by_edge) {
        for operation in Operation::ALL {
            assert_eq!(
                answer(operation, table, &as_is),
                answer(operation, edge, &as_is),
                "{operation:?} of `{}` differs between the table and the edge",
                edge.alias
            );
        }
    }
}

/// §FS-imports.graph.cycle: a mount whose canonical root is already on the chain
/// that reached it is refused, naming the chain.
#[test]
fn a_mount_of_the_consumer_itself_closes_a_cycle() {
    let (_, app) = fixture("a_mount_of_the_consumer_itself_closes_a_cycle");
    let mut config = load_config_at(&app, &app).expect("app loads");
    config.set_mount_edges(vec![MountEdge {
        alias: "back".into(),
        segment: "back".into(),
        written: ".".into(),
        root: app.clone(),
        source: None,
        missing: false,
        project: "app".into(),
        project_source: None,
    }]);
    let error = load_workspace_projects(&mut ProjectRecords::of(config))
        .err()
        .expect("a cycle is refused");
    assert!(
        format!("{error:#}").contains("import cycle: app → back"),
        "{error:#}"
    );
}
