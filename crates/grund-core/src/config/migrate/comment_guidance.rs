//! §FS-config-migrate.2.2: local reasons, actual mapping destinations and
//! preservation advice for comments that the carrying policy lists.

use super::keymap::joined;
use super::record::MappedKey;
use crate::config::v1::{LineKind, LineRecord};

/// §FS-config-migrate.2.2: guidance reads authored mappings, never a second
/// translation table. A refused or unloadable candidate has no promised places.
pub(super) struct Guidance<'a> {
    pub(super) lines: &'a [LineRecord],
    pub(super) mapped: &'a [MappedKey],
    pub(super) available: bool,
}

impl Guidance<'_> {
    /// §FS-config-migrate.2.2: a standalone comment has no attachment to follow.
    pub(super) fn standalone(&self) -> String {
        self.manual("no attached key or header".into())
    }

    /// §FS-config-migrate.2.2: preserve truthful dropped, omitted and collision
    /// reasons, and explain split/merged attachments using their key mappings.
    pub(super) fn attached(
        &self,
        record: &LineRecord,
        single: Option<String>,
        present: bool,
    ) -> String {
        if !self.available {
            return "destinations are unavailable; retain the prose, repair the settings or candidate loading problem above and preview again before choosing its placement".into();
        }
        if let Some(destination) = single {
            let reason = if present {
                format!("trailing comments collide at the shared destination {destination}")
            } else {
                format!("the destination key {destination} is omitted from the candidate")
            };
            return self.manual(reason);
        }
        let mappings: Vec<_> = match &record.kind {
            LineKind::Header { .. } => {
                let end = self
                    .lines
                    .iter()
                    .find(|line| {
                        line.line > record.line && matches!(line.kind, LineKind::Header { .. })
                    })
                    .map_or(usize::MAX, |line| line.line);
                self.mapped
                    .iter()
                    .filter(|mapped| mapped.line > record.line && mapped.line < end)
                    .collect()
            }
            _ => self
                .mapped
                .iter()
                .filter(|mapped| mapped.line == record.line)
                .collect(),
        };
        let mut destinations = Vec::new();
        for mapping in &mappings {
            // §FS-config-migrate.2.2: implicit invariants name no output place.
            if mapping.spelling.contains('[')
                && !mapping.spelling.starts_with("no [")
                && !destinations.contains(&mapping.spelling)
            {
                destinations.push(mapping.spelling.clone());
            }
        }
        if destinations.is_empty() {
            let dropped = match &record.kind {
                LineKind::Key { section, .. } => section == "output",
                LineKind::Header { name, .. } => name == "output",
                _ => false,
            };
            return self.manual(if dropped {
                "the setting or table is dropped; it has no version-2 config destination".into()
            } else {
                "the attachment has no written version-2 destination (its setting is implicit or omitted)".into()
            });
        }
        let merged = mappings.iter().any(|mapping| {
            self.mapped
                .iter()
                .any(|other| other.line != mapping.line && other.spelling == mapping.spelling)
        });
        let reason = if matches!(record.kind, LineKind::Header { .. }) {
            "header splits or merges its authored settings"
        } else if merged {
            "settings merge at a shared destination"
        } else {
            "setting splits or merges into version-2 settings"
        };
        format!(
            "{reason}: {}; retain the prose, authorize the listed drops with --write --drop-comments, then re-add it at one of these named destinations after a successful write (subject to all other refusals and gates)",
            joined(&destinations)
        )
    }

    /// §FS-config-migrate.2.2: no unique carrier requires a manual placement,
    /// explicit consent and a successful write, never a promise to clear gates.
    fn manual(&self, reason: String) -> String {
        if !self.available {
            return format!(
                "{reason}; destinations are unavailable; retain the prose, repair the settings or candidate loading problem above and preview again before choosing a manual placement"
            );
        }
        format!(
            "{reason}; retain the prose and choose its placement manually; authorize the listed drops with --write --drop-comments, then re-add it at that chosen place after a successful write (subject to all other refusals and gates)"
        )
    }
}
