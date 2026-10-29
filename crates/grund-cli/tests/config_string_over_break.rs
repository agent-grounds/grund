//! Black-box: a string in a version-2 array closes on the line it opens on
//! (§FS-config-v2.reader.5.strings). Ports grund.160's reproducer (#1089): through grund
//! 0.21.x an array a later line closes is read as before, the break as a space, and warns at
//! its key's line naming the open line and 0.22.0 (§FS-config-v2.reader.5.strings.window,
//! §FS-config-v2.checks.config-not-toml); a `"""` string says version 2 does not read it
//! (§FS-config-v2.reader.5.strings.multi-line); an array that never closes names the open
//! line in its `unclosed array` refusal (§FS-config-v2.reader.5); and `config show` writes a
//! line break inside a value as `\n`, so its output loads back (§FS-config.4.2).

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const SPEC: &str = "# SPEC-one: One\n\nLead.\n";
const CODE: &str = "// Realizes \u{a7}SPEC-one.\nfn main() {}\n";

/// The config every case writes, `exclude`'s key on line 10.
const HEAD: &str = "grund_config_version = 2\n\n[schema.kinds.SPEC]\nfiles = [\"specs/spec.md\"]\n\n\
                    [schema.kinds.code]\nfolders = [\"src\"]\n\n[schema.sources]\nexclude = ";

/// The `config-not-toml` warning for the array at line 10 whose string is open at the end
/// of line `open`.
fn not_toml(open: usize) -> String {
    format!(
        "warning: grund.toml:10: unclosed array (a string is still open at the end of line \
         {open}), read for now with the line break as a space; close the string on line {open}; \
         this warning becomes an error in grund 0.22.0\n"
    )
}

/// The refusal of the array at line 10 that no line closes, its string open at the end of
/// line `open`.
fn unclosed(open: usize) -> String {
    format!(
        "error: grund.toml:10: unclosed array (a string is still open at the end of line \
         {open})\n"
    )
}

struct Fixture(PathBuf);

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Fixture {
    /// A clean tree whose `grund.toml` writes `exclude = <exclude>` at line 10.
    fn excluding(exclude: &str) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("grund-1089-{}-{serial}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let config = format!("{HEAD}{exclude}");
        for (path, contents) in [
            ("grund.toml", config.as_str()),
            ("specs/spec.md", SPEC),
            ("src/main.rs", CODE),
        ] {
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

    /// `config show`'s `exclude` line under `[schema.sources]`, requiring exit 0 and
    /// `stderr` exactly.
    fn shown_exclude(&self, stderr: &str) -> String {
        let run = self.run(&["config", "show"]);
        assert_eq!(
            (run.code, run.stderr.as_str()),
            (Some(0), stderr),
            "{}",
            run.stdout
        );
        run.stdout
            .split("\n[")
            .find(|table| table.starts_with("schema.sources]"))
            .and_then(|table| table.lines().find(|line| line.starts_with("exclude = ")))
            .unwrap_or_else(|| panic!("no [schema.sources] exclude in:\n{}", run.stdout))
            .to_string()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// grund.160's first case: `"a` / `b",` still loads as `a b`, and every command that reports
/// the config's own findings warns at the key's line, naming line 11 and 0.22.0.
#[test]
#[ignore = "red until #1089 warns config-not-toml; implement removes this"]
fn a_string_over_a_break_warns_and_is_read_as_before() {
    let fx = Fixture::excluding("[\n  \"a\nb\",\n]\n");
    fx.expect(&["check"], 0, "", &not_toml(11));
    fx.expect(&["config", "validate"], 0, "", &not_toml(11));
    assert_eq!(fx.shown_exclude(&not_toml(11)), "exclude = [\"a b\"]");
}

/// grund.160's second case: since #1083 kept a comma inside a string, `"a,` / `b",` loads
/// too, as `a, b`, and warns the same way.
#[test]
#[ignore = "red until #1089 warns config-not-toml; implement removes this"]
fn a_string_holding_a_comma_over_a_break_warns_and_is_read_as_before() {
    let fx = Fixture::excluding("[\n  \"a,\nb\",\n]\n");
    fx.expect(&["check"], 0, "", &not_toml(11));
    assert_eq!(fx.shown_exclude(&not_toml(11)), "exclude = [\"a, b\"]");
}

/// The key's own line can be the open line, and only the first open line is named.
#[test]
#[ignore = "red until #1089 warns config-not-toml; implement removes this"]
fn the_warning_names_the_first_line_that_ends_inside_a_string() {
    Fixture::excluding("[\"a\nb\"]\n").expect(&["check"], 0, "", &not_toml(10));
    Fixture::excluding("[\n  \"a\n  b\n  c\",\n]\n").expect(&["check"], 0, "", &not_toml(11));
}

/// A `\"` at a line's end escapes the quote, so the string is open there: `"a\"` / `b",`
/// is read as `a" b` and warns at line 11.
#[test]
#[ignore = "red until #1089 warns config-not-toml; implement removes this"]
fn an_escaped_quote_at_a_line_end_leaves_the_string_open() {
    let fx = Fixture::excluding("[\n  \"a\\\"\n  b\",\n]\n");
    fx.expect(&["check"], 0, "", &not_toml(11));
    assert_eq!(fx.shown_exclude(&not_toml(11)), "exclude = [\"a\\\" b\"]");
}

/// §FS-config-v2.checks.config-not-toml: the warning is one JSON finding with its code, the
/// config file and the key's line; `--ignore` brings back `success` and `--only` selects it.
/// It names 0.22.0 as the release it becomes the refusal in (§FS-config-v2.checks.config-not-toml.1).
#[test]
#[ignore = "red until #1089 warns config-not-toml; implement removes this"]
fn the_warning_is_a_selectable_finding_with_its_path_and_line() {
    let fx = Fixture::excluding("[\n  \"a\nb\",\n]\n");
    let run = fx.run(&["check", "--format", "json"]);
    assert_eq!(
        (run.code, run.stdout.as_str()),
        (Some(0), ""),
        "{}",
        run.stderr
    );
    let lines: Vec<&str> = run.stderr.lines().collect();
    assert_eq!(lines.len(), 1, "{}", run.stderr);
    for part in [
        "\"severity\":\"warning\"",
        "\"path\":\"grund.toml\"",
        "\"line\":10",
        "\"code\":\"config-not-toml\"",
        "a string is still open at the end of line 11",
        "this warning becomes an error in grund 0.22.0",
    ] {
        assert!(lines[0].contains(part), "{part} missing from {}", lines[0]);
    }
    fx.expect(
        &["check", "--ignore", "config-not-toml"],
        0,
        "success\n",
        "",
    );
    fx.expect(
        &["check", "--only", "config-not-toml"],
        0,
        "",
        &not_toml(11),
    );
}

/// §FS-config-v2.reader.5.strings.multi-line: a string opened with `"""` is TOML, so it
/// is not `config-not-toml`; it says version 2 does not read it, and loads as before.
/// Its code is its own (§FS-config-v2.checks.config-multiline-string).
#[test]
#[ignore = "red until #1089 warns config-multiline-string; implement removes this"]
fn a_multi_line_string_warns_that_version_2_does_not_read_it() {
    let fx = Fixture::excluding("[\n  \"\"\"a\nb\"\"\",\n]\n");
    let warning = "warning: grund.toml:10: unclosed array (a multi-line string opens on line \
                   11, which version 2 does not read), read for now with the line break as a \
                   space; write the string on line 11 as one basic string; this warning \
                   becomes an error in grund 0.22.0\n";
    fx.expect(&["check"], 0, "", warning);
    assert_eq!(
        fx.shown_exclude(warning),
        "exclude = [\"\\\"\\\"a b\\\"\\\"\"]"
    );
    let run = fx.run(&["check", "--format", "json"]);
    assert!(
        run.stderr.contains("\"code\":\"config-multiline-string\""),
        "{}",
        run.stderr
    );
}

/// An array that never closes is refused as before, at its key's line, now naming its open
/// line; the `#` on the line after it is never read as a comment that swallows the quote,
/// and the refusal claims no release, because it was refused before.
#[test]
#[ignore = "red until #1089 names the open line in `unclosed array`; implement removes this"]
fn an_array_that_never_closes_names_its_open_line() {
    let fx = Fixture::excluding("[\n  \"a\n#b\",\n]\n");
    fx.expect(&["check"], 2, "", &unclosed(11));
    fx.expect(&["config", "show"], 2, "", &unclosed(11));
    Fixture::excluding("[\"a,b]\n").expect(&["check"], 2, "", &unclosed(10));
}

/// §FS-config.4.2: a line break, a tab and a carriage return inside a value are written as
/// their escapes, and what `config show` prints loads back to the same values.
#[test]
#[ignore = "red until #1089 escapes control characters on the v2 path; implement removes this"]
fn config_show_writes_a_line_break_inside_a_value_as_its_escape() {
    let fx = Fixture::excluding("[\"a\\nb\", \"c\\td\\re\"]\n");
    let shown = "exclude = [\"a\\nb\", \"c\\td\\re\"]";
    assert_eq!(fx.shown_exclude(""), shown);
    let printed = fx.run(&["config", "show"]).stdout;
    fs::write(fx.0.join("grund.toml"), &printed).unwrap();
    assert_eq!(fx.shown_exclude(""), shown);
}

/// Guard, green today and kept: strings that each close on their own line, a comma inside
/// one and a trailing comma after the last, load as written with no warning.
#[test]
fn strings_closed_on_their_own_lines_load_with_no_warning() {
    let fx = Fixture::excluding("[\n  \"a,b\",\n  \"c\",\n]\n");
    fx.expect(&["check"], 0, "success\n", "");
    assert_eq!(fx.shown_exclude(""), "exclude = [\"a,b\", \"c\"]");
}
