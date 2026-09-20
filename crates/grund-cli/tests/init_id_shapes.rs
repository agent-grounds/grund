//! Black-box coverage for the effective ID shapes in `init --docs` scaffolds.
//! §FS-init.2.1.3: every scaffold-owned example follows the illustrated kind's
//! override or the repository fallback, on fresh creation and forced refresh.

use std::fs;
use std::path::Path;

#[path = "support/init_fixture.rs"]
mod init_fixture;

use init_fixture::{manifest_dir, run_grund, workdir};

fn assert_example(root: &Path, relative: &str, expected: &str) {
    let path = root.join(relative);
    let contents = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read expected scaffold {}: {error}", path.display()));
    assert!(
        contents.contains(expected),
        "{relative} should contain configured ID example `{expected}`, got:\n{contents}"
    );
}

fn replace_repo_format(root: &Path, replacement: &str) {
    let path = root.join("grund.toml");
    let before = fs::read_to_string(&path).expect("read generated grund.toml");
    let needle = "format = \"{kind}-{number}-{slug}\"";
    assert!(
        before.contains(needle),
        "generated config should contain the default ID format"
    );
    fs::write(&path, before.replacen(needle, replacement, 1)).expect("write configured format");
}

fn assert_slug_only_scaffold(root: &Path) {
    for (relative, expected) in [
        ("docs/grund.md", "# GRUND-<slug>: …"),
        ("docs/goals.md", "# GOAL-<slug>: …"),
        ("requirements.md", "## FS-<slug>: …"),
        ("docs/architecture/README.md", "`AR-<slug>` ID"),
        ("docs/architecture/README.md", "`§AR-<slug>.<section>`"),
        (
            "docs/architecture/README.md",
            "`# AR-<slug>: [<path>](<path>)`",
        ),
        ("docs/decisions/architectural/README.md", "`DA-<slug>` ID"),
        ("docs/decisions/functional/README.md", "`DF-<slug>` ID"),
    ] {
        assert_example(root, relative, expected);
    }
}

#[test]
fn init_docs_force_refreshes_every_example_to_the_repository_format() {
    let target = workdir("init_docs_force_refreshes_every_example_to_the_repository_format");
    let first = run_grund(
        &["init", target.to_str().unwrap(), "--docs"],
        manifest_dir(),
    );
    assert!(
        first.status.success(),
        "initial init --docs failed: {}",
        String::from_utf8_lossy(&first.stderr)
    );

    // The default remains the numbered-with-slug teaching shape.
    assert_example(&target, "docs/grund.md", "GRUND-NNN-slug");
    assert_example(
        &target,
        "docs/architecture/README.md",
        "§AR-NNN-<slug>.<section>",
    );

    let scaffold_paths = [
        "docs/grund.md",
        "docs/goals.md",
        "requirements.md",
        "docs/architecture/README.md",
        "docs/decisions/architectural/README.md",
        "docs/decisions/functional/README.md",
    ];
    let before = scaffold_paths
        .iter()
        .map(|relative| (*relative, fs::read(target.join(relative)).unwrap()))
        .collect::<Vec<_>>();

    replace_repo_format(&target, "format = \"{kind}-{slug}\"");
    let configured = fs::read(target.join("grund.toml")).expect("read configured grund.toml");

    // Without --force, an owned scaffold remains byte-for-byte untouched even
    // when its teaching shape no longer matches the config (§FS-init.3.3).
    let preserved = run_grund(
        &["init", target.to_str().unwrap(), "--docs"],
        manifest_dir(),
    );
    assert!(
        preserved.status.success(),
        "non-force init --docs failed: {}",
        String::from_utf8_lossy(&preserved.stderr)
    );
    for (relative, expected) in &before {
        assert_eq!(
            fs::read(target.join(relative)).unwrap(),
            *expected,
            "non-force init changed {relative}"
        );
    }

    let forced = run_grund(
        &["init", target.to_str().unwrap(), "--docs", "--force"],
        manifest_dir(),
    );
    assert!(
        forced.status.success(),
        "forced init --docs failed: {}",
        String::from_utf8_lossy(&forced.stderr)
    );
    assert_eq!(
        fs::read(target.join("grund.toml")).unwrap(),
        configured,
        "--force must preserve the effective configuration (§FS-init.3.5)"
    );
    assert_example(&target, "AGENTS.md", "<KIND>-<slug>[.<section>]");
    assert_slug_only_scaffold(&target);
}

#[test]
fn init_docs_renders_kind_overrides_and_fallbacks_in_compatibility_fs_home() {
    let target = workdir("init_docs_renders_kind_overrides_and_fallbacks_in_compatibility_fs_home");
    fs::create_dir_all(target.join(".agents")).expect("create config directory");
    fs::write(
        target.join(".agents/grund.toml"),
        r#"grund_config_version = 1

[id]
format = "{kind}:{number}"

[[kinds]]
kind = "GOAL"
file = "docs/goals.md"
title = "Goals"

[[kinds]]
kind = "FS"
folder = "docs/functional-spec"
title = "Functional spec"
format = "{kind}:{slug}"

[[kinds]]
kind = "AR"
folder = "docs/architecture"
title = "Architecture"
format = "{kind}_{number}_{slug}"
"#,
    )
    .expect("write mixed-format config");

    let output = run_grund(
        &["init", target.to_str().unwrap(), "--docs"],
        manifest_dir(),
    );
    assert!(
        output.status.success(),
        "mixed-format init --docs failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !target.join("requirements.md").exists(),
        "the compatibility FS folder replaces the file-form FS home"
    );

    // GRUND, DF, and DA have no row; GOAL has a row without an override. All
    // four use the repository's number-only fallback and literal colon.
    for (relative, expected) in [
        ("docs/grund.md", "# GRUND:<NNN>: …"),
        ("docs/goals.md", "# GOAL:<NNN>: …"),
        ("docs/decisions/architectural/README.md", "`DA:<NNN>` ID"),
        ("docs/decisions/functional/README.md", "`DF:<NNN>` ID"),
    ] {
        assert_example(&target, relative, expected);
    }

    // FS and AR demonstrate authoritative kind overrides: slug-only in the
    // compatibility home and numbered-with-slug with literal underscores.
    assert_example(&target, "docs/functional-spec/README.md", "# FS:<slug>: …");
    for expected in [
        "`AR_<NNN>_<slug>` ID",
        "`§AR_<NNN>_<slug>.<section>`",
        "`# AR_<NNN>_<slug>: [<path>](<path>)`",
    ] {
        assert_example(&target, "docs/architecture/README.md", expected);
    }
}
