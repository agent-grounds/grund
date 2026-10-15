//! Test module: which homes hold a path, the one list both "the one kind whose
//! home holds it" and "is it in any home" are read from (§AR-scanner.4.7,
//! §FS-declarations.stubs.shape).

use super::context::{file_home_kind, in_some_home};
use crate::config::{Config, load_config};
use crate::testing::{test_root, write};

/// One home `spec/fs` for FS, a nested citable home `spec/fs/ar` for AR, a
/// non-citable place `notes`, a file home `GOALS.md`, and the complement `code`.
fn project() -> Config {
    let root = test_root("context_homes_holding");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"spec/fs\"\nindex = false\n\n\
         [[kinds]]\nkind = \"AR\"\nfolder = \"spec/fs/ar\"\nindex = false\n\n\
         [[kinds]]\nkind = \"notes\"\nfolder = \"notes\"\ncitable = false\n\n\
         [[kinds]]\nkind = \"GOAL\"\nfile = \"GOALS.md\"\n\n\
         [[kinds]]\nkind = \"code\"\ncitable = false\n",
    );
    load_config(&root).expect("load config")
}

fn answers(config: &Config, relative: &str) -> (Option<String>, bool) {
    let path = config.frame().root().join(relative);
    (
        file_home_kind(&path, config.schema(), config.frame()),
        in_some_home(&path, config.schema(), config.frame()),
    )
}

#[test]
fn one_home_is_the_one_kind_and_a_home() {
    let config = project();
    assert_eq!(
        answers(&config, "spec/fs/x.md"),
        (Some("FS".to_string()), true)
    );
}

#[test]
fn nested_homes_are_two_homes_so_no_one_kind_but_still_a_home() {
    let config = project();
    assert_eq!(answers(&config, "spec/fs/ar/x.md"), (None, true));
}

#[test]
fn a_non_citable_place_is_a_home() {
    let config = project();
    assert_eq!(
        answers(&config, "notes/x.md"),
        (Some("notes".to_string()), true)
    );
}

#[test]
fn a_file_home_holds_its_file_and_nothing_beside_it() {
    let config = project();
    assert_eq!(
        answers(&config, "GOALS.md"),
        (Some("GOAL".to_string()), true)
    );
    assert_eq!(answers(&config, "GOALS-old.md"), (None, false));
}

#[test]
fn the_complement_place_is_no_home() {
    let config = project();
    assert_eq!(answers(&config, "docs/notes/y.md"), (None, false));
    assert_eq!(answers(&config, "src/x.rs"), (None, false));
}
