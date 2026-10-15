//! Black-box coverage for the effective ID shapes in `init --scaffold` scaffolds.
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

/// §FS-init.2.4.3: the starter writes no `id_format`, so a project sets one by
/// adding `[schema] id_format` below the envelope.
fn set_repo_format(root: &Path, format: &str) {
    let path = root.join("grund.toml");
    let before = fs::read_to_string(&path).expect("read generated grund.toml");
    assert!(
        !before.contains("id_format"),
        "generated config should leave the ID format at its default"
    );
    let envelope = "project_name = \"";
    let at = before
        .find(envelope)
        .expect("generated config names its project");
    let at = at + before[at..].find('\n').expect("project_name ends its line") + 1;
    let after = format!(
        "{}\n[schema]\nid_format = \"{format}\"\n{}",
        &before[..at],
        &before[at..]
    );
    fs::write(&path, after).expect("write configured format");
}

fn assert_scaffold_shape(root: &Path, shape: &str) {
    for (relative, expected) in [
        ("docs/grund.md", "# GRUND-{shape}: …"),
        ("docs/goals.md", "# GOAL-{shape}: …"),
        ("requirements.md", "## FS-{shape}: …"),
        ("docs/architecture/README.md", "`AR-{shape}` ID"),
        ("docs/architecture/README.md", "`\u{a7}AR-{shape}.<section>`"),
        (
            "docs/architecture/README.md",
            "`# AR-{shape}: [<path>](<path>)`",
        ),
        ("docs/decisions/architectural/README.md", "`DA-{shape}` ID"),
        ("docs/decisions/functional/README.md", "`DF-{shape}` ID"),
    ] {
        assert_example(root, relative, &expected.replace("{shape}", shape));
    }
}

#[test]
fn init_scaffold_force_refreshes_every_example_to_the_repository_format() {
    let target = workdir("init_scaffold_force_refreshes_every_example_to_the_repository_format");
    let first = run_grund(
        &["init", target.to_str().unwrap(), "--scaffold"],
        manifest_dir(),
    );
    assert!(
        first.status.success(),
        "initial init --scaffold failed: {}",
        String::from_utf8_lossy(&first.stderr)
    );

    // §FS-init.2.4.3: the default is v2's slug-only teaching shape.
    assert_scaffold_shape(&target, "<slug>");

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

    set_repo_format(&target, "{kind}-{number}-{slug}");
    let configured = fs::read(target.join("grund.toml")).expect("read configured grund.toml");

    // Without --force, an owned scaffold remains byte-for-byte untouched even
    // when its teaching shape no longer matches the config (§FS-init.3.3).
    let preserved = run_grund(
        &["init", target.to_str().unwrap(), "--scaffold"],
        manifest_dir(),
    );
    assert!(
        preserved.status.success(),
        "non-force init --scaffold failed: {}",
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
        &["init", target.to_str().unwrap(), "--scaffold", "--force"],
        manifest_dir(),
    );
    assert!(
        forced.status.success(),
        "forced init --scaffold failed: {}",
        String::from_utf8_lossy(&forced.stderr)
    );
    assert_eq!(
        fs::read(target.join("grund.toml")).unwrap(),
        configured,
        "--force must preserve the effective configuration (§FS-init.3.5)"
    );
    assert_example(&target, "AGENTS.md", "<KIND>-<NNN>-<slug>[.<section>]");
    assert_scaffold_shape(&target, "<NNN>-<slug>");
}

#[test]
fn init_scaffold_renders_kind_overrides_and_fallbacks_in_compatibility_fs_home() {
    let target =
        workdir("init_scaffold_renders_kind_overrides_and_fallbacks_in_compatibility_fs_home");
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
        &["init", target.to_str().unwrap(), "--scaffold"],
        manifest_dir(),
    );
    assert!(
        output.status.success(),
        "mixed-format init --scaffold failed: {}",
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

#[test]
fn init_scaffold_preserves_literal_kind_tokens_in_fallback_and_override_shapes() {
    let target =
        workdir("init_scaffold_preserves_literal_kind_tokens_in_fallback_and_override_shapes");
    fs::create_dir_all(target.join(".agents")).expect("create config directory");
    fs::write(
        target.join(".agents/grund.toml"),
        r#"grund_config_version = 1

[id]
format = "{kind}<KIND>:{slug}"

[[kinds]]
kind = "GOAL"
file = "docs/goals.md"
title = "Goals"
format = "{kind}<KIND>_{number}"
"#,
    )
    .expect("write literal-delimiter config");

    let output = run_grund(
        &["init", target.to_str().unwrap(), "--scaffold"],
        manifest_dir(),
    );
    assert!(
        output.status.success(),
        "literal-delimiter init --scaffold failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Absent kind rows use the repository fallback without interpreting the
    // literal `<KIND>` delimiter as the schematic kind placeholder.
    assert_example(&target, "docs/grund.md", "# GRUND<KIND>:<slug>: …");
    for expected in [
        "`AR<KIND>:<slug>` ID",
        "`§AR<KIND>:<slug>.<section>`",
        "`# AR<KIND>:<slug>: [<path>](<path>)`",
    ] {
        assert_example(&target, "docs/architecture/README.md", expected);
    }

    // A kind override follows the same literal-preservation rule while proving
    // that its effective format, rather than the repository fallback, won.
    assert_example(&target, "docs/goals.md", "# GOAL<KIND>_<NNN>: …");
}
