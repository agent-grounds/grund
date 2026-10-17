//! Black-box init guidance for each config version (§FS-init.2.3.8.1).
//! The v2 cases port #784's successful-init/absent-key reproducer through the
//! existing init fixture; v1's two sentences remain compatibility controls.

use std::fs;
use std::path::{Path, PathBuf};

#[path = "support/init_fixture.rs"]
mod init_fixture;

use init_fixture::{manifest_dir, run_grund, workdir};

fn fixture(name: &str, config: &str) -> PathBuf {
    let target = workdir(name);
    fs::create_dir_all(target.join("docs/fs")).unwrap();
    fs::write(target.join("docs/fs/a.md"), "# FS-a: Example\n").unwrap();
    fs::write(target.join("grund.toml"), config).unwrap();
    target
}

fn init_bare_note(target: &Path) -> String {
    let output = run_grund(&["init", target.to_str().unwrap()], manifest_dir());
    assert!(
        output.status.success(),
        "init must succeed before checking guidance: {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let agents = fs::read_to_string(target.join("AGENTS.md")).unwrap();
    let opening = agents
        .lines()
        .find(|line| line.contains("Bare ID-shaped tokens"))
        .expect("opening paragraph explains bare ID-shaped tokens");
    opening[opening.find("Bare ID-shaped tokens").unwrap()..].to_string()
}

fn assert_v2_guidance(target: &Path, marker: &str) {
    let config = fs::read_to_string(target.join("grund.toml")).unwrap();
    assert!(config.contains("grund_config_version = 2"), "{config}");
    assert!(!config.contains("[reference]") && !config.contains("strict"));
    let note = init_bare_note(target);
    for forbidden in [
        "[reference]",
        "strict",
        "compatibility override",
        "grund fmt --marker",
    ] {
        assert!(
            !note.contains(forbidden),
            "v2 guidance must not mention the removed setting or its recovery advice ({forbidden}):\n{note}"
        );
    }
    for required in [
        "Bare ID-shaped tokens are ignored".to_string(),
        format!("only `{marker}`-prefixed citations"),
        "version 2".to_string(),
        "no opt-out".to_string(),
    ] {
        assert!(
            note.contains(&required),
            "missing `{required}` in v2 guidance:\n{note}"
        );
    }
}

const V2: &str = "grund_config_version = 2\nproject_name = \"demo\"\n\n\
                  [schema.kinds.FS]\nfolders = [\"docs/fs\"]\n";

fn v1(strict: bool) -> String {
    format!(
        "grund_config_version = 1\nproject_name = \"demo\"\n\n\
         [reference]\nstrict = {strict}\nmarker = \"@\"\n\n\
         [id]\nformat = \"{{kind}}-{{slug}}\"\nnamed_sections = true\n\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\n\n\
         [scan]\ninclude = [\"docs/fs\"]\nextensions = [\"md\"]\n"
    )
}

#[test]
fn citation_guidance_minimal_v2_does_not_claim_a_removed_key() {
    let target = fixture("citation_guidance_minimal_v2", V2);
    assert_v2_guidance(&target, "§");
}

#[test]
fn citation_guidance_v2_uses_the_configured_marker() {
    let config = format!("{V2}\n[schema]\nmarker = \"@\"\n");
    let target = fixture("citation_guidance_v2_custom_marker", &config);
    assert_v2_guidance(&target, "@");
}

#[test]
fn citation_guidance_migrated_v2_uses_fixed_recognition() {
    let target = fixture("citation_guidance_migrated_v2", &v1(true));
    let migration = run_grund(
        &[
            "config",
            "migrate",
            "--to",
            "2",
            "--write",
            target.to_str().unwrap(),
        ],
        manifest_dir(),
    );
    assert!(
        migration.status.success(),
        "migration must succeed: {}{}",
        String::from_utf8_lossy(&migration.stdout),
        String::from_utf8_lossy(&migration.stderr)
    );
    assert_v2_guidance(&target, "@");
}

#[test]
fn citation_guidance_v1_strict_true_retains_its_sentence() {
    let target = fixture("citation_guidance_v1_strict_true", &v1(true));
    assert_eq!(
        init_bare_note(&target),
        "Bare ID-shaped tokens are ignored — `[reference] strict = true` is set in `grund.toml`, so only `@`-prefixed citations are checked."
    );
}

#[test]
fn citation_guidance_v1_strict_false_retains_its_sentence() {
    let target = fixture("citation_guidance_v1_strict_false", &v1(false));
    assert_eq!(
        init_bare_note(&target),
        "Bare ID-shaped tokens are also recognized as citations because `[reference] strict = false` is set in `grund.toml`; remove that compatibility override or set strict back to `true` to require the `@` marker (run `grund fmt --marker` first to upgrade existing bare citations)."
    );
}
