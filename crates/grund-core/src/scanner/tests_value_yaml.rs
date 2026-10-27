//! A value home's YAML sources (§FS-values.2.2, §FS-values.2.2.4): which files
//! are sources in each config version, the one path order across formats, and
//! a source the window keeps from enrolling.

use std::path::{Path, PathBuf};

use crate::config::load_config;
use crate::model::{Catalog, DeclarationSource, ValueFormat, YamlDeferred};
use crate::testing::{scan_tree, test_root, write};

const KIND: &str =
    "[[kinds]]\nkind = \"CONST\"\nfolder = \"values\"\nindex = false\nvalues = true\n";

fn v1(name: &str, kinds: &str, scan: &str) -> PathBuf {
    let root = test_root(name).canonicalize().expect("canonical fixture");
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 1\n\n[reference]\nstrict = false\n\n\
             [id]\nformat = \"{{kind}}-{{slug}}\"\nslug_pattern = \"[a-z][a-z-]*\"\n\n\
             {kinds}\n[scan]\ninclude = [\"docs\"]\n{scan}"
        ),
    );
    root
}

fn v2(name: &str, sources: &str) -> PathBuf {
    let root = test_root(name).canonicalize().expect("canonical fixture");
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 2\n\
             [schema.kinds.CONST]\nfolders = [\"values\"]\nindex = false\nform = \"value\"\n\
             {sources}"
        ),
    );
    root
}

fn scan(root: &Path) -> Catalog {
    let config = load_config(root).expect("load fixture config");
    let (findings, errors) = scan_tree(&config, Some(root), true).expect("scan fixture");
    assert!(errors.is_empty(), "readable fixture: {errors:?}");
    findings
}

/// Each declared value as (slug, file name, format), in catalog order.
fn values(findings: &Catalog) -> Vec<(String, String, ValueFormat)> {
    findings
        .declarations
        .values()
        .flatten()
        .filter_map(|declaration| {
            let DeclarationSource::Json { format, .. } = &declaration.source else {
                return None;
            };
            let file = declaration.file.file_name()?.to_string_lossy().into_owned();
            Some((declaration.id.slug.clone()?, file, *format))
        })
        .collect()
}

fn slugs(findings: &Catalog) -> Vec<String> {
    values(findings)
        .into_iter()
        .map(|(slug, ..)| slug)
        .collect()
}

/// §FS-values.2.2: JSON and YAML sources at one folder home read in one path
/// order, each in its own format.
#[test]
fn a_folder_home_reads_both_formats_in_one_path_order() {
    let root = v1("yaml_value_mixed_formats", KIND, "");
    write(&root.join("values/b.yaml"), "CONST-bee: [2]\n");
    write(&root.join("values/a.json"), "{\"CONST-ant\": [1]}\n");
    write(&root.join("values/c.yml"), "CONST-cat: ['3']\n");
    write(&root.join("values/nested/d.yaml"), "CONST-dog: [4]\n");
    let mut found = values(&scan(&root));
    found.sort();
    assert_eq!(
        found,
        [
            ("ant".into(), "a.json".into(), ValueFormat::Json),
            ("bee".into(), "b.yaml".into(), ValueFormat::Yaml),
            ("cat".into(), "c.yml".into(), ValueFormat::Yaml),
        ]
    );
    let findings = scan(&root);
    let order = findings
        .yaml_value_sources
        .iter()
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert_eq!(order, ["b.yaml", "c.yml"]);
}

/// §FS-values.2.2: in version 1 the effective `[scan] extensions` claim a YAML
/// file first, whether the config writes the list or takes the default.
#[test]
fn a_v1_scanned_yaml_extension_is_no_value_source() {
    let listed = v1(
        "yaml_value_v1_listed",
        KIND,
        "extensions = [\"md\", \"yaml\"]\n",
    );
    write(&listed.join("values/a.yaml"), "CONST-ant: [1]\n");
    write(&listed.join("values/b.yml"), "CONST-bee: [2]\n");
    assert_eq!(slugs(&scan(&listed)), ["bee"]);

    let default = v1("yaml_value_v1_default", KIND, "");
    write(&default.join("values/a.yaml"), "CONST-ant: [1]\n");
    assert_eq!(slugs(&scan(&default)), ["ant"]);
}

/// §FS-config-v2.schema.sources.values: a v2 file no language claims falls back
/// on its extension; one a `data` language claims is JSON whatever its name.
#[test]
fn a_v2_yaml_file_is_read_by_its_language_or_its_extension() {
    let fallback = v2("yaml_value_v2_fallback", "");
    write(&fallback.join("values/a.yaml"), "CONST-ant: [1]\n");
    assert_eq!(
        values(&scan(&fallback)),
        [("ant".into(), "a.yaml".into(), ValueFormat::Yaml)]
    );

    let mapped = v2(
        "yaml_value_v2_data",
        "[schema.sources.extensions]\nyaml = \"json\"\n",
    );
    write(&mapped.join("values/a.yaml"), "{\"CONST-ant\": [1]}\n");
    let findings = scan(&mapped);
    assert_eq!(
        values(&findings),
        [("ant".into(), "a.yaml".into(), ValueFormat::Json)]
    );
    assert!(findings.yaml_value_sources.is_empty());
}

/// §FS-values.9: without a value kind no YAML file is opened as a value source.
#[test]
fn no_yaml_file_is_read_without_a_value_kind() {
    let root = v1(
        "yaml_value_no_value_kind",
        "[[kinds]]\nkind = \"CONST\"\nfolder = \"values\"\nindex = false\n",
        "",
    );
    write(&root.join("values/a.yaml"), "CONST-ant: [1]\n");
    let findings = scan(&root);
    assert!(slugs(&findings).is_empty());
    assert!(findings.yaml_value_sources.is_empty());
}

/// §FS-values.2.2.4: a source with any finding declares nothing, and one party
/// to a duplicate leaves the JSON declaration the only one.
#[test]
fn a_yaml_source_with_a_finding_declares_nothing() {
    let root = v1("yaml_value_window_holds_back", KIND, "");
    write(&root.join("values/a.json"), "{\"CONST-ant\": [1]}\n");
    write(
        &root.join("values/b.yaml"),
        "CONST-ant: [1]\nCONST-bee: [2]\n",
    );
    write(
        &root.join("values/c.yaml"),
        "CONST-cat: [true]\nCONST-cow: [3]\n",
    );
    write(&root.join("values/d.yaml"), "CONST-dog: [4\n");
    let findings = scan(&root);
    assert_eq!(slugs(&findings), ["ant"]);
    assert_eq!(findings.invalid_value_declarations.len(), 1);
    let deferred = findings
        .yaml_deferred
        .iter()
        .map(|record| match record {
            YamlDeferred::Duplicate { id, sites } => format!("duplicate {id:?} x{}", sites.len()),
            YamlDeferred::Unread { line, .. } => format!("unread at {line}"),
            YamlDeferred::Fetched { kind, .. } => format!("fetched {kind}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(deferred.len(), 2, "{deferred:?}");
    assert!(deferred[0].starts_with("unread at "), "{deferred:?}");
    assert!(deferred[1].starts_with("duplicate ") && deferred[1].ends_with("x2"));
}

/// §FS-values.2.2: a `file` home that is a YAML file is its kind's source, and
/// the scan does not also report it as an unread root.
#[test]
fn a_yaml_file_home_is_its_kinds_source() {
    let root = v1(
        "yaml_value_file_home",
        "[[kinds]]\nkind = \"CONST\"\nfile = \"values/limits.yaml\"\nvalues = true\n",
        "",
    );
    write(&root.join("values/limits.yaml"), "CONST-room-limit: [12]\n");
    write(&root.join("docs/README.md"), "# Docs\n");
    let config = load_config(&root).expect("load fixture config");
    let (findings, errors) = scan_tree(&config, Some(&root), true).expect("scan fixture");
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(slugs(&findings), ["room-limit"]);
}
