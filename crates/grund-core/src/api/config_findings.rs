//! What a loaded config carries on its own (§AR-system.2.9): the redundant
//! discovery pair (§FS-check.4.3), the deprecated `.agents/` location
//! (§FS-check.4.11) and a file read as version 1
//! (§FS-check.checks.config-version-1), all known from the file the run read
//! rather than from the walk.
//!
//! They are data because the published `config_warnings` reads them
//! (§FS-config.4.1) and so does the run beside them. Neither prints: the
//! `warning:` line a process frontend puts them on belongs to `grund-cli`
//! (§AR-system.2.9.1, §AR-bindings.2).

use crate::checker::sort_diagnostics;
use crate::config::{Config, Display, config_file_under, home_form_of};
/// §FS-check.4.3: the warning for a config root holding both discovery names —
/// the bare `grund.toml` won, and the `.agents/grund.toml` beside it is read by
/// nothing (§FS-config.1.1). `line`-less, so it prints as a CLI-level `warning:`
/// on stderr: it says which file the run read, not what is wrong at a site.
use crate::model::remedies;
use crate::model::{Code, Diagnostic, format_path};

fn redundant_config_warning(config: &Config) -> Option<Diagnostic> {
    let ignored = config.redundant_config_file.as_ref()?;
    let winner = config.config_file.as_ref()?;
    Some(Diagnostic {
        code: Code::REDUNDANT_CONFIG,
        path: None,
        line: None,
        column: None,
        message: format!(
            "{} is ignored — {} takes precedence; {}",
            format_path(ignored),
            format_path(winner),
            remedies::redundant_config()
        ),
        sites: Vec::new(),
        authority: Vec::new(),
    })
}

/// §FS-check.4.11: the warning for a config the run read from the deprecated
/// `.agents/` location — the file still governs the project, so the message
/// names the move a reader can type rather than a fault (§FS-config.1.2.1).
/// `line`-less for §FS-check.4.3's reason, which this finding shares whole: the subject
/// is which file the run read, not a site inside it.
///
/// Keyed off the file actually read, which is what excludes the redundant pair
/// by construction: there the bare name won the tie, so `config_file` already
/// names the home path and §FS-check.4.3 is the finding the directory earns
/// (§FS-config.1.1).
fn deprecated_config_location_warning(config: &Config) -> Option<Diagnostic> {
    let found = config.config_file.as_ref()?;
    let home = home_form_of(found)?;
    Some(Diagnostic {
        code: Code::DEPRECATED_CONFIG_LOCATION,
        path: None,
        line: None,
        column: None,
        message: format!(
            "{} is a deprecated config location — {}",
            format_path(found),
            remedies::deprecated_config_location(format_path(&home))
        ),
        sites: Vec::new(),
        authority: Vec::new(),
    })
}

/// §FS-check.checks.config-version-1: the caution's code, which the migrate report
/// gate drops from both runs it compares (§FS-config-migrate.5.2).
pub(super) const CONFIG_VERSION_1: Code = Code::CONFIG_VERSION_1;

/// §FS-check.checks.config-version-1: the one sentence that tells a reader the
/// config at `file` is on version 1 and names the command that migrates it,
/// ending at `--to 2` when `dir` is the report's base. `config migrate` prints
/// it for a member (§FS-config-migrate.8) and the caution below prints it for
/// any config, so the two cannot drift apart.
pub(super) fn version_1_caution(file: &str, dir: Option<&str>) -> String {
    let mut line = format!("{file} is version 1; {}", remedies::config_version_1("it"));
    if let Some(dir) = dir.filter(|dir| !dir.is_empty()) {
        line.push(' ');
        line.push_str(dir);
    }
    line
}

/// §FS-check.checks.config-version-1, §FS-config.5.3: the caution a config file
/// read as version 1 earns, whether it wrote `1` or no version. The reader the
/// file chose already recorded its version on the project (§FS-config.5), so
/// nothing rereads the file; a zero-config project also loads v1's defaults but
/// has no `config_file`, and no file means no caution (§GOAL-zero-config).
///
/// The file and the project root it configures are both report paths, spelled
/// by `display`, the report's own (§FS-config.3.6.1): in a workspace that is the
/// root's, not the member's (§FS-workspace.8.1).
fn config_version_1_warning(config: &Config, display: Display<'_>) -> Option<Diagnostic> {
    let read = config.config_file.as_ref()?;
    if config.project().version != 1 {
        return None;
    }
    let file = config_file_under(&config.root, read);
    Some(Diagnostic {
        code: CONFIG_VERSION_1,
        path: None,
        line: None,
        column: None,
        message: version_1_caution(&display.path(&file), Some(&display.path(&config.root))),
        sites: Vec::new(),
        authority: Vec::new(),
    })
}

/// §FS-config-v2.checks.config-md-not-markdown: each warning the version-2 reader made of the
/// file it loaded, at its config line, behind the `<config>:<line>:` breadcrumb
/// (§FS-errors.2.2.1) that JSON carries as `path` and `line` too (§FS-errors.5.2.3). Version 1
/// makes none (§FS-config.5.2).
pub(super) fn load_warnings(config: &Config, display: Display<'_>) -> Vec<Diagnostic> {
    let warnings = &config.project().load_warnings.0;
    let warnings = warnings.iter().map(|warning| {
        let file = config_file_under(&config.root, &warning.at.path);
        Diagnostic {
            code: warning.code,
            message: format!(
                "{}:{}: {}",
                display.path(&file),
                warning.at.line,
                warning.message
            ),
            path: Some(file),
            line: Some(warning.at.line),
            column: None,
            sites: Vec::new(),
            authority: Vec::new(),
        }
    });
    warnings.collect()
}

/// The findings a loaded config carries on its own — the redundant discovery
/// pair (§FS-check.4.3), the deprecated `.agents/` location (§FS-check.4.11),
/// a file read as version 1 (§FS-check.checks.config-version-1) and the version-2
/// reader's load warnings (§FS-config-v2.checks), in that order, which the report
/// then sorts (§FS-check.checks.config-version-1.1). All are known from the file the
/// run read rather than from the walk, which is why they arrive together. The
/// version-2 reader's window warnings are spent from grund 0.22.0
/// (§FS-config-v2.checks.config-not-toml).
///
/// One list rather than a call each at each site: `grund check` and `grund config
/// validate` name them per project ([`workspace_config_diagnostics`]) and `grund
/// config show` names them for the one config it prints, and a finding that reached only some of those surfaces would be a
/// finding a repository can hide from by choosing a command.
pub(super) fn config_diagnostics<'a>(
    config: &'a Config,
    display: Display<'_>,
) -> impl Iterator<Item = Diagnostic> + 'a {
    redundant_config_warning(config)
        .into_iter()
        .chain(deprecated_config_location_warning(config))
        .chain(config_version_1_warning(config, display))
        .chain(load_warnings(config, display))
}

/// The config findings a workspace names (§FS-check.4.11.3): the root's config,
/// each project the workspace expansion returns, a member that is the root itself
/// counted once, and each `include_root = false` block the expansion reached, which
/// expansion keeps on the root's run since it returns no project for one
/// (§AR-config.1.5). All are spelled from the root's report base (§FS-errors.4) and
/// in check's report order. One config, not one scope, and a workspace may mix the
/// two discovery forms (§FS-workspace.2). `grund check` and `grund config validate`
/// both name the set from here (§FS-config.4.1.1), so the two cannot disagree about
/// which configs a workspace has.
pub(super) fn workspace_config_diagnostics<'a>(
    root_config: &Config,
    projects: impl IntoIterator<Item = &'a Config>,
) -> Vec<Diagnostic> {
    let display = root_config.display();
    let mut findings = config_diagnostics(root_config, display).collect::<Vec<_>>();
    for project in projects {
        if project.root != root_config.root {
            findings.extend(config_diagnostics(project, display));
        }
    }
    for block in root_config.workspace_opted_out_blocks() {
        findings.extend(config_diagnostics(block, display));
    }
    sort_diagnostics(&mut findings);
    findings
}

/// The config findings the config a command holds names: the workspace's set
/// where `grund config validate` stored one (§FS-config.4.1.1), else its own
/// config's alone, which is all `grund config show` names (§FS-config.4.2).
pub(super) fn held_config_diagnostics(config: &Config) -> Vec<Diagnostic> {
    match config.workspace_config_findings() {
        Some(findings) => findings.to_vec(),
        None => config_diagnostics(config, config.display()).collect(),
    }
}
