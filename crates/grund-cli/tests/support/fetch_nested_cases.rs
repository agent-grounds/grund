//! The nested-declaration fetch refusals (§FS-fetch.3.1), using the normal E2E golden
//! runner. The full pass skips them through `case_runner::RED_UNTIL_1155` until #1155
//! is implemented; implement deletes that list and this test's `#[ignore]`.

use std::path::PathBuf;

use super::case_runner;

/// §FS-fetch.3.1: each nested-declaration shape is refused with exit 2, the nested ID
/// named and the tree byte-identical.
#[test]
#[ignore = "red until #1155 refuses a nested declaration in fetch output; implement removes this"]
fn e2e_fetch_nested_declaration_refusals() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cases = root.join("tests/e2e/cases");
    let outcomes = case_runner::RED_UNTIL_1155
        .iter()
        .map(|name| case_runner::run_case(&root, &cases.join(name), case_runner::CaseKind::E2e))
        .collect::<Vec<_>>();
    case_runner::assert_every_case_passed("fetch nested declaration", &outcomes);
}
