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

use crate::config::display_path;
use crate::grammar::flatten_cross_ref_links;
use crate::model::{Declaration, Finding, Id, ShowOutput, ShowRenderMode};
use crate::queries::{
    LspShowContext, ShowFormat, ShowMode, ShowOpts, render_show_output_json,
    show_declaration_with_home,
};
use crate::resolver::{
    WorkspaceContext, WorkspaceProject, load_reported_workspace_context, with_member_id_candidates,
};

use super::report::context_run_warnings;
use crate::scanner::resolve_id_arg;
use crate::workspace::split_qualified_id_arg;

/// Programmatic declaration read. This mirrors `grund show` resolution but
/// returns the structured body instead of printing it.

pub fn show(id_arg: &str, opts: ShowOpts) -> Result<ShowOutput> {
    show_with_scope(id_arg, opts, true).1
}

/// The ID read behind `grund <ID>` (§FS-show).
///
/// The run's `[workspace]` warnings come back beside the answer rather than on it,
/// because a refusal settled *after* the workspace pass — an unknown alias, an
/// unresolvable ID — leaves an `Err` with nowhere to carry a caution the reader is
/// already owed (§FS-check.4.10.7, §FS-workspace.6.1.7).
#[doc(hidden)]
pub fn show_with_scope(
    id_arg: &str,
    opts: ShowOpts,
    path_provided: bool,
) -> (Vec<Finding>, Result<ShowOutput>) {
    let mut run_warnings = Vec::new();
    let output = show_run(
        id_arg,
        opts,
        path_provided,
        &mut run_warnings,
        &mut serde_json::Value::Null,
    );
    (run_warnings, output)
}

/// Query metadata from the same loaded context (§FS-distribution.3.2.2.2).
pub(super) fn show_data(
    id: &str,
    opts: ShowOpts,
    explicit: bool,
) -> (Vec<Finding>, Result<(ShowOutput, serde_json::Value)>) {
    let mut warnings = Vec::new();
    let mut metadata = serde_json::Value::Null;
    let result = show_run(id, opts, explicit, &mut warnings, &mut metadata);
    (warnings, result.map(|out| (out, metadata)))
}

fn show_run(
    id_arg: &str,
    opts: ShowOpts,
    path_provided: bool,
    run_warnings: &mut Vec<Finding>,
    metadata: &mut serde_json::Value,
) -> Result<ShowOutput> {
    // §AR-config.1.5: a command-line run has no buffers; the editor's answer is
    // `show_from_snapshot`, over the context its snapshot loaded with them.
    // §AR-resolver.3.3: the ID resolves against the whole owning project.
    let context = load_reported_workspace_context(&opts.path, path_provided)?;
    *run_warnings = context_run_warnings(&context);
    show_answer(
        &context,
        id_arg,
        opts.section,
        opts.mode,
        opts.format,
        metadata,
    )
}

/// The editor's citation preview, answered from the context its snapshot was
/// loaded from rather than from a load of its own (§FS-lsp.responsiveness.3,
/// §AR-lsp.5.2): the same answer `grund <ID>` gives after its load, so the
/// bytes are the CLI's (§FS-lsp.1.2.1, §REQ-one-answer.text.parity).
pub fn show_from_snapshot(
    snapshot: &LspShowContext,
    id_arg: &str,
    mode: ShowMode,
    format: ShowFormat,
) -> Result<ShowOutput> {
    show_answer(
        &snapshot.context,
        id_arg,
        None,
        mode,
        format,
        &mut serde_json::Value::Null,
    )
}

/// The answer half of `grund <ID>` (§FS-show): resolves the argument against a
/// context already loaded, slices the body and renders its format. The CLI's
/// load and the editor's snapshot both hand their context here, so there is one
/// answer whichever surface asks (§AR-lsp.5.2).
fn show_answer(
    context: &WorkspaceContext,
    id_arg: &str,
    opts_section: Option<String>,
    mode: ShowMode,
    format: ShowFormat,
    metadata: &mut serde_json::Value,
) -> Result<ShowOutput> {
    let source = ShowSource::resolve(context, id_arg, opts_section)?;
    let ShowSource {
        project,
        id,
        section,
        ..
    } = &source;
    let config = &project.config;
    *metadata = crate::queries::show_metadata(config, &project.findings, id, section.as_deref());
    let (mut output, home) =
        source.read(context, mode.render_mode(), format == ShowFormat::Markdown)?;
    if format != ShowFormat::Markdown {
        // §FS-show.3.2.1, §FS-show.3.2.2: flatten ordinary prose while preserving
        // authored wrappers in Markdown fences, Markdown by the file's language.
        let reading = config.reading(&output.path);
        output.body = flatten_cross_ref_links(&output.body, reading.lexical, reading.markdown);
    }
    if format == ShowFormat::Json {
        let json = render_show_output_json(
            config,
            &project.findings.files,
            context.render_config(),
            home,
            id,
            section.as_deref(),
            mode.render_mode(),
            &output,
        );
        output.json = Some(json);
    }
    metadata["path"] = serde_json::json!(display_path(context.render_config(), &output.path));
    Ok(output)
}

/// The coordinate a `grund <ID>` argument names, resolved and not yet read
/// (§FS-show.2): `refs --from` resolves its source through the same steps, so a
/// source `show` would refuse is refused with the same bytes (§FS-refs.from).
pub(super) struct ShowSource<'c, 'a> {
    pub(super) project: &'c WorkspaceProject,
    pub(super) id: Id,
    pub(super) section: Option<String>,
    alias: Option<String>,
    raw_id: &'a str,
}

impl<'c, 'a> ShowSource<'c, 'a> {
    /// The project the argument's alias names, or the current one; its scan's
    /// first error; the ID under that project's grammar; and the section, inline or
    /// passed beside it but not both — refused in that order.
    pub(super) fn resolve(
        context: &'c WorkspaceContext,
        id_arg: &'a str,
        opts_section: Option<String>,
    ) -> Result<Self> {
        let (alias, raw_id) = split_qualified_id_arg(id_arg)?;
        let project = match alias.as_deref() {
            Some(name) => context
                .project_by_alias(name)
                .ok_or_else(|| {
                    if let Some(refusal) = context.missing_import(name) {
                        refusal
                    } else if !context.workspace_loaded {
                        anyhow!(crate::model::OperationDiagnostic::new("query", "unknown-project", format!(
                            "unknown project alias `{name}`\nnote: workspace aliases are defined in the root grund.toml under [workspace]"
                        )))
                    } else {
                        anyhow!(crate::model::OperationDiagnostic::new("query", "unknown-project", format!(
                            "unknown project alias `{name}`\nknown aliases: {}",
                            context.aliases().join(", ")
                        )))
                    }
                })?,
            None => context.current_project().ok_or_else(|| {
                let known = context.aliases().join(", ");
                if known.is_empty() {
                    anyhow!(crate::model::OperationDiagnostic::new("query", "query-failed", format!("unqualified ID requires a project alias when include_root = false")))
                } else {
                    anyhow!(crate::model::OperationDiagnostic::new("query", "query-failed", format!(
                        "unqualified ID requires a project alias when include_root = false\nknown aliases: {known}"
                    )))
                }
            })?,
        };
        // §FS-workspace.8.7.3: rendered against the run's config, not the target
        // project's — the same spelling `check` uses for the same tree.
        if let Some((file, message)) = project.scan_errors.first() {
            // §FS-distribution.3.3.2: scan failures are located filesystem outcomes.
            let path = display_path(context.render_config(), file);
            let mut failure = crate::model::OperationDiagnostic::new(
                "filesystem",
                "io",
                format!("{path}: {}", crate::scanner::scan_error_message(message)),
            );
            failure.path = Some(path);
            failure.details = serde_json::json!({"scan_errors": project.scan_errors.iter()
                .map(|(path,message)| serde_json::json!({"path": display_path(context.render_config(),path),"message":crate::scanner::scan_error_message(message)}))
                .collect::<Vec<_>>()});
            return Err(failure.into());
        }
        let config = &project.config;
        let (id, inline_section) =
            resolve_id_arg(raw_id, config.schema(), config.frame(), &project.findings)
                .map_err(|error| anyhow::Error::new(error.diagnostic()))?;
        if opts_section.is_some() && inline_section.is_some() {
            return Err(anyhow!(crate::model::OperationDiagnostic::new(
                "query",
                "query-failed",
                format!("--section cannot be combined with an inline section")
            )));
        }
        Ok(Self {
            project,
            id,
            section: opts_section.or(inline_section),
            alias,
            raw_id,
        })
    }

    /// The body in `mode` and the record it is read out of, a stub's paired
    /// declaration rather than the stub (§FS-show.2.3.7), or the refusal the
    /// coordinate earns: not found, ambiguous, a broken stub, or a section the
    /// record lacks or holds twice.
    pub(super) fn read(
        &self,
        context: &'c WorkspaceContext,
        mode: ShowRenderMode,
        include_heading: bool,
    ) -> Result<(ShowOutput, &'c Declaration)> {
        show_declaration_with_home(
            &self.project.config,
            context.render_config(),
            &self.project.findings,
            &self.id,
            Some(self.raw_id),
            self.section.as_deref(),
            mode,
            include_heading,
        )
        // §FS-workspace.8.1.1: a miss names the projects that do declare the ID,
        // read off the context this run already loaded.
        .map_err(|err| with_member_id_candidates(err, context, self.alias.as_deref(), self.raw_id))
    }
}
