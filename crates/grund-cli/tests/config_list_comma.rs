//! Black-box: a `grund.toml` list element ends only at a comma outside a string, so a comma
//! inside a basic string is part of the element and is read as written (§FS-config.3.11), in
//! a version-2 list on one line or across lines (§FS-config-v2.reader.5) and in a version-1
//! list (§FS-config.3). Ports #1076's reproducer. Nothing that was refused is now accepted: a
//! version-1 trailing comma is still `expected string` at its line (§FS-config.4.3).

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const SPEC: &str = "# SPEC-one: One\n\nLead.\n";
const CODE: &str = "// Realizes \u{a7}SPEC-one.\nfn main() {}\n";
/// A dangling citation: it fails `check` wherever the directory holding it is scanned.
const STRAY: &str = "// Cites \u{a7}SPEC-gone.\n";
const STRAY_FOUND: &str = "src/a,b/stray.rs:1: error: unknown reference SPEC-gone; did you mean \
                           SPEC-one?\n";
const V1_CAUTION: &str =
    "warning: grund.toml is version 1; migrate it with grund config migrate --to 2\n";
const V1_HEAD: &str = "grund_config_version = 1\n\n[id]\nformat = \"{kind}-{slug}\"\n\n";
const V1_KINDS: &str = "\n[[kinds]]\nkind = \"SPEC\"\nfolder = \"specs\"\nindex = false\n";

struct Fixture(PathBuf);

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Fixture {
    fn with(files: &[(&str, &str)]) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("grund-1076-{}-{serial}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        for (path, contents) in files {
            let path = dir.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, contents).unwrap();
        }
        Self(dir)
    }

    /// `specs/a,b/spec.md` declares `SPEC-one`, which `src/main.rs` cites.
    fn comma_spec(config: &str) -> Self {
        Self::with(&[
            ("grund.toml", config),
            ("specs/a,b/spec.md", SPEC),
            ("src/main.rs", CODE),
        ])
    }

    /// A clean tree but for `src/a,b/stray.rs`, whose citation dangles.
    fn comma_stray(config: &str) -> Self {
        Self::with(&[
            ("grund.toml", config),
            ("specs/spec.md", SPEC),
            ("src/main.rs", CODE),
            ("src/a,b/stray.rs", STRAY),
        ])
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
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn v2(spec_files: &str, exclude: &str) -> String {
    format!(
        "grund_config_version = 2\n\n[schema.kinds.SPEC]\nfiles = {spec_files}\n\n\
         [schema.kinds.code]\nfolders = [\"src\"]\n\n[schema.sources]\nexclude = {exclude}\n"
    )
}

/// The issue's first case: a v2 place holding a comma names that file.
#[test]
fn a_v2_files_element_holding_a_comma_names_that_file() {
    let fx = Fixture::comma_spec(&v2("[\"specs/a,b/spec.md\"]", "[]"));
    fx.expect(&["check"], 0, "success\n", "");
    fx.expect(&["list"], 0, "SPEC-one  specs/a,b/spec.md:1  One\n", "");
}

/// The issue's second case: a v2 exclude entry holding a comma keeps that directory out.
#[test]
fn a_v2_exclude_element_holding_a_comma_keeps_that_directory_out() {
    let config = v2("[\"specs/spec.md\"]", "[\"ab\", \"a,b\"]");
    Fixture::comma_stray(&config).expect(&["check"], 0, "success\n", "");
}

/// Guard, green today: the comma-named directory is scanned when nothing excludes it, so the
/// exclude cases above pass only by keeping it out.
#[test]
fn the_comma_named_directory_is_scanned_when_nothing_excludes_it() {
    let fx = Fixture::comma_stray(&v2("[\"specs/spec.md\"]", "[\"ab\"]"));
    fx.expect(&["check"], 1, STRAY_FOUND, "");
    let v1 = format!("{V1_HEAD}[scan]\ninclude = [\"specs\", \"src\"]\n{V1_KINDS}");
    Fixture::comma_stray(&v1).expect(&["check"], 1, STRAY_FOUND, V1_CAUTION);
}

/// The same two lists written across lines, each with TOML's trailing comma (§FS-config-v2.reader.5).
#[test]
fn a_v2_list_across_lines_reads_a_comma_inside_a_string() {
    let fx = Fixture::comma_spec(&v2("[\n  \"specs/a,b/spec.md\",\n]", "[]"));
    fx.expect(&["check"], 0, "success\n", "");
    fx.expect(&["list"], 0, "SPEC-one  specs/a,b/spec.md:1  One\n", "");
    let config = v2("[\"specs/spec.md\"]", "[\n  \"ab\",\n  \"a,b\",\n]");
    Fixture::comma_stray(&config).expect(&["check"], 0, "success\n", "");
}

/// A v1 list reads the same element the same way: `[scan] exclude` keeps `a,b` out.
#[test]
fn a_v1_exclude_element_holding_a_comma_keeps_that_directory_out() {
    let config =
        format!("{V1_HEAD}[scan]\ninclude = [\"specs\", \"src\"]\nexclude = [\"a,b\"]\n{V1_KINDS}");
    Fixture::comma_stray(&config).expect(&["check"], 0, "", V1_CAUTION);
}

/// Guard, green today and kept: a v1 list ending in a trailing comma, or holding an unclosed
/// string, is still refused at its line as `expected string`.
#[test]
fn a_v1_trailing_comma_or_unclosed_string_is_still_expected_string() {
    for exclude in ["[\"ab\",]", "[\"a,b]"] {
        let config = format!(
            "{V1_HEAD}[scan]\ninclude = [\"specs\", \"src\"]\nexclude = {exclude}\n{V1_KINDS}"
        );
        let refusal = "error: grund.toml:8: expected string\n";
        Fixture::comma_stray(&config).expect(&["check"], 2, "", refusal);
    }
}

/// The closed `lead_size_warning` table separates its members by the same rule, so a comma
/// inside `unit` is part of the unit it names (§FS-config.3.1.2).
#[test]
fn a_comma_inside_the_lead_size_warning_unit_is_part_of_that_unit() {
    let config = format!(
        "{V1_HEAD}[reference]\nlead_size_warning = {{ max = 40, unit = \"lines,words\" }}\n\n\
         [scan]\ninclude = [\"specs\", \"src\"]\n{V1_KINDS}"
    );
    let refusal = "error: grund.toml:7: unknown lead_size_warning unit `lines,words` (expected \
                   lines, words, or bytes)\n";
    Fixture::comma_spec(&config).expect(&["check"], 2, "", refusal);
}

/// A brace pattern always holds a comma. What the place then matches is #1069's; this pins only
/// that the line is no longer refused as a string that is not one.
#[test]
fn a_brace_pattern_element_is_not_refused_as_expected_string() {
    let fx = Fixture::comma_spec(&v2("[\"specs/{001-a,002-b}/spec.md\"]", "[]"));
    let run = fx.run(&["check"]);
    assert!(
        !run.stderr.contains("expected string"),
        "exit {:?}, stderr:\n{}",
        run.code,
        run.stderr
    );
}
