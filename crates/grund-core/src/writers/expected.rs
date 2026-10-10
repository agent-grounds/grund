//! `Expected`, rendered (§AR-checker.1.3, §AR-system.2.11): what presentation
//! decided, as the bytes the checker compares with disk. The managed-block
//! sections are rendered through the same renderers `grund init` writes with —
//! citation directions, clickable citations per conversation surface, and the
//! chapter-rule section — so the command that clears a drift finding and the
//! check that raises it ask one component for one answer from opposite
//! directions (§AR-checker.2.7). The chapter-rule section is rendered from the
//! check run's own catalog, so building it adds no walk (§FS-init.2.3.4.15).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::checker::{configured_rule_sentences, workspace_vocabulary};
use crate::config::Config;
use crate::grammar::parse_id_arg;
use crate::model::{Catalog, Expected, ExpectedEntrypoint, ExpectedSection};
use crate::resolver::{WorkspaceCheckTarget, index_link_targets, markdown_link_target};
use crate::scanner::{CANONICAL_AGENT_ENTRYPOINT, companion_agent_entrypoints};
use crate::templates::{
    ConversationSurface, citation_directions_section, clickable_citations_section,
};

/// Render `Expected` for one project's check (§AR-checker.1.3).
///
/// `findings` is the run's resolution scope (§FS-check.1.3.6.1), never its
/// report scope, so a path holding no rule declaration still renders every
/// chapter-rule bullet and a narrowed run compares against the render
/// `grund check .` compares against (§FS-rules.9.1.1). `workspace` is the project
/// map the run loaded, which is what lets a run at the workspace root resolve a
/// member's cross-boundary rule and so catch a member block missing its bullet
/// (§FS-rules.9.1).
///
/// §FS-check.3.5.4: the chapter-rule section is rendered whether or not every
/// configured rule resolved. A rule this scope cannot verify still earns its
/// bullet (§FS-rules.4.1.2), so the section is left out only for a genuinely
/// invalid rule — the one case that already carries its own located error at
/// the rule's heading.
pub(crate) fn expected(
    findings: &Catalog,
    config: &Config,
    workspace: &BTreeMap<String, WorkspaceCheckTarget<'_>>,
) -> Expected {
    let rule_rows = config
        .kinds
        .iter()
        .any(|kind| kind.rules)
        .then(|| {
            let vocab = workspace_vocabulary(config, workspace);
            configured_rule_sentences(findings, config, &vocab).ok()
        })
        .flatten()
        .map(|rules| rules.rows);
    // §FS-init.2.3.5.10: render destinations from the same loaded declarations
    // that supplied the validated, ordered rule sentences.
    let rule_guidance = rule_rows.as_deref().map(|rows| (findings, rows));
    let root = &config.root;
    let canonical = root.join(CANONICAL_AGENT_ENTRYPOINT);
    let canonical_exists = canonical.exists();
    let mut entrypoints = Vec::new();
    if canonical_exists {
        entrypoints.push(expected_entrypoint(config, canonical, true, rule_guidance));
    }
    let mut entrypoint_probe_error = None;
    match companion_agent_entrypoints(root) {
        Ok(companions) => entrypoints.extend(companions.into_iter().map(|companion| {
            expected_entrypoint(config, companion, canonical_exists, rule_guidance)
        })),
        Err(error) => entrypoint_probe_error = Some(error),
    }
    Expected {
        block_version: block_version(config),
        entrypoints,
        entrypoint_probe_error,
        index_targets: index_link_targets(&config.project().presentation, config, findings),
    }
}

/// The managed-block version this config asks for: v15 once a kind declares
/// rules, v14 otherwise (§FS-init.2.3.7).
pub(crate) fn block_version(config: &Config) -> u32 {
    if config.kinds.iter().any(|kind| kind.rules) {
        15
    } else {
        14
    }
}

/// One entrypoint's config-derived sections, rendered for its own surface
/// (§FS-init.2.3.6.1): the local-conversation sentence derives from
/// `[reference] conversation` and varies by entrypoint (§FS-init.2.3.4.17.2), so
/// each file is rendered for the surface `init` chose for it.
pub(crate) fn expected_entrypoint(
    config: &Config,
    path: PathBuf,
    require_block: bool,
    rule_guidance: Option<(&Catalog, &[(String, String)])>,
) -> ExpectedEntrypoint {
    let mut sections = vec![
        ExpectedSection {
            heading: "### Citation directions",
            noun: "citation directions",
            bytes: citation_directions_section(config.project(), config.run()),
        },
        ExpectedSection {
            heading: "### Clickable citations",
            noun: "clickable citations",
            bytes: clickable_citations_section(
                config.project(),
                ConversationSurface::for_entrypoint(&path),
            ),
        },
    ];
    if let Some((findings, rows)) = rule_guidance {
        sections.push(ExpectedSection {
            heading: "### Chapter rules",
            noun: "chapter rules",
            bytes: chapter_rules_section(config, &path, findings, rows),
        });
    }
    ExpectedEntrypoint {
        path,
        require_block,
        sections,
    }
}

/// The shared chapter-rule section for `init` and the drift comparison's
/// `Expected` (§FS-init.2.3.5.10). Preserve ordered authored titles and use the
/// formatter's destination/anchor resolver for live citations relative to this
/// entrypoint. Only the citation is rendered; no formatter runs over the
/// authored sentence.
pub(crate) fn chapter_rules_section(
    config: &Config,
    path: &Path,
    findings: &Catalog,
    rows: &[(String, String)],
) -> String {
    let mut section = String::from(
        "### Chapter rules\n\n`must`/`must not` are `grund check` errors; `should`/`should not` are suggestions (`grund check --suggestions`).\n\n",
    );
    for (origin, sentence) in rows {
        let citation = format!("{}{origin}", config.marker);
        let target = config
            .fmt_cross_refs_enabled
            .then(|| {
                let (id, _) = parse_id_arg(origin, &config.grammar).ok()?;
                markdown_link_target(
                    path,
                    &id,
                    None,
                    &config.project().presentation,
                    config,
                    findings,
                )
            })
            .flatten();
        let citation = match target {
            Some(target) => format!("[{citation}]({target})"),
            None => citation,
        };
        section.push_str(&format!("- {sentence} {citation}\n"));
    }
    section
}
