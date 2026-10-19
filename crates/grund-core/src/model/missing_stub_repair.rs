//! Shared missing-target repair guidance from recorded independent homes
//! (§FS-declarations.stubs.verdict.repair). No target is read or paired here.

use std::path::Path;

use super::{Declaration, PairedDeclarations, StubResolution, format_path};

/// The missing stubs and real homes of one ID, derived once for both located
/// and site-free explanations (§FS-declarations.stubs.verdict.repair).
pub(crate) struct MissingStubRepair<'a> {
    missing: Vec<(&'a Declaration, &'a Path)>,
    homes: Vec<&'a Declaration>,
}

impl<'a> MissingStubRepair<'a> {
    /// Read the scanner's verdicts and existing pairing, including real targets
    /// represented by healthy stubs (§FS-declarations.stubs.verdict.repair).
    pub(crate) fn new(paired: &PairedDeclarations<'a>) -> Self {
        let mut missing = Vec::new();
        let mut homes = Vec::new();
        for declaration in paired.records() {
            if !declaration.is_stub {
                homes.push(declaration);
            } else if matches!(declaration.stub_resolution, Some(StubResolution::Missing))
                && let Some(target) = &declaration.defined_in
            {
                missing.push((declaration, target.as_path()));
            }
        }
        Self { missing, homes }
    }

    /// Append the same located facts to duplicate and query messages, leaving
    /// their original sites untouched (§FS-declarations.stubs.verdict.repair).
    pub(crate) fn located_suffix(
        &self,
        id: &str,
        display_path: impl Fn(&Path) -> String,
    ) -> String {
        if self.missing.is_empty() {
            return String::new();
        }
        let mut suffix = String::new();
        for (stub, target) in &self.missing {
            suffix.push_str(&format!(
                "; broken stub: {}:{} links to missing {}",
                display_path(&stub.file),
                stub.line,
                format_path(target)
            ));
        }
        let homes = self
            .homes
            .iter()
            .map(|home| format!("{}:{}", display_path(&home.file), home.line))
            .collect::<Vec<_>>();
        match homes.as_slice() {
            [] => {}
            [home] => suffix.push_str(&format!(
                "; {id} is declared at {home}; point the stub link there"
            )),
            _ => suffix.push_str(&format!(
                "; {id} has several real declaration homes: {}",
                homes.join(", ")
            )),
        }
        suffix
    }

    /// Connect a missing-target finding to its companion without naming another
    /// site or assuming selection retained it (§FS-declarations.stubs.verdict.repair).
    pub(crate) fn site_free_suffix(&self, id: &str) -> String {
        match self.homes.len() {
            _ if self.missing.is_empty() => String::new(),
            0 => String::new(),
            1 => format!(
                "; {id} has one real declaration home; point the stub link there \
                 (see the duplicate declaration finding for its location)"
            ),
            _ => format!(
                "; {id} has several real declaration homes \
                 (see the duplicate declaration finding for their locations)"
            ),
        }
    }
}
