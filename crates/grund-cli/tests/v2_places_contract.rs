//! §FS-config-v2.schema.places: several places per row, the complement and
//! ownership, run alone through the ordinary e2e fixture runner; each case also
//! runs in the full e2e suite. The `config-v1-*` case is the v1 half that must
//! not move (§FS-config.5.2).
#[path = "support/case_runner.rs"]
#[allow(dead_code)]
mod case_runner;

use case_runner::{CaseKind::E2e, assert_every_case_passed, run_case};

#[test]
fn v2_places_contract() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cases = root.join("tests/e2e/cases");
    let outcomes = [
        "config-v2-places-code-citable-refused",
        "config-v2-places-code-scan-false-refused",
        "config-v2-places-config-show",
        "config-v2-places-default-complement",
        "config-v2-places-duplicate-link-refused",
        "config-v2-places-duplicate-normalized-refused",
        "config-v2-places-duplicate-refused",
        "config-v2-places-files-and-folders",
        "config-v2-places-full-scanned-under-unscanned",
        "config-v2-places-full-under-scanned-non-citable",
        "config-v2-places-homeless-citable-kind",
        "config-v2-places-id-json",
        "config-v2-places-index-per-folder",
        "config-v2-places-inside-member-refused",
        "config-v2-places-link-out-refused",
        "config-v2-places-listed-file-overrides-folder",
        "config-v2-places-nested-deepest-wins",
        "config-v2-places-no-place-refused",
        "config-v2-places-outside-root-refused",
        "config-v2-places-stub-in-any-folder",
        "config-v2-places-two-folders",
        "config-v2-places-workspace-full",
        "config-v2-places-workspace-narrowed",
        "config-v1-places-nested-home-falls-to-complement",
        "schema-table-v2-code-places",
    ]
    .iter()
    .map(|name| run_case(&root, &cases.join(name), E2e))
    .collect::<Vec<_>>();
    assert_every_case_passed("v2 places", &outcomes);
}
