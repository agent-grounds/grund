//! Compare managed block sections without discarding drift (§FS-check.3.5.4),
//! retaining their texts for workspace entry detail (§FS-check.3.5.3.1).

use crate::grammar::{WORKSPACE_SECTIONS, block_sections};
use crate::model::ExpectedSection;

/// A stale section retains both texts for §FS-check.3.5.3.1's entry detail.
/// An absent side has no text and therefore no body entries.
pub(super) struct StaleSection<'a> {
    pub(super) noun: String,
    pub(super) expected: &'a str,
    pub(super) current: &'a str,
}

/// The sections of the root block `current` that differ from
/// `expected`, in the render's order (§FS-check.3.5.3, §FS-check.3.5.4): one
/// expected and absent, one present and unexpected, one whose bytes differ, and,
/// where the sections agree but their order does not, the first one out of
/// place. Where the run does not compare the workspace sections both sides lose
/// them first (§FS-check.3.5.5).
/// The retained texts explain each workspace difference (§FS-check.3.5.3.1).
pub(super) fn stale_sections<'a>(
    expected: &'a [ExpectedSection],
    current: &'a str,
    workspace_sections: bool,
) -> Vec<StaleSection<'a>> {
    let compared = |noun: &str| workspace_sections || !WORKSPACE_SECTIONS.contains(&noun);
    let expected: Vec<&ExpectedSection> = expected
        .iter()
        .filter(|section| compared(&section.noun))
        .collect();
    let on_disk: Vec<(String, &str)> = block_sections(current)
        .iter()
        .map(|section| (section.noun(), section.bytes))
        .filter(|(noun, _)| compared(noun))
        .collect();
    let mut stale: Vec<StaleSection<'a>> = Vec::new();
    for section in &expected {
        let found = on_disk.iter().find(|(noun, _)| *noun == section.noun);
        if found.is_none_or(|(_, bytes)| *bytes != section.bytes) {
            stale.push(StaleSection {
                noun: section.noun.clone(),
                expected: &section.bytes,
                current: found.map_or("", |(_, bytes)| *bytes),
            });
        }
    }
    for (noun, bytes) in &on_disk {
        if !expected.iter().any(|section| section.noun == *noun)
            && !stale.iter().any(|section| section.noun == *noun)
        {
            stale.push(StaleSection {
                noun: noun.clone(),
                expected: "",
                current: bytes,
            });
        }
    }
    if stale.is_empty()
        && let Some((section, _)) = expected
            .iter()
            .zip(&on_disk)
            .find(|(section, (noun, _))| section.noun != *noun)
    {
        stale.push(StaleSection {
            noun: section.noun.clone(),
            expected: &section.bytes,
            current: on_disk
                .iter()
                .find(|(noun, _)| *noun == section.noun)
                .map_or("", |(_, bytes)| *bytes),
        });
    }
    stale
}

/// §FS-check.3.5.3.1: compare nonblank body lines verbatim and positionally,
/// leaving byte drift with equal entries to the section-level fallback.
pub(super) fn workspace_entry_detail(expected: &str, current: &str) -> String {
    let mut expected = workspace_entries(expected);
    let mut current = workspace_entries(current);
    loop {
        let pair = (expected.next(), current.next());
        match pair {
            (None, None) => return "section text or order differs".into(),
            (expected, current) if expected != current => {
                return format!(
                    "expected {}; found {}",
                    expected.unwrap_or("<missing>"),
                    current.unwrap_or("<missing>")
                );
            }
            _ => {}
        }
    }
}

/// §FS-check.3.5.3.1: omit the heading and blank lines without changing entries.
fn workspace_entries(section: &str) -> impl Iterator<Item = &str> {
    section
        .lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
}
