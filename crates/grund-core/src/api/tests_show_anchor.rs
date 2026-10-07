//! The heading anchor `show --format=json` carries is the fragment
//! `grund fmt --cross-refs` derives for the same coordinate, under every
//! `anchor_format` profile (§FS-show.3.1.3.1, §FS-output-shapes.4).

use super::*;
use crate::queries::{ShowFormat, ShowMode, ShowOpts};
use crate::testing::{test_root, write};
use serde_json::Value;
use std::path::Path;

const PROFILES: [&str; 5] = ["github", "gitlab", "mkdocs", "pandoc", "none"];

/// Headings that take each profile's slugger through its edge cases: a run of
/// punctuation that github keeps as `--` and mkdocs collapses, a heading that
/// carries a wrapped citation, and HTML-shaped spans the renderer drops.
const ANCHORS: &str = "# FS-anchors: Anchors — the \"cases\" (`--x`)\n\nLead.\n\n\
## 1. Runs -- of --- dashes\n\nOne.\n\n\
## 2. Linked [\u{a7}FS-other](FS-other.md#stale) heading\n\nTwo.\n\n\
## 3. HTML <span>tag</span> & <ID> text\n\nThree.\n\n\
### 3.1 Child: one, two\n\nChild.\n";

const COORDINATES: [&str; 5] = [
    "FS-anchors",
    "FS-anchors.1",
    "FS-anchors.2",
    "FS-anchors.3",
    "FS-anchors.3.1",
];

fn config(profile: &str) -> String {
    format!(
        "grund_config_version = 1\n[reference]\nstrict = false\n[id]\nformat = \"{{kind}}-{{slug}}\"\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n\
         [fmt.cross_refs]\nanchor_format = \"{profile}\"\n"
    )
}

/// What `fmt --cross-refs --write` wrote for each coordinate: the fragment
/// after `#`, or `None` for a bare file link.
fn formatter_fragments(root: &Path) -> Vec<(String, Option<String>)> {
    write(
        &root.join("docs/cite.md"),
        &COORDINATES.map(|id| format!("§{id}\n")).concat(),
    );
    format_references(FmtOpts {
        path: root.to_path_buf(),
        path_provided: true,
        write: true,
        cross_refs: true,
        ..FmtOpts::default()
    })
    .expect("fmt --cross-refs");
    std::fs::read_to_string(root.join("docs/cite.md"))
        .unwrap()
        .lines()
        .map(|line| {
            let (label, target) = line
                .strip_prefix("[§")
                .and_then(|rest| rest.strip_suffix(')'))
                .and_then(|rest| rest.split_once("]("))
                .unwrap_or_else(|| panic!("fmt wrapped the citation: {line}"));
            let fragment = target.split_once('#').map(|(_, f)| f.to_string());
            (label.to_string(), fragment)
        })
        .collect()
}

fn shown(root: &Path, id: &str, mode: ShowMode) -> Value {
    let output = show(
        id,
        ShowOpts {
            path: root.to_path_buf(),
            section: None,
            mode,
            format: ShowFormat::Json,
        },
    )
    .unwrap();
    serde_json::from_str(&output.json.expect("json requested")).unwrap()
}

fn anchor_of(value: &Value) -> Option<String> {
    match value.get("anchor") {
        Some(Value::String(anchor)) => Some(anchor.clone()),
        Some(Value::Null) => None,
        other => panic!("anchor must be a string or null, got {other:?} in {value}"),
    }
}

#[test]
fn show_anchor_equals_formatter_fragment_under_every_profile() {
    for profile in PROFILES {
        let root = test_root(&format!("show_anchor_profile_{profile}"));
        write(&root.join("grund.toml"), &config(profile));
        write(&root.join("docs/FS-anchors.md"), ANCHORS);
        write(&root.join("docs/FS-other.md"), "# FS-other: Other\n");
        let fragments = formatter_fragments(&root);
        assert_eq!(fragments.len(), COORDINATES.len(), "{profile}");

        for (coordinate, fragment) in &fragments {
            assert_eq!(
                profile == "none",
                fragment.is_none(),
                "{profile}: fmt fragment for {coordinate}"
            );
            for (label, mode) in [
                ("lead", ShowMode::Lead),
                ("brief", ShowMode::Brief),
                ("toc", ShowMode::Toc),
                ("full", ShowMode::Full),
            ] {
                let value = shown(&root, coordinate, mode);
                assert_eq!(
                    anchor_of(&value),
                    *fragment,
                    "{profile}: show {coordinate} --{label}"
                );
            }
        }

        let toc = shown(&root, "FS-anchors", ShowMode::Toc);
        let entries = toc["sections"].as_array().expect("toc sections");
        assert_eq!(entries.len(), 4, "{profile}");
        for entry in entries {
            let coordinate = format!("FS-anchors.{}", entry["path"].as_str().unwrap());
            let fragment = fragments
                .iter()
                .find(|(label, _)| *label == coordinate)
                .map(|(_, fragment)| fragment.clone())
                .unwrap();
            assert_eq!(
                anchor_of(entry),
                fragment,
                "{profile}: toc entry {coordinate}"
            );
        }
    }
}
