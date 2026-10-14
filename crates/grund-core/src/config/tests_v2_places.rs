//! Test module: a v2 row's several places and the one ownership rule every
//! per-file pass asks (§FS-config-v2.schema.places.ownership) — the deepest
//! folder wins, a listed file beats its folder, v1 keeps its nested rule, a
//! place is refused when it repeats another (§FS-config-v2.schema.places.boundaries),
//! `scan = false` answers nesting both ways, each folder keeps its index, and a
//! stub may sit in any folder of its kind.

use std::path::Path;

use super::*;
use crate::testing::{scan_tree, test_root, write};

const HEAD: &str = "grund_config_version = 2\nproject_name = \"example\"\n[schema.sources]\nlanguages = [\"markdown\"]\n";

fn load(name: &str, toml: &str) -> anyhow::Result<Config> {
    let root = test_root(name);
    write(&root.join("grund.toml"), toml);
    load_config(&root)
}

/// The refusal `load` raised, in full.
fn refusal(result: anyhow::Result<Config>) -> String {
    match result {
        Ok(_) => panic!("the file should be refused"),
        Err(err) => format!("{err:#}"),
    }
}

fn loaded(name: &str, toml: &str) -> Config {
    load(name, toml).unwrap_or_else(|err| panic!("{name}: the file should load: {err:#}"))
}

/// The name of the row that owns `path`, `None` for the complement.
fn owner<'a>(config: &'a Config, path: &str) -> Option<&'a str> {
    let owner = config.project().schema.owner(Path::new(path));
    owner.map(|owner| owner.row.name.as_str())
}

/// §FS-config-v2.schema.places.ownership: the deepest folder holding a file owns
/// it, across rows and within one; a deeper `code` folder hands it to the complement.
#[test]
fn the_deepest_folder_owns_a_file() {
    let toml = format!(
        "{HEAD}[schema.kinds.FS]\nfolders = [\"docs/spec\", \"docs/spec/plugins\"]\n\
         [schema.kinds.page]\ncitable = false\nfolders = [\"docs\"]\n\
         [schema.kinds.code]\nfolders = [\"docs/spec/generated\"]\n"
    );
    let config = loaded("places_deepest", &toml);
    assert_eq!(owner(&config, "docs/guide.md"), Some("page"));
    assert_eq!(owner(&config, "docs/spec/login.md"), Some("FS"));
    assert_eq!(owner(&config, "docs/spec/plugins/a.md"), Some("FS"));
    let deepest = config
        .project()
        .schema
        .owner(Path::new("docs/spec/plugins/a.md"));
    assert_eq!(
        deepest.and_then(|owner| owner.place.path()),
        Some("docs/spec/plugins"),
        "of one row's two nested folders, the deeper one owns the file"
    );
    assert_eq!(owner(&config, "docs/spec/generated/x.md"), None);
    assert_eq!(owner(&config, "src/lib.md"), None, "a file in no place");
}

/// §FS-config-v2.schema.places.ownership: a file listed in a row's `files`
/// belongs to that row, however deep a folder holding it sits.
#[test]
fn a_listed_file_beats_every_folder() {
    let toml = format!(
        "{HEAD}[schema.kinds.GOAL]\nfiles = [\"docs/spec/goals.md\"]\n\
         [schema.kinds.FS]\nfolders = [\"docs/spec\"]\n\
         [schema.kinds.page]\ncitable = false\nfolders = [\"docs\"]\nfiles = [\"README.md\"]\n"
    );
    let config = loaded("places_listed_file", &toml);
    assert_eq!(owner(&config, "docs/spec/goals.md"), Some("GOAL"));
    assert_eq!(owner(&config, "docs/spec/other.md"), Some("FS"));
    assert_eq!(owner(&config, "README.md"), Some("page"));
}

/// §FS-config.3.9.2: v1 keeps its rule — a file in two nested homes falls to the
/// complement — while the same rows in v2 give it to the deeper place.
#[test]
fn v1_keeps_a_nested_file_on_the_complement() {
    let v1 = "grund_config_version = 1\n\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs/spec\"\n\
              index = false\n\n[[kinds]]\nkind = \"page\"\ncitable = false\nfolder = \"docs\"\n";
    let config = loaded("places_v1_nested", v1);
    assert_eq!(owner(&config, "docs/spec/login.md"), None);
    assert_eq!(owner(&config, "docs/guide.md"), Some("page"));

    let v2 = format!(
        "{HEAD}[schema.kinds.FS]\nfolders = [\"docs/spec\"]\nindex = false\n\
         [schema.kinds.page]\ncitable = false\nfolders = [\"docs\"]\n"
    );
    let config = loaded("places_v2_nested", &v2);
    assert_eq!(owner(&config, "docs/spec/login.md"), Some("FS"));
}

/// §FS-config-v2.schema.places.boundaries.1-3: two spellings of one place, and
/// two places linking to one target, are one place; a case-only difference is
/// one place exactly where the filesystem folds case.
#[test]
fn a_place_repeated_by_spelling_case_or_link_is_refused() {
    let toml = format!("{HEAD}[schema.kinds.FS]\nfolders = [\"docs\", \"./docs/\"]\n");
    let err = refusal(load("places_dup_spelling", &toml));
    assert!(
        err.contains("grund.toml:6: place `./docs/` in `[schema.kinds.FS] folders` is already a place of `[schema.kinds.FS]` (line 6)"),
        "{err}"
    );

    let toml = format!(
        "{HEAD}[schema.kinds.FS]\nfolders = [\"docs\"]\n\
         [schema.kinds.page]\ncitable = false\nfolders = [\"Docs\"]\n"
    );
    let root = test_root("places_dup_case");
    write(&root.join("docs/a.md"), "# A\n");
    write(&root.join("grund.toml"), &toml);
    let folds_case = root.join("Docs").exists();
    match load_config(&root) {
        Ok(_) => assert!(
            !folds_case,
            "a case-folding filesystem resolves both to one target"
        ),
        Err(err) => {
            assert!(
                folds_case,
                "a case-sensitive filesystem keeps two places: {err:#}"
            );
            assert!(format!("{err:#}").contains("is already a place of `[schema.kinds.FS]`"));
        }
    }

    #[cfg(unix)]
    {
        let toml = format!(
            "{HEAD}[schema.kinds.FS]\nfolders = [\"docs\"]\n\
             [schema.kinds.page]\ncitable = false\nfolders = [\"alias\"]\n"
        );
        let root = test_root("places_dup_link");
        write(&root.join("docs/a.md"), "# A\n");
        crate::testing::symlink("docs", &root.join("alias"));
        write(&root.join("grund.toml"), &toml);
        let err = refusal(load_config(&root));
        assert!(
            err.contains("grund.toml:8: place `alias` in `[schema.kinds.page] folders` is already a place of `[schema.kinds.FS]` (line 6)"),
            "{err}"
        );
    }
}

/// The IDs the walk declared, as `KIND-slug`.
fn declared(config: &Config) -> Vec<String> {
    let (catalog, _) = scan_tree(config, None, false).expect("scan");
    catalog
        .declarations
        .keys()
        .map(|id| format!("{}-{}", id.kind, id.slug.as_deref().unwrap_or("")))
        .collect()
}

/// §FS-config-v2.schema.places.boundaries.6: a scanned place under an unscanned
/// one is walked, and an unscanned place under a scanned one is pruned.
#[test]
fn scan_false_answers_nesting_both_ways() {
    let toml = format!(
        "{HEAD}[schema.kinds.FS]\nfolders = [\"archive/live\", \"docs\"]\nindex = false\n\
         [schema.kinds.archive]\ncitable = false\nscan = false\nfolders = [\"archive\"]\n\
         [schema.kinds.vendor]\ncitable = false\nscan = false\nfolders = [\"docs/vendor\"]\n"
    );
    let root = test_root("places_scan_false_nesting");
    write(&root.join("grund.toml"), &toml);
    write(&root.join("archive/live/a.md"), "# FS-live: Live\n");
    write(&root.join("archive/old.md"), "# FS-old: Old\n");
    write(&root.join("docs/b.md"), "# FS-doc: Doc\n");
    write(&root.join("docs/vendor/v.md"), "# FS-vendored: Vendored\n");
    let config = load_config(&root).unwrap_or_else(|err| panic!("{err:#}"));
    assert_eq!(declared(&config), ["FS-doc", "FS-live"]);
}

/// §FS-config-v2.schema.places.ownership: each folder of a citable row keeps its
/// own index, listing what that folder owns — never what a nested place owns.
#[test]
fn each_folder_keeps_its_own_index() {
    let toml = format!(
        "{HEAD}[schema.kinds.FS]\nfolders = [\"docs/a\", \"docs/b\"]\nindex = \"INDEX.md\"\n\
         [schema.kinds.AR]\nfolders = [\"docs/a/arch\"]\n\
         [schema.kinds.GOAL]\nfiles = [\"docs/goals.md\"]\n"
    );
    let config = loaded("places_index_per_folder", &toml);
    let schema = &config.project().schema;
    let fs = schema.rows.iter().find(|row| row.name == "FS").expect("FS");
    assert_eq!(
        fs.index_paths(),
        [
            ("docs/a", Path::new("docs/a/INDEX.md").to_path_buf()),
            ("docs/b", Path::new("docs/b/INDEX.md").to_path_buf()),
        ]
    );
    let goal = schema
        .rows
        .iter()
        .find(|row| row.name == "GOAL")
        .expect("GOAL");
    assert!(
        goal.index_paths().is_empty(),
        "a `files` place is not indexed"
    );
    assert!(schema.folder_owns("docs/a", Path::new("docs/a/x.md")));
    assert!(!schema.folder_owns("docs/a", Path::new("docs/b/y.md")));
    assert!(!schema.folder_owns("docs/a", Path::new("docs/a/arch/z.md")));
    assert!(schema.folder_owns("docs/a/arch", Path::new("docs/a/arch/z.md")));
}

/// §FS-declarations.stubs: a stub may sit in any folder of its kind, and a file
/// only the complement holds is no home.
#[test]
fn a_stub_may_sit_in_any_folder_of_its_kind() {
    let toml = format!(
        "{HEAD}[schema.kinds.FS]\nfolders = [\"docs/a\", \"docs/b\"]\n\
         [schema.kinds.code]\nfolders = [\"src\", \"docs\"]\n"
    );
    let config = loaded("places_stub_any_folder", &toml);
    let schema = &config.project().schema;
    assert!(schema.in_some_home(Path::new("docs/a/x.md")));
    assert!(schema.in_some_home(Path::new("docs/b/x.md")));
    assert!(!schema.in_some_home(Path::new("docs/x.md")));
    assert!(!schema.in_some_home(Path::new("src/x.md")));
}
