//! The one rule that picks the name a file's reading is taken from (§AR-config.1.6,
//! §FS-scan.identity.reading), per sort of file, and the guard that no caller gets a reading any
//! other way. Unix-only: the cases are about links.
//! Fixture builders retain their temporary-root owner (§FS-repository-maintenance.4.1).

use std::path::Path;

use crate::config::Config;
use crate::config::load_config;
use crate::testing::{symlink, test_root, write};

const V2: &str = "grund_config_version = 2\nproject_name = \"names\"\n\
    [schema.sources]\nlanguages = [\"markdown\", \"rust\"]\n\
    [schema.sources.extensions]\ntxt = \"rust\"\n";
const V1_EMPTY: &str = "grund_config_version = 1\n[scan]\nextensions = [\"md\", \"\"]\n";

fn project(case: &str, config: &str) -> (crate::testing::TemporaryRoot, Config) {
    let root = test_root(case);
    write(&root.join("grund.toml"), config);
    let config = load_config(&root).expect("the project loads");
    (root, config)
}

/// The language name the rule chose for `link`, and whether the reading it builds is Markdown.
fn chosen(config: &Config, link: &Path) -> (Option<String>, bool) {
    let (schema, frame) = (config.schema(), config.frame());
    let name = frame.read_name(schema, link);
    assert_eq!(name.spelling(), link, "the spelling handed is kept");
    let language = name.language().map(|language| language.name.clone());
    (language, frame.reading(schema, link).markdown)
}

/// §FS-scan.identity.reading: a file whose own name claims a reading is read by it, under a link
/// of any name.
#[test]
fn a_file_whose_own_name_claims_a_reading_is_read_by_that_name() {
    let (root, config) = project("read_name_own", V2);
    write(&root.join("fs/real.txt"), "text\n");
    symlink("real.txt", &root.join("fs/alias.md"));
    assert_eq!(
        chosen(&config, &root.join("fs/alias.md")),
        (Some("rust".to_string()), false)
    );
}

/// §FS-scan.identity.reading.claims: `LICENSE` claims nothing, so each link keeps its own reading.
#[test]
fn a_file_whose_own_name_claims_nothing_keeps_the_reaching_name() {
    let (root, config) = project("read_name_nothing", V2);
    write(&root.join("LICENSE"), "text\n");
    symlink("../LICENSE", &root.join("docs/license.md"));
    symlink("../LICENSE", &root.join("docs/license.rs"));
    assert_eq!(
        chosen(&config, &root.join("docs/license.md")),
        (Some("markdown".to_string()), true)
    );
    assert_eq!(
        chosen(&config, &root.join("docs/license.rs")),
        (Some("rust".to_string()), false)
    );
}

/// §FS-scan.identity.reading.outside: a target outside the config root decides nothing, whatever
/// its name would claim.
#[test]
fn a_file_outside_the_root_keeps_the_reaching_name() {
    let (root, config) = project("read_name_outside", V2);
    let outside = test_root("read_name_outside_target");
    write(&outside.join("real.rs"), "text\n");
    symlink(
        outside.join("real.rs").to_str().expect("a UTF-8 path"),
        &root.join("docs/out.md"),
    );
    assert_eq!(
        chosen(&config, &root.join("docs/out.md")),
        (Some("markdown".to_string()), true)
    );
}

/// §FS-scan.identity.reading.claims, §FS-scan.scope.extensions.empty: a version 1 list holding `""`
/// reads `LICENSE`, so `LICENSE` claims a reading and the link stops being Markdown.
#[test]
fn a_version_1_empty_extension_claims_a_file_with_no_extension() {
    let (root, config) = project("read_name_v1_empty", V1_EMPTY);
    write(&root.join("LICENSE"), "text\n");
    symlink("../LICENSE", &root.join("docs/license.md"));
    let link = root.join("docs/license.md");
    assert_eq!(chosen(&config, &link), (None, false));
    let (schema, frame) = (config.schema(), config.frame());
    assert_eq!(frame.read_name(schema, &link).extension(), None);
}

/// Every Rust source of this crate but the test modules, with its path under `src`.
fn crate_sources(dir: &Path, found: &mut Vec<(String, String)>) {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for entry in std::fs::read_dir(dir).expect("a source directory") {
        let path = entry.expect("a source entry").path();
        if path.is_dir() {
            crate_sources(&path, found);
            continue;
        }
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if name.ends_with(".rs") && !name.starts_with("tests_") {
            let relative = path.strip_prefix(&src).expect("under src");
            let text = std::fs::read_to_string(&path).expect("a readable source");
            found.push((relative.to_string_lossy().replace('\\', "/"), text));
        }
    }
}

/// §AR-config.1.6: no caller can get a reading from a bare spelling. The compiler holds the type:
/// `FileReading::of` takes a `ReadName`, whose fields and constructors are `read_name.rs`'s alone.
/// This fails on the three ways round that the compiler cannot see: a second file asking
/// `FileReading::of`, one calling `as_spelled` were it widened, or one asking `language_of` of a
/// bare path and reading the file from the answer.
#[test]
fn no_caller_gets_a_reading_from_a_bare_spelling() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut sources = Vec::new();
    crate_sources(&src, &mut sources);
    assert!(sources.len() > 50, "the scan found the crate's sources");
    let rule = "config/read_name.rs";
    // `file_reading.rs` asks of a path the config wrote, which is a setting and no file's reading
    // (§AR-config.1.6), and migration asks of a probe it builds.
    let language_owners = [
        rule,
        "config/sources.rs",
        "config/file_reading.rs",
        "config/migrate/records_gate.rs",
    ];
    let mut bypasses = Vec::new();
    for (path, text) in &sources {
        let is_rule = path == rule;
        for token in ["FileReading::of(", "ReadName {", "as_spelled("] {
            if !is_rule && text.contains(token) {
                bypasses.push(format!("{path}: {token}"));
            }
        }
        if !language_owners.contains(&path.as_str()) && text.contains(".language_of(") {
            bypasses.push(format!("{path}: .language_of("));
        }
    }
    assert!(bypasses.is_empty(), "{bypasses:#?}");
}
