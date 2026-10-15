//! Test module: the v1 `[scan] extensions` default follows grund's built-in
//! language table (§FS-scan.scope.extensions.default), and the copies `grund
//! init` writes out as teaching material and the JSON Schema publishes say the same
//! (§FS-init.2.4.3, §FS-config.4).

use std::path::Path;

use super::v1::default_project;
use crate::grammar::{DocCommentRule, classified_extensions, extension_rule};

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
