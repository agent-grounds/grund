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

/// The findings a loaded config carries on its own — the redundant discovery
/// pair (§FS-check.4.3), the deprecated `.agents/` location (§FS-check.4.11)
/// and a file read as version 1 (§FS-check.checks.config-version-1), in that
/// order, which the report then sorts (§FS-check.checks.config-version-1.1). All
/// are known from the file the run read rather than from the walk, which is why
/// they arrive together.
///
/// One list rather than a call each at each site: `grund check` names them per
/// project and `grund config validate` and `grund config show` name them for
/// one, and a finding that reached only some of those surfaces would be a
/// finding a repository can hide from by choosing a command.
pub(super) fn config_diagnostics<'a>(
    config: &'a Config,
    display: Display<'_>,
) -> impl Iterator<Item = Diagnostic> + 'a {
    redundant_config_warning(config)
        .into_iter()
        .chain(deprecated_config_location_warning(config))
        .chain(config_version_1_warning(config, display))
}
