//! Effective input/anchor evidence from shared discovery (§FS-check.6.1.3).
use crate::support::*;
use grund::{WatchObservation, watch_test};
use grund_core::{CheckFindingSelection, CheckInput};
use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
};

fn edit_case(setup: impl FnOnce(&Fixture), edit: impl FnOnce(&Fixture)) {
    let f = Fixture::new();
    setup(&f);
    let before = f.ordinary();
    let mut h = Harness::start(&f, None, None);
    h.matches(&before);
    edit(&f);
    let after = f.ordinary();
    assert!(
        before.stdout != after.stdout
            || before.stderr != after.stderr
            || before.status != after.status,
        "inventory edit must distinguish ordinary output"
    );
    h.matches(&after);
    h.stop(after.status);
}

#[test]
fn watch_effective_input_inventory_matrix() {
    let _serial = crate::support::serial();
    // Both names/precedence, invalid initial coverage and last-usable inventory.
    edit_case(
        |f| f.write(".agents/grund.toml", "grund_config_version = \"bad\"\n"),
        |f| {
            std::fs::remove_file(f.0.join("grund.toml")).unwrap();
        },
    );
    edit_case(
        |f| f.write("grund.toml", "grund_config_version = \"bad\"\n"),
        |f| f.write("grund.toml", CONFIG),
    );
    edit_case(
        |_| {},
        |f| f.write("grund.toml", "grund_config_version = \"bad\"\n"),
    );
    // Resolution-wide kind homes, ignored include paths and root replacement.
    edit_case(
        |f| f.write("src/main.rs", BAD),
        |f| f.write(".ignore", "src/main.rs\n"),
    );
    edit_case(
        |f| f.write("src/main.rs", BAD),
        |f| f.write("src/.ignore", "main.rs\n"),
    );
    edit_case(
        |f| f.write("src/main.rs", BAD),
        |f| {
            f.write(
                "docs/functional-spec/FS-missing.md",
                "# FS-missing: Now declared\n",
            )
        },
    );
    // Index/agent/checker probes and an absent stub target.
    edit_case(
        |_| {},
        |f| f.write("docs/functional-spec/README.md", "# FS index\n"),
    );
    edit_case(|_| {}, |f| f.write("AGENTS.md", "\u{a7}FS-absent\n"));
    edit_case(
        |f| {
            f.write(
                "docs/functional-spec/FS-live.md",
                "# FS-live: [definition](../../impl.rs)\n",
            )
        },
        |f| f.write("impl.rs", "/// FS-live: Definition\n"),
    );
    // Workspace membership including absent optional and trailing-glob parents.
    edit_case(
        |f| {
            f.write(
                "grund.toml",
                &format!("{CONFIG}\n[workspace]\nmembers = [\"packages/*\"]\n"),
            )
        },
        |f| {
            f.write("packages/a/grund.toml", "grund_config_version = 1\nproject_name = \"a\"\n[id]\nformat = \"{kind}-{slug}\"\n");
            f.write("packages/a/src/main.rs", "// \u{a7}FS-member-missing\n");
        },
    );
    edit_case(
        |f| {
            f.write(
                "grund.toml",
                &format!(
                    "{CONFIG}\n[workspace]\nmembers = []\noptional_members = [\"optional\"]\n"
                ),
            )
        },
        |f| {
            f.write("optional/grund.toml", "grund_config_version = 1\nproject_name = \"optional\"\n[id]\nformat = \"{kind}-{slug}\"\n");
            f.write("optional/src/main.rs", "// \u{a7}FS-member-missing\n");
        },
    );
    edit_case(
        |f| {
            f.write("grund.toml", &format!("{CONFIG}\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs/functional-spec\"\n[[kinds]]\nkind = \"CONST\"\nfolder = \"values\"\nvalues = true\nindex = false\n"));
            f.write("values/catalog.json", "{}");
            f.write("src/main.rs", "// \u{a7}CONST-region\n");
        },
        |f| f.write("values/catalog.json", "{\"CONST-region\": [\"CH\"]}"),
    );
    edit_case(
        |f| {
            std::fs::create_dir_all(f.0.join(".git/info")).unwrap();
            f.write("src/main.rs", BAD);
        },
        |f| f.write(".git/info/exclude", "src/main.rs\n"),
    );
    edit_case(
        |f| f.write("src/main.rs", BAD),
        |f| {
            std::fs::create_dir_all(f.0.join(".git")).unwrap();
            f.write(".gitignore", "src/main.rs\n");
        },
    );
}

#[test]
fn watch_inventory_ancestor_claims_and_ignore() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    f.write("child/grund.toml", CONFIG);
    f.write("child/src/main.rs", BAD);
    let mut opts = f.opts();
    opts.path = f.0.join("child");
    let before = watch_test::one_shot(opts.clone(), &CheckFindingSelection::default(), None);
    let mut h = Harness::options(
        opts.clone(),
        CheckFindingSelection::default(),
        None,
        None,
        None,
    );
    h.matches(&before);
    f.write(".ignore", "child/src/main.rs\n");
    let ignored = watch_test::one_shot(opts.clone(), &CheckFindingSelection::default(), None);
    assert_ne!(before.stdout, ignored.stdout);
    h.matches(&ignored);
    f.write(
        "grund.toml",
        &format!("{CONFIG}\n[workspace]\nmembers = [\"child\"]\n"),
    );
    let claimed = watch_test::one_shot(opts, &CheckFindingSelection::default(), None);
    assert!(
        claimed.stdout != ignored.stdout
            || claimed.stderr != ignored.stderr
            || claimed.status != ignored.status
    );
    h.matches(&claimed);
    h.stop(claimed.status);
}

#[test]
fn watch_inventory_nested_members_addition_removal() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    f.write(
        "grund.toml",
        &format!("{CONFIG}\n[workspace]\nmembers = [\"group\"]\n"),
    );
    f.write("group/grund.toml", "grund_config_version = 1\nproject_name = \"group\"\n[id]\nformat = \"{kind}-{slug}\"\n[workspace]\nmembers = [\"packages/*\"]\n");
    std::fs::create_dir_all(f.0.join("group/packages")).unwrap();
    let mut h = Harness::start(&f, None, None);
    let before = h.completed();
    f.write(
        "group/packages/leaf/grund.toml",
        "grund_config_version = 1\nproject_name = \"leaf\"\n[id]\nformat = \"{kind}-{slug}\"\n",
    );
    f.write(
        "group/packages/leaf/src/main.rs",
        "// \u{a7}FS-nested-missing\n",
    );
    let added = f.ordinary();
    assert!(added.stdout != before.stdout || added.stderr != before.stderr);
    h.matches(&added);
    std::fs::remove_dir_all(f.0.join("group/packages/leaf")).unwrap();
    h.matches(&before);
    h.stop(before.status);
}

#[test]
fn watch_inventory_shallow_filters_and_missing_anchors() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    let missing = f.0.join("absent/nested/grund.toml");
    let input = CheckInput {
        path: missing.clone(),
        recursive: false,
    };
    let coverage = watch_test::coverage(input.clone());
    assert_eq!(coverage.get(&f.0), Some(&false));
    assert!(!watch_test::event_relevant(
        &[input.clone()],
        &changed_event(f.0.join("unrelated.txt"))
    ));
    assert!(watch_test::event_relevant(
        &[input],
        &changed_event(f.0.join("absent"))
    ));
    let tree = CheckInput {
        path: f.0.join("src"),
        recursive: true,
    };
    let coverage = watch_test::coverage(tree);
    assert_eq!(coverage.get(&f.0.join("src")), Some(&true));
    assert_eq!(coverage.get(&f.0), Some(&false));
}

#[test]
fn watch_inventory_shared_resolver_and_narrowed_scope() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    let inputs = Arc::new(Mutex::new(BTreeSet::new()));
    let record = inputs.clone();
    let observer: grund_core::CheckInputObserver = Arc::new(move |input| {
        record.lock().unwrap().insert(input);
        true
    });
    let mut opts = f.opts();
    opts.path = f.0.join("src/main.rs");
    opts.path_provided = true;
    grund_core::with_check_input_observer(Some(observer), || {
        let _ = grund_core::check_with_run_warnings(opts);
    });
    let inputs = inputs.lock().unwrap();
    for path in [
        "grund.toml",
        ".agents/grund.toml",
        "docs/functional-spec",
        "src",
        ".ignore",
        ".gitignore",
        ".git/info/exclude",
        "AGENTS.md",
    ] {
        assert!(
            inputs.iter().any(|input| input.path == f.0.join(path)),
            "missing shared input {path}"
        );
    }
    assert!(
        inputs
            .iter()
            .any(|input| input.path.ends_with(".gitconfig"))
    );
    assert!(
        inputs
            .iter()
            .any(|input| input.path.ends_with("git/config"))
    );
    assert!(
        inputs
            .iter()
            .any(|input| input.path.ends_with("git/ignore"))
    );
}

#[cfg(unix)]
#[test]
fn watch_inventory_followed_external_target_replacement() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    f.write("external/target.rs", CLEAN);
    std::os::unix::fs::symlink(f.0.join("external/target.rs"), f.0.join("src/link.rs")).unwrap();
    let mut h = Harness::start(&f, None, None);
    let initial = h.completed();
    f.write("external/next.rs", BAD);
    std::fs::rename(f.0.join("external/next.rs"), f.0.join("external/target.rs")).unwrap();
    let after = f.ordinary();
    assert_ne!(initial.stdout, after.stdout);
    h.matches(&after);
    h.stop(after.status);
    assert!(h.history.iter().any(|e| matches!(e, WatchObservation::Input(input) if input.path == f.0.join("external/target.rs"))));
}

#[test]
fn watch_inventory_narrowed_resolution_change() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    f.write("src/main.rs", BAD);
    let mut opts = f.opts();
    opts.path = f.0.join("src/main.rs");
    opts.path_provided = true;
    let mut h = Harness::options(
        opts.clone(),
        CheckFindingSelection::default(),
        None,
        None,
        None,
    );
    let before = h.completed();
    f.write(
        "docs/functional-spec/FS-missing.md",
        "# FS-missing: Repaired\n",
    );
    let after = watch_test::one_shot(opts, &CheckFindingSelection::default(), None);
    assert_ne!(before.stdout, after.stdout);
    h.matches(&after);
    h.stop(0);
}
