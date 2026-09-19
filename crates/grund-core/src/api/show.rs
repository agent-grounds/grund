//! The published `show` contract and the one adapter behind it
//! (§AR-system.2.9): four entry points over one function that resolves the
//! argument against the loaded workspace, slices the body and renders the JSON
//! shape when asked (§FS-show, §FS-distribution.3.1).
//!
//! The adapter is here rather than in `check.rs`'s company because it is what
//! §AR-core-module-layout.2 names as the reason the contract had grown: it holds
//! the project selection, the alias refusals and the section conflict, none of
//! which is a published signature. The options and the typed refusal it raises
//! belong to the query that answers, so they are `queries/show_query.rs`
//! (§AR-system.2.7).

use anyhow::{Result, anyhow};
use std::collections::BTreeMap;
use std::path::PathBuf;

use super::lsp_snapshot::normalized_overlays;
use crate::config::display_path;
use crate::grammar::flatten_cross_ref_links;
use crate::model::{Finding, ShowOutput, TextOverlays};
use crate::queries::{
    ShowFormat, ShowOpts, render_show_output_json, show_declaration_with_overlays,
};
use crate::resolver::{load_workspace_context_with_overlays, with_member_id_candidates};

use super::report::context_run_warnings;
use crate::scanner::resolve_id_arg;
use crate::workspace::split_qualified_id_arg;

/// Programmatic declaration read. This mirrors `grund show` resolution but
/// returns the structured body instead of printing it.

pub fn show(id_arg: &str, opts: ShowOpts) -> Result<ShowOutput> {
    show_with_scope(id_arg, opts, true).1
}

pub fn show_with_overlays(
    id_arg: &str,
    opts: ShowOpts,
    open_documents: BTreeMap<PathBuf, String>,
) -> Result<ShowOutput> {
    show_with_scope_and_overlays(id_arg, opts, true, &normalized_overlays(open_documents)).1
}

/// The ID read behind `grund <ID>` (§FS-show).
///
/// The run's `[workspace]` warnings come back beside the answer rather than on it,
/// because a refusal settled *after* the workspace pass — an unknown alias, an
/// unresolvable ID — leaves an `Err` with nowhere to carry a caution the reader is
/// already owed (§FS-check.4.7.2, §FS-check.4.10.7, §FS-workspace.6.1.7).
#[doc(hidden)]
pub fn show_with_scope(
    id_arg: &str,
    opts: ShowOpts,
    path_provided: bool,
) -> (Vec<Finding>, Result<ShowOutput>) {
    show_with_scope_and_overlays(id_arg, opts, path_provided, &TextOverlays::new())
}

fn show_with_scope_and_overlays(
    id_arg: &str,
    opts: ShowOpts,
    path_provided: bool,
    overlays: &TextOverlays,
) -> (Vec<Finding>, Result<ShowOutput>) {
    let mut run_warnings = Vec::new();
    let output = show_run(id_arg, opts, path_provided, overlays, &mut run_warnings);
    (run_warnings, output)
}

fn show_run(
    id_arg: &str,
    opts: ShowOpts,
    path_provided: bool,
    overlays: &TextOverlays,
    run_warnings: &mut Vec<Finding>,
) -> Result<ShowOutput> {
    let context = load_workspace_context_with_overlays(&opts.path, path_provided, overlays, false)?;
    *run_warnings = context_run_warnings(&context);
    let (alias, raw_id) = split_qualified_id_arg(id_arg)?;
    let project = match alias.as_deref() {
        Some(name) => context
            .project_by_alias(name)
            .ok_or_else(|| {
                if !context.workspace_loaded {
                    anyhow!(
                        "unknown project alias `{name}`\nnote: workspace aliases are defined in the root grund.toml under [workspace]"
                    )
                } else {
                    anyhow!(
                        "unknown project alias `{name}`\nknown aliases: {}",
                        context.aliases().join(", ")
                    )
                }
            })?,
        None => context.current_project().ok_or_else(|| {
            let known = context.aliases().join(", ");
            if known.is_empty() {
                anyhow!("unqualified ID requires a project alias when include_root = false")
            } else {
                anyhow!(
                    "unqualified ID requires a project alias when include_root = false\nknown aliases: {known}"
                )
            }
        })?,
    };
    // §FS-workspace.8.7.3: rendered against the run's config, not the target
    // project's — the same spelling `check` uses for the same tree.
    if let Some((file, message)) = project.scan_errors.first() {
        return Err(anyhow!(
            "{}: {}",
            display_path(context.render_config(), file),
            message
        ));
    }
    let config = &project.config;
    let (id, inline_section) =
        resolve_id_arg(raw_id, config, &project.findings).map_err(|err| anyhow!("{err}"))?;
    if opts.section.is_some() && inline_section.is_some() {
        return Err(anyhow!(
            "--section cannot be combined with an inline section"
        ));
    }
    let section = opts.section.or(inline_section);
    let mut output = show_declaration_with_overlays(
        config,
        context.render_config(),
        &project.findings,
        &id,
        section.as_deref(),
        opts.mode.render_mode(),
        opts.format == ShowFormat::Markdown,
        overlays,
    )
    // §FS-workspace.8.1.1: a miss names the projects that do declare the ID,
    // read off the context this run already loaded.
    .map_err(|err| with_member_id_candidates(err, &context, alias.as_deref(), raw_id))?;
    if opts.format != ShowFormat::Markdown {
        output.body = flatten_cross_ref_links(&output.body, config.lexical());
    }
    if opts.format == ShowFormat::Json {
        let json = render_show_output_json(
            config,
            context.render_config(),
            &id,
            section.as_deref(),
            opts.mode.render_mode(),
            &output,
        );
        output.json = Some(json);
    }
    Ok(output)
}
