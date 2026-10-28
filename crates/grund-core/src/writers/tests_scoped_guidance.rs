//! Test module: the scoped files `init` owns (§FS-init.2.3.5.13.1,
//! §FS-init.2.3.5.14): the list on disk it reads them from, the byte-exact test
//! that deletes an orphan only when its block was the whole file, which files
//! of an orphan's directory lose their block, a chapter rule written into its
//! subject kind's scoped block, and the root section the swap of
//! `finish_root_block` leaves out when no rule is declared (§FS-init.2.3.4.10).

use std::fs;

use super::init::{InitOpts, init};
use super::init_scoped::cut_block;
use super::scoped_files::listed_scoped_files;
use crate::testing::{test_root, write};

const BLOCK: &str = "<!-- BEGIN GRUND MANAGED BLOCK -->\n## Grounding with grund (v21)\n\nScoped.\n<!-- END GRUND MANAGED BLOCK -->\n";

/// §FS-init.2.3.5.13.1: only a `` - `<path>`: `` line of the `### Scoped
/// guidance` section names a path, and a path that is absolute, climbs with
/// `..`, or does not end in `/AGENTS.md` is ignored.
#[test]
fn the_list_on_disk_names_only_relative_scoped_agents_files() {
    let block = "<!-- BEGIN GRUND MANAGED BLOCK -->\n## Grounding with grund (v21)\n\n### Citation directions\n\n- `outside/AGENTS.md`: FS\n\n### Scoped guidance\n\nRead it first:\n\n- `docs/spec/AGENTS.md`: FS\n- `/etc/AGENTS.md`: X\n- `docs/../../AGENTS.md`: Y\n- `docs/CLAUDE.md`: Z\n- `AGENTS.md`: W\n- `tests/AGENTS.md`: tests/e2e/\n\n### Clickable citations\n\n- `after/AGENTS.md`: V\n<!-- END GRUND MANAGED BLOCK -->\n";
    assert_eq!(
        listed_scoped_files(block),
        ["docs/spec/AGENTS.md", "tests/AGENTS.md"]
    );
    assert!(
        listed_scoped_files(
            "<!-- BEGIN GRUND MANAGED BLOCK -->\n<!-- END GRUND MANAGED BLOCK -->\n"
        )
        .is_empty()
    );
}

/// §FS-init.2.3.5.14.2: a file is deleted only when the block is the whole of
/// it, byte for byte; one byte more and the block is cut, every other byte
/// kept. A file with no block is left alone, and a dry run writes nothing.
#[test]
fn an_orphan_is_deleted_only_when_its_block_was_the_whole_file() {
    let root = test_root("scoped_orphan_cut");
    let whole = root.join("whole/AGENTS.md");
    write(&whole, BLOCK);
    assert!(matches!(cut_block(&whole, true), Ok(Some(true))));
    assert!(whole.exists(), "a dry run deletes nothing");
    assert!(matches!(cut_block(&whole, false), Ok(Some(true))));
    assert!(!whole.exists());

    let trailing = root.join("trailing/AGENTS.md");
    write(&trailing, &format!("{BLOCK}\n"));
    assert!(matches!(cut_block(&trailing, false), Ok(Some(false))));
    assert_eq!(fs::read_to_string(&trailing).unwrap(), "\n");

    let around = root.join("around/AGENTS.md");
    write(&around, &format!("# Notes\n\n{BLOCK}tail\n"));
    assert!(matches!(cut_block(&around, false), Ok(Some(false))));
    assert_eq!(fs::read_to_string(&around).unwrap(), "# Notes\n\ntail\n");

    let plain = root.join("plain/AGENTS.md");
    write(&plain, "# Notes\n");
    assert!(matches!(cut_block(&plain, false), Ok(None)));
    assert_eq!(fs::read_to_string(&plain).unwrap(), "# Notes\n");
}

/// §FS-init.2.3.5.14.3: the `AGENTS.md` and each standalone companion of an
/// orphan's directory lose their block each on its own account, whether the
/// `AGENTS.md` is there or not, and only a symlink to that `AGENTS.md` is
/// touched, never one that is itself the `AGENTS.md`.
#[cfg(unix)]
#[test]
fn an_orphans_files_lose_their_block_each_on_its_own_account() {
    use super::scoped_files::OrphanFile::{Agents, Linked, Standalone};
    use super::scoped_files::orphan_files;
    use std::os::unix::fs::symlink;
    let root = test_root("scoped_orphan_files");
    write(&root.join("both/AGENTS.md"), BLOCK);
    symlink("AGENTS.md", root.join("both/CLAUDE.md")).unwrap();
    write(&root.join("both/GEMINI.md"), BLOCK);
    assert_eq!(
        orphan_files(&root, "both"),
        [
            ("AGENTS.md", Agents),
            ("CLAUDE.md", Linked),
            ("GEMINI.md", Standalone)
        ]
    );

    write(&root.join("gone/CLAUDE.md"), BLOCK);
    symlink("AGENTS.md", root.join("gone/GEMINI.md")).unwrap();
    assert_eq!(
        orphan_files(&root, "gone"),
        [("CLAUDE.md", Standalone), ("GEMINI.md", Linked)]
    );

    write(&root.join("AGENTS.md"), BLOCK);
    write(&root.join("linked/GEMINI.md"), BLOCK);
    symlink("../AGENTS.md", root.join("linked/AGENTS.md")).unwrap();
    symlink("../AGENTS.md", root.join("linked/CLAUDE.md")).unwrap();
    assert_eq!(orphan_files(&root, "linked"), [("GEMINI.md", Standalone)]);
}

/// §FS-init.2.3.5.11, §FS-init.2.3.5.10: a chapter rule renders in the scoped
/// block of the kind it is about, not in the root block, and the run's blocks
/// carry v46.
#[test]
fn a_chapter_rule_renders_in_its_subject_kinds_scoped_block() {
    let root = test_root("scoped_chapter_rule");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\nproject_name = \"t\"\n[schema.sources]\nlanguages = [\"markdown\"]\n[schema.kinds.FS]\nfolders = [\"docs/fs\"]\nindex = false\n[schema.kinds.RULE]\nfolders = [\"docs/rules\"]\nindex = false\nform = \"rule\"\n",
    );
    write(
        &root.join("docs/rules/RULE-terms.md"),
        "# RULE-terms: Each FS must have exactly one Terms chapter.\n\nEvery FS defines its words.\n",
    );
    write(
        &root.join("docs/fs/FS-a.md"),
        "# FS-a: A spec\n\n## Terms\n\nWords.\n",
    );
    init(InitOpts {
        target: root.clone(),
        no_vcs: true,
        ..InitOpts::default()
    })
    .expect("init");
    let scoped = fs::read_to_string(root.join("docs/fs/AGENTS.md")).expect("scoped file");
    assert!(
        scoped.starts_with(
            "<!-- BEGIN GRUND MANAGED BLOCK -->\n## Grounding with grund (block v46)\n"
        )
    );
    assert!(scoped.contains("### Chapter rules\n"), "{scoped}");
    assert!(
        scoped.contains("- Each FS must have exactly one Terms chapter. "),
        "{scoped}"
    );
    let agents = fs::read_to_string(root.join("AGENTS.md")).expect("root file");
    assert!(
        agents.contains("## Grounding with grund (block v46)"),
        "{agents}"
    );
    assert!(!agents.contains("### Chapter rules"), "{agents}");
    assert!(agents.contains("- `docs/fs/AGENTS.md`: FS\n"), "{agents}");

    let again = init(InitOpts {
        target: root.clone(),
        no_vcs: true,
        ..InitOpts::default()
    })
    .expect("init again");
    assert!(
        again.events.iter().all(|event| event.verb == "exists"),
        "a second run is a no-op: {:?}",
        again.events
    );
}

/// A version-2 project `t` whose GRUND kind is `docs/grund.md`, with `rows` added
/// and `dirs` laid, so each scoped destination exists (§FS-init.2.3.5.11.4).
fn init_v2(name: &str, rows: &str, dirs: &[&str]) -> std::path::PathBuf {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 2\nproject_name = \"t\"\n[schema.kinds.GRUND]\nfiles = [\"docs/grund.md\"]\n{rows}"
        ),
    );
    write(&root.join("docs/grund.md"), "# GRUND-why: Why\n\nWhy.\n");
    for dir in dirs {
        fs::create_dir_all(root.join(dir)).expect("create place");
    }
    init(InitOpts {
        target: root.clone(),
        no_vcs: true,
        ..InitOpts::default()
    })
    .expect("init");
    root
}

/// §FS-init.2.3.4.10, §FS-init.2.3.5.12.1: with no rule and grounding on one
/// scoped home alone, the all-rows section has a grounding clause and the root's
/// has none, so the swap takes the section out with its separator: one blank
/// line stays where it stood, and the scoped block holds the grounding sentence
/// alone.
#[test]
#[ignore = "red until #1075 renders Citation directions only from declared rules; implement removes this"]
fn the_swap_takes_a_section_only_scoped_rows_ground_out_with_its_separator() {
    let root = init_v2(
        "scoped_grounding_swap",
        "[schema.kinds.e2e]\nfolders = [\"tests/e2e\"]\ncitable = false\n[rules.citations.e2e.grounding]\nmust = \"file\"\n",
        &["tests/e2e"],
    );
    let agents = fs::read_to_string(root.join("AGENTS.md")).expect("root file");
    assert!(!agents.contains("### Citation directions"), "{agents}");
    assert!(!agents.contains("\n\n\n"), "{agents}");
    assert!(
        agents.contains("- **Always cite the most-specific point.**\n\n### Scoped guidance\n"),
        "{agents}"
    );
    let scoped = fs::read_to_string(root.join("tests/e2e/AGENTS.md")).expect("scoped file");
    assert!(
        scoped.ends_with(
            "\n\n### Citation directions\n\nEvery file under tests/e2e/ must cite one.\n<!-- END GRUND MANAGED BLOCK -->\n"
        ),
        "{scoped}"
    );
}

/// §FS-init.2.3.4.10: scoped only by a field, with no rule and no grounding,
/// neither block has a section, so there is nothing to swap and the root block
/// is whole from its first byte.
#[test]
#[ignore = "red until #1075 renders Citation directions only from declared rules; implement removes this"]
fn a_config_scoped_only_by_a_field_has_no_section_to_swap() {
    let root = init_v2(
        "scoped_field_swap",
        "[schema.kinds.FS]\nfolders = [\"docs/fs\"]\n[schema.kinds.FS.fields.front-matter]\npresence = \"must\"\n",
        &["docs/fs"],
    );
    let agents = fs::read_to_string(root.join("AGENTS.md")).expect("root file");
    assert!(
        agents.starts_with(
            "# t — agent instructions\n\n<!-- BEGIN GRUND MANAGED BLOCK -->\n## Grounding with grund (block v"
        ),
        "{agents}"
    );
    assert!(!agents.contains("### Citation directions"), "{agents}");
    assert!(!agents.contains("\n\n\n"), "{agents}");
    let scoped = fs::read_to_string(root.join("docs/fs/AGENTS.md")).expect("scoped file");
    assert!(scoped.contains("### Declaration shapes\n"), "{scoped}");
    assert!(!scoped.contains("### Citation directions"), "{scoped}");
}
