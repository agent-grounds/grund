//! A workspace member that must cite no other project says so in its own config
//! (§FS-config.3.9.3): it closes its default (§FS-config.3.9.4,
//! §FS-config-v2.rules.citations.default) and names its own kinds' targets bare
//! under `may`. `examples/workspace-isolated-member/` runs that recipe and its two
//! costs (§FS-examples.5.5), and the citation-directions guide, with both copies of
//! the setup skill's `citations` reference, teaches it. Issue #1077: the guide
//! never showed it, and its one sentence on a member's own run sent the reader
//! to the workspace root instead.
//!
//! The example's goldens are the binary's words, because the shared runner
//! compares them on every `cargo test`. So the guide is held to the goldens and
//! the example's member config here, never to a copy of either, and neither can
//! drift from what grund prints.

use std::fs;
use std::path::{Path, PathBuf};

use super::case_runner::discover_examples;
use super::repo_root;

const EXAMPLE: &str = "examples/workspace-isolated-member";
const LEAKS: &str = "isolated-1-leaks";
const MEMBER_CLEAN: &str = "isolated-2-member-clean";
const NO_CODE_ROW: &str = "isolated-3-no-code-row";
const PINNED_SIBLING: &str = "isolated-4-pinned-sibling";
const MEMBER_CONFIG: &str = "repo/ce/substratevm/grund.toml";
const GUIDE: &str = "docs/user-facing/citation-directions.md";
const SKILL_COPIES: [&str; 2] = [
    "skills/grund-init/references/citations.md",
    "crates/grund-core/assets/skills/grund-init/references/citations.md",
];

fn read(root: &Path, relative: &str) -> String {
    fs::read_to_string(root.join(relative)).unwrap_or_else(|err| panic!("read {relative}: {err}"))
}

fn run_file(root: &Path, run: &str, file: &str) -> String {
    read(root, &format!("{EXAMPLE}/{run}/{file}"))
}

/// The member's citation rules as the example writes them: its config from the
/// `[rules.citations]` line to the end, which is what a reader appends.
fn recipe(root: &Path, run: &str) -> String {
    let config = run_file(root, run, MEMBER_CONFIG);
    let start = config
        .find("[rules.citations]\n")
        .unwrap_or_else(|| panic!("{run}: the member config has no [rules.citations]"));
    config[start..].to_string()
}

fn collapsed(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// One fenced block: the line its opening fence sits on, its info string, its body.
struct Fence {
    line: usize,
    info: String,
    body: String,
}

/// The backtick fences of a Markdown page, in order. The guide nests none.
fn fences(page: &str) -> Vec<Fence> {
    let mut fences = Vec::new();
    let mut open: Option<Fence> = None;
    for (line, text) in page.lines().enumerate() {
        let trimmed = text.trim_start();
        let Some(rest) = trimmed.strip_prefix("```") else {
            if let Some(fence) = open.as_mut() {
                fence.body.push_str(text);
                fence.body.push('\n');
            }
            continue;
        };
        match open.take() {
            Some(fence) if rest.trim().is_empty() => fences.push(fence),
            Some(mut fence) => {
                fence.body.push_str(text);
                fence.body.push('\n');
                open = Some(fence);
            }
            None => {
                open = Some(Fence {
                    line,
                    info: rest.trim().to_string(),
                    body: String::new(),
                })
            }
        }
    }
    fences
}

/// The text of the nearest `##`-or-deeper heading above `line`, outside a fence.
fn heading_above(page: &str, fences: &[Fence], line: usize) -> Option<String> {
    let lines: Vec<&str> = page.lines().collect();
    (0..line)
        .rev()
        .filter(|&index| {
            !fences.iter().any(|fence| {
                fence.line <= index && index <= fence.line + fence.body.lines().count() + 1
            })
        })
        .find_map(|index| {
            let text = lines[index];
            let hashes = text.len() - text.trim_start_matches('#').len();
            (hashes >= 2 && text[hashes..].starts_with(' ')).then(|| text[hashes + 1..].to_string())
        })
}

/// Lowercase letters and digits only: a GitHub anchor and the heading it was made
/// from agree on these, so this says which heading a fragment names without
/// spelling GitHub's slug rules out again. lychee checks that the fragment resolves.
fn letters(text: &str) -> String {
    text.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// The fragments of the Markdown links in `text`, the part after `#` of each
/// `](…#…)` destination.
fn link_fragments(text: &str) -> Vec<&str> {
    text.split("](")
        .skip(1)
        .filter_map(|tail| tail.split(')').next())
        .filter_map(|destination| destination.split_once('#').map(|(_, fragment)| fragment))
        .collect()
}

/// The paragraph that explains a target pinned to another project.
fn workspace_paragraph(page: &str) -> Option<&str> {
    page.split("\n\n")
        .find(|paragraph| paragraph.contains("`alias/KIND`"))
}

/// What one surface is missing of the member recipe, one line per gap.
fn recipe_gaps(name: &str, page: &str, recipe: &str, costs: &[String]) -> Vec<String> {
    let mut gaps = Vec::new();
    let fences = fences(page);
    match fences
        .iter()
        .find(|fence| fence.info == "toml" && fence.body.ends_with(recipe))
    {
        None => gaps.push(format!(
            "{name}: no ```toml fence ends with the example member's rules verbatim:\n{recipe}"
        )),
        Some(fence) => match heading_above(page, &fences, fence.line) {
            None => gaps.push(format!(
                "{name}: the recipe sits under no `##` heading of its own"
            )),
            Some(heading) => {
                let linked = workspace_paragraph(page).is_some_and(|paragraph| {
                    link_fragments(paragraph)
                        .iter()
                        .any(|fragment| letters(fragment) == letters(&heading))
                });
                if !linked {
                    gaps.push(format!(
                        "{name}: the workspace paragraph (the one on `alias/KIND`) does not link \
                         the recipe's heading `{heading}`"
                    ));
                }
            }
        },
    }
    let text = collapsed(page);
    for cost in costs {
        if !text.contains(&collapsed(cost)) {
            gaps.push(format!(
                "{name}: does not quote the cost as grund prints it: {cost}"
            ));
        }
    }
    if !page.contains(EXAMPLE) {
        gaps.push(format!("{name}: does not link {EXAMPLE}"));
    }
    gaps
}

fn run_dir(root: &Path, run: &str) -> PathBuf {
    root.join(EXAMPLE).join(run)
}

/// §FS-config.3.9.3: the recipe the guide teaches runs. The member names no other
/// project, yet the run from the root refuses its citation of `ee` from a kind file
/// and from a homeless one; once the leaks are gone the member's own run passes.
/// The shared runner compares each golden; this says what they must hold, and
/// fails where the example is deleted rather than letting the runs shrink.
#[test]
fn the_isolated_member_example_runs_the_recipe() {
    let root = repo_root();
    let examples = discover_examples(&root);
    for run in [LEAKS, MEMBER_CLEAN, NO_CODE_ROW, PINNED_SIBLING] {
        assert!(
            examples.contains(&run_dir(&root, run)),
            "{EXAMPLE}/{run} is not in the maintained suite"
        );
    }

    let rules = recipe(&root, LEAKS);
    assert!(rules.contains("default = \"must-not\""), "{rules}");
    assert!(rules.contains("[rules.citations.code]\n"), "{rules}");
    assert!(
        !rules.contains('/'),
        "the recipe names another project: {rules}"
    );
    assert_eq!(
        run_file(&root, LEAKS, "command.args").trim(),
        "check {repo}"
    );
    assert_eq!(run_file(&root, LEAKS, "expected.exit"), "1\n");
    let leaks = run_file(&root, LEAKS, "expected.stdout");
    let refused: Vec<&str> = leaks.lines().collect();
    assert_eq!(refused.len(), 2, "{leaks}");
    assert!(
        refused[0].starts_with("ce/substratevm/docs/notes.md:"),
        "{leaks}"
    );
    assert!(refused[0].contains(": error: code must not cite ee/FS (citation direction)"));
    assert!(
        refused[1].starts_with("ce/substratevm/docs/requirements.md:"),
        "{leaks}"
    );
    assert!(refused[1].contains(": error: FS must not cite ee/FS (citation direction)"));

    assert_eq!(recipe(&root, MEMBER_CLEAN), rules);
    let member = run_file(&root, MEMBER_CLEAN, "command.args");
    assert_eq!(member.trim(), "check {repo}/ce/substratevm");
    assert_eq!(run_file(&root, MEMBER_CLEAN, "expected.exit"), "0\n");
    assert_eq!(
        run_file(&root, MEMBER_CLEAN, "expected.stdout"),
        "success\n"
    );

    // The first cost: drop the homeless kind's row and its local citations fail.
    let without_code = recipe(&root, NO_CODE_ROW);
    assert_eq!(
        format!("{without_code}\n[rules.citations.code]\nmay = [\"FS\"]\n"),
        rules
    );
    assert_eq!(run_file(&root, NO_CODE_ROW, "expected.exit"), "1\n");
    assert!(
        run_file(&root, NO_CODE_ROW, "expected.stdout")
            .contains(": error: code must not cite FS (citation direction)")
    );

    // The second cost: a pinned sibling under `may` fails the member's own load.
    let pinned = recipe(&root, PINNED_SIBLING);
    assert!(
        pinned.contains("may = [\"FS\", \"compiler/FS\"]"),
        "{pinned}"
    );
    let own = run_file(&root, PINNED_SIBLING, "command.args");
    assert_eq!(own.trim(), "check {repo}/ce/substratevm");
    assert_eq!(run_file(&root, PINNED_SIBLING, "expected.exit"), "2\n");
    let refusal = run_file(&root, PINNED_SIBLING, "expected.stderr");
    assert!(
        refusal.contains("unknown project alias compiler"),
        "{refusal}"
    );
    assert!(
        refusal.contains("check from the workspace root"),
        "{refusal}"
    );
}

/// §FS-config.3.9.3, issue #1077: the guide teaches the member recipe the example
/// runs, in its own section the workspace paragraph links, and quotes its two
/// costs as the example's goldens print them; the skill's two copies say the same,
/// and the example is linked from the guide, the examples index and the README.
#[test]
#[ignore = "red until #1077 teaches the member recipe in the guide, both skill copies, the example README and both indexes; implement removes this"]
fn the_guide_teaches_the_member_recipe_the_example_runs() {
    let root = repo_root();
    let rules = recipe(&root, LEAKS);
    // The homeless file's local citation, through the finding's message: the
    // repair advice after it is the ordinary one and not this cost's lesson.
    let code_row = run_file(&root, NO_CODE_ROW, "expected.stdout");
    let code_row = code_row
        .lines()
        .find(|line| line.contains(": error: code must not cite FS ("))
        .and_then(|line| line.split(" \u{2014} ").next())
        .expect("the code-row run refuses a local citation")
        .to_string();
    // The member's own load refusal, the whole line.
    let pinned = run_file(&root, PINNED_SIBLING, "expected.stderr")
        .trim_end()
        .to_string();
    let costs = [code_row, pinned];

    let mut gaps = recipe_gaps(GUIDE, &read(&root, GUIDE), &rules, &costs);
    for copy in SKILL_COPIES {
        gaps.extend(recipe_gaps(copy, &read(&root, copy), &rules, &costs));
    }

    let readme = format!("{EXAMPLE}/README.md");
    match fs::read_to_string(root.join(&readme)) {
        Err(_) => gaps.push(format!("{readme} is missing")),
        Ok(text) => {
            if !text.contains(GUIDE) {
                gaps.push(format!("{readme}: does not link {GUIDE}"));
            }
            for cost in &costs {
                if !collapsed(&text).contains(&collapsed(cost)) {
                    gaps.push(format!("{readme}: does not quote the cost: {cost}"));
                }
            }
        }
    }
    if !read(&root, "examples/README.md").contains("](workspace-isolated-member/)") {
        gaps.push("examples/README.md: no Workflows row links workspace-isolated-member/".into());
    }
    if !read(&root, "README.md").contains(&format!("{EXAMPLE}/")) {
        gaps.push(format!("README.md: does not link {EXAMPLE}/"));
    }
    assert!(
        gaps.is_empty(),
        "{} gap(s) in teaching the member recipe:\n{}",
        gaps.len(),
        gaps.join("\n")
    );
}
