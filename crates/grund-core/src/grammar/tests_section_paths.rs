//! Test module: what *beneath* means for a section coordinate (§FS-refs.2).
//! The e2e cases pin what `grund refs --descendants` prints on a tree; these
//! pin the relation underneath, where a case per boundary would be a fixture
//! tree per boundary. The boundaries are the point: this predicate exists
//! because a string prefix answers the question wrong, and quietly — `3.1`
//! swallowing `3.10` through `3.19` is an eightfold error that sized a
//! migration before the flag existed to ask properly.

use super::*;

/// §FS-refs.2: the requested coordinate is in its own subtree. A `--descendants`
/// query is a widening, never a shift, so the exact-coordinate sites a bare
/// `--section` lists are all still in the answer.
#[test]
fn a_coordinate_is_its_own_descendant() {
    assert!(path_at_or_under("1.1", "1.1", "."));
    assert!(path_at_or_under(
        "checks.duplicate",
        "checks.duplicate",
        "."
    ));
    assert!(!path_at_or_under("1", "1.1", "."));
}

/// §FS-refs.2, §FS-config.3.3.4: the numeric boundary, and the whole reason the
/// relation is written down rather than spelled as a `starts_with`. `1.1`
/// reaches `1.1.2` and never the sibling `1.10`, whose text begins with the
/// same three characters.
#[test]
fn a_numeric_sibling_that_shares_a_prefix_is_not_a_descendant() {
    assert!(path_at_or_under("1.1.2", "1.1", "."));
    assert!(!path_at_or_under("1.10", "1.1", "."));
    assert!(!path_at_or_under("1.10.3", "1.1", "."));
    assert!(!path_at_or_under("3.160", "3.16", "."));
}

/// §FS-refs.2: the same boundary on the named grammar, where a component is a
/// handle rather than a number and a hyphen is an ordinary character inside
/// one. `checks.duplicate` reaches `checks.duplicate.1` and never the sibling
/// `checks.duplicate-section`.
#[test]
fn a_named_sibling_that_shares_a_prefix_is_not_a_descendant() {
    assert!(path_at_or_under(
        "checks.duplicate.1",
        "checks.duplicate",
        "."
    ));
    assert!(!path_at_or_under(
        "checks.duplicate-section",
        "checks.duplicate",
        "."
    ));
    assert!(!path_at_or_under(
        "checks.duplicate-section.1",
        "checks.duplicate",
        "."
    ));
    // The parent chapter is above the requested coordinate, not beneath it.
    assert!(!path_at_or_under("checks", "checks.duplicate", "."));
}

/// §FS-refs.2: the relation is the same one at every depth — it counts
/// components, so nothing about it is bounded by how far down the tree goes.
#[test]
fn the_relation_holds_at_arbitrary_depth() {
    assert!(path_at_or_under("1.1.1.1.1", "1", "."));
    assert!(path_at_or_under("1.1.1.1.1", "1.1.1.1", "."));
    assert!(path_at_or_under(
        "checks.duplicate.1.2.3",
        "checks.duplicate",
        "."
    ));
    assert!(!path_at_or_under("1.1.1.1.1", "2", "."));
}

/// §FS-config.3.3.4: the separator is the caller's, because a coordinate space
/// changes boundary exactly once. A bare declaration enters its section tree
/// through the configured outer `section_separator` — `:` here — while a path
/// already inside that tree continues through `.` at every depth. This is the
/// seam `citation_under_title` chooses for the LSP and `refs` never meets,
/// dealing only in recorded section paths.
#[test]
fn the_component_separator_is_the_callers() {
    // A bare ID under `section_separator = ":"`.
    assert!(path_at_or_under("FS-check:3", "FS-check", ":"));
    assert!(!path_at_or_under("FS-check.3", "FS-check", ":"));
    // An ID that is already carrying a section: its children continue in `.`.
    assert!(path_at_or_under("FS-check:3.1", "FS-check:3", "."));
    assert!(!path_at_or_under("FS-check:30", "FS-check:3", "."));
}
