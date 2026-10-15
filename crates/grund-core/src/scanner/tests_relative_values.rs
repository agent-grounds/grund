//! Test module: value roots and components written in the relative form, read at
//! the canonical path the record holds for each (§FS-values.2.4, §FS-values.2.5).

use crate::config::load_config;
use crate::model::{Catalog, Declaration, ValueRootOrigin};
use crate::testing::{scan_findings, test_root, write};

/// `DOC-offer` is `body` under its H1 heading, in a project whose `DOC` kind
/// declares the `values` chapter and whose `CONST` kind holds whole values.
fn scan(name: &str, body: &str) -> Catalog {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n[reference]\nstrict = true\n\
         require_grounding = false\n\n[id]\nformat = \"{kind}-{slug}\"\n\
         named_sections = true\n\n[[kinds]]\nkind = \"CONST\"\nfolder = \"values\"\n\
         index = false\nvalues = true\n\n[[kinds]]\nkind = \"DOC\"\nfolder = \"docs\"\n\
         index = false\nvalue_chapter = \"values\"\n\n[scan]\ninclude = [\"docs\", \"values\"]\n",
    );
    write(
        &root.join("docs/offer.md"),
        &format!("# DOC-offer: Offer\n\n{body}"),
    );
    write(
        &root.join("values/price.md"),
        "# CONST-price: Price\n\n## .1. 1200\n## .2. USD\n",
    );
    let config = load_config(&root).expect("load config");
    scan_findings(&config, &root)
}

fn declaration<'a>(findings: &'a Catalog, kind: &str) -> &'a Declaration {
    findings
        .declarations
        .values()
        .flatten()
        .find(|decl| decl.id.kind == kind)
        .expect("declared")
}

/// Every component value of the one declaration of `kind` as `path=value@column`.
fn values(findings: &Catalog, kind: &str) -> Vec<String> {
    let decl = declaration(findings, kind);
    decl.sections
        .iter()
        .filter_map(|(path, info)| {
            let value = info.value.as_ref()?;
            Some(format!("{path}={}@{}", value.decoded, value.column))
        })
        .collect()
}

/// Every invalid value site as `line:column: message`, in line order.
fn invalid(findings: &Catalog) -> Vec<String> {
    let mut sites = findings
        .invalid_value_declarations
        .iter()
        .collect::<Vec<_>>();
    sites.sort_by_key(|site| (site.line, site.column));
    sites
        .into_iter()
        .map(|site| format!("{}:{:?}: {}", site.line, site.column, site.message))
        .collect()
}

/// §FS-values.2.4.3: `### .1. 45` under a marked root is the component `1.1`,
/// and `#### .1: 24` under a relative chapter's relative root is
/// `values.aux-voltage.1` (§FS-values.2.5); each value's column is on the line as
/// written. A whole declaration's relative components are its `.1` and `.2`.
#[test]
fn relative_components_and_a_relative_chapter_hold_their_values() {
    let findings = scan(
        "relative_value_components",
        "## 1. Regional floor <!-- grund:value -->\n### .1. 45\n\n\
         ## .values: Values\n\n### .aux-voltage: Auxiliary supply voltage\n\n\
         #### .1: 24\n#### .2: V\n",
    );
    assert_eq!(invalid(&findings), Vec::<String>::new());
    assert_eq!(
        values(&findings, "DOC"),
        [
            "1.1=45@9",
            "values.aux-voltage.1=24@10",
            "values.aux-voltage.2=V@10"
        ]
    );
    let roots = declaration(&findings, "DOC")
        .sections
        .iter()
        .filter_map(|(path, info)| Some((path.as_str(), info.value_root.as_ref()?)))
        .map(|(path, root)| (path, root.valid, root.origin))
        .collect::<Vec<_>>();
    assert_eq!(
        roots,
        [
            ("1", true, ValueRootOrigin::Marker { column: 22 }),
            ("values.aux-voltage", true, ValueRootOrigin::Chapter),
        ]
    );
    assert_eq!(values(&findings, "CONST"), ["1=1200@8", "2=USD@8"]);
}

/// §FS-values.2.4: a relative numeric heading carries the marker as its complete
/// form does, and a relative component of it is judged in physical order.
#[test]
fn a_relative_heading_is_a_marked_root() {
    let findings = scan(
        "relative_marked_root",
        "## .1. Regional floor <!-- grund:value -->\n### .2. 45\n",
    );
    assert_eq!(
        invalid(&findings),
        [
            "4:None: embedded value component coordinate must follow physical order \
          (expected `1.1`)"
        ]
    );
    let root = declaration(&findings, "DOC").sections["1"]
        .value_root
        .as_ref()
        .expect("a marked root");
    assert_eq!(
        (root.valid, root.origin),
        (false, ValueRootOrigin::Marker { column: 23 })
    );
}

/// §FS-values.2.4.4: a marker on a relative named heading, or on a relative
/// heading no parent takes, grants nothing and is reported at the marker; a
/// relative marked root that repeats a path is reported as a duplicate there.
#[test]
fn a_marker_no_relative_root_takes_is_reported_at_the_marker() {
    let findings = scan(
        "relative_misplaced_marker",
        "## .notes: Notes <!-- grund:value -->\n\n#### .1. Deep <!-- grund:value -->\n\n\
         ## 2. Floor <!-- grund:value -->\n### .1. 45\n\
         ## .2. Again <!-- grund:value -->\n### .1. 46\n",
    );
    let numeric = "embedded value marker must be on a citable numeric section heading";
    assert_eq!(
        invalid(&findings)
            .into_iter()
            .filter(|site| !site.contains("component"))
            .collect::<Vec<_>>(),
        [
            format!("3:Some(18): {numeric}"),
            format!("5:Some(15): {numeric}"),
            "9:Some(14): embedded value root coordinate is duplicated".to_string(),
        ]
    );
}

/// §FS-declarations.section.relative.where, §FS-values.2.4: a marked relative
/// heading in a doc-comment is the root its complete form and the Markdown
/// relative form are, and holds the same component.
#[test]
fn a_doc_comment_relative_root_is_judged_as_its_complete_and_markdown_forms() {
    let root = test_root("relative_value_doc_comment");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n[reference]\nstrict = true\n\
         require_grounding = false\n\n[id]\nformat = \"{kind}-{slug}\"\n\n\
         [[kinds]]\nkind = \"DOC\"\nfolder = \"docs\"\nindex = false\n\n\
         [scan]\ninclude = [\"docs\", \"src\"]\n",
    );
    let body = |path: &str, prefix: &str| {
        let complete = path.is_empty();
        let (root, leaf) = if complete {
            ("1.1.", "1.1.1.")
        } else {
            (".1.", ".1.")
        };
        format!(
            "{prefix}## 1. Overview\n{prefix}\n{prefix}### {root} Floor <!-- grund:value -->\n\
             {prefix}\n{prefix}#### {leaf} 45\n"
        )
    };
    write(
        &root.join("src/a.rs"),
        &format!(
            "/// DOC-rel: Relative\n///\n{}pub fn a() {{}}\n\n\
             /// DOC-abs: Complete\n///\n{}pub fn b() {{}}\n",
            body("rel", "/// "),
            body("", "/// "),
        ),
    );
    write(
        &root.join("docs/m.md"),
        &format!("# DOC-md: Markdown\n\n{}", body("rel", "")),
    );
    let config = load_config(&root).expect("load config");
    let findings = scan_findings(&config, &root);
    assert_eq!(invalid(&findings), Vec::<String>::new());
    for slug in ["rel", "abs", "md"] {
        let decl = findings
            .declarations
            .values()
            .flatten()
            .find(|decl| decl.id.slug.as_deref() == Some(slug))
            .expect("declared");
        let held = decl
            .sections
            .iter()
            .filter_map(|(path, info)| Some(format!("{path}={}", info.value.as_ref()?.decoded)))
            .collect::<Vec<_>>();
        assert_eq!(held, ["1.1.1=45"], "DOC-{slug}");
    }
}
