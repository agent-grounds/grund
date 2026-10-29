//! Black-box: a version-2 array takes one trailing comma, after an element
//! (§FS-config-v2.reader.5.commas). Ports grund.158's reproducer (#1087): through grund 0.21.x
//! an array whose run of commas before its `]` holds a comma with no element before it is read
//! as before and warns at its key's line, naming the line of that comma and 0.22.0
//! (§FS-config-v2.reader.5.commas.window, §FS-config-v2.checks.config-not-toml), while one
//! trailing comma after an element, and every shape refused before, keep their verdicts.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const SPEC: &str = "# SPEC-one: One\n\nLead.\n";
const CODE: &str = "// Realizes \u{a7}SPEC-one.\nfn main() {}\n";

/// The version-2 config every case extends, `[schema.sources]` on line 8 and its `exclude`
/// key on line 9, as in grund.158's fixture.
const HEAD: &str = "grund_config_version = 2\n\n[schema.kinds.code]\nfolders = [\"src\"]\n\n\
                    [schema.kinds.SPEC]\nfiles = [\"specs/spec.md\"]\n[schema.sources]\n";

/// The bare refusal of an array the readers before 0.21.0 refused too.
const REFUSAL: &str = "error: grund.toml:9: expected string\n";

/// The warning for the array at line 9 whose first comma with no element before it stands on
/// line `line` (§FS-config-v2.reader.5.commas.window).
fn commas(line: usize) -> String {
    format!(
        "warning: grund.toml:9: expected string, read for now with every comma before the `]` \
         taken off; delete each comma on line {line} that has no element before it; this \
         warning becomes an error in grund 0.22.0\n"
    )
}

struct Fixture(PathBuf);

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Fixture {
    /// A clean version-2 tree whose `exclude`, on line 9, is `value`, which may run on
    /// across the lines after it.
    fn exclude(value: &str) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("grund-1087-{}-{serial}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let config = format!("{HEAD}exclude = {value}\n");
        let files = [
            ("grund.toml", config.as_str()),
            ("specs/spec.md", SPEC),
            ("src/main.rs", CODE),
        ];
        for (path, contents) in files {
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
    fn shown(&self, stderr: &str) -> String {
        let run = self.run(&["config", "show"]);
        assert_eq!(
            (run.code, run.stderr.as_str()),
            (Some(0), stderr),
            "{}",
            run.stdout
        );
        run.stdout
            .split("\n[")
            .find(|section| section.starts_with("schema.sources]\n"))
            .and_then(|section| section.lines().find(|line| line.starts_with("exclude = ")))
            .unwrap_or_else(|| panic!("no [schema.sources] exclude in:\n{}", run.stdout))
            .to_string()
    }

    /// `check --format json`'s stderr lines, requiring exit 0 and an empty stdout.
    fn json(&self) -> Vec<String> {
        let run = self.run(&["check", "--format", "json"]);
        assert_eq!(
            (run.code, run.stdout.as_str()),
            (Some(0), ""),
            "{}",
            run.stderr
        );
        run.stderr.lines().map(str::to_string).collect()
    }

    /// The window's whole contract for an array whose one finding is `warning`: `check`,
    /// `config validate` and `config show` print it and exit 0, `config show` prints the
    /// value read as before, `shown`, JSON carries the code and the key's line, and
    /// `--ignore config-not-toml` brings back `success`.
    fn warns(&self, warning: &str, shown: &str) {
        self.expect(&["check"], 0, "", warning);
        self.expect(&["config", "validate"], 0, "", warning);
        assert_eq!(self.shown(warning), shown);
        let lines = self.json();
        assert_eq!(lines.len(), 1, "{lines:?}");
        for part in [
            "\"severity\":\"warning\"",
            "\"path\":\"grund.toml\"",
            "\"line\":9",
            "\"code\":\"config-not-toml\"",
            "read for now with every comma before the `]` taken off",
            "this warning becomes an error in grund 0.22.0",
        ] {
            assert!(lines[0].contains(part), "{part} missing from {}", lines[0]);
        }
        self.expect(
            &["check", "--ignore", "config-not-toml"],
            0,
            "success\n",
            "",
        );
    }

    /// The loads-silently contract: `check` prints `success`, `config validate` nothing, and
    /// `config show` prints `shown` with nothing on stderr.
    fn loads(&self, shown: &str) {
        self.expect(&["check"], 0, "success\n", "");
        self.expect(&["config", "validate"], 0, "", "");
        assert_eq!(self.shown(""), shown);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// grund.158's case, line 9, one comma against two: `["ab",]` loads as `["ab"]` with no word,
/// and `["ab",,]` still loads as `["ab"]` but warns, naming line 9 and 0.22.0.
#[test]
fn one_trailing_comma_loads_and_a_second_one_warns() {
    Fixture::exclude("[\"ab\",]").loads("exclude = [\"ab\"]");
    Fixture::exclude("[\"ab\",,]").warns(&commas(9), "exclude = [\"ab\"]");
}

/// §FS-config-v2.reader.5.commas.window: every run of two or more adjacent commas after the
/// last element is taken off as before, whatever whitespace stands around it, and warns.
#[test]
fn adjacent_commas_after_the_last_element_warn_and_are_read_as_before() {
    for value in ["[\"ab\" ,, ]", "[\"ab\",,,]"] {
        Fixture::exclude(value).warns(&commas(9), "exclude = [\"ab\"]");
    }
}

/// §FS-config-v2.reader.5.commas.window: in an array written across lines the warning stays
/// at the key's line, and its remedy names line 10, where the doubled comma stands.
#[test]
fn a_doubled_comma_on_a_later_line_names_the_line_it_stands_on() {
    Fixture::exclude("[\n  \"ab\",,\n]").warns(&commas(10), "exclude = [\"ab\"]");
}

/// §FS-config-v2.reader.5.commas.window: an array with no element and one comma, or one run of
/// them, is read as the empty list and warns, naming the comma's line; a comment beside the
/// `[` is read as whitespace is.
#[test]
fn an_array_of_commas_alone_warns_and_is_read_as_empty() {
    for (value, line) in [
        ("[,]", 9),
        ("[ , ]", 9),
        ("[,,]", 9),
        ("[\n  ,\n]", 10),
        ("[ # c\n  ,\n]", 10),
    ] {
        Fixture::exclude(value).warns(&commas(line), "exclude = []");
    }
}

/// §FS-config-v2.reader.5.commas.window: an array with an open line and a doubled comma earns
/// both warnings. `config validate` and `config show` keep the config's order, the open line's
/// first; `check` sorts them (§FS-errors.4.1), so there the commas' comes first.
#[test]
fn an_open_line_and_a_doubled_comma_earn_both_warnings() {
    let fx = Fixture::exclude("[\n  \"a\nb\",,\n]");
    let open = "warning: grund.toml:9: unclosed array (a string is still open at the end of \
                line 10), read for now with the line break as a space; close the string on \
                line 10; this warning becomes an error in grund 0.22.0\n";
    let commas = commas(11);
    fx.expect(&["check"], 0, "", &format!("{commas}{open}"));
    let config_order = format!("{open}{commas}");
    fx.expect(&["config", "validate"], 0, "", &config_order);
    assert_eq!(fx.shown(&config_order), "exclude = [\"a b\"]");
    let lines = fx.json();
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(
        lines[0].contains("every comma before the `]`"),
        "{}",
        lines[0]
    );
    assert!(
        lines[1].contains("still open at the end of line 10"),
        "{}",
        lines[1]
    );
    for line in &lines {
        assert!(line.contains("\"code\":\"config-not-toml\""), "{line}");
        assert!(line.contains("\"line\":9"), "{line}");
    }
    fx.expect(
        &["check", "--ignore", "config-not-toml"],
        0,
        "success\n",
        "",
    );
}

/// §FS-config-v2.reader.5.commas.window: an array with a string in it that closes early does not
/// load, so it earns neither warning; it is refused at its key's line as the early close, with
/// the release that made it one (§FS-config-v2.reader.6.refusal).
#[test]
fn an_early_close_beside_a_doubled_comma_is_refused_as_the_early_close() {
    let fx = Fixture::exclude("[\"a\" \"b\",,]");
    let refusal = "error: grund.toml:9: expected string; this became an error in grund 0.22.0\n";
    fx.expect(&["check"], 2, "", refusal);
    fx.expect(&["config", "validate"], 1, "", refusal);
    fx.expect(&["config", "show"], 2, "", refusal);
    fx.expect(&["check", "--format", "json"], 2, "", refusal);
}

/// Guard, green today and kept: a shape the readers before refused, because whitespace or an
/// element stood between two commas, keeps its bare `expected string` and earns no warning
/// (§FS-config-v2.reader.5.commas, §FS-config-v2.reader.5.commas.refusal).
#[test]
fn shapes_refused_before_keep_their_bare_refusal() {
    for value in [
        "[,\"a\"]",
        "[\"a\",,\"b\"]",
        "[\"ab\", ,]",
        "[ , , ]",
        "[\n  \"ab\",\n  ,\n]",
    ] {
        let fx = Fixture::exclude(value);
        fx.expect(&["check"], 2, "", REFUSAL);
        fx.expect(&["config", "validate"], 1, "", REFUSAL);
        fx.expect(&["config", "show"], 2, "", REFUSAL);
    }
}

/// Guard, green today and kept: an empty array, and one trailing comma after the last element,
/// load as written with no warning (§FS-config-v2.reader.5.commas).
#[test]
fn one_trailing_comma_after_an_element_and_empty_arrays_load_with_no_warning() {
    for (value, shown) in [
        ("[]", "exclude = []"),
        ("[ ]", "exclude = []"),
        ("[\n]", "exclude = []"),
        ("[\"ab\",]", "exclude = [\"ab\"]"),
        ("[\"ab\" , ]", "exclude = [\"ab\"]"),
        ("[\n  \"ab\"\n  ,\n]", "exclude = [\"ab\"]"),
    ] {
        Fixture::exclude(value).loads(shown);
    }
}
