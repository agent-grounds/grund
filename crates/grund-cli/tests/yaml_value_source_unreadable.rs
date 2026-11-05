//! Black-box: a value source the scan cannot open is an incomplete scan at exit `2` from every
//! reader of the catalog, whichever its format (§FS-values.2.2.4, §FS-values.5.3). Git carries no
//! mode a fixture could keep unreadable, so this is the one row of that table's first line the
//! e2e corpus cannot hold; its malformed UTF-8 and its parse failure are cases there.
//! The contextual I/O reason is shared with ordinary reads (§FS-check.2.4.1).
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const CONFIG: &str = "grund_config_version = 2\nproject_name = \"vals\"\n\n\
    [schema.kinds.CONST]\nfolders = [\"values\"]\nform = \"value\"\n\n\
    [schema.kinds.page]\ncitable = false\nfiles = [\"docs/offer.md\"]\n";

/// A value home holding one good JSON source and one more, `limits.<extension>`, nobody may open.
struct Fixture {
    root: PathBuf,
    source: PathBuf,
}

impl Fixture {
    fn new(extension: &str, contents: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-1133-unreadable-{extension}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("values")).expect("create the value home");
        fs::create_dir_all(root.join("docs")).expect("create the docs");
        fs::write(root.join("grund.toml"), CONFIG).expect("write the config");
        fs::write(
            root.join("values/prices.json"),
            "{\n  \"CONST-field-price\": [1200, \"USD\"]\n}\n",
        )
        .expect("write the good source");
        fs::write(
            root.join("values/README.md"),
            "- [§CONST-field-price](prices.json)\n",
        )
        .expect("write the index");
        fs::write(
            root.join("docs/offer.md"),
            "# Offer\n\nThe field price is `1200` (§CONST-field-price.1).\n",
        )
        .expect("write the citer");
        let source = root.join(format!("values/limits.{extension}"));
        fs::write(&source, contents).expect("write the source");
        fs::set_permissions(&source, fs::Permissions::from_mode(0o000)).expect("take its mode");
        Self { root, source }
    }

    /// Whether the mode bars this process: it does not bar root, which proves nothing here.
    fn is_unreadable(&self) -> bool {
        fs::read(&self.source).is_err()
    }

    fn ask(&self, args: &[&str]) -> (i32, String) {
        let output = run(&self.root, args);
        let code = output.status.code().expect("exited with a status");
        (code, String::from_utf8_lossy(&output.stderr).into_owned())
    }

    /// The existing parity tests compare every reader with check; the new reason
    /// regressions below pin its bytes independently, without preserving Rust's spelling.
    fn read_failure(&self, format: &str) -> String {
        let (code, stderr) = self.ask(&["check"]);
        assert_eq!(code, 2, "{stderr}");
        let line = stderr.trim().strip_prefix("error: ").unwrap();
        let line = line
            .strip_suffix("; make the path readable, or keep it out of the scan")
            .unwrap();
        let path = self.source.strip_prefix(&self.root).unwrap().display();
        assert!(line.starts_with(&format!("{path}: could not read value {format} source: ")));
        line.to_string()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::set_permissions(&self.source, fs::Permissions::from_mode(0o644));
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn run(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(root)
        .output()
        .expect("run grund")
}

/// Every reader of the catalog exits `2` on the source's own line; `id` names no remedy.
fn assert_incomplete_for_every_reader(fixture: &Fixture, line: &str) {
    let scan = format!("error: {line}; make the path readable, or keep it out of the scan\n");
    for args in [
        &["check"][..],
        &["list"],
        &["show", "CONST-field-price"],
        &["show", "CONST-room-limit"],
        &["refs", "CONST-field-price"],
        &["cover"],
    ] {
        assert_eq!(fixture.ask(args), (2, scan.clone()), "grund {args:?}");
    }
    let id = format!("error: {line}\n");
    assert_eq!(fixture.ask(&["id", "CONST", "title"]), (2, id), "grund id");
}

/// The control: what the YAML source below is held to.
#[test]
fn an_unreadable_json_source_is_an_incomplete_scan_for_every_reader() {
    let fixture = Fixture::new("json", "{\n  \"CONST-room-limit\": [40]\n}\n");
    if !fixture.is_unreadable() {
        eprintln!("skipped: the mode does not bar this user, so nothing here can be unreadable");
        return;
    }
    assert_incomplete_for_every_reader(&fixture, &fixture.read_failure("JSON"));
}

#[test]
fn an_unreadable_yaml_source_is_an_incomplete_scan_for_every_reader() {
    let fixture = Fixture::new("yaml", "CONST-room-limit: [40]\n");
    if !fixture.is_unreadable() {
        eprintln!("skipped: the mode does not bar this user, so nothing here can be unreadable");
        return;
    }
    assert_incomplete_for_every_reader(&fixture, &fixture.read_failure("YAML"));
}

#[test]
fn json_value_source_uses_the_shared_permission_reason() {
    let fixture = Fixture::new("json-reason.json", "{\"CONST-room-limit\": [40]}\n");
    assert!(fixture.is_unreadable(), "run as an unprivileged user");
    assert_incomplete_for_every_reader(
        &fixture,
        "values/limits.json-reason.json: could not read value JSON source: permission denied",
    );
    assert_json_reason(&fixture, "values/limits.json-reason.json", "JSON");
}

#[test]
fn yaml_value_source_uses_the_shared_permission_reason() {
    let fixture = Fixture::new("yaml-reason.yaml", "CONST-room-limit: [40]\n");
    assert!(fixture.is_unreadable(), "run as an unprivileged user");
    assert_incomplete_for_every_reader(
        &fixture,
        "values/limits.yaml-reason.yaml: could not read value YAML source: permission denied",
    );
    assert_json_reason(&fixture, "values/limits.yaml-reason.yaml", "YAML");
}

fn assert_json_reason(fixture: &Fixture, path: &str, format: &str) {
    let (code, stderr) = fixture.ask(&["check", "--format", "json"]);
    assert_eq!(code, 2, "{stderr}");
    let records: Vec<serde_json::Value> = stderr
        .lines()
        .map(|line| serde_json::from_str(line).expect("NDJSON record"))
        .collect();
    let record = records
        .iter()
        .find(|record| record["path"] == path && record["code"] == "io")
        .unwrap_or_else(|| panic!("missing value-source I/O failure:\n{stderr}"));
    assert_eq!(
        record["message"],
        format!(
            "could not read value {format} source: permission denied; make the path readable, or keep it out of the scan"
        ),
        "{stderr}"
    );
}
