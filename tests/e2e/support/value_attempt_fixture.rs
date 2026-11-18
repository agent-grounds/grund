//! Isolated command fixtures for §FS-values.8 and §FS-values.3.1.1.

use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub const CONFIG: &str = "grund_config_version = 2\n\
    [schema.kinds.CONST]\nfolders = [\"values\"]\nform = \"value\"\n\
    [schema.kinds.DOC]\nfolders = [\"docs\"]\n";
pub const VALUE: &str = "# CONST-field-price: Field price\n## 1. 1200\n## 2. USD\n";
pub const FORM: &str = "value binding must be exactly `literal` (marker-prefixed full value ID with one positive numeric field)";
pub const SPLIT: &str =
    "value binding must be on one physical line; join the literal and its citation";

pub struct Fixture(pub PathBuf);

impl Fixture {
    pub fn new(name: &str) -> Self {
        let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join("fmt_value_attempts")
            .join(format!("{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        let init = Command::new("git")
            .args(["init", "-q"])
            .arg(&path)
            .output()
            .unwrap();
        assert!(init.status.success(), "{init:?}");
        Self(path)
    }

    pub fn project(&self, config: &str, value: &str, body: &str) {
        self.write("grund.toml", config);
        self.write("values/field-price.md", value);
        // Linked indexes avoid unrelated errors masking an erased binding refusal.
        self.write(
            "values/README.md",
            "# Values\n\n- [§CONST-field-price](field-price.md#const-field-price-field-price)\n",
        );
        self.write(
            "docs/README.md",
            "# Docs\n\n- [§DOC-offer](offer.md#doc-offer-offer)\n",
        );
        self.write("docs/offer.md", &format!("# DOC-offer: Offer\n\n{body}\n"));
    }

    pub fn write(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    pub fn read(&self, path: &str) -> String {
        fs::read_to_string(self.0.join(path)).unwrap()
    }

    pub fn run(&self, args: &[&str]) -> (i32, String) {
        let out = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .env("NO_COLOR", "1")
            .output()
            .unwrap();
        assert!(
            String::from_utf8_lossy(&out.stderr)
                .lines()
                .all(|line| line
                    .contains("is version 1; migrate it with grund config migrate --to 2")),
            "{args:?}: {out:?}"
        );
        (
            out.status.code().unwrap(),
            String::from_utf8(out.stdout).unwrap(),
        )
    }

    pub fn check(&self, code: Option<&str>, message: Option<&str>) -> Vec<Value> {
        let (status, text) = self.run(&["check", "--format", "json"]);
        let errors: Vec<Value> = text
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).expect("NDJSON finding"))
            .filter(|row| row["severity"] == "error")
            .collect();
        assert_eq!(status, i32::from(code.is_some()), "{text}");
        assert_eq!(errors.len(), usize::from(code.is_some()), "{text}");
        if let Some(code) = code {
            assert_eq!(errors[0]["code"], code, "{text}");
        }
        if let Some(message) = message {
            assert_eq!(
                errors[0]["message"],
                format!("invalid value binding: {message}"),
                "{text}"
            );
        }
        errors
    }

    /// Run the reported sequence before checking byte/preview invariants, so a
    /// failure records both the lost diagnostic and the destructive rewrite.
    pub fn preserves_refusal(&self, path: &str, expected: &str, message: Option<&str>) {
        let before = self.check(Some("invalid-value-binding"), message);
        let original = self.read(path);
        let dry = self.run(&["fmt"]);
        assert_eq!(self.read(path), original, "dry run wrote the fixture");
        let preview = self.run(&["fmt", "--check"]);
        assert_eq!(self.read(path), original, "--check wrote the fixture");
        assert_eq!(self.run(&["fmt", "--write"]).0, 0);
        let written = self.read(path);
        let after = self.run(&["check", "--format", "json"]);
        let after_errors: Vec<Value> = after
            .1
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .filter(|row| row["severity"] == "error")
            .collect();
        assert_eq!(self.run(&["fmt", "--write"]).0, 0);
        assert_eq!(self.read(path), written, "second write must be stable");
        assert!(
            after.0 == 1 && after_errors == before && written == expected,
            "refusal must survive default fmt --write with citation bytes intact\n\
             before errors: {before:?}\nafter status: {}\nafter errors: {after_errors:?}\n\
             expected file: {expected:?}\nactual file: {written:?}",
            after.0
        );
        // Pure refusals propose no links in any formatter mode. The case with
        // an ordinary occurrence still has a real link to propose.
        if original == expected {
            assert_eq!(preview.0, 0, "{preview:?}");
            assert!(!dry.1.contains(path), "{dry:?}");
        } else {
            assert_eq!(preview.0, 1, "{preview:?}");
            assert!(dry.1.contains(path), "{dry:?}");
        }
        assert_eq!(self.run(&["fmt", "--check"]).0, 0);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
