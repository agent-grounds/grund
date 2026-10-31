//! Config *discovery*: which file governs a directory, and where the walk stops
//! (§FS-config.1, §DF-config-file-location). Kept apart from the reader in
//! `v1/parse.rs` because the two answer different questions — "which file" versus
//! "what does this file say" — and only this half knows there are two names
//! (§AR-core-module-layout.1).

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use super::compiled::{Compiled, compile};
use super::project::Project;
use super::record::Config;
use super::run::{Invocation, Run};
use super::{v1, v2, validate};
use crate::model::{format_path, relative_from_base};

/// The two names one directory may hold its config under, in probe order
/// (§FS-config.1): the bare root-visible `grund.toml` first, then
/// `.agents/grund.toml`. Fixed order, not a search — §DF-config-file-location.2.2
/// gives the tie to the form `grund init` generates, so the file a project is
/// told to write is the file that governs it.
const CONFIG_NAMES: [&[&str]; 2] = [&["grund.toml"], &[".agents", "grund.toml"]];

/// §FS-declarations.checks.misplaced-declaration.unlisted-project: an unread
/// config's optional identity, using the same top-level/string grammar as both
/// readers. No project load, regex compilation, or refusal for malformed input,
/// and no finding: a version-2 name is read as the load reads it, and one the load
/// refuses, a string that closes early among them, is passed over
/// (§FS-config-v2.reader.6.refusal).
pub(crate) fn unread_project_name(text: &str, path: &Path) -> Option<String> {
    let version_2 = v2::selects(text);
    let mut name = None;
    for (idx, raw) in text.lines().enumerate() {
        let line = v1::strip_comment(raw).trim();
        if line.starts_with('[') {
            break;
        }
        if let Some((key, value)) = line.split_once('=')
            && key.trim() == "project_name"
        {
            if name.is_some() {
                return None;
            }
            let (line, value) = (idx + 1, value.trim());
            let value = match version_2 {
                true => v2::read_string(path, line, value),
                false => v1::parse_string(path, line, value),
            }
            .ok()?;
            if value.is_empty() || value.chars().any(char::is_control) {
                return None;
            }
            name = Some(value);
        }
    }
    name
}

/// A string list read by a reader that reads one key of a config without loading it: as
/// the load of a version-2 file reads it when `version_2`, of a version-1 file otherwise,
/// `None` for a value the load refuses, and no finding recorded
/// (§FS-config-v2.reader.6.refusal). The ancestor climb reads `[workspace] members`
/// through this (§FS-workspace.6.1.7.5).
pub(crate) fn unloaded_string_list(
    version_2: bool,
    path: &Path,
    line: usize,
    value: &str,
) -> Option<Vec<String>> {
    if version_2 {
        // Only `Ok` and `Err` are read here, so the refusal's 0.22.0 clause is moot.
        v2::read_string_list(path, line, value, false).ok()
    } else {
        v1::parse_string_list(path, line, value).ok()
    }
}

/// Every config name `dir` actually carries, in precedence order (§FS-config.1).
///
/// `is_file()` rather than `exists()`, because with two names the test decides
/// *precedence*, not merely presence: a directory named `grund.toml` must not
/// outrank a real `.agents/grund.toml` beside it, which is what a bare
/// existence test would do. The one case that changes is such a directory,
/// which is now climbed past instead of reaching the parser as an unreadable
/// file — a dangling symlink at either name was already a miss under both
/// tests, since `exists()` follows symlinks too.
fn config_files_in(dir: &Path) -> impl Iterator<Item = PathBuf> + use<'_> {
    // §FS-check.6.1.1: precedence probes are inputs, including absent names.
    super::observe_config_candidates(dir);
    CONFIG_NAMES
        .iter()
        .map(|segments| {
            segments
                .iter()
                .fold(dir.to_path_buf(), |acc, s| acc.join(s))
        })
        .filter(|candidate| candidate.is_file())
}

/// The config file `dir` carries, or `None` — the one probe every discovery site
/// funnels through, so root and workspace member ask the same question
/// (§FS-config.1).
pub(crate) fn config_file_in(dir: &Path) -> Option<PathBuf> {
    config_files_in(dir).next()
}

/// The config `dir` ignores because the higher-precedence name outranks it
/// (§FS-config.1.1) — `Some` only when the directory carries both names, which is
/// the redundant pair `check` warns about (§FS-check.4.3).
fn redundant_config_file_in(dir: &Path) -> Option<PathBuf> {
    config_files_in(dir).nth(1)
}

/// Where a config read from the deprecated `.agents/` location belongs — the
/// bare `grund.toml` beside the directory that holds it — or `None` when the
/// file the run read is the home form already (§FS-config.1.2).
///
/// Here rather than beside the message it feeds (§FS-check.4.11) because only
/// this module knows there are two names (§AR-core-module-layout.1), and the
/// answer is a fact about the pair of names rather than about how it reads. The
/// directory two components up is the config root, which the move leaves where
/// it is: relative paths never resolved against `.agents/` (§FS-config.1).
pub(crate) fn home_form_of(config_file: &Path) -> Option<PathBuf> {
    let deprecated: PathBuf = CONFIG_NAMES[1].iter().collect();
    if !config_file.ends_with(deprecated) {
        return None;
    }
    let home: PathBuf = CONFIG_NAMES[0].iter().collect();
    Some(config_file.parent()?.parent()?.join(home))
}

/// The config file `root` read, given `read`, the spelling discovery recorded:
/// the name it ends in, joined to the root (§FS-config.1). Discovery spells
/// `read` before `[output] relative_paths` is known, so a message that names the
/// file re-spells this path the way the report spells every other
/// (§FS-config.3.6.1).
pub(crate) fn config_file_under(root: &Path, read: &Path) -> PathBuf {
    let name = if home_form_of(read).is_some() {
        CONFIG_NAMES[1]
    } else {
        CONFIG_NAMES[0]
    };
    name.iter()
        .fold(root.to_path_buf(), |path, part| path.join(part))
}

/// The directory a path argument starts discovery from: a file's parent
/// directory, and the working directory for a file named without a directory
/// part, whose `parent()` is the empty path rather than `None` (§FS-config.1.4);
/// any other path is its own start. Uncanonicalized, so each caller resolves
/// `.` the way it resolves every other relative start.
pub(crate) fn discovery_start_dir(path: &Path) -> &Path {
    if !path.is_file() {
        return path;
    }
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    }
}

/// Discover and load the effective config: walk upward from `start` for the
/// nearest directory carrying either config name (§FS-config.1), parse it over
/// the defaults (§FS-config.principle.unit), or fall back to the pure defaults
/// if none is found (§GOAL-zero-config).
///
/// Why the fallback root is the working directory: `[scan] include` must resolve
/// against the repository, so `grund check src/` scopes *into* `src/` instead of
/// looking for `src/docs`, `src/e2e`, `src/src`.
///
/// The run it builds is `invocation`'s (§AR-config.1.5), and only `open()` in the
/// workspace component asks.
pub(crate) fn discover_config(invocation: &Invocation) -> Result<Config> {
    let start = invocation.path.as_path();
    // §FS-check.6.1.1, §FS-check.6.1.3: cover the lexical invocation and its
    // followed targets before type probes or canonical discovery lose the link.
    anyhow::ensure!(
        super::observe_input(start, false),
        "watch input coverage failed"
    );
    let start_dir = discovery_start_dir(start).to_path_buf();
    // Resolve to an absolute path before walking up, mirroring how `cargo` finds
    // `Cargo.toml` (§FS-config.1): a relative `.` or `subdir/` must still discover
    // a `grund.toml` in an ancestor directory.
    let walk_start = fs::canonicalize(&start_dir).unwrap_or(start_dir);
    let mut cursor = Some(walk_start.as_path());
    while let Some(dir) = cursor {
        // Both names are probed at *every* level before climbing, so a bare
        // member config shadows an ancestor's `.agents/` one exactly as a nested
        // `.agents/grund.toml` always did (§FS-config.1, §DF-config-file-location.2.1).
        if config_file_in(dir).is_some() {
            return config_at(dir, &walk_start, None, invocation);
        }
        cursor = dir.parent();
    }
    // Zero-config (§GOAL-zero-config): the "project root" is the current working
    // directory, never the path passed on the command line. Reports stay relative to
    // `cli_base` (the resolved path arg) when `relative_paths` is off (§FS-config.3.6.1).

    // §FS-distribution.3.3.3: embedding scopes this fallback to its supplied root.
    // §FS-lsp.2.2.1: an editor's explicit zero-config folder is the project anchor
    // even when the server process started elsewhere.
    let root = match super::call_scope::embedding_base() {
        _ if invocation.anchors_zero_config() => crate::sources::canonical_snapshot_path(start),
        Some(base) => fs::canonicalize(&base).unwrap_or(base),
        None => std::env::current_dir()
            .ok()
            .and_then(|cwd| fs::canonicalize(&cwd).ok())
            .unwrap_or_else(|| walk_start.clone()),
    };
    let mut run = Run::of(root, walk_start, invocation);
    // §FS-check.2.2.2.5: the one place a run learns that no config applies to it.
    run.found_no_config = true;
    let config = defaults_under(run, invocation);
    // §FS-check.6.1.3: zero-config still has effective roots and probes.
    super::observe_config(&config);
    Ok(config)
}

/// Load the config rooted at `root` (no upward walk), using `cli_base` for
/// path rendering, under `invocation` (§AR-config.1.5). The one shared loader
/// both upward discovery (`discover_config`) and direct workspace-member
/// loading funnel through (§AR-workspace.5.1).
///
/// `report_base` is the base config-error paths are rendered from.
pub(crate) fn config_at(
    root: &Path,
    cli_base: &Path,
    report_base: Option<&Path>,
    invocation: &Invocation,
) -> Result<Config> {
    let root = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let candidate = config_file_in(&root);
    let mut run = Run::of(root.clone(), cli_base.to_path_buf(), invocation);
    // Report config errors against a stable relative path, never the absolute
    // discovered path (§FS-errors.4: deterministic, no absolute paths outside the
    // configured root).

    // §FS-cli.3.4: under `--path-base=invocation` every location the load records,
    // and so every config-load error, is spelled from the CLI base.
    let from_cli_base = run.path_base == Some(super::call_scope::PathBase::Invocation);
    let spelled_from = if from_cli_base {
        Some(cli_base)
    } else {
        report_base
    };
    let report_relative = |path: &Path| match spelled_from {
        Some(base) => relative_from_base(base, path),
        None => path
            .strip_prefix(&root)
            .map(Path::to_path_buf)
            .unwrap_or_else(|_| path.to_path_buf()),
    };
    run.location_base = spelled_from.map_or_else(|| root.clone(), Path::to_path_buf);
    // §FS-check.4.3.2: the loser of a two-name tie is recorded, not read, so every
    // surface that reports on the config can name the file grund ignored.
    run.redundant_config_file = redundant_config_file_in(&root).map(|path| report_relative(&path));
    let config = match candidate {
        None => defaults_under(run, invocation),
        Some(candidate) => {
            let report_path = report_relative(&candidate);
            run.config_file = Some(report_path.clone());
            // §FS-distribution.3.3.2: classify all config-load failures at discovery.
            // §FS-lsp.2.2.4.1: and name the file it failed on, which an editor marks.
            read_config(&candidate, &report_path, &root, run, invocation).map_err(|mut error| {
                if let Some(diagnostic) = error.downcast_mut::<crate::model::OperationDiagnostic>()
                {
                    diagnostic.file = Some(candidate.clone());
                    error
                } else {
                    let mut diagnostic =
                        crate::model::OperationDiagnostic::from_error("config", "config", error);
                    diagnostic.path = Some(format_path(&report_path));
                    diagnostic.file = Some(candidate.clone());
                    diagnostic.into()
                }
            })?
        }
    };
    // §FS-check.6.1.1: expose coverage before member expansion/checker reads.
    super::observe_config(&config);
    Ok(config)
}

/// The zero-config façade under `run` (§GOAL-zero-config): the v1 default
/// project, which no file narrowed (§AR-config.3.2).
fn defaults_under(run: Run, invocation: &Invocation) -> Config {
    let project = v1::default_project();
    let compiled = compile(&project).expect("default grammar must compile");
    settled(&project, &compiled, run, invocation)
}

/// The façade over `project` under `run`, its one fact that needs the project
/// settled first: whether the walk reads the ordinary roots beside the path,
/// which a `[workspace]` block decides for the editor (§AR-config.1.5,
/// §AR-lsp.5.1.1). Built once; nothing sets a scope field after.
fn settled(
    project: &Project,
    compiled: &Compiled,
    mut run: Run,
    invocation: &Invocation,
) -> Config {
    run.scope.resolution_wide = invocation.resolution_wide(&run.root, project.workspace.declared);
    Config::from_records(project, &run, compiled)
}

/// Read one config file into the façade (§AR-config.2): lowered by the reader
/// of the version that spelled it, judged once (§AR-config.4), and compiled
/// (§AR-config.1.5). `report_path` is the path every error names.
///
/// The file's own `grund_config_version` selects its reader (§FS-config.5):
/// an explicit `2` the v2 reader, anything else v1, which refuses a version it
/// does not know. The steps interleave in v1's order, so a file with several
/// errors reports the first one it always did (§AR-config.4): the `[reference]`
/// meanings, then the `[[kinds]]` refusals and the kind table, then the grammar,
/// then the member lists and `[citations]`. The v2 reader has refused every
/// spelling before the shared judgement starts (§FS-config-v2.reader.4).
fn read_config(
    read_path: &Path,
    report_path: &Path,
    root: &Path,
    run: Run,
    invocation: &Invocation,
) -> Result<Config> {
    // §FS-config-migrate.5.2: the report gate's candidate, read in the file's place.
    let text = match super::call_scope::config_text_override(read_path) {
        Some(text) => text,
        // §FS-check.6.1.1: cover this effective input before its shared read.
        None => super::input_read_to_string(read_path)
            .with_context(|| format!("read {}", format_path(report_path)))?,
    };
    config_from_text(&text, report_path, root, run, invocation)
}

/// The façade the config `grund init` is about to write at `root` will load
/// to, before it is on disk: `text` read as `root`'s `grund.toml` by the reader
/// a discovered file goes through, so the managed block `init` renders describes
/// exactly the file it writes, a kind with no built-in default included
/// (§FS-init.2.3.8, §FS-init.2.4.10).
pub(crate) fn pending_config(text: &str, root: &Path, invocation: &Invocation) -> Result<Config> {
    let run = Run::of(root.to_path_buf(), root.to_path_buf(), invocation);
    config_from_text(text, Path::new("grund.toml"), root, run, invocation)
}

/// The façade `text` loads to as the config at `root`, judged and compiled as
/// a file read from disk is (§AR-config.2).
fn config_from_text(
    text: &str,
    report_path: &Path,
    root: &Path,
    run: Run,
    invocation: &Invocation,
) -> Result<Config> {
    let project = if v2::selects(text) {
        let project = v2::read(text, report_path, root)?;
        validate::reference(report_path, &project)?;
        project
    } else {
        let read = v1::read_sections(text, report_path)?;
        validate::reference(report_path, &read.project)?;
        read.lower_kinds(report_path, root)?
    };
    validate::kinds(report_path, &project)?;
    let compiled = compile(&project)
        .with_context(|| format!("{}: invalid [id] grammar", format_path(report_path)))?;
    validate::lists(report_path, &project)?;
    Ok(settled(&project, &compiled, run, invocation))
}

/// Discovery from `start` as a read-only command makes it: the loader the unit
/// tests state a project through (§AR-config.1.5).
#[cfg(test)]
pub(crate) fn load_config(start: &Path) -> Result<Config> {
    discover_config(&Invocation::new(
        start,
        true,
        super::run::Selection::Discovery,
    ))
}

/// `config_at` with no report base, as a read-only command loads a member.
#[cfg(test)]
pub(crate) fn load_config_at(root: &Path, cli_base: &Path) -> Result<Config> {
    let invocation = Invocation::new(root, false, super::run::Selection::Project);
    config_at(root, cli_base, None, &invocation)
}
