//! Black-box: a version-2 string ends at its first unescaped quote (§FS-config-v2.reader.6).
//! Ports grund.159's reproducer (#1088): through grund 0.21.x a scalar, an element or a quoted
//! key whose string closes early is read as before and warns at its key's line, naming the
//! string as written, the line it closes on and 0.22.0 (§FS-config-v2.reader.6.window,
//! §FS-config-v2.checks.config-not-toml); a `"""` string on one line says version 2 does not
//! read it (§FS-config-v2.reader.6.multi-line). Version 1 keeps its reading for good
//! (§FS-config.5.2) and meets the rule at `config migrate` (§FS-config-migrate.3.1.8).

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const SPEC: &str = "# SPEC-one: One\n\nLead.\n";
const CODE: &str = "// Realizes \u{a7}SPEC-one.\nfn main() {}\n";

/// The version-2 config every case extends, its next line line 8.
const HEAD: &str = "grund_config_version = 2\n\n[schema.kinds.code]\nfolders = [\"src\"]\n\n\
                    [schema.kinds.SPEC]\nfiles = [\"specs/spec.md\"]\n";

/// The `config-not-toml` warning for an element at the key on line `key`, whose string
/// `text` closes early on line `close`.
fn element(key: usize, text: &str, close: usize) -> String {
    format!(
        "warning: grund.toml:{key}: expected string, read for now as the one string `{text}`; \
         end the string on line {close} at its closing quote: write `\\\"` for a quote inside \
         it, or a `,` between two strings; this warning becomes an error in grund 0.22.0\n"
    )
}

/// The `config-not-toml` warning for a scalar or a quoted key on line `key` whose string
/// `text` closes early.
fn scalar(key: usize, text: &str) -> String {
    format!(
        "warning: grund.toml:{key}: expected string, read for now as the one string `{text}`; \
         end the string on line {key} at its closing quote: write `\\\"` for a quote inside \
         it; this warning becomes an error in grund 0.22.0\n"
    )
}

/// The #1089 warning for the array at line 10 whose string is open at the end of line 11.
const OPEN_LINE: &str = "warning: grund.toml:10: unclosed array (a string is still open at the \
                         end of line 11), read for now with the line break as a space; close \
                         the string on line 11; this warning becomes an error in grund 0.22.0\n";

/// The version-1 caution, the only finding a version-1 fixture here earns.
const VERSION_1: &str = "warning: grund.toml is version 1; migrate it with grund config \
                         migrate --to 2\n";

struct Fixture(PathBuf);

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Fixture {
    /// A clean version-2 tree whose `grund.toml` is [`HEAD`] followed by `tail`.
    fn v2(tail: &str) -> Self {
        Self::with(
            &format!("{HEAD}{tail}"),
            &[("specs/spec.md", SPEC), ("src/main.rs", CODE)],
        )
    }

    /// A tree whose `grund.toml` is `config`, holding `files`.
    fn with(config: &str, files: &[(&str, &str)]) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("grund-1088-{}-{serial}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        for (path, contents) in [("grund.toml", config)].iter().chain(files) {
            let path = dir.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, contents).unwrap();
        }
        Self(dir)
    }

    fn run(&self, args: &[&str]) -> Run {
        let out = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .expect("run grund");
        Run {
            code: out.status.code(),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        }
    }

    /// Run `grund args…` and require exit `code`, stdout and stderr exactly.
    fn expect(&self, args: &[&str], code: i32, stdout: &str, stderr: &str) {
        let run = self.run(args);
        assert_eq!(
            (run.code, run.stdout.as_str(), run.stderr.as_str()),
            (Some(code), stdout, stderr),
            "grund {}",
            args.join(" ")
        );
    }

    /// `config show`'s line under `[<table>]` that starts with `key`, requiring exit 0 and
    /// `stderr` exactly.
    fn shown(&self, table: &str, key: &str, stderr: &str) -> String {
        let run = self.run(&["config", "show"]);
        assert_eq!(
            (run.code, run.stderr.as_str()),
            (Some(0), stderr),
            "{}",
            run.stdout
        );
        let header = format!("{table}]\n");
        run.stdout
            .split("\n[")
            .find(|section| section.starts_with(&header))
            .and_then(|section| section.lines().find(|line| line.starts_with(key)))
            .unwrap_or_else(|| panic!("no [{table}] {key} in:\n{}", run.stdout))
            .to_string()
    }

    /// The window's whole contract for a config whose one finding is `warning`, code
    /// `code` at line `line`: `check`, `config validate` and `config show` print it and
    /// exit 0, `config show` prints `shown` under `[table]`, JSON carries the code, the
    /// file and the line, and `--ignore` brings back `success`.
    fn warns(&self, code: &str, line: usize, warning: &str, table: &str, shown: &str) {
        self.expect(&["check"], 0, "", warning);
        self.expect(&["config", "validate"], 0, "", warning);
        let key = shown.split(" = ").next().unwrap();
        assert_eq!(self.shown(table, key, warning), shown);
        let run = self.run(&["check", "--format", "json"]);
        assert_eq!(
            (run.code, run.stdout.as_str()),
            (Some(0), ""),
            "{}",
            run.stderr
        );
        let lines: Vec<&str> = run.stderr.lines().collect();
        assert_eq!(lines.len(), 1, "{}", run.stderr);
        for part in [
            "\"severity\":\"warning\"".to_string(),
            "\"path\":\"grund.toml\"".to_string(),
            format!("\"line\":{line}"),
            format!("\"code\":\"{code}\""),
            "this warning becomes an error in grund 0.22.0".to_string(),
        ] {
            assert!(lines[0].contains(&part), "{part} missing from {}", lines[0]);
        }
        self.expect(&["check", "--ignore", code], 0, "success\n", "");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// grund.159's first case: `fetch = "one" "two"` still loads as the one string `one" "two`,
/// and warns with the scalar's remedy, the `\"` half alone.
#[test]
fn a_scalar_that_closes_early_warns_and_is_read_as_before() {
    Fixture::v2("fetch = \"one\" \"two\"\n").warns(
        "config-not-toml",
        8,
        &scalar(8, "one\" \"two"),
        "schema.kinds.SPEC",
        "fetch = \"one\\\" \\\"two\"",
    );
}

/// grund.159's second case: `folders = ["a" "b"]`, a comma forgotten, still loads as the one
/// folder `a" "b`, and warns with both halves of an element's remedy.
#[test]
fn an_element_that_closes_early_warns_and_is_read_as_before() {
    Fixture::v2("folders = [\"a\" \"b\"]\n").warns(
        "config-not-toml",
        8,
        &element(8, "a\" \"b", 8),
        "schema.kinds.SPEC",
        "folders = [\"a\\\" \\\"b\"]",
    );
}

/// grund.159's third case: a `[presentation.kinds]` title is a scalar like any other.
#[test]
fn a_presentation_title_that_closes_early_warns_and_is_read_as_before() {
    Fixture::v2("\n[presentation.kinds.SPEC]\ntitle = \"one\" \"two\"\n").warns(
        "config-not-toml",
        10,
        &scalar(10, "one\" \"two"),
        "presentation.kinds.SPEC",
        "title = \"one\\\" \\\"two\"",
    );
}

/// §FS-config-v2.reader.6: a quoted key is a string of its own, and earns its own warning
/// with the scalar's remedy.
#[test]
fn a_quoted_key_that_closes_early_warns_and_is_read_as_before() {
    Fixture::v2("\n[schema.sources.extensions]\n\"md\" \"x\" = \"markdown\"\n").warns(
        "config-not-toml",
        10,
        &scalar(10, "md\" \"x"),
        "schema.sources.extensions",
        "\"md\\\" \\\"x\" = \"markdown\"",
    );
}

/// §FS-config-v2.reader.6.multi-line: `title = """one"""` is TOML version 2 does not read;
/// it loads as `""one""`, its quotes kept, and earns `config-multiline-string`.
#[test]
fn a_triple_quoted_string_on_one_line_warns_that_version_2_does_not_read_it() {
    let warning = "warning: grund.toml:10: expected string (a multi-line string, which version 2 \
                   does not read), read for now as `\"\"one\"\"`, its quotes kept; write the \
                   string on line 10 as one basic string; this warning becomes an error in \
                   grund 0.22.0\n";
    Fixture::v2("\n[presentation.kinds.SPEC]\ntitle = \"\"\"one\"\"\"\n").warns(
        "config-multiline-string",
        10,
        warning,
        "presentation.kinds.SPEC",
        "title = \"\\\"\\\"one\\\"\\\"\"",
    );
}

/// §FS-config-v2.reader.6.window: in an array written across lines the warning stays at the
/// key's line, and its remedy names line 12, where the string closes early.
#[test]
fn an_element_on_a_later_line_names_the_line_it_closes_on() {
    Fixture::v2("\n[schema.sources]\nexclude = [\n  \"a\",\n  \"b\" \"c\",\n]\n").warns(
        "config-not-toml",
        10,
        &element(10, "b\" \"c", 12),
        "schema.sources",
        "exclude = [\"a\", \"b\\\" \\\"c\"]",
    );
}

/// §FS-config-v2.reader.6.window: a value earns one warning however many of its strings
/// close early, naming the first.
#[test]
fn a_value_with_two_early_closes_earns_one_warning_naming_the_first() {
    Fixture::v2("\n[schema.sources]\nexclude = [\"a\" \"b\", \"c\" \"d\"]\n").warns(
        "config-not-toml",
        10,
        &element(10, "a\" \"b", 10),
        "schema.sources",
        "exclude = [\"a\\\" \\\"b\", \"c\\\" \\\"d\"]",
    );
}

/// §FS-config-v2.reader.6.window: an array with an open line and an early close earns both
/// warnings, the open line's first, the early close naming the joined text and line 12.
#[test]
#[ignore = "red until #1088 warns for an early close after #1089's open line; implement removes this"]
fn an_open_line_and_an_early_close_earn_both_warnings() {
    let fx = Fixture::v2("\n[schema.sources]\nexclude = [\n  \"a\nb\" \"c\",\n]\n");
    let both = format!("{OPEN_LINE}{}", element(10, "a b\" \"c", 12));
    fx.expect(&["check"], 0, "", &both);
    fx.expect(&["config", "validate"], 0, "", &both);
    assert_eq!(
        fx.shown("schema.sources", "exclude = ", &both),
        "exclude = [\"a b\\\" \\\"c\"]"
    );
    let run = fx.run(&["check", "--format", "json"]);
    let lines: Vec<&str> = run.stderr.lines().collect();
    assert_eq!(lines.len(), 2, "{}", run.stderr);
    assert!(
        lines[0].contains("still open at the end of line 11"),
        "{}",
        lines[0]
    );
    assert!(
        lines[1].contains("read for now as the one string"),
        "{}",
        lines[1]
    );
    for line in lines {
        assert!(line.contains("\"code\":\"config-not-toml\""), "{line}");
        assert!(line.contains("\"line\":10"), "{line}");
    }
    fx.expect(
        &["check", "--ignore", "config-not-toml"],
        0,
        "success\n",
        "",
    );
}

/// Guard, green today and kept: a `"""` string a line break runs through is one construct,
/// and earns #1089's warning alone once its lines are joined (§FS-config-v2.reader.6.multi-line).
#[test]
fn a_triple_quoted_string_over_a_break_earns_only_the_open_line_warning() {
    let fx = Fixture::v2("\n[schema.sources]\nexclude = [\n  \"\"\"a\nb\"\"\",\n]\n");
    let warning = "warning: grund.toml:10: unclosed array (a multi-line string opens on line \
                   11, which version 2 does not read), read for now with the line break as a \
                   space; write the string on line 11 as one basic string; this warning \
                   becomes an error in grund 0.22.0\n";
    fx.expect(&["check"], 0, "", warning);
    assert_eq!(
        fx.shown("schema.sources", "exclude = ", warning),
        "exclude = [\"\\\"\\\"a b\\\"\\\"\"]"
    );
}

/// Guard, green today and kept: a value the readers before refused keeps its bare refusal
/// (§FS-config-v2.reader.6.refusal).
#[test]
fn a_value_refused_before_is_refused_as_before() {
    let fx = Fixture::v2("\n[schema.sources]\nexclude = [\"target\" \"a,b\"]\n");
    let refusal = "error: grund.toml:10: expected string\n";
    fx.expect(&["check"], 2, "", refusal);
    fx.expect(&["config", "show"], 2, "", refusal);
}

/// Guard, green today and kept: strings that close at their end, with an escaped quote, an
/// escaped backslash and a comment after the value, load as written with no warning.
#[test]
fn strings_that_close_at_their_end_load_with_no_warning() {
    let fx = Fixture::v2(
        "fetch = \"a\\\\\" # a comment\n\n[schema.sources]\nexclude = [\"a\\\"b\", \"c\\\\d\"]\n",
    );
    fx.expect(&["check"], 0, "success\n", "");
    assert_eq!(
        fx.shown("schema.sources", "exclude = ", ""),
        "exclude = [\"a\\\"b\", \"c\\\\d\"]"
    );
}

/// A version-1 tree whose `[[kinds]] folder` is `docs" "x`, its key on line 8.
fn v1_folder() -> Fixture {
    Fixture::with(
        "grund_config_version = 1\n\n[id]\nnamed_sections = true\n\n[[kinds]]\nkind = \"SPEC\"\n\
         folder = \"docs\" \"x\"\nindex = false\n\n[scan]\ninclude = [\"docs\", \"src\"]\n",
        &[
            ("docs/spec.md", "# SPEC-001-one: One\n\nLead.\n"),
            (
                "src/main.rs",
                "// Realizes \u{a7}SPEC-001-one.\nfn main() {}\n",
            ),
        ],
    )
}

/// A version-1 tree whose `[scan] include` is `["a"b", "c"]`, its key on line 12.
fn v1_include() -> Fixture {
    Fixture::with(
        "grund_config_version = 1\n\n[id]\nnamed_sections = true\n\n[[kinds]]\nkind = \"SPEC\"\n\
         folder = \"docs\"\nindex = false\n\n[scan]\ninclude = [\"a\"b\", \"c\"]\n",
        &[
            ("docs/spec.md", "# SPEC-001-one: One\n\nLead.\n"),
            ("a\"b/notes.txt", "Notes.\n"),
            (
                "c/main.rs",
                "// Realizes \u{a7}SPEC-001-one.\nfn main() {}\n",
            ),
        ],
    )
}

/// Guard, green today and kept: version 1 keeps its reading for good (§FS-config.5.2), so
/// both fixtures load with only the version-1 caution, their strings read as before.
#[test]
fn version_1_keeps_its_reading_with_only_the_version_1_caution() {
    let folder = v1_folder();
    folder.expect(&["check"], 0, "", VERSION_1);
    assert_eq!(
        folder.shown("[kinds]", "folder = ", VERSION_1),
        "folder = \"docs\\\" \\\"x\""
    );
    let include = v1_include();
    include.expect(&["check"], 0, "", VERSION_1);
    assert_eq!(
        include.shown("scan", "include = ", VERSION_1),
        "include = [\"a\\\"b\", \"c\"]"
    );
}

/// §FS-config-migrate.3.1.8: migrating a version-1 scalar that closes early is refused at its
/// key's line with the scalar's version-1 repair, and nothing is written.
#[test]
fn migrating_a_version_1_scalar_that_closes_early_is_refused() {
    let fx = v1_folder();
    let before = fs::read(fx.0.join("grund.toml")).unwrap();
    let report = "grund.toml: version 1 → 2, preview, nothing written\n\
                  refused: grund.toml:8: [[kinds]] folder reads the one string `docs\" \"x`, and \
                  version 2 ends a string at its first unescaped quote\n  \
                  repair: end the string at its closing quote under version 1: write `\\\"` for \
                  a quote inside it; grund check shows its effect\n\
                  would not write: 1 refusal (follow the repairs above)\n";
    fx.expect(&["config", "migrate", "--to", "2"], 1, "", report);
    assert_eq!(fs::read(fx.0.join("grund.toml")).unwrap(), before);
}

/// §FS-config-migrate.3.1.8: migrating a version-1 element that closes early is refused at
/// its key's line with both halves of an element's version-1 repair.
#[test]
fn migrating_a_version_1_element_that_closes_early_is_refused() {
    let fx = v1_include();
    let report = "grund.toml: version 1 → 2, preview, nothing written\n\
                  refused: grund.toml:12: [scan] include reads the one string `a\"b`, and \
                  version 2 ends a string at its first unescaped quote\n  \
                  repair: end the string at its closing quote under version 1: write `\\\"` for \
                  a quote inside it, or a `,` between two strings; grund check shows its effect\n\
                  would not write: 1 refusal (follow the repairs above)\n";
    fx.expect(&["config", "migrate", "--to", "2"], 1, "", report);
}
