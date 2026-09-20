//! Adapter from the resolved Markdown model to rule facts (§FS-rules.5.1,
//! §AR-rules.3). It parses no sentence and evaluates no constraint.

use super::RuleAnchor;
use super::facts::{Completeness, FactHeader, NodeKey, NodeMeta, RuleFacts, SiteKey, SiteMeta};
use crate::config::Config;
use crate::grammar::{render_id, section_display_name};
use crate::model::{Findings, Id, is_stub_for_inline_decl};
use crate::resolver::WorkspaceCheckTarget;
use std::collections::BTreeMap;

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

    for (alias, findings, config) in projects {
        let local = *alias == selected;
        for (id, homes) in &findings.declarations {
            let bare_label = render_id(&config.grammar, id);
            let label = if local {
                bare_label.clone()
            } else {
                format!("{alias}/{bare_label}")
            };
            // A healthy Markdown stub and the inline declaration it names are
            // one catalog home. Canonicalize before minting opaque keys so every
            // downstream relation refers to the same node (§FS-rules.5.1,
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
                    if section
                        .split('.')
                        .any(|part| part.bytes().all(|byte| byte.is_ascii_digit()))
                    {
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

    // The relation is the chapter tree, not a flattened declaration-to-section
    // index. This keeps direct-chapter counts direct (§FS-rules.5.1).
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
            if target_homes
                .iter()
                .filter(|home| !is_stub_for_inline_decl(&target_config.root, home, target_homes))
                .count()
                != 1
            {
                // Unknown and ambiguous targets retain their ordinary resolver
                // findings but never become logical edges (§FS-rules.5.1).
                continue;
            }
            let Some(target_declaration) = declarations
                .get(&(target_alias.to_string(), citation.id.clone()))
                .cloned()
            else {
                continue;
            };
            let target = match citation.section.as_ref() {
                Some(section) => {
                    let Some(chapter) = chapters
                        .get(&(
                            target_alias.to_string(),
                            citation.id.clone(),
                            section.clone(),
                        ))
                        .cloned()
                    else {
                        // An authored but unresolved coordinate remains an
                        // ordinary resolver diagnostic and contributes no edge
                        // (§FS-rules.5.1).
                        continue;
                    };
                    chapter
                }
                None => target_declaration,
            };
            let immediate = citation
                .enclosing_section
                .as_ref()
                .and_then(|section| {
                    chapters.get(&(alias.to_string(), source_id.clone(), section.clone()))
                })
                .cloned()
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
                },
            );
        }
    }
    facts
}
