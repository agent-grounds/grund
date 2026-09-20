//! Adapter from the resolved Markdown model to rule facts (§FS-rules.5.1,
//! §AR-rules.3). It parses no sentence and evaluates no constraint.

use super::RuleAnchor;
use super::facts::{Completeness, FactHeader, NodeKey, NodeMeta, RuleFacts, SiteKey, SiteMeta};
use crate::config::Config;
use crate::grammar::render_id;
use crate::model::{Findings, is_stub_for_inline_decl};
use std::collections::BTreeMap;

pub(crate) fn adapt_markdown(findings: &Findings, config: &Config, complete: bool) -> RuleFacts {
    let project = config
        .project_name
        .clone()
        .unwrap_or_else(|| "local".into());
    let mut facts = RuleFacts {
        header: FactHeader {
            schema: 1,
            project: project.clone(),
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
    let mut declarations = BTreeMap::new();
    let mut chapters = BTreeMap::new();
    for (id, homes) in &findings.declarations {
        let label = render_id(&config.grammar, id);
        for (ordinal, home) in homes.iter().enumerate() {
            let key = NodeKey(format!("{project}:markdown:decl:{label}:{ordinal}"));
            declarations
                .entry(id.clone())
                .or_insert_with(|| key.clone());
            facts.decl.push((key.clone(), id.kind.clone()));
            facts.nodes.insert(
                key.clone(),
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
                    "{project}:markdown:chapter:{label}:{section}:{ordinal}"
                ));
                chapters.insert((id.clone(), section.clone()), chapter.clone());
                facts
                    .chapter
                    .push((chapter.clone(), section.clone(), info.title.clone()));
                facts.nodes.insert(
                    chapter,
                    NodeMeta {
                        label: format!("{label}.{section}"),
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
    // `contains` is the chapter tree, not a flattened declaration-to-section
    // index. This lets direct-chapter counts stay direct while `site_in` below
    // can name every ancestor unit (§FS-rules.5.1, §AR-rules.3).
    for ((id, path), chapter) in &chapters {
        let parent = path
            .rsplit_once('.')
            .and_then(|(parent, _)| chapters.get(&(id.clone(), parent.to_string())))
            .cloned()
            .or_else(|| declarations.get(id).cloned());
        if let Some(parent) = parent {
            facts.contains.push((parent, chapter.clone()));
        }
    }
    for (ordinal, citation) in findings.citations.iter().enumerate() {
        let Some(source_id) = citation.enclosing_declaration.as_ref() else {
            continue;
        };
        let Some(target_homes) = findings.declarations.get(&citation.id) else {
            continue;
        };
        if target_homes
            .iter()
            .filter(|home| !is_stub_for_inline_decl(&config.root, home, target_homes))
            .count()
            != 1
        {
            // Unknown and ambiguous targets retain their ordinary resolver
            // findings but never become logical edges (§FS-rules.5.1).
            continue;
        }
        let Some(target_declaration) = declarations.get(&citation.id).cloned() else {
            continue;
        };
        let target = citation
            .section
            .as_ref()
            .and_then(|section| chapters.get(&(citation.id.clone(), section.clone())))
            .cloned()
            .unwrap_or(target_declaration);
        let Some(declaration) = declarations.get(source_id).cloned() else {
            continue;
        };
        let immediate = citation
            .enclosing_section
            .as_ref()
            .and_then(|section| chapters.get(&(source_id.clone(), section.clone())))
            .cloned()
            .unwrap_or_else(|| declaration.clone());
        let site = SiteKey(format!("{project}:markdown:site:{ordinal}"));
        facts.cites.push((site.clone(), immediate.clone(), target));
        facts.site_in.push((site.clone(), declaration.clone()));
        if let Some(section) = &citation.enclosing_section {
            let mut path = section.as_str();
            loop {
                if let Some(chapter) = chapters.get(&(source_id.clone(), path.to_string())) {
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
    facts
}
