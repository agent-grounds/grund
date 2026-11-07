//! §AR-lsp.5.1.1, §AR-scanner.6.6: a snapshot anchored at a path partitions a failure reading an
//! E2E case as `grund check` of that path does. The partition is the one `check` asks, of the
//! failure's location (§FS-check.1.3.6.3), so a failure located at the cases root made every anchor
//! below that root call it outside, and the location at the file that failed is the whole repair.

#[path = "binaries.rs"]
mod binaries;

use grund_core::{LspSnapshot, LspSnapshotOpts, lsp_snapshot};
use std::fs;
use std::path::{Path, PathBuf};

const CONFIG: &str = "grund_config_version = 2\nproject_name = \"p\"\n\n\
    [schema]\nslug_pattern = \"[a-z][a-z-]*\"\n\n\
    [schema.kinds.E2E]\nfolders = [\"e2e/cases\"]\n";

const FILE: &str = "e2e/cases/b/expected.exit";

/// Cases `a` and `c` healthy, and `b` holding bytes that do not decode in `expected.exit`.
fn fixture(name: &str) -> PathBuf {
    let root = binaries::repo_root()
        .join("target/lsp-snapshot-e2e-case-failure-work")
        .join(format!("{name}-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove stale fixture");
    }
    for case in ["a", "b", "c"] {
        fs::create_dir_all(root.join("e2e/cases").join(case)).expect("create the case");
        fs::write(
            root.join("e2e/cases").join(case).join("expected.exit"),
            "0\n",
        )
        .expect("write the manifest");
    }
    fs::write(root.join(FILE), b"\xff\xfe\n").expect("break the manifest");
    fs::write(root.join("grund.toml"), CONFIG).expect("write the config");
    root.canonicalize().expect("canonical fixture root")
}

fn snapshot(path: PathBuf) -> LspSnapshot {
    lsp_snapshot(LspSnapshotOpts {
        path,
        path_provided: true,
        ..Default::default()
    })
    .expect("lsp_snapshot")
}

/// Every `io` diagnostic in `findings` as its path and message, whatever base it is spelled from.
fn io_at<'a>(findings: &'a [grund_core::Finding], root: &Path) -> Vec<(String, &'a str)> {
    findings
        .iter()
        .filter(|finding| finding.code == "io")
        .map(|finding| {
            let path = finding.path.as_deref().unwrap_or_default();
            let relative = Path::new(path)
                .strip_prefix(root)
                .unwrap_or(Path::new(path));
            (
                relative.to_string_lossy().replace('\\', "/"),
                finding.message.as_str(),
            )
        })
        .collect()
}

#[test]
fn an_anchor_at_the_failing_case_or_file_holds_the_failure() {
    let root = fixture("holds");
    for anchor in ["e2e/cases/b", FILE] {
        let report = snapshot(root.join(anchor)).report;
        let errors = io_at(&report.errors, &root);
        assert_eq!(errors.len(), 1, "{anchor}: {report:#?}");
        assert!(errors[0].0.ends_with(FILE), "{anchor}: {report:#?}");
        assert!(
            io_at(&report.warnings, &root).is_empty(),
            "{anchor} holds the failure, so nothing says it lies outside: {report:#?}"
        );
    }
}

#[test]
fn an_anchor_at_a_healthy_sibling_cautions_at_the_failing_file() {
    let root = fixture("sibling");
    let report = snapshot(root.join("e2e/cases/c")).report;
    assert!(io_at(&report.errors, &root).is_empty(), "{report:#?}");
    let cautions = io_at(&report.warnings, &root);
    assert_eq!(cautions.len(), 1, "{report:#?}");
    assert!(cautions[0].0.ends_with(FILE), "{report:#?}");
    assert!(
        cautions[0].1.contains("outside the report scope"),
        "{report:#?}"
    );
}
