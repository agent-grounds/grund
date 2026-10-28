//! Test module: where a row's guidance lands under scoped guidance
//! (§FS-init.2.3.5.11) and what each destination's block renders
//! (§FS-init.2.3.5.12): the deepest common directory over a row's places, the
//! rows that stay at the root, and the grounding sentence each destination gets.

use super::scoped_guidance::destination;
use super::*;
use crate::config::{Config, load_config};
use crate::testing::{test_root, write};

/// Load a version-2 config from its tables, written into a fresh test root.
fn v2(name: &str, tables: &str) -> Config {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        &format!("grund_config_version = 2\nproject_name = \"t\"\n{tables}"),
    );
    load_config(&root).expect("load config")
}

/// The destination of `row` in `config`.
fn dest(config: &Config, row: &str) -> String {
    let row = config
        .schema()
        .rows
        .iter()
        .find(|candidate| candidate.name == row)
        .expect("row");
    destination(row)
}

/// The layout of `config`, with every directory existing unless `missing`
/// names it.
fn layout(config: &Config, missing: &[&str]) -> Layout {
    Layout::of(config.project(), config.run(), &[], &|dir| {
        !missing.contains(&dir)
    })
}

/// §FS-init.2.3.5.11.1: siblings share their parent, a folder nested in another
/// lands in the outer one, a file counts as its parent, and a file at the root
/// or places sharing no leading component land at the root.
#[test]
fn a_rows_destination_is_the_deepest_directory_holding_its_places() {
    let config = v2(
        "scoped_destination_cases",
        r#"[schema.kinds.SIB]
folders = ["docs/spec/core", "docs/spec/plugins"]
[schema.kinds.NEST]
folders = ["docs/arch", "docs/arch/deep"]
[schema.kinds.MIX]
folders = ["docs/df/core"]
files = ["docs/df.md"]
[schema.kinds.ROOTFILE]
files = ["ROADMAP.md"]
[schema.kinds.APART]
folders = ["docs/a", "tests/b"]
"#,
    );
    assert_eq!(dest(&config, "SIB"), "docs/spec");
    assert_eq!(dest(&config, "NEST"), "docs/arch");
    assert_eq!(dest(&config, "MIX"), "docs");
    assert_eq!(dest(&config, "ROOTFILE"), "");
    assert_eq!(dest(&config, "APART"), "");
}

/// §FS-init.2.3.5.11.2-.4: a kind with no place stays at the root, a row that
/// renders nothing has no destination, and a destination directory that does
/// not exist renders at the root.
#[test]
fn only_a_row_that_renders_something_lands_and_only_where_its_directory_exists() {
    let config = v2(
        "scoped_layout_rows",
        r#"[schema.kinds.GOAL]
files = ["docs/goals.md"]
[schema.kinds.FS]
folders = ["docs/spec"]
[schema.kinds.AR]
folders = ["docs/arch"]
[schema.kinds.ADR]
[rules.citations.FS]
must = ["GOAL"]
[rules.citations.AR]
must = ["FS"]
[rules.citations.ADR]
must = ["FS"]
"#,
    );
    let scoped = layout(&config, &["docs/arch"]);
    assert_eq!(scoped.scoped_dirs().collect::<Vec<_>>(), ["docs/spec"]);
    assert!(scoped.lands("FS", "docs/spec"));
    // GOAL renders nothing; AR's directory is missing; ADR has no place.
    assert!(scoped.lands("GOAL", "") && scoped.lands("AR", "") && scoped.lands("ADR", ""));
    let pointer = scoped.pointer().expect("a row lands outside the root");
    assert!(
        pointer.ends_with("- `docs/spec/AGENTS.md`: FS\n"),
        "{pointer}"
    );
    let root = scoped
        .root_citation_directions(config.project(), config.run())
        .expect("the config declares rules");
    assert!(
        root.contains("**AR**") && root.contains("**ADR**"),
        "{root}"
    );
    assert!(!root.contains("**FS**"), "{root}");
}

/// §FS-init.2.3.5.11.5: `rules = "root"` and every version-1 config place every
/// row at the root, so no scoped file and no pointer render.
#[test]
fn rules_root_and_version_one_render_everything_at_the_root() {
    let rooted = v2(
        "scoped_rules_root",
        "[schema.kinds.FS]\nfolders = [\"docs/spec\"]\n[rules.citations.FS]\nmust = [\"FS\"]\n[presentation]\nrules = \"root\"\n",
    );
    assert!(!layout(&rooted, &[]).is_scoped());
    let root = test_root("scoped_version_one");
    write(
        &root.join("grund.toml"),
        "[[kinds]]\nkind = \"FS\"\nfolder = \"docs/spec\"\n[citations.FS]\nmust = [\"FS\"]\n",
    );
    let v1 = load_config(&root).expect("load v1");
    let v1_layout = layout(&v1, &[]);
    assert!(!v1_layout.is_scoped() && v1_layout.pointer().is_none());
    assert_eq!(
        v1_layout.root_citation_directions(v1.project(), v1.run()),
        citation_directions_section(v1.project(), v1.run()),
    );
}

/// §FS-init.2.3.5.7.1: each destination's grounding sentence names only the
/// clauses that land there — a home's clause in its scoped block, the
/// source-file clause in the complement's.
#[test]
fn each_destination_renders_its_own_grounding_clause() {
    let config = v2(
        "scoped_grounding_sentence",
        r#"[schema.kinds.FS]
folders = ["docs/spec"]
[schema.kinds.test]
citable = false
folders = ["tests/e2e", "tests/integration"]
[rules.citations.test]
must = ["FS"]
[rules.citations.grounding]
must = "file"
"#,
    );
    let scoped = layout(&config, &[]);
    let (project, run) = (config.project(), config.run());
    let root = scoped
        .root_citation_directions(project, run)
        .expect("the config declares rules");
    assert!(
        root.contains(
            "(`grund check --suggestions`). Every source file must cite a declared ID or declare one inline.\n"
        ),
        "{root}"
    );
    let block = scoped.scoped_block(project, run, "tests", 21, None);
    assert!(
        block.contains(
            "(`grund check --suggestions`). Every file under tests/e2e/, tests/integration/ must cite one.\n"
        ),
        "{block}"
    );
    assert!(block.contains("- Each file in **tests/e2e/, tests/integration/** must cite FS.\n"));
    assert!(!block.contains("### Declaration shapes") && !block.contains("### Scoped guidance"));
}

/// §FS-init.2.3.5.12.2: a kind's shape renders in the words of `grund schema`'s
/// field lines, in its destination's block, and a version-1 config renders none.
#[test]
fn a_kinds_shape_renders_in_its_destination_in_the_schema_views_words() {
    let config = v2(
        "scoped_declaration_shape",
        r#"[schema.kinds.DF]
folders = ["docs/df"]
closed = "must"
[schema.kinds.DF.fields.status]
one_of = ["Proposed", "Accepted"]
presence = "must"
position = "first"
"#,
    );
    let scoped = layout(&config, &[]);
    assert_eq!(scoped.scoped_dirs().collect::<Vec<_>>(), ["docs/df"]);
    assert_eq!(scoped.declaration_shapes(config.project(), ""), None);
    assert_eq!(
        scoped
            .declaration_shapes(config.project(), "docs/df")
            .as_deref(),
        Some(
            "### Declaration shapes\n\n`grund check` holds each declaration of these kinds to these fields, as `grund schema` lists them:\n\n- **DF**\n  - field status must · first · one of Proposed, Accepted\n  - closed must\n"
        )
    );
}

/// §FS-init.2.3.5.12.1, §FS-init.2.3.5.13: the complement's scoped block carries
/// the pointer too, so a source declaration's guidance names where its kind's
/// shape lives.
#[test]
fn the_complements_scoped_block_carries_the_pointer() {
    let config = v2(
        "scoped_complement_pointer",
        r#"[schema.kinds.FS]
folders = ["docs/spec"]
[schema.kinds.code]
folders = ["src/app", "src/lib"]
[rules.citations.FS]
must = ["FS"]
[rules.citations.code]
must = ["FS"]
"#,
    );
    let scoped = layout(&config, &[]);
    assert_eq!(
        scoped.scoped_dirs().collect::<Vec<_>>(),
        ["docs/spec", "src"]
    );
    let (project, run) = (config.project(), config.run());
    let src = scoped.scoped_block(project, run, "src", 21, None);
    assert!(
        src.ends_with("- `src/AGENTS.md`: code\n<!-- END GRUND MANAGED BLOCK -->\n"),
        "{src}"
    );
    let spec = scoped.scoped_block(project, run, "docs/spec", 21, None);
    assert!(!spec.contains("### Scoped guidance"), "{spec}");
}
