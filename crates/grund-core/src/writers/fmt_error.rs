//! The two refusals `fmt` carries out of the engine as values: the
//! whole-declaration-set rewrite that found unreadable paths (§FS-fmt.3.2), and
//! the one that found a conflict block (§FS-fmt.7.4), each having written
//! nothing. They are `pub` error types of the embedding surface, so `grund-cli`
//! matches on them and adds its own `error:` prefix (§AR-core-module-layout.2).

use crate::model::Finding;
use crate::scanner::ApiScanError;

/// A whole-declaration-set rewrite refused because its completed scan found
/// unreadable paths (§FS-fmt.3.2). Kept structured so API callers can inspect the
/// complete set and each CLI adapter can add its own `error:` prefix.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct FmtScanAbort {
    pub scan_errors: Vec<ApiScanError>,
}

impl std::fmt::Display for FmtScanAbort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.scan_errors.is_empty() {
            return f.write_str("nothing was rewritten");
        }
        for (index, error) in self.scan_errors.iter().enumerate() {
            if index > 0 {
                f.write_str("\n")?;
            }
            write!(
                f,
                "nothing was rewritten: {}: {}",
                error.path, error.message
            )?;
        }
        Ok(())
    }
}

impl std::error::Error for FmtScanAbort {}

/// A whole-declaration-set rewrite refused because the project holds a conflict
/// block (§FS-fmt.7.4, §AR-scanner.2.9): `conflicts` is each
/// block's `conflict-marker` finding, the one `check` reports
/// (§FS-declarations.checks.conflict-marker), in root-then-members order in a
/// workspace, files then lines within a project (§FS-fmt.3.3). Its own type
/// beside [`FmtScanAbort`], which stays as it is, because a block is a finding
/// with a line and sites where an unreadable path is not.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct FmtConflictAbort {
    pub conflicts: Vec<Finding>,
}

impl std::fmt::Display for FmtConflictAbort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.conflicts.is_empty() {
            return f.write_str("nothing was rewritten");
        }
        for (index, conflict) in self.conflicts.iter().enumerate() {
            if index > 0 {
                f.write_str("\n")?;
            }
            let path = conflict.path.as_deref().unwrap_or_default();
            let line = conflict.line.unwrap_or_default();
            write!(
                f,
                "nothing was rewritten: {path}:{line}: {}",
                conflict.message
            )?;
        }
        Ok(())
    }
}

impl std::error::Error for FmtConflictAbort {}
