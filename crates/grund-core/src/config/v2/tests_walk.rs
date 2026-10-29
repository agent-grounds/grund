//! Test module: a string in an array closes on the line it opens on
//! (§FS-config-v2.reader.5.strings). An array the walk refuses as `unclosed array` names
//! the first line that ended inside a string, whichever line of the array that is, and
//! the four readers of a string agree on what a quote is (§FS-config.3.11); `config show`
//! writes a line break inside a value as its escape (§FS-config.4.2). Through grund 0.21.x
//! the run of commas before an array's `]` is taken off as before
//! (§FS-config-v2.reader.5.commas.window).

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

/// Guard, green today and kept through grund 0.21.x (§FS-config-v2.reader.5.commas.window):
/// the whole run of adjacent commas before the `]` is taken off, as the readers before took
/// it off, so every shape that loaded before reads the same value, and a run with whitespace
/// inside it leaves the comma the element reader refuses (§FS-config-v2.reader.5.commas).
#[test]
fn the_run_of_commas_before_the_bracket_is_taken_off_as_before() {
    for (text, value) in [
        ("[\"ab\",,]", "[\"ab\"]"),
        ("[\"ab\" ,, ]", "[\"ab\" ]"),
        ("[\"ab\",,,]", "[\"ab\"]"),
        ("[\n  \"ab\",,\n]", "[ \"ab\"]"),
        ("[,]", "[]"),
        ("[ , ]", "[ ]"),
        ("[,,]", "[]"),
        ("[\n  ,\n]", "[ ]"),
        ("[\"ab\",]", "[\"ab\"]"),
        ("[\"ab\" , ]", "[\"ab\" ]"),
        ("[\n  \"ab\"\n  ,\n]", "[ \"ab\" ]"),
        ("[]", "[]"),
        ("[\"ab\", ,]", "[\"ab\", ]"),
        ("[ , , ]", "[ , ]"),
        ("[\n  \"ab\",\n  ,\n]", "[ \"ab\", ]"),
        ("[,\"a\"]", "[,\"a\"]"),
        ("[\"a\",,\"b\"]", "[\"a\",,\"b\"]"),
    ] {
        assert_eq!(joined(text).as_deref(), Some(value), "{text:?}");
    }
}
