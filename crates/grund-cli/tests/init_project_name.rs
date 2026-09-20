//! Project-name precedence for generated canonical entrypoints (§FS-init.2.3.8).

use std::fs;

#[path = "support/init_fixture.rs"]
mod init_fixture;

use init_fixture::{manifest_dir, run_grund, workdir};

fn canonical_heading(target: &std::path::Path) -> String {
    fs::read_to_string(target.join("AGENTS.md"))
        .expect("read AGENTS.md")
        .lines()
        .next()
        .expect("AGENTS.md has a heading")
        .to_string()
}

/// §FS-init.2.3.8: the reported two-command sequence keeps the configured
/// generated identity when `--force` rewrites the canonical entrypoint. Both
/// successful states are asserted so a broken setup cannot impersonate the
/// regression, and the config bytes prove that the second name was read rather
/// than rewritten.
#[test]
fn init_force_keeps_configured_name_across_reported_sequence() {
    let root = workdir("init_force_keeps_configured_name_across_reported_sequence");
    let target = root.join("probe/main");
    fs::create_dir_all(&target).expect("create probe/main target");

    let first = run_grund(
        &[
            "init",
            target.to_str().unwrap(),
            "--docs",
            "--name",
            "probe",
        ],
        manifest_dir(),
    );
    assert!(
        first.status.success(),
        "initial init failed: stderr={}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_eq!(canonical_heading(&target), "# probe — agent instructions");
    let config_before = fs::read(target.join("grund.toml")).expect("read initial config");

    let forced = run_grund(
        &["init", target.to_str().unwrap(), "--docs", "--force"],
        manifest_dir(),
    );
    assert!(
        forced.status.success(),
        "forced init failed: stderr={}",
        String::from_utf8_lossy(&forced.stderr)
    );
    assert_eq!(
        canonical_heading(&target),
        "# probe — agent instructions",
        "forced canonical generation must use target-local project_name"
    );
    assert_eq!(
        fs::read(target.join("grund.toml")).expect("read config after force"),
        config_before,
        "forced init must leave the existing config byte-for-byte unchanged"
    );
}

/// §FS-init.2.3.8: first canonical creation uses `project_name` from either
/// supported config location, even when the directory basename differs.
#[test]
fn init_first_canonical_creation_uses_name_from_either_target_config_location() {
    for (case, config_rel) in [("bare", "grund.toml"), ("agents", ".agents/grund.toml")] {
        let root = workdir(&format!(
            "init_first_canonical_creation_uses_name_from_{case}_config"
        ));
        let target = root.join("main");
        let config_path = target.join(config_rel);
        fs::create_dir_all(config_path.parent().unwrap()).expect("create target config directory");
        let config = format!("grund_config_version = 1\nproject_name = \"Configured {case}\"\n");
        fs::write(&config_path, &config).expect("write target config");

        let output = run_grund(&["init", target.to_str().unwrap()], manifest_dir());
        assert!(
            output.status.success(),
            "{case} config init failed: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            canonical_heading(&target),
            format!("# Configured {case} — agent instructions"),
            "first canonical generation ignored {config_rel} project_name"
        );
        assert_eq!(
            fs::read_to_string(&config_path).expect("read target config after init"),
            config,
            "init rewrote existing {config_rel}"
        );
    }
}

/// §FS-init.1 / §FS-init.2.3.8: an explicit name has priority over a different
/// configured name but does not mutate that existing config.
#[test]
fn init_explicit_name_overrides_configured_name_without_rewriting_config() {
    let root = workdir("init_explicit_name_overrides_configured_name_without_rewriting_config");
    let target = root.join("main");
    fs::create_dir_all(&target).expect("create target");
    let config = "grund_config_version = 1\nproject_name = \"Configured\"\n";
    fs::write(target.join("grund.toml"), config).expect("write target config");

    let output = run_grund(
        &["init", target.to_str().unwrap(), "--name", "Explicit"],
        manifest_dir(),
    );
    assert!(
        output.status.success(),
        "explicit-name init failed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        canonical_heading(&target),
        "# Explicit — agent instructions"
    );
    assert_eq!(
        fs::read_to_string(target.join("grund.toml")).expect("read target config after init"),
        config,
        "an explicit --name must not rewrite an existing config"
    );
}

/// §FS-init.1 / §FS-init.2.3.8: when a target config supplies no project name,
/// canonical generation falls back to the resolved target basename and still
/// leaves the config byte-for-byte unchanged.
#[test]
fn init_name_falls_back_to_target_basename_when_config_omits_project_name() {
    let root = workdir("init_name_falls_back_to_target_basename_when_config_omits_project_name");
    let target = root.join("main");
    fs::create_dir_all(&target).expect("create target");
    let config = "grund_config_version = 1\n";
    fs::write(target.join("grund.toml"), config).expect("write target config");

    let output = run_grund(&["init", target.to_str().unwrap()], manifest_dir());
    assert!(
        output.status.success(),
        "basename-fallback init failed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(canonical_heading(&target), "# main — agent instructions");
    assert_eq!(
        fs::read_to_string(target.join("grund.toml")).expect("read target config after init"),
        config,
        "basename fallback must not add project_name to an existing config"
    );
}

/// §FS-init.2.3.8: absent a target config, init does not inherit an ancestor's
/// project identity; it uses the target basename and creates a target-local
/// config carrying that name.
#[test]
fn init_name_fallback_does_not_inherit_ancestor_project_identity() {
    let root = workdir("init_name_fallback_does_not_inherit_ancestor_project_identity");
    let ancestor_config = "grund_config_version = 1\nproject_name = \"Ancestor\"\n";
    fs::write(root.join("grund.toml"), ancestor_config).expect("write ancestor config");
    let target = root.join("nested/main");
    fs::create_dir_all(&target).expect("create nested target");

    let output = run_grund(&["init", target.to_str().unwrap()], manifest_dir());
    assert!(
        output.status.success(),
        "nested init failed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(canonical_heading(&target), "# main — agent instructions");
    assert!(
        fs::read_to_string(target.join("grund.toml"))
            .expect("read target-local config")
            .contains("project_name = \"main\""),
        "init must create a target-local config with the basename identity"
    );
    assert_eq!(
        fs::read_to_string(root.join("grund.toml")).expect("read ancestor config after init"),
        ancestor_config,
        "nested init must not mutate the ancestor config"
    );
}
