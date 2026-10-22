//! `Expected`, rendered (§AR-checker.1.3, §AR-system.2.11): what presentation
//! decided, as the bytes the checker compares with disk. The managed-block
//! sections are rendered through the same renderers `grund init` writes with —
//! citation directions, clickable citations per conversation surface, and the
//! chapter-rule section — so the command that clears a drift finding and the
//! check that raises it ask one component for one answer from opposite
//! directions (§AR-checker.2.7). The chapter-rule section is rendered from the
//! check run's own catalog, so building it adds no walk (§FS-init.2.3.4.15).

use crate::sources::path_exists;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::checker::{configured_rule_sentences, workspace_vocabulary};
use crate::config::Config;
use crate::grammar::AGENTS_BLOCK_BASE_VERSION;
use crate::grammar::parse_id_arg;
use crate::model::{Catalog, Expected, ExpectedEntrypoint, ScopedExpectation};
use crate::resolver::{
    SettledHeadings, WorkspaceCheckTarget, index_link_targets, markdown_link_target,
};
use crate::scanner::{CANONICAL_AGENT_ENTRYPOINT, companion_agent_entrypoints};

use super::guidance::{Guidance, RuleGuidance};
use super::scoped_files::{
    OrphanFile, OwnedCompanion, ScopedRecord, orphan_files, owned_companions, scoped_companions,
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
    let rules = config
        .kinds
        .iter()
        .any(|kind| kind.rules)
        .then(|| {
            let vocab =
                workspace_vocabulary(config.schema(), config.rules(), config.frame(), workspace);
            configured_rule_sentences(findings, config.schema(), config.frame(), &vocab).ok()
        })
        .flatten();
    // §FS-init.2.3.5.10: render destinations from the same loaded declarations
    // that supplied the validated, ordered rule sentences.
    let guidance = Guidance::new(
        config,
        rules.as_ref().map(|rules| RuleGuidance {
            findings,
            rows: &rules.rows,
            subjects: &rules.subjects,
        }),
    );
    let root = &config.root;
    let canonical = root.join(CANONICAL_AGENT_ENTRYPOINT);
    let canonical_exists = path_exists(&canonical);
    let mut entrypoints = Vec::new();
    if canonical_exists {
        entrypoints.push(guidance.root_entrypoint(canonical, true));
    }
    let mut entrypoint_probe_error = None;
    match companion_agent_entrypoints(root, config.frame().sources()) {
        Ok(companions) => entrypoints.extend(
            companions
                .into_iter()
                .map(|companion| guidance.root_entrypoint(companion, canonical_exists)),
        ),
        Err(error) => entrypoint_probe_error = Some(error),
    }
    let non_units = scoped_entrypoints(config, &guidance, &mut entrypoints);
    Expected {
        block_version: guidance.version,
        entrypoints,
        entrypoint_probe_error,
        index_targets: index_link_targets(
            &config.project().presentation,
            config.schema(),
            config.frame(),
            findings,
            &SettledHeadings::default(),
        ),
        non_units,
    }
}

/// The scoped entrypoints `check` compares (§FS-check.3.5): each destination's
/// `AGENTS.md` and the standalone companions beside it, then the files of each
/// orphan the list on disk names that lose their block (§FS-init.2.3.5.14.3),
/// all only where a root entrypoint already carries a block. Returns every
/// scoped file and companion no row's `files` lists, which are guidance about a
/// directory's units and never one of them (§FS-init.2.3.5.11).
fn scoped_entrypoints(
    config: &Config,
    guidance: &Guidance<'_>,
    entrypoints: &mut Vec<ExpectedEntrypoint>,
) -> BTreeSet<PathBuf> {
    let root = &config.root;
    let record = ScopedRecord::read(root, config.frame().sources());
    let live: Vec<&str> = guidance.layout.scoped_dirs().collect();
    let orphans = record.orphans(&live);
    let mut files = BTreeSet::new();
    for (dir, orphan) in live
        .iter()
        .map(|dir| (*dir, false))
        .chain(orphans.iter().map(|dir| (*dir, true)))
    {
        let agents = root.join(dir).join(CANONICAL_AGENT_ENTRYPOINT);
        let companions = scoped_companions(&root.join(dir));
        files.insert(agents.clone());
        files.extend(companions);
        if !record.managed {
            continue;
        }
        if orphan {
            for (name, what) in orphan_files(root, dir) {
                if what != OrphanFile::Linked {
                    entrypoints.push(ExpectedEntrypoint {
                        path: root.join(dir).join(name),
                        require_block: false,
                        sections: Vec::new(),
                        scoped: Some(ScopedExpectation::Orphan),
                    });
                }
            }
        } else {
            entrypoints.push(guidance.scoped_entrypoint(dir, agents, true));
            for (name, owned) in owned_companions(root, dir) {
                if owned == OwnedCompanion::Standalone {
                    let path = root.join(dir).join(name);
                    entrypoints.push(guidance.scoped_entrypoint(dir, path, true));
                }
            }
        }
    }
    let claimed: BTreeSet<PathBuf> = config
        .schema()
        .rows
        .iter()
        .flat_map(|row| row.files())
        .map(|file| root.join(file))
        .collect();
    files.retain(|path| !claimed.contains(path));
    files
}

/// The managed-block version this config asks for without scoped guidance:
/// the base revision plus one once a kind declares rules (§FS-init.2.3.7.1);
/// `Guidance` adds two for a scoped file or a declaration shape.
pub(crate) fn block_version(config: &Config) -> u32 {
    AGENTS_BLOCK_BASE_VERSION + u32::from(config.kinds.iter().any(|kind| kind.rules))
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
                    config.frame(),
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
