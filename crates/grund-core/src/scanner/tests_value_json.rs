//! JSON value-home shape, order, and duplicate ownership (§FS-values.2.2,
//! §FS-values.2.3).

use std::path::{Path, PathBuf};

use super::scan_tree;
use crate::config::load_config;
use crate::model::Findings;
use crate::testing::{test_root, write};

fn value_root(name: &str, kinds: &str) -> PathBuf {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 1\n\n\
             [reference]\nstrict = false\n\n\
             [id]\nformat = \"{{kind}}-{{slug}}\"\nslug_pattern = \"[a-z][a-z-]*\"\n\n\
             {kinds}\n\
             [scan]\ninclude = [\"docs\"]\n"
        ),
    );
    root
}

fn one_kind(name: &str) -> PathBuf {
    value_root(
        name,
        "[[kinds]]\nkind = \"CONST\"\nfolder = \"values\"\nindex = false\nvalues = true\n\n",
    )
}

fn scan(root: &Path) -> Findings {
    let config = load_config(root).expect("load value config");
    scan_tree(&config, Some(root), true)
        .expect("scan value fixture")
        .0
}

fn declarations<'a>(findings: &'a Findings, kind: &str, slug: &str) -> &'a [crate::Declaration] {
    findings
        .declarations
        .iter()
        .find(|(id, _)| id.kind == kind && id.slug.as_deref() == Some(slug))
        .map(|(_, declarations)| declarations.as_slice())
        .unwrap_or(&[])
}

#[test]
fn json_values_preserve_scalar_kind_spelling_and_member_order() {
    let root = one_kind("json_values_preserve_scalar_kind_spelling_and_member_order");
    write(
        &root.join("values/catalog.json"),
        "{\n  \"CONST-price\": [1.20e3, \"USD\"],\n  \"CONST-region\": [\"CH\"]\n}\n",
    );
    let findings = scan(&root);
    let price = &declarations(&findings, "CONST", "price")[0];
    assert_eq!(
        price
            .sections
            .values()
            .map(|section| section
                .value
                .as_ref()
                .map(|value| value.source_slice.as_str()))
            .collect::<Vec<_>>(),
        [Some("1.20e3"), Some("\"USD\"")]
    );
    assert_eq!(
        findings
            .declarations
            .keys()
            .filter_map(|id| id.slug.as_deref())
            .collect::<Vec<_>>(),
        ["price", "region"]
    );
}

#[test]
fn json_values_reject_root_key_and_element_shape_independently() {
    let root = one_kind("json_values_reject_root_key_and_element_shape_independently");
    write(&root.join("values/a-root.json"), "[]\n");
    write(&root.join("values/b-key.json"), "{\"FS-wrong\":[1]}\n");
    write(
        &root.join("values/c-element.json"),
        "{\"CONST-bad\":[true]}\n",
    );
    let findings = scan(&root);
    assert_eq!(
        findings
            .invalid_value_declarations
            .iter()
            .map(|site| site.message.as_str())
            .collect::<Vec<_>>(),
        [
            "value JSON root must be an object",
            "JSON value key must belong to owning kind `CONST`",
            "JSON value component must be a number or a nonempty edge-unspaced string without backticks or control characters",
        ]
    );
}

#[test]
fn repeated_and_cross_file_json_keys_remain_duplicate_declarations() {
    let repeated = one_kind("json_value_duplicate_key");
    write(
        &repeated.join("values/catalog.json"),
        "{\"CONST-price\":[1],\"CONST-price\":[1]}\n",
    );
    assert_eq!(declarations(&scan(&repeated), "CONST", "price").len(), 2);

    let cross_file = one_kind("json_value_cross_file_duplicate");
    write(&cross_file.join("values/a.json"), "{\"CONST-price\":[1]}\n");
    write(&cross_file.join("values/b.json"), "{\"CONST-price\":[1]}\n");
    assert_eq!(declarations(&scan(&cross_file), "CONST", "price").len(), 2);
}

#[test]
fn markdown_json_and_shared_home_claims_remain_duplicate_declarations() {
    let mixed = one_kind("markdown_json_value_duplicate");
    write(&mixed.join("values/a.json"), "{\"CONST-price\":[1]}\n");
    write(
        &mixed.join("values/CONST-price.md"),
        "# CONST-price: Price\n\n## 1. 1\n",
    );
    assert_eq!(declarations(&scan(&mixed), "CONST", "price").len(), 2);

    let shared = value_root(
        "shared_json_value_home_duplicate",
        "[[kinds]]\nkind = \"CONST\"\nfolder = \"values\"\nindex = false\nvalues = true\n\n\
         [[kinds]]\nkind = \"RATE\"\nfolder = \"values\"\nindex = false\nvalues = true\n\n",
    );
    write(&shared.join("values/a.json"), "{\"CONST-price\":[1]}\n");
    assert_eq!(declarations(&scan(&shared), "CONST", "price").len(), 2);
}
