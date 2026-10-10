//! Test module: what `compile` derives for the scanner (§AR-config.6.1) — the
//! rows whose files record grounding structure, each found through the level
//! rule the checker cuts units with.

use super::*;
use crate::testing::{test_root, write};

/// The demand of the config `body` loads from a fresh tree named `name`.
fn demand(name: &str, body: &str) -> ScanDemand {
    let root = test_root(name);
    write(&root.join("grund.toml"), body);
    let config = load_config(&root).expect("the config loads");
    config.compiled().demand.clone()
}

/// §AR-config.6.1: a level-1 tree — every configuration written before the
/// keys existed — names no row, so the scanner's one-field exit excuses it.
#[test]
fn a_level_one_tree_demands_no_structure() {
    let demand = demand(
        "a_level_one_tree_demands_no_structure",
        "grund_config_version = 1\n\n[reference]\nrequire_grounding = true\n",
    );
    assert!(demand.is_empty());
    for row in ["FS", "AR", "code"] {
        assert!(!demand.records_structure(row), "{row} records nothing");
    }
}

/// §AR-config.6.1: a row at level 2 is the one row whose files record
/// structure; a row that inherits level 1 records nothing.
#[test]
fn a_level_two_row_demands_structure_for_itself_alone() {
    let demand = demand(
        "a_level_two_row_demands_structure_for_itself_alone",
        "grund_config_version = 1\n\n\
             [reference]\nrequire_grounding = true\n\n\
             [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\n\n\
             [[kinds]]\nkind = \"runbook\"\nfolder = \"runbooks\"\ncitable = false\n\
             grounding_level = 2\n",
    );
    assert!(!demand.is_empty());
    assert!(demand.records_structure("runbook"));
    assert!(!demand.records_structure("FS"));
    assert!(!demand.records_structure("code"));
}

/// §AR-config.6.1: the complement is a row like any other — named by the row
/// that declares it, or `code` with the `[reference]` default where none does.
#[test]
fn the_complement_demands_structure_under_its_own_name() {
    let declared = demand(
        "the_complement_demands_structure_under_its_own_name_declared",
        "grund_config_version = 1\n\n\
             [reference]\nrequire_grounding = true\n\n\
             [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\n\n\
             [[kinds]]\nkind = \"src\"\ncitable = false\ngrounding_level = 2\n",
    );
    assert!(declared.records_structure("src"));
    assert!(!declared.records_structure("FS"));
    assert!(!declared.records_structure("code"));

    let homeless = demand(
        "the_complement_demands_structure_under_its_own_name_homeless",
        "grund_config_version = 1\n\n\
             [reference]\nrequire_grounding = true\ngrounding_level = 2\n\n\
             [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\n",
    );
    assert!(
        homeless.records_structure("code"),
        "the undeclared complement"
    );
    assert!(
        homeless.records_structure("FS"),
        "the default reaches every row"
    );
}
