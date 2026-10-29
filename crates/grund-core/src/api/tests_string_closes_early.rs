//! Test module: a version-2 string ends at its first unescaped quote
//! (§FS-config-v2.reader.6). Through grund 0.21.x a string that closes early is read
//! as before and earns one warning per value at its key's line, naming the string as
//! written and the line it closes on (§FS-config-v2.reader.6.window,
//! §FS-config-v2.checks.config-not-toml); a `"""` string on one line earns
//! `config-multiline-string` instead (§FS-config-v2.reader.6.multi-line). A string
//! that closes at its end, a value refused before, a `"""` string over a break and a
//! version-1 file earn nothing new.

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

/// The window's findings the config at `root` carries: code, line and message.
fn window_findings(root: &Path) -> Vec<(&'static str, Option<usize>, String)> {
    let config = load_config(root).expect("the config loads");
    config_diagnostics(&config, config.display())
        .map(|d| (d.code.as_str(), d.line, d.message))
        .filter(|(code, ..)| matches!(*code, "config-not-toml" | "config-multiline-string"))
        .collect()
}

/// An element's warning at `key`, naming `text` and the line it closes on, `close`.
fn element(key: usize, text: &str, close: usize) -> (&'static str, Option<usize>, String) {
    let message = format!(
        "grund.toml:{key}: expected string, read for now as the one string `{text}`; end the \
         string on line {close} at its closing quote: write `\\\"` for a quote inside it, or a \
         `,` between two strings; this warning becomes an error in grund 0.22.0"
    );
    ("config-not-toml", Some(key), message)
}

/// A scalar's or a key's warning at `key`, naming `text`.
fn scalar(key: usize, text: &str) -> (&'static str, Option<usize>, String) {
    let message = format!(
        "grund.toml:{key}: expected string, read for now as the one string `{text}`; end the \
         string on line {key} at its closing quote: write `\\\"` for a quote inside it; this \
         warning becomes an error in grund 0.22.0"
    );
    ("config-not-toml", Some(key), message)
}

/// The SPEC kind's `fetch` and `title`, as the load reads them.
fn spec_kind(root: &Path) -> (Option<String>, Option<String>) {
    let config = load_config(root).expect("the config loads");
    let spec = config
        .kinds
        .iter()
        .find(|k| k.kind == "SPEC")
        .expect("SPEC");
    (spec.fetch.clone(), spec.title.clone())
}

/// §FS-config-v2.reader.6.window: a scalar that closes early is read as before, every
/// quote kept, and earns one warning whose remedy is the `\"` half alone.
#[test]
fn a_scalar_that_closes_early_is_read_as_before_and_warns_once() {
    let root = v2("scalar", "fetch = \"one\" \"two\"\n");
    assert_eq!(spec_kind(&root).0.as_deref(), Some("one\" \"two"));
    assert_eq!(window_findings(&root), vec![scalar(8, "one\" \"two")]);
}

/// §FS-config-v2.reader.6.window: an element that closes early is read as the one
/// element it is today, and earns one warning with both halves of the remedy.
#[test]
fn an_element_that_closes_early_is_read_as_before_and_warns_once() {
    let root = v2(
        "element",
        "\n[schema.sources]\nexclude = [\"docs\" \"specs\"]\n",
    );
    let config = load_config(&root).expect("the config loads");
    let model = config
        .project()
        .schema
        .sources
        .model
        .as_ref()
        .expect("a v2 model");
    assert_eq!(model.exclude, vec!["docs\" \"specs".to_string()]);
    assert_eq!(
        window_findings(&root),
        vec![element(10, "docs\" \"specs", 10)]
    );
}

/// §FS-config-v2.reader.6.window: a value earns one warning however many of its
/// strings close early, naming the first.
#[test]
fn a_value_with_two_early_closes_earns_one_warning_naming_the_first() {
    let root = v2(
        "two",
        "\n[schema.sources]\nexclude = [\"a\" \"b\", \"c\" \"d\"]\n",
    );
    assert_eq!(window_findings(&root), vec![element(10, "a\" \"b", 10)]);
}

/// §FS-config-v2.reader.6.window: in an array written across lines the warning stays
/// at the key's line and names the line the string closes on.
#[test]
fn an_element_on_a_later_line_names_the_line_it_closes_on() {
    let root = v2(
        "later",
        "\n[schema.sources]\nexclude = [\n  \"a\",\n  \"b\" \"c\",\n]\n",
    );
    assert_eq!(window_findings(&root), vec![element(10, "b\" \"c", 12)]);
}

/// §FS-config-v2.reader.6: a quoted key is a string of its own, read as before, and
/// earns its own warning with the scalar's remedy.
#[test]
fn a_quoted_key_that_closes_early_is_read_as_before_and_warns() {
    let root = v2(
        "key",
        "\n[schema.sources.extensions]\n\"md\" \"x\" = \"markdown\"\n",
    );
    let config = load_config(&root).expect("the config loads");
    let model = config
        .project()
        .schema
        .sources
        .model
        .as_ref()
        .expect("a v2 model");
    let extensions = &model.extensions;
    assert!(
        extensions.iter().any(|(suffix, _)| suffix == "md\" \"x"),
        "{extensions:?}"
    );
    assert_eq!(window_findings(&root), vec![scalar(10, "md\" \"x")]);
}

/// §FS-config-v2.reader.6: the text in backticks is the string as written, so an
/// escape before the early close is shown as it is written, and `\\` closes nothing.
#[test]
fn the_warning_shows_escapes_as_written() {
    let escaped_quote = v2("escaped-quote", "fetch = \"a\\\"b\" \"c\"\n");
    assert_eq!(
        window_findings(&escaped_quote),
        vec![scalar(8, "a\\\"b\" \"c")]
    );
    let escaped_backslash = v2("escaped-backslash", "fetch = \"a\\\\\" \"b\"\n");
    assert_eq!(
        window_findings(&escaped_backslash),
        vec![scalar(8, "a\\\\\" \"b")]
    );
}

/// §FS-config-v2.reader.6.multi-line: a `"""` string on one line is read as before,
/// its quotes kept, and earns `config-multiline-string` in place of `config-not-toml`.
#[test]
fn a_triple_quoted_string_on_one_line_warns_that_version_2_does_not_read_it() {
    let root = v2(
        "triple",
        "\n[presentation.kinds.SPEC]\ntitle = \"\"\"one\"\"\"\n",
    );
    assert_eq!(spec_kind(&root).1.as_deref(), Some("\"\"one\"\""));
    let message = "grund.toml:10: expected string (a multi-line string, which version 2 does not \
                   read), read for now as `\"\"one\"\"`, its quotes kept; write the string on \
                   line 10 as one basic string; this warning becomes an error in grund 0.22.0";
    assert_eq!(
        window_findings(&root),
        vec![("config-multiline-string", Some(10), message.to_string())]
    );
}

/// §FS-config-v2.reader.6.window: an array with an open line and an early close earns
/// both warnings, the open line's first, and the early close names the joined text.
#[test]
fn an_open_line_and_an_early_close_earn_both_warnings() {
    let root = v2(
        "both",
        "\n[schema.sources]\nexclude = [\n  \"a\nb\" \"c\",\n]\n",
    );
    let open = "grund.toml:10: unclosed array (a string is still open at the end of line 11), \
                read for now with the line break as a space; close the string on line 11; this \
                warning becomes an error in grund 0.22.0";
    assert_eq!(
        window_findings(&root),
        vec![
            ("config-not-toml", Some(10), open.to_string()),
            element(10, "a b\" \"c", 12)
        ]
    );
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

/// §FS-config-v2.reader.6.multi-line: the `"""` string is the one its third quote opens, so
/// a break inside a plain string after it leaves the element's one-line `"""` warning.
#[test]
fn a_break_after_a_one_line_triple_quoted_string_keeps_its_warning() {
    let root = v2(
        "triple-then-break",
        "\n[schema.sources]\nexclude = [\n  \"\"\"a\"\"\" \"b\nc\",\n]\n",
    );
    let open = "grund.toml:10: unclosed array (a string is still open at the end of line 11), \
                read for now with the line break as a space; close the string on line 11; this \
                warning becomes an error in grund 0.22.0";
    let message = "grund.toml:10: expected string (a multi-line string, which version 2 does not \
                   read), read for now as `\"\"a\"\"\" \"b c`, its quotes kept; write the string \
                   on line 11 as one basic string; this warning becomes an error in grund 0.22.0";
    assert_eq!(
        window_findings(&root),
        vec![
            ("config-not-toml", Some(10), open.to_string()),
            ("config-multiline-string", Some(10), message.to_string())
        ]
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
    assert_eq!(spec_kind(&root).0.as_deref(), Some("a\\"));
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
    let Err(err) = load_config(&root) else {
        panic!("the config is refused");
    };
    let err = format!("{err:#}");
    assert!(err.ends_with("grund.toml:10: expected string"), "{err}");
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
