//! Structural questions over scanner-owned section records, shared by check and
//! fetch (§FS-fetch.3.sections). These helpers read only the record; each caller
//! owns rule strength, scope and diagnostic presentation.

use std::collections::BTreeMap;

use super::{Declaration, SectionInfo, named_section_path};

impl Declaration {
    /// The expected depth when a section's heading mismatches its complete path
    /// (§FS-declarations.checks.section-heading-level). Relative headings already
    /// take their depth from their parent (§FS-config.3.3.3), and front-matter
    /// coordinates have no heading (§FS-fields.front-matter.coordinates).
    pub(crate) fn section_heading_mismatch(
        &self,
        path: &str,
        section: &SectionInfo,
    ) -> Option<usize> {
        if section.relative || section.in_front_matter() {
            return None;
        }
        let expected = self.heading_level + path.split('.').count();
        (section.heading_level != expected).then_some(expected)
    }

    /// Every collided canonical path and its sorted claimant lines
    /// (§FS-declarations.checks.duplicate-section), which check, fetch and show's
    /// ambiguity refusal each render themselves. The map holds the first heading
    /// and `duplicate_sections` the rest (§AR-scanner.2.2.3), so `lines[0]` is where
    /// a finding anchors; with no map entry the earliest recorded claimant does.
    pub(crate) fn section_collisions(&self) -> BTreeMap<&str, Vec<usize>> {
        let mut colliding: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
        for (path, info) in &self.duplicate_sections {
            colliding.entry(path.as_str()).or_default().push(info.line);
        }
        for (path, lines) in &mut colliding {
            lines.extend(self.sections.get(*path).map(|first| first.line));
            lines.sort_unstable();
        }
        colliding
    }

    /// The first proper prefix of `path` that this declaration's section map
    /// lacks, where `path` bears a name: what makes the section an orphan
    /// (§FS-declarations.checks.orphan-section). `None` for a numeric path, which
    /// needs no parent recorded, and for a path whose every prefix is. Check,
    /// fetch and relative-parent resolution all read the complete recorded set.
    pub(crate) fn missing_section_prefix(&self, path: &str) -> Option<String> {
        let parts = path.split('.').collect::<Vec<_>>();
        if !named_section_path(path) {
            return None;
        }
        (1..parts.len())
            .map(|end| parts[..end].join("."))
            .find(|prefix| !self.sections.contains_key(prefix))
    }
}
