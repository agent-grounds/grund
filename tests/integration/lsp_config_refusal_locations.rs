//! Real config-load refusals retain their named lines through publication
//! (§FS-lsp.2.2.4.1), with CLI message agreement (§FS-lsp.1.1). Exercise the
//! shared core/transport boundary (§AR-lsp.2), not prelocated mock diagnostics.
//! Fixture owners remove their roots after all consumers (§FS-repository-maintenance.4.1).

#[path = "binaries.rs"]
mod binaries;
#[path = "lsp_import_session.rs"]
mod lsp_import_session;
#[path = "../../crates/grund-lsp/tests/support/mod.rs"]
mod support;

use lsp_import_session::{Composition, DEPENDENCY_CONFIG, Session};
use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::process::Command;
use support::*;

const V2: &str = "grund_config_version = 2\nproject_name = \"example\"\n\
    [schema.sources]\nlanguages = [\"markdown\"]\n";
const V1: &str = "grund_config_version = 1\nproject_name = \"example\"\n";

fn value_config(home: &str) -> String {
    format!("{V2}[schema.kinds.CONST]\nfolders = [\"{home}\"]\nindex = false\nform = \"value\"\n")
}

/// The CLI is also a guard against testing an unrelated earlier refusal.
fn cli_error(root: &Path, expected: &str) -> String {
    let output = Command::new(binaries::grund())
        .args(["check", "."])
        .current_dir(root)
        .output()
        .expect("run grund check");
    assert_eq!(output.status.code(), Some(2), "a config-load refusal");
    let stderr = String::from_utf8(output.stderr).expect("CLI stderr");
    let message = stderr
        .trim_end()
        .strip_prefix("error: ")
        .expect("CLI error");
    assert!(
        message.contains(expected),
        "expected {expected:?}: {message}"
    );
    message.to_string()
}

fn named_line(message: &str) -> u64 {
    let line = message
        .strip_prefix("grund.toml:")
        .and_then(|rest| rest.split_once(':'))
        .and_then(|(line, _)| line.parse::<u64>().ok())
        .expect("the actual CLI error must name its config line");
    assert!(
        line > 1,
        "the fixture must distinguish the fallback: {message}"
    );
    line - 1
}

fn assert_refusal(diagnostics: &[Value], cli: &str, line: u64) {
    assert_eq!(diagnostics.len(), 1, "one config error: {diagnostics:?}");
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic["message"], cli, "unchanged CLI message");
    assert_eq!(diagnostic["severity"], 1);
    assert_eq!(diagnostic["source"], "grund");
    assert_eq!(diagnostic["code"], "config");
    assert_eq!(
        diagnostic["range"]["start"]["line"], line,
        "config refusal must mark the line its CLI message names: {cli}"
    );
    assert_eq!(diagnostic["range"]["end"]["line"], line);
}

fn startup(config: &str, expected: &str, located: bool, setup: impl FnOnce(&Path)) {
    let _ = SERVER_BINARY.set(binaries::grund_lsp());
    let root = test_root("1150-config-location");
    fs::write(root.join("grund.toml"), config).expect("write config");
    setup(&root);
    let cli = cli_error(&root, expected);
    let line = if located { named_line(&cli) } else { 0 };
    let (mut child, mut stdin, receiver) = start_server(&root);
    let diagnostics = recv_diagnostics(&receiver, &mut child, "grund.toml");
    send_message(
        &mut stdin,
        json!({"jsonrpc":"2.0", "id":2, "method":"shutdown", "params":null}),
    );
    recv_response_or_panic(&receiver, &mut child, 2);
    send_message(&mut stdin, json!({"jsonrpc":"2.0", "method":"exit"}));
    drop(stdin);
    wait_for_exit(&mut child);
    assert_refusal(&diagnostics, &cli, line);
}

macro_rules! located_refusal {
    ($name:ident, $config:expr, $expected:expr) => {
        #[test]
        fn $name() {
            startup(&$config, $expected, true, |_| {});
        }
    };
}

located_refusal!(
    absent_value_home,
    value_config("docs/limits"),
    "does not exist: docs/limits"
);

#[test]
fn file_instead_of_value_folder() {
    startup(
        &value_config("docs/limits.md"),
        "must be an existing folder",
        true,
        |root| {
            fs::write(root.join("docs/limits.md"), "").unwrap();
        },
    );
}

#[test]
fn value_home_outside_root() {
    startup(
        &format!("{V1}[[kinds]]\nkind = \"CONST\"\nfolder = \"..\"\nvalues = true\n"),
        "must normalize inside the project root",
        true,
        |_| {},
    );
}

#[test]
fn folder_instead_of_value_file() {
    startup(
        &format!("{V1}[[kinds]]\nkind = \"CONST\"\nfile = \"docs\"\nvalues = true\n"),
        "must be an existing file",
        true,
        |_| {},
    );
}

// Each located producer in kind_values.rs, including the v1-only combinations.
located_refusal!(
    uncitable_values,
    format!("{V1}[[kinds]]\nkind = \"CONST\"\ncitable = false\nvalues = true\n"),
    "sets `values = true` with `citable = false`"
);
located_refusal!(
    values_without_home,
    format!("{V1}[[kinds]]\nkind = \"CONST\"\nvalues = true\n"),
    "without exactly one `file` or `folder` home"
);
located_refusal!(
    chapter_without_named_sections,
    format!("{V1}[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nvalue_chapter = \"values\"\n"),
    "named_sections is not true"
);
const NAMED: &str = "[id]\nnamed_sections = true\n";
located_refusal!(
    uncitable_chapter,
    format!(
        "{V1}{NAMED}[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\ncitable = false\nvalue_chapter = \"values\"\n"
    ),
    "sets `value_chapter` with `citable = false`"
);
located_refusal!(
    unscanned_chapter,
    format!(
        "{V1}{NAMED}[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nscan = false\nvalue_chapter = \"values\"\n"
    ),
    "sets `value_chapter` with `scan = false`"
);
located_refusal!(
    chapter_with_whole_value,
    format!(
        "{V1}{NAMED}[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nvalues = true\nvalue_chapter = \"values\"\n"
    ),
    "sets `value_chapter` and `values = true`"
);
located_refusal!(
    chapter_without_home,
    format!("{V1}{NAMED}[[kinds]]\nkind = \"FS\"\nvalue_chapter = \"values\"\n"),
    "sets `value_chapter` without exactly one"
);
located_refusal!(
    nameless_v1_kind,
    format!("{V1}[[kinds]]\nfolder = \"docs\"\n"),
    "every [[kinds]] entry must declare a `kind`"
);

// Other config-load producers that render a known line into plain errors.
located_refusal!(
    invalid_workspace_member,
    format!("{V1}[workspace]\nmembers = [\"../escape\"]\n"),
    "invalid [workspace] member"
);
located_refusal!(
    member_in_both_lists,
    format!("{V1}[workspace]\nmembers = [\"member\"]\noptional_members = [\"member\"]\n"),
    "both"
);
located_refusal!(
    optional_member_glob,
    format!("{V1}[workspace]\noptional_members = [\"members/*\"]\n"),
    "optional_members may not use a glob"
);
located_refusal!(
    invalid_optional_alias,
    format!("{V1}[workspace]\noptional_members = [\"Bad_Name\"]\n"),
    "workspace member `Bad_Name`"
);
located_refusal!(
    invalid_lead_size_max,
    format!("{V1}[reference]\nlead_size_warning = {{ max = \"no\", unit = \"words\" }}\n"),
    "lead_size_warning max must be a non-negative integer"
);
located_refusal!(
    invalid_lead_size_unit_type,
    format!("{V1}[reference]\nlead_size_warning = {{ max = 10, unit = 42 }}\n"),
    "lead_size_warning unit must be a string"
);
located_refusal!(
    invalid_citation_target,
    format!("{V1}[citations.FS]\nmust = [\"*/\"]\n"),
    "citation target"
);

#[test]
fn parse_error_keeps_its_named_line() {
    startup(
        "grund_config_version = 2\nproject_name = \"example\"\n[schema.sources\n",
        "expected `[table]`",
        true,
        |_| {},
    );
}

#[test]
fn unlocated_refusal_keeps_first_line_fallback() {
    startup(
        &format!(
            "{V1}[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\n\
        [[kinds]]\nkind = \"FS\"\nfolder = \"other\"\n"
        ),
        "grund.toml: kind `FS` is declared twice",
        false,
        |_| {},
    );
}

#[test]
fn semantic_refusal_reload_and_recovery() {
    let composition = Composition::new("1150-reload");
    let healthy = value_config("docs");
    composition.write("enterprise/grund.toml", &healthy);
    composition.write("enterprise/docs/limits.json", "{}\n");
    let mut session = Session::start(&composition, &["enterprise"], false);
    assert!(session.diagnostics_on("enterprise/grund.toml").is_empty());
    composition.write("enterprise/grund.toml", &value_config("absent"));
    let cli = cli_error(&composition.path("enterprise"), "does not exist: absent");
    session.watched(&[("enterprise/grund.toml", 2)]);
    let diagnostics = session.diagnostics_on("enterprise/grund.toml");
    composition.write("enterprise/grund.toml", &healthy);
    session.watched(&[("enterprise/grund.toml", 2)]);
    let recovered = session.diagnostics_on("enterprise/grund.toml");
    session.shutdown();
    assert!(recovered.is_empty(), "successful reload clears the refusal");
    assert_eq!(diagnostics.len(), 1);
    let message = diagnostics[0]["message"].as_str().expect("message");
    assert!(message.starts_with(&cli), "CLI text: {message}");
    assert!(
        message.contains("last good scan"),
        "retained scan: {message}"
    );
    assert_eq!(
        diagnostics[0]["range"]["start"]["line"],
        named_line(&cli),
        "{message}"
    );
}

#[test]
fn imported_semantic_refusal_keeps_import_site_precedence() {
    let composition = Composition::new("1150-import-site");
    composition.write(
        "graal/substratevm/grund.toml",
        &DEPENDENCY_CONFIG.replace("\"values\"", "\"absent\""),
    );
    let mut session = Session::start(&composition, &["enterprise"], false);
    let diagnostics = session.diagnostics_on("enterprise/grund.toml");
    session.shutdown();
    let refusal = diagnostics
        .iter()
        .find(|d| d["severity"] == 1)
        .expect("import refusal");
    let message = refusal["message"].as_str().expect("message");
    assert!(message.contains("does not exist: absent"), "{message}");
    assert!(
        message.contains("note: imported as `native-image` from grund.toml:4"),
        "{message}"
    );
    // The dependency's named line differs from the consumer's import header.
    let dependency_line = named_line(
        message
            .strip_prefix("../graal/substratevm/")
            .expect("dependency path"),
    );
    assert_ne!(
        dependency_line, 3,
        "the fixture must distinguish both locations"
    );
    assert_eq!(refusal["range"]["start"]["line"], 3, "{message}");
}
