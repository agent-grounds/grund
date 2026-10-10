//! Test module: a stub's sections are read from its target outside the walk,
//! once per target per scan, through the input observation a watching run
//! subscribes from (§FS-check.3.2.1, §FS-check.6.1.1, §AR-scanner.4.6). The e2e
//! cases pin what `check` reports; these pin what the scan reads to report it.

use std::path::Path;
use std::sync::{Arc, Mutex};

use super::*;
use crate::config::{Config, load_config};
use crate::model::{Catalog, CheckInput, Id, normalize_path_lexically, with_check_input_observer};
use crate::testing::{scan_tree, test_root, write};

const CONFIG: &str = "grund_config_version = 1\n\n[reference]\nstrict = true\n\
require_grounding = false\n\n[id]\nformat = \"{kind}-{slug}\"\n\n[[kinds]]\n\
kind = \"FS\"\nfolder = \"docs\"\nindex = false\n\n[scan]\ninclude = [\"docs\"]\n";

/// Two declarations, each with a section `1`, in one file outside the scan.
const SOURCE: &str = concat!(
    "/// FS-second: Second\n///\n/// Second lead.\n///\n",
    "/// ## 1. Detail\n///\n/// Second detail.\npub fn second() {}\n\n",
    "/// FS-third: Third\n///\n/// Third lead.\n///\n",
    "/// ## 1. More\n///\n/// Third detail.\npub fn third() {}\n",
);

/// The same file with `FS-second`'s section gone.
const SOURCE_WITHOUT_SECTION: &str = concat!(
    "/// FS-second: Second\n///\n/// Second lead.\npub fn second() {}\n\n",
    "/// FS-third: Third\n///\n/// Third lead.\n///\n",
    "/// ## 1. More\n///\n/// Third detail.\npub fn third() {}\n",
);

fn id(slug: &str) -> Id {
    Id {
        kind: "FS".to_string(),
        num: None,
        slug: Some(slug.to_string()),
    }
}

/// A scanned `FS-first` with a section, and stubs of `FS-second` and `FS-third`
/// in `docs/` pointing at `source.rs`, which `[scan] include` leaves out.
fn stub_repo(name: &str) -> (Config, Catalog) {
    let root = test_root(name);
    write(&root.join("grund.toml"), CONFIG);
    write(
        &root.join("docs/first.md"),
        "# FS-first: First\n\nFirst lead.\n\n## 1. Here\n\nFirst detail.\n",
    );
    for slug in ["second", "third"] {
        write(
            &root.join(format!("docs/{slug}.md")),
            &format!("# FS-{slug}: [../source.rs](../source.rs)\n"),
        );
    }
    write(&root.join("source.rs"), SOURCE);
    rescan(&root)
}

fn rescan(root: &Path) -> (Config, Catalog) {
    let config = load_config(root).expect("load config");
    let (findings, _) = scan_tree(&config, None, false).expect("scan");
    (config, findings)
}

/// One scan of `root` under an observer that refuses `refused`'s read, and every
/// input it was asked to cover.
fn observed_scan(root: &Path, refused: Option<&Path>) -> (Catalog, Vec<CheckInput>) {
    let config = load_config(root).expect("load config");
    let seen: Arc<Mutex<Vec<CheckInput>>> = Arc::default();
    let sink = seen.clone();
    let refused = refused.map(Path::to_path_buf);
    let observer = Arc::new(move |input: CheckInput| {
        let covered = refused.as_ref() != Some(&input.path);
        sink.lock().expect("observer").push(input);
        covered
    });
    let (findings, _) = with_check_input_observer(Some(observer), || {
        scan_tree(&config, None, false).expect("scan")
    });
    let seen = seen.lock().expect("observer").clone();
    (findings, seen)
}

#[test]
fn a_target_is_read_once_per_scan() {
    let (config, _) = stub_repo("stub_home_read_once");
    let source = normalize_path_lexically(&config.root.join("source.rs"));
    let (findings, seen) = observed_scan(&config.root, None);
    let resolves = |slug: &str, section: &str| section_resolves(&findings, &id(slug), section);

    assert_eq!(
        seen.iter().filter(|input| input.path == source).count(),
        3,
        "§FS-check.6.1.1: each of the two stubs covers its target as it resolves it, \
         and the scan reads the target once, however many stubs and IDs name it"
    );
    assert!(resolves("first", "1"), "a recorded section resolves");
    assert!(
        resolves("second", "1"),
        "§FS-check.3.2.1: the target's section"
    );
    assert!(resolves("third", "1"), "another ID in the same target");
    assert!(
        !resolves("second", "2"),
        "a section the target does not declare"
    );

    write(&config.root.join("source.rs"), SOURCE_WITHOUT_SECTION);
    assert!(
        resolves("second", "1"),
        "this scan answers from the text it read"
    );
    let (_, findings) = rescan(&config.root);
    assert!(
        !section_resolves(&findings, &id("second"), "1"),
        "the next scan, as `--watch` makes one, reads the target again"
    );
}

#[test]
fn the_target_read_is_observed_before_it_is_made() {
    let observe = |refuse: bool| {
        let name = format!("stub_home_observed_{refuse}");
        let (config, _) = stub_repo(&name);
        let source = normalize_path_lexically(&config.root.join("source.rs"));
        let (findings, seen) = observed_scan(&config.root, refuse.then_some(source.as_path()));
        let resolved = section_resolves(&findings, &id("second"), "1");
        let observed = seen.iter().any(|input| input.path == source);
        (resolved, observed)
    };

    assert_eq!(
        observe(false),
        (true, true),
        "§FS-check.6.1.1: a watching run subscribes to the target it read"
    );
    assert_eq!(
        observe(true),
        (false, true),
        "the read goes through the observation: one it refuses never happens"
    );
}
