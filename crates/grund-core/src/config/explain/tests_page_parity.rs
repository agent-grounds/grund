//! §FS-config-explain.3.1: one rule puts a key's type into words, and the
//! rendered configuration page uses the same words, its Markdown aside. The
//! page is read as `scripts/render_config_reference.py` writes it, and every key
//! the gates admit that has a row there is held to the answer's words.

use std::collections::BTreeMap;
use std::path::Path;

use super::schema;
use crate::config::vocabulary::admitted;

/// The page's Type cell for each `(version, header, key)`, `""` for the top
/// level, backticks dropped and `\|` read as `|`; a table that has "the same
/// keys as" another gets its rows.
fn page_types() -> BTreeMap<(u32, String, String), String> {
    let page =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/user-facing/configuration.md");
    let page = std::fs::read_to_string(page).expect("the configuration page is readable");
    let mut types = BTreeMap::new();
    let (mut version, mut header) = (0, String::new());
    for line in page.lines() {
        if let Some(title) = line.strip_prefix("## Version ") {
            version = title
                .trim()
                .parse()
                .expect("a version heading names its number");
        } else if line == "### Top level" {
            header.clear();
        } else if let Some(title) = line.strip_prefix("### ") {
            header = title.trim_matches('`').to_string();
        } else if let Some(same) = line.strip_prefix("The same keys as `") {
            let same = same.trim_end_matches("`.");
            let rows: Vec<_> = types
                .iter()
                .filter(|((v, h, _), _)| *v == version && h == same)
                .map(|((_, _, key), words): (&(u32, String, String), &String)| {
                    (key.clone(), words.clone())
                })
                .collect();
            for (key, words) in rows {
                types.insert((version, header.clone(), key), words);
            }
        } else if version > 0 && line.starts_with("| `") {
            let cells: Vec<String> = line
                .replace("\\|", "\u{1}")
                .split('|')
                .map(|cell| cell.trim().replace('\u{1}', "|").replace('`', ""))
                .collect();
            types.insert(
                (version, header.clone(), cells[1].clone()),
                cells[2].clone(),
            );
        }
    }
    types
}

#[test]
fn every_key_on_the_page_is_typed_in_the_answers_words() {
    let page = page_types();
    let schema = schema::embedded();
    let (mut compared, mut differ) = (0, Vec::new());
    for version in [1, 2] {
        for (header, keys) in admitted(version) {
            for key in keys.iter().filter(|key| !key.contains('.')) {
                let node = schema::key_node(schema, version, &header, key).expect("a node");
                let answer = schema::value_words(schema, node);
                let Some(words) = page.get(&(version, header.clone(), key.clone())) else {
                    // The page gives an array of tables a heading of its own, not a row.
                    if answer != "array of tables" {
                        differ.push(format!("version {version} `{header} {key}`: no row"));
                    }
                    continue;
                };
                compared += 1;
                if *words != answer {
                    differ.push(format!(
                        "version {version} `{header} {key}`: page {words:?}, answer {answer:?}"
                    ));
                }
            }
        }
    }
    assert_eq!(differ, Vec::<String>::new());
    assert!(compared > 100, "only {compared} keys were compared");
}
