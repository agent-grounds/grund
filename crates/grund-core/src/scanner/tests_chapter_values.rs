//! Chapter-declared value roots: which sections the schema enrolls, what the
//! declared chapter may hold, which delimited forms become bindings, and the
//! comparison that follows (§FS-config.3.4.13, §FS-values.2.5, §FS-values.3.1,
//! §FS-values.5.1).

use std::path::PathBuf;

use crate::api::{check, lsp_snapshot, show};
use crate::config::Config;
use crate::model::{Catalog, Id, ShowRenderMode, ValueRootOrigin};
use crate::queries::{LspSnapshotOpts, ShowFormat, ShowMode, ShowOpts, show_declaration};
use crate::testing::{embedded_value_config, scan_tree, test_root, write};

/// `AR` declarations under `docs/`, with `values` as the declared chapter and
/// named sections on, which the key requires (§FS-config.3.4.13).
fn chapter_config(root: PathBuf, chapter: Option<&str>) -> Config {
    let mut config = embedded_value_config(root);
    config.named_sections = true;
    for kind in &mut config.kinds {
        if kind.kind == "AR" {
            kind.folder = Some("docs".to_string());
            kind.file = None;
            kind.value_chapter = chapter.map(str::to_string);
        }
    }
    config.rebuild_grammar().expect("rebuild named grammar");
    config
}

fn scan_chapter(name: &str, chapter: Option<&str>, source: &str) -> (Config, Catalog) {
    let root = test_root(name);
    let path = root.join("docs/power.md");
    write(&path, source);
    let config = chapter_config(root, chapter);
    let (findings, errors) = scan_tree(&config, Some(&path), true).expect("scan chapter fixture");
    assert!(errors.is_empty(), "fixture should be readable: {errors:?}");
    (config, findings)
}

fn root_origins(findings: &Catalog) -> Vec<(String, ValueRootOrigin)> {
    findings
        .declarations
        .values()
        .flatten()
        .flat_map(|decl| decl.sections.iter())
        .filter_map(|(path, info)| {
            info.value_root
                .as_ref()
                .map(|root| (path.clone(), root.origin))
        })
        .collect()
}

const PROBE: &str = "# AR-004-value-probe: Auxiliary power\n\n\
     ## values: Values\n\n\
     ### values.aux-voltage: Auxiliary supply voltage\n\n\
     #### values.aux-voltage.1: 24\n";

/// Only a named direct child of the declared chapter is enrolled, and it records
/// that the chapter rather than a marker made it (§FS-values.2.5,
/// §AR-scanner.2.2.8). The chapter heading itself is not a root, a numeric child
/// of the chapter is not one, and a same-named chapter nested deeper gains
/// nothing.
#[test]
fn only_a_named_direct_child_of_the_declared_chapter_is_enrolled() {
    let (_, findings) = scan_chapter(
        "chapter_value_enrollment",
        Some("values"),
        "# AR-004-value-probe: Auxiliary power\n\n\
         ## values: Values\n\n\
         ### values.aux-voltage: Auxiliary supply voltage\n\n\
         #### values.aux-voltage.1: 24\n\n\
         ## subsystems: Subsystems\n\n\
         ### subsystems.values: Values\n\n\
         #### subsystems.values.flow: Flow rate\n\n\
         ##### subsystems.values.flow.1: 12\n",
    );
    assert_eq!(
        root_origins(&findings),
        vec![("values.aux-voltage".to_string(), ValueRootOrigin::Chapter)],
        "the chapter heading, a deeper same-named chapter, and its children are not roots"
    );
}

/// The key alone is the authority: the identical document under a kind that
/// declares no chapter grows no root at all (§FS-values.9).
#[test]
fn without_the_key_the_same_document_grows_no_root() {
    let (_, findings) = scan_chapter("chapter_value_no_opt_in", None, PROBE);
    assert!(
        root_origins(&findings).is_empty(),
        "enrollment is gated on `value_chapter`"
    );
}

/// The marker route is untouched and keeps its column, so one declaration may
/// carry both authorities and each is recorded as the route that made it
/// (§FS-values.2.4, §AR-scanner.2.2.8).
#[test]
fn a_marked_numeric_root_keeps_its_own_origin_beside_a_chapter_root() {
    let (_, findings) = scan_chapter(
        "chapter_value_beside_marker",
        Some("values"),
        "# AR-004-value-probe: Auxiliary power\n\n\
         ## values: Values\n\n\
         ### values.aux-voltage: Auxiliary supply voltage\n\n\
         #### values.aux-voltage.1: 24\n\n\
         ## 2. Regulator <!-- grund:value -->\n\n\
         ### 2.1. 10\n",
    );
    let mut origins = root_origins(&findings);
    origins.sort_by(|left, right| left.0.cmp(&right.0));
    assert_eq!(
        origins,
        vec![
            (
                "2".to_string(),
                ValueRootOrigin::Marker {
                    column: "## 2. Regulator ".len() + 1
                }
            ),
            ("values.aux-voltage".to_string(), ValueRootOrigin::Chapter),
        ]
    );
}

/// A marked root's section-map title drops the separating space and tag, as its
/// anchor and LSP title do, while the raw heading `show` prints keeps it; a
/// chapter root has no tag to drop (§FS-values.2.4.2).
#[test]
fn a_marked_roots_map_title_drops_its_tag() {
    let (config, findings) = scan_chapter(
        "chapter_value_marked_title",
        Some("values"),
        "# AR-004-value-probe: Auxiliary power\n\n\
         ## values: Values\n\n\
         ### values.aux-voltage: Auxiliary supply voltage\n\n\
         #### values.aux-voltage.1: 24\n\n\
         ## 2. Regulator <!-- grund:value -->\n\n\
         ### 2.1. 10\n",
    );
    let id = Id {
        kind: "AR".to_string(),
        num: Some(4),
        slug: Some("value-probe".to_string()),
    };
    let toc = show_declaration(
        &config,
        &config,
        &findings,
        &id,
        None,
        ShowRenderMode::Outline,
        false,
    )
    .expect("--toc maps the declaration");
    let titles: Vec<_> = toc
        .sections
        .iter()
        .map(|section| (section.path.as_str(), section.title.as_str()))
        .collect();
    assert_eq!(
        titles,
        vec![
            ("values", "Values"),
            ("values.aux-voltage", "Auxiliary supply voltage"),
            ("values.aux-voltage.1", "24"),
            ("2", "Regulator"),
            ("2.1", "10"),
        ]
    );
    assert!(
        toc.body.contains("## 2. Regulator <!-- grund:value -->\n"),
        "the raw heading keeps its tag: {:?}",
        toc.body
    );
}

/// `show --toc` and the editor give each section one title. A chapter root's tag
/// is no mark (§FS-values.2.4.4), so both keep it; a marked root's is dropped
/// by both (§FS-values.2.4.2), even when the tag is all its title holds.
#[test]
fn toc_and_editor_title_a_tagged_chapter_root_alike() {
    let root = test_root("chapter_value_tagged_root_titles");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n[id]\nnamed_sections = true\n\
         [[kinds]]\nkind = \"AR\"\nfolder = \"docs\"\nindex = false\n\
         value_chapter = \"values\"\n[scan]\ninclude = [\"docs\"]\n",
    );
    write(
        &root.join("docs/power.md"),
        "# AR-004-probe: Power\n\n\
         ## values: Values\n\n\
         ### values.aux-voltage: Aux voltage <!-- grund:value -->\n\n\
         #### values.aux-voltage.1: 24\n\n\
         ## 1. Marked <!-- grund:value -->\n\n\
         ### 1.1 Leaf\n\n\
         ## 2. <!-- grund:value -->\n\n\
         25\n",
    );
    let opts = ShowOpts {
        path: root.clone(),
        section: None,
        mode: ShowMode::Toc,
        format: ShowFormat::Text,
    };
    let toc = show("AR-004-probe", opts).expect("--toc maps the declaration");
    let snapshot = lsp_snapshot(LspSnapshotOpts {
        path: root,
        path_provided: true,
        open_documents: Default::default(),
    })
    .expect("the editor reads the tree");
    let titles: Vec<_> = toc
        .sections
        .iter()
        .map(|section| {
            let editor = snapshot
                .sections
                .iter()
                .find(|editor| editor.query_id == format!("AR-004-probe.{}", section.path))
                .map(|editor| editor.text.as_str());
            (section.path.as_str(), section.title.as_str(), editor)
        })
        .collect();
    assert_eq!(
        titles,
        vec![
            ("values", "Values", Some("values: Values")),
            (
                "values.aux-voltage",
                "Aux voltage <!-- grund:value -->",
                Some("values.aux-voltage: Aux voltage <!-- grund:value -->"),
            ),
            (
                "values.aux-voltage.1",
                "24",
                Some("values.aux-voltage.1: 24")
            ),
            ("1", "Marked", Some("1. Marked")),
            ("1.1", "Leaf", Some("1.1 Leaf")),
            ("2", "", Some("2.")),
        ]
    );
}

/// The declared chapter holds named value-root headings and nothing else, and
/// each offending line is located on its own (§FS-values.2.5, §FS-check.3.20).
/// A root's own subtree is not reported here: the component run owns it.
#[test]
fn the_declared_chapter_holds_only_named_root_headings() {
    let (_, findings) = scan_chapter(
        "chapter_value_level_shape",
        Some("values"),
        "# AR-004-value-probe: Auxiliary power\n\n\
         ## values: Values\n\n\
         Chapter lead prose.\n\n\
         ### values.1: A numeric child of the chapter\n\n\
         ### values.aux-voltage: Auxiliary supply voltage\n\n\
         #### values.aux-voltage.1: 24\n",
    );
    let chapter_sites = findings
        .invalid_value_declarations
        .iter()
        .filter(|site| site.message.contains("declared value chapter"))
        .map(|site| site.line)
        .collect::<Vec<_>>();
    assert_eq!(chapter_sites, vec![5, 7]);
    assert!(
        findings.invalid_value_declarations.len() == 2,
        "the valid sibling root is untouched: {:?}",
        findings.invalid_value_declarations
    );
}

/// A chapter root has no marker, so a root-level failure is located at the root
/// heading and carries no column (§FS-values.2.4.3).
#[test]
fn a_rootless_chapter_root_is_reported_at_its_own_heading() {
    let (_, findings) = scan_chapter(
        "chapter_value_zero_components",
        Some("values"),
        "# AR-004-value-probe: Auxiliary power\n\n\
         ## values: Values\n\n\
         ### values.aux-voltage: Auxiliary supply voltage\n",
    );
    let site = findings
        .invalid_value_declarations
        .iter()
        .find(|site| site.message.contains("contiguous numbered components"))
        .expect("a chapter root with no components is invalid");
    assert_eq!((site.line, site.column), (5, None));
}

/// A repository on disk whose `AR` kind declares `values` as its value chapter
/// — what the checker and the formatter are asked about, since both load the
/// config themselves.
fn chapter_repo(name: &str, body: &str) -> PathBuf {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n         [reference]\nstrict = true\n\n         [id]\nnamed_sections = true\n\n         [[kinds]]\nkind = \"AR\"\nfolder = \"docs\"\nindex = false\nvalue_chapter = \"values\"\n\n         [scan]\ninclude = [\"docs\"]\n",
    );
    write(&root.join("docs/power.md"), body);
    root
}

/// The binding grammar is a valid root path then an optional positive numeric
/// immediate coordinate: the root itself binds and agrees, the chapter heading
/// and a zero coordinate are refused, and the component compares
/// (§FS-values.3.1, §FS-values.3.1.1, §FS-values.3.1.2, §FS-values.5.1).
#[test]
fn a_chapter_binding_compares_and_its_near_misses_are_refused() {
    let root = chapter_repo(
        "chapter_value_binding_grammar",
        "# AR-004-value-probe: Auxiliary power\n\n\
         ## values: Values\n\n\
         ### values.aux-voltage: Auxiliary supply voltage\n\n\
         #### values.aux-voltage.1: 24\n\n\
         ## 2. Use\n\n\
         Mismatch: `48` (\u{a7}AR-004-value-probe.values.aux-voltage.1)\n\n\
         Root: `24` (\u{a7}AR-004-value-probe.values.aux-voltage)\n\n\
         Chapter: `24` (\u{a7}AR-004-value-probe.values)\n\n\
         Zero: `24` (\u{a7}AR-004-value-probe.values.aux-voltage.0)\n",
    );
    let report = check(&root).expect("check the chapter fixture");
    let codes = report
        .errors
        .iter()
        .map(|finding| (finding.line.unwrap_or(0), finding.code))
        .collect::<Vec<_>>();
    assert!(
        codes.contains(&(11, "value-mismatch")),
        "the exact binding compares: {codes:?}"
    );
    assert!(
        !codes.iter().any(|(line, _)| *line == 13),
        "the root binds and agrees, so line 13 is silent: {codes:?}"
    );
    for line in [15, 17] {
        assert!(
            codes.contains(&(line, "invalid-value-binding")),
            "line {line} must be an attempted binding, not prose: {codes:?}"
        );
    }
    let _ = std::fs::remove_dir_all(root);
}

/// The same delimited form aimed at a named section outside every declared
/// chapter stays ordinary prose and one ordinary citation (§FS-values.3.1.1).
#[test]
fn a_named_section_outside_every_chapter_stays_prose() {
    let root = chapter_repo(
        "chapter_value_named_prose",
        "# AR-004-value-probe: Auxiliary power\n\n\
         ## notes: Notes\n\n\
         ### notes.aux-voltage: Auxiliary supply voltage\n\n\
         #### notes.aux-voltage.1: 24\n\n\
         ## 2. Use\n\n\
         Prose: `48` (\u{a7}AR-004-value-probe.notes.aux-voltage.1)\n",
    );
    let report = check(&root).expect("check the prose fixture");
    assert!(
        report
            .errors
            .iter()
            .all(|finding| !finding.code.contains("value")),
        "nothing about values may be reported: {:?}",
        report.errors
    );
    let _ = std::fs::remove_dir_all(root);
}
