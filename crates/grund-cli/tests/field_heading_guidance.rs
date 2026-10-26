//! Binary regressions for the appended field repair (§FS-declarations.checks.unmarked-heading.2.2).
//! Field recognition and v1 output remain §FS-fields.identity.name and §FS-fields.v1.

use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const FILE: &str = "docs/tbl/orders.md";
const SCHEMA: &str = "[schema.kinds.TBL.fields.schema]\ntitle = \"Schema\"\npresence = \"must\"\nposition = \"first\"\n";

struct Fixture(PathBuf);

impl Fixture {
    fn new(name: &str, fields: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-999-{name}-{}", std::process::id()));
        fs::create_dir_all(root.join("docs/tbl")).unwrap();
        let fixture = Self(root);
        fixture.config(&format!(
            "grund_config_version = 2\nproject_name = \"unmarked-field\"\n\n\
             [schema.sources]\nlanguages = [\"markdown\"]\n\n\
             [schema.kinds.TBL]\nfolders = [\"docs/tbl\"]\nindex = false\n\n{fields}"
        ));
        fixture
    }

    fn config(&self, config: &str) {
        fs::write(self.0.join("grund.toml"), config).unwrap();
    }

    fn doc(&self, body: &str) {
        fs::write(self.0.join(FILE), body).unwrap();
    }

    fn check(&self, extra: &[&str], exit: i32) -> String {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .arg("check")
            .arg(&self.0)
            .args(["--only", "missing-field", "--only", "unmarked-heading"])
            .args(extra)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(exit), "{output:?}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.is_empty(), "{stderr}");
        String::from_utf8(output.stdout).unwrap()
    }

    fn json(&self) -> Vec<Value> {
        self.check(&["--format", "json"], 1)
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn prefix(marks: &str, title: &str, path: &str) -> String {
    format!(
        "unmarked heading inside TBL-orders; number it ({marks} {path}. {title}) as TBL-orders.{path}, declare an ID, or use a bold label; this became an error in grund 0.16.0"
    )
}

fn suffix(name: &str, marks: &str, title: &str) -> String {
    format!("; kind TBL declares the field {name}, written {marks} {name}: {title}")
}

fn finding(code: &str, line: u64, message: &str) -> Value {
    json!({"severity": "error", "path": FILE, "line": line, "code": code,
        "message": message, "sites": null, "authority": null})
}

fn repair_contract(name: &str, fields: &str, bare: &str, title: &str) {
    let fixture = Fixture::new(name, fields);
    let body = |heading: &str| {
        format!("# TBL-orders: Orders\n\nAn order table.\n\n{heading}\n\nThe table schema.\n")
    };
    fixture.doc(&body(&format!("## {bare}")));
    let original = fixture.check(&[], 1);
    let rows = fixture.json();
    let missing = "TBL-orders has no schema field; kind TBL declares it; add its schema section";
    let old = prefix("##", bare, "1");
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert_eq!(rows[0], finding("missing-field", 1, missing));
    let message = rows[1]["message"].as_str().unwrap();
    assert!(message.starts_with(&old), "{message}");
    assert_eq!(rows[1], finding("unmarked-heading", 5, message));
    assert_eq!(
        original,
        format!("{FILE}:1: error: {missing}\n{FILE}:5: error: {message}\n")
    );

    // The issue's two edits: numbering loses only unmarked-heading; naming fixes both.
    fixture.doc(&body(&format!("## 1. {bare}")));
    assert_eq!(fixture.json(), vec![finding("missing-field", 1, missing)]);
    fixture.doc(&body(&format!("## schema: {title}")));
    assert_eq!(fixture.check(&[], 0), "success\n");

    assert_eq!(message, format!("{old}{}", suffix("schema", "##", title)));
}

#[test]
fn field_heading_exact_title_and_repairs() {
    repair_contract("title", SCHEMA, "Schema", "Schema");
}

#[test]
fn field_heading_exact_name_uses_configured_title_and_repairs() {
    let fields = SCHEMA.replace("Schema", "Table layout");
    repair_contract("name", &fields, "schema", "Table layout");
}

#[test]
fn field_heading_name_without_configured_title_and_repairs() {
    let fields = SCHEMA.replace("title = \"Schema\"\n", "");
    repair_contract("no-title", &fields, "schema", "schema");
}

#[test]
fn field_heading_exact_scope_uses_actual_duplicate_declaration_site() {
    let fixture = Fixture::new("scope", SCHEMA);
    fixture.doc(concat!(
        "## TBL-orders: First site\n\n",
        "### Schema\n\n",  // Direct here, despite another site's H1 declaration.
        "#### Schema\n\n", // Nested at this site, despite another site's H1 declaration.
        "# TBL-orders: Second site\n\n",
        "## Unrelated\n\n## SCHEMA\n\n## Schemas\n\n",
        "## 3. Parent\n\n### Schema\n\n",
        "## Schema ##\n\n", // Exact parsed title after the ATX closing hashes are removed.
        "# Outside\n\n## Schema\n",
    ));
    let rows = fixture.json();
    let headings = rows
        .iter()
        .filter(|row| row["code"] == "unmarked-heading")
        .collect::<Vec<_>>();
    assert_eq!(
        headings
            .iter()
            .map(|row| row["line"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [3, 5, 9, 11, 13, 17, 19]
    );
    for row in &headings[1..6] {
        assert!(
            !row["message"]
                .as_str()
                .unwrap()
                .contains("declares the field"),
            "{row}"
        );
    }
    for (row, marks) in [(headings[0], "###"), (headings[6], "##")] {
        let direct = row["message"].as_str().unwrap();
        assert!(
            direct.ends_with(&suffix("schema", marks, "Schema")),
            "missing field repair: {direct}"
        );
    }
}

#[test]
fn field_heading_collision_uses_configuration_header_order() {
    let fixture = Fixture::new("order", "");
    for (first_title, second_name, second_title, bare) in [
        ("Shared", "alpha", "Shared", "Shared"),
        ("alpha", "alpha", "Other", "alpha"),
    ] {
        fixture.config(&format!(
            "grund_config_version = 2\n[schema.kinds.TBL]\nfolders = [\"docs/tbl\"]\nindex = false\n\
             [schema.kinds.TBL.fields.zeta]\ntitle = \"{first_title}\"\npresence = \"must\"\n\
             [schema.kinds.TBL.fields.{second_name}]\ntitle = \"{second_title}\"\n"
        ));
        fixture.doc(&format!("# TBL-orders: Orders\n\n## {bare}\n"));
        let rows = fixture.json();
        assert_eq!(rows.len(), 2, "{rows:?}");
        let message = rows[1]["message"].as_str().unwrap();
        assert_eq!(
            message,
            format!(
                "{}{}",
                prefix("##", bare, "1"),
                suffix("zeta", "##", first_title)
            )
        );
    }
}

#[test]
fn field_heading_preserves_v1_value_chapter_and_relative_suffix() {
    let fixture = Fixture::new("compatibility", "");
    fixture.config(concat!(
        "grund_config_version = 1\n[id]\nnamed_sections = true\n",
        "[[kinds]]\nkind = \"TBL\"\nfolder = \"docs/tbl\"\nindex = false\nvalue_chapter = \"values\"\n",
        "[scan]\ninclude = [\"docs\"]\n",
    ));
    fixture.doc("# TBL-orders: Orders\n\n## values\n");
    // V1's synthesized values field must never produce the new clause.
    let output = Command::new(env!("CARGO_BIN_EXE_grund"))
        .arg("check")
        .arg(&fixture.0)
        .args(["--only", "unmarked-heading"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("{FILE}:3: error: {}\n", prefix("##", "values", "1"))
    );
    assert!(output.stderr.is_empty(), "{output:?}");

    // A dotted direct title gets both clauses, with the entire old relative clause first.
    let fields = SCHEMA.replace("Schema", ".a.b: Schema");
    fixture.config(&format!("grund_config_version = 2\n[schema.kinds.TBL]\nfolders = [\"docs/tbl\"]\nindex = false\n{fields}"));
    fixture.doc("# TBL-orders: Orders\n\n## .a.b: Schema\n");
    let rows = fixture.json();
    let message = rows
        .iter()
        .find(|row| row["code"] == "unmarked-heading")
        .unwrap()["message"]
        .as_str()
        .unwrap();
    let old = format!(
        "{}; under TBL-orders a relative heading is written ## .1. Schema",
        prefix("##", ".a.b: Schema", "1")
    );
    assert!(message.starts_with(&old), "{message}");
    assert_eq!(
        message,
        format!("{old}{}", suffix("schema", "##", ".a.b: Schema"))
    );
}
