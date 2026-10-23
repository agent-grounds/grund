//! Ordinary agreement fixtures own even empty trees (§AR-ci.10.1, §AR-ci.10.2).

use super::fixture_checks::{legacy_candidate, lifetimes, occupied};
use super::*;

#[test]
fn grund_977_general_tree_lifetimes() {
    lifetimes(|| Tree::new("977-general-lifetimes", &[]), |tree| &tree.0);
}

#[test]
fn grund_977_general_tree_occupied_root() {
    let name = "977-general-occupied";
    occupied(
        legacy_candidate(name),
        || Tree::new(name, &[]),
        |tree| &tree.0,
    );
}
