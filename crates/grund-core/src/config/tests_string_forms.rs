//! Test module: which multi-line string row and which quote reading a file reads
//! (§FS-languages.multi-line-string.suffix), one case per clause. A built-in reads its own, over
//! its own suffix and over a mapped one; a `source` definition, and a version-1 file, read their
//! own suffix's; a `document` definition reads none. Each reading is observed where it acts,
//! never on the function that answers it: whether a line begins inside a literal
//! (`FileReading::literals`), and whether a qualified citation after a Rust lifetime or a
//! Haskell prime, or inside a literal, is reported by `check` (§FS-citations.recognized.contexts).
//! Fixture builders retain their temporary-root owner (§FS-repository-maintenance.4).

use super::*;
use crate::testing::{TemporaryRoot, check_run, test_root, write};

/// A Rust file: a lifetime before a trailing comment on line 2, a raw string whose interior is
/// line 4.
const RUST: &str = "// \u{a7}FS-a\nfn f(x: &'static str) {} // \u{a7}other/FS-after-lifetime\n\
                    const S: &str = r#\"\n// \u{a7}other/FS-in-raw\n\"#;\n";

/// A Java text block whose interior is line 3.
const JAVA: &str = "// \u{a7}FS-a\nclass Text { String s = \"\"\"\n\
                    // \u{a7}other/FS-in-string\n\"\"\"; }\n";

/// A C++ raw string whose interior is line 3.
const CPP: &str = "// \u{a7}FS-a\nconst char *s = R\"x(\n// \u{a7}other/FS-in-raw\n)x\";\n";

/// A Haskell prime before a trailing comment on line 2.
const HASKELL: &str = "-- \u{a7}FS-a\nf' = 1 -- \u{a7}other/FS-after-prime\n";

/// A version-2 project reading `docs` and `src`, its `[schema.sources]` keys `sources`.
fn v2(sources: &str) -> String {
    format!(
        "grund_config_version = 2\n[schema.sources]\ndeclarations = \"any-comment\"\n{sources}\
         [schema.kinds.FS]\nfolders = [\"docs\"]\nindex = false\n\
         [schema.kinds.code]\nfolders = [\"src\"]\n"
    )
}

/// The project `toml` at a fresh root, with `docs/FS-a.md` and each `(path, text)` under `src/`.
fn tree(name: &str, toml: &str, files: &[(&str, &str)]) -> (Config, TemporaryRoot) {
    let root = test_root(&format!("string-forms-{name}"));
    write(&root.join("grund.toml"), toml);
    write(&root.join("docs/FS-a.md"), "# FS-a: A\n\nText.\n");
    for (path, text) in files {
        write(&root.join("src").join(path), text);
    }
    let config = load_config(&root).unwrap_or_else(|err| panic!("{name} loads: {err:#}"));
    (config, root)
}

/// Whether line `line` (1-based) of `src/<path>` begins inside a multi-line literal.
fn inside(config: &Config, path: &str, line: usize) -> bool {
    let file = config.root.join("src").join(path);
    let text = std::fs::read_to_string(&file).expect("read the fixture file");
    config.reading(&file).literals(&text).entry(line - 1) != 0
}

/// `src/<path>:<line>` of every `unknown project alias` finding `check` reports, sorted.
fn reported(root: &TemporaryRoot) -> Vec<String> {
    let run = check_run(root, false);
    let mut sites: Vec<String> = run
        .report
        .errors
        .iter()
        .filter(|finding| finding.code == "unknown-project")
        .map(|finding| {
            let path = finding.path.as_deref().expect("a located finding");
            let name = path.file_name().expect("a file").to_string_lossy();
            format!("src/{name}:{}", finding.line.unwrap_or(0))
        })
        .collect();
    sites.sort();
    sites
}

/// §FS-languages.multi-line-string.suffix: a built-in reads its own row and quote reading over
/// its own suffix and over a mapped one, because a mapping takes it whole. `txt = "rust"` reads
/// raw strings and lifetimes, `hh = "cpp"` C++ raw strings, built-in `haskell` primes, and
/// `h = "c"` no row, because C has none. This passes before the fix and must keep passing.
#[test]
fn a_builtin_reads_its_own_row_over_its_own_and_a_mapped_suffix() {
    let toml = v2(
        "languages = [\"markdown\", \"rust\", \"c\", \"cpp\", \"haskell\"]\n\
         [schema.sources.extensions]\ntxt = \"rust\"\nh = \"c\"\nhh = \"cpp\"\n",
    );
    let (config, root) = tree(
        "builtin",
        &toml,
        &[
            ("lib.rs", RUST),
            ("a.txt", RUST),
            ("a.h", CPP),
            ("a.hh", CPP),
            ("a.hs", HASKELL),
        ],
    );
    assert!(inside(&config, "lib.rs", 4), "rust over .rs");
    assert!(inside(&config, "a.txt", 4), "rust over .txt");
    assert!(inside(&config, "a.hh", 3), "cpp over .hh");
    assert!(!inside(&config, "a.h", 3), "c over .h");
    assert_eq!(
        reported(&root),
        ["src/a.h:3", "src/a.hs:2", "src/a.txt:2", "src/lib.rs:2"]
    );
}

/// §FS-languages.multi-line-string.suffix, §FS-config-v2.schema.sources.definitions: a file read
/// through a `source` definition reads the row its own suffix names, so `java = "myjava"` reads
/// Java text blocks and the citation inside one is string data.
#[test]
fn a_source_definition_over_java_reads_text_blocks() {
    let toml = v2(
        "languages = [\"markdown\", \"myjava\"]\n[schema.sources.extensions]\njava = \"myjava\"\n\
         [schema.sources.definitions.myjava]\nclass = \"source\"\ncomment_prefixes = [\"//\"]\n",
    );
    let (config, root) = tree("definition-java", &toml, &[("Text.java", JAVA)]);
    assert!(inside(&config, "Text.java", 3));
    assert_eq!(reported(&root), Vec::<String>::new());
}

/// §FS-languages.multi-line-string.suffix: a `source` definition over `.rs` reads Rust raw
/// strings and Rust's quote reading, so the citation after `'static` is reported and the one in
/// the raw string is not, as version 1 reads the same file.
#[test]
fn a_source_definition_over_rs_reads_raw_strings_and_lifetimes() {
    let toml = v2(
        "languages = [\"markdown\", \"myrs\"]\n[schema.sources.extensions]\nrs = \"myrs\"\n\
         [schema.sources.definitions.myrs]\nclass = \"source\"\ncomment_prefixes = [\"//\"]\n",
    );
    let (config, root) = tree("definition-rs", &toml, &[("lib.rs", RUST)]);
    assert!(inside(&config, "lib.rs", 4));
    assert_eq!(reported(&root), ["src/lib.rs:2"]);
}

/// §FS-languages.multi-line-string.suffix: a `source` definition over `.hs` reads the Haskell
/// prime, which no row of the forms table names, so the citation after `f'` is reported, as it
/// is under the built-in `haskell` and under version 1.
#[test]
fn a_source_definition_over_hs_reads_haskell_primes() {
    let toml = v2(
        "languages = [\"markdown\", \"myhs\"]\n[schema.sources.extensions]\nhs = \"myhs\"\n\
         [schema.sources.definitions.myhs]\nclass = \"source\"\ncomment_prefixes = [\"--\"]\n",
    );
    let (_config, root) = tree("definition-hs", &toml, &[("a.hs", HASKELL)]);
    assert_eq!(reported(&root), ["src/a.hs:2"]);
}

/// §FS-languages.multi-line-string.suffix, §FS-languages.multi-line-string.unrecognized: a
/// `source` definition over a suffix that names no row reads none, so a C++ raw-string shape
/// in a `.txt` file opens nothing. This passes before the fix and must keep passing.
#[test]
fn a_source_definition_over_txt_reads_no_row() {
    let toml = v2(
        "languages = [\"markdown\", \"mytxt\"]\n[schema.sources.extensions]\ntxt = \"mytxt\"\n\
         [schema.sources.definitions.mytxt]\nclass = \"source\"\ncomment_prefixes = [\"//\"]\n",
    );
    let (config, root) = tree("definition-txt", &toml, &[("a.txt", CPP)]);
    assert!(!inside(&config, "a.txt", 3));
    assert_eq!(reported(&root), ["src/a.txt:3"]);
}

/// §FS-languages.multi-line-string.suffix: a file read through a `document` definition reads no
/// row, whatever its suffix. This passes before the fix and must keep passing.
#[test]
fn a_document_definition_over_rs_reads_no_row() {
    let toml = v2(
        "languages = [\"markdown\", \"mydoc\"]\n[schema.sources.extensions]\nrs = \"mydoc\"\n\
         [schema.sources.definitions.mydoc]\nclass = \"document\"\n",
    );
    let (config, _root) = tree("definition-document", &toml, &[("lib.rs", RUST)]);
    assert!(!inside(&config, "lib.rs", 4));
}

/// §FS-languages.multi-line-string.suffix: a version-1 file reads the row and quote reading its
/// extension names. This is the reading migration must keep; it passes before the fix and must
/// keep passing.
#[test]
fn a_version_1_file_reads_its_extension_row_and_quote_reading() {
    let toml = "grund_config_version = 1\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\n\
                index = false\n[scan]\ninclude = [\"docs\", \"src\"]\n\
                extensions = [\"md\", \"rs\", \"java\", \"hs\"]\n";
    let (config, root) = tree(
        "version-1",
        toml,
        &[("lib.rs", RUST), ("Text.java", JAVA), ("a.hs", HASKELL)],
    );
    assert!(inside(&config, "lib.rs", 4));
    assert!(inside(&config, "Text.java", 3));
    assert_eq!(reported(&root), ["src/a.hs:2", "src/lib.rs:2"]);
}
