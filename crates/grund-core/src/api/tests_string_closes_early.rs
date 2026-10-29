//! Test module: a version-2 string ends at its first unescaped quote
//! (§FS-config-v2.reader.6). From grund 0.22.0 a string that closes early is refused at
//! its key's line as `expected string`, ending with the release that made it one where
//! it loaded through the 0.21.x window (§FS-config-v2.reader.6.refusal); a `"""` string
//! on one line is refused with a parenthetical naming the form
//! (§FS-config-v2.reader.6.multi-line). A string that closes at its end, a value refused
//! before, a `"""` string over a break and a version-1 file keep what they had.

use std::path::{Path, PathBuf};

use super::config_findings::config_diagnostics;
use crate::config::load_config;
use crate::testing::{test_root, write};

/// The version-2 config every case extends, its next line line 8.
const HEAD: &str = "grund_config_version = 2\n\n[schema.kinds.code]\nfolders = [\"src\"]\n\n\
                    [schema.kinds.SPEC]\nfiles = [\"specs/spec.md\"]\n";

/// A clean version-2 project whose `grund.toml` is [`HEAD`] followed by `tail`.
fn v2(name: &str, tail: &str) -> PathBuf {
    project(name, &format!("{HEAD}{tail}"))
}

fn project(name: &str, config: &str) -> PathBuf {
    let root = test_root(&format!("string-closes-early-{name}"));
    write(&root.join("grund.toml"), config);
    write(&root.join("specs/spec.md"), "# SPEC-one: One\n\nLead.\n");
    write(
        &root.join("src/main.rs"),
        "// Realizes \u{a7}SPEC-one.\nfn main() {}\n",
    );
    root
}

/// The open-line and multi-line findings the config at `root` carries: code, line and
/// message.
fn window_findings(root: &Path) -> Vec<(&'static str, Option<usize>, String)> {
    let config = load_config(root).expect("the config loads");
    config_diagnostics(&config, config.display())
        .map(|d| (d.code.as_str(), d.line, d.message))
        .filter(|(code, ..)| matches!(*code, "config-not-toml" | "config-multiline-string"))
        .collect()
}

/// Require the config at `root` to be refused, the load's error ending with `expected`.
fn refused(root: &Path, expected: &str) {
    let Err(err) = load_config(root) else {
        panic!("the config is refused");
    };
    let err = format!("{err:#}");
    assert!(err.ends_with(expected), "{err}");
}

/// The 0.22.0 refusal of a string that closes early in the value at line `key`, which
/// loaded through the 0.21.x window.
fn suffixed(key: usize) -> String {
    format!("grund.toml:{key}: expected string; this became an error in grund 0.22.0")
}

/// The 0.22.0 refusal of a `"""` string on one line in the value at line `key`.
fn multi_line(key: usize) -> String {
    format!(
        "grund.toml:{key}: expected string (a multi-line string, which version 2 does not \
         read); this became an error in grund 0.22.0"
    )
}

/// The SPEC kind's `fetch`, as the load reads it.
fn spec_fetch(root: &Path) -> Option<String> {
    let config = load_config(root).expect("the config loads");
    let spec = config
        .kinds
        .iter()
        .find(|k| k.kind == "SPEC")
        .expect("SPEC");
    spec.fetch.clone()
}

/// §FS-config-v2.reader.6.refusal: a scalar that closes early is refused with the suffix.
#[test]
#[ignore = "red until #1094's implementation; implement removes this"]
fn a_scalar_that_closes_early_is_refused() {
    refused(&v2("scalar", "fetch = \"one\" \"two\"\n"), &suffixed(8));
}

/// §FS-config-v2.reader.6.refusal: an element that closes early is refused with the
/// suffix.
#[test]
#[ignore = "red until #1094's implementation; implement removes this"]
fn an_element_that_closes_early_is_refused() {
    let root = v2(
        "element",
        "\n[schema.sources]\nexclude = [\"docs\" \"specs\"]\n",
    );
    refused(&root, &suffixed(10));
}

/// §FS-config-v2.reader.6.refusal: a value is refused once however many of its strings
/// close early.
#[test]
#[ignore = "red until #1094's implementation; implement removes this"]
fn a_value_with_two_early_closes_is_refused() {
    let root = v2(
        "two",
        "\n[schema.sources]\nexclude = [\"a\" \"b\", \"c\" \"d\"]\n",
    );
    refused(&root, &suffixed(10));
}

/// §FS-config-v2.reader.6.refusal: in an array written across lines the refusal is at
/// the key's line, as every value's is.
#[test]
#[ignore = "red until #1094's implementation; implement removes this"]
fn an_element_on_a_later_line_is_refused_at_its_key() {
    let root = v2(
        "later",
        "\n[schema.sources]\nexclude = [\n  \"a\",\n  \"b\" \"c\",\n]\n",
    );
    refused(&root, &suffixed(10));
}

/// §FS-config-v2.reader.6.refusal: a quoted key is a string of its own, and is refused.
#[test]
#[ignore = "red until #1094's implementation; implement removes this"]
fn a_quoted_key_that_closes_early_is_refused() {
    let root = v2(
        "key",
        "\n[schema.sources.extensions]\n\"md\" \"x\" = \"markdown\"\n",
    );
    refused(&root, &suffixed(10));
}

/// §FS-config-v2.reader.6: an escaped quote before the early close, or an escaped
/// backslash before the closing quote, leaves the string closing early, and refused.
#[test]
#[ignore = "red until #1094's implementation; implement removes this"]
fn escapes_before_an_early_close_are_refused() {
    refused(
        &v2("escaped-quote", "fetch = \"a\\\"b\" \"c\"\n"),
        &suffixed(8),
    );
    let root = v2("escaped-backslash", "fetch = \"a\\\\\" \"b\"\n");
    refused(&root, &suffixed(8));
}

/// §FS-config-v2.reader.6.multi-line: a `"""` string on one line is refused with the
/// parenthetical, then the suffix.
#[test]
#[ignore = "red until #1094's implementation; implement removes this"]
fn a_triple_quoted_string_on_one_line_is_refused() {
    let root = v2(
        "triple",
        "\n[presentation.kinds.SPEC]\ntitle = \"\"\"one\"\"\"\n",
    );
    refused(&root, &multi_line(10));
}

/// §FS-config-v2.reader.6.refusal: an array with an open line is joined first, and its
/// string that closes early is then refused; the config does not load, so it carries no
/// open-line warning.
#[test]
#[ignore = "red until #1094's implementation; implement removes this"]
fn an_open_line_and_an_early_close_are_refused_as_the_early_close() {
    let root = v2(
        "both",
        "\n[schema.sources]\nexclude = [\n  \"a\nb\" \"c\",\n]\n",
    );
    refused(&root, &suffixed(10));
}

/// §FS-config-v2.reader.6.multi-line: the `"""` string is the one its third quote opens,
/// so a break inside a plain string after it leaves the element a one-line `"""` string,
/// refused with its parenthetical once the array's lines are joined.
#[test]
#[ignore = "red until #1094's implementation; implement removes this"]
fn a_break_after_a_one_line_triple_quoted_string_is_refused() {
    let root = v2(
        "triple-then-break",
        "\n[schema.sources]\nexclude = [\n  \"\"\"a\"\"\" \"b\nc\",\n]\n",
    );
    refused(&root, &multi_line(10));
}

/// Guard, green today and kept: a `"""` string a line break runs through is one
/// construct, warned for by §FS-config-v2.reader.5.strings.multi-line alone.
#[test]
fn a_triple_quoted_string_over_a_break_earns_only_the_open_line_warning() {
    let root = v2(
        "triple-break",
        "\n[schema.sources]\nexclude = [\n  \"\"\"a\nb\"\"\",\n]\n",
    );
    let message = "grund.toml:10: unclosed array (a multi-line string opens on line 11, which \
                   version 2 does not read), read for now with the line break as a space; \
                   write the string on line 11 as one basic string; this warning becomes an \
                   error in grund 0.22.0";
    assert_eq!(
        window_findings(&root),
        vec![("config-multiline-string", Some(10), message.to_string())]
    );
}

/// §FS-config-v2.reader.6.multi-line: every `"""` string a line break runs through is
/// #1089's construct, not only the one across the array's first open line, so two of them
/// earn its one warning alone.
#[test]
fn two_triple_quoted_strings_over_breaks_earn_only_the_open_line_warning() {
    let root = v2(
        "two-triple-breaks",
        "\n[schema.sources]\nexclude = [\n  \"\"\"a\nb\"\"\",\n  \"\"\"c\nd\"\"\",\n]\n",
    );
    let message = "grund.toml:10: unclosed array (a multi-line string opens on line 11, which \
                   version 2 does not read), read for now with the line break as a space; \
                   write the string on line 11 as one basic string; this warning becomes an \
                   error in grund 0.22.0";
    assert_eq!(
        window_findings(&root),
        vec![("config-multiline-string", Some(10), message.to_string())]
    );
}

/// §FS-config-v2.reader.6.multi-line: a `"""` string over a break after a plain open line
/// is #1089's construct too, and earns nothing beyond that array's one warning.
#[test]
fn a_triple_quoted_string_over_a_break_after_a_plain_open_line_earns_only_that_warning() {
    let root = v2(
        "plain-then-triple",
        "\n[schema.sources]\nexclude = [\n  \"a\nb\",\n  \"\"\"c\nd\"\"\",\n]\n",
    );
    let open = "grund.toml:10: unclosed array (a string is still open at the end of line 11), \
                read for now with the line break as a space; close the string on line 11; this \
                warning becomes an error in grund 0.22.0";
    assert_eq!(
        window_findings(&root),
        vec![("config-not-toml", Some(10), open.to_string())]
    );
}

/// Guard, green today and kept: strings that close at their end, an escaped quote, an
/// escaped backslash and a comment after the value among them, earn nothing.
#[test]
fn strings_that_close_at_their_end_earn_nothing() {
    let root = v2(
        "valid",
        "fetch = \"a\\\\\" # a comment\n\n[schema.sources]\nexclude = [\"a\\\"b\", \"c\\\\d\"]\n",
    );
    assert_eq!(spec_fetch(&root).as_deref(), Some("a\\"));
    assert_eq!(window_findings(&root), vec![]);
}

/// Guard, green today and kept: a value the readers before refused is refused as
/// before, with the bare `expected string` (§FS-config-v2.reader.6.refusal).
#[test]
fn a_value_refused_before_is_refused_as_before() {
    let root = v2(
        "target",
        "\n[schema.sources]\nexclude = [\"target\" \"a,b\"]\n",
    );
    refused(&root, "grund.toml:10: expected string");
}

/// Guard, green today and kept: version 1 keeps its reading for good and earns no
/// warning of its own (§FS-config.5.2); only the version-1 caution is carried.
#[test]
fn version_1_keeps_its_reading_and_earns_nothing_new() {
    let root = project(
        "v1",
        "grund_config_version = 1\n\n[[kinds]]\nkind = \"SPEC\"\nfolder = \"docs\" \"x\"\n\
         index = false\n",
    );
    let config = load_config(&root).expect("the config loads");
    let spec = config
        .kinds
        .iter()
        .find(|k| k.kind == "SPEC")
        .expect("SPEC");
    assert_eq!(spec.folder.as_deref(), Some("docs\" \"x"));
    let codes: Vec<&str> = config_diagnostics(&config, config.display())
        .map(|d| d.code.as_str())
        .collect();
    assert_eq!(codes, vec!["config-version-1"]);
}
