//! Test module: the v1 `[scan] extensions` default follows grund's built-in
//! language table (§FS-scan.scope.extensions.default), and the copies `grund
//! init` writes out as teaching material and the JSON Schema publishes say the same
//! (§FS-init.2.4.3, §FS-config.4). And what the list says of a file with no extension
//! (§FS-scan.scope.extensions.empty): the one test's answers, and that every writer of the
//! list leaves the record's reading following it.
//! Fixture builders retain their temporary-root owner (§FS-repository-maintenance.4.1).

use std::path::{Path, PathBuf};

use super::project::Sources;
use super::v1::default_project;
use super::{Config, load_config};
use crate::grammar::{DocCommentRule, classified_extensions, extension_rule};
use crate::testing::{test_root, write};

fn default_extensions() -> Vec<String> {
    default_project().schema.sources.extensions
}

/// The default before #705. A language the table gains is appended, so every
/// earlier default stays a prefix of the current one.
const PRE_705_DEFAULT: &[&str] = &[
    "md", "rs", "go", "java", "kt", "ts", "tsx", "js", "py", "c", "cpp", "swift", "scala", "rb",
    "php", "cs", "lisp", "scm", "clj", "sql", "hs", "lhs", "lua", "ada", "adb", "ads",
];

/// Every extension §FS-inline-citation-style.1.1.2 gives a doc-comment recognizer.
const SPEC_CLASSIFIED: &[&str] = &[
    "rs", "c", "h", "cpp", "cc", "cxx", "hpp", "hh", "hxx", "m", "mm", "java", "cs", "kt", "kts",
    "scala", "swift", "js", "jsx", "mjs", "cjs", "ts", "tsx", "php", "dart", "py", "lua", "hs",
    "lhs", "r", "R", "go", "rb", "sh", "bash", "zsh", "sql",
];

#[test]
fn default_is_md_then_the_language_table_in_order() {
    let expected: Vec<String> = std::iter::once("md")
        .chain(classified_extensions())
        .map(str::to_string)
        .collect();
    let actual = default_extensions();
    assert_eq!(actual, expected);
    assert_eq!(&actual[..PRE_705_DEFAULT.len()], PRE_705_DEFAULT);
}

#[test]
fn every_classified_extension_is_in_the_default() {
    let default = default_extensions();
    for extension in SPEC_CLASSIFIED {
        assert!(
            !matches!(extension_rule(extension), DocCommentRule::None),
            "`{extension}` should have a doc-comment recognizer"
        );
        assert!(
            default.iter().any(|listed| listed == extension),
            "`{extension}` is classified but the default does not read it"
        );
    }
    // A row with no recognizer still names an extension the default reads.
    for extension in classified_extensions() {
        assert!(default.iter().any(|listed| listed == extension));
    }
}

/// The quoted strings of the template's `extensions = [...]` line.
/// §FS-init.2.4.3: the starters write no `extensions`, so a new project
/// takes the compiled default rather than a copy that could fall behind it.
#[test]
fn init_templates_leave_extensions_at_the_compiled_default() {
    let templates = [
        include_str!("../../assets/templates/software-development/grund.toml"),
        include_str!("../../assets/templates/none/grund.toml"),
    ];
    for template in templates {
        assert!(!template.contains("extensions"), "{template}");
    }
}

/// The JSON Schema publishes each key's default (§FS-config.4), and the
/// reference page is rendered from it, so its copy must be the compiled one.
#[test]
fn schema_publishes_the_compiled_default() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../schemas/grund.schema.json");
    let schema: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("the schema is readable"))
            .expect("the schema is JSON");
    let published: Vec<String> = serde_json::from_value(
        schema["definitions"]["v1"]["properties"]["scan"]["properties"]["extensions"]["default"]
            .clone(),
    )
    .expect("the schema publishes an extensions default");
    assert_eq!(published, default_extensions());
}

/// `sources.lists_extension(path)` under `["md"]`, `["md", ""]` and `[""]`, in that order.
fn listed_under(path: &Path) -> [bool; 3] {
    let lists: [&[&str]; 3] = [&["md"], &["md", ""], &[""]];
    lists.map(|list| {
        let mut sources = default_project().schema.sources;
        sources.set_extensions(list.iter().map(ToString::to_string).collect());
        sources.lists_extension(path)
    })
}

/// §FS-scan.scope.extensions.empty: the empty string lists a name with no extension, and only the
/// empty string does; a name with an extension goes by that extension whatever else is listed.
#[test]
fn the_empty_string_lists_a_name_with_no_extension_and_no_other() {
    assert_eq!(listed_under(Path::new("FACTS")), [false, true, true]);
    assert_eq!(listed_under(Path::new("FACTS.")), [false, true, true]);
    assert_eq!(listed_under(Path::new("a.md")), [true, true, false]);
}

/// §FS-scan.scope.extensions.empty: an extension that is not UTF-8 is no extension to list.
#[cfg(unix)]
#[test]
fn the_empty_string_lists_a_name_whose_extension_is_not_utf8() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let name = Path::new(OsStr::from_bytes(b"FACTS.\xff"));
    assert_eq!(listed_under(name), [false, true, true]);
}

/// The façade of a file holding `toml`.
fn loaded(name: &str, toml: &str) -> crate::testing::Rooted<Config> {
    let root = test_root(&format!("scan-extensions-{name}"));
    write(&root.join("grund.toml"), toml);
    let config = load_config(&root).expect("the file loads");
    root.hold(config)
}

fn sources_of(config: &Config) -> &Sources {
    &config.project().schema.sources
}

const V1_HOLDING_THE_ENTRY: &str =
    "grund_config_version = 1\n\n[scan]\nextensions = [\"md\", \"\"]\n";

/// §FS-scan.scope.extensions.empty: the version 1 reader, with the entry and without it.
#[test]
fn the_version_1_reader_reads_no_extension_only_through_the_entry() {
    let holding = loaded("v1-holding", V1_HOLDING_THE_ENTRY);
    assert!(sources_of(&holding).reads_no_extension);
    assert!(sources_of(&holding).lists_extension(Path::new("FACTS")));

    let without = loaded(
        "v1-without",
        "grund_config_version = 1\n\n[scan]\nextensions = [\"md\"]\n",
    );
    assert!(!sources_of(&without).reads_no_extension);
    assert!(!sources_of(&without).lists_extension(Path::new("FACTS")));
}

/// §FS-scan.scope.extensions.empty: version 2 has no such entry, so its list reads none of these.
#[test]
fn a_version_2_file_reads_no_extension() {
    let config = loaded(
        "v2",
        "grund_config_version = 2\nproject_name = \"example\"\n[schema.sources]\nlanguages = [\"markdown\"]\n",
    );
    assert!(!sources_of(&config).reads_no_extension);
    assert!(!sources_of(&config).lists_extension(Path::new("FACTS")));
}

/// §AR-config.5, §FS-scan.scope.extensions.empty: a record the façade rebuilds from its list holds
/// the reading that list gives, whether the edit added the entry or took it away.
#[test]
fn a_rebuilt_record_reads_what_its_list_holds() {
    let mut added = Config::default_for(PathBuf::from("/repo"));
    assert!(!sources_of(&added).reads_no_extension);
    added.extensions.push(String::new());
    assert!(sources_of(&added).reads_no_extension);

    let mut removed = loaded("facade-removed", V1_HOLDING_THE_ENTRY);
    assert!(sources_of(&removed).reads_no_extension);
    removed.extensions.retain(|ext| !ext.is_empty());
    assert!(!sources_of(&removed).reads_no_extension);
}
