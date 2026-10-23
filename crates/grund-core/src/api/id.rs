//! The published `id` contract (§AR-system.2.9): the next conflict-free
//! declaration ID for a kind and a title, as data rather than a printed report
//! (§FS-id, §AR-bindings.2).
//!
//! The two questions the answer is built from — what a title slugs to and how an
//! `Id` renders under `[id] format` — are the writers' (§AR-system.2.8), read
//! downward. The CLI renders the returned proposal; no process adapter lives in
//! the engine (§AR-system.2.9.1).

use anyhow::Result;
use std::path::PathBuf;

use super::config::config_run_warnings;
use crate::config::{
    Invocation, KindScope, KindTable, KindVerdict, Quote, Selection, display_path,
};
use crate::model::{Finding, Id};
use crate::scanner::{e2e_case_dir_name, scan_tree_strict};
use crate::sources::paths_same_location;
use crate::workspace::open;
use crate::writers::{format_id, slugify_title};

#[derive(Clone)]
#[non_exhaustive]
pub struct IdOpts {
    pub path: PathBuf,
    pub path_provided: bool,
    pub width: usize,
}

impl Default for IdOpts {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            path_provided: false,
            width: 3,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct IdProposal {
    pub id: String,
    pub kind: String,
    pub number: Option<u32>,
    pub slug: String,
    /// The kind's first folder, which every reader of `folder` keeps reading.
    pub folder: Option<String>,
    /// Every folder of the kind's row in written order, for a version-2 config
    /// only (§FS-id.2.2); `None` under v1, whose object is unchanged.
    pub folders: Option<Vec<String>>,
    pub file: Option<String>,
    pub e2e_case_dir: Option<String>,
    /// Whether the kind's configured `file` already holds exactly one
    /// declaration of the kind, which is what makes the new declaration's
    /// hint an H1 rather than the H2 convention (§FS-id.2.3,
    /// §FS-config.3.4.4). It is a fact about the file's contents, not about
    /// the kind's name: `GRUND`'s scaffolded home is the shape, not the test.
    pub file_holds_single_declaration: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IdProposalOutcome {
    Proposed(IdProposal),
    /// The kind is not one `id` can mint from (§FS-id.1.1): a name the config does
    /// not hold, or one it holds with `citable = false` (§FS-config.3.4.1).
    /// `headline` says which — the two are different mistakes, and the caller
    /// prints the same shape for both, with `known` listing the citable kinds.
    UnknownKind {
        headline: String,
        known: Vec<String>,
    },
    Rejected {
        message: String,
    },
}

impl IdProposalOutcome {
    /// The `known kinds:` line a frontend prints under an `UnknownKind` headline
    /// (§FS-id.1.1): the one renderer every refusal that names kinds shares, so
    /// a project with no kinds reads `none` here too (§FS-list.1.1.1).
    pub fn known_kinds_line(known: &[String]) -> String {
        crate::config::known_kinds_line(known.iter().map(String::as_str))
    }
}

/// Programmatic `id`: compute the next conflict-free declaration ID without
/// parsing CLI flags or printing the text/JSON report (§AR-bindings.2).
pub fn propose_id(kind: &str, title: &str, opts: IdOpts) -> Result<IdProposalOutcome> {
    propose_id_with_run_warnings(kind, title, opts).1
}

/// [`propose_id`] for a frontend that also renders the run's `[workspace]`
/// warnings (§FS-check.4.10.7): `id` resolves a block's member
/// boundary like every other walking command, and the outcome is an enum with
/// nowhere to carry a caution.
#[doc(hidden)]
pub fn propose_id_with_run_warnings(
    kind: &str,
    title: &str,
    opts: IdOpts,
) -> (Vec<Finding>, Result<IdProposalOutcome>) {
    let mut run_warnings = Vec::new();
    let outcome = propose_id_run(kind, title, opts, &mut run_warnings);
    (run_warnings, outcome)
}

fn propose_id_run(
    kind: &str,
    title: &str,
    opts: IdOpts,
    run_warnings: &mut Vec<Finding>,
) -> Result<IdProposalOutcome> {
    // §AR-config.1.5: the project the path discovers, read as `check`'s
    // resolution scope is, so the number and the collision are the whole project's.
    let config = open(&Invocation::new(&opts.path, true, Selection::Check))?;
    *run_warnings = config_run_warnings(&config);
    // §AR-config.1.5: `grund id` asks in the project the run loads.
    let table = KindTable::standalone(config.schema());
    let scope = KindScope::Project("");
    let headline = match table.verdict(scope, kind) {
        KindVerdict::Citable => None,
        // §FS-id.1.1: a non-citable kind is configured and still has nothing to
        // mint, so it is refused in the same shape with its reason in place of
        // "unknown" — a real row in the table is not a typo (§FS-errors.2.5.1).
        KindVerdict::NonCitable(entry) => Some(entry.non_citable_reason(Quote::Backtick)),
        _ => Some(format!("unknown kind `{kind}`")),
    };
    let configured = config.kinds.iter().find(|candidate| candidate.kind == kind);
    let (None, Some(kind_config)) = (&headline, configured) else {
        return Ok(IdProposalOutcome::UnknownKind {
            headline: headline.unwrap_or_else(|| format!("unknown kind `{kind}`")),
            known: table
                .citable(scope)
                .into_iter()
                .map(str::to_string)
                .collect(),
        });
    };
    let slug = slugify_title(title, &config.slug_pattern);
    if slug.is_empty() {
        return Ok(IdProposalOutcome::Rejected {
            message: format!("title produces empty slug after normalization: \"{title}\""),
        });
    }
    let findings = scan_tree_strict(
        config.schema(),
        config.frame(),
        Some(&opts.path),
        opts.path_provided,
    )?;
    let kind_format = kind_config.effective_format(&config);
    let uses_number = kind_format.contains("{number}");
    let number = if uses_number {
        let max = findings
            .declarations
            .keys()
            .filter(|id| id.kind == kind)
            .filter_map(|id| id.num)
            .max()
            .unwrap_or(0);
        Some(max + 1)
    } else {
        None
    };
    let id = Id {
        kind: kind.to_string(),
        num: number,
        // §FS-config.3.2.8: allocation chooses width independently of source digits.
        slug: if kind_format.contains("{slug}") {
            Some(slug.clone())
        } else {
            None
        },
    };
    let rendered = format_id(&id, &config, opts.width);
    // §FS-id.3.2: print only what `check` accepts, read by the parser a declaration is read by.
    if config.grammar.parse_token(&rendered).as_ref() != Some(&id) {
        return Ok(IdProposalOutcome::Rejected {
            message: format!(
                "title produces slug \"{slug}\" that slug_pattern \"{}\" does not accept: \"{title}\"",
                config.slug_pattern
            ),
        });
    }
    if let Some(decls) = findings.declarations.get(&id)
        && let Some(decl) = decls.first()
    {
        return Ok(IdProposalOutcome::Rejected {
            message: format!(
                "proposed ID `{}` already declared at {}:{}",
                rendered,
                display_path(&config, &decl.file),
                decl.line
            ),
        });
    }
    // §FS-id.2.3: the H1 hint is owed where the kind's file holds the kind's single declaration,
    // counted in that file — a declaration of the kind written elsewhere is
    // §FS-declarations.checks.misplaced-declaration's error, not this hint's fact.
    let file_holds_single_declaration = kind_config.file.as_deref().is_some_and(|file| {
        let home = config.root.join(file);
        findings
            .declarations
            .iter()
            .filter(|(declared, _)| declared.kind == kind)
            .flat_map(|(_, decls)| decls.iter())
            .filter(|decl| paths_same_location(&decl.file, &home))
            .count()
            == 1
    });
    Ok(IdProposalOutcome::Proposed(IdProposal {
        e2e_case_dir: (kind == "E2E").then(|| e2e_case_dir_name(config.schema(), &rendered)),
        file_holds_single_declaration,
        id: rendered,
        kind: kind.to_string(),
        number,
        slug,
        folder: kind_config.folder.clone(),
        folders: (config.project().version == 2).then(|| {
            let row = config.schema().rows.iter().find(|row| row.name == kind);
            row.into_iter()
                .flat_map(|row| row.folders())
                .map(str::to_string)
                .collect()
        }),
        file: kind_config.file.clone(),
    }))
}
