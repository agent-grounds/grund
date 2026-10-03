//! Test module: init next-step and scaffold guidance (§AR-core-module-layout.1.5)

use crate::testing::{test_root, write};
use crate::writers::{InitFsHome, InitOpts, InitOutput, docs_scaffold, init};

#[test]
fn init_next_guidance_uses_effective_legacy_fs_home() {
    let root = test_root("init_next_guidance_uses_effective_legacy_fs_home");
    write(&root.join("grund.toml"), "grund_config_version = 1\n");

    let init_output = init(InitOpts {
        target: root,
        docs: true,
        dry_run: true,
        // §FS-init.1.2.3: a bare temp root no VCS marker covers.
        no_vcs: true,
        ..InitOpts::default()
    })
    .expect("init dry run");
    let next = init_output.next.expect("next guidance");
    assert_eq!(
        next.fs_home,
        InitFsHome::Folder {
            path: "docs/functional-spec".to_string()
        }
    );

    let rendered = next.render();
    assert!(
        rendered.contains("then add it under docs/functional-spec"),
        "next guidance should point at the effective legacy FS home: {rendered}"
    );
    assert!(
        !rendered.contains("requirements.md"),
        "next guidance must not rebuild the new default FS home for compatibility configs: {rendered}"
    );
}

#[test]
fn init_next_guidance_uses_effective_custom_fs_file() {
    let root = test_root("init_next_guidance_uses_effective_custom_fs_file");
    write(
        &root.join("grund.toml"),
        r#"grund_config_version = 1

[[kinds]]
kind = "FS"
title = "Requirements"
file = "specs/requirements.md"
"#,
    );

    let init_output = init(InitOpts {
        target: root,
        docs: true,
        dry_run: true,
        // §FS-init.1.2.3: a bare temp root no VCS marker covers.
        no_vcs: true,
        ..InitOpts::default()
    })
    .expect("init dry run");
    let next = init_output.next.expect("next guidance");
    assert_eq!(
        next.fs_home,
        InitFsHome::File {
            path: "specs/requirements.md".to_string(),
            heading_name: "H2",
            heading_marker: "##",
        }
    );

    let rendered = next.render();
    assert!(
        rendered.contains("then add it to specs/requirements.md"),
        "next guidance should point at the configured FS file: {rendered}"
    );
    assert!(
        !rendered.contains("then add it to requirements.md"),
        "next guidance must not rebuild the generated default FS file for custom configs: {rendered}"
    );
}

#[test]
fn e2e_readme_scaffold_uses_effective_fs_home() {
    let files = docs_scaffold(&InitFsHome::Folder {
        path: "docs/functional-spec".to_string(),
    });
    let e2e_readme = files
        .iter()
        .find(|(path, _)| path == "tests/e2e/README.md")
        .map(|(_, contents)| contents)
        .expect("e2e README scaffold");

    assert!(
        e2e_readme.contains("`docs/functional-spec`"),
        "e2e README should name the effective FS home: {e2e_readme}"
    );
    assert!(
        !e2e_readme.contains("`requirements.md`"),
        "e2e README must not hard-code the generated default for legacy/custom homes: {e2e_readme}"
    );

    let files = docs_scaffold(&InitFsHome::File {
        path: "specs/requirements.md".to_string(),
        heading_name: "H2",
        heading_marker: "##",
    });
    let e2e_readme = files
        .iter()
        .find(|(path, _)| path == "tests/e2e/README.md")
        .map(|(_, contents)| contents)
        .expect("e2e README scaffold");
    assert!(
        e2e_readme.contains("`specs/requirements.md`"),
        "e2e README should name the configured FS file: {e2e_readme}"
    );
}

/// Runs `init` over a tree it first initialised, after a `grund.toml` change
/// that made the managed block stale: the refresh `grund check` asks for.
fn refresh_after_config_change(name: &str, fs_home_exists: bool) -> InitOutput {
    let root = test_root(name);
    let config = "grund_config_version = 1\n\n[[kinds]]\nkind = \"FS\"\n\
                  folder = \"docs/functional-spec\"\ntitle = \"What\"\n";
    write(&root.join("grund.toml"), config);
    if fs_home_exists {
        write(
            &root.join("docs/functional-spec/README.md"),
            "# Functional spec\n",
        );
    }
    let opts = || InitOpts {
        target: root.clone(),
        // §FS-init.1.2.3: a bare temp root no VCS marker covers.
        no_vcs: true,
        ..InitOpts::default()
    };
    init(opts()).expect("first init");
    write(
        &root.join("grund.toml"),
        &format!(
            "{config}\n[[kinds]]\nkind = \"RFC\"\nfolder = \"docs/rfcs\"\ntitle = \"Proposals\"\n"
        ),
    );
    let output = init(opts()).expect("refresh init");
    let verbs: Vec<_> = output
        .events
        .iter()
        .map(|e| (e.verb, e.path.as_str()))
        .collect();
    assert_eq!(verbs, [("updated", "AGENTS.md"), ("exists", "grund.toml")]);
    output
}

#[test]
fn refresh_of_a_complete_setup_has_no_next_guidance() {
    // §FS-init.2.2.2.1: a refresh whose effective FS home exists teaches nothing.
    let output = refresh_after_config_change("refresh_of_a_complete_setup", true);
    assert!(
        output.next.is_none(),
        "a refresh of a complete setup must not print the next: block: {:?}",
        output.next.map(|next| next.render())
    );
}

#[test]
fn refresh_without_the_fs_home_keeps_next_guidance() {
    // §FS-init.2.2.2.1: the counter-case keeps the block, step 1 included.
    let output = refresh_after_config_change("refresh_without_the_fs_home", false);
    let rendered = output.next.expect("next guidance").render();
    assert!(
        rendered.contains("1. re-run with --docs to scaffold the FS home (docs/functional-spec)"),
        "a refresh with no FS home must keep the scaffold advice: {rendered}"
    );
}
