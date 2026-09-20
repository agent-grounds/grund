//! Binary-level contract for the `agents-init` message ramp: the five messages
//! §FS-errors.3.6 carries through its two-release migration, and the release
//! they reach their final templates in (§FS-errors.3.6.1).
//!
//! Its own target rather than a group inside `check_finding_selection.rs`: that
//! file is about *selecting* findings, and these cases are about the text of
//! one code across a release boundary — they fail for different reasons and on
//! different days.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const MAINTENANCE_TAIL: &str =
    " — repo maintenance; citation checks still ran; wording changes in grund 0.14.0";

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
            "<!-- BEGIN GRUND MANAGED BLOCK -->\n## Grounding with grund (v10)\n\ncurrent managed block\n",
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

fn release(text: &str) -> (u64, u64, u64) {
    let mut parts = text.split('.').map(|part| {
        part.split(|ch: char| !ch.is_ascii_digit())
            .next()
            .unwrap_or("0")
            .parse::<u64>()
            .unwrap_or_else(|_| panic!("not a version: {text}"))
    });
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}

/// §FS-errors.3.6.1: the five final templates land in `0.14.0`, where the
/// compatibility tail is removed. Below that release there are no final
/// templates to observe, so what this case pins is the window itself, in the
/// shape `warning_phase_cannot_survive_the_release_it_names` uses for the other
/// ramp this tree carries: while the tail is still shipping, this tree may not
/// be at or above the release the tail names, and the closing clause of the
/// final form may not have leaked in early.
///
/// From `0.14.0` the other branch is the live one: no tail, and every one of
/// the five wearing §FS-errors.3.6.1's fixed frame — the `repo maintenance: `
/// classification in front and the `(does not affect citation validity)` clause
/// behind. The frame is asserted rather than the five filled strings because
/// the templates interpolate the managed-block version, which is not what this
/// ramp is about.
#[test]
fn the_agents_init_tail_cannot_survive_the_release_it_names() {
    let messages = agents_init_messages();
    assert_eq!(messages.len(), 5, "{messages:#?}");
    let tailed = messages
        .iter()
        .filter(|message| message.ends_with(MAINTENANCE_TAIL))
        .count();

    if tailed > 0 {
        assert!(
            release(env!("CARGO_PKG_VERSION")) < release("0.14.0"),
            "this tree reached 0.14.0; land §FS-errors.3.6.1's five final templates instead of shipping the compatibility tail"
        );
        assert_eq!(
            tailed, 5,
            "the tail is all five messages' or none's: {messages:#?}"
        );
        for message in &messages {
            let legacy = message
                .strip_suffix(MAINTENANCE_TAIL)
                .expect("checked above");
            assert!(
                !legacy.is_empty() && !legacy.contains("does not affect citation validity"),
                "the final template's closing clause landed before its release: {message}"
            );
        }
        return;
    }

    for message in &messages {
        assert!(
            message.starts_with("repo maintenance: "),
            "the final template opens with its classification: {message}"
        );
        assert!(
            message.ends_with(" (does not affect citation validity)"),
            "the final template closes with its reassurance: {message}"
        );
    }
}
