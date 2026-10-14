//! Test module: the multi-line string pre-pass (§FS-languages.multi-line-string)
//! — each recognized form's opener and closer, and the two bounds that keep a
//! misread opener from hiding a citation.

use std::path::Path;

use super::multi_line_string::{MultiLineStrings, WHOLE_LINE};
use super::never_rewrite::{is_inside_string_literal, qualified_suppressed_in_source};

const W: usize = WHOLE_LINE;

/// The entry state of every line of `text`, read as a file named `name`.
fn entries(name: &str, text: &str) -> Vec<usize> {
    let literals = MultiLineStrings::of(
        Path::new(name).extension().and_then(|e| e.to_str()),
        text,
        true,
    );
    (0..text.lines().count())
        .map(|index| literals.entry(index))
        .collect()
}

/// §FS-languages.multi-line-string.forms: the Rust rows.
#[test]
fn rust_raw_and_ordinary_strings_carry_to_their_closers() {
    assert_eq!(
        entries("a.rs", "let s = r#\"\nbody\n\"#; // tail\n"),
        [0, W, 2]
    );
    assert_eq!(entries("a.rs", "let s = br\"\nbody\n\";\n"), [0, W, 1]);
    // An escaped quote does not close an ordinary string.
    assert_eq!(entries("a.rs", "let s = \"a \\\" b\nc\" + d\n"), [0, 2]);
}

/// §FS-languages.multi-line-string.forms: `"#` does not close `r##"`.
#[test]
fn a_raw_string_closes_only_on_its_own_hash_count() {
    assert_eq!(entries("a.rs", "r##\"\na \"# no\n\"##\n"), [0, W, 3]);
}

/// §FS-languages.multi-line-string.forms: `bar"` opens an ordinary string, and
/// in Rust neither an apostrophe nor a character literal `'"'` opens anything.
#[test]
fn rust_prefixes_are_whole_words_and_apostrophes_open_nothing() {
    assert_eq!(entries("a.rs", "x = bar\"\ny\"\n"), [0, 2]);
    assert_eq!(
        entries("a.rs", "// don't\nlet q = '\"';\nfn f<'a>() {}\n"),
        [0, 0, 0]
    );
}

/// §FS-languages.multi-line-string.forms: the triple-quoted row, and Dart's `'''`.
#[test]
fn triple_quoted_strings_carry_in_their_languages() {
    let java = "String s = \"\"\"\n  x\n  \"\"\"; // t\n";
    assert_eq!(entries("A.java", java), [0, W, 5]);
    assert_eq!(entries("a.dart", "var s = '''\nx\n''';\n"), [0, W, 3]);
    // Java has no `'''` form.
    assert_eq!(entries("A.java", "x = '''\ny\n''';\n"), [0, 0, 0]);
}

/// §FS-languages.multi-line-string.forms: C#'s raw run closes only on a run of the
/// same length, and a verbatim string reads `""` as a quote inside it.
#[test]
fn csharp_raw_runs_and_verbatim_doubled_quotes() {
    let raw = "var s = \"\"\"\"\na \"\"\" b\n\"\"\"\";\n";
    assert_eq!(entries("a.cs", raw), [0, W, 4]);
    assert_eq!(entries("a.cs", "var s = @\"\na \"\" b\n\";\n"), [0, W, 1]);
    assert_eq!(entries("a.cs", "var s = $@\"\nb\n\";\n"), [0, W, 1]);
}

/// §FS-languages.multi-line-string.forms: a C++ raw string closes on `)delim"`
/// alone, and a delimiter past 16 characters opens nothing.
#[test]
fn cpp_raw_strings_close_on_their_delimiter() {
    assert_eq!(
        entries("a.cpp", "auto s = R\"x(\na )\" b\n)x\";\n"),
        [0, W, 3]
    );
    assert_eq!(entries("a.cpp", "auto s = u8R\"(\nb\n)\";\n"), [0, W, 2]);
    let long = "auto s = R\"abcdefghijklmnopq(\nb\n)abcdefghijklmnopq\";\n";
    assert_eq!(entries("a.cpp", long), [0, 0, 0]);
}

/// §FS-languages.multi-line-string.forms: Go's raw string and the JS template.
#[test]
fn backtick_strings_carry() {
    assert_eq!(entries("a.go", "s := `\nb\n`\n"), [0, W, 1]);
    assert_eq!(entries("a.ts", "const s = `\nb \\` still\n`;\n"), [0, W, 1]);
}

/// §FS-languages.multi-line-string.forms: a backtick inside `${ … }` opens a
/// nested template, and a template whose nesting never balances opens nothing.
#[test]
fn template_literals_track_their_interpolations() {
    let nested = "const t = `a ${\n`b\nc`\n} d\n`;\n";
    assert_eq!(entries("a.js", nested), [0, W, W, W, 1]);
    let braces = "const t = `a ${ {x: 1} }\nb`;\n";
    assert_eq!(entries("a.js", braces), [0, 2]);
    assert_eq!(entries("a.js", "const t = `a ${ {\n`\n"), [0, 0]);
}

/// §FS-languages.multi-line-string.forms: a mid-line triple quote in Python, while a
/// line-initial one keeps the docstring rule and an assigned one the data rule.
#[test]
fn python_carries_only_the_mid_line_triple_quote() {
    assert_eq!(entries("a.py", "x(\"\"\"\nbody\n\"\"\")\n"), [0, W, 3]);
    assert_eq!(entries("a.py", "x(rb'''\nbody\n''')\n"), [0, W, 3]);
    assert_eq!(entries("a.py", "\"\"\"\nbody\n\"\"\"\n"), [0, 0, 0]);
    assert_eq!(entries("a.py", "X = \"\"\"\nbody\n\"\"\"\n"), [0, 0, 0]);
    // A one-line docstring's closer is not a mid-line opener.
    let docstring = "def f():\n    \"\"\"Doc.\"\"\"\n    g(1)\n\"\"\"\n";
    assert_eq!(entries("a.py", docstring), [0, 0, 0, 0]);
}

/// §FS-languages.multi-line-string.forms: a shell heredoc's body starts on the next
/// line, and `<<-` drops the closer's leading tabs.
#[test]
fn shell_heredocs_carry_from_the_next_line() {
    assert_eq!(
        entries("a.sh", "cat <<EOF | sort\nbody\nEOF\necho\n"),
        [0, W, 3, 0]
    );
    assert_eq!(entries("a.sh", "cat <<-EOF\n\tbody\n\tEOF\n"), [0, W, 4]);
    assert_eq!(entries("a.sh", "cat << 'EOF'\nbody\nEOF\n"), [0, W, 3]);
    // Without `-`, an indented word is not the closer.
    assert_eq!(entries("a.sh", "cat <<EOF\n\tEOF\nEOF\n"), [0, W, 3]);
}

/// §FS-languages.multi-line-string.forms: `<<<` is a here-string in shell, not a
/// heredoc.
#[test]
fn a_here_string_is_not_a_heredoc() {
    assert_eq!(entries("a.sh", "cat <<<word\nbody\nword\n"), [0, 0, 0]);
}

/// §FS-languages.multi-line-string.forms: PHP's heredoc and nowdoc close on the
/// identifier as a line's first text, with code after it.
#[test]
fn php_heredocs_close_on_their_identifier() {
    assert_eq!(entries("a.php", "$x = <<<'EOT'\nbody\n  EOT;\n"), [0, W, 5]);
    assert_eq!(entries("a.php", "$x = <<<EOT\nEOTX\nEOT;\n"), [0, W, 3]);
}

/// §FS-languages.multi-line-string.bound: an opener in a line comment, after one,
/// or on a line inside a `/* … */` block opens nothing.
#[test]
fn an_opener_in_a_comment_opens_nothing() {
    assert_eq!(entries("a.sh", "# cat <<EOF\nx\nEOF\n"), [0, 0, 0]);
    assert_eq!(entries("a.rs", "// r#\"\nx\n\"#\n"), [0, 0, 0]);
    assert_eq!(entries("a.rs", "/*\nr#\"\n*/\nx\n\"#\n"), [0, 0, 0, 0, 0]);
    // Rust's `r#"` is not a `#` comment.
    assert_eq!(entries("a.rs", "f(r#\"\nx\n\"#)\n"), [0, W, 2]);
}

/// §FS-languages.multi-line-string.bound: an opener with no closer opens nothing,
/// and neither does the quote inside it.
#[test]
fn an_opener_with_no_closer_opens_nothing() {
    assert_eq!(entries("a.rs", "let s = r#\"\nx\n"), [0, 0]);
    assert_eq!(entries("a.sh", "cat <<EOF\nx\n"), [0, 0]);
    assert_eq!(entries("a.rs", "let s = r#\"x\ny \"\n"), [0, 0]);
}

/// §FS-languages.multi-line-string: a form that closes on its own line carries
/// nothing, and the search resumes after it.
#[test]
fn a_one_line_form_carries_nothing_and_the_search_resumes() {
    assert_eq!(entries("a.rs", "f(r#\"a\"#, \"\nb\");\n"), [0, 2]);
}

/// §FS-languages.multi-line-string.unrecognized: the forms left out carry nothing —
/// a Ruby heredoc, a Lua long string, a shell ordinary string, a C backslash
/// continuation, and a second heredoc on one line.
#[test]
fn the_unrecognized_forms_carry_nothing() {
    assert_eq!(entries("a.rb", "s = <<~EOF\nbody\nEOF\n"), [0, 0, 0]);
    assert_eq!(entries("a.lua", "s = [[\nbody\n]]\n"), [0, 0, 0]);
    assert_eq!(entries("a.sh", "s=\"first\nbody\n\"\n"), [0, 0, 0]);
    assert_eq!(entries("a.c", "s = \"a\\\nb\";\n"), [0, 0]);
    let two = "cat <<A <<B\na\nA\nb\nB\n";
    assert_eq!(entries("a.sh", two), [0, W, 1, 0, 0]);
}

/// §AR-scanner.2.3.1: the shared predicate reads a line in the state it begins in —
/// inside the literal up to the closer, the per-line toggle neutral after it.
#[test]
fn the_shared_predicate_starts_in_the_entry_state() {
    let line = "\"#); // §api/FS-x";
    assert!(is_inside_string_literal(line, 2, 0));
    assert!(!is_inside_string_literal(line, 2, 7));
    // On one line the closing quote would open a string.
    assert!(is_inside_string_literal(line, 0, 7));
    assert!(is_inside_string_literal("anything", WHOLE_LINE, 3));
    // A template's closing backtick opens no inline-code span either.
    assert!(!qualified_suppressed_in_source(
        "`; // §api/FS-x",
        1,
        false,
        6
    ));
}

/// §FS-languages.multi-line-string.bound: Rust's `'` is read as Rust reads it — a
/// character literal is one unit, a lifetime is no quote — so neither `'"'` nor an
/// odd count of lifetimes turns a later `"` into an opener or hides one.
#[test]
fn rust_char_literals_and_lifetimes_are_read_as_rust_reads_them() {
    let after_static = "const M: &'static str = \"it's\";\n// x\nfn f() { g(\"y\"); }\n";
    assert_eq!(entries("a.rs", after_static), [0, 0, 0]);
    let char_quote = "fn q(s: &'static str) -> bool { s.contains('\"') }\n// x\nh(\"y\");\n";
    assert_eq!(entries("a.rs", char_quote), [0, 0, 0]);
    // A string opened after a lifetime is carried.
    assert_eq!(
        entries("a.rs", "fn f<'a>() -> &'a str { \"a\nb\" }\n"),
        [0, 2]
    );
    assert_eq!(
        entries("a.rs", "let c = '\\''; let s = b\"x\ny\";\n"),
        [0, 2]
    );
}

/// §FS-languages.multi-line-string.bound: a `/* … */` opens nothing up to its
/// `*/`, wherever on its line it starts, and the search resumes after the `*/` —
/// on the same line or a later one — so a form opened past it is still carried.
#[test]
fn a_block_comment_opens_nothing_to_its_close_and_the_search_resumes() {
    let js = "foo(); /* runs on\n and `bar\n*/ x();\nconst t = `y`;\n";
    assert_eq!(entries("a.js", js), [0, 0, 0, 0]);
    let rust = "f(); /* \"a\n*/ let s = \"b\nc\";\n";
    assert_eq!(entries("a.rs", rust), [0, 0, 2]);
    // Rust's blocks nest, so the search resumes at the `*/` that balances.
    let nested = "f(); /* a\n /* b */ \"c\n*/ let s = \"d\ne\";\n";
    assert_eq!(entries("a.rs", nested), [0, 0, 0, 2]);
    let line_initial_nested = "/* a\n /* b */\n \"c\n*/\nlet s = \"d\";\n";
    assert_eq!(entries("a.rs", line_initial_nested), [0, 0, 0, 0, 0]);
    // So do Kotlin's, Scala's, Swift's and Dart's; Java's do not.
    let triple = "/* a /* b */ val s = \"\"\"x\n c\n*/\nval t = \"\"\"y\"\"\"\n";
    for name in ["a.kt", "a.kts", "a.scala", "a.swift", "a.dart"] {
        assert_eq!(entries(name, triple), [0, 0, 0, 0], "{name}");
    }
    assert_eq!(entries("a.java", triple), [0, W, W, 11]);
    // A block that closes on its own line, at its start or after code.
    assert_eq!(entries("a.js", "/* n */ const t = `a\nb`;\n"), [0, 2]);
    let sequence = "f(); /* a */ g(); /* b\n `c\n*/\nconst t = `y`;\n";
    assert_eq!(entries("a.js", sequence), [0, 0, 0, 0]);
    // Code after the `*/` of a block that opened at a line's start.
    assert_eq!(
        entries("a.js", "/* a\n `b\n*/ const t = `c\nd`;\n"),
        [0, 0, 0, 2]
    );
}

/// §FS-languages.multi-line-string.bound: a `/*` with no `*/` later ends the
/// search, and every later line reads as it would without the rule.
#[test]
fn an_unclosed_block_comment_ends_the_search() {
    assert_eq!(entries("a.js", "f(); /* a\nconst t = `b\nc`;\n"), [0, 0, 0]);
    assert_eq!(entries("a.js", "/* a\nconst t = `b\nc`;\n"), [0, 0, 0]);
}

/// §FS-languages.multi-line-string.bound: a backtick inside a regular-expression
/// literal opens no template; a division does not start one, and neither does a
/// `/` with no closing `/` on its line.
#[test]
fn a_regex_literal_hides_its_backticks_and_a_division_does_not() {
    let regex = "s.replace(/`/g, \"\");\n// x\nconst t = `y`;\n";
    assert_eq!(entries("a.js", regex), [0, 0, 0]);
    assert_eq!(entries("a.ts", "return /[/`]/.test(s);\nf(`a`);\n"), [0, 0]);
    // `a / b` divides, so the template after it is carried.
    assert_eq!(entries("a.js", "x = a / b + `c\nd`;\n"), [0, 2]);
    // A division at a line's start, and the `</` of a JSX closing tag.
    assert_eq!(entries("a.js", "x = a\n  / b + `c\nd`;\n"), [0, 0, 2]);
    assert_eq!(entries("a.jsx", "x = <p>a</p>; t = `b\nc`;\n"), [0, 2]);
}

/// §FS-languages.multi-line-string.bound: a regular expression that holds a quote
/// or a comment opener is a guess, so the rest of its line is read both past it and
/// through it. Where the two open the same thing it is carried; where they differ,
/// the search ends and no later line is read as inside a form.
#[test]
fn a_guessed_skip_is_read_both_ways_and_ends_the_search_where_they_differ() {
    assert_eq!(entries("a.js", "x = /\"a\"/; t = `b\nc`;\n"), [0, 2]);
    let differ = "r = /a\"/; t = `b\nc`;\n// x\nu = `d\ne`;\n";
    assert_eq!(entries("a.js", differ), [0, 0, 0, 0, 0]);
    let jsx = "x = <b>a</b>{\"/d\"}{\"`\"};\n// x\nconst t = `y`;\n";
    assert_eq!(entries("a.jsx", jsx), [0, 0, 0]);
    let postfix = "x = i++ / 2; // the ` key\n// x\nconst t = `y`;\n";
    assert_eq!(entries("a.js", postfix), [0, 0, 0]);
    let block = "x = i++ / 2; /* on\n `a\n*/\nconst t = `y`;\n";
    assert_eq!(entries("a.js", block), [0, 0, 0, 0]);
    let after_quote = "x = i++ / 2 + \"s\"; /* on\n `a\n*/\nconst t = `y`;\n";
    assert_eq!(entries("a.js", after_quote), [0, 0, 0, 0]);
    // A quote-free regex is still skipped, and an escaped quote in it is no quote.
    assert_eq!(entries("a.js", "f(/\\`/, `a\nb`);\n"), [0, 2]);
}

/// §FS-languages.multi-line-string.bound: a Rust character literal is no guess,
/// so `'"'` is skipped whole and a string opened after it is carried.
#[test]
fn a_rust_char_literal_holding_a_quote_is_skipped_whole() {
    let char_quote = "f() { let c = '\"'; let s = \"a\nb\"; }\n// x\ng(\"y\");\n";
    assert_eq!(entries("a.rs", char_quote), [0, 2, 0, 0]);
}

/// §FS-languages.multi-line-string.bound: `<<` inside `(( … ))` is a shift.
#[test]
fn a_shell_shift_inside_arithmetic_is_no_heredoc() {
    assert_eq!(entries("a.sh", "x=$(( 1 << n ))\necho a\nn\n"), [0, 0, 0]);
    assert_eq!(
        entries("a.sh", "(( 1 )); cat <<EOF\nbody\nEOF\n"),
        [0, W, 3]
    );
}
