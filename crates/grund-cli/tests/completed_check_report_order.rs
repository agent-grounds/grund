//! Completed embedding reports obey §FS-errors.4.1 independently of workspace
//! dispatch. CLI cautions retain their separately rendered order.

use grund_core::{EmbeddingRequest, embedding_call};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct Repo(PathBuf);

impl Repo {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-952-{name}-{}", std::process::id()));
        assert!(!root.exists(), "fixture must be fresh: {}", root.display());
        fs::create_dir_all(&root).unwrap();
        assert!(
            Command::new("git")
                .args(["init", "--quiet"])
                .arg(&root)
                .status()
                .unwrap()
                .success()
        );
        Self(root)
    }

    fn write(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn check(root: &Path) -> Value {
    let envelope = embedding_call(EmbeddingRequest {
        operation: "check".into(),
        root: root.to_path_buf(),
        explicit: true,
        args: Vec::new(),
        options: json!({}),
    });
    assert_eq!(envelope["failure"], Value::Null, "{envelope}");
    assert_eq!(envelope["result"]["had_scan_errors"], false, "{envelope}");
    envelope["result"]["report"].clone()
}

fn warning(code: &str, path: Value, line: Value, message: &str) -> Value {
    json!({"severity":"warning", "code":code, "path":path, "line":line,
        "channel":null, "column":null, "message":message, "sites":[], "authority":[]})
}

fn codes(report: &Value) -> Vec<&str> {
    report["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| finding["code"].as_str().unwrap())
        .collect()
}

/// Port of grund.142's minimal same-tree reproducer through the bindings' entrypoint.
#[test]
#[ignore = "red until #952 completes report ordering; implement removes this"]
fn completed_report_order_single_project_matches_empty_workspace() {
    let repo = Repo::new("single-workspace");
    let config = concat!(
        "grund_config_version = 1\nproject_name = \"solo\"\n",
        "[id]\nformat = \"{kind}-{slug}\"\n",
        "[[kinds]]\nkind = \"FS\"\nfile = \"docs/fs.md\"\n"
    );
    repo.write("grund.toml", config);
    repo.write("docs/fs.md", "# FS-a: Alpha\n");
    let single = check(&repo.0);
    repo.write(
        "grund.toml",
        &format!("{config}\n[workspace]\nmembers = []\n"),
    );
    let workspace = check(&repo.0);
    let expected = json!({"errors":[], "suggestions":[], "warnings":[
        warning("config-version-1", Value::Null, Value::Null,
            "grund.toml is version 1; migrate it with grund config migrate --to 2"),
        warning("unused", json!("docs/fs.md"), json!(1), "declared but never cited: FS-a")
    ]});
    // Check complete contents before asserting raw order: setup drift is a different failure.
    for (arm, report) in [("single", &single), ("workspace", &workspace)] {
        assert_eq!(report["errors"], expected["errors"], "{arm}");
        assert_eq!(report["suggestions"], expected["suggestions"], "{arm}");
        let findings = report["warnings"].as_array().unwrap();
        let wanted = expected["warnings"].as_array().unwrap();
        assert_eq!(findings.len(), wanted.len(), "{arm}: {report}");
        assert!(
            wanted.iter().all(|finding| findings.contains(finding)),
            "{arm}: {report}"
        );
    }
    assert_eq!(
        codes(&workspace),
        ["config-version-1", "unused"],
        "workspace raw order"
    );
    assert_eq!(
        codes(&single),
        ["config-version-1", "unused"],
        "single-project raw order"
    );
    assert_eq!(single, expected);
    assert_eq!(workspace, expected);
}

/// The relative-paths-false shape: an empty path scope earns two pathless cautions.
#[test]
#[ignore = "red until #952 completes report ordering; implement removes this"]
fn completed_report_order_two_pathless_cautions() {
    let repo = Repo::new("pathless");
    repo.write(
        "grund.toml",
        concat!(
            "grund_config_version = 1\n",
            "[reference]\nstrict = false\n[output]\nrelative_paths = false\n"
        ),
    );
    fs::create_dir_all(repo.0.join("sub")).unwrap();
    let scope = repo.0.join("sub");
    let report = check(&scope);
    assert_eq!(report["errors"], json!([]));
    assert_eq!(report["suggestions"], json!([]));
    let warnings = report["warnings"].as_array().unwrap();
    assert_eq!(warnings.len(), 2, "{report}");
    assert!(
        warnings.iter().all(|finding| finding["path"].is_null()
            && finding["line"].is_null()
            && finding["column"].is_null()
            && finding["channel"].is_null()
            && finding["severity"] == "warning"
            && finding["sites"] == json!([])
            && finding["authority"] == json!([])),
        "{report}"
    );
    let cli = Command::new(env!("CARGO_BIN_EXE_grund"))
        .arg("check")
        .arg(&scope)
        .current_dir(&scope)
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    assert!(cli.status.success(), "{cli:?}");
    assert!(cli.stdout.is_empty(), "{cli:?}");
    let message = |code: &str| {
        warnings
            .iter()
            .find(|finding| finding["code"] == code)
            .unwrap()["message"]
            .as_str()
            .unwrap()
    };
    assert_eq!(
        message("config-version-1"),
        "../grund.toml is version 1; migrate it with grund config migrate --to 2 .."
    );
    assert!(message("empty-scan").starts_with("nothing to scan — no files under `.` matched"));
    assert_eq!(
        String::from_utf8(cli.stderr).unwrap(),
        format!(
            "warning: {}\nwarning: {}\n",
            message("config-version-1"),
            message("empty-scan")
        ),
        "CLI caution order"
    );
    assert_eq!(
        codes(&report),
        ["config-version-1", "empty-scan"],
        "raw message order"
    );
}
