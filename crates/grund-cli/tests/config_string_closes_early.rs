//! Black-box: a version-2 string ends at its first unescaped quote (§FS-config-v2.reader.6).
//! Ports grund.159's reproducer (#1088) and #1094's: from grund 0.22.0 a scalar, an element or
//! a quoted key whose string closes early is refused at its key's line as `expected string`,
//! ending with the release that made it one, where it loaded through the 0.21.x window
//! (§FS-config-v2.reader.6.refusal); a `"""` string on one line is refused with a parenthetical
//! saying version 2 does not read it (§FS-config-v2.reader.6.multi-line). A value refused
//! before keeps the bare message, and #1089's open-line window is untouched. Version 1 keeps its
//! reading for good (§FS-config.5.2) and meets the rule at `config migrate`
//! (§FS-config-migrate.3.1.8).

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const SPEC: &str = "# SPEC-one: One\n\nLead.\n";
const CODE: &str = "// Realizes \u{a7}SPEC-one.\nfn main() {}\n";

/// The version-2 config every case extends, its next line line 8.
const HEAD: &str = "grund_config_version = 2\n\n[schema.kinds.code]\nfolders = [\"src\"]\n\n\
                    [schema.kinds.SPEC]\nfiles = [\"specs/spec.md\"]\n";

/// The 0.22.0 refusal of a string that closes early in the value at line `key`, which
/// loaded through the 0.21.x window (§FS-config-v2.reader.6.refusal).
fn refusal(key: usize) -> String {
    format!("error: grund.toml:{key}: expected string; this became an error in grund 0.22.0\n")
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

    /// The refusal's whole contract for a config whose load fails with `refusal`: `check`
    /// and `config show` exit 2, `config validate` 1, each printing it alone, and a JSON run
    /// prints it as the same CLI-level error (§FS-config.4.3).
    fn refuses(&self, refusal: &str) {
        self.expect(&["check"], 2, "", refusal);
        self.expect(&["config", "validate"], 1, "", refusal);
        self.expect(&["config", "show"], 2, "", refusal);
        self.expect(&["check", "--format", "json"], 2, "", refusal);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// grund.159's first case: `fetch = "one" "two"` loaded as the one string `one" "two` through
/// 0.21.x, and is refused from 0.22.0 with the suffix.
#[test]
fn a_scalar_that_closes_early_is_refused() {
    Fixture::v2("fetch = \"one\" \"two\"\n").refuses(&refusal(8));
}

/// grund.159's second case: `folders = ["a" "b"]`, a comma forgotten, is refused.
#[test]
fn an_element_that_closes_early_is_refused() {
    Fixture::v2("folders = [\"a\" \"b\"]\n").refuses(&refusal(8));
}

/// grund.159's third case: a `[presentation.kinds]` title is a scalar like any other.
#[test]
fn a_presentation_title_that_closes_early_is_refused() {
    Fixture::v2("\n[presentation.kinds.SPEC]\ntitle = \"one\" \"two\"\n").refuses(&refusal(10));
}

/// §FS-config-v2.reader.6.refusal: a quoted key is a string of its own, and is refused at
/// its line.
#[test]
fn a_quoted_key_that_closes_early_is_refused() {
    Fixture::v2("\n[schema.sources.extensions]\n\"md\" \"x\" = \"markdown\"\n")
        .refuses(&refusal(10));
}

/// §FS-config-v2.reader.6.multi-line: `title = """one"""` is refused with the parenthetical
/// that names the form, then the suffix.
#[test]
fn a_triple_quoted_string_on_one_line_is_refused_as_one_version_2_does_not_read() {
    let refusal = "error: grund.toml:10: expected string (a multi-line string, which version 2 \
                   does not read); this became an error in grund 0.22.0\n";
    Fixture::v2("\n[presentation.kinds.SPEC]\ntitle = \"\"\"one\"\"\"\n").refuses(refusal);
}

/// §FS-config-v2.reader.6.refusal: in an array written across lines the refusal stays at the
/// key's line, as every value's does.
#[test]
fn an_element_on_a_later_line_is_refused_at_its_key() {
    Fixture::v2("\n[schema.sources]\nexclude = [\n  \"a\",\n  \"b\" \"c\",\n]\n")
        .refuses(&refusal(10));
}

/// §FS-config-v2.reader.6.refusal: a value with two strings that close early is refused once.
#[test]
fn a_value_with_two_early_closes_is_refused_once() {
    Fixture::v2("\n[schema.sources]\nexclude = [\"a\" \"b\", \"c\" \"d\"]\n").refuses(&refusal(10));
}

/// §FS-config-v2.reader.6.refusal: an array with an open line is joined first, as #1089's
/// window still reads it, and its string that closes early is then refused; the open line's
/// warning is not printed, because the config does not load.
#[test]
fn an_open_line_and_an_early_close_are_refused_as_the_early_close() {
    Fixture::v2("\n[schema.sources]\nexclude = [\n  \"a\nb\" \"c\",\n]\n").refuses(&refusal(10));
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

/// §FS-config-v2.reader.6.multi-line: every `"""` string a line break runs through is #1089's
/// construct, not only the one across the first open line, so two of them earn its one warning.
#[test]
fn two_triple_quoted_strings_over_breaks_earn_only_the_open_line_warning() {
    let fx = Fixture::v2(
        "\n[schema.sources]\nexclude = [\n  \"\"\"a\nb\"\"\",\n  \"\"\"c\nd\"\"\",\n]\n",
    );
    let warning = "warning: grund.toml:10: unclosed array (a multi-line string opens on line \
                   11, which version 2 does not read), read for now with the line break as a \
                   space; write the string on line 11 as one basic string; this warning \
                   becomes an error in grund 0.22.0\n";
    fx.expect(&["check"], 0, "", warning);
    fx.expect(&["config", "validate"], 0, "", warning);
}

/// §FS-config-v2.reader.6.multi-line: a `"""` string over a break after a plain open line is
/// #1089's construct too, and earns nothing beyond that array's one warning.
#[test]
fn a_triple_quoted_string_over_a_break_after_a_plain_open_line_earns_only_that_warning() {
    let fx = Fixture::v2("\n[schema.sources]\nexclude = [\n  \"a\nb\",\n  \"\"\"c\nd\"\"\",\n]\n");
    fx.expect(&["check"], 0, "", OPEN_LINE);
    fx.expect(&["config", "validate"], 0, "", OPEN_LINE);
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

/// A version-1 tree whose `[scan] exclude` is `["a"b", "c"]`, its key on line 13. No file
/// path holds the quote, which Windows refuses in a name.
fn v1_exclude() -> Fixture {
    Fixture::with(
        "grund_config_version = 1\n\n[id]\nnamed_sections = true\n\n[[kinds]]\nkind = \"SPEC\"\n\
         folder = \"docs\"\nindex = false\n\n[scan]\ninclude = [\"docs\", \"src\"]\n\
         exclude = [\"a\"b\", \"c\"]\n",
        &[
            ("docs/spec.md", "# SPEC-001-one: One\n\nLead.\n"),
            (
                "src/main.rs",
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
    let exclude = v1_exclude();
    exclude.expect(&["check"], 0, "", VERSION_1);
    assert_eq!(
        exclude.shown("scan", "exclude = ", VERSION_1),
        "exclude = [\"a\\\"b\", \"c\"]"
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
    let fx = v1_exclude();
    let report = "grund.toml: version 1 → 2, preview, nothing written\n\
                  refused: grund.toml:13: [scan] exclude reads the one string `a\"b`, and \
                  version 2 ends a string at its first unescaped quote\n  \
                  repair: end the string at its closing quote under version 1: write `\\\"` for \
                  a quote inside it, or a `,` between two strings; grund check shows its effect\n\
                  would not write: 1 refusal (follow the repairs above)\n";
    fx.expect(&["config", "migrate", "--to", "2"], 1, "", report);
}
