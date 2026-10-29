//! Black-box: a `'` string is one version 2 does not read (§FS-config-v2.reader.6.literal).
//! Ports grund.162's reproducer (#1103): a scalar or an element written as a TOML literal
//! string, `'…'`, or a multi-line literal string, `'''…'''`, is refused at its key's line as
//! `expected string`, exit 2, as before, and the refusal now names the form and the remedy.
//! Version 1 and any other malformed value keep the bare message.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const SPEC: &str = "# SPEC-one: One\n\nLead.\n";
const CODE: &str = "// Realizes \u{a7}SPEC-one.\nfn main() {}\n";

/// The version-2 config the table cases extend, its next line line 8.
const HEAD: &str = "grund_config_version = 2\n\n[schema.kinds.code]\nfolders = [\"src\"]\n\n\
                    [schema.kinds.SPEC]\nfiles = [\"specs/spec.md\"]\n";

/// The refusal of a literal string at line `line`, naming `form`.
fn refusal(line: usize, form: &str) -> String {
    format!(
        "error: grund.toml:{line}: expected string ({form}, which version 2 does not read); \
         write it as a basic string, \"...\", with \\\" for a quote inside it\n"
    )
}

struct Fixture(PathBuf);

impl Fixture {
    /// A tree whose `grund.toml` is `config`, holding one spec and one source file.
    fn with(config: &str) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("grund-1103-{}-{serial}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        for (path, contents) in [
            ("grund.toml", config),
            ("specs/spec.md", SPEC),
            ("src/main.rs", CODE),
        ] {
            let path = dir.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, contents).unwrap();
        }
        Self(dir)
    }

    /// Require `grund args…` to exit 2 with nothing on stdout and exactly `stderr`.
    fn refuses(&self, args: &[&str], stderr: &str) {
        let out = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .expect("run grund");
        assert_eq!(
            (
                out.status.code(),
                String::from_utf8_lossy(&out.stdout).as_ref(),
                String::from_utf8_lossy(&out.stderr).as_ref()
            ),
            (Some(2), "", stderr),
            "grund {}",
            args.join(" ")
        );
    }

    /// `check` and `schema` both refuse the config with `stderr`: every command that loads
    /// the config refuses it alike (§FS-config.4.3).
    fn refused(&self, stderr: &str) {
        self.refuses(&["check"], stderr);
        self.refuses(&["schema"], stderr);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
#[ignore = "red until #1103 names the literal-string form in parse_string's refusal; implement removes this"]
fn a_scalar_literal_string_is_refused_naming_its_form() {
    Fixture::with("grund_config_version = 2\nproject_name = 'x'\n")
        .refused(&refusal(2, "a literal string"));
}

#[test]
#[ignore = "red until #1103 names the literal-string form in parse_string's refusal; implement removes this"]
fn a_presentation_title_literal_string_is_refused_naming_its_form() {
    // The issue's case: the quotes, `&` and `*` inside are not what is refused.
    Fixture::with(&format!(
        "{HEAD}\n[presentation.kinds.SPEC]\ntitle = 'Drawbacks & \"open\" *questions*'\n"
    ))
    .refused(&refusal(10, "a literal string"));
}

#[test]
#[ignore = "red until #1103 names the literal-string form in parse_string's refusal; implement removes this"]
fn a_multi_line_literal_string_is_refused_naming_its_form() {
    Fixture::with("grund_config_version = 2\nproject_name = '''x'''\n")
        .refused(&refusal(2, "a multi-line literal string"));
}

#[test]
#[ignore = "red until #1103 names the literal-string form in parse_string's refusal; implement removes this"]
fn an_element_literal_string_is_refused_naming_its_form() {
    Fixture::with(&format!("{HEAD}\n[schema.kinds.X]\nfolders = ['specs']\n"))
        .refused(&refusal(10, "a literal string"));
}

#[test]
#[ignore = "red until #1103 names the literal-string form in parse_string's refusal; implement removes this"]
fn a_later_element_literal_string_is_refused_naming_its_form() {
    Fixture::with(&format!(
        "{HEAD}\n[schema.kinds.X]\nfolders = [\"docs\", 'specs']\n"
    ))
    .refused(&refusal(10, "a literal string"));
}

#[test]
fn version_1_keeps_the_bare_message_for_a_literal_string() {
    Fixture::with("grund_config_version = 1\n\n[scan]\nexclude = ['target']\n")
        .refused("error: grund.toml:4: expected string\n");
}

#[test]
fn a_value_malformed_otherwise_keeps_the_bare_message() {
    Fixture::with("grund_config_version = 2\nproject_name = true\n")
        .refused("error: grund.toml:2: expected string\n");
}
