//! §AR-lsp.5.1.1 — `grund_core::lsp_snapshot` anchored below a workspace root and
//! outside every member narrows its report the way `grund_core::check` of the same
//! path does (§AR-resolver.3.3), while its navigation records stay the whole
//! workspace's. The fixture is agent-grounds/grund#858's, before any `fmt --write`:
//! a root with `include_root = false`, members `api` and `web` that each cite their
//! own unlocked section `format.1`, and an empty `tools/sub/` no member contains.
//! Its variant nests `api` at `apps/api`, so the snapshot of `apps/` keeps one
//! member's findings and drops the other's, and narrowing cannot pass by blanking.

use grund_core::{Finding, LspSnapshot, LspSnapshotOpts, Report, check, lsp_snapshot};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

const ROOT_CONFIG: &str = "grund_config_version = 1\n\n[reference]\nmarker = \"\u{a7}\"\n\
                           strict = true\n\n[id]\nformat = \"{kind}-{slug}\"\n\
                           slug_pattern = \"[a-z][a-z0-9-]*\"\n\n[workspace]\n";
const MEMBER_CONFIG: &str = "grund_config_version = 1\n\n[reference]\nmarker = \"\u{a7}\"\n\
                             strict = true\n\n[id]\nformat = \"{kind}-{slug}\"\n\
                             slug_pattern = \"[a-z][a-z0-9-]*\"\nnamed_sections = true\n\n\
                             [scan]\ninclude = [\"docs\", \"src\"]\n";

static NEXT: AtomicUsize = AtomicUsize::new(0);

/// grund#858's workspace with its members at `members`, each named for its last
/// component, and an empty `tools/sub/`. No section lock is written, so every
/// citation of `format.1` is an error `check` from the root reports.
fn workspace(what: &str, members: &[&str]) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "grund-lsp-outside-member-{what}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("tools/sub")).expect("create tools/sub");
    let write = |path: &str, body: &str| {
        let path = root.join(path);
        fs::create_dir_all(path.parent().expect("parent")).expect("create fixture dir");
        fs::write(path, body).expect("write fixture file");
    };
    let quoted: Vec<String> = members.iter().map(|m| format!("\"{m}\"")).collect();
    write(
        "grund.toml",
        &format!(
            "{ROOT_CONFIG}members = [{}]\ninclude_root = false\n",
            quoted.join(", ")
        ),
    );
    for member in members {
        let name = member.rsplit('/').next().expect("member name");
        write(
            &format!("{member}/grund.toml"),
            &format!("project_name = \"{name}\"\n{MEMBER_CONFIG}"),
        );
        write(
            &format!("{member}/docs/FS-{name}.md"),
            &format!(
                "# FS-{name}: {name}\n\n## format: Formatting\n\n### .1: Width\n\nWrap at 80.\n"
            ),
        );
        write(
            &format!("{member}/docs/FS-{name}-audit.md"),
            &format!("# FS-{name}-audit: Audit\n\nCites \u{a7}FS-{name}.format.1.\n"),
        );
        write(
            &format!("{member}/src/lib.rs"),
            &format!("/// Wraps (\u{a7}FS-{name}.format.1).\npub fn wrap() {{}}\n"),
        );
    }
    root.canonicalize().expect("canonical workspace root")
}

fn snapshot(path: PathBuf) -> LspSnapshot {
    lsp_snapshot(LspSnapshotOpts {
        path,
        path_provided: true,
        ..Default::default()
    })
    .expect("lsp_snapshot")
}

/// One diagnostic as `severity code root-relative-path:line message`, so `check`'s
/// display paths and the snapshot's absolute paths compare as the same finding. The
/// version 1 caution is a `check` surface and never the snapshot's
/// (§FS-check.checks.config-version-1), so it is left out of the comparison.
fn diagnostics(root: &Path, report: &Report) -> Vec<String> {
    let key = |finding: &Finding| {
        let path = finding.path.as_deref().map(|path| {
            let path = Path::new(path);
            let absolute = if path.is_absolute() {
                path.to_path_buf()
            } else {
                std::env::current_dir().expect("cwd").join(path)
            };
            let absolute = absolute.canonicalize().unwrap_or(absolute);
            absolute
                .strip_prefix(root)
                .map(|relative| relative.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| path.to_string_lossy().into_owned())
        });
        format!(
            "{} {} {}:{} {}",
            finding.severity,
            finding.code,
            path.unwrap_or_default(),
            finding.line.unwrap_or_default(),
            finding.message
        )
    };
    let mut keys: Vec<String> = report
        .errors
        .iter()
        .chain(&report.warnings)
        .filter(|finding| finding.code != "config-version-1")
        .map(key)
        .collect();
    keys.sort();
    keys
}

/// Whether any diagnostic is anchored in `file`, a root-relative path.
fn anchored_in(diagnostics: &[String], file: &str) -> bool {
    diagnostics.iter().any(|line| {
        line.split(' ')
            .nth(2)
            .is_some_and(|at| at.starts_with(&format!("{file}:")))
    })
}

/// From a folder no member contains and nothing is scanned under, the snapshot reports
/// what `check(tools/sub)` reports: none of the members' errors.
#[test]
fn a_snapshot_outside_every_member_reports_what_check_reports() {
    let root = workspace("issue", &["api", "web"]);
    let whole = diagnostics(&root, &check(&root).expect("check(.)"));
    assert!(
        anchored_in(&whole, "api/src/lib.rs") && anchored_in(&whole, "web/src/lib.rs"),
        "control: check(.) reports both members' unlocked citations: {whole:#?}"
    );

    let scoped = diagnostics(
        &root,
        &check(&root.join("tools/sub")).expect("check(tools/sub)"),
    );
    let snapped = diagnostics(&root, &snapshot(root.join("tools/sub")).report);
    assert!(
        !scoped.iter().any(|line| line.starts_with("error ")),
        "control: check(tools/sub) reports no error: {scoped:#?}"
    );
    assert!(
        !snapped.iter().any(|line| line.starts_with("error ")),
        "lsp_snapshot(tools/sub) reports the members' errors outside the anchor: {snapped:#?}"
    );
    assert_eq!(
        snapped, scoped,
        "lsp_snapshot(tools/sub) must report what check(tools/sub) reports"
    );
}

/// The navigation records are the workspace's, not the anchor's: a narrowed report
/// leaves every member's declarations in the snapshot.
#[test]
fn a_snapshot_outside_every_member_carries_every_members_records() {
    let root = workspace("records", &["api", "web"]);
    let snapshot = snapshot(root.join("tools/sub"));
    let mut files: Vec<String> = snapshot
        .declarations
        .iter()
        .map(|declaration| {
            let path = declaration.path.canonicalize().expect("declaration path");
            let relative = path.strip_prefix(&root).expect("declared in the workspace");
            relative.to_string_lossy().replace('\\', "/")
        })
        .collect();
    files.sort();
    files.dedup();
    assert_eq!(
        files,
        [
            "api/docs/FS-api-audit.md",
            "api/docs/FS-api.md",
            "web/docs/FS-web-audit.md",
            "web/docs/FS-web.md",
        ],
        "the files the snapshot's declarations sit in"
    );
}

/// Positive control: `apps/` holds `apps/api` alone, so the snapshot of `apps` keeps
/// `api`'s errors and drops `web`'s, as `check(apps)` does.
#[test]
fn a_snapshot_outside_every_member_keeps_the_members_under_it() {
    let root = workspace("nested", &["apps/api", "web"]);
    let scoped = diagnostics(&root, &check(&root.join("apps")).expect("check(apps)"));
    let snapped = diagnostics(&root, &snapshot(root.join("apps")).report);
    for file in ["apps/api/docs/FS-api-audit.md", "apps/api/src/lib.rs"] {
        assert!(
            anchored_in(&scoped, file),
            "control: check(apps) reports {file}: {scoped:#?}"
        );
        assert!(
            anchored_in(&snapped, file),
            "lsp_snapshot(apps) drops {file}, inside the anchor: {snapped:#?}"
        );
    }
    for file in ["web/docs/FS-web-audit.md", "web/src/lib.rs"] {
        assert!(
            !anchored_in(&snapped, file),
            "lsp_snapshot(apps) reports {file}, outside the anchor: {snapped:#?}"
        );
    }
    assert_eq!(
        snapped, scoped,
        "lsp_snapshot(apps) must report what check(apps) reports"
    );
}

/// §FS-check.4.9.4: the announcement of an absent optional member is located at the
/// root's `grund.toml`, so an anchor outside it drops it as `check(tools/sub)` does,
/// and the narrowed report, silent, earns the one empty-scan caution of the anchor.
#[test]
fn a_snapshot_outside_every_member_narrows_the_absent_member_announcement() {
    let root = workspace("absent", &["api", "web"]);
    let config = fs::read_to_string(root.join("grund.toml")).expect("read grund.toml");
    fs::write(
        root.join("grund.toml"),
        format!("{config}optional_members = [\"gone\"]\n"),
    )
    .expect("write grund.toml");
    let whole = diagnostics(&root, &check(&root).expect("check(.)"));
    assert!(
        anchored_in(&whole, "grund.toml"),
        "control: check(.) announces the absent member: {whole:#?}"
    );

    let scoped = diagnostics(
        &root,
        &check(&root.join("tools/sub")).expect("check(tools/sub)"),
    );
    let snapped = diagnostics(&root, &snapshot(root.join("tools/sub")).report);
    assert!(
        !anchored_in(&snapped, "grund.toml"),
        "lsp_snapshot(tools/sub) announces the absent member outside the anchor: {snapped:#?}"
    );
    assert_eq!(
        snapped, scoped,
        "lsp_snapshot(tools/sub) must report what check(tools/sub) reports"
    );
}
