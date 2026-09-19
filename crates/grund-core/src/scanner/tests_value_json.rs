//! JSON value-home shape, order, and duplicate ownership (§FS-values.2.2,
//! §FS-values.2.3).

use std::path::{Path, PathBuf};

use super::json::{JsonNode, JsonReader};
use super::scan_tree;
use crate::config::load_config;
use crate::model::{DeclarationSource, Findings, ValueComponentKind};
use crate::testing::{test_root, write};

fn value_root(name: &str, kinds: &str) -> PathBuf {
    let root = test_root(name)
        .canonicalize()
        .expect("canonical value fixture");
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
    let (findings, errors) = scan_tree(&config, Some(root), true).expect("scan value fixture");
    assert!(errors.is_empty(), "readable value fixture: {errors:?}");
    findings
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
    let text = concat!(
        "{\n",
        "  \"CONST-region\": [\"C\\u0048\"],\n",
        "  \"CONST-price\": [\n",
        "    1.20e3,\n",
        "    \"\\u0055SD\"\n",
        "  ]\n",
        "}\n",
    );
    let file = root.join("values/catalog.json");
    write(&file, text);
    // §FS-values.2.2: inspect physical order before the lookup map can sort the keys.
    let JsonNode::Object(members, _) = JsonReader::parse(text).expect("read JSON") else {
        panic!("expected object");
    };
    assert_eq!(
        members
            .iter()
            .map(|member| member.key.decoded.as_str())
            .collect::<Vec<_>>(),
        ["CONST-region", "CONST-price"]
    );
    for (member, key, source) in [
        (
            &members[0],
            "\"CONST-region\"",
            "\"CONST-region\": [\"C\\u0048\"]",
        ),
        (
            &members[1],
            "\"CONST-price\"",
            "\"CONST-price\": [\n    1.20e3,\n    \"\\u0055SD\"\n  ]",
        ),
    ] {
        assert_eq!(&text[member.key.span.start..member.key.span.end], key);
        assert_eq!(&text[member.span.start..member.span.end], source);
    }
    let findings = scan(&root);
    let price = &declarations(&findings, "CONST", "price")[0];
    let region = &declarations(&findings, "CONST", "region")[0];
    for (declaration, member, line, end_line) in
        [(region, &members[0], 2, 2), (price, &members[1], 3, 6)]
    {
        assert_eq!((&declaration.file, declaration.line), (&file, line));
        assert_eq!(
            (declaration.body_start, declaration.body_end),
            (line, end_line)
        );
        assert_eq!(declaration.value_valid, Some(true));
        let DeclarationSource::Json {
            member_slice,
            key_column,
            key_text,
        } = &declaration.source
        else {
            panic!("expected retained JSON member");
        };
        assert_eq!(*key_column, 3);
        assert_eq!(key_text, &text[member.key.span.start..member.key.span.end]);
        assert_eq!(member_slice, &text[member.span.start..member.span.end]);
    }
    for (declaration, section, kind, decoded, raw, line, column) in [
        (
            region,
            "1",
            ValueComponentKind::String,
            "CH",
            "\"C\\u0048\"",
            2,
            20,
        ),
        (
            price,
            "1",
            ValueComponentKind::Number,
            "1.20e3",
            "1.20e3",
            4,
            5,
        ),
        (
            price,
            "2",
            ValueComponentKind::String,
            "USD",
            "\"\\u0055SD\"",
            5,
            5,
        ),
    ] {
        let component = &declaration.sections[section];
        let value = component.value.as_ref().expect("valid scalar");
        assert_eq!((component.line, value.column), (line, column));
        assert_eq!(
            (
                value.kind,
                value.decoded.as_str(),
                value.source_slice.as_str()
            ),
            (kind, decoded, raw)
        );
    }
    assert!(findings.invalid_value_declarations.is_empty());
}

#[test]
fn json_values_reject_root_key_and_element_shape_independently() {
    let root = value_root(
        "json_values_reject_root_key_and_element_shape_independently",
        "[[kinds]]\nkind = \"CONST\"\nfolder = \"values\"\nindex = false\nvalues = true\n\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"docs/specs\"\nindex = false\n\n",
    );
    let cases = [
        (
            "a-root",
            "\n []\n",
            2,
            2,
            "value JSON root must be an object",
        ),
        (
            "b-invalid-id",
            "{\n  \"not-an-id\": [1]\n}\n",
            2,
            3,
            "JSON value key must be a full unqualified local ID",
        ),
        (
            "c-wrong-owner",
            "{\n  \"FS-wrong\": [1]\n}\n",
            2,
            3,
            "JSON value key must belong to owning kind `CONST`",
        ),
        (
            "d-element",
            "{\n  \"CONST-bad\": [\n    true\n  ]\n}\n",
            4,
            5,
            "JSON value component must be a number or a nonempty edge-unspaced string without backticks or control characters",
        ),
    ];
    for (name, contents, _, _, _) in cases {
        write(&root.join(format!("values/{name}.json")), contents);
    }
    let findings = scan(&root);
    assert_eq!(
        findings
            .invalid_value_declarations
            .iter()
            .map(|site| (
                site.file.clone(),
                site.line,
                site.column,
                site.message.as_str()
            ))
            .collect::<Vec<_>>(),
        cases.map(|(name, _, line, column, message)| (
            root.join(format!("values/{name}.json")),
            line,
            Some(column),
            message
        ))
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
