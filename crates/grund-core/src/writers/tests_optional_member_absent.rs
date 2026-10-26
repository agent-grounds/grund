//! Test module: `grund init` keeps a block's `### Workspace members` section
//! while an optional member is absent (§FS-init.2.3.4.15.8) — the splice on an
//! older block, the section rendered where there is none to keep, one note per
//! absent member, each companion's own section, `--force` (§FS-init.3.4), the
//! nested tree, and the expansion error that still suppresses the section.

use std::fs;
use std::path::{Path, PathBuf};

use super::init_block::keep_workspace_members;
use super::{InitAgentEntrypointSelection, InitOpts, InitOutput, init};
use crate::grammar::{AgentsBlockLookup, WORKSPACE_MEMBERS, block_sections, find_agents_block};
use crate::testing::{test_root, write};

const KEPT: &str = "keeps its `### Workspace members` section as it was";

fn project(root: &Path, name: &str, file: &str, id: &str, extra: &str) {
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 2\nproject_name = \"{name}\"\n\n\
             [schema.kinds.FS]\nfiles = [\"{file}\"]\n{extra}"
        ),
    );
    write(&root.join(file), &format!("# {id}: A spec\n\nText.\n"));
}

/// A workspace root `shop` with the member `billing` and the optional members
/// `optional` names, in the order given, each a project of its own.
fn shop(name: &str, optional: &[&str]) -> PathBuf {
    let root = test_root(name);
    let listed: Vec<String> = optional
        .iter()
        .map(|m| format!("\"packages/{m}\""))
        .collect();
    let workspace = format!(
        "\n[workspace]\nmembers = [\"packages/billing\"]\noptional_members = [{}]\n",
        listed.join(", ")
    );
    project(&root, "shop", "requirements.md", "FS-receipt", &workspace);
    for member in std::iter::once(&"billing").chain(optional) {
        let dir = root.join("packages").join(member);
        project(&dir, member, "spec.md", &format!("FS-{member}"), "");
    }
    root
}

fn run(target: &Path, opts: InitOpts) -> InitOutput {
    init(InitOpts {
        target: target.to_path_buf(),
        // §FS-init.1.2.3: a bare temp root no VCS marker covers.
        no_vcs: true,
        ..opts
    })
    .expect("init")
}

fn init_at(target: &Path) -> InitOutput {
    run(target, InitOpts::default())
}

/// Every project initialized, members first, so each list links every entrypoint.
fn full_checkout(root: &Path, members: &[&str]) {
    for member in members {
        init_at(&root.join("packages").join(member));
    }
    init_at(root);
}

fn events(output: &InitOutput) -> Vec<String> {
    output
        .events
        .iter()
        .map(|event| format!("{} {}", event.verb, event.path))
        .collect()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).expect("read entrypoint")
}

/// The bytes of the `### Workspace members` section of `text`'s managed block.
fn members_section(text: &str) -> Option<String> {
    let AgentsBlockLookup::Found(block) = find_agents_block(text) else {
        return None;
    };
    block_sections(&text[block.start..block.end])
        .into_iter()
        .find(|section| section.noun() == WORKSPACE_MEMBERS)
        .map(|section| section.bytes.to_string())
}

fn absent(root: &Path, member: &str) {
    fs::remove_dir_all(root.join("packages").join(member)).expect("remove member");
}

fn note(member: &str, clauses: &str) -> String {
    format!(
        "optional workspace member `packages/{member}` (namespace `{member}`) is absent, so {clauses}"
    )
}

/// §FS-init.2.3.4.15.8: a block at an older version keeps its section in the
/// rendering that wrote it, while the rest of the block is upgraded.
#[test]
fn an_older_block_keeps_its_member_list_bytes() {
    let root = shop("an_older_block_keeps_its_member_list_bytes", &["secret"]);
    full_checkout(&root, &["billing", "secret"]);
    let agents = root.join("AGENTS.md");
    let current = read(&agents);
    let section = members_section(&current).expect("full checkout lists members");
    let older = "### Workspace members\n\n- `billing`, `secret`\n\n";
    write(
        &agents,
        &current
            .replace("(block v43)", "(block v38)")
            .replace(&section, older),
    );
    absent(&root, "secret");

    let output = init_at(&root);

    assert_eq!(events(&output)[0], "updated AGENTS.md");
    let written = read(&agents);
    assert!(written.contains("(block v43)"), "{written}");
    assert_eq!(members_section(&written).as_deref(), Some(older));
    assert_eq!(output.notes, [note("secret", &format!("AGENTS.md {KEPT}"))]);
}

/// §FS-init.2.3.4.15.8: a block with no member list gets one rendered from what
/// the run can read, and the note says the new section does not list the member,
/// in the conditional under `--check`.
#[test]
fn a_block_without_the_section_renders_what_it_can_read() {
    let root = shop(
        "a_block_without_the_section_renders_what_it_can_read",
        &["secret"],
    );
    full_checkout(&root, &["billing", "secret"]);
    let agents = root.join("AGENTS.md");
    let current = read(&agents);
    let section = members_section(&current).expect("full checkout lists members");
    write(&agents, &current.replace(&section, ""));
    absent(&root, "secret");

    let check = run(
        &root,
        InitOpts {
            check: true,
            ..InitOpts::default()
        },
    );
    let output = init_at(&root);

    let new = "'s new `### Workspace members` section";
    assert_eq!(events(&check)[0], "would-update AGENTS.md");
    assert_eq!(
        check.notes,
        [note("secret", &format!("AGENTS.md{new} would not list it"))]
    );
    assert_eq!(events(&output)[0], "updated AGENTS.md");
    assert_eq!(
        output.notes,
        [note("secret", &format!("AGENTS.md{new} does not list it"))]
    );
    let written = members_section(&read(&agents)).expect("section rendered");
    assert!(written.contains("`billing`") && !written.contains("secret"));
}

/// §FS-init.2.3.4.15.8: the section is kept once however many members are
/// absent, and each absent member gets its own note, ordered by namespace rather
/// than by the order the config lists them in.
#[test]
fn two_absent_members_keep_one_section_and_give_two_notes() {
    let root = shop(
        "two_absent_members_keep_one_section_and_give_two_notes",
        &["vault", "secret"],
    );
    full_checkout(&root, &["billing", "vault", "secret"]);
    let before = read(&root.join("AGENTS.md"));
    absent(&root, "vault");
    absent(&root, "secret");

    let output = run(
        &root,
        InitOpts {
            check: true,
            ..InitOpts::default()
        },
    );

    assert_eq!(events(&output)[0], "exists AGENTS.md");
    let clause = "AGENTS.md would keep its `### Workspace members` section as it is";
    assert_eq!(
        output.notes,
        [note("secret", clause), note("vault", clause)]
    );
    assert_eq!(read(&root.join("AGENTS.md")), before);
}

fn with_claude() -> InitOpts {
    InitOpts {
        agent_selection: InitAgentEntrypointSelection {
            canonical: true,
            claude: true,
            ..InitAgentEntrypointSelection::default()
        },
        ..InitOpts::default()
    }
}

/// §FS-init.2.3.4.15.8: each root entrypoint keeps its own section, and nothing
/// is copied from one into another; the note has one clause per entrypoint, in
/// report order.
#[test]
fn each_companion_keeps_its_own_section() {
    let root = shop("each_companion_keeps_its_own_section", &["secret"]);
    init_at(&root.join("packages/billing"));
    init_at(&root.join("packages/secret"));
    run(&root, with_claude());
    let claude = root.join("CLAUDE.md");
    let agents_section = members_section(&read(&root.join("AGENTS.md"))).expect("AGENTS.md");
    let current = read(&claude);
    let section = members_section(&current).expect("CLAUDE.md lists members");
    let own = format!("{section}- a line only CLAUDE.md has\n\n");
    write(&claude, &current.replace(&section, &own));
    absent(&root, "secret");

    let output = run(&root, with_claude());

    assert_eq!(
        events(&output)[..2],
        ["exists AGENTS.md", "exists CLAUDE.md"]
    );
    assert_eq!(
        members_section(&read(&root.join("AGENTS.md"))),
        Some(agents_section)
    );
    assert_eq!(members_section(&read(&claude)), Some(own));
    assert_eq!(
        output.notes,
        [note(
            "secret",
            &format!("AGENTS.md {KEPT}; CLAUDE.md {KEPT}")
        )]
    );
}

/// §FS-init.2.3.4.15.8: a companion this run creates has no section to keep, so
/// it lists what the run can read and the note says so beside the kept one.
#[test]
fn a_new_companion_gets_the_new_clause() {
    let root = shop("a_new_companion_gets_the_new_clause", &["secret"]);
    full_checkout(&root, &["billing", "secret"]);
    absent(&root, "secret");

    let output = run(&root, with_claude());

    assert_eq!(
        events(&output)[..2],
        ["exists AGENTS.md", "wrote CLAUDE.md"]
    );
    let new = members_section(&read(&root.join("CLAUDE.md"))).expect("CLAUDE.md");
    assert!(new.contains("`billing`") && !new.contains("secret"));
    assert_eq!(
        output.notes,
        [note(
            "secret",
            &format!(
                "AGENTS.md {KEPT}; CLAUDE.md's new `### Workspace members` section does not list it"
            )
        )]
    );
}

/// §FS-init.3.4: `--force` discards the canonical `AGENTS.md`'s previous contents
/// save the member list, which it reads before it overwrites the file.
#[test]
fn force_keeps_the_member_list() {
    let root = shop("force_keeps_the_member_list", &["secret"]);
    full_checkout(&root, &["billing", "secret"]);
    let agents = root.join("AGENTS.md");
    let current = read(&agents);
    let section = members_section(&current).expect("full checkout lists members");
    write(&agents, &format!("{current}\nLocal notes.\n"));
    absent(&root, "secret");

    let output = run(
        &root,
        InitOpts {
            force: true,
            ..InitOpts::default()
        },
    );

    assert_eq!(events(&output)[0], "wrote AGENTS.md");
    let written = read(&agents);
    assert!(!written.contains("Local notes."), "{written}");
    assert_eq!(members_section(&written), Some(section));
    assert_eq!(output.notes, [note("secret", &format!("AGENTS.md {KEPT}"))]);
}

/// §FS-init.2.3.4.15.1, §FS-init.2.3.4.15.8: an optional member absent one block
/// down is absent from the whole tree, so the nested group's root and a present
/// member keep their lists too, and the note names it by its whole alias path.
#[test]
fn the_nested_group_root_and_a_present_member_decide_from_the_outermost_tree() {
    let root = test_root("nested_group_root_and_present_member_decide_from_outermost");
    let hw = root.join("groups/hw");
    project(
        &root,
        "shop",
        "requirements.md",
        "FS-receipt",
        "\n[workspace]\nmembers = [\"groups/hw\"]\n",
    );
    project(
        &hw,
        "hw",
        "hw.md",
        "FS-hw",
        "\n[workspace]\nmembers = [\"board\"]\noptional_members = [\"sprayer\"]\n",
    );
    project(&hw.join("board"), "board", "spec.md", "FS-board", "");
    project(&hw.join("sprayer"), "sprayer", "spec.md", "FS-sprayer", "");
    for target in [
        hw.join("board"),
        hw.join("sprayer"),
        hw.clone(),
        root.clone(),
    ] {
        init_at(&target);
    }
    fs::remove_dir_all(hw.join("sprayer")).expect("remove sprayer");

    let clause = "AGENTS.md would keep its `### Workspace members` section as it is";
    for target in [&root, &hw, &hw.join("board")] {
        let output = run(
            target,
            InitOpts {
                check: true,
                ..InitOpts::default()
            },
        );
        assert_eq!(events(&output)[0], "exists AGENTS.md", "{target:?}");
        assert_eq!(
            output.notes,
            [format!(
                "optional workspace member `sprayer` (namespace `hw/sprayer`) is absent, so {clause}"
            )],
            "{target:?}"
        );
    }
}

/// §FS-init.2.3.4.15, §FS-init.2.3.4.15.8: an expansion error still suppresses
/// the section, absent optional member or not — the run cannot tell that member
/// is the only gap — and with no list there is nothing to keep or note.
#[test]
fn an_expansion_error_still_suppresses_the_section() {
    let root = shop(
        "an_expansion_error_still_suppresses_the_section",
        &["secret"],
    );
    full_checkout(&root, &["billing", "secret"]);
    absent(&root, "secret");
    absent(&root, "billing");

    let output = init_at(&root);

    assert_eq!(events(&output)[0], "updated AGENTS.md");
    assert_eq!(members_section(&read(&root.join("AGENTS.md"))), None);
    assert!(output.notes.is_empty(), "{:?}", output.notes);
}

/// §FS-init.2.3.4.15.8: the splice keeps nothing where the file has no managed
/// block, or its block no member list — the section is then rendered.
#[test]
fn the_splice_keeps_nothing_without_a_section_on_disk() {
    let render = "<!-- BEGIN GRUND MANAGED BLOCK -->\n## Grounding with grund (block v43)\n\n\
                  ### Workspace members\n\n- [`billing`](packages/billing/AGENTS.md)\n\n\
                  ### Rules\n\nRule.\n<!-- END GRUND MANAGED BLOCK -->\n";
    assert_eq!(keep_workspace_members("# Notes\n", render), None);
    let without = render.replace(
        "### Workspace members\n\n- [`billing`](packages/billing/AGENTS.md)\n\n",
        "",
    );
    assert_eq!(keep_workspace_members(&without, render), None);
    let kept = render.replace("`billing`", "`secret`");
    assert_eq!(keep_workspace_members(&kept, render), Some(kept.clone()));
}
