//! What a loaded config carries on its own (§AR-system.2.9): the redundant
//! discovery pair (§FS-check.4.3), the deprecated `.agents/` location
//! (§FS-check.4.11), a file read as version 1
//! (§FS-check.checks.config-version-1), a version-2 array read across a
//! string open at a line's end, and a version-2 string
//! read past its closing quote (§FS-config-v2.checks.config-not-toml,
//! §FS-config-v2.checks.config-multiline-string), all known from the file the
//! run read rather than from the walk.
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

/// §FS-config-v2.reader.5.strings.window, §FS-config-v2.reader.6.window: the clause each
/// window's warning ends with, naming the release from which the value is refused instead
/// (§FS-config-v2.reader.5.strings.refusal, §FS-config-v2.reader.6.refusal).
const STRINGS_WINDOW_CLAUSE: &str = "this warning becomes an error in grund 0.22.0";

/// §FS-config-v2.checks.config-not-toml, §FS-config-v2.checks.config-multiline-string:
/// one warning per array the version-2 reader read across a string still open at a
/// line's end. Its message is the refusal the array becomes, what it is read as
/// until then, the remedy and the release (§REQ-backwards-compatibility.2), behind
/// the `<config>:<line>:` breadcrumb that refusal prints (§FS-errors.2.2.1). It
/// anchors at the same key's line, so JSON carries the line too.
fn string_over_break_warnings(config: &Config) -> impl Iterator<Item = Diagnostic> + '_ {
    config.project().strings_over_breaks.iter().map(|over| {
        let (at, open) = (&over.at, over.open);
        let (code, remedy) = if open.multi_line {
            let remedy = remedies::config_multiline_string(open.line);
            (Code::CONFIG_MULTILINE_STRING, remedy)
        } else {
            let remedy = remedies::config_not_toml_open_line(open.line);
            (Code::CONFIG_NOT_TOML, remedy)
        };
        Diagnostic {
            code,
            path: Some(at.path.clone()),
            line: Some(at.line),
            column: None,
            message: format!(
                "{}:{}: unclosed array{}, read for now with the line break as a space; \
                 {remedy}; {STRINGS_WINDOW_CLAUSE}",
                format_path(&at.path),
                at.line,
                open.parenthetical()
            ),
            sites: Vec::new(),
            authority: Vec::new(),
        }
    })
}

/// §FS-config-v2.checks.config-not-toml, §FS-config-v2.checks.config-multiline-string:
/// one warning per value, and per quoted key, the version-2 reader read past a string's
/// closing quote (§FS-config-v2.reader.6.window), or whose `"""` string it read on one
/// line (§FS-config-v2.reader.6.multi-line). Its message is the refusal the value
/// becomes, the string as it is read until then, the remedy for the line the string
/// closes on and the release, behind the `<config>:<line>:` breadcrumb of the key's
/// line, which JSON carries too. Pushed after an array's open-line warning, as the
/// reader meets the two (§FS-config-v2.reader.6.window).
fn string_closes_early_warnings(config: &Config) -> impl Iterator<Item = Diagnostic> + '_ {
    config.project().strings_closing_early.iter().map(|early| {
        let (at, text, line) = (&early.at, &early.text, early.line);
        let (code, refusal, read, remedy) = if early.multi_line {
            let refusal = " (a multi-line string, which version 2 does not read)";
            let read = format!("`{text}`, its quotes kept");
            let remedy = remedies::config_multiline_string(line);
            (Code::CONFIG_MULTILINE_STRING, refusal, read, remedy)
        } else {
            let read = format!("the one string `{text}`");
            let remedy = if early.element {
                remedies::config_not_toml_element(line)
            } else {
                remedies::config_not_toml_scalar_or_key(line)
            };
            (Code::CONFIG_NOT_TOML, "", read, remedy)
        };
        Diagnostic {
            code,
            path: Some(at.path.clone()),
            line: Some(at.line),
            column: None,
            message: format!(
                "{}:{}: expected string{refusal}, read for now as {read}; {remedy}; \
                 {STRINGS_WINDOW_CLAUSE}",
                format_path(&at.path),
                at.line,
            ),
            sites: Vec::new(),
            authority: Vec::new(),
        }
    })
}

/// The findings a loaded config carries on its own — the redundant discovery
/// pair (§FS-check.4.3), the deprecated `.agents/` location (§FS-check.4.11),
/// a file read as version 1 (§FS-check.checks.config-version-1) and the
/// version-2 window's warnings (§FS-config-v2.checks.config-not-toml), in that
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
        .chain(string_over_break_warnings(config))
        .chain(string_closes_early_warnings(config))
}
