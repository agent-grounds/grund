//! The managed block's `### Workspace members` section through the CLI (§FS-init.2.3.4.15): which
//! projects it lists from where `grund init` runs, and the discoverability line above them
//! (§FS-init.2.3.4.15.2). The renderer's own cases are
//! `crates/grund-core/src/writers/tests_workspace_members.rs`.

use std::fs;

#[path = "support/init_fixture.rs"]
mod init_fixture;

use init_fixture::{manifest_dir, run_grund, workdir};

#[test]
fn init_workspace_companion_only_omits_self_and_uses_marker() {
    // §FS-init.2.3.4.15: a companion-only workspace init omits self just like
    // canonical init, retains foreign rows, and uses the local citation marker
    // in the discoverability line §FS-init.2.3.4.15.2 gives verbatim.
    let root = workdir("init_workspace_companion_only_omits_self_and_uses_marker");
    fs::create_dir_all(root.join(".agents")).expect("create root config dir");
    fs::write(
        root.join(".agents/grund.toml"),
        "project_name = \"root\"\n\n[workspace]\nmembers = [\"apps/api\"]\n",
    )
    .expect("write workspace config");
    let api = root.join("apps/api");
    fs::create_dir_all(api.join(".agents")).expect("create api config dir");
    fs::write(
        api.join(".agents/grund.toml"),
        "[reference]\nmarker = \"@\"\nstrict = true\n",
    )
    .expect("write api config");

    let output = run_grund(&["init", api.to_str().unwrap(), "--claude"], manifest_dir());
    assert!(
        output.status.success(),
        "init failed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !api.join("AGENTS.md").exists(),
        "explicit companion-only init should not create AGENTS.md"
    );

    let claude = fs::read_to_string(api.join("CLAUDE.md")).expect("read CLAUDE.md");
    assert!(
        claude.lines().any(|line| line
            == "Cross-project citations use @alias/<ID>, one alias segment per workspace level."),
        "workspace citation hint should be the spec's line, with the configured marker:\n{claude}"
    );
    assert!(
        !claude.contains("`api`"),
        "companion-only init must omit its canonical self project:\n{claude}"
    );
    assert!(
        claude.contains("- [`root`](../../)\n") && !claude.contains("not yet initialized"),
        "the foreign root row should link its directory, unannotated (§FS-init.2.3.4.15.6):\n{claude}"
    );
}
