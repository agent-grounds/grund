//! Test module: `[schema.sources]` in a v2 file (§FS-config-v2.schema.sources),
//! held at the loader: what loads, what each refusal says and the line it is
//! located at (§FS-config.4.3), and that `config show` prints the source model in
//! v2 spelling that loads back to itself (§FS-config-v2.schema.sources.effective).

use super::*;
use crate::testing::{test_root, write};

/// The envelope and the `[schema.sources]` header; a case's keys start at line 4.
const HEAD: &str = "grund_config_version = 2\nproject_name = \"example\"\n[schema.sources]\n";
const ROW: &str = "[schema.kinds.FS]\n";

fn load(name: &str, toml: &str) -> anyhow::Result<Config> {
    let root = test_root(name);
    write(&root.join("grund.toml"), toml);
    load_config(&root)
}

fn loads(name: &str, toml: &str) -> Project {
    match load(name, toml) {
        Ok(config) => config.project().clone(),
        Err(err) => panic!("{name}: the v2 file should load: {err:#}"),
    }
}

/// The refusal's text, which must name `grund.toml:<line>:` and carry `message`.
fn refused(name: &str, toml: &str, line: usize, message: &str) {
    let err = match load(name, toml) {
        Ok(_) => panic!("{name}: the file should be refused"),
        Err(err) => format!("{err:#}"),
    };
    let at = format!("grund.toml:{line}:");
    assert!(err.contains(&at), "{name}: expected {at}, got: {err}");
    assert!(
        err.contains(message),
        "{name}: expected `{message}`, got: {err}"
    );
}

/// §FS-config-v2.schema.sources: the published example loads — languages, a
/// remapped suffix, a definition the mapping reaches, the mode and the globs.
#[test]
fn v2_sources_example_loads() {
    let toml = format!(
        "{HEAD}languages = [\"markdown\", \"rust\", \"notes\"]\ndeclarations = \"doc-comments\"\n\
         exclude = [\"generated\", \"/docs/archive\", \"src/**/vendor/**\"]\n\
         [schema.sources.extensions]\nrustdoc = \"rust\"\nnote = \"notes\"\n\
         [schema.sources.definitions.notes]\nclass = \"source\"\ncomment_prefixes = [\"%%\"]\n{ROW}"
    );
    assert_eq!(loads("v2_sources_example", &toml).version, 2);
}

/// §FS-config-v2.schema.sources.languages, §FS-config-v2.defaults.2: an omitted
/// list is the epoch's default set, and `[]` is a list that admits nothing.
#[test]
fn v2_sources_omitted_and_empty_languages_load() {
    let omitted = format!("grund_config_version = 2\n{ROW}");
    assert_eq!(loads("v2_sources_omitted", &omitted).version, 2);
    let empty = format!("{HEAD}languages = []\n{ROW}");
    assert_eq!(loads("v2_sources_empty", &empty).version, 2);
}

/// §FS-config-v2.schema.sources.epoch: an opt-in row loads when it is named, and
/// `declarations = "any-comment"` is the other mode.
#[test]
fn v2_sources_opt_in_rows_and_any_comment_load() {
    let toml = format!(
        "{HEAD}languages = [\"markdown\", \"shell\", \"r\", \"objectivec\", \"dart\"]\n\
         declarations = \"any-comment\"\n{ROW}"
    );
    assert_eq!(loads("v2_sources_opt_in", &toml).version, 2);
}

/// §FS-config-v2.schema.sources.definitions: every documented key and value,
/// including a document class that keeps prefixes and a docstring-only source.
#[test]
fn v2_sources_definitions_take_every_documented_form() {
    let toml = format!(
        "{HEAD}languages = [\"notes\", \"pydoc\", \"pages\", \"facts\"]\n\
         [schema.sources.definitions.notes]\nclass = \"source\"\ncomment_prefixes = [\"//\", \"#\"]\n\
         doc_comments = \"c-family\"\ndocstrings = \"off\"\n\
         [schema.sources.definitions.pydoc]\nclass = \"source\"\ndocstrings = \"python\"\n\
         doc_comments = \"python\"\n\
         [schema.sources.definitions.pages]\nclass = \"document\"\ncomment_prefixes = [\"#\"]\n\
         [schema.sources.definitions.facts]\nclass = \"data\"\n{ROW}"
    );
    assert_eq!(loads("v2_sources_definitions", &toml).version, 2);
}

/// §FS-config-v2.schema.sources.languages: an unknown or repeated language is
/// refused at the `languages` line.
#[test]
fn v2_sources_language_refusals_are_located() {
    let cases = [
        (
            "unknown",
            "languages = [\"markdown\", \"cobol\"]\n",
            "unknown language `cobol` in [schema.sources] languages",
        ),
        (
            "twice",
            "languages = [\"markdown\", \"rust\", \"rust\"]\n",
            "language `rust` is listed twice in [schema.sources] languages",
        ),
    ];
    for (name, body, message) in cases {
        let toml = format!("{HEAD}{body}{ROW}");
        refused(&format!("v2_sources_lang_{name}"), &toml, 4, message);
    }
}

/// §FS-config-v2.schema.sources.extensions: a mapping names an admitted language
/// through a literal final extension, refused at the entry's line otherwise.
#[test]
fn v2_sources_extension_refusals_are_located() {
    let cases = [
        (
            "unknown_target",
            "languages = [\"markdown\"]\n[schema.sources.extensions]\nnote = \"notes\"\n",
            "unknown language `notes` in [schema.sources.extensions]",
        ),
        (
            "unadmitted_target",
            "languages = [\"markdown\"]\n[schema.sources.extensions]\nrustdoc = \"rust\"\n",
            "[schema.sources.extensions] rustdoc maps to `rust`, which [schema.sources] languages does not admit",
        ),
        (
            "glob_suffix",
            "languages = [\"rust\"]\n[schema.sources.extensions]\n\"*.rs\" = \"rust\"\n",
            "invalid suffix `*.rs` in [schema.sources.extensions]",
        ),
        (
            "dotted_suffix",
            "languages = [\"rust\"]\n[schema.sources.extensions]\n\"tar.rs\" = \"rust\"\n",
            "invalid suffix `tar.rs` in [schema.sources.extensions]",
        ),
        (
            "unadmitted_definition",
            "languages = [\"markdown\"]\n[schema.sources.extensions]\nnote = \"notes\"\n\
             [schema.sources.definitions.notes]\nclass = \"source\"\ncomment_prefixes = [\"%%\"]\n",
            "[schema.sources.extensions] note maps to `notes`, which [schema.sources] languages does not admit",
        ),
    ];
    for (name, body, message) in cases {
        let toml = format!("{HEAD}{body}{ROW}");
        refused(&format!("v2_sources_ext_{name}"), &toml, 6, message);
    }
}

/// §FS-config-v2.schema.sources.definitions: each invalid definition is refused
/// at the line of the header or key that breaks it.
#[test]
fn v2_sources_definition_refusals_are_located() {
    let def = "languages = [\"notes\"]\n[schema.sources.definitions.notes]\n";
    let cases: [(&str, String, usize, &str); 12] = [
        (
            "builtin",
            "languages = [\"rust\"]\n[schema.sources.definitions.rust]\nclass = \"source\"\n\
             comment_prefixes = [\"//\"]\n"
                .to_string(),
            5,
            "cannot redefine the built-in language `rust`",
        ),
        (
            "bad_name",
            "languages = [\"markdown\"]\n[schema.sources.definitions.Notes]\nclass = \"source\"\n\
             comment_prefixes = [\"%%\"]\n"
                .to_string(),
            5,
            "invalid language name `Notes` (expected [a-z][a-z0-9_]*)",
        ),
        (
            "no_class",
            format!("{def}comment_prefixes = [\"%%\"]\n"),
            5,
            "[schema.sources.definitions.notes] needs `class`",
        ),
        (
            "unknown_class",
            format!("{def}class = \"binary\"\n"),
            6,
            "`binary`",
        ),
        (
            "data_prefixes",
            format!("{def}class = \"data\"\ncomment_prefixes = [\"%%\"]\n"),
            7,
            "[schema.sources.definitions.notes] class = \"data\" takes no comment_prefixes",
        ),
        (
            "document_profile",
            format!("{def}class = \"document\"\ndoc_comments = \"c-family\"\n"),
            7,
            "[schema.sources.definitions.notes] class = \"document\" takes no doc_comments",
        ),
        (
            "no_host",
            format!("{def}class = \"source\"\n"),
            5,
            "[schema.sources.definitions.notes] reads no comment",
        ),
        (
            "python_profile_without_docstrings",
            format!(
                "{def}class = \"source\"\ncomment_prefixes = [\"#\"]\ndoc_comments = \"python\"\n"
            ),
            8,
            "docstrings = \"python\"",
        ),
        (
            "unreachable_marker",
            format!(
                "{def}class = \"source\"\ncomment_prefixes = [\"#\"]\ndoc_comments = \"c-family\"\n"
            ),
            8,
            "doc_comments = \"c-family\" is unreachable",
        ),
        (
            "unknown_profile",
            format!(
                "{def}class = \"source\"\ncomment_prefixes = [\"#\"]\ndoc_comments = \"rdoc\"\n"
            ),
            8,
            "`rdoc`",
        ),
        (
            "repeated_prefix",
            format!("{def}class = \"source\"\ncomment_prefixes = [\"%%\", \"%%\"]\n"),
            7,
            "`%%`",
        ),
        (
            "unknown_key",
            format!("{def}class = \"source\"\ncomment_prefixes = [\"%%\"]\nsyntax = \"rust\"\n"),
            8,
            "unknown key `syntax` in [schema.sources.definitions.notes]",
        ),
    ];
    for (name, body, line, message) in cases {
        let toml = format!("{HEAD}{body}{ROW}");
        refused(&format!("v2_sources_def_{name}"), &toml, line, message);
    }
}

/// §FS-config-v2.schema.sources.declarations, §FS-config-v2.schema.sources.exclude:
/// an unknown mode and a pattern the glob parser refuses fail at their line.
#[test]
fn v2_sources_mode_and_glob_refusals_are_located() {
    let mode = format!("{HEAD}languages = [\"rust\"]\ndeclarations = \"notes-only\"\n{ROW}");
    refused(
        "v2_sources_mode",
        &mode,
        5,
        "unknown [schema.sources] declarations `notes-only` (expected doc-comments or any-comment)",
    );
    let glob = format!("{HEAD}languages = [\"rust\"]\nexclude = [\"src/[a\"]\n{ROW}");
    refused("v2_sources_glob", &glob, 5, "src/[a");
}

/// §FS-config-v2.schema.sources.effective: `config show` writes the whole source
/// model — an omitted list as the epoch's default rows — and it loads back to
/// the same text.
#[test]
fn v2_sources_config_show_round_trips() {
    let toml = format!(
        "{HEAD}languages = [\"markdown\", \"rust\", \"notes\"]\nexclude = [\"generated\"]\n\
         [schema.sources.extensions]\nrustdoc = \"rust\"\nnote = \"notes\"\n\
         [schema.sources.definitions.notes]\nclass = \"source\"\ncomment_prefixes = [\"%%\"]\n{ROW}"
    );
    let shown = loads("v2_sources_show", &toml)
        .v2_toml()
        .expect("a v2 file prints in v2 spelling");
    for line in [
        "languages = [\"markdown\", \"rust\", \"notes\"]",
        "declarations = \"doc-comments\"",
        "exclude = [\"generated\"]",
        "[schema.sources.extensions]\nrustdoc = \"rust\"\nnote = \"notes\"",
        "[schema.sources.definitions.notes]\nclass = \"source\"\ncomment_prefixes = [\"%%\"]\n\
         doc_comments = \"none\"\ndocstrings = \"off\"",
    ] {
        assert!(shown.contains(line), "expected `{line}` in:\n{shown}");
    }
    let again = loads("v2_sources_show_again", &shown).v2_toml();
    assert_eq!(again.as_deref(), Some(shown.as_str()));

    let epoch = loads(
        "v2_sources_show_epoch",
        &format!("grund_config_version = 2\n{ROW}"),
    )
    .v2_toml()
    .expect("a v2 file prints in v2 spelling");
    let default_rows = "languages = [\"markdown\", \"json\", \"rust\", \"c\", \"cpp\", \"java\", \
         \"kotlin\", \"scala\", \"swift\", \"javascript\", \"typescript\", \"csharp\", \"php\", \
         \"python\", \"go\", \"ruby\", \"sql\", \"lua\", \"haskell\", \"lisp\", \"scheme\", \
         \"clojure\", \"ada\"]";
    assert!(
        epoch.contains(default_rows),
        "expected the epoch in:\n{epoch}"
    );
}
