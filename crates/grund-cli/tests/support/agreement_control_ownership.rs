//! The real no-config control owns an uncovered root (§AR-ci.10.2, §AR-ci.10.3).

use super::*;
use crate::agreement_tree::fixture_checks::{lifetimes, occupied};

fn uncovered_tree() -> Tree {
    let tree = no_config_tree();
    for ancestor in tree.0.ancestors() {
        for name in ["grund.toml", ".agents/grund.toml"] {
            assert!(
                !ancestor.join(name).is_file(),
                "configured fixture: {}",
                ancestor.join(name).display()
            );
        }
    }
    tree
}

#[test]
fn grund_977_no_config_lifetimes() {
    lifetimes(uncovered_tree, |tree| &tree.0);
}

#[test]
fn grund_977_no_config_occupied_root() {
    let candidate = std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .join(format!("grund-no-config-control-{}", std::process::id()));
    occupied(candidate, uncovered_tree, |tree| &tree.0);
}
