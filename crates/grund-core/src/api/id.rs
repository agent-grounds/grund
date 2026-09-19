//! The published `id` contract (§AR-system.2.9): the next conflict-free
//! declaration ID for a kind and a title, as data rather than a printed report
//! (§FS-id, §AR-bindings.2).
//!
//! The two questions the answer is built from — what a title slugs to and how an
//! `Id` renders under `[id] format` — are the writers' (§AR-system.2.8), read
//! downward, so this surface and the deprecated `compat/id.rs` adapter cannot
//! disagree about either.

use anyhow::Result;
use std::path::PathBuf;

use super::config::config_run_warnings;
use crate::config::{display_path, kind_prefixes, non_citable_kind_error};
use crate::model::{Finding, Id};
use crate::scanner::{e2e_case_dir_name, scan_tree_strict};
use crate::workspace::resolve_workspace_config;
use crate::writers::{format_id, slugify_title};

#[derive(Clone)]
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
pub struct IdProposal {
    pub id: String,
    pub kind: String,
    pub number: Option<u32>,
    pub slug: String,
    pub folder: Option<String>,
    pub file: Option<String>,
    pub e2e_case_dir: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
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

/// Programmatic `id`: compute the next conflict-free declaration ID without
/// parsing CLI flags or printing the text/JSON report (§AR-bindings.2).
pub fn propose_id(kind: &str, title: &str, opts: IdOpts) -> Result<IdProposalOutcome> {
    propose_id_with_run_warnings(kind, title, opts).1
}

/// [`propose_id`] for a frontend that also renders the run's `[workspace]`
/// warnings (§FS-check.4.7.2, §FS-check.4.10.7): `id` resolves a block's member
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
    let config = resolve_workspace_config(&opts.path)?;
    *run_warnings = config_run_warnings(&config);
    let configured = config.kinds.iter().find(|candidate| candidate.kind == kind);
    let Some(kind_config) = configured.filter(|candidate| candidate.citable) else {
        // §FS-id.1.1: a non-citable kind is configured and still has nothing to
        // mint, so it is refused in the same shape with its reason in place of
        // "unknown" — a real row in the table is not a typo.
        return Ok(IdProposalOutcome::UnknownKind {
            headline: match configured {
                Some(candidate) => non_citable_kind_error(candidate),
                None => format!("unknown kind `{kind}`"),
            },
            known: kind_prefixes(&config.kinds),
        });
    };
    let slug = slugify_title(title, &config.slug_pattern);
    if slug.is_empty() {
        return Ok(IdProposalOutcome::Rejected {
            message: format!("title produces empty slug after normalization: \"{title}\""),
        });
    }
    let findings = scan_tree_strict(&config, Some(&opts.path), opts.path_provided)?;
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
        slug: if kind_format.contains("{slug}") {
            Some(slug.clone())
        } else {
            None
        },
    };
    let rendered = format_id(&id, &config, opts.width);
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
    Ok(IdProposalOutcome::Proposed(IdProposal {
        e2e_case_dir: (kind == "E2E").then(|| e2e_case_dir_name(&config, &rendered)),
        id: rendered,
        kind: kind.to_string(),
        number,
        slug,
        folder: kind_config.folder.clone(),
        file: kind_config.file.clone(),
    }))
}
