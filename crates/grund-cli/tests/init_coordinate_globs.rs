//! The managed block, the query guide and the `grund-init` skill teach coordinate globs in place
//! of the `jq` pipe (§FS-init.2.3.4.3.1), and every block from v27 does (§FS-init.2.3.7.1).

use std::fs;

#[path = "support/init_fixture.rs"]
mod init_fixture;

use init_fixture::{manifest_dir, run_grund, workdir};

const QUERYING_URL: &str =
    "https://github.com/agent-grounds/grund/blob/main/docs/user-facing/querying.md";

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

fn glob_bullet(agents: &str) -> Option<&str> {
    agents
        .lines()
        .find(|line| line.starts_with("- ") && line.contains("grund list --glob"))
}

/// The default format writes `-` after `{kind}`, so the bullet's pattern is `<KIND>-*`; it
/// counts with `--total`, reads with `show --batch --glob`, names `**` and `{a,b}`, and links
/// the guide absolutely, with no `jq` left in the block.
#[test]
fn init_block_teaches_the_glob_recipe() {
    let agents = init_agents(&workdir("init_block_teaches_the_glob_recipe"));
    assert!(
        agents.contains("## Grounding with grund (block v43)"),
        "the current base block, v43, teaches globs"
    );
    let bullet = glob_bullet(&agents).expect("a bullet teaching `grund list --glob`");
    for needle in [
        "grund list --glob '<KIND>-*",
        "--total",
        "grund show --batch --glob '<KIND>-*",
        "--toc --format json",
        "`**`",
        "`{a,b}`",
        QUERYING_URL,
    ] {
        assert!(
            bullet.contains(needle),
            "the glob bullet lacks `{needle}`: {bullet}"
        );
    }
    assert!(
        !agents.contains("| jq"),
        "the block still teaches the jq pipe"
    );
}

/// A `{kind}{number}` project's IDs have no `-` after the kind, so it is taught `<KIND>*`.
#[test]
fn init_block_builds_the_pattern_from_the_format() {
    let target = workdir("init_block_builds_the_pattern_from_the_format");
    fs::create_dir_all(target.join("docs/fs")).expect("create docs/fs");
    fs::write(
        target.join("grund.toml"),
        "grund_config_version = 1\n\n[id]\nformat = \"{kind}{number}\"\n\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\nindex = false\n",
    )
    .expect("write grund.toml");
    let agents = init_agents(&target);
    let bullet = glob_bullet(&agents).expect("a bullet teaching `grund list --glob`");
    assert!(bullet.contains("--glob '<KIND>*"), "{bullet}");
    assert!(
        !agents.contains("'<KIND>-*"),
        "a `{{kind}}{{number}}` project has no `-`"
    );
}

/// The guide carries the glob recipes and the review, and both skill copies' `queries`
/// section teaches the glob recipe in place of the pipe.
#[test]
fn querying_guide_and_skill_teach_the_glob_recipe() {
    let root = manifest_dir();
    let guide = fs::read_to_string(root.join("docs/user-facing/querying.md"))
        .expect("docs/user-facing/querying.md, the Querying grund guide");
    for recipe in [
        "list --glob",
        "list --total",
        "show --batch --glob",
        "{,.**}",
        "cover",
        "--lines",
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
            text.contains("show --batch --glob"),
            "{skill} does not teach the glob recipe"
        );
        assert!(!text.contains("| jq"), "{skill} still teaches the jq pipe");
        assert!(
            text.contains(QUERYING_URL),
            "{skill} does not link the guide"
        );
    }
}
