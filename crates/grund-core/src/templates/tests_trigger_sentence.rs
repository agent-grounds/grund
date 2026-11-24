//! Test module: the managed block's trigger sentence names the ID that follows
//! the trigger (§FS-init.2.3.8), because a grund-aware editor converts the
//! trigger only once a token with an ID's shape follows it (§FS-lsp.1.4). The
//! configs are agent-grounds/grund#1370's and two that move the example.

use crate::config::load_config;
use crate::templates::ConversationSurface;
use crate::testing::{test_root, write};
use crate::writers::render_agents_append_block_at;

/// The managed block `init` renders for `config` written into a fresh root.
fn block(name: &str, config: &str) -> String {
    let root = test_root(name);
    write(&root.join("grund.toml"), config);
    let config = load_config(&root).expect("load config");
    render_agents_append_block_at("demo", &config, &root, true, ConversationSurface::Plain)
}

/// The one sentence of `block` that carries `example`, the trigger written
/// straight before the example ID in one code span; it panics where none does.
fn trigger_sentence<'a>(block: &'a str, example: &str) -> &'a str {
    let code = format!("`{example}`");
    block
        .lines()
        .flat_map(|line| line.split_inclusive(". "))
        .find(|sentence| sentence.contains(&code))
        .unwrap_or_else(|| panic!("no sentence names the trigger example {code}:\n{block}"))
}

/// §FS-init.2.3.8: the sentence names `example`, says the trigger becomes
/// `marker`, and never says the ID has to be recognized or resolve.
fn assert_names_the_following_id(block: &str, example: &str, marker: &str) {
    let sentence = trigger_sentence(block, example);
    assert!(
        sentence.contains(&format!("`{marker}`")),
        "the sentence says what the trigger becomes: {sentence}"
    );
    let lower = sentence.to_lowercase();
    for word in ["recogni", "resolv"] {
        assert!(
            !lower.contains(word),
            "an unresolved ID converts too, so `{word}` misleads: {sentence}"
        );
    }
}

/// §FS-init.2.3.8: agent-grounds/grund#1370's project — one kind `FS`, format
/// `{kind}-{slug}`, the default trigger and marker — no longer reads that a bare
/// `$$` becomes `§`, and names `$$FS-user-login` instead.
#[test]
fn the_trigger_sentence_names_a_following_id_of_the_projects_kind() {
    let block = block(
        "trigger_sentence_one_fs_kind",
        "grund_config_version = 2\nproject_name = \"demo\"\n\n\
         [schema]\nid_format = \"{kind}-{slug}\"\n\n\
         [schema.kinds.FS]\nfiles = [\"requirements.md\"]\n",
    );
    assert!(
        !block.contains("Type `$$` in a grund-aware editor and it becomes `§`."),
        "the block promises that a bare trigger converts:\n{block}"
    );
    assert_names_the_following_id(&block, "$$FS-user-login", "§");
}

/// §FS-init.2.3.8: the example is the guided kind's *effective* format under
/// the configured trigger, so it converts in the host repo: a lone `REQ` row
/// with its own numbered format and the trigger `%%` gives `%%REQ-042-user-login`.
#[test]
fn the_trigger_example_follows_the_guided_kinds_effective_format_and_trigger() {
    let block = block(
        "trigger_sentence_numbered_req_kind",
        "grund_config_version = 2\nproject_name = \"demo\"\n\n\
         [presentation]\ntrigger = \"%%\"\n\n\
         [schema]\nid_format = \"{kind}-{slug}\"\n\n\
         [schema.kinds.REQ]\nfiles = [\"requirements.md\"]\n\
         id_format = \"{kind}-{number}-{slug}\"\n",
    );
    assert_names_the_following_id(&block, "%%REQ-042-user-login", "§");
}

/// §FS-init.2.3.8: with no citable kind the format is filled with `FS`, as
/// the worked citation example's is.
#[test]
fn with_no_citable_kind_the_trigger_example_falls_back_to_fs() {
    let block = block(
        "trigger_sentence_no_citable_kind",
        "grund_config_version = 2\nproject_name = \"demo\"\n\n\
         [schema]\nid_format = \"{kind}-{slug}\"\n\n\
         [schema.kinds.skill]\nfolders = [\"skills\"]\ncitable = false\n",
    );
    assert_names_the_following_id(&block, "$$FS-user-login", "§");
}

/// §FS-init.2.3.8: under v1 `[reference] strict = false` a bare ID-shaped
/// token is a citation, so the sentence names a following ID with no example,
/// and the fresh block stays clean (§FS-init.2.3.8).
#[test]
fn in_non_strict_mode_the_trigger_sentence_carries_no_example() {
    let block = block(
        "trigger_sentence_non_strict",
        "grund_config_version = 1\nproject_name = \"demo\"\n\n\
         [reference]\nmarker = \"§\"\nstrict = false\n\n\
         [id]\nformat = \"{kind}-{number}-{slug}\"\n",
    );
    let sentence = block
        .lines()
        .flat_map(|line| line.split_inclusive(". "))
        .find(|sentence| sentence.contains("grund-aware editor"))
        .unwrap_or_else(|| panic!("no trigger sentence:\n{block}"));
    assert!(
        sentence.contains("Type `$$` followed by an ID in a grund-aware editor;"),
        "the sentence names a following ID: {sentence}"
    );
    assert!(
        sentence.contains("`§`"),
        "it says what the trigger becomes: {sentence}"
    );
    for absent in ["e.g.", "user-login", "<KIND>"] {
        assert!(
            !sentence.contains(absent),
            "no example in non-strict mode: {sentence}"
        );
    }
    let lower = sentence.to_lowercase();
    for word in ["recogni", "resolv"] {
        assert!(!lower.contains(word), "`{word}` misleads: {sentence}");
    }
}
