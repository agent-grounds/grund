//! §FS-config-v2.rollout.fmt-values: recognized formatting keys refuse values
//! with actionable guidance while preserving the old message, location and status.
//! Dynamic assertions intentionally leave the new suffix's prose unfrozen.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

const HEAD: &str = "grund_config_version = 2\nproject_name = \"demo\"\n\n[presentation.fmt]\n";

struct Fixture(PathBuf);

impl Fixture {
    fn new(name: &str, settings: &str) -> Self {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join("presentation_fmt_values")
            .join(name);
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let init = Command::new("git")
            .args(["init", "-q"])
            .arg(&root)
            .output()
            .expect("initialize isolated fixture repository");
        assert!(init.status.success(), "{init:?}");
        fs::write(root.join("grund.toml"), format!("{HEAD}{settings}")).unwrap();
        Self(root)
    }

    fn run(&self, verb: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(["config", verb, "."])
            .current_dir(&self.0)
            .output()
            .expect("spawn grund")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn refusal(name: &str, key: &str, authored: &str, quoted_value: &str, executed: &str) {
    let fixture = Fixture::new(name, &format!("{key} = {authored}\n"));
    let output = fixture.run("validate");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(output.stdout.is_empty(), "refusal belongs on stderr");
    assert_eq!(stderr.lines().count(), 1, "one diagnostic line: {stderr:?}");
    assert!(
        !stderr.contains(['\r', '\t']),
        "control characters must be escaped: {stderr:?}"
    );
    let prefix = format!(
        "error: grund.toml:5: `[presentation.fmt] {key}` is part of the v2 format \
         but not supported by this grund yet"
    );
    let suffix = stderr.strip_prefix(&prefix).unwrap_or_else(|| {
        panic!("old refusal prefix and authored location must survive: {stderr}")
    });
    assert!(
        suffix.contains(quoted_value),
        "missing rejected value {quoted_value} after preserved prefix; got: {stderr}"
    );
    let assignment = format!("{key} = \"{executed}\"");
    assert!(
        suffix.contains(&assignment),
        "missing executed assignment {assignment} after preserved prefix; got: {stderr}"
    );
}

#[test]
fn links_off_names_value_and_executed_assignment() {
    refusal("links-off", "links", "\"off\"", "\"off\"", "index");
}

#[test]
fn anchors_none_names_value_and_executed_assignment() {
    refusal("anchors-none", "anchors", "\"none\"", "\"none\"", "github");
}

#[test]
fn links_all_names_value_and_executed_assignment() {
    refusal("links-all", "links", "\"all\"", "\"all\"", "index");
}

#[test]
fn anchors_gitlab_names_value_and_executed_assignment() {
    refusal(
        "anchors-gitlab",
        "anchors",
        "\"gitlab\"",
        "\"gitlab\"",
        "github",
    );
}

#[test]
fn links_escaped_value_stays_on_one_line() {
    let quoted = r#""quo\"te\\path\nnext\r\t""#;
    refusal("links-escaped", "links", quoted, quoted, "index");
}

#[test]
fn anchors_escaped_value_stays_on_one_line() {
    let quoted = r#""quo\"te\\path\nnext\r\t""#;
    refusal("anchors-escaped", "anchors", quoted, quoted, "github");
}

#[test]
fn explicit_and_omitted_defaults_validate_and_show_effective_settings() {
    for (name, settings) in [
        ("omitted", ""),
        ("explicit", "anchors = \"github\"\nlinks = \"index\"\n"),
    ] {
        let fixture = Fixture::new(name, settings);
        let validated = fixture.run("validate");
        assert_eq!(validated.status.code(), Some(0), "{validated:?}");
        assert!(validated.stdout.is_empty() && validated.stderr.is_empty());
        let shown = fixture.run("show");
        assert_eq!(shown.status.code(), Some(0), "{shown:?}");
        assert!(shown.stderr.is_empty(), "{shown:?}");
        let effective = String::from_utf8(shown.stdout).unwrap();
        assert!(
            effective.contains(
                "[presentation.fmt]\nanchors = \"github\"\nlinks = \"index\"\nexclude = []\n"
            ),
            "{effective}"
        );
    }
}

#[test]
fn malformed_and_non_string_values_keep_parse_errors() {
    for key in ["anchors", "links"] {
        for (index, (authored, message)) in [
            ("false", "expected string"),
            ("17", "expected string"),
            ("\"unterminated", "expected string"),
            (r#""bad\q""#, r#"invalid escape sequence `\q` in string"#),
        ]
        .iter()
        .enumerate()
        {
            let fixture = Fixture::new(
                &format!("parse-{key}-{index}"),
                &format!("{key} = {authored}\n"),
            );
            let output = fixture.run("validate");
            assert_eq!(output.status.code(), Some(1), "{output:?}");
            assert!(output.stdout.is_empty());
            assert_eq!(
                String::from_utf8(output.stderr).unwrap(),
                format!("error: grund.toml:5: {message}\n")
            );
        }
    }
}
