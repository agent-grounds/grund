//! Binary-level contract for the licence §FS-check.3.24.2 claims: the text the
//! declaration-local section finding shipped with survives as a verbatim
//! contiguous prefix of every one of its three shapes, at the same offsets, so
//! the release attribution §FS-check.3.24.1 adds is an append rather than a
//! wording change — on the owned shape, wherever the owner has the section. Where
//! it has not, the one exception §FS-check.3.24.2 names holds instead: the head
//! through the token and the attribution at the end are all that is kept
//! (§FS-check.3.24.3).
//!
//! Its own target rather than a group inside `local_section_citations.rs`: that
//! file records the finding's *current* bytes and is expected to move whenever
//! they do, which is exactly the edit this case exists to refuse. A prefix
//! assertion kept beside the full strings it guards would be rewritten in the
//! same pass that broke it.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The three shapes' text as the rule shipped it in `0.14.0`, before
/// §FS-check.3.24.1's clauses were appended. Written out rather than derived,
/// because a derived prefix would move with the code it is holding still.
const SHIPPED: [&str; 3] = [
    "local section citation \u{a7}2; write \u{a7}FS-a.2",
    "unsupported local section citation \u{a7}2abc; write a full citation or <\u{a7}>2abc to show the shape without citing it",
    "local section citation \u{a7}2 has no enclosing declaration; write a full citation or <\u{a7}>2 to show the shape without citing it",
];

/// §FS-check.3.24.1: the clause every shape ends with — the pair of releases the
/// verdict moved between.
const ATTRIBUTION: &str = " \u{2014} unchecked in grund 0.13.1, an error in 0.14.0";

/// One owned site the formatter would write, one digit-starting unsupported
/// token, and one ownerless site — §FS-check.3.24's three shapes in one tree.
fn fixture(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/local-section-message-append")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("docs")).expect("create fixture tree");
    write(
        root.join("grund.toml"),
        concat!(
            "grund_config_version = 1\nproject_name = \"local-section-append\"\n\n",
            "[reference]\nstrict = true\nrequire_grounding = false\n\n",
            "[id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\n",
            "[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n\n",
            "[scan]\ninclude = [\"docs\"]\nextensions = [\"md\"]\n\n",
            "[output]\nrelative_paths = true\n",
        ),
    );
    write(
        root.join("docs/FS-a.md"),
        concat!(
            "# FS-a: A thing\n\n",
            "Owned \u{a7}2.\n",
            "Glued \u{a7}2abc.\n\n",
            "## 2. Target\n",
        ),
    );
    write(
        root.join("docs/outside.md"),
        "# Notes\n\nOwnerless \u{a7}2.\n",
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

/// Every `local-section-citation` message a `--only` run printed, without its
/// `<path>:<line>: error: ` prefix.
fn local_section_messages(root: &Path) -> Vec<String> {
    let output = Command::new(env!("CARGO_BIN_EXE_grund"))
        .arg("check")
        .arg(root)
        .args(["--only", "local-section-citation"])
        .output()
        .expect("run grund check");
    let stdout = std::str::from_utf8(&output.stdout).expect("stdout is UTF-8");
    let stderr = std::str::from_utf8(&output.stderr).expect("stderr is UTF-8");
    let messages = stdout
        .lines()
        .filter_map(|line| line.split_once(": error: "))
        .map(|(_, message)| message.to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        messages.len(),
        3,
        "the fixture is the three shapes; stdout {stdout:?} stderr {stderr:?}"
    );
    messages
}

/// §FS-check.3.24.2: the licence rests on this and nothing else. A consumer
/// matching the shipped text still matches, so the change needs no deprecation
/// path of its own under §REQ-backwards-compatibility.1 — and an edit that
/// rewords the *head* of one of these lines while believing it only appends is
/// exactly what has to fail here.
#[test]
fn the_shipped_local_section_text_is_still_a_verbatim_prefix() {
    let messages = local_section_messages(&fixture("prefix"));
    for shipped in SHIPPED {
        let matched = messages
            .iter()
            .filter(|message| message.starts_with(shipped))
            .count();
        assert_eq!(
            matched, 1,
            "{shipped:?} is no longer a verbatim contiguous prefix of one finding: {messages:#?}"
        );
    }
    for (message, shipped) in messages.iter().zip(SHIPPED) {
        let appended = message.strip_prefix(shipped).unwrap_or_else(|| {
            panic!("shape order changed: {message:?} is not {shipped:?} + a tail")
        });
        assert!(
            appended.starts_with(ATTRIBUTION),
            "§FS-check.3.24.1: the attribution follows the shipped text immediately: {appended:?}"
        );
    }
}

/// §FS-check.3.24's third bullet: a digit-starting mixed, named, or glued tail
/// names the releases too, and never the command — §FS-fmt.2.4 leaves it
/// byte-identical, so an offer to run the formatter would answer
/// `rewrote 0 lines` (§FS-check.3.17.5). Pinned here because the e2e golden
/// covers the owned and ownerless shapes only.
#[test]
fn a_digit_starting_unsupported_token_names_the_releases_without_the_command() {
    let messages = local_section_messages(&fixture("glued"));
    let glued = messages
        .iter()
        .find(|message| message.starts_with("unsupported local section citation"))
        .unwrap_or_else(|| panic!("the glued shape: {messages:#?}"));
    assert!(
        glued.ends_with(ATTRIBUTION),
        "the attribution is the whole of its tail: {glued:?}"
    );
    assert!(
        !glued.contains("grund fmt --write"),
        "no owner to expand it against, so no command is offered: {glued:?}"
    );
}

/// §FS-check.3.24.2's one exception, and the narrower promise it keeps. Where
/// §FS-fmt.2.4.6 refuses the rewrite because the owner lacks the section, the
/// shipped `write <citation>` is replaced (§FS-check.3.24.3), so it is no longer
/// a prefix there; what a consumer can still match is the head through
/// `local section citation <token>; ` and the attribution, which ends the line
/// with nothing after it. The resolving control beside it keeps every byte. Its
/// own fixture, because the one above is the three shapes and this is a fourth
/// site rather than a fourth shape — the owned shape again, with a section its
/// owner does not have.
#[test]
fn an_absent_site_keeps_the_head_and_the_attribution_and_drops_the_shipped_remedy() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/local-section-message-append/absent-target");
    let _ = fs::remove_dir_all(&root);
    write(
        root.join("grund.toml"),
        concat!(
            "grund_config_version = 1\nproject_name = \"local-section-absent\"\n\n",
            "[reference]\nstrict = true\nrequire_grounding = false\n\n",
            "[id]\nformat = \"{kind}-{slug}\"\n\n",
            "[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n\n",
            "[scan]\ninclude = [\"docs\"]\nextensions = [\"md\"]\n\n",
            "[output]\nrelative_paths = true\n",
        ),
    );
    write(
        root.join("docs/FS-a.md"),
        concat!(
            "# FS-a: A thing\n\n",
            "Owned and resolving \u{a7}2.\n",
            "Owned and absent \u{a7}9.9.\n\n",
            "## 2. Target\n",
        ),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_grund"))
        .arg("check")
        .arg(&root)
        .args(["--only", "local-section-citation"])
        .output()
        .expect("run grund check");
    let stdout = std::str::from_utf8(&output.stdout).expect("stdout is UTF-8");
    let messages = stdout
        .lines()
        .filter_map(|line| line.split_once(": error: "))
        .map(|(_, message)| message.to_string())
        .collect::<Vec<_>>();
    assert_eq!(messages.len(), 2, "two owned sites: {stdout:?}");
    assert_eq!(
        messages[0],
        format!("{}{ATTRIBUTION}; run `grund fmt --write`", SHIPPED[0]),
        "the resolving control keeps every byte, the command clause included"
    );
    let absent = &messages[1];
    assert!(
        absent.starts_with("local section citation \u{a7}9.9; "),
        "§FS-check.3.24.2: the head through the token is kept: {absent:?}"
    );
    assert!(
        absent.ends_with(ATTRIBUTION),
        "§FS-check.3.24.2: the attribution still ends the line, with nothing after it: {absent:?}"
    );
    assert!(
        !absent.starts_with("local section citation \u{a7}9.9; write \u{a7}FS-a.9.9"),
        "§FS-check.3.24.2's exception: the shipped remedy names a section FS-a lacks, so it is replaced rather than kept as a prefix: {absent:?}"
    );
}
