//! Adapter from the resolved Markdown model to rule facts (§FS-rules.5.1,
//! §AR-rules.3). It parses no sentence and evaluates no constraint.

use super::RuleAnchor;
use super::facts::{Completeness, FactHeader, NodeKey, NodeMeta, RuleFacts, SiteKey, SiteMeta};
use crate::config::Config;
use crate::grammar::{render_id, section_display_name};
use crate::model::{Declaration, Findings, Id, is_stub_for_inline_decl};
use crate::resolver::{SectionHome, WorkspaceCheckTarget, section_home};
use std::collections::{BTreeMap, BTreeSet};

/// Adapt one standalone project. The producer-neutral schema uses the stable
/// project name as its selected scope (§AR-rules.3).
pub(crate) fn adapt_markdown(findings: &Findings, config: &Config, complete: bool) -> RuleFacts {
    let project = config
        .project_name
        .as_deref()
        .unwrap_or("local")
        .to_string();
    adapt_projects(&project, &[(project.as_str(), findings, config)], complete)
}

/// Adapt the complete resolved workspace while keeping the selected member's
/// subjects local. External declarations carry qualified kind/labels, so the
/// same engine handles local, pinned, and any-member object selectors without
/// learning resolver records (§FS-rules.2, §AR-rules.3).
pub(crate) fn adapt_workspace(
    selected: &str,
    workspace: &BTreeMap<String, WorkspaceCheckTarget<'_>>,
    complete: bool,
) -> RuleFacts {
    let projects = workspace
        .iter()
        .map(|(alias, target)| (alias.as_str(), target.findings, target.config))
        .collect::<Vec<_>>();
    adapt_projects(selected, &projects, complete)
}

fn adapt_projects(
    selected: &str,
    projects: &[(&str, &Findings, &Config)],
    complete: bool,
) -> RuleFacts {
    let mut facts = RuleFacts {
        header: FactHeader {
            schema: 1,
            project: selected.to_string(),
            producer: "markdown-v1".into(),
            completeness: if complete {
                Completeness::Complete
            } else {
                Completeness::Incomplete
            },
        },
        decl: Vec::new(),
        chapter: Vec::new(),
        contains: Vec::new(),
        cites: Vec::new(),
        site_in: Vec::new(),
        nodes: BTreeMap::new(),
        sites: BTreeMap::new(),
    };
    let mut declarations: BTreeMap<(String, Id), NodeKey> = BTreeMap::new();
    let mut chapters: BTreeMap<(String, Id, String), NodeKey> = BTreeMap::new();
    // The IDs whose home outside the walk a citation resolved into, its chapters minted.
    let mut minted_homes: BTreeSet<(String, Id)> = BTreeSet::new();

    for (alias, findings, config) in projects {
        let local = *alias == selected;
        for (id, homes) in &findings.declarations {
            let bare_label = render_id(&config.grammar, id);
            let label = if local {
                bare_label.clone()
            } else {
                format!("{alias}/{bare_label}")
            };
            // A healthy Markdown stub and its inline declaration are one catalog home.
            // Canonicalize before minting keys so relations share a node (§FS-rules.5.1,
            // §FS-list.2.5).
            for (ordinal, home) in homes
                .iter()
                .filter(|home| !is_stub_for_inline_decl(&config.root, home, homes))
                .enumerate()
            {
                let key = NodeKey(format!(
                    "{selected}:markdown:{alias}:decl:{bare_label}:{ordinal}"
                ));
                declarations
                    .entry((alias.to_string(), id.clone()))
                    .or_insert_with(|| key.clone());
                let kind = if local {
                    id.kind.clone()
                } else {
                    format!("{alias}/{}", id.kind)
                };
                facts.decl.push((key.clone(), kind));
                facts.nodes.insert(
                    key,
                    NodeMeta {
                        label: label.clone(),
                        anchor: RuleAnchor {
                            path: home.file.to_string_lossy().into_owned(),
                            line: home.line,
                            column: None,
                        },
                    },
                );
                for (section, info) in &home.sections {
                    if !is_rule_unit(section) {
                        continue;
                    }
                    let chapter = NodeKey(format!(
                        "{selected}:markdown:{alias}:chapter:{bare_label}:{section}:{ordinal}"
                    ));
                    chapters.insert(
                        (alias.to_string(), id.clone(), section.clone()),
                        chapter.clone(),
                    );
                    facts.chapter.push((
                        chapter.clone(),
                        section.clone(),
                        section_display_name(&info.title, section).to_string(),
                    ));
                    facts.nodes.insert(
                        chapter,
                        NodeMeta {
                            label: format!("{label}{}{section}", config.section_separator),
                            anchor: RuleAnchor {
                                path: home.file.to_string_lossy().into_owned(),
                                line: info.line,
                                column: None,
                            },
                        },
                    );
                }
            }
        }
    }

    for (alias, findings, _) in projects {
        for (ordinal, citation) in findings.citations.iter().enumerate() {
            let Some(source_id) = citation.enclosing_declaration.as_ref() else {
                continue;
            };
            let Some(declaration) = declarations
                .get(&(alias.to_string(), source_id.clone()))
                .cloned()
            else {
                continue;
            };
            let target_alias = citation.namespace.as_deref().unwrap_or(alias);
            let Some((_, target_findings, target_config)) = projects
                .iter()
                .find(|(candidate, _, _)| *candidate == target_alias)
            else {
                continue;
            };
            let Some(target_homes) = target_findings.declarations.get(&citation.id) else {
                continue;
            };
            let mut resolved = target_homes
                .iter()
                .filter(|home| !is_stub_for_inline_decl(&target_config.root, home, target_homes));
            let (Some(_), None) = (resolved.next(), resolved.next()) else {
                // Unknown and ambiguous targets retain their ordinary resolver
                // findings but never become logical edges (§FS-rules.5.1).
                continue;
            };
            let Some(target_declaration) = declarations
                .get(&(target_alias.to_string(), citation.id.clone()))
                .cloned()
            else {
                continue;
            };
            // §FS-rules.5.1: a resolved section that is no rule unit counts for its nearest
            // named ancestor chapter, or else its declaration. It resolves where `check`
            // finds it, a stub's in its target, scanned or not (§FS-check.3.2.1).
            let (target, newly_counted) = match citation.section.as_ref() {
                Some(section) => {
                    let Some(home) =
                        section_home(target_findings, target_config, &citation.id, section)
                    else {
                        continue;
                    };
                    // §FS-check.3.2.1: the lookup read the home on this miss; mint from that.
                    if let SectionHome::Unscanned(home) = home
                        && minted_homes.insert((target_alias.to_string(), citation.id.clone()))
                    {
                        mint_home_chapters(
                            &mut facts,
                            &mut chapters,
                            (target_alias, &citation.id, &target_declaration),
                            home,
                            &target_config.section_separator,
                        );
                    }
                    (
                        nearest_chapter(&chapters, target_alias, &citation.id, section)
                            .unwrap_or(target_declaration),
                        !is_rule_unit(section),
                    )
                }
                None => (target_declaration, false),
            };
            // The source side resolves the same way, agreeing with `site_in`.
            let immediate = citation
                .enclosing_section
                .as_deref()
                .and_then(|section| nearest_chapter(&chapters, alias, source_id, section))
                .unwrap_or_else(|| declaration.clone());
            let site = SiteKey(format!("{selected}:markdown:{alias}:site:{ordinal}"));
            facts.cites.push((site.clone(), immediate, target));
            facts.site_in.push((site.clone(), declaration));
            if let Some(section) = &citation.enclosing_section {
                let mut path = section.as_str();
                loop {
                    if let Some(chapter) =
                        chapters.get(&(alias.to_string(), source_id.clone(), path.to_string()))
                    {
                        facts.site_in.push((site.clone(), chapter.clone()));
                    }
                    let Some((parent, _)) = path.rsplit_once('.') else {
                        break;
                    };
                    path = parent;
                }
            }
            facts.sites.insert(
                site,
                SiteMeta {
                    label: citation.text.clone(),
                    anchor: RuleAnchor {
                        path: citation.file.to_string_lossy().into_owned(),
                        line: citation.line,
                        column: Some(citation.column),
                    },
                    newly_counted,
                },
            );
        }
    }

    // The relation is the chapter tree, not a flattened declaration-to-section
    // index. This keeps direct-chapter counts direct (§FS-rules.5.1). It is drawn
    // last, so it reaches the chapters a citation minted.
    for ((alias, id, path), chapter) in &chapters {
        let parent = path
            .rsplit_once('.')
            .and_then(|(parent, _)| chapters.get(&(alias.clone(), id.clone(), parent.to_string())))
            .cloned()
            .or_else(|| declarations.get(&(alias.clone(), id.clone())).cloned());
        if let Some(parent) = parent {
            facts.contains.push((parent, chapter.clone()));
        }
    }

    facts
}

/// The rule units of `home`, the declaration of `id` a stub stands for in a target
/// the walk did not reach, as nodes under that stub's declaration node `owner`.
/// The home answers a citation into it and nothing else (§FS-check.3.2.1): the
/// citation counts for the chapter it names, as on the scanned tree, and with no
/// `chapter` row no subject or count of chapters reaches the node (§AR-rules.3).
/// A stub encloses no citation, so minting them part-way through the citations
/// moves no source side already drawn.
fn mint_home_chapters(
    facts: &mut RuleFacts,
    chapters: &mut BTreeMap<(String, Id, String), NodeKey>,
    (alias, id, owner): (&str, &Id, &NodeKey),
    home: &Declaration,
    separator: &str,
) {
    let label = facts.nodes[owner].label.clone();
    for (section, info) in &home.sections {
        if !is_rule_unit(section) {
            continue;
        }
        let chapter = NodeKey(format!("{}:home:{section}", owner.0));
        chapters.insert(
            (alias.to_string(), id.clone(), section.clone()),
            chapter.clone(),
        );
        facts.nodes.insert(
            chapter,
            NodeMeta {
                label: format!("{label}{separator}{section}"),
                anchor: RuleAnchor {
                    path: home.file.to_string_lossy().into_owned(),
                    line: info.line,
                    column: None,
                },
            },
        );
    }
}

/// A section with an all-digit component is no rule unit (§FS-rules.2).
fn is_rule_unit(section: &str) -> bool {
    !section
        .split('.')
        .any(|part| part.bytes().all(|byte| byte.is_ascii_digit()))
}

/// The chapter at `path`, or at its nearest ancestor that is one: numbered
/// sections are no rule units, so they resolve upward (§FS-rules.5.1).
fn nearest_chapter(
    chapters: &BTreeMap<(String, Id, String), NodeKey>,
    alias: &str,
    id: &Id,
    path: &str,
) -> Option<NodeKey> {
    let mut path = path;
    loop {
        if let Some(chapter) = chapters.get(&(alias.to_string(), id.clone(), path.to_string())) {
            return Some(chapter.clone());
        }
        path = path.rsplit_once('.')?.0;
    }
}
