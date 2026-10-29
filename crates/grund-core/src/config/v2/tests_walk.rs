//! Test module: a string in an array closes on the line it opens on
//! (§FS-config-v2.reader.5.strings). An array the walk refuses as `unclosed array` names
//! the first line that ended inside a string, whichever line of the array that is, and
//! the four readers of a string agree on what a quote is (§FS-config.3.11); `config show`
//! writes a line break inside a value as its escape (§FS-config.4.2). An array takes one
//! trailing comma, after an element, and from grund 0.22.0 any other comma before its `]` is
//! the element reader's to refuse (§FS-config-v2.reader.5.commas.refusal).

use std::path::Path;

use super::render;
use super::walk::{Reader, joined_value, walk};

/// The refusal the walk gives a `[schema.sources]` whose `exclude` is `value`, the key on
/// line 2, or `None` when the walk reads it.
fn refusal(value: &str) -> Option<String> {
    let text = format!("[schema.sources]\nexclude = {value}");
    let mut reader = Reader::new(Path::new("grund.toml"));
    walk(&text, &mut reader).err().map(|err| format!("{err:#}"))
}

/// What an array still open at the end of the file is refused as, its open line `<n>`.
fn unclosed(open: usize) -> Option<String> {
    Some(format!(
        "grund.toml:2: unclosed array (a string is still open at the end of line {open})"
    ))
}

/// §FS-config-v2.reader.5: the key's own line is the open line.
#[test]
fn an_array_open_in_a_string_at_its_key_line_names_that_line() {
    assert_eq!(refusal("[\"a,b]\n"), unclosed(2));
}

/// §FS-config-v2.reader.5: a continuation line is the open line, and only the first such
/// line is named, however many after it also end inside the string.
#[test]
fn an_array_open_in_a_string_on_a_later_line_names_the_first_such_line() {
    assert_eq!(refusal("[\n  \"a\",\n  \"b\n"), unclosed(4));
    assert_eq!(refusal("[\n  \"a\n  b\n  c\n"), unclosed(3));
}

/// §FS-config-v2.reader.5.strings: a `\"` at a line's end escapes the quote, so the
/// string is still open there.
#[test]
fn an_escaped_quote_at_a_line_end_leaves_the_string_open() {
    assert_eq!(refusal("[\n  \"a\\\"\n"), unclosed(3));
}

/// §FS-config-v2.reader.5.strings.multi-line: a string opened with `"""` is named as one
/// version 2 does not read, not as one left unclosed.
#[test]
fn a_multi_line_string_names_what_version_2_does_not_read() {
    assert_eq!(
        refusal("[\n  \"\"\"a\n"),
        Some(
            "grund.toml:2: unclosed array (a multi-line string opens on line 3, which \
             version 2 does not read)"
                .to_string()
        )
    );
}

/// Guard, green today and kept: an array whose strings all close on their own lines and
/// whose `[` never closes is refused as before, with no parenthetical.
#[test]
fn an_array_whose_strings_close_on_their_lines_is_unclosed_with_no_parenthetical() {
    assert_eq!(
        refusal("[\n  \"a\",\n  \"b,c\",\n"),
        Some("grund.toml:2: unclosed array".to_string())
    );
}

/// §FS-config.3.11: `open_brackets` reads a quote by the rule the other readers do, so a
/// `\"` outside a string opens none, and the `]` after it closes the array. The element
/// is then refused for what it is, not as an array that swallowed the rest of the file.
#[test]
fn an_escaped_quote_outside_a_string_opens_no_string() {
    let refusal = refusal("[a\\\"]\n\n[presentation]\ntrigger = \"$$\"\n");
    assert_eq!(refusal, Some("grund.toml:2: expected string".to_string()));
}

/// §FS-config.4.2: the version-2 writer escapes a line feed, a tab and a carriage return,
/// which the reader reads back, and writes `\` and `"` as before.
#[test]
fn the_v2_writer_escapes_a_line_break_inside_a_value() {
    assert_eq!(render::string("a\nb\tc\rd"), "\"a\\nb\\tc\\rd\"");
    assert_eq!(render::string("a\\\"b"), "\"a\\\\\\\"b\"");
}

/// The value `joined_value` reads from `text`, the value of a key on line 1 followed by the
/// lines after it.
fn joined(text: &str) -> Option<String> {
    let mut lines = text.lines().enumerate();
    let (_, first) = lines.next().unwrap();
    joined_value(first, 1, &mut lines).value
}

/// §FS-config-v2.reader.5.commas: `joined_value` takes off at most one comma before the `]`,
/// and only one with an element before it; every other comma stays for the element reader.
#[test]
#[ignore = "red until #1098 makes array_value take off at most one comma; implement removes this"]
fn only_one_trailing_comma_after_an_element_comes_off_before_the_bracket() {
    for (text, off) in [
        ("[\"ab\",,]", 0),
        ("[\"ab\" ,, ]", 0),
        ("[\"ab\",,,]", 0),
        ("[\n  \"ab\",,\n]", 0),
        ("[,]", 0),
        ("[ , ]", 0),
        ("[,,]", 0),
        ("[\n  ,\n]", 0),
        ("[\"ab\", ,]", 0),
        ("[ , , ]", 0),
        ("[\n  \"ab\",\n  ,\n]", 0),
        ("[,\"a\"]", 0),
        ("[\"a\",,\"b\"]", 0),
        ("[\"ab\",]", 1),
        ("[\"ab\" , ]", 1),
        ("[\n  \"ab\"\n  ,\n]", 1),
        ("[]", 0),
    ] {
        let value = joined(text).unwrap_or_else(|| panic!("{text:?} closes"));
        let kept = value.matches(',').count();
        assert_eq!(
            kept + off,
            text.matches(',').count(),
            "{text:?} read as {value:?}"
        );
    }
}

/// §FS-config-v2.reader.5.commas.refusal: the walk refuses each array whose text loaded through
/// the window as `expected string` with 0.22.0's suffix, at its key's line, and each shape it
/// refused before with the bare message.
#[test]
#[ignore = "red until #1098 refuses a comma with no element before it; implement removes this"]
fn a_comma_with_no_element_before_it_is_refused_naming_0_22_where_it_loaded_before() {
    let became = "grund.toml:2: expected string; this became an error in grund 0.22.0";
    for text in [
        "[\"ab\",,]",
        "[\"ab\" ,, ]",
        "[\"ab\",,,]",
        "[\n  \"ab\",,\n]",
        "[,]",
        "[ , ]",
        "[,,]",
        "[\n  ,\n]",
        "[ # c\n  ,\n]",
    ] {
        assert_eq!(refusal(text).as_deref(), Some(became), "{text:?}");
    }
    for text in [
        "[,\"a\"]",
        "[\"a\",,\"b\"]",
        "[\"ab\", ,]",
        "[ , , ]",
        "[\n  \"ab\",\n  ,\n]",
    ] {
        let bare = "grund.toml:2: expected string";
        assert_eq!(refusal(text).as_deref(), Some(bare), "{text:?}");
    }
}

/// Guard, green today and kept (§FS-config-v2.reader.5.commas): the one trailing comma after
/// an element, and an array with no comma before its `]`, load.
#[test]
fn one_trailing_comma_after_an_element_loads() {
    for text in [
        "[]",
        "[ ]",
        "[\n]",
        "[\"ab\",]",
        "[\"ab\" , ]",
        "[\n  \"ab\"\n  ,\n]",
    ] {
        assert_eq!(refusal(text), None, "{text:?}");
    }
}
