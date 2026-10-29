//! §FS-integrations.2.3: wherever `grund integrations` offers setup it names
//! `grund-lsp`, so detection closes on the way to it, in the catalog and in the
//! detected list alike (§FS-integrations.2.1). The no-client form is byte-stable
//! only for a fixed environment (§FS-integrations.2.2, §FS-integrations.6), so
//! each run clears the caller's environment, keeps `PATH`, points `HOME` and
//! `XDG_CONFIG_HOME` at an empty sandbox, and sets only what its case is about.
//! Unix only, like the other integrations runs: the cleared environment is the
//! fixture, and a Windows child needs more of it than `PATH`.
#![cfg(unix)]

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// What both detection forms close with: the preview hint and the setup guide
/// (§FS-integrations.2.1), then the way to `grund-lsp` (§FS-integrations.2.3).
const CLOSING: &str = concat!(
    "\n",
    "Run `grund integrations <client>` to preview one before installing.\n",
    "Setup guide: https://github.com/agent-grounds/grund/blob/main/docs/user-facing/clickable-citations.md\n",
    "Editors get go-to-definition and hover from grund-lsp (IntelliJ: `grund-lsp integrations lsp4ij`).\n",
    "LSP guide: https://github.com/agent-grounds/grund/blob/main/docs/user-facing/lsp.md\n",
);

/// Run `grund <args>` with nothing from the caller's environment but `PATH`,
/// plus `vars`, in an empty sandbox; return `(exit, stdout, stderr)`.
fn run(case: &str, vars: &[(&str, &str)], args: &[&str]) -> (Option<i32>, String, String) {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("integrations-lsp-pointer")
        .join(case);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("home")).expect("create sandbox HOME");
    fs::create_dir_all(root.join("xdg")).expect("create sandbox XDG_CONFIG_HOME");
    let output = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("HOME", root.join("home"))
        .env("XDG_CONFIG_HOME", root.join("xdg"))
        .envs(vars.iter().copied())
        .current_dir(&root)
        .output()
        .expect("run grund integrations");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// §FS-integrations.2.3: IntelliJ's terminal sets `TERMINAL_EMULATOR`, which
/// detection does not read (§FS-integrations.2), so the IntelliJ user the ticket
/// is about gets the catalog, and the catalog has to tell them where their
/// editor's integration is.
#[test]
fn the_catalog_an_intellij_terminal_gets_closes_on_the_way_to_grund_lsp() {
    let jediterm = [
        ("TERM", "xterm-256color"),
        ("TERMINAL_EMULATOR", "JetBrains-JediTerm"),
    ];
    let expected = format!(
        "{}{CLOSING}",
        concat!(
            "No supported terminal or editor detected. Available integrations:\n",
            "  codium   grund integrations codium --write\n",
            "  iterm2   grund integrations iterm2 --write\n",
            "  kitty    grund integrations kitty --write\n",
            "  tmux     grund integrations tmux --write\n",
            "  vscode   grund integrations vscode --write\n",
            "  wezterm  grund integrations wezterm --write\n",
        )
    );
    assert_eq!(
        run("catalog-jediterm", &jediterm, &["integrations"]),
        (Some(0), expected, String::new())
    );
}

/// §FS-integrations.2.1, §FS-integrations.2.3: a detected client closes with the
/// same lines as the catalog, so a user in WezTerm learns of `grund-lsp` too.
#[test]
fn a_detected_client_closes_on_the_same_way_to_grund_lsp() {
    let expected = format!(
        "{}{CLOSING}",
        concat!(
            "Detected integrations for this environment:\n",
            "  wezterm  grund integrations wezterm --write\n",
        )
    );
    assert_eq!(
        run(
            "detected-wezterm",
            &[("TERM_PROGRAM", "WezTerm")],
            &["integrations"]
        ),
        (Some(0), expected, String::new())
    );
}

/// §FS-integrations.2.3, §FS-integrations.5: the pointer is text a person reads.
/// The JSON detection plan carries none, so these are its bytes before the
/// pointer and after it, for both environments above.
#[test]
fn the_json_detection_plan_carries_no_pointer() {
    let clients = |wezterm: bool| {
        format!(
            concat!(
                "[",
                r#"{{"client":"codium","detected":false,"installed":false,"install_kind":"extension","install":"grund integrations codium --write"}},"#,
                r#"{{"client":"iterm2","detected":false,"installed":false,"install_kind":"manual","install":"grund integrations iterm2 --write"}},"#,
                r#"{{"client":"kitty","detected":false,"installed":false,"install_kind":"block","install":"grund integrations kitty --write"}},"#,
                r#"{{"client":"tmux","detected":false,"installed":false,"install_kind":"block","install":"grund integrations tmux --write"}},"#,
                r#"{{"client":"vscode","detected":false,"installed":false,"install_kind":"extension","install":"grund integrations vscode --write"}},"#,
                r#"{{"client":"wezterm","detected":{},"installed":false,"install_kind":"block","install":"grund integrations wezterm --write"}}"#,
                "]",
            ),
            wezterm
        )
    };
    let json = ["integrations", "--format", "json"];
    assert_eq!(
        run(
            "json-jediterm",
            &[("TERMINAL_EMULATOR", "JetBrains-JediTerm")],
            &json
        ),
        (
            Some(0),
            format!("{{\"detected\":[],\"clients\":{}}}\n", clients(false)),
            String::new()
        )
    );
    assert_eq!(
        run("json-wezterm", &[("TERM_PROGRAM", "WezTerm")], &json),
        (
            Some(0),
            format!(
                "{{\"detected\":[\"wezterm\"],\"clients\":{}}}\n",
                clients(true)
            ),
            String::new()
        )
    );
}
