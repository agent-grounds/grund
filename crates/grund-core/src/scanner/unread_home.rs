//! The home `fetch` refuses because the scan would not read what it writes (§FS-fetch.4.2.1):
//! the one function behind `fetch`'s refusal and `check`'s tail at that home
//! (§FS-check.3.1.3.1). The name rules and the kind are the frame's; a folder home's `<ID>.md`
//! that passes them is then asked of the walk's own read predicate (§AR-scanner.1.13), which
//! reads the ignore files and the `exclude` rules exactly as the walk does.

use std::path::PathBuf;

use super::read_predicate::excluding_rule;
use super::walk::scan_roots;
use crate::config::{Frame, RefusedHome, Schema};

/// The home of `kind` that `fetch` refuses and why, `None` where it writes. `unclaimed` is the
/// local ID no file in a `folder` home declares, as the caller's scan says; `None` asks nothing
/// of a folder (§FS-fetch.5.5). No file is read: the rules are asked of the path, from the
/// config root and the roots the walk starts from, so a configured home outruns a rule as it
/// does in the walk (§AR-scanner.1.7).
pub(crate) fn fetch_refused_home(
    schema: &Schema,
    frame: Frame<'_>,
    kind: &str,
    unclaimed: Option<&str>,
) -> Option<(PathBuf, RefusedHome)> {
    if let Some(refused) = frame.fetch_refused_by_name(schema, frame.root(), kind, unclaimed) {
        return Some(refused);
    }
    let snapshot = frame.fetch_snapshot_path(schema, frame.root(), kind, unclaimed?)?;
    let (roots, _) = scan_roots(schema, frame, None, false).ok()?;
    let rule = excluding_rule(schema, frame, &roots, &snapshot)?;
    Some((snapshot, RefusedHome::Excluded(rule)))
}
