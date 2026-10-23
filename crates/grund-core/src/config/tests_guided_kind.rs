//! Test module: the row init's guidance names (§FS-init.2.2.2.3), which the
//! `next:` block allocates in and the managed block's discovery rung leads its
//! `--kind` example with (§FS-init.2.3.4.3).

use crate::config::{Config, guided_kind, load_config};
use crate::testing::{test_root, write};

fn load(name: &str, kinds: &str) -> Config {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        &format!("grund_config_version = 1\n\n{kinds}"),
    );
    load_config(&root).unwrap_or_else(|err| panic!("{name}: {err:#}"))
}

fn guided(config: &Config) -> Option<&str> {
    guided_kind(&config.kinds).map(|row| row.kind.as_str())
}

/// §FS-init.2.2.2.3: FS with a home wins wherever it sits in config order.
#[test]
fn fs_with_a_home_is_guided_first() {
    let config = load(
        "guided_kind_fs_second",
        "[[kinds]]\nkind = \"REQ\"\nfolder = \"docs/req\"\n\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\n",
    );
    assert_eq!(guided(&config), Some("FS"));
}

/// §FS-init.2.2.2.3: a homeless FS gives way to the first citable row with a
/// home, and a non-citable row never qualifies.
#[test]
fn a_homeless_fs_gives_way_to_a_homed_row() {
    let config = load(
        "guided_kind_homeless_fs",
        "[[kinds]]\nkind = \"skill\"\nfolder = \"skills\"\ncitable = false\n\n\
         [[kinds]]\nkind = \"FS\"\n\n\
         [[kinds]]\nkind = \"REQ\"\nfolder = \"docs/req\"\n",
    );
    assert_eq!(guided(&config), Some("REQ"));
}

/// §FS-init.2.2.2.3: with no home anywhere, the first citable row; with no
/// citable row, nothing.
#[test]
fn homeless_and_empty_configs() {
    let homeless = load(
        "guided_kind_all_homeless",
        "[[kinds]]\nkind = \"skill\"\nfolder = \"skills\"\ncitable = false\n\n\
         [[kinds]]\nkind = \"REQ\"\n\n[[kinds]]\nkind = \"FS\"\n",
    );
    assert_eq!(guided(&homeless), Some("REQ"));
    let empty = load("guided_kind_empty", "kinds = []\n");
    assert_eq!(guided(&empty), None);
}
