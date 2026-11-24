//! §FS-config-migrate.5.3: the guidance comparison. The root `AGENTS.md` block
//! `grund init` would write is rendered in memory for the file as read and for
//! the candidate, and a difference in the block's version or in one of the
//! five generated sections is a refusal. Every other difference passes: it
//! describes the config, and `grund check` reports it until `grund init` runs.
//! The grounding setting the citation directions name is compared in one
//! grammar (§FS-config-migrate.5.3).

use crate::config::migrate::{Located, MigrateRefusal, RefusalAnchor};
use crate::config::{Invocation, Selection, config_at, with_config_text};
use crate::writers::{ComparedGuidance, RootGuidance, root_guidance};

const KEEP: &str = "none keeps the generated guidance; keep version 1 until that";

/// The noun of the one generated section whose config grammar is compared in
/// one grammar (§FS-config-migrate.5.3), which the report gate reads back
/// from a stale finding (§FS-config-migrate.5.2).
pub(super) const DIRECTIONS: &str = "citation directions";

/// What the guidance gate found: its refusals, and whether it passed the root
/// citation directions section only because the grounding setting is compared
/// in one grammar (§FS-config-migrate.5.3), which the report gate then drops
/// the stale finding for (§FS-config-migrate.5.2).
#[derive(Debug)]
pub(super) struct GuidanceGate {
    pub(super) refusals: Vec<MigrateRefusal>,
    pub(super) directions_in_one_grammar: bool,
}

/// The guidance gate of the candidate against the file, or `None` when either
/// config does not load, which the report gate then says. The file as read is
/// rendered with the grounding setting in the candidate's grammar
/// (§FS-config-migrate.5.3), so the refusal's `- ` lines show what is
/// compared rather than a spelling the comparison set aside.
pub(super) fn guidance_gate(
    located: &Located,
    candidate: &str,
    value_lines: &[(String, usize)],
) -> Option<GuidanceGate> {
    let render = |grammar: Option<u32>| {
        let invocation = Invocation::new(&located.root, false, Selection::Project);
        config_at(&located.root, &located.root, None, &invocation)
            .ok()
            .map(|config| root_guidance(config, grammar))
    };
    let after = with_config_text(&located.file, candidate, || render(None))?;
    let before = render(Some(after.grammar))?;
    Some(GuidanceGate {
        refusals: refusals(&before.guidance, &after.guidance, value_lines),
        directions_in_one_grammar: spelled_apart(&before, &after),
    })
}

/// §FS-config-migrate.5.3: whether the root citation directions differ as
/// `init` writes them and are equal once the setting is spelled in one grammar.
pub(super) fn spelled_apart(before: &ComparedGuidance, after: &ComparedGuidance) -> bool {
    let compared = |side: &ComparedGuidance| {
        side.guidance
            .sections
            .iter()
            .find(|section| section.noun == DIRECTIONS)
            .map(|section| section.bytes.trim_end().to_string())
    };
    before.live_directions.trim_end() != after.live_directions.trim_end()
        && compared(before) == compared(after)
}

/// Each compared difference between `before` and `after` as a refusal.
pub(super) fn refusals(
    before: &RootGuidance,
    after: &RootGuidance,
    value_lines: &[(String, usize)],
) -> Vec<MigrateRefusal> {
    let mut out = Vec::new();
    for (old, new) in before.sections.iter().zip(&after.sections) {
        if old.bytes.trim_end() == new.bytes.trim_end() {
            continue;
        }
        let shapes = if old.heading == "### Declaration shapes" {
            value_shapes(&new.bytes, value_lines)
        } else {
            Vec::new()
        };
        if shapes.is_empty() {
            out.push(section_refusal(&old.noun, &old.bytes, &new.bytes));
        } else {
            out.extend(shapes);
        }
    }
    if out.is_empty() && before.version != after.version {
        out.push(MigrateRefusal {
            anchor: RefusalAnchor::Unset,
            what: format!(
                "the AGENTS.md block grund init writes would go from v{} to v{}, which grund check compares",
                before.version, after.version
            ),
            details: Vec::new(),
            repair: format!("{KEEP} block may change"),
        });
    }
    out
}

/// One refusal per value row the candidate's `### Declaration shapes` lists,
/// at its `values` line, with the bullets it would write.
fn value_shapes(section: &str, value_lines: &[(String, usize)]) -> Vec<MigrateRefusal> {
    value_lines
        .iter()
        .filter_map(|(row, line)| {
            let head = format!("- **{row}**");
            let bullets: Vec<&str> = section
                .lines()
                .skip_while(|text| *text != head)
                .enumerate()
                .take_while(|(at, text)| *at == 0 || text.starts_with("  - "))
                .map(|(_, text)| text)
                .collect();
            (!bullets.is_empty()).then(|| MigrateRefusal {
                anchor: RefusalAnchor::Line(*line),
                what: format!(
                    "[[kinds]] `{row}` values = true adds a \"Declaration shapes\" section to the AGENTS.md block in version 2, which grund check compares"
                ),
                details: bullets
                    .iter()
                    .map(|text| format!("would write: {text}"))
                    .collect(),
                // §FS-config-migrate.3.1.7: the repair says what removal costs and the rewrite.
                repair: "removing values alone ends the comparison of every binding that quoted its values; to keep them compared, under version 1 rewrite its roots as marked sections, remove values and cite the new paths".into(),
            })
        })
        .collect()
}

/// A differing section other than a value row's shape, line by line.
fn section_refusal(noun: &str, before: &str, after: &str) -> MigrateRefusal {
    let old: Vec<&str> = before.lines().collect();
    let new: Vec<&str> = after.lines().collect();
    let mut details: Vec<String> = old
        .iter()
        .filter(|line| !new.contains(line))
        .map(|line| format!("- {line}"))
        .collect();
    details.extend(
        new.iter()
            .filter(|line| !old.contains(line))
            .map(|line| format!("+ {line}")),
    );
    MigrateRefusal {
        anchor: RefusalAnchor::Unset,
        what: format!(
            "the AGENTS.md block grund init writes would change its {noun} section, which grund check compares"
        ),
        details,
        repair: format!("{KEEP} section may change"),
    }
}
