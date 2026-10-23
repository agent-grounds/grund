//! Test module: the kind and place init's guidance names, and the refresh test
//! that reads the same selection (§FS-init.2.2.2.3, §FS-init.2.2.2.1).

use std::fs;
use std::path::{Path, PathBuf};

use crate::testing::{test_root, write};
use crate::writers::{InitAllocation, InitAllocationHome, InitFsHome, InitOpts, InitOutput, init};

const V2: &str = "grund_config_version = 2\nproject_name = \"demo\"\n";

/// A v1 project whose one kind, `REQ`, is homed in `docs/req` (§FS-init-fixtures.8).
const REQ_V1: &str = "grund_config_version = 1\n\n[[kinds]]\nkind = \"REQ\"\n\
                      folder = \"docs/req\"\n\n[scan]\ninclude = [\"README.md\"]\n";

/// A fresh root holding `config`, a `README.md`, and every directory in `dirs`.
fn tree(name: &str, config: &str, dirs: &[&str]) -> PathBuf {
    let root = test_root(name);
    write(&root.join("grund.toml"), config);
    write(&root.join("README.md"), "# Demo\n");
    for dir in dirs {
        fs::create_dir_all(root.join(dir)).expect("create home");
    }
    root
}

/// `init` over `root`, a bare temp root no VCS marker covers (§FS-init.1.2.3).
fn run(root: &Path, dry_run: bool, check: bool) -> InitOutput {
    init(InitOpts {
        target: root.to_path_buf(),
        dry_run,
        check,
        no_vcs: true,
        ..InitOpts::default()
    })
    .expect("init")
}

/// The selection a first run over `config` and `dirs` makes.
fn selection(name: &str, config: &str, dirs: &[&str]) -> Option<InitAllocation> {
    let next = run(&tree(name, config, dirs), false, false).next;
    next.expect("a first run teaches").allocation
}

fn allocation(kind: &str, home: Option<InitAllocationHome>, home_exists: bool) -> InitAllocation {
    InitAllocation {
        kind: kind.to_string(),
        home,
        home_exists,
    }
}

fn folder(path: &str) -> Option<InitAllocationHome> {
    Some(InitAllocationHome::Folder {
        path: path.to_string(),
    })
}

/// Initialise `root`, then give the `REQ` row a title, which makes the managed
/// block stale: the refresh `grund check` asks for (§FS-check.3.5).
fn stale(root: &Path) {
    run(root, false, false);
    let config = fs::read_to_string(root.join("grund.toml")).expect("config");
    let titled = config.replacen("kind = \"REQ\"\n", "kind = \"REQ\"\ntitle = \"What\"\n", 1);
    assert_ne!(config, titled, "the config names REQ");
    write(&root.join("grund.toml"), &titled);
}

/// The refresh of `config` over `dirs`, which reports only the updated block.
fn refresh(name: &str, config: &str, dirs: &[&str]) -> InitOutput {
    let root = tree(name, config, dirs);
    stale(&root);
    let output = run(&root, false, false);
    let verbs: Vec<_> = output
        .events
        .iter()
        .map(|e| (e.verb, e.path.as_str()))
        .collect();
    assert_eq!(verbs, [("updated", "AGENTS.md"), ("exists", "grund.toml")]);
    output
}

/// §FS-init.2.2.2.3: a row's first file comes before its first folder, whichever
/// key is written first, and the folder existing does not move the selection.
#[test]
fn a_rows_first_file_is_selected_before_its_folder() {
    let rows = "\n[schema.kinds.REQ]\nfolders = [\"docs/req\"]\nfiles = [\"requirements.md\"]\n";
    let file = Some(InitAllocationHome::File {
        path: "requirements.md".to_string(),
        heading_name: "H2",
        heading_marker: "##",
    });
    assert_eq!(
        selection(
            "allocation_file_first",
            &format!("{V2}{rows}"),
            &["docs/req"]
        ),
        Some(allocation("REQ", file, false))
    );
}

/// §FS-init.2.2.2.3: the first place in written order, though a later one exists.
#[test]
fn the_first_place_is_selected_though_a_later_one_exists() {
    let rows = "\n[schema.kinds.REQ]\nfolders = [\"docs/a\", \"docs/b\"]\n";
    assert_eq!(
        selection(
            "allocation_first_place",
            &format!("{V2}{rows}"),
            &["docs/b"]
        ),
        Some(allocation("REQ", folder("docs/a"), false))
    );
}

/// §FS-init.2.2.2.3: a non-citable row never qualifies, the complement included.
#[test]
fn non_citable_rows_and_the_complement_are_skipped() {
    let rows = "\n[schema.kinds.code]\nfolders = [\"src\"]\n\n[schema.kinds.runbook]\n\
                citable = false\nfolders = [\"docs/runbooks\"]\n\n[schema.kinds.ADR]\n\
                folders = [\"docs/adr\"]\n";
    let dirs = ["src", "docs/runbooks", "docs/adr"];
    assert_eq!(
        selection("allocation_non_citable", &format!("{V2}{rows}"), &dirs),
        Some(allocation("ADR", folder("docs/adr"), true))
    );
}

/// §FS-init.2.2.2.3: an FS row with no home is not FS with a home, so the first
/// row with one wins; the scaffold keeps the built-in FS folder for itself.
#[test]
fn an_fs_row_without_a_home_falls_through_to_a_row_with_one() {
    let rows = "\n[schema.kinds.FS]\n\n[schema.kinds.REQ]\nfolders = [\"docs/req\"]\n";
    let root = tree("allocation_fs_without_home", &format!("{V2}{rows}"), &[]);
    let next = run(&root, false, false).next.expect("a first run teaches");
    assert_eq!(
        next.allocation,
        Some(allocation("REQ", folder("docs/req"), false))
    );
    assert_eq!(
        next.fs_home,
        InitFsHome::Folder {
            path: "docs/functional-spec".to_string()
        }
    );
}

/// §FS-init.2.2.2.3: where no citable row has a home the first is selected, with
/// no home and so none on disk; the block names no directory.
#[test]
fn a_selection_without_a_home_has_none_on_disk() {
    let rows = "\n[schema.kinds.NOTE]\n\n[schema.kinds.FS]\n";
    let root = tree("allocation_no_home", &format!("{V2}{rows}"), &[]);
    let next = run(&root, false, false).next.expect("a first run teaches");
    assert_eq!(next.allocation, Some(allocation("NOTE", None, false)));
    let rendered = next.render();
    assert!(
        rendered.contains("ID=$(grund id NOTE \"…\")  then declare it in a source doc-comment"),
        "{rendered}"
    );
    assert!(!rendered.contains("docs/functional-spec"), "{rendered}");
}

/// §FS-init.2.2.2.3: a first run names an existing home and never asks for it.
#[test]
fn a_first_run_names_an_existing_home_without_asking_for_it() {
    let root = tree("allocation_first_existing", REQ_V1, &["docs/req"]);
    let next = run(&root, false, false).next.expect("a first run teaches");
    assert_eq!(
        next.allocation,
        Some(allocation("REQ", folder("docs/req"), true))
    );
    assert_eq!(
        next.render(),
        "\nnext:\n  1. run `grund check`\n  \
         2. allocate an ID:  ID=$(grund id REQ \"…\")  then add it under docs/req\n\
         see AGENTS.md for the full workflow.\n"
    );
}

/// §FS-init.2.2.2.3: a first run asks for a missing home, and never by `--scaffold`.
#[test]
fn a_first_run_asks_for_a_missing_home() {
    let root = tree("allocation_first_missing", REQ_V1, &[]);
    let rendered = run(&root, false, false).next.expect("guidance").render();
    assert!(
        rendered.contains("\n  1. create the REQ home (docs/req)\n  2. run `grund check`\n"),
        "{rendered}"
    );
    assert!(!rendered.contains("--scaffold"), "{rendered}");
}

/// §FS-init.2.2.2.1: a refresh whose selected home exists teaches nothing.
#[test]
fn a_refresh_with_the_selected_home_teaches_nothing() {
    let output = refresh("allocation_refresh_existing", REQ_V1, &["docs/req"]);
    assert_eq!(output.next, None);
}

/// §FS-init.2.2.2.1: a refresh whose selected home is absent keeps the block.
#[test]
fn a_refresh_without_the_selected_home_keeps_the_block() {
    let output = refresh("allocation_refresh_missing", REQ_V1, &[]);
    let rendered = output.next.expect("guidance").render();
    assert!(
        rendered.contains("1. create the REQ home (docs/req)"),
        "{rendered}"
    );
}

/// §FS-init.2.2.2.1: a refresh of a row with no home has no home left to create.
#[test]
fn a_refresh_of_a_row_without_a_home_teaches_nothing() {
    let config = REQ_V1.replace("folder = \"docs/req\"\n", "");
    let output = refresh("allocation_refresh_no_home", &config, &[]);
    assert_eq!(output.next, None);
}

/// §FS-init.2.2.2.1: the previews see the same complete refresh. Neither writes,
/// and `--check` still reports the pending update it exits `1` on.
#[test]
fn the_previews_of_a_complete_refresh_write_nothing_and_teach_nothing() {
    let root = tree("allocation_refresh_previews", REQ_V1, &["docs/req"]);
    stale(&root);
    let before = fs::read(root.join("AGENTS.md")).expect("entrypoint");
    for check in [false, true] {
        let output = run(&root, true, check);
        let verbs: Vec<_> = output
            .events
            .iter()
            .map(|e| (e.verb, e.path.as_str()))
            .collect();
        assert_eq!(
            verbs,
            [("would-update", "AGENTS.md"), ("exists", "grund.toml")]
        );
        assert_eq!(output.next, None, "check: {check}");
        assert!(output.has_pending_changes(), "check: {check}");
        assert_eq!(
            fs::read(root.join("AGENTS.md")).expect("entrypoint"),
            before
        );
    }
}

/// §FS-init.2.2.2.3, §FS-init.2.2.2.2: `allocation` is `None` exactly where the
/// config has no citable row, the answer `declares_kinds` gives.
#[test]
fn allocation_is_none_exactly_without_a_citable_row() {
    let non_citable = "\n[schema.kinds.runbook]\ncitable = false\nfolders = [\"docs/runbooks\"]\n";
    let built_in = Some(allocation("FS", folder("docs/functional-spec"), false));
    for (name, config, expected) in [
        ("allocation_none_no_kinds", V2.to_string(), None),
        (
            "allocation_none_non_citable",
            format!("{V2}{non_citable}"),
            None,
        ),
        (
            "allocation_built_in_kinds",
            "grund_config_version = 1\n".to_string(),
            built_in,
        ),
    ] {
        let next = run(&tree(name, &config, &[]), false, false)
            .next
            .expect("guidance");
        assert_eq!(next.declares_kinds, next.allocation.is_some(), "{name}");
        assert_eq!(next.allocation, expected, "{name}");
    }
}
