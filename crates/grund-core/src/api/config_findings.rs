//! What a loaded config carries on its own (§AR-system.2.9): the redundant
//! discovery pair (§FS-check.4.3) and the deprecated `.agents/` location
//! (§FS-check.4.11), both known from the file the run read rather than from the
//! walk.
//!
//! They left the deprecated path's `output` category because the published
//! `config_warnings` reads them (§FS-config.4.1) and so does the run beside them,
//! which had the api reading a renderer (§AR-system.4). Neither prints: the
//! `warning:` line a `config` frontend puts them on is `compat/output.rs`'s and
//! `grund-cli`'s own.

/// §FS-check.4.3: the warning for a config root holding both discovery names —
/// the bare `grund.toml` won, and the `.agents/grund.toml` beside it is read by
/// nothing (§FS-config.1.1). `line`-less, so it prints as a CLI-level `warning:`
/// on stderr: it says which file the run read, not what is wrong at a site.
use crate::config::{Config, home_form_of};
use crate::model::{Diagnostic, format_path};

fn redundant_config_warning(config: &Config) -> Option<Diagnostic> {
    let ignored = config.redundant_config_file.as_ref()?;
    let winner = config.config_file.as_ref()?;
    Some(Diagnostic {
        code: "redundant-config",
        path: None,
        line: None,
        column: None,
        message: format!(
            "{} is ignored — {} takes precedence; delete one",
            format_path(ignored),
            format_path(winner)
        ),
        sites: Vec::new(),
    })
}

/// §FS-check.4.11: the warning for a config the run read from the deprecated
/// `.agents/` location — the file still governs the project, so the message
/// names the move a reader can type rather than a fault (§FS-config.1.2).
/// `line`-less for §4.3's reason, which this finding shares whole: the subject
/// is which file the run read, not a site inside it.
///
/// Keyed off the file actually read, which is what excludes the redundant pair
/// by construction: there the bare name won the tie, so `config_file` already
/// names the home path and §4.3 is the finding the directory earns
/// (§FS-config.1.1).
fn deprecated_config_location_warning(config: &Config) -> Option<Diagnostic> {
    let found = config.config_file.as_ref()?;
    let home = home_form_of(found)?;
    Some(Diagnostic {
        code: "deprecated-config-location",
        path: None,
        line: None,
        column: None,
        message: format!(
            "{} is a deprecated config location — move it to {}",
            format_path(found),
            format_path(&home)
        ),
        sites: Vec::new(),
    })
}

/// The findings a loaded config carries on its own — the redundant discovery
/// pair (§FS-check.4.3) and the deprecated `.agents/` location (§FS-check.4.11).
/// Both are known from the file the run read rather than from the walk, which is
/// why they arrive together.
///
/// One list rather than a call each at each site: `grund check` names them per
/// project and `grund config validate` and `grund config show` name them for
/// one, and a finding that reached only some of those surfaces would be a
/// finding a repository can hide from by choosing a command.
pub(super) fn config_diagnostics(config: &Config) -> impl Iterator<Item = Diagnostic> {
    redundant_config_warning(config)
        .into_iter()
        .chain(deprecated_config_location_warning(config))
}
