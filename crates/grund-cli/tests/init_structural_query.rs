//! The managed block, the user-facing guide and the `grund-init` skill teach how
//! to query grund's structure (§FS-init.2.3.4.3.1): selecting units with
//! `list --selector` and expanding them with `show --batch --toc --format json`.

use std::fs;

#[path = "support/init_fixture.rs"]
mod init_fixture;

use init_fixture::{manifest_dir, run_grund, workdir};

const QUERYING_URL: &str =
    "https://github.com/agent-grounds/grund/blob/main/docs/user-facing/querying.md";

/// The block lands in other repositories, so it teaches the pipeline inline and
/// links the guide absolutely, in one bullet (§FS-init.2.3.4.3.1).
#[test]
fn init_block_teaches_selector_to_batch_query_and_links_the_guide() {
    let target = workdir("init_block_teaches_selector_to_batch_query_and_links_the_guide");
    let output = run_grund(
        &["init", target.to_str().unwrap(), "--agents-md"],
        manifest_dir(),
    );
    assert!(
        output.status.success(),
        "init failed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let agents = fs::read_to_string(target.join("AGENTS.md")).expect("read AGENTS.md");
    let bullet = agents.lines().find(|line| {
        line.starts_with("- ")
            && line.contains("grund list --selector")
            && line.contains("grund show --batch --toc --format json")
            && line.contains(QUERYING_URL)
    });
    assert!(
        bullet.is_some(),
        "the managed block must carry one bullet naming `grund list --selector`, \
         `grund show --batch --toc --format json` and {QUERYING_URL}"
    );
    assert!(
        !agents.contains("(docs/user-facing/querying.md)"),
        "the block must not link a repo-relative guide its downstream repository lacks"
    );
}

/// The guide holds every recipe the spec names, and both skill copies teach the
/// subtree expansion and link the guide (§FS-init.2.3.4.3.1).
#[test]
fn querying_guide_and_skill_hold_the_query_recipes() {
    let root = manifest_dir();
    let guide = fs::read_to_string(root.join("docs/user-facing/querying.md"))
        .expect("docs/user-facing/querying.md, the Querying grund guide");
    for recipe in [
        "list --selector",
        "list --kind",
        "show --batch --toc --format json",
        ".result.sections[]",
        "show --batch --brief",
        "show --batch --full",
        "--all",
        "refs",
        "--descendants",
        "cover --format json",
        "enclosing_declaration",
        "enclosing_section",
    ] {
        assert!(
            guide.contains(recipe),
            "querying.md does not teach `{recipe}`"
        );
    }
    for skill in [
        "skills/grund-init/SKILL.md",
        "crates/grund-core/assets/skills/grund-init/SKILL.md",
    ] {
        let text = fs::read_to_string(root.join(skill)).expect(skill);
        assert!(
            text.contains("show --batch --toc --format json"),
            "{skill} does not teach the subtree expansion"
        );
        assert!(
            text.contains(QUERYING_URL),
            "{skill} does not link the guide"
        );
    }
}
