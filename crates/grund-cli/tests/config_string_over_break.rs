//! Black-box: a string in a version-2 array closes on the line it opens on
//! (§FS-config-v2.reader.5.strings). Ports grund.160's reproducer (#1089) and #1118's: from
//! grund 0.22.0 the open line is the array's last, so an array is refused at its key's line as
//! `unclosed array`, naming the open line, and the release that made it one where the readers
//! before would have loaded it (§FS-config-v2.reader.5.strings.refusal); a `"""` string says
//! version 2 does not read it (§FS-config-v2.reader.5.strings.multi-line); an array they refused
//! too, one that never closes among them, keeps its bare refusal; `config-not-toml` and
//! `config-multiline-string` are spent (§FS-config-v2.checks.config-not-toml); and `config show`
//! writes a line break inside a value as `\n`, so its output loads back (§FS-config.4.2).

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const SPEC: &str = "# SPEC-one: One\n\nLead.\n";
const CODE: &str = "// Realizes \u{a7}SPEC-one.\nfn main() {}\n";

/// The config every case writes, `exclude`'s key on line 10.
const HEAD: &str = "grund_config_version = 2\n\n[schema.kinds.SPEC]\nfiles = [\"specs/spec.md\"]\n\n\
                    [schema.kinds.code]\nfolders = [\"src\"]\n\n[schema.sources]\nexclude = ";

/// The 0.22.0 refusal of the array at line 10 whose string is open at the end of line
/// `open`, which loaded through the 0.21.x window (§FS-config-v2.reader.5.strings.refusal).
fn became(open: usize) -> String {
    format!(
        "error: grund.toml:10: unclosed array (a string is still open at the end of line \
         {open}); this became an error in grund 0.22.0\n"
    )
}

/// The refusal of the array at line 10 that the readers before refused too, its string open
/// at the end of line `open`.
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

    /// The refusal's whole contract (§FS-config.4.3): `check` and `config show` exit 2 and
    /// `config validate` 1, each printing `refusal` alone, and neither `--format json` nor
    /// `--ignore config-not-toml` changes it, because a refused load is no finding.
    fn refused(&self, refusal: &str) {
        self.expect(&["check"], 2, "", refusal);
        self.expect(&["config", "validate"], 1, "", refusal);
        self.expect(&["config", "show"], 2, "", refusal);
        self.expect(&["check", "--format", "json"], 2, "", refusal);
        self.expect(&["check", "--ignore", "config-not-toml"], 2, "", refusal);
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

/// grund.160's first case, §FS-config-v2.reader.5.strings.refusal: `"a` / `b",` loaded as
/// `a b` through 0.21.x, and from 0.22.0 its open line, line 11, ends the array, which is
/// refused at its key's line, naming that line and 0.22.0, where `config-not-toml` warned
/// (§FS-config-v2.checks.config-not-toml.1).
#[test]
fn a_string_over_a_break_is_refused_naming_its_open_line() {
    Fixture::excluding("[\n  \"a\nb\",\n]\n").refused(&became(11));
}

/// grund.160's second case, §FS-config-v2.reader.5.strings.refusal: `"a,` / `b",`, which
/// loaded as `a, b`, is refused the same way.
#[test]
fn a_string_holding_a_comma_over_a_break_is_refused() {
    Fixture::excluding("[\n  \"a,\nb\",\n]\n").refused(&became(11));
}

/// §FS-config-v2.reader.5.strings: the key's own line can be the open line, and the first
/// open line is the array's last, so it is the one named.
#[test]
fn the_refusal_names_the_first_line_that_ends_inside_a_string() {
    Fixture::excluding("[\"a\nb\"]\n").refused(&became(10));
    Fixture::excluding("[\n  \"a\n  b\n  c\",\n]\n").refused(&became(11));
}

/// §FS-config-v2.reader.5.strings: a `\"` at a line's end escapes the quote, so the string
/// is open there, and `"a\"` / `b",` is refused naming line 11.
#[test]
fn an_escaped_quote_at_a_line_end_leaves_the_string_open() {
    Fixture::excluding("[\n  \"a\\\"\n  b\",\n]\n").refused(&became(11));
}

/// §FS-config-v2.reader.5.strings.multi-line: a string opened with `"""` is TOML, so the
/// refusal says version 2 does not read it, with the suffix, because it loaded through the
/// window as `""a b""`.
#[test]
fn a_multi_line_string_over_a_break_is_refused_as_one_version_2_does_not_read() {
    let refusal = "error: grund.toml:10: unclosed array (a multi-line string opens on line 11, \
                   which version 2 does not read); this became an error in grund 0.22.0\n";
    Fixture::excluding("[\n  \"\"\"a\nb\"\"\",\n]\n").refused(refusal);
}

/// §FS-config-v2.reader.5.strings, §FS-config-v2.reader.5.strings.multi-line: three quotes in a
/// row open a multi-line string, so `""""a` leaves line 11 open, the line TOML names, where
/// reading quote by quote named line 12 (#1091's review, R1-04). The readers before did not
/// close the array either, so the refusal claims no release.
#[test]
fn a_run_of_four_quotes_opens_a_multi_line_string_on_its_own_line() {
    let refusal = "error: grund.toml:10: unclosed array (a multi-line string opens on line 11, \
                   which version 2 does not read)\n";
    Fixture::excluding("[\n\"\"\"\"a\nb\"\"\",\n\"c\",\n]\n").refused(refusal);
}

/// §FS-config-v2.reader.5.strings.refusal: the suffix says the readers before loaded the array,
/// their reading decides it end to end, so `"""a"`, in which reading quote by quote met no open
/// line, loaded as `""a` and names 0.22.0, and `"a` / `b", 1,`, which that reading closed and
/// then refused for its `1`, claims no release.
#[test]
fn the_refusal_names_0_22_exactly_where_the_readers_before_loaded_the_array() {
    let refusal = "error: grund.toml:10: unclosed array (a multi-line string opens on line 11, \
                   which version 2 does not read); this became an error in grund 0.22.0\n";
    Fixture::excluding("[\n  \"\"\"a\"\n]\n").refused(refusal);
    Fixture::excluding("[\n  \"a\nb\", 1,\n]\n").refused(&unclosed(11));
}

/// §FS-config-v2.reader.5.strings: a line's `#` comment is found a quote at a time, so on a line
/// holding a `"""` run comment text can move the refusal: in `"""a"b""", # "x` the `#` falls
/// inside a string read quote by quote, and the comment's quote leaves line 11 open, where TOML
/// reads the string `a"b` and a comment. The readers before refused it too.
#[test]
fn a_comment_beside_a_triple_quoted_string_is_found_a_quote_at_a_time() {
    Fixture::excluding("[\n  \"\"\"a\"b\"\"\", # \"x\n]\n").refused(&unclosed(11));
}

/// §FS-config-v2.checks.config-not-toml, §FS-config-v2.checks.config-multiline-string: from
/// 0.22.0 nothing fires either code, so each is spent and stays
/// (§FS-config-v2.checks.config-not-toml.1): `--ignore` still accepts it and drops nothing, and
/// `--only` judges nothing and says so, holding back `success` (§FS-check.1.4.1).
#[test]
fn both_codes_are_spent_and_say_so_when_selected() {
    let fx = Fixture::excluding("[\n  \"a,b\",\n  \"c\",\n]\n");
    for code in ["config-not-toml", "config-multiline-string"] {
        fx.expect(&["check", "--ignore", code], 0, "success\n", "");
        let off = format!(
            "warning: --only {code} judged nothing \u{2014} the check is spent from grund 0.22.0 \
             and never fires\n"
        );
        fx.expect(&["check", "--only", code], 0, "", &off);
    }
}

/// Guard, green today and kept (§FS-config-v2.reader.5.strings.refusal): an array the readers
/// before never closed is refused as before, at its key's line, naming its open line; the `#`
/// on the line after it is never read as a comment that swallows the quote, and the refusal
/// claims no release, because it was refused before.
#[test]
fn an_array_that_never_closes_names_its_open_line() {
    let fx = Fixture::excluding("[\n  \"a\n#b\",\n]\n");
    fx.expect(&["check"], 2, "", &unclosed(11));
    fx.expect(&["config", "show"], 2, "", &unclosed(11));
    Fixture::excluding("[\"a,b]\n").expect(&["check"], 2, "", &unclosed(10));
}

/// §FS-config.4.2: a line break, a tab and a carriage return inside a value are written as
/// their escapes, and what `config show` prints loads back to the same values.
#[test]
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
