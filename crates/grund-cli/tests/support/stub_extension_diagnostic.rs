//! The extension-refused stub's explanation and controls
//! (§FS-declarations.stubs.target.3), using the normal E2E golden runner.

use std::path::PathBuf;

use super::case_runner;

fn assert_cases(names: &[&str]) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let outcomes = names
        .iter()
        .map(|name| {
            case_runner::run_case(
                &root,
                &root.join("tests/e2e/cases").join(name),
                case_runner::CaseKind::E2e,
            )
        })
        .collect::<Vec<_>>();
    case_runner::assert_every_case_passed("stub extension diagnostic", &outcomes);
}

/// §FS-declarations.stubs.target.3: a real C doc-comment declares AR-c, but h
/// is absent from the default extensions. Only the explanation changes: text
/// and JSON retain an error at the stub line and exit 1, inside and outside include.
#[test]
fn an_unlisted_extension_explains_why_the_target_is_not_scanned() {
    assert_cases(&[
        "check-stub-extension-refused",
        "check-stub-extension-refused-json",
        "check-stub-extension-refused-outside-include",
        "stub-unsupported-extension",
        "check-stub-unlisted-extension-symlink-target-duplicate",
    ]);
}

/// §FS-declarations.stubs.target.3: listing h reads the very same declaration,
/// including through a stub outside include, and the broken-stub error goes away.
#[test]
fn listing_the_extension_makes_the_stub_healthy() {
    assert_cases(&[
        "check-stub-extension-listed",
        "check-stub-extension-listed-outside-include",
    ]);
}

/// §FS-declarations.stubs.target.3: a read target lacking AR-c keeps the lacks-ID
/// explanation. The extension refusal must never borrow that message.
#[test]
fn a_read_target_lacking_the_id_keeps_its_diagnostic() {
    assert_cases(&["check-stub-extension-listed-lacks-id"]);
}
