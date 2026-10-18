//! Delivered editor setup guidance (§FS-lsp.2.3), checked offline as part of
//! the repository gate (§AR-ci.1), without an editor or an LSP server.

/// Require related claims in one prose paragraph, rather than accepting words
/// scattered through configuration snippets or unrelated editor instructions.
fn explains(paragraphs: &[String], groups: &[&[&str]]) -> bool {
    paragraphs.iter().any(|paragraph| {
        groups
            .iter()
            .all(|alternatives| alternatives.iter().any(|text| paragraph.contains(text)))
    })
}

/// §FS-lsp.2.3: the named clients' activation prerequisite, symptoms and remedy
/// must be discoverable where the user configures the VSCode / VSCodium client.
#[test]
fn vscode_setup_explains_workspace_trust() {
    let guide = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/user-facing/lsp.md"),
    )
    .expect("read delivered LSP guide");
    let section = guide
        .split_once("\n## VSCode / VSCodium\n")
        .expect("VSCode / VSCodium setup section")
        .1
        .split("\n## ")
        .next()
        .unwrap();
    let mut in_code = false;
    let prose = section
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
    let paragraphs: Vec<String> = prose
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
            "explain the named clients' default disablement without untrusted-workspace support",
            &[
                &["simple lsp client"],
                &["glspc"],
                &["default"],
                &["untrusted-workspace", "untrusted workspace"],
                &["support"],
                &["do not declare", "lack", "without", "no untrusted"],
                &[
                    "disabl",
                    "cannot activate",
                    "does not activate",
                    "never activate",
                ],
            ],
        ),
        (
            "name Restricted Mode in the status bar",
            &[&["restricted mode"], &["status bar", "status-bar"]],
        ),
        (
            "connect inactive-client silence to missing hover and diagnostics",
            &[
                &[
                    "no hover",
                    "missing hover",
                    "hover is unavailable",
                    "hover and diagnostics are unavailable",
                ],
                &[
                    "no diagnostics",
                    "missing diagnostics",
                    "diagnostics are unavailable",
                    "no hover or diagnostics",
                    "no hover and diagnostics",
                ],
            ],
        ),
        (
            "include the absent grund-lsp process/server-start symptom",
            &[
                &["grund-lsp"],
                &[
                    "no grund-lsp process",
                    "no running grund-lsp",
                    "does not start",
                    "never starts",
                    "won't start",
                    "not launched",
                ],
            ],
        ),
        (
            "give Workspaces: Manage Workspace Trust as the folder-trust remedy",
            &[
                &["workspaces: manage workspace trust"],
                &[
                    "trust the folder",
                    "trust this folder",
                    "trust a folder",
                    "trust your folder",
                    "trust the workspace",
                    "trust this workspace",
                ],
            ],
        ),
        (
            "explain parent trust covering descendants including sibling checkouts",
            &[
                &["trust"],
                &["parent"],
                &["cover", "include", "applies"],
                &["subfolder", "subdirector", "descendant"],
                &["sibling"],
            ],
        ),
    ];
    let missing: Vec<_> = checks
        .iter()
        .filter(|(_, groups)| !explains(&paragraphs, groups))
        .map(|(description, _)| format!("MISSING: {description}"))
        .collect();
    assert!(
        missing.is_empty(),
        "docs/user-facing/lsp.md, VSCode / VSCodium setup:\n{}",
        missing.join("\n")
    );
}
