//! Test module: the template a `grund init` run lays (§FS-init.1.6) — the two
//! built-ins and their refusals, the note an explicit template earns over an
//! existing config, and what the `none` template's run writes, renders and
//! advises (§FS-init.2.2.2.2, §FS-init.2.3.4.4.2, §FS-init.2.4.9).

use std::fs;
use std::path::{Path, PathBuf};

use super::init_template::resolve_template;
use super::{InitError, InitOpts, InitOutput, init};
use crate::templates::{Template, render_grund_toml};
use crate::testing::{test_root, write};

fn run(root: &Path, template: Option<&str>, docs: bool) -> Result<InitOutput, InitError> {
    init(InitOpts {
        target: root.to_path_buf(),
        name: Some("demo".to_string()),
        template: template.map(str::to_string),
        docs,
        // §FS-init.1.2.3: a bare temp root no VCS marker covers.
        no_vcs: true,
        ..InitOpts::default()
    })
}

/// Every file under `root`, relative and sorted, with its bytes.
fn tree(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) {
        for entry in fs::read_dir(dir).expect("read dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                let rel = path.strip_prefix(root).expect("under root").to_path_buf();
                out.push((rel, fs::read(&path).expect("read file")));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}

/// §FS-init.1.6.1, §FS-init.1.6.3, §FS-init.1.6.4: no name is the default, the
/// two built-ins resolve by name, and the refusals name what is built in.
#[test]
fn a_template_name_resolves_or_is_refused() {
    assert_eq!(
        resolve_template(None, true),
        Ok(Template::SoftwareDevelopment)
    );
    assert_eq!(
        resolve_template(Some("software-development"), true),
        Ok(Template::SoftwareDevelopment)
    );
    assert_eq!(resolve_template(Some("none"), false), Ok(Template::None));
    assert_eq!(
        resolve_template(Some("docs"), false),
        Err("unknown template `docs` — the built-in templates are software-development and none; a template from a path is not supported yet".to_string())
    );
    assert_eq!(
        resolve_template(Some("none"), true),
        Err("--scaffold has nothing to lay: the none template has no starter files".to_string())
    );
}

/// §FS-init.1.6.3, §FS-init.1.6.4: a refused template writes nothing at all.
#[test]
fn a_refused_template_writes_nothing() {
    for (case, template, docs) in [("unknown", "docs", false), ("none_scaffold", "none", true)] {
        let root = test_root(&format!("init_template_refused_{case}"));
        let err = run(&root, Some(template), docs).expect_err("refused");
        assert!(
            err.output.events.is_empty(),
            "{case}: {:?}",
            err.output.events
        );
        assert!(tree(&root).is_empty(), "{case}: {:?}", tree(&root));
    }
}

/// §FS-init.1.6.1: naming the default lays the tree no name lays, starter files
/// included.
#[test]
fn naming_the_default_template_lays_the_same_tree() {
    let bare = test_root("init_template_default_unnamed");
    let named = test_root("init_template_default_named");
    run(&bare, None, true).expect("init");
    run(&named, Some("software-development"), true).expect("init");
    assert_eq!(tree(&bare), tree(&named));
}

/// §FS-init.1.6.2, §FS-init.2.4.9, §FS-init.3.6: `none` writes its config and the
/// entrypoint, and nothing in a home; the guidance teaches the first kind.
#[test]
fn the_none_template_writes_a_config_with_no_kinds() {
    let root = test_root("init_template_none");
    let output = run(&root, Some("none"), false).expect("init");
    let files = tree(&root)
        .into_iter()
        .map(|(path, _)| path)
        .collect::<Vec<_>>();
    assert_eq!(
        files,
        vec![PathBuf::from("AGENTS.md"), PathBuf::from("grund.toml")]
    );
    assert_eq!(
        fs::read_to_string(root.join("grund.toml")).expect("config"),
        render_grund_toml(Template::None, "demo", None)
    );
    let next = output.next.expect("next guidance");
    assert!(!next.declares_kinds);
    assert_eq!(
        next.render(),
        "\nnext:\n  1. declare a first kind in grund.toml, then run `grund init` again so AGENTS.md lists it\n  2. run `grund check`\n  3. allocate an ID:  ID=$(grund id <KIND> \"…\")  then add it to that kind's home\nsee AGENTS.md for the full workflow.\n"
    );
    assert!(output.notes.is_empty(), "{:?}", output.notes);
}

/// §FS-init.2.3.4.4.2: the block a config with no kinds renders names no empty
/// set, and teaches the first `[schema.kinds.<KIND>]` table of the v2 file
/// `none` writes instead (§FS-init.2.4.9); it declares no rule, so it has no
/// Citation directions section (§FS-init.2.3.4.10).
#[test]
fn the_none_template_block_teaches_the_first_kind() {
    let root = test_root("init_template_none_block");
    run(&root, Some("none"), false).expect("init");
    let block = fs::read_to_string(root.join("AGENTS.md")).expect("entrypoint");
    assert!(
        block.contains("[.<section>]` (no `KIND` is declared yet), cited"),
        "{block}"
    );
    assert!(!block.contains("KIND ∈ {}"), "{block}");
    assert!(
        block.contains(
            "### Project map\n\nNo kinds are declared yet. Add one `[schema.kinds.<KIND>]` table"
        ),
        "{block}"
    );
    assert!(!block.contains("### Citation directions"), "{block}");
    assert!(!block.contains("No kind is declared yet"), "{block}");
}

/// §FS-init.1.6.5: an explicit template over an existing config keeps the config
/// and says so; a run that named no template gets no note.
#[test]
fn an_explicit_template_over_a_config_earns_a_note() {
    let root = test_root("init_template_existing_config");
    write(&root.join("grund.toml"), "grund_config_version = 1\n");
    let named = run(&root, Some("none"), false).expect("init");
    assert_eq!(
        named.notes,
        vec!["grund.toml exists, so the none template's grund.toml is not written — init never overwrites a config".to_string()]
    );
    assert_eq!(
        fs::read_to_string(root.join("grund.toml")).expect("config"),
        "grund_config_version = 1\n"
    );
    let unnamed = run(&root, None, false).expect("init");
    assert!(unnamed.notes.is_empty(), "{:?}", unnamed.notes);
}
