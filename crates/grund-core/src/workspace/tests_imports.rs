//! Test module: an import's path is judged on the canonical directory it names
//! (§FS-imports.declaration.path): a symlink is followed exactly as far as the
//! directory whose identity is then checked, and two spellings of one root are one
//! target.

use super::expand_workspace_tree;
use crate::config::load_config_at;
use crate::testing::{canonical_test_path, test_root, write};

/// A consumer `app` beside a target `lib`, with `link` a symlink to `app` itself and
/// `alias` one to `lib`, and `app` importing `dep` from `path`.
fn consumer(name: &str, path: &str) -> std::path::PathBuf {
    let base = canonical_test_path(&test_root(name));
    let app = base.join("app");
    let lib = base.join("lib");
    write(
        &lib.join("grund.toml"),
        "grund_config_version = 2\nproject_name = \"lib\"\n",
    );
    write(
        &app.join("grund.toml"),
        &format!(
            "grund_config_version = 2\nproject_name = \"app\"\n\n\
             [workspace.imports.dep]\npath = \"{path}\"\nproject = \"lib\"\n"
        ),
    );
    std::os::unix::fs::symlink(&app, base.join("link")).expect("link to app");
    std::os::unix::fs::symlink(&lib, base.join("alias")).expect("link to lib");
    app
}

/// §FS-imports.failures: a symlink that resolves to the consumer is the consumer.
#[test]
fn an_import_through_a_symlink_to_its_own_root_is_refused_at_its_path_line() {
    let app = consumer("an_import_through_a_symlink_to_its_own_root", "../link");
    let mut config = load_config_at(&app, &app).expect("app loads");
    let error = expand_workspace_tree(&mut config)
        .err()
        .expect("the import names its own root");
    assert_eq!(
        format!("{error:#}"),
        "grund.toml:5: import `dep` resolves to the config root that declares it"
    );
}

/// §FS-imports.declaration.path: the edge carries the canonical root, so a link to
/// the target and the target itself are one tree (§FS-imports.graph.diamond).
#[test]
fn an_import_through_a_symlink_mounts_the_canonical_target() {
    let app = consumer(
        "an_import_through_a_symlink_mounts_the_canonical",
        "../alias",
    );
    let mut config = load_config_at(&app, &app).expect("app loads");
    expand_workspace_tree(&mut config).expect("the import is valid");
    let mounts = &config.run().workspace.mounts;
    assert_eq!(mounts.len(), 1);
    let lib = canonical_test_path(&app.join("../lib"));
    assert_eq!(
        (mounts[0].alias.as_str(), mounts[0].written.as_str()),
        ("dep", "../alias")
    );
    assert_eq!(mounts[0].root, lib, "the edge is the canonical target");
}
