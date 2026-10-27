//! Adapter from the resolved Markdown model to rule facts (§FS-rules.5.1,
//! §AR-rules.3). It parses no sentence and evaluates no constraint.

use super::RuleAnchor;
use super::facts::{Completeness, FactHeader, NodeKey, NodeMeta, RuleFacts, SiteKey, SiteMeta};
use crate::grammar::{Grammar, render_catalog_id, section_display_name, unnamed_component};
use crate::model::{Catalog, Id, paired_declarations, section_resolves};
use std::collections::BTreeMap;

/// What the adapter reads of one project, and nothing more: the rules
/// component holds no configuration façade (§AR-config.5, §AR-rules.3). The
/// checker builds it from the project it checks.
#[derive(Clone, Copy)]
pub(crate) struct MarkdownProject<'a> {
    /// The stable project name, the selected scope of a standalone adaptation.
    pub(crate) name: Option<&'a str>,
    /// The ID grammar labels render through.
    pub(crate) grammar: &'a Grammar,
    /// The separator between an ID and its section in a chapter label.
    pub(crate) section_separator: &'a str,
}

/// Adapt one standalone project. The producer-neutral schema uses the stable
/// project name as its selected scope (§AR-rules.3).
pub(crate) fn adapt_markdown(
    findings: &Catalog,
    project: MarkdownProject<'_>,
    complete: bool,
) -> RuleFacts {
    let selected = project.name.unwrap_or("local").to_string();
    adapt_workspace(
        &selected,
        &[(selected.as_str(), findings, project)],
        complete,
    )
}

/// Adapt the complete resolved workspace while keeping the selected member's
/// subjects local. External declarations carry qualified kind/labels, so the
/// same engine handles local, pinned, and any-member object selectors without
/// learning resolver records (§FS-rules.5.1, §AR-rules.3). An ID's declarations, its
/// nodes, the edges into it and the sections a citation of it reaches are read off
/// each stub's verdict (§AR-scanner.4.6), and a declaration's chapters and
/// citations off the record it is paired with, so a stub's target outside the walk
/// lends both, judged as scanned ones since §FS-rules.7.11 closed
/// (§FS-declarations.stubs.target.5).
pub(crate) fn adapt_workspace(
    selected: &str,
    projects: &[(&str, &Catalog, MarkdownProject<'_>)],
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

    for (alias, findings, project) in projects {
        let local = *alias == selected;
        for (id, homes) in &findings.declarations {
            let bare_label = render_catalog_id(project.grammar, &findings.declarations, id);
            let label = if local {
                bare_label.clone()
            } else {
                format!("{alias}/{bare_label}")
            };
            // A healthy Markdown stub and its inline declaration are one declaration once
            // paired (§AR-scanner.4.6): one node, so relations share it
            // (§FS-rules.5.1).
            let paired = paired_declarations(homes);
            for (ordinal, home) in paired.stand_ins().enumerate() {
                // §FS-declarations.stubs.target.5: the chapters are the paired record's,
                // and §FS-declarations.stubs.pairing.4 anchors the declaration at it.
                let records = paired
                    .iter()
                    .filter(|paired| std::ptr::eq(paired.stand_in, home));
                for (nth, record) in records.enumerate() {
                    // A target declaring the ID twice is two declarations, as scanned: each
                    // record its own node, anchored at its own line.
                    let ordinal = match nth {
                        0 => ordinal.to_string(),
                        nth => format!("{ordinal}.{nth}"),
                    };
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
                                path: record.record.file.to_string_lossy().into_owned(),
                                line: record.record.line,
                                column: None,
                            },
                        },
                    );
                    for (section, info) in &record.record.sections {
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
                            (!info.in_front_matter()).then(|| {
                                section_display_name(&info.title, section, info.relative)
                                    .into_owned()
                            }),
                        ));
                        facts.nodes.insert(
                            chapter,
                            NodeMeta {
                                label: format!("{label}{}{section}", project.section_separator),
                                anchor: RuleAnchor {
                                    path: record.record.file.to_string_lossy().into_owned(),
                                    line: info.line,
                                    column: None,
                                },
                            },
                        );
                    }
                }
            }
        }
    }

    for (alias, findings, _) in projects {
        // §FS-declarations.stubs.target.5: a stub's target lends its citations as edges.
        for (ordinal, citation) in findings.edges().enumerate() {
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
            let Some((_, target_findings, _)) = projects
                .iter()
                .find(|(candidate, _, _)| *candidate == target_alias)
            else {
                continue;
            };
            let Some(target_homes) = target_findings.declarations.get(&citation.id) else {
                continue;
            };
            // A stub whose target declares the ID twice stands for two declarations, so
            // its ID is ambiguous here as it is to `check` (§AR-scanner.4.6).
            if paired_declarations(target_homes).len() != 1 {
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
            // §FS-rules.5.1, §FS-rules.7.12: each section counts for its nearest named ancestor
            // chapter, or else its declaration, including mixed paths. A stub's sections
            // resolve from its target, scanned or not (§AR-scanner.4.6).
            let target = match citation.section.as_ref() {
                Some(section) => {
                    if !section_resolves(target_findings, &citation.id, section) {
                        continue;
                    }
                    nearest_chapter(&chapters, target_alias, &citation.id, section)
                        .unwrap_or(target_declaration)
                }
                None => target_declaration,
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
                },
            );
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

    facts
}

/// A section is a rule unit where a chapter subject could name it, which the
/// sentence grammar answers (§AR-rules.2, §AR-rules.3).
fn is_rule_unit(section: &str) -> bool {
    unnamed_component(section).is_none()
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
