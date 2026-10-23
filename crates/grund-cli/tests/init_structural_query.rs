//! The managed block, the user-facing guide and the `grund-init` skill teach how
//! to query grund's structure (§FS-init.2.3.4.3.1): selecting units and expanding
//! them with `show --batch … --toc --format json`. What the glob recipe says is
//! pinned in `init_coordinate_globs.rs`; this file holds what that leaves out.

use std::fs;

#[path = "support/init_fixture.rs"]
mod init_fixture;

use init_fixture::{manifest_dir, run_grund, workdir};

const QUERYING_URL: &str =
    "https://github.com/agent-grounds/grund/blob/main/docs/user-facing/querying.md";

/// The block lands in other repositories, so the bullet that expands units links the
/// guide absolutely and never by a repo-relative path (§FS-init.2.3.4.3.1).
#[test]
fn init_block_links_the_query_guide_absolutely() {
    let target = workdir("init_block_links_the_query_guide_absolutely");
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
            && line.contains("grund show --batch")
            && line.contains("--toc --format json")
            && line.contains(QUERYING_URL)
    });
    assert!(
        bullet.is_some(),
        "the managed block must carry one bullet naming `grund show --batch`, \
         `--toc --format json` and {QUERYING_URL}"
    );
    assert!(
        !agents.contains("(docs/user-facing/querying.md)"),
        "the block must not link a repo-relative guide its downstream repository lacks"
    );
}

/// The guide holds every recipe the spec names, and both skill copies' `queries`
/// section teaches the subtree expansion and links the guide (§FS-init.2.3.4.3.1). The explicit stdin form
/// stays in the guide; the skill teaches it through `--glob`.
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
        "skills/grund-init/references/queries.md",
        "crates/grund-core/assets/skills/grund-init/references/queries.md",
    ] {
        let text = fs::read_to_string(root.join(skill)).expect(skill);
        assert!(
            text.contains("show --batch --glob") && text.contains("--toc --format json"),
            "{skill} does not teach the subtree expansion"
        );
        assert!(
            text.contains(QUERYING_URL),
            "{skill} does not link the guide"
        );
    }
}

/// The guide teaches the declarations nothing cites yet as one call, `list --unused`, with
/// `--kind E2E` for the e2e cases it skips and the JSON form, and links the checking
/// guide for why an index entry is not a use; both skill copies name the call, so an agent
/// that reads them never loops `grund refs` over every ID (§FS-init.2.3.4.3.1).
#[test]
#[ignore = "red until #938 the guide and skill teach grund list --unused; implement removes this"]
fn querying_guide_and_skill_teach_the_unused_recipe() {
    let root = manifest_dir();
    let guide = fs::read_to_string(root.join("docs/user-facing/querying.md"))
        .expect("docs/user-facing/querying.md, the Querying grund guide");
    for recipe in [
        "grund list --unused",
        "--unused --kind E2E",
        "--unused --format json",
        "checking.md#when-only-an-index-cites-a-declaration",
    ] {
        assert!(
            guide.contains(recipe),
            "querying.md does not teach `{recipe}`"
        );
    }
    for skill in [
        "skills/grund-init/references/queries.md",
        "crates/grund-core/assets/skills/grund-init/references/queries.md",
    ] {
        let text = fs::read_to_string(root.join(skill)).expect(skill);
        assert!(
            text.contains("grund list --unused"),
            "{skill} does not name `grund list --unused`"
        );
    }
}
