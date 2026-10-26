//! The managed block's cheap-grounding list teaches `grund list --unused` as one bullet directly
//! after the `grund list --kind` one (§FS-init.2.3.4.3), in every v43–v46 block
//! (§FS-init.2.3.7.1).

use std::fs;

#[path = "support/init_fixture.rs"]
mod init_fixture;

use init_fixture::{manifest_dir, run_grund, workdir};

const KIND_BULLET: &str =
    "- `grund list` / `grund list --kind FS,REQ` — discover IDs if you get lost";
const UNUSED_BULLET: &str =
    "- `grund list --unused` — the declarations nothing cites yet, in one call";

fn init_agents(target: &std::path::Path) -> String {
    let output = run_grund(
        &["init", target.to_str().unwrap(), "--agents-md"],
        manifest_dir(),
    );
    assert!(
        output.status.success(),
        "init failed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    fs::read_to_string(target.join("AGENTS.md")).expect("read AGENTS.md")
}

/// The bullet reads exactly as the spec fixes it, follows the `--kind` bullet, and is taught once.
#[test]
fn init_block_teaches_the_unused_rung() {
    let agents = init_agents(&workdir("init_block_teaches_the_unused_rung"));
    assert!(
        agents.contains("## Grounding with grund (block v43)"),
        "the current base block, v43, teaches the rung"
    );
    let lines: Vec<&str> = agents.lines().collect();
    let kind = lines
        .iter()
        .position(|line| *line == KIND_BULLET)
        .expect("the `grund list --kind` bullet");
    assert_eq!(
        lines.get(kind + 1).copied(),
        Some(UNUSED_BULLET),
        "the bullet after `grund list --kind` is not the `--unused` rung"
    );
    assert_eq!(
        agents.matches("grund list --unused").count(),
        1,
        "the block teaches `grund list --unused` once"
    );
}
