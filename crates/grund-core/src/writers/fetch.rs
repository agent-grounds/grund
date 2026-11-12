//! The explicit external-snapshot materializer (§FS-fetch). This module owns the
//! order of a fetch and the integration's execution; every scanner and query
//! remains a reader under §REQ-runs-offline. Where the output goes is
//! `fetch_target.rs`, whether it is admitted `fetch_admit.rs` and
//! `fetch_readback.rs`, the splice `fetch_splice.rs`, and the atomic install it
//! ends in `fetch_write.rs`.

use std::path::Path;

use super::fetch_admit::admit;
use super::fetch_target::{Before, Home, Request, declared_elsewhere, place};
use super::fetch_write::install;
use crate::config::{
    KindConfig, ProjectRecords, RefusedHome, fetch_integration_path, fetch_names_nothing_reason,
    fetch_runs_nothing, run_warning_findings,
};
use crate::grammar::parse_id_arg;
use crate::model::Finding;
use crate::resolver::{fetch_place, load_alias_path, load_unscanned, settled_run_warnings};
use crate::scanner::{fetch_refused_home, scan_tree_with_workspace};
use crate::workspace::{resolve_workspace_config, split_qualified_id_arg};

/// Which public CLI exit class a fetch refusal belongs to (§FS-fetch.7).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FetchFailureKind {
    Query,
    Operational,
}

/// A located fetch refusal with its fixed exit class (§FS-fetch.7).
#[derive(Debug)]
#[non_exhaustive]
pub struct FetchFailure {
    pub kind: FetchFailureKind,
    pub message: String,
}

impl std::fmt::Display for FetchFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for FetchFailure {}

fn fetch_query(message: impl Into<String>) -> FetchFailure {
    FetchFailure {
        kind: FetchFailureKind::Query,
        message: message.into(),
    }
}

pub(super) fn fetch_operational(message: impl Into<String>) -> FetchFailure {
    FetchFailure {
        kind: FetchFailureKind::Operational,
        message: message.into(),
    }
}

/// Retain the source beside the unchanged fetch refusal (§FS-distribution.3.3.2).
pub(super) fn fetch_io(
    diagnostic: &mut Option<anyhow::Error>,
    path: &Path,
    error: std::io::Error,
    context: String,
) -> FetchFailure {
    let failure = fetch_operational(format!("{context}: {error}"));
    let source = crate::model::OperationDiagnostic::filesystem(path, &error, context);
    *diagnostic = Some(anyhow::Error::new(error).context(source));
    failure
}

/// Materialize exactly one local or qualified external ID (§FS-fetch.1).
pub fn fetch_snapshot(raw: &str, path: &Path) -> std::result::Result<(), FetchFailure> {
    fetch_snapshot_with_run_warnings(raw, path).1
}

/// [`fetch_snapshot`] for a frontend that also renders the run's `[workspace]`
/// warnings (§FS-check.4.10.11, §FS-workspace.6.1.7): `fetch` resolves
/// a block's member boundary like every other walking command, and its failure
/// is a typed refusal with nowhere to carry a caution.
#[doc(hidden)]
pub fn fetch_snapshot_with_run_warnings(
    raw: &str,
    path: &Path,
) -> (Vec<Finding>, std::result::Result<(), FetchFailure>) {
    let mut run_warnings = Vec::new();
    let result = fetch_run(raw, path, &mut run_warnings, &mut None);
    (run_warnings, result)
}

/// Structured source errors without changing FetchFailure (§FS-distribution.3.1).
pub(crate) fn fetch_with_diagnostics(raw: &str, path: &Path) -> (Vec<Finding>, anyhow::Result<()>) {
    let mut warnings = Vec::new();
    let mut diagnostic = None;
    let result = fetch_run(raw, path, &mut warnings, &mut diagnostic).map_err(|failure| {
        diagnostic.unwrap_or_else(|| {
            crate::model::OperationDiagnostic::new(
                match failure.kind {
                    FetchFailureKind::Query => "query",
                    FetchFailureKind::Operational => "operation",
                },
                "fetch",
                failure.message,
            )
            .into()
        })
    });
    (warnings, result)
}

fn fetch_run(
    raw: &str,
    path: &Path,
    run_warnings: &mut Vec<Finding>,
    diagnostic: &mut Option<anyhow::Error>,
) -> std::result::Result<(), FetchFailure> {
    let root_config = resolve_workspace_config(path).map_err(|err| {
        let message = format!("{err:#}");
        *diagnostic = Some(err);
        fetch_operational(message)
    })?;
    let report_base = root_config.display().base().to_path_buf();
    // §FS-fetch.1, §FS-workspace.8.11: the one alias-path splitter, so `fetch` refuses
    // a malformed path with the words `show` prints, still at the operational exit.
    let (namespace, local) =
        split_qualified_id_arg(raw).map_err(|err| fetch_operational(format!("{err:#}")))?;
    // §FS-imports.writes.answer: the ID query's selection, over the alias path alone.
    // §FS-workspace.1.2: the citation targets `check` scans the selected project with.
    let (selected, targets) = if let Some(namespace) = namespace.as_deref() {
        let context =
            load_alias_path(ProjectRecords::of(root_config), namespace).map_err(|err| {
                let message = format!("{err:#}");
                *diagnostic = Some(err);
                fetch_operational(message)
            })?;
        let config = context.render_config();
        *run_warnings = run_warning_findings(config, context.run_warnings.clone());
        let selected = context
            .project_at_alias(namespace)
            .map_err(|err| fetch_operational(format!("{err:#}")))?
            .config
            .clone();
        (selected, context.citation_targets())
    } else {
        *run_warnings = run_warning_findings(&root_config, settled_run_warnings(root_config.run()));
        let selected = ProjectRecords::of(root_config);
        // A member that does not load fails `check` before any scan, so it costs fetch nothing.
        let targets = if selected.project().workspace.declared {
            load_unscanned(selected.clone())
                .map(|context| context.citation_targets())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        (selected, targets)
    };

    let (id, section) =
        parse_id_arg(local, &selected.grammar).map_err(|err| fetch_query(format!("{err:#}")))?;
    if section.is_some() {
        return Err(fetch_query(format!("invalid ID `{raw}`")));
    }
    let kind = selected
        .kinds
        .iter()
        .find(|kind| kind.kind == id.kind)
        .ok_or_else(|| fetch_query(format!("invalid ID `{raw}`")))?;
    let fetch = kind.fetch.as_deref().ok_or_else(|| {
        fetch_operational(format!("kind `{}` has no fetch integration", kind.kind))
    })?;
    // The snapshot goes to the kind's first place on purpose: a fetched row's
    // several places are unspecified (§FS-config-v2.schema.places), one is the home.
    let home = match (&kind.file, &kind.folder) {
        (Some(file), None) => Home::File(selected.root.join(file)),
        (None, Some(folder)) => Home::Folder(selected.root.join(folder)),
        _ => {
            return Err(fetch_operational(format!(
                "kind `{}` cannot fetch without exactly one snapshot home",
                kind.kind
            )));
        }
    };
    // §FS-fetch.6.1: the one scan, after every refusal that needs none and before the spawn.
    let (catalog, errors) =
        scan_tree_with_workspace(selected.schema(), selected.frame(), None, false, &targets)
            .map_err(|err| {
                let message = format!("cannot discover declarations for {local}: {err:#}");
                *diagnostic = Some(err);
                fetch_operational(message)
            })?;
    let before = Before {
        catalog,
        errors,
        targets,
    };
    let request = Request {
        config: &selected,
        id: &id,
        local,
        report_base: &report_base,
        home,
    };
    refuse_home_without_markdown(&request, &before, kind)?;
    // §FS-imports.writes: asked last, so only a command that can run is named.
    let place_at = fetch_place(selected.run(), &report_base, namespace.as_deref(), local);
    if let Some(refusal) = place_at.import_refusal() {
        return Err(fetch_operational(refusal));
    }
    // §FS-fetch.6.1: where the output goes is the scan's answer, settled before the spawn and
    // refused, if it must be, once the output is known to be one declaration.
    let mut placing = None;
    let placed = place(&request, &before, &mut placing).map_err(|failure| (failure, placing));
    let integration = fetch_integration_path(&selected.root, fetch);

    // §FS-fetch.2.1: immediately before the spawn, behind every earlier refusal.
    if fetch_runs_nothing(fetch, &integration) {
        let reason = fetch_names_nothing_reason(&selected, &kind.kind, &report_base);
        return Err(fetch_operational(format!(
            "cannot run fetch integration `{fetch}` for {raw}: {reason}"
        )));
    }
    // §FS-fetch.5.3: a declaration outside either home stops the fetch before the spawn.
    declared_elsewhere(&request, &before)?;

    // §FS-fetch.2: direct argv invocation, inherited environment, no shell and
    // no implicit stdin. Stdout is data and is never forwarded to the caller.
    let output = std::process::Command::new(&integration)
        .arg(local)
        .current_dir(&selected.root)
        .output()
        .map_err(|err| {
            fetch_io(
                diagnostic,
                &integration,
                err,
                format!("cannot run fetch integration `{fetch}` for {raw}"),
            )
        })?;
    if !output.status.success() {
        let status = output
            .status
            .code()
            .map_or_else(|| "signal".to_string(), |code| code.to_string());
        return Err(fetch_operational(format!(
            "fetch integration `{fetch}` for {raw} exited {status}"
        )));
    }
    let snapshot = std::str::from_utf8(&output.stdout).map_err(|_| {
        fetch_operational(format!(
            "fetch integration `{fetch}` for {raw} emitted non-UTF-8 output"
        ))
    })?;
    // §FS-fetch.admission: installed only if the next scan reads it back as what was asked for.
    let admitted = admit(&request, &before, placed, snapshot, diagnostic)?;
    install(
        &admitted.install_at,
        &request.shown(&admitted.path),
        admitted.bytes.as_bytes(),
        admitted.expected(),
        diagnostic,
    )
}

/// §FS-fetch.4, §FS-fetch.4.2, §FS-fetch.4.2.1: a value source and a file read as source hold no
/// Markdown, and a home the scan does not read would never be read back; the answer is spelled
/// from the run's report base (§FS-errors.4), as `FetchPlace` spells its target. A value source's
/// way out names a folder without it, since one kept beside a fetched snapshot of its own ID is a
/// duplicate (§FS-values.2.3); a source file's moves its declarations too, since one left there
/// is what the scan goes on reading. A folder home is asked about the requested ID's `<ID>.md`
/// only where the scan read no file in the home declaring it (§FS-fetch.5.5).
fn refuse_home_without_markdown(
    request: &Request<'_>,
    before: &Before,
    kind: &KindConfig,
) -> std::result::Result<(), FetchFailure> {
    let selected = request.config;
    let claimed = before
        .catalog
        .declarations
        .get(request.id)
        .into_iter()
        .flatten()
        .any(|decl| !decl.is_stub && request.home.holds(&decl.file));
    let unclaimed = (!claimed).then_some(request.local);
    let Some((home, reason)) =
        fetch_refused_home(selected.schema(), selected.frame(), &kind.kind, unclaimed)
    else {
        return Ok(());
    };
    let shown = request.shown(&home);
    let why = match reason {
        RefusedHome::ValueSource => "a value source holds no Markdown; \
             move the kind to a Markdown file or a folder without it, or remove its `fetch`"
            .to_string(),
        RefusedHome::SourceFile | RefusedHome::SourceFileValue => {
            "a file read as source holds no Markdown; \
             move the kind and its declarations to a Markdown file or a folder, \
             or remove its `fetch`"
                .to_string()
        }
        // §FS-fetch.4.2.1: version 1 names no document language, so its repair is Markdown.
        RefusedHome::Unread(reason) => {
            let name = if selected.frame().version == 1 {
                "the scan reads as Markdown"
            } else {
                "a listed document language reads"
            };
            format!(
                "{reason}, so the scan would not read what fetch writes; give the file a name \
                 {name}, or remove its `fetch`"
            )
        }
        // §FS-fetch.4.2.1: the file's name is the ID's, so the repair is the rule.
        RefusedHome::Excluded(rule) => {
            let spell = |path: &Path| request.shown(path);
            format!(
                "{}, so the scan would not read what fetch writes; {}, and fetch again",
                rule.reason(&spell),
                rule.lift(&spell)
            )
        }
    };
    Err(fetch_operational(format!(
        "kind `{}` cannot fetch into {shown}: {why}",
        kind.kind
    )))
}
