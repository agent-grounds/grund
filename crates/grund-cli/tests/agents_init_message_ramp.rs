//! Binary-level contract for the `agents-init` messages: the five final
//! templates §FS-errors.3.6.1 fixes, which the two-release migration of
//! §FS-errors.3.6 ended in.
//!
//! Its own target rather than a group inside `check_finding_selection.rs`: that
//! file is about *selecting* findings, and these cases are about the text of
//! one code across a release boundary — they fail for different reasons and on
//! different days.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixture_root(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/agents-init-message-ramp")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("docs/functional-spec")).expect("create fixture tree");
    write(
        root.join("grund.toml"),
        concat!(
            "grund_config_version = 1\n\n",
            "[reference]\nmarker = \"\u{a7}\"\nstrict = true\n\n",
            "[id]\nformat = \"{kind}-{slug}\"\nslug_pattern = \"[a-z][a-z0-9-]*\"\n\n",
            "[scan]\ninclude = [\".\"]\n\n",
            "[[kinds]]\nkind = \"FS\"\nfolder = \"docs/functional-spec\"\nindex = false\n",
        ),
    );
    write(
        root.join("docs/functional-spec/FS-live.md"),
        "# FS-live: Live behavior\n\nThe behavior cites \u{a7}FS-live.\n",
    );
    root
}

fn write(path: impl AsRef<Path>, text: &str) {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create fixture parent");
    }
    fs::write(path, text).expect("write fixture file");
}

/// The one `agents-init` finding a `--only agents-init` run printed, without
/// its `<path>:<line>: error: ` prefix.
fn agents_init_line(root: &Path) -> String {
    let output: Output = Command::new(env!("CARGO_BIN_EXE_grund"))
        .arg("check")
        .arg(root)
        .args(["--only", "agents-init"])
        .output()
        .expect("run grund check");
    let stdout = std::str::from_utf8(&output.stdout).expect("stdout is UTF-8");
    let stderr = std::str::from_utf8(&output.stderr).expect("stderr is UTF-8");
    let mut lines = stdout.lines();
    let line = lines
        .next()
        .unwrap_or_else(|| panic!("no finding; stdout {stdout:?} stderr {stderr:?}"));
    assert_eq!(lines.next(), None, "one finding per fixture: {stdout:?}");
    line.split_once(": error: ")
        .unwrap_or_else(|| panic!("not a located error line: {line:?}"))
        .1
        .to_string()
}

/// The five §FS-errors.3.6 `agents-init` messages this binary actually prints,
/// in the order §FS-errors.3.6.1 lists them.
///
/// Four states are three lines of `AGENTS.md` each. The fifth — a *stale*
/// managed block — needs a whole rendered current-version block for one section
/// to drift against, so it is read from the e2e case that already carries it
/// rather than duplicated here.
fn agents_init_messages() -> Vec<String> {
    let mut messages = Vec::new();
    for (name, agents) in [
        (
            "malformed",
            "<!-- BEGIN GRUND MANAGED BLOCK -->\n## Grounding with grund (v14)\n\ncurrent managed block\n",
        ),
        ("outdated", "## Grounding with grund (v3)\n\nlegacy block\n"),
        (
            "unsupported",
            "## Grounding with grund (v999)\n\nfuture block\n",
        ),
    ] {
        let root = fixture_root(name);
        write(root.join("AGENTS.md"), agents);
        messages.push(agents_init_line(&root));
    }

    let stale = include_str!("../../../tests/e2e/cases/check-conversation-drift/expected.stdout");
    messages.insert(
        3,
        stale
            .lines()
            .find_map(|line| line.split_once(": error: "))
            .expect("the stale golden's one finding")
            .1
            .to_string(),
    );

    let missing = fixture_root("missing");
    write(missing.join("AGENTS.md"), "# Existing agents\n");
    messages.push(agents_init_line(&missing));
    messages
}

/// §FS-errors.3.6.1: the five final templates, filled for these fixtures, in the
/// order the spec lists them — no compatibility tail, the `repo maintenance: `
/// classification in front, and the `(does not affect citation validity)` clause
/// behind.
#[test]
fn the_agents_init_messages_are_the_five_final_templates() {
    assert_eq!(
        agents_init_messages(),
        vec![
            "repo maintenance: malformed grund managed block: missing `<!-- END GRUND MANAGED BLOCK -->` (does not affect citation validity)",
            "repo maintenance: outdated grund init block v3 — run `grund init` to update to v14 (does not affect citation validity)",
            "repo maintenance: unsupported grund init block v999 — this grund supports v14 (does not affect citation validity)",
            "repo maintenance: stale grund init block: clickable citations differ from grund.toml — run `grund init` to refresh (does not affect citation validity)",
            "repo maintenance: missing grund init block v14 — run `grund init` to install it (does not affect citation validity)",
        ]
    );
}
