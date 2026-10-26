//! Shared missing-target repair guidance from recorded independent homes
//! (§FS-declarations.stubs.repair). No target is read or paired here.

use super::remedies;
use std::path::Path;

use super::{Declaration, PairedDeclarations, StubResolution, format_path};

/// The missing stubs and real homes of one ID, derived once for both located
/// and site-free explanations (§FS-declarations.stubs.repair).
pub(crate) struct MissingStubRepair<'a> {
    missing: Vec<(&'a Declaration, &'a Path)>,
    homes: Vec<&'a Declaration>,
}

impl<'a> MissingStubRepair<'a> {
    /// Read the scanner's verdicts (§AR-scanner.4.6) and existing pairing, including
    /// real targets represented by healthy stubs (§FS-declarations.stubs.repair).
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

    /// Compare the recorded declaration files as the scanner's OwnFile branch
    /// does, without reading a target (§FS-declarations.stubs.repair).
    fn sole_home_is_own_file(&self, stub: &Declaration) -> bool {
        matches!(self.homes.as_slice(), [home] if home.file == stub.file)
    }

    /// Append the same located facts to duplicate and query messages, leaving
    /// their original sites untouched (§FS-declarations.stubs.repair).
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
            [home] => {
                suffix.push_str(&format!("; {id} is declared at {home}"));
                if self
                    .missing
                    .iter()
                    .any(|(stub, _)| self.sole_home_is_own_file(stub))
                {
                    for (stub, _) in &self.missing {
                        let site = format!("{}:{}", display_path(&stub.file), stub.line);
                        if self.sole_home_is_own_file(stub) {
                            suffix.push_str(&format!(
                                "; stub {site} shares the real home's file; \
                                 linking to the same file will not pair it"
                            ));
                        } else {
                            suffix.push_str(&format!("; {}", remedies::duplicate_home(site)));
                        }
                    }
                } else {
                    suffix.push_str(&format!("; {}", remedies::duplicate_there()));
                }
            }
            _ => suffix.push_str(&format!(
                "; {id} has several real declaration homes: {}",
                homes.join(", ")
            )),
        }
        suffix
    }

    /// The located clause of one missing stub, `broken stub: …` with the homes and
    /// the repair after it, as [`Self::located_suffix`] writes it when that stub is
    /// the only missing one beside the same homes: the line `refs` warns with for a
    /// stub it set aside (§FS-refs.4.1.1). `None` for a stub whose target is not
    /// missing, which this guidance does not cover.
    pub(crate) fn stub_clause(
        &self,
        stub: &Declaration,
        id: &str,
        display_path: impl Fn(&Path) -> String,
    ) -> Option<String> {
        let missing: Vec<_> = self
            .missing
            .iter()
            .filter(|(missing, _)| std::ptr::eq(*missing, stub))
            .copied()
            .collect();
        if missing.is_empty() {
            return None;
        }
        let alone = Self {
            missing,
            homes: self.homes.clone(),
        };
        let suffix = alone.located_suffix(id, display_path);
        Some(suffix.strip_prefix("; ").unwrap_or(&suffix).to_string())
    }

    /// Connect a missing-target finding to its companion without naming another
    /// site or assuming selection retained it (§FS-declarations.stubs.repair).
    pub(crate) fn site_free_suffix(&self, id: &str, stub: &Declaration) -> String {
        match self.homes.len() {
            _ if self.missing.is_empty() => String::new(),
            0 => String::new(),
            1 if self.sole_home_is_own_file(stub) => format!(
                "; {id} has one real declaration home in this stub's own file; \
                 linking to the same file will not pair it \
                 (see the duplicate declaration finding for its location)"
            ),
            // §FS-errors.5.5.1: both versions give broken-stub's case alike.
            1 => format!(
                "; {id} has one real declaration home; {} \
                 (see the duplicate declaration finding for its location)",
                remedies::broken_stub_one_home(2)
            ),
            _ => format!(
                "; {id} has several real declaration homes \
                 (see the duplicate declaration finding for their locations)"
            ),
        }
    }
}
