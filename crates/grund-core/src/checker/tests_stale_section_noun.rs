//! Test module: the stale-block finding's text and the noun read back out of it
//! (§FS-errors.3.6.1, §FS-check.3.5.3), the pair `grund config migrate`'s
//! report gate relies on (§FS-config-migrate.5.2).

use super::agents::{stale_block, stale_section_noun};
use std::path::Path;

/// Every noun §FS-check.3.5.3 lists, and a heading someone wrote into the block.
const NOUNS: [&str; 13] = [
    "citation scheme",
    "grounding from a citation",
    "project map",
    "project namespaces",
    "workspace members",
    "declarations and citations",
    "rules",
    "citation directions",
    "clickable citations",
    "declaration shapes",
    "chapter rules",
    "scoped guidance",
    "my notes",
];

/// §FS-errors.3.6.1: the text a stale `noun` is reported with, for the run's own
/// block (`owner` is `None`) or another project's. With no section bytes a
/// workspace section carries the fallback detail (§FS-check.3.5.3.1).
fn expected(noun: &str, owner: Option<&str>) -> String {
    let tail = "(does not affect citation validity)";
    if matches!(noun, "project namespaces" | "workspace members") {
        let dir = owner.map(|dir| format!(" in {dir}")).unwrap_or_default();
        format!(
            "repo maintenance: stale grund init block: the {noun} section differs from the \
             workspace (section text or order differs) — run `grund init`{dir} to refresh {tail}"
        )
    } else {
        let run_it = owner
            .map(|dir| format!("; run it in {dir}"))
            .unwrap_or_default();
        format!(
            "repo maintenance: stale grund init block: the {noun} section differs from \
             grund.toml — run `grund init` to refresh {tail}{run_it}"
        )
    }
}

// §FS-errors.3.6.1: both stale templates name the section as one subject, and
// the noun reads back out of either, with or without another project's directory.
#[test]
fn every_stale_noun_round_trips_through_the_section_differs_templates() {
    for owner in [None, Some("packages/billing")] {
        for noun in NOUNS {
            let finding = stale_block(Path::new("AGENTS.md"), 3, noun, owner, None);
            assert_eq!(finding.message, expected(noun, owner), "{noun} / {owner:?}");
            assert_eq!(
                stale_section_noun(&finding.message),
                Some(noun),
                "{noun} / {owner:?}"
            );
        }
    }
}

// §FS-errors.3.6.1: the other four templates are not stale findings and name no
// section.
#[test]
fn the_other_agents_init_templates_yield_no_noun() {
    for message in [
        "repo maintenance: malformed grund managed block: missing `<!-- END GRUND MANAGED BLOCK -->` (does not affect citation validity)",
        "repo maintenance: outdated grund init block v3 — run `grund init` to update to v43 (does not affect citation validity)",
        "repo maintenance: unsupported grund init block v999 — this grund supports v43 (does not affect citation validity)",
        "repo maintenance: missing grund init block v43 — run `grund init` to install it (does not affect citation validity); run it in packages/billing",
    ] {
        assert_eq!(stale_section_noun(message), None, "{message}");
    }
}
