//! Black-box: a version-2 array takes one trailing comma, after an element
//! (§FS-config-v2.reader.5.commas). Ports grund.158's reproducer (#1087) and #1098's: from grund
//! 0.22.0 an array whose run of commas before its `]` holds a comma with no element before it is
//! refused at its key's line as `expected string`, ending with the release that made it one
//! where the array's text loaded through the 0.21.x window
//! (§FS-config-v2.reader.5.commas.refusal), while one trailing comma after an element, and every
//! shape refused before, keep their verdicts.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const SPEC: &str = "# SPEC-one: One\n\nLead.\n";
const CODE: &str = "// Realizes \u{a7}SPEC-one.\nfn main() {}\n";

/// The version-2 config every case extends, `[schema.sources]` on line 8 and the key under
/// test on line 9, as in grund.158's fixture.
const HEAD: &str = "grund_config_version = 2\n\n[schema.kinds.code]\nfolders = [\"src\"]\n\n\
                    [schema.kinds.SPEC]\nfiles = [\"specs/spec.md\"]\n[schema.sources]\n";

/// The bare refusal of an array the readers before 0.21.0 refused too.
const REFUSAL: &str = "error: grund.toml:9: expected string\n";

/// The refusal of an array at line 9 whose text loaded through the window
/// (§FS-config-v2.reader.5.commas.refusal).
const BECAME: &str = "error: grund.toml:9: expected string; this became an error in grund 0.22.0\n";

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
        Self::sources("exclude", value)
    }

    /// A clean version-2 tree whose `[schema.sources]` key `key`, on line 9, is `value`.
    fn sources(key: &str, value: &str) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("grund-1087-{}-{serial}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let config = format!("{HEAD}{key} = {value}\n");
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

    /// The refusal's whole contract (§FS-config.4.3): `check` and `config show` exit 2 and
    /// `config validate` 1, each printing `refusal` alone on stderr and nothing on stdout, and
    /// `--ignore config-not-toml` drops nothing, because a refused load is no finding.
    fn refused(&self, refusal: &str) {
        self.expect(&["check"], 2, "", refusal);
        self.expect(&["config", "validate"], 1, "", refusal);
        self.expect(&["config", "show"], 2, "", refusal);
        self.expect(&["check", "--ignore", "config-not-toml"], 2, "", refusal);
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

/// grund.158's case, line 9, one comma against two (§FS-config-v2.reader.5.commas.refusal):
/// `["ab",]` loads as `["ab"]` with no word, and `["ab",,]` is refused, naming 0.22.0.
#[test]
fn one_trailing_comma_loads_and_a_second_one_is_refused() {
    Fixture::exclude("[\"ab\",]").loads("exclude = [\"ab\"]");
    Fixture::exclude("[\"ab\",,]").refused(BECAME);
}

/// §FS-config-v2.reader.5.commas.refusal: every run of two or more adjacent commas after the
/// last element is refused, naming 0.22.0, whatever whitespace stands around the run.
#[test]
fn adjacent_commas_after_the_last_element_are_refused() {
    for value in ["[\"ab\" ,, ]", "[\"ab\",,,]"] {
        Fixture::exclude(value).refused(BECAME);
    }
}

/// §FS-config-v2.reader.5.commas.refusal: in an array written across lines the refusal is at
/// the key's line, line 9, though the doubled comma stands on line 10.
#[test]
fn a_doubled_comma_on_a_later_line_is_refused_at_the_keys_line() {
    Fixture::exclude("[\n  \"ab\",,\n]").refused(BECAME);
}

/// §FS-config-v2.reader.5.commas.refusal: an array with no element and one comma, or one run of
/// them, is refused, naming 0.22.0; a comment beside the `[` is read as whitespace is.
#[test]
fn an_array_of_commas_alone_is_refused() {
    for value in ["[,]", "[ , ]", "[,,]", "[\n  ,\n]", "[ # c\n  ,\n]"] {
        Fixture::exclude(value).refused(BECAME);
    }
}

/// §FS-config-v2.reader.5.commas.refusal: the suffix is the array's text's, not the key's, so a
/// window shape under a key that would refuse its element anyway still names 0.22.0: 0.21.x
/// said `unknown language` for `["nope",,]`. One trailing comma after the element keeps that
/// refusal, and a shape refused before keeps its bare `expected string`.
#[test]
fn a_window_shape_names_0_22_whatever_its_key_would_have_said() {
    Fixture::sources("languages", "[\"nope\",,]").refused(BECAME);
    let unknown = "error: grund.toml:9: unknown language `nope` in [schema.sources] languages\n";
    Fixture::sources("languages", "[\"nope\",]").refused(unknown);
    Fixture::sources("languages", "[\"nope\", ,]").refused(REFUSAL);
}

/// §FS-config-v2.reader.5.commas.refusal, pinned while the open-line window of
/// §FS-config-v2.reader.5.strings.window is still open: the lines are joined, and the joined
/// array's doubled comma is refused, and its open line earns no warning beside the refusal.
#[test]
fn an_open_line_and_a_doubled_comma_are_refused_alone() {
    Fixture::exclude("[\n  \"a\nb\",,\n]").refused(BECAME);
}

/// §FS-config-v2.reader.5.commas.refusal: an array with a string in it that closes early as
/// well is refused once, at its key's line, with the release that made it one
/// (§FS-config-v2.reader.6.refusal); neither construct earns a warning beside it.
#[test]
fn an_early_close_beside_a_doubled_comma_is_refused_as_the_early_close() {
    let fx = Fixture::exclude("[\"a\" \"b\",,]");
    let refusal = "error: grund.toml:9: expected string; this became an error in grund 0.22.0\n";
    fx.expect(&["check"], 2, "", refusal);
    fx.expect(&["config", "validate"], 1, "", refusal);
    fx.expect(&["config", "show"], 2, "", refusal);
    fx.expect(&["check", "--format", "json"], 2, "", refusal);
}

/// §FS-config-v2.reader.5.commas.refusal: the elements are read before the suffix is earned, so
/// where the run's removal leaves an element that is not a string, that element's bare refusal
/// is the array's, as 0.21.x refused it (#1110's review, R1-01).
#[test]
fn an_element_refused_first_keeps_its_bare_refusal() {
    Fixture::exclude("[1,,]").refused(REFUSAL);
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
        "[\n  ,\n  ,\n]",
    ] {
        Fixture::exclude(value).refused(REFUSAL);
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
