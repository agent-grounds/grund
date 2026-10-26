//! The run's one reading of whose kinds are what (§FS-scan.projects.kinds,
//! §AR-config.1.5): each project's kinds and its named-sections setting, read
//! from its config alone, and every kind-name question answered from that
//! reading within the asking command's scope. It is config-time work — no file
//! is read — so it lives in the config component, and a run that reads a
//! workspace builds, judges and holds its table with one call to
//! [`install_kind_table`] (§AR-workspace.5.2). A run that reads no workspace
//! holds the table of its one project ([`KindTable::standalone`]).
//!
//! The refusal a configured but non-citable kind earns is rendered here too,
//! once, in the quoting each surface already uses (§FS-errors.2.5.1), alone
//! since the one-grammar window closed (§FS-errors.3.8.1).

use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use super::project::Schema;
use super::record::{CODE_SOURCE_KIND, Config};
use super::validate::reject_unaccepted_citation_targets;

/// How a surface quotes a kind name (§FS-errors.2.5.1): a backtick on `list
/// --kind`, `grund id` and in config, a double quote on a selector and a rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Quote {
    Backtick,
    Double,
}

impl Quote {
    fn wrap(self, name: &str) -> String {
        match self {
            Self::Backtick => format!("`{name}`"),
            Self::Double => format!("\"{name}\""),
        }
    }
}

/// One configured kind of one project: its name, whether it declares IDs, and
/// the home a refusal names (§FS-config.3.4.1).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KindEntry {
    pub(crate) name: String,
    pub(crate) citable: bool,
    /// `<folder>/` or the file, `None` for a kind with no place (§FS-check.3.11).
    pub(crate) home: Option<String>,
}

impl KindEntry {
    /// §FS-errors.2.5.1: why this configured kind is refused where a citable
    /// one is wanted, in `quote`'s spelling: the whole refusal on every surface
    /// since the one-grammar window closed (§FS-errors.3.8.1,
    /// §DF-one-grammar-lands-in-0-19).
    pub(crate) fn non_citable_reason(&self, quote: Quote) -> String {
        let kind = quote.wrap(&self.name);
        match &self.home {
            Some(home) => format!("kind {kind} declares no IDs — {home} is not a citable home"),
            None => format!("kind {kind} declares no IDs"),
        }
    }
}

/// One project's kinds, read from its config (§FS-scan.projects.kinds).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KindProject {
    /// The alias path a citation names the project by; empty in a run that
    /// holds no workspace.
    pub(crate) alias: String,
    /// Every `[[kinds]]` row, citable or not, in configuration order.
    pub(crate) kinds: Vec<KindEntry>,
    /// `[id] named_sections` of this project's own config.
    pub(crate) named_sections: bool,
    /// `code`, where the config writes no complement of its own
    /// (§FS-config.3.9.2): a kind with no row, which only
    /// [`KindTable::config_verdict`] reads.
    implicit_complement: Option<KindEntry>,
}

impl KindProject {
    /// The one reader of a schema's kinds: every row, whether it declares IDs,
    /// and its home (§FS-config.3.4, §FS-config.3.4.1).
    pub(crate) fn of(alias: impl Into<String>, schema: &Schema) -> Self {
        Self {
            alias: alias.into(),
            kinds: schema
                .rows
                .iter()
                .map(|row| KindEntry {
                    name: row.name.clone(),
                    citable: row.kind.is_some(),
                    // §FS-config-v2.schema.places: every place, and none for the complement.
                    home: row.place_labels().filter(|_| !row.is_complement()),
                })
                .collect(),
            named_sections: schema.ids.named_sections,
            implicit_complement: (!schema.rows.iter().any(|row| row.is_complement())).then(|| {
                KindEntry {
                    name: CODE_SOURCE_KIND.to_string(),
                    citable: false,
                    home: None,
                }
            }),
        }
    }

    /// The citable kinds, in configuration order.
    pub(crate) fn citable(&self) -> impl Iterator<Item = &str> {
        self.kinds
            .iter()
            .filter(|kind| kind.citable)
            .map(|kind| kind.name.as_str())
    }

    /// The configured kinds that declare no IDs, in configuration order.
    pub(crate) fn non_citable(&self) -> impl Iterator<Item = &KindEntry> {
        self.kinds.iter().filter(|kind| !kind.citable)
    }

    fn entry(&self, name: &str) -> Option<&KindEntry> {
        self.kinds.iter().find(|kind| kind.name == name)
    }
}

/// The projects a kind-name question is asked in (§FS-scan.projects.kinds).
#[derive(Clone, Copy, Debug)]
pub(crate) enum KindScope<'a> {
    /// The projects `list` selects: every one, or those `--project` names.
    Selected(&'a BTreeSet<String>),
    /// One project, by alias: a rule subject, or a bare object or target of
    /// the project whose config reads it.
    Project(&'a str),
    /// The project an `alias/KIND` target or object pins.
    Pinned(&'a str),
    /// Every project, for a `*/KIND` target or object; in a run with no
    /// workspace that is its one project, the only one it can judge.
    Any,
}

/// What a kind name is within one scope (§FS-scan.projects.kinds).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KindVerdict<'a> {
    Citable,
    /// Configured but not citable anywhere in scope: the first such entry, in
    /// load order, names the home.
    NonCitable(&'a KindEntry),
    Unknown,
    /// A pinned alias the workspace does not load.
    UnknownAlias,
    /// The scope names other projects, and the run holds no workspace.
    Unverifiable,
}

/// The run's kind table (§AR-config.1.5): one row per project the run's config
/// declares.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct KindTable {
    pub(crate) projects: Vec<KindProject>,
    /// Whether the run holds a workspace, so an alias can be judged at all
    /// (§FS-rules.4.1.1): its absence is what makes a namespaced object kind
    /// unverifiable here rather than invalid.
    pub(crate) workspace: bool,
}

impl KindTable {
    /// The table of a run that expanded a workspace: every project it loaded,
    /// by alias, read once (§FS-scan.projects.kinds).
    pub(crate) fn build<'a>(projects: impl IntoIterator<Item = (&'a str, &'a Schema)>) -> Self {
        Self {
            projects: projects
                .into_iter()
                .map(|(alias, schema)| KindProject::of(alias, schema))
                .collect(),
            workspace: true,
        }
    }

    /// The table of a run that holds no workspace: its one project.
    pub(crate) fn standalone(schema: &Schema) -> Self {
        Self {
            projects: vec![KindProject::of("", schema)],
            workspace: false,
        }
    }

    /// A table of one project whose kinds are `citable`, each citable: the
    /// object table of a vocabulary a test builds by hand.
    #[cfg(test)]
    pub(crate) fn of_citable(citable: &BTreeSet<String>) -> Self {
        Self {
            projects: vec![KindProject {
                alias: String::new(),
                kinds: citable
                    .iter()
                    .map(|name| KindEntry {
                        name: name.clone(),
                        citable: true,
                        home: None,
                    })
                    .collect(),
                named_sections: false,
                implicit_complement: None,
            }],
            workspace: false,
        }
    }

    /// Adds the projects a rule's run loaded, by alias, to the table of the
    /// project whose config reads the rule (§FS-rules.4.1.1): holding any is what
    /// lets a pinned or `*/` object be judged rather than found unverifiable.
    pub(crate) fn add_projects<'a>(
        &mut self,
        projects: impl IntoIterator<Item = (&'a str, &'a Schema)>,
    ) {
        let before = self.projects.len();
        self.projects.extend(
            projects
                .into_iter()
                .map(|(alias, schema)| KindProject::of(alias, schema)),
        );
        self.workspace |= self.projects.len() > before;
    }

    /// The projects `scope` reads, in load order. A pinned alias the table
    /// does not hold reads none.
    fn in_scope<'s>(&'s self, scope: KindScope<'s>) -> impl Iterator<Item = &'s KindProject> {
        self.projects.iter().filter(move |project| match scope {
            KindScope::Selected(filter) => filter.is_empty() || filter.contains(&project.alias),
            KindScope::Project(alias) => !self.workspace || project.alias == alias,
            KindScope::Pinned(alias) => self.workspace && project.alias == alias,
            KindScope::Any => true,
        })
    }

    /// §FS-scan.projects.kinds: the one verdict for `name` in `scope`. Citable
    /// wins where two projects configure the name differently.
    pub(crate) fn verdict<'s>(&'s self, scope: KindScope<'s>, name: &str) -> KindVerdict<'s> {
        let mut non_citable = None;
        for project in self.in_scope(scope) {
            match project.entry(name) {
                Some(entry) if entry.citable => return KindVerdict::Citable,
                Some(entry) => {
                    non_citable.get_or_insert(entry);
                }
                None => {}
            }
        }
        // §FS-rules.4.1: a scope naming other projects that the run cannot read
        // is judged no further than the citable answer it already has.
        if matches!(scope, KindScope::Pinned(_) | KindScope::Any) && !self.workspace {
            return KindVerdict::Unverifiable;
        }
        if let Some(entry) = non_citable {
            return KindVerdict::NonCitable(entry);
        }
        match scope {
            KindScope::Pinned(alias) if !self.projects.iter().any(|p| p.alias == alias) => {
                KindVerdict::UnknownAlias
            }
            _ => KindVerdict::Unknown,
        }
    }

    /// §FS-config.3.9.5: the verdict on a name a config reads as a `[citations]`
    /// kind, citing or cited — [`Self::verdict`] with one explicit case. `code`,
    /// the complement a config that writes none of its own still has
    /// (§FS-config.3.9.2), has no row; config has always read it as configured
    /// and not citable, a kind that cites and is never cited, and keeps that
    /// answer. Every other surface called it unknown, and since `0.19.0` gives
    /// this verdict's reason alone (§FS-errors.3.8.1,
    /// [`Self::unknown_or_complement`]).
    pub(crate) fn config_verdict<'s>(
        &'s self,
        scope: KindScope<'s>,
        name: &str,
    ) -> KindVerdict<'s> {
        match self.verdict(scope, name) {
            KindVerdict::Unknown => self
                .in_scope(scope)
                .find_map(|project| {
                    project
                        .implicit_complement
                        .as_ref()
                        .filter(|entry| entry.name == name)
                })
                .map_or(KindVerdict::Unknown, KindVerdict::NonCitable),
            verdict => verdict,
        }
    }

    /// §FS-errors.3.8.1: `legacy`, the refusal a surface gives a name `scope`
    /// does not configure, or, where `scope` configures the name without IDs —
    /// the complement `code` with no row included — §FS-errors.2.5.1's reason in
    /// `quote`'s spelling in its place (§FS-list.1.1,
    /// [`KindEntry::non_citable_reason`]).
    pub(crate) fn unknown_or_complement(
        &self,
        scope: KindScope<'_>,
        name: &str,
        legacy: String,
        quote: Quote,
    ) -> String {
        match self.config_verdict(scope, name) {
            KindVerdict::NonCitable(entry) => entry.non_citable_reason(quote),
            _ => legacy,
        }
    }

    /// The citable kinds of `scope`, once each, in configuration order: the
    /// names its `known kinds:` line lists (§FS-list.1.1).
    pub(crate) fn citable<'s>(&'s self, scope: KindScope<'s>) -> Vec<&'s str> {
        let mut seen = BTreeSet::new();
        self.in_scope(scope)
            .flat_map(KindProject::citable)
            .filter(|name| seen.insert(*name))
            .collect()
    }

    /// The `known kinds:` line a refusal in `scope` ends with.
    pub(crate) fn known_kinds_line(&self, scope: KindScope<'_>) -> String {
        known_kinds_line(self.citable(scope))
    }

    /// The kinds of `scope` that are configured and citable in none of it,
    /// each with the entry that names its home, and `code` where it is the
    /// complement with no row that nothing in scope configures otherwise
    /// (§FS-errors.2.5.1).
    pub(crate) fn non_citable<'s>(&'s self, scope: KindScope<'s>) -> Vec<&'s KindEntry> {
        let mut seen = BTreeSet::new();
        self.in_scope(scope)
            .flat_map(|project| project.non_citable().chain(&project.implicit_complement))
            .filter(|entry| {
                matches!(
                    self.config_verdict(scope, &entry.name),
                    KindVerdict::NonCitable(_)
                )
            })
            .filter(|entry| seen.insert(entry.name.as_str()))
            .collect()
    }

    /// The same kinds by name, each with the entry a selector and a rule
    /// refuse it by (§FS-errors.2.5.1, [`KindEntry::non_citable_reason`]).
    pub(crate) fn non_citable_entries(&self, scope: KindScope<'_>) -> BTreeMap<String, KindEntry> {
        self.non_citable(scope)
            .into_iter()
            .map(|entry| (entry.name.clone(), entry.clone()))
            .collect()
    }

    /// Whether every project of `scope` turns named sections on: the setting a
    /// subject whose kind is not yet read is held to.
    pub(crate) fn named_sections(&self, scope: KindScope<'_>) -> bool {
        self.in_scope(scope).all(|project| project.named_sections)
    }

    /// The citable kinds of `scope` whose own project turns named sections on:
    /// a named chapter of each is addressable whatever another project sets
    /// (§FS-scan.projects.kinds, §REQ-one-answer.sections.named).
    pub(crate) fn named_section_kinds<'s>(&'s self, scope: KindScope<'s>) -> BTreeSet<&'s str> {
        self.in_scope(scope)
            .filter(|project| project.named_sections)
            .flat_map(KindProject::citable)
            .collect()
    }
}

/// §AR-config.1.5: the one call a run that reads a workspace makes. Builds the
/// table from `projects`, each by alias, judges every project's pinned and `*/`
/// `[citations]` targets against it once, in load order, and has `root` and
/// every project hold it. The first target the run cannot accept fails the load
/// (§FS-config.3.9.5.2, §DF-one-grammar-lands-in-0-19). Expansion makes it for a
/// full run, and a narrowed `cover` run makes it over the workspace its config
/// declares (§FS-scan.projects.kinds). `whole` says `projects` is every project
/// of the workspace: a reading that lost one it could not load is held but never
/// judged, since what that project configures is no ground for a refusal.
pub(crate) fn install_kind_table<'a>(
    root: &mut Config,
    projects: impl IntoIterator<Item = (&'a str, &'a mut Config)>,
    whole: bool,
) -> Result<()> {
    let mut projects = projects.into_iter().collect::<Vec<_>>();
    let table = Arc::new(KindTable::build(
        projects
            .iter()
            .map(|(alias, config)| (*alias, config.schema())),
    ));
    if whole {
        for (_, config) in &projects {
            reject_unaccepted_citation_targets(config, &table)?;
        }
    }
    for (_, config) in &mut projects {
        config.set_kind_table(Arc::clone(&table));
    }
    root.set_kind_table(table);
    Ok(())
}

/// The `known kinds:` line that ends an unknown `--kind` (§FS-list.1.1), a
/// `--selector` refusal that recovers no kind (§FS-rule-refusals.selectors), and a
/// `check --rule` refusal that offers no form where a kind is missing
/// (§FS-rule-refusals.sentences.rewrite.no-form): every citable kind named in
/// `citable`, once, in configuration order — and `grund id`'s too (§FS-id.1.1).
/// A scope with no citable kind at all says so and how to start, rather than
/// ending on an empty list (§FS-list.1.1.1).
pub(crate) fn known_kinds_line<'a>(citable: impl IntoIterator<Item = &'a str>) -> String {
    let mut seen = BTreeSet::new();
    let known = citable
        .into_iter()
        .filter(|kind| seen.insert(*kind))
        .collect::<Vec<_>>();
    if known.is_empty() {
        return "known kinds: none — declare one in grund.toml first".to_string();
    }
    format!("known kinds: {}", known.join(", "))
}

/// §FS-errors.3.7.1: why a target or object naming other projects cannot be
/// judged in a run that holds no workspace — the reason names what the scope
/// cannot reach, never a candidate it cannot have (§FS-check.3.8.3).
pub(crate) fn unverifiable_reason(alias: Option<&str>, kind: &str) -> String {
    let reason = match alias {
        Some(alias) => format!(
            "unknown project alias {alias}; no workspace is in scope here, so the alias cannot be resolved"
        ),
        None => {
            format!("no workspace is in scope here, so no namespace can be searched for {kind}")
        }
    };
    format!(
        "{reason} — {}",
        crate::model::remedies::invalid_rule_workspace()
    )
}

#[cfg(test)]
#[path = "tests_kinds.rs"]
mod tests_kinds;
