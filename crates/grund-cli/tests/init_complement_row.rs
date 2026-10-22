//! §FS-init.2.3.4.4.3 — the title of the complement's Project-map row. An
//! untitled `code` row with places reads `Every scanned file no other row claims`,
//! never `Declaration`, whether `grund config migrate --to 2` made it from a
//! version-1 `[scan] include` (agent-grounds/grund#782) or a version-2 file wrote
//! it directly. A configured title wins, every other untitled row still reads
//! `Declaration`, and the citation-directions subject does not move
//! (§FS-init.2.3.5.2), so the migration's guidance comparison
//! (§FS-config-migrate.5.3) passes as it did.
//!
//! These are not golden cases under `tests/e2e/cases/`: the reported path is
//! three commands on one tree — `init`, `config migrate --to 2 --write`, `init` —
//! and a golden case runs one.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

#[path = "support/init_fixture.rs"]
mod init_fixture;

use init_fixture::{manifest_dir, run_grund, workdir};

/// The title §FS-init.2.3.4.4.3 gives an untitled complement row.
const COMPLEMENT_TITLE: &str = "Every scanned file no other row claims";

/// The note §FS-config-migrate.5.3 prints when the complement gains a row.
const MIGRATE_NOTE: &str = "note: the next grund init adds a `code` row to the AGENTS.md \
                            Project map, because the complement's places are now listed; \
                            grund check reports the block stale until then";

/// A version-1 config shaped like rhei's: one `FS` folder, and a `[scan] include`
/// that lists files and folders beyond it. Version 1 has no field for the
/// complement's title, so nothing here titles it.
const V1_RHEI_SHAPED: &str = r#"grund_config_version = 1
[reference]
strict = true
[id]
format = "{kind}-{slug}"
named_sections = true
[[kinds]]
kind = "FS"
folder = "docs/fs"
index = false
[scan]
include = ["AGENTS.md", "README.md", "docs", "crates", "scripts"]
"#;

/// What the migration writes for that `[scan] include`.
const MIGRATED_CODE_ROW: &str = "[schema.kinds.code]\n\
                                 files = [\"AGENTS.md\", \"README.md\"]\n\
                                 folders = [\"docs\", \"crates\", \"scripts\"]\n";

/// The complement's row once that config is migrated, less its title.
const MIGRATED_PLACES: &str = "- [AGENTS.md](AGENTS.md), [README.md](README.md), \
                               [docs/](docs), [crates/](crates), [scripts/](scripts)";

/// A version-2 config written directly: an untitled `FS` row, an untitled
/// non-citable row, and an untitled complement with a file and a folder.
const V2_UNTITLED: &str = r#"grund_config_version = 2
project_name = "complement"
[schema.kinds.FS]
folders = ["docs/spec"]
[schema.kinds.e2e]
citable = false
folders = ["tests/e2e"]
[schema.kinds.code]
files = ["build.gradle"]
folders = ["src"]
[rules.citations.code]
should = ["FS"]
"#;

/// The complement's row in that config, less its title.
const V2_PLACES: &str = "- [build.gradle](build.gradle), [src/](src)";

/// The directions bullet for an untitled complement, before the fix and after.
const UNTITLED_SUBJECT_LINE: &str =
    "- Each source file outside the Project map (**code**) that cites anything should cite FS.";

fn v1_tree(name: &str, config: &str) -> PathBuf {
    let target = workdir(name);
    for dir in ["docs/fs", "crates/x", "scripts"] {
        fs::create_dir_all(target.join(dir)).expect("create place");
    }
    fs::write(target.join("grund.toml"), config).expect("write grund.toml");
    fs::write(target.join("docs/fs/a.md"), "# FS-a: A\n\nBody.\n").expect("write FS-a");
    fs::write(target.join("crates/x/lib.rs"), "// cites \u{a7}FS-a\n").expect("write lib.rs");
    fs::write(target.join("README.md"), "# Readme\n").expect("write README.md");
    target
}

fn v2_tree(name: &str, config: &str) -> PathBuf {
    let target = workdir(name);
    for dir in ["docs/spec", "tests/e2e", "src"] {
        fs::create_dir_all(target.join(dir)).expect("create place");
    }
    fs::write(target.join("grund.toml"), config).expect("write grund.toml");
    fs::write(target.join("build.gradle"), "plugins {}\n").expect("write build.gradle");
    target
}

fn grund(args: &[&str], target: &Path) -> Output {
    let mut args = args.to_vec();
    args.push(target.to_str().expect("utf-8 fixture path"));
    run_grund(&args, manifest_dir())
}

fn succeeds(args: &[&str], target: &Path) -> Output {
    let output = grund(args, target);
    assert!(
        output.status.success(),
        "grund {args:?} failed: exit {:?}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn agents(target: &Path) -> String {
    fs::read_to_string(target.join("AGENTS.md")).expect("read AGENTS.md")
}

/// The lines of the block's `### <heading>` section, up to the next `### `.
fn section(agents: &str, heading: &str) -> Vec<String> {
    let start = format!("### {heading}");
    agents
        .lines()
        .skip_while(|line| *line != start)
        .skip(1)
        .take_while(|line| !line.starts_with("### "))
        .map(str::to_string)
        .collect()
}

fn map_rows(target: &Path) -> Vec<String> {
    section(&agents(target), "Project map")
        .into_iter()
        .filter(|line| line.starts_with("- "))
        .collect()
}

/// agent-grounds/grund#782, the reported path: a rhei-shaped version-1 project,
/// migrated, gets a complement row that says what it is.
#[test]
fn a_migrated_complement_row_says_it_is_the_complement() {
    let target = v1_tree("complement-row-migrated", V1_RHEI_SHAPED);
    succeeds(&["init"], &target);
    succeeds(&["config", "migrate", "--to", "2", "--write"], &target);
    succeeds(&["init"], &target);

    let rows = map_rows(&target);
    let code_row = rows
        .iter()
        .find(|row| row.starts_with(MIGRATED_PLACES))
        .unwrap_or_else(|| panic!("no complement row in the Project map: {rows:#?}"));
    assert!(
        !code_row.ends_with(": Declaration"),
        "the complement row claims to be a declaration home: {code_row}"
    );
    assert_eq!(
        rows,
        [
            "- [FS](docs/fs): Declaration".to_string(),
            format!("{MIGRATED_PLACES}: {COMPLEMENT_TITLE}"),
        ]
    );
}

/// The migration writes no title for the complement, and a config with
/// `[citations.code]` rules migrates without refusal: the directions section is
/// the same before and after, because the subject keeps `(**code**)`.
#[test]
fn migrating_with_code_rules_keeps_the_directions_section() {
    let config = format!("{V1_RHEI_SHAPED}[citations.code]\nshould = [\"FS\"]\n");
    let target = v1_tree("complement-row-directions", &config);
    succeeds(&["init"], &target);
    let before = section(&agents(&target), "Citation directions");

    let migrate = succeeds(&["config", "migrate", "--to", "2", "--write"], &target);
    let report = String::from_utf8_lossy(&migrate.stderr);
    assert!(
        !report.contains("refused:"),
        "the migration refused: {report}"
    );
    assert!(report.contains(MIGRATE_NOTE), "no row note: {report}");
    let migrated = fs::read_to_string(target.join("grund.toml")).expect("read grund.toml");
    assert!(migrated.contains(MIGRATED_CODE_ROW), "{migrated}");
    assert!(
        !migrated.contains("[presentation.kinds"),
        "the migration wrote a title: {migrated}"
    );

    succeeds(&["init"], &target);
    let after = section(&agents(&target), "Citation directions");
    assert_eq!(after, before);
    assert!(
        after.iter().any(|line| line == UNTITLED_SUBJECT_LINE),
        "{after:#?}"
    );
    succeeds(&["check"], &target);
}

/// A version-2 file that writes an untitled complement with places directly gets
/// the same title the migrated one does.
#[test]
fn an_untitled_v2_complement_row_says_it_is_the_complement() {
    let target = v2_tree("complement-row-v2", V2_UNTITLED);
    succeeds(&["init"], &target);
    assert_eq!(
        map_rows(&target),
        [
            "- [FS](docs/spec): Declaration".to_string(),
            "- [tests/e2e/](tests/e2e): Declaration".to_string(),
            format!("{V2_PLACES}: {COMPLEMENT_TITLE}"),
        ]
    );
}

/// The title is wording the block renders, not a config default: `config show`
/// prints none, `init` writes none, and the directions subject stays untitled.
#[test]
fn the_complement_title_is_rendered_not_configured() {
    let target = v2_tree("complement-row-rendered", V2_UNTITLED);
    succeeds(&["init"], &target);
    assert_eq!(
        fs::read_to_string(target.join("grund.toml")).expect("read grund.toml"),
        V2_UNTITLED
    );
    let show = succeeds(&["config", "show"], &target);
    let show = String::from_utf8_lossy(&show.stdout);
    assert!(!show.contains("[presentation.kinds"), "{show}");
    assert!(!show.contains(COMPLEMENT_TITLE), "{show}");
    let directions = section(&agents(&target), "Citation directions");
    assert!(
        directions.iter().any(|line| line == UNTITLED_SUBJECT_LINE),
        "{directions:#?}"
    );
}

/// A configured title wins on the complement, and an untitled ordinary row,
/// citable or not, still reads `Declaration`.
#[test]
fn a_configured_title_wins_and_other_untitled_rows_read_declaration() {
    let config =
        format!("{V2_UNTITLED}[presentation.kinds.code]\ntitle = \"Gradle build and sources\"\n");
    let target = v2_tree("complement-row-titled", &config);
    succeeds(&["init"], &target);
    assert_eq!(
        map_rows(&target),
        [
            "- [FS](docs/spec): Declaration".to_string(),
            "- [tests/e2e/](tests/e2e): Declaration".to_string(),
            format!("{V2_PLACES}: Gradle build and sources"),
        ]
    );
}
