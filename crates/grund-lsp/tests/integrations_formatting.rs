//! Delivered LSP4IJ setup for the live citation transform (§FS-lsp.2.4).
//! Exercise the installed binary and the IntelliJ guide without claiming an IDE run.

mod integrations_support;
mod support;

use integrations_support::*;
use serde_json::Value;
use std::fs;
use std::path::Path;

/// §FS-lsp.2.4: preview teaches the client setting, its location and purpose.
#[test]
fn preview_teaches_live_transform_client_configuration() {
    let sandbox = Sandbox::with_binary(&workspace_binary(), "formatting-preview");
    let project = sandbox.project("project");
    let output = run(&sandbox.binary, &project, ["integrations", "lsp4ij"]);
    assert_success(&output, "preview");
    assert_guidance(&[("preview", stdout(&output))]);
}

/// §FS-lsp.2.4: both successful write outcomes print the delivered README.
#[test]
fn writes_teach_live_transform_client_configuration() {
    let sandbox = Sandbox::with_binary(&workspace_binary(), "formatting-write");
    let project = sandbox.project("project");
    let root = sandbox.root.join("import");
    let args = ["integrations", "lsp4ij", "--write", root.to_str().unwrap()];
    let created = run(&sandbox.binary, &project, args);
    assert_success(&created, "created write");
    let readme = fs::read_to_string(root.join("README.md")).expect("read generated README");
    let unchanged = run(&sandbox.binary, &project, args);
    assert_success(&unchanged, "unchanged write");
    assert!(stdout(&created).starts_with("created "));
    assert!(stdout(&unchanged).starts_with("unchanged "));
    assert!(stdout(&created).contains(&readme));
    assert!(stdout(&unchanged).contains(&readme));
    assert_guidance(&[
        ("generated README", readme),
        ("created output", stdout(&created)),
        ("unchanged output", stdout(&unchanged)),
    ]);
}

/// §FS-lsp.2.3: the IntelliJ guide teaches the same setup and a typing example.
#[test]
fn guide_teaches_live_transform_client_configuration() {
    let guide = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/user-facing/lsp.md"),
    )
    .expect("read LSP guide");
    let section = guide
        .split_once("\n## IntelliJ family\n")
        .expect("IntelliJ setup section")
        .1
        .split("\n## ")
        .next()
        .unwrap();
    let mut missing = guidance_missing(section);
    // Accept any illustrated citation ID, but require its actual transformed text.
    let has_example = section.match_indices("$$").any(|(offset, _)| {
        let id: String = section[offset + 2..]
            .chars()
            .take_while(|ch| ch.is_alphanumeric() || matches!(ch, '-' | '.'))
            .collect();
        id.contains('-') && section.contains(&format!("§{id}"))
    });
    if !has_example {
        missing.push("typing example with a citation ID and its transformed text");
    }
    assert!(
        missing.is_empty(),
        "IntelliJ guide: missing {}",
        missing.join("; ")
    );
}

/// Check valid, true-valued JSON and related prose, rather than a keyword that
/// could occur in a warning, another editor's setup, or a false-valued example.
fn guidance_missing(text: &str) -> Vec<&'static str> {
    let has_json = text.split("```json").skip(1).any(|block| {
        block.split_once("```").is_some_and(|(json, _)| {
            serde_json::from_str::<Value>(json.trim()).is_ok_and(|value| {
                value["format"]["existingFormatterOverrideable"].as_bool() == Some(true)
            })
        })
    });
    let mut in_code = false;
    let prose = text
        .lines()
        .filter(|line| {
            if line.trim_start().starts_with("```") {
                in_code = !in_code;
                false
            } else {
                !in_code
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let paragraphs: Vec<_> = prose
        .split("\n\n")
        .map(|paragraph| {
            paragraph
                .replace(['`', '*'], "")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase()
        })
        .collect();
    let checks: &[(&str, &[&[&str]])] = &[
        (
            "where to set/merge the imported grund LSP client configuration",
            &[
                &["settings"],
                &["languages & frameworks"],
                &["language servers"],
                &["grund lsp"],
                &["client configuration"],
                &["merge", "set the", "add the", "enter", "paste"],
            ],
        ),
        (
            "preserving existing client settings when merging the format entry",
            &[
                &["existing", "other"],
                &["settings", "configuration"],
                &["preserv", "keep", "retain", "merge"],
                &["format"],
            ],
        ),
        (
            "why an existing formatter can withhold on-type requests",
            &[
                &["formatter"],
                &["when", "if "],
                &["on-type", "on type", "textdocument/ontypeformatting"],
                &[
                    "withhold",
                    "prevent",
                    "suppress",
                    "does not send",
                    "won't send",
                ],
            ],
        ),
        (
            "connection to the live $$ to § transform",
            &[
                &["$$"],
                &["§"],
                &["transform", "convert", "becomes", "replace"],
                &["typ"],
            ],
        ),
    ];
    let mut missing = Vec::new();
    if !has_json {
        missing.push("valid JSON setting format.existingFormatterOverrideable to true");
    }
    for (description, groups) in checks {
        let explained = paragraphs.iter().any(|paragraph| {
            groups
                .iter()
                .all(|alternatives| alternatives.iter().any(|word| paragraph.contains(word)))
        });
        if !explained {
            missing.push(*description);
        }
    }
    missing
}

fn assert_guidance(surfaces: &[(&str, String)]) {
    let failures: Vec<_> = surfaces
        .iter()
        .flat_map(|(name, text)| {
            guidance_missing(text)
                .into_iter()
                .map(move |missing| format!("{name}: missing {missing}"))
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
