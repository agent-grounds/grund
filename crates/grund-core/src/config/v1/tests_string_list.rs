//! §FS-config.3.11: a string list is cut into its elements only at a comma outside a string,
//! a string closing at the first `"` that is not part of a `\"` escape; every element that is
//! not one basic string keeps the refusal it had (§FS-config.4.3).

use std::path::Path;

use super::{parse_string_list, string_close};

fn read(value: &str) -> Vec<String> {
    parse_string_list(Path::new("grund.toml"), 7, value)
        .unwrap_or_else(|error| panic!("`{value}` is refused: {error}"))
}

fn refusal(value: &str) -> String {
    match parse_string_list(Path::new("grund.toml"), 7, value) {
        Ok(read) => panic!("`{value}` reads as {read:?}"),
        Err(error) => error.to_string(),
    }
}

#[test]
fn a_comma_inside_a_string_belongs_to_its_element() {
    assert_eq!(read(r#"["a,b", "c"]"#), ["a,b", "c"]);
    assert_eq!(read(r#"[",", ",,"]"#), [",", ",,"]);
    assert_eq!(
        read(r#"["specs/{001-a,002-b}/spec.md"]"#),
        ["specs/{001-a,002-b}/spec.md"]
    );
}

#[test]
fn an_escaped_quote_does_not_close_the_string_and_an_escaped_backslash_does_not_escape() {
    assert_eq!(read(r#"["a\",b", "c"]"#), [r#"a",b"#, "c"]);
    assert_eq!(read(r#"["a\\", "b,c"]"#), [r"a\", "b,c"]);
}

/// Green today and kept: lists without a comma in a string read as they always have.
#[test]
fn a_list_without_a_comma_in_a_string_reads_as_before() {
    assert_eq!(read("[]"), Vec::<String>::new());
    assert_eq!(read(r#"["a", "b"]"#), ["a", "b"]);
    assert_eq!(read(r#"[ "a" ,"b" ]"#), ["a", "b"]);
}

/// Green before the rule and kept: the rule refuses nothing new. A piece that is not one basic
/// string, a quote dropped or a comma missing between two strings, is still cut at each comma.
#[test]
fn a_malformed_element_keeps_its_refusal() {
    for value in [r#"["a",]"#, r#"["a,b]"#, r#"[a, "b"]"#, r#"["a", ,"b"]"#] {
        assert_eq!(refusal(value), "grund.toml:7: expected string", "{value}");
    }
    for value in [
        r#"["a,"b"]"#,
        r#"["a", "b,"c"]"#,
        r#"["a, "b"]"#,
        r#"["target" "a,b"]"#,
    ] {
        assert_eq!(refusal(value), "grund.toml:7: expected string", "{value}");
    }
    assert_eq!(refusal(r#""a, b""#), "grund.toml:7: expected string list");
}

/// Pins the reading these lists had before the rule, not an endorsement of it: a piece that is
/// not one basic string is cut at each comma and each part read as before. Version 1 keeps this
/// reading for good (§FS-config.5.2), and meets the rule only at `config migrate`
/// (§FS-config-migrate.3.1.8).
#[test]
fn a_list_read_before_the_rule_is_read_the_same() {
    assert_eq!(read(r#"["a"b", "c"]"#), [r#"a"b"#, "c"]);
    assert_eq!(read(r#"["x" \"y", "z"]"#), [r#"x" "y"#, "z"]);
    assert_eq!(read(r#"["a" "b"]"#), [r#"a" "b"#]);
}

/// §FS-config.3.11: the one close every reader shares, read from just past an opening quote,
/// is the first `"` no odd run of backslashes escapes, wherever the value's last quote stands.
#[test]
fn a_string_closes_at_its_first_unescaped_quote() {
    for (written, close) in [
        (r#""a""#, 2),
        (r#""a\"b""#, 5),
        (r#""a\\""#, 4),
        (r#""a" "b""#, 2),
        (r#""a"b""#, 2),
        (r#""""a""""#, 1),
    ] {
        assert_eq!(
            string_close(written.as_bytes(), 1),
            Some(close),
            "{written}"
        );
    }
    assert_eq!(string_close(br#""a\""#, 1), None);
}
