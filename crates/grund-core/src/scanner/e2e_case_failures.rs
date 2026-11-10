//! The E2E case pass of a run that reads no declaration set (§FS-fmt.7.2): `fmt` reads the
//! cases through `check`'s own reader rather than a copy of it (§AR-system.principle).

use std::path::Path;

use super::e2e::scan_e2e_cases;
use super::tree::ScanError;
use super::walk_failures::sort_failures;
use super::walk_reporting::WalkedTree;

use crate::config::{Frame, Schema, scope_is_config_root};
use crate::model::{Catalog, FailureNames};

/// The walk's failures with those of the E2E case pass beside them, for a run that read no
/// declaration set and so met no case (§FS-fmt.7.2, §FS-declarations.e2e-case.failure): `fmt`
/// names each bad case file `check` names on the same scope. The cases are read as `check` reads
/// them, by [`scan_e2e_cases`] over what the walk met with `check`'s resolution scope
/// (§FS-check.1.3.6.1), and `narrow` keeps those the report scope holds, as `check` keeps them.
/// One failure both passes meet is one line (§AR-scanner.1.relisted), and the list is sorted once
/// more for both (§FS-errors.4.1).
pub(crate) fn with_e2e_case_failures(
    schema: &Schema,
    frame: Frame<'_>,
    scope: Option<&Path>,
    explicit_scope: bool,
    walked: &mut WalkedTree,
    narrow: impl FnOnce(Vec<ScanError>, &FailureNames) -> Vec<ScanError>,
) -> Vec<ScanError> {
    let mut reported = std::mem::take(&mut walked.errors);
    let walk_reported = reported.len();
    let mut findings = Catalog {
        walked_dirs: std::mem::take(&mut walked.dirs),
        failure_names: walked.failure_names.clone(),
        ..Catalog::default()
    };
    // §FS-check.1.3.6.1: `check`'s `resolution_wide`, a path below the root.
    let below_root = explicit_scope
        && !scope_is_config_root(
            frame.root(),
            scope.unwrap_or(Path::new(".")),
            explicit_scope,
        );
    scan_e2e_cases(
        schema,
        frame,
        scope,
        explicit_scope,
        below_root,
        &walked.e2e_cases,
        &mut reported,
        &mut findings,
    );
    let cases = reported.split_off(walk_reported);
    reported.extend(narrow(cases, &findings.failure_names));
    sort_failures(&mut reported);
    reported
}
