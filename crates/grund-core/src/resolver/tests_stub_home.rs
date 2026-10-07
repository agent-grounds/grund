//! Test module: a stub's sections are read from its target outside the walk on a
//! miss, once per target per run, through the input observation a watching run
//! subscribes from (§FS-check.3.2.1, §FS-check.6.1.1). The e2e cases pin what
//! `check` reports; these pin what it reads to report it.

use std::path::Path;
use std::sync::{Arc, Mutex};

use super::*;
use crate::config::{Config, load_config};
use crate::model::{Catalog, CheckInput, Id, normalize_path_lexically, with_check_input_observer};
use crate::scanner::scan_tree;
use crate::testing::{test_root, write};

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

#[test]
fn a_target_is_read_on_a_miss_and_once_per_run() {
    let (config, findings) = stub_repo("stub_home_read_once");
    let resolves =
        |slug: &str, section: &str| section_resolves(&findings, &config, &id(slug), section);
    let reads = || findings.stub_targets.read_count();

    assert!(resolves("first", "1"), "a recorded section resolves");
    assert_eq!(
        reads(),
        0,
        "a section a recorded declaration holds reads nothing"
    );
    assert!(
        resolves("second", "1"),
        "§FS-check.3.2.1: the target's section"
    );
    assert_eq!(reads(), 1, "the miss reads the stub's target");
    assert!(resolves("third", "1"), "another ID in the same target");
    assert!(
        !resolves("second", "2"),
        "a section the target does not declare"
    );
    assert_eq!(
        reads(),
        1,
        "one read per target, however many IDs and sections ask"
    );

    write(&config.root.join("source.rs"), SOURCE_WITHOUT_SECTION);
    assert!(
        resolves("second", "1"),
        "this run answers from the text it read"
    );
    let (config, findings) = rescan(&config.root);
    assert!(
        !section_resolves(&findings, &config, &id("second"), "1"),
        "the next run, as `--watch` makes one, reads the target again"
    );
}

#[test]
fn the_target_read_is_observed_before_it_is_made() {
    let observe = |refuse: bool| {
        let name = format!("stub_home_observed_{refuse}");
        let (config, findings) = stub_repo(&name);
        let source = normalize_path_lexically(&config.root.join("source.rs"));
        let seen: Arc<Mutex<Vec<CheckInput>>> = Arc::default();
        let sink = seen.clone();
        let target = source.clone();
        let observer = Arc::new(move |input: CheckInput| {
            let covered = !(refuse && input.path == target);
            sink.lock().expect("observer").push(input);
            covered
        });
        let resolved = with_check_input_observer(Some(observer), || {
            section_resolves(&findings, &config, &id("second"), "1")
        });
        let observed = seen
            .lock()
            .expect("observer")
            .iter()
            .any(|input| input.path == source);
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
