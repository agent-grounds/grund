//! §FS-config.4.2.3.1: every place that sends a reader to `grund config show`
//! names `grund config explain` for what `show` leaves out, and none says that
//! `show` prints every setting (#1068). `show`'s own bytes are not this file's:
//! the `config-show-fresh-scaffold` e2e case holds them.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The sentence §FS-config.4.2.3.1 asks for wherever a place's subject allows.
const SENTENCE: &str =
    "`grund config show` prints what applies, `grund config explain` every setting";

/// The claims a place made before #1068: that `show` prints every setting.
const OVERCLAIMS: [&str; 5] = [
    "prints every setting",
    "prints every effective setting",
    "every effective setting is printed",
    "prints each setting",
    "prints the table each one lives in",
];

/// The places that send a reader to `grund config show`: each paragraph that
/// names it also names `grund config explain`.
const POINTERS: [&str; 10] = [
    "crates/grund-core/assets/templates/software-development/grund.toml",
    "crates/grund-core/assets/templates/none/grund.toml",
    "skills/grund-init/references/config.md",
    "crates/grund-core/assets/skills/grund-init/references/config.md",
    "README.md",
    "docs/user-facing/init.md",
    "docs/user-facing/setup.md",
    "examples/init-template-none/README.md",
    "examples/quickstart/README.md",
    "docs/user-facing/configuration.md",
];

/// The specification that describes `show`: it says what `show` prints, not more.
const SPECS: [&str; 4] = [
    "docs/functional-spec/FS-config.md",
    "docs/functional-spec/FS-init.md",
    "docs/functional-spec/FS-inline-citation-style.md",
    "docs/decisions/functional/DF-config-explain.md",
];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(place: &str) -> String {
    fs::read_to_string(root().join(place)).unwrap_or_else(|error| panic!("read {place}: {error}"))
}

/// `text`'s paragraphs, each with its wrapped lines joined by one space.
fn paragraphs(text: &str) -> Vec<String> {
    text.split("\n\n")
        .map(|paragraph| paragraph.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect()
}

/// The overclaims `text` still makes, read across its line wraps.
fn overclaims(text: &str) -> Vec<&'static str> {
    let joined = text.split_whitespace().collect::<Vec<_>>().join(" ");
    OVERCLAIMS
        .into_iter()
        .filter(|claim| joined.contains(claim))
        .collect()
}

/// The paragraphs of `text` that send a reader to `show` and never name `explain`.
fn unpointed(text: &str) -> Vec<String> {
    paragraphs(text)
        .into_iter()
        .filter(|p| p.contains("grund config show") && !p.contains("grund config explain"))
        .collect()
}

/// `grund <args>` from the repository root: its exit code and stdout.
fn grund(args: &[&str], cwd: &Path) -> (Option<i32>, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("spawn grund");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
    )
}

/// §FS-config.4.2.3.1: no place that sends a reader to `show`, and no point of
/// the specification that describes it, says `show` prints every setting.
#[test]
fn no_place_says_show_prints_every_setting() {
    let mut found = Vec::new();
    for place in POINTERS.iter().chain(SPECS.iter()) {
        for claim in overclaims(&read(place)) {
            found.push(format!("{place}: `{claim}`"));
        }
    }
    assert!(
        found.is_empty(),
        "these places still say `grund config show` prints every setting, though it leaves \
         out an opt-in table that is off:\n{}",
        found.join("\n")
    );
}

/// §FS-config.4.2.3.1: every paragraph that sends a reader to `show` names
/// `grund config explain` for what `show` leaves out.
#[test]
fn every_pointer_to_show_names_explain() {
    let mut found = Vec::new();
    for place in POINTERS {
        for paragraph in unpointed(&read(place)) {
            found.push(format!("{place}: {paragraph}"));
        }
    }
    assert!(
        found.is_empty(),
        "these paragraphs send a reader to `grund config show` and never name \
         `grund config explain`:\n{}",
        found.join("\n\n")
    );
}

/// §FS-config.4.2.3.1, §FS-init.2.4.5: the comment a fresh `init` writes is the
/// sentence, after what the file holds, on one line of at most 100 columns.
#[test]
fn the_shipped_header_is_the_sentence() {
    let header = "# Only non-defaults; `grund config show` prints what applies, \
                  `grund config explain` every setting.";
    assert!(header.contains(SENTENCE), "{header}");
    assert!(header.len() <= 100, "{} columns: {header}", header.len());
    for template in POINTERS.iter().take(2) {
        assert_eq!(
            read(template).lines().next(),
            Some(header),
            "{template}: the first line says what `show` prints and names `explain` for the rest"
        );
    }
}

/// §FS-config.4.2.3.1, §FS-init.5.5: the configuration questions `grund guide
/// config` prints, from the copy the binary ships, say the sentence where they
/// send a version-2 reader to `show`.
#[test]
fn guide_config_sends_a_version_2_reader_to_show_and_explain() {
    let (code, guide) = grund(&["guide", "config"], &root());
    assert_eq!(code, Some(0), "grund guide config:\n{guide}");
    let claims = overclaims(&guide);
    assert!(
        claims.is_empty(),
        "grund guide config still says {claims:?}"
    );
    let joined = guide.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        joined.contains(&format!("for a version-2 file, {SENTENCE}")),
        "grund guide config does not say, for a version-2 file, `{SENTENCE}`"
    );
}

/// §FS-config.4.2.3.1, §FS-cli.2.3: the help of `grund init` and of `grund config`
/// name `grund config explain` beside `show`, and neither says `show` prints every
/// setting; `grund init -h` says the sentence.
#[test]
fn help_names_explain_beside_show() {
    for (args, sentence) in [
        (&["init", "-h"][..], true),
        (&["config", "-h"][..], false),
        (&["config", "show", "-h"][..], false),
    ] {
        let (code, help) = grund(args, &root());
        assert_eq!(code, Some(0), "grund {args:?}:\n{help}");
        let claims = overclaims(&help);
        assert!(claims.is_empty(), "grund {args:?} still says {claims:?}");
        let show = help
            .lines()
            .position(|line| line.contains("grund config show") || line.starts_with("  show "))
            .unwrap_or_else(|| panic!("grund {args:?} never names `show`:\n{help}"));
        let entry = help
            .lines()
            .skip(show)
            .take(2)
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            entry.contains("grund config explain"),
            "grund {args:?}: where it names `show` it never names `grund config explain`:\n{entry}"
        );
        if sentence {
            assert!(entry.contains(SENTENCE), "grund {args:?}: {entry}");
        }
    }
}
