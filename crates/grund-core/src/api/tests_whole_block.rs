//! Test module: `check` holds the whole managed block (§FS-check.3.5.4) — the
//! render it compares is `init`'s on either conversation surface, the parts a
//! stale finding names, the directory a finding about another project's block
//! names (§FS-errors.3.6.1), and the runs that leave the two workspace sections
//! out (§FS-check.3.5.5, §FS-check.4.9.1).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use super::{CheckOpts, CheckOutput, check_with_opts};
use crate::Finding;
use crate::config::load_config;
use crate::grammar::block_sections;
use crate::testing::{current_block, scan_tree, test_root, write};
use crate::writers::{InitOpts, WorkspaceBlocks, expected, init};

const BEGIN: &str = "<!-- BEGIN GRUND MANAGED BLOCK -->";
const END: &str = "<!-- END GRUND MANAGED BLOCK -->";

fn project(root: &Path, name: &str, file: &str, id: &str) {
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 2\nproject_name = \"{name}\"\n\n\
             [schema.kinds.FS]\nfiles = [\"{file}\"]\n"
        ),
    );
    write(&root.join(file), &format!("# {id}: A spec\n\nText.\n"));
}

/// A workspace root `shop` with the members `members` names, each a project
/// of its own.
fn workspace(name: &str, members: &[&str], optional: &[&str]) -> std::path::PathBuf {
    let root = test_root(name);
    project(&root, "shop", "requirements.md", "FS-receipt");
    let list = |names: &[&str]| {
        names
            .iter()
            .map(|m| format!("\"packages/{m}\""))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let toml = fs::read_to_string(root.join("grund.toml")).expect("root config");
    write(
        &root.join("grund.toml"),
        &format!(
            "{toml}\n[workspace]\nmembers = [{}]\noptional_members = [{}]\n",
            list(members),
            list(optional)
        ),
    );
    for member in members.iter().chain(optional) {
        let dir = root.join("packages").join(member);
        project(&dir, member, "spec.md", &format!("FS-{member}"));
    }
    root
}

fn init_at(root: &Path) {
    init(InitOpts {
        target: root.to_path_buf(),
        // §FS-init.1.2.3: a bare temp root no VCS marker covers.
        no_vcs: true,
        ..InitOpts::default()
    })
    .expect("init");
}

/// `init` once in every project of the workspace at `root`: no block says which
/// projects are initialized, so the order leaves none stale (§FS-init.2.3.4.15.6,
/// §FS-init.requirements.clean).
fn init_workspace(root: &Path, members: &[&Path]) {
    members.iter().for_each(|member| init_at(member));
    init_at(root);
}

fn check(path: &Path) -> CheckOutput {
    check_with_opts(CheckOpts {
        path: path.to_path_buf(),
        path_provided: true,
        ..CheckOpts::default()
    })
    .expect("check")
}

fn agents_init(output: &CheckOutput) -> Vec<String> {
    let line = |f: &Finding| format!("{}: {}", f.path.as_deref().unwrap_or_default(), f.message);
    output
        .report
        .errors
        .iter()
        .filter(|f| f.code == "agents-init")
        .map(line)
        .collect()
}

/// The bytes of `path`'s managed block before its end delimiter, as the
/// expected parts partition them.
fn block_on_disk(path: &Path) -> String {
    let text = fs::read_to_string(path).expect("entrypoint");
    let start = text.find(BEGIN).expect("begin delimiter");
    let end = text.find(END).expect("end delimiter");
    text[start..end].to_string()
}

/// §FS-check.3.5.4, §FS-init.2.3.5.8: the parts `Expected` holds a root
/// entrypoint to are, in order, the block `init` wrote there, on the plain
/// surface of `AGENTS.md` and the linked one of a standalone `CLAUDE.md`.
#[test]
fn expected_root_block_is_inits_on_the_plain_and_linked_surfaces() {
    let root = test_root("whole_block_surfaces");
    project(&root, "demo", "requirements.md", "FS-a");
    // §FS-config-v2.presentation: the opinion the linked surface renders.
    let toml = fs::read_to_string(root.join("grund.toml")).expect("config");
    write(
        &root.join("grund.toml"),
        &format!("{toml}\n[presentation]\nconversation = \"link\"\n"),
    );
    write(&root.join("AGENTS.md"), "# demo\n");
    write(&root.join("CLAUDE.md"), "# demo\n");
    init_at(&root);
    let config = load_config(&root).expect("config");
    let (catalog, _) = scan_tree(&config, Some(&root), true).expect("scan");
    let rendered = expected(
        &catalog,
        &config,
        &BTreeMap::new(),
        WorkspaceBlocks::LeftOut,
    );
    let mut blocks = Vec::new();
    for name in ["AGENTS.md", "CLAUDE.md"] {
        let entrypoint = rendered
            .entrypoints
            .iter()
            .find(|e| e.path.ends_with(name))
            .unwrap_or_else(|| panic!("{name} is expected"));
        let whole: String = entrypoint
            .sections
            .iter()
            .map(|s| s.bytes.as_str())
            .collect();
        assert_eq!(whole, block_on_disk(&root.join(name)), "{name}");
        blocks.push(whole);
    }
    assert_ne!(blocks[0], blocks[1], "the two surfaces render apart");
    assert_eq!(agents_init(&check(&root)), Vec::<String>::new());
}

/// §FS-check.3.5.3: the parts a stale finding names, in block order, and the
/// parts cover every byte before the end delimiter.
#[test]
fn block_sections_name_every_part_of_the_block() {
    let block = current_block();
    let sections = block_sections(&block);
    let nouns: Vec<String> = sections.iter().map(|s| s.noun()).collect();
    assert_eq!(
        nouns,
        [
            "citation scheme",
            "grounding from a citation",
            "project map",
            "declarations and citations",
            "rules",
            "citation directions",
            "clickable citations",
        ]
    );
    let whole: String = sections.iter().map(|s| s.bytes).collect();
    assert_eq!(whole, block[..block.find(END).expect("end")]);
}

/// §FS-check.3.5.5, §FS-init.2.3.4.15: after `init` in every project the
/// root's run renders the members' bullets as `init` did; a member joining
/// turns every block red, and each finding about another project's block —
/// a companion under `.claude/` included — names that project's root
/// (§FS-errors.3.6.1). Inside the member the workspace sections are left out.
/// §FS-check.3.5.3.1: each detail compares the first unequal body lines positionally.
#[test]
fn a_new_member_names_each_block_by_the_directory_to_refresh_it_in() {
    let root = workspace("whole_block_new_member", &["billing"], &[]);
    let billing = root.join("packages/billing");
    write(&billing.join("AGENTS.md"), "# billing\n");
    write(&billing.join(".claude/CLAUDE.md"), "# billing\n");
    init_workspace(&root, &[&billing]);
    assert_eq!(agents_init(&check(&root)), Vec::<String>::new());

    let toml = fs::read_to_string(root.join("grund.toml")).expect("root config");
    write(
        &root.join("grund.toml"),
        &toml.replace(
            "members = [\"packages/billing\"]",
            "members = [\"packages/billing\", \"packages/notify\"]",
        ),
    );
    project(
        &root.join("packages/notify"),
        "notify",
        "spec.md",
        "FS-notify",
    );
    let members = |dir: &str| {
        let (entry, found) = if dir.is_empty() {
            (
                "- [`notify`](packages/notify/) *(not yet initialized)*",
                "<missing>",
            )
        } else {
            (
                "- [`notify`](../notify/) *(not yet initialized)*",
                "- [`shop`](../../AGENTS.md)",
            )
        };
        let refresh = if dir.is_empty() {
            "run `grund init` to refresh".to_string()
        } else {
            format!("run `grund init` in {dir} to refresh")
        };
        format!(
            "repo maintenance: stale grund init block: the workspace members section differs \
             from the workspace (expected {entry}; found {found}) — {refresh} \
             (does not affect citation validity)"
        )
    };
    assert_eq!(
        agents_init(&check(&root)),
        [
            format!("AGENTS.md: {}", members("")),
            format!(
                "packages/billing/.claude/CLAUDE.md: {}",
                members("packages/billing")
            ),
            format!(
                "packages/billing/AGENTS.md: {}",
                members("packages/billing")
            ),
        ]
    );
    assert_eq!(agents_init(&check(&billing)), Vec::<String>::new());
}

/// §FS-check.3.5.5, §FS-check.4.9.1: a checkout without an optional member
/// leaves the workspace sections out, and says so in the member's warning
/// only where the run verified a block.
#[test]
fn an_absent_optional_member_leaves_the_workspace_sections_out() {
    let root = workspace("whole_block_optional", &["billing"], &["secret"]);
    init_workspace(&root, &[&root.join("packages/billing")]);
    assert_eq!(agents_init(&check(&root)), Vec::<String>::new());

    fs::remove_dir_all(root.join("packages/secret")).expect("remove the optional member");
    let warning = |output: &CheckOutput| {
        output
            .report
            .warnings
            .iter()
            .find(|f| f.code == "optional-member-absent")
            .map(|f| f.message.clone())
            .expect("the absent member's warning")
    };
    let output = check(&root);
    assert_eq!(agents_init(&output), Vec::<String>::new());
    assert!(
        warning(&output)
            .ends_with("so this run does not cover it, nor the managed blocks' workspace sections"),
        "{}",
        warning(&output)
    );

    fs::remove_file(root.join("AGENTS.md")).expect("remove the root block");
    fs::remove_file(root.join("packages/billing/AGENTS.md")).expect("remove the member block");
    let output = check(&root);
    assert!(
        warning(&output).ends_with("so this run does not cover it"),
        "{}",
        warning(&output)
    );
}
