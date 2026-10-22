//! `grund guide` prints a front page by default, and the core of the setup skill or
//! any reference section by name (§FS-init.5, §FS-init.5.5, §FS-init.5.6). Every
//! assertion reads the skill files from the repository rather than a copy of
//! them, so editing the guide never means refreshing a third copy here.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The reference sections in Contents order (§FS-init.5.5).
const SECTIONS: [&str; 10] = [
    "config",
    "citations",
    "languages",
    "members",
    "scoped-guidance",
    "schema",
    "queries",
    "editor",
    "integrations",
    "rules",
];

/// The budget on the core, the section `setup` (§FS-init.5.5).
const CORE_BUDGET: usize = 16_000;

/// Every `##` to `####` heading line of the single-file guide at 9d78aaaf38, before
/// the split. The split moves text and loses none (§FS-init.5.5), so each of these
/// lines is in exactly one file of the skill.
const SINGLE_FILE_HEADINGS: [&str; 35] = [
    "## Workflow",
    "## Repo Analysis First",
    "## Asking Style",
    "## Init Questions",
    "## Existing Specs Adoption",
    "## Config Questions",
    "### Top-level",
    "### `[reference]`",
    "### `[id]`",
    "### `[[kinds]]`",
    "### `[scan]`",
    "### `[output]`",
    "### `[fmt.cross_refs]`",
    "### `[citations]`",
    "### Citation directions",
    "## Recommendation Heuristics",
    "## Language And Repo Shape Examples",
    "### Rust",
    "### TypeScript / JavaScript",
    "### Python",
    "### Go",
    "### Java / Kotlin / Gradle",
    "### C / C++",
    "### C# / .NET",
    "### Polyglot Monorepo",
    "### Version 2: `[schema.sources]`",
    "### Version 1 to version 2",
    "## Splitting off a member",
    "## Validation",
    "## Scoped Guidance",
    "## Seeing the Schema",
    "## Querying the Structure",
    "## Editor Setup",
    "## Clickable Citation Integrations",
    "### Chapter rules",
];

/// The repository root, which holds both skill trees.
fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn skill_dir() -> PathBuf {
    manifest_dir().join("skills/grund-init")
}

fn embedded_skill_dir() -> PathBuf {
    manifest_dir().join("crates/grund-core/assets/skills/grund-init")
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn core() -> String {
    read(&skill_dir().join("SKILL.md"))
}

fn section(name: &str) -> String {
    read(&skill_dir().join("references").join(format!("{name}.md")))
}

/// Run the command and require the success shape of §FS-init.5: exit `0`, empty
/// stderr, the text on stdout.
fn print(args: &[&str]) -> String {
    let mut argv = vec!["guide"];
    argv.extend_from_slice(args);
    let output = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(&argv)
        .current_dir(manifest_dir())
        .output()
        .expect("spawn grund");
    assert_eq!(
        (
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).as_ref()
        ),
        (Some(0), ""),
        "grund {argv:?}"
    );
    String::from_utf8(output.stdout).expect("UTF-8 output")
}

/// The `##` headings of `text` with their line ranges, skipping fenced code.
fn h2_sections(text: &str) -> Vec<(String, Vec<&str>)> {
    let mut sections: Vec<(String, Vec<&str>)> = Vec::new();
    let mut fence: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim_start();
        let marker: String = trimmed
            .chars()
            .take_while(|c| *c == '`' || *c == '~')
            .collect();
        if marker.len() >= 3 {
            match &fence {
                None => fence = Some(marker.clone()),
                Some(open) if trimmed.trim_end() == open.as_str() => fence = None,
                Some(_) => {}
            }
        } else if fence.is_none()
            && let Some(title) = line.strip_prefix("## ")
        {
            sections.push((title.trim().to_string(), Vec::new()));
            continue;
        }
        if let Some((_, lines)) = sections.last_mut() {
            lines.push(line);
        }
    }
    sections
}

fn h2_body<'a>(sections: &'a [(String, Vec<&'a str>)], title: &str) -> &'a [&'a str] {
    sections
        .iter()
        .find(|(heading, _)| heading == title)
        .map(|(_, lines)| lines.as_slice())
        .unwrap_or_else(|| panic!("the core has no `## {title}` section"))
}

/// The command lines inside fenced code blocks of `lines`, in order.
fn fenced_commands<'a>(lines: &[&'a str]) -> Vec<&'a str> {
    let mut in_fence = false;
    let mut commands = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
        } else if in_fence && trimmed.starts_with("grund ") {
            commands.push(trimmed);
        }
    }
    commands
}

/// §FS-init.5.5, §FS-init.5.3: `setup` prints `SKILL.md` byte for byte, and that
/// core stays within its budget.
#[test]
fn setup_is_the_core_within_its_budget() {
    let printed = print(&["setup"]);
    assert_eq!(printed, core(), "`grund guide setup` is not SKILL.md");
    assert!(
        printed.len() <= CORE_BUDGET,
        "the core is {} bytes, over the {CORE_BUDGET}-byte budget",
        printed.len()
    );
}

/// §FS-init.5.1: the core's first three `##` sections are Fresh repository,
/// Contents and Workflow, and the fresh path names its three commands in order.
#[test]
fn core_opens_with_fresh_repository_then_contents_then_workflow() {
    let printed = print(&["setup"]);
    let sections = h2_sections(&printed);
    let opening: Vec<&str> = sections
        .iter()
        .take(3)
        .map(|(title, _)| title.as_str())
        .collect();
    assert_eq!(opening, ["Fresh repository", "Contents", "Workflow"]);
    let commands = fenced_commands(h2_body(&sections, "Fresh repository"));
    let shapes = ["grund init --scaffold", "grund init --check", "grund check"];
    assert_eq!(
        commands.len(),
        shapes.len(),
        "Fresh repository's fenced commands: {commands:?}"
    );
    for (command, shape) in commands.iter().zip(shapes) {
        assert!(
            command.starts_with(shape),
            "Fresh repository runs {commands:?}, not {shapes:?} in that order"
        );
    }
}

/// §FS-init.5.1: Workflow has an adopting agent fetch the `config` and `citations`
/// sections, where the config questions live, before it asks them.
#[test]
fn workflow_fetches_the_config_and_citations_sections() {
    let printed = print(&["setup"]);
    let sections = h2_sections(&printed);
    let workflow = h2_body(&sections, "Workflow").join("\n");
    for name in ["config", "citations"] {
        assert!(
            workflow.contains(&format!("`grund guide {name}`")),
            "Workflow does not fetch `grund guide {name}` before asking the config questions"
        );
    }
}

/// §FS-init.5.1, §FS-init.5.5: Contents names every reference section by its
/// `grund guide <name>` invocation, in the closed order.
#[test]
fn contents_names_every_section_in_order() {
    let printed = print(&["setup"]);
    let sections = h2_sections(&printed);
    let contents = h2_body(&sections, "Contents").join("\n");
    let mut named: Vec<String> = Vec::new();
    for occurrence in contents.split("`grund guide ").skip(1) {
        let name: String = occurrence
            .chars()
            .take_while(|c| c.is_ascii_lowercase() || *c == '-')
            .collect();
        if !name.is_empty() && !name.starts_with('-') && !named.contains(&name) {
            named.push(name);
        }
    }
    assert_eq!(named, SECTIONS, "Contents names the sections out of order");
}

/// §FS-init.5.3: `grund guide NAME` prints `references/NAME.md` byte for byte.
#[test]
fn each_section_prints_its_reference_file() {
    for name in SECTIONS {
        assert_eq!(
            print(&[name]),
            section(name),
            "`grund guide {name}` is not references/{name}.md"
        );
    }
}

/// §FS-init.5.3: both skill directories hold `SKILL.md` and one reference file per
/// section, byte-identical file for file, with no extra file on either side.
#[test]
fn skill_trees_hold_the_core_and_one_file_per_section() {
    let mut expected: Vec<String> = SECTIONS.iter().map(|name| format!("{name}.md")).collect();
    expected.sort();
    for dir in [skill_dir(), embedded_skill_dir()] {
        let mut top: Vec<String> = fs::read_dir(&dir)
            .unwrap_or_else(|error| panic!("read {}: {error}", dir.display()))
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        top.sort();
        assert_eq!(top, ["SKILL.md", "references"], "{}", dir.display());
        let references = dir.join("references");
        let mut files: Vec<String> = fs::read_dir(&references)
            .unwrap_or_else(|error| panic!("read {}: {error}", references.display()))
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        files.sort();
        assert_eq!(files, expected, "{}", references.display());
    }
    for file in std::iter::once("SKILL.md".to_string())
        .chain(SECTIONS.iter().map(|name| format!("references/{name}.md")))
    {
        assert_eq!(
            read(&skill_dir().join(&file)),
            read(&embedded_skill_dir().join(&file)),
            "{file} differs between the two skill trees"
        );
    }
}

/// §FS-init.5.5: `--full` prints the core, then every section in Contents order,
/// one newline between consecutive files.
#[test]
fn full_prints_the_core_then_every_section() {
    let mut expected = core();
    for name in SECTIONS {
        expected.push('\n');
        expected.push_str(&section(name));
    }
    assert_eq!(print(&["--full"]), expected);
}

/// §FS-init.5.5: every heading line of the single-file guide is in exactly one file
/// of the skill, so the split deferred text and lost none.
#[test]
fn every_heading_of_the_single_file_guide_is_in_exactly_one_file() {
    let files: Vec<(String, String)> = std::iter::once(("SKILL.md".to_string(), core()))
        .chain(
            SECTIONS
                .iter()
                .map(|name| (format!("references/{name}.md"), section(name))),
        )
        .collect();
    let misplaced: Vec<String> = SINGLE_FILE_HEADINGS
        .iter()
        .filter_map(|heading| {
            let holders: Vec<&str> = files
                .iter()
                .filter(|(_, text)| text.lines().any(|line| line == *heading))
                .map(|(file, _)| file.as_str())
                .collect();
            (holders.len() != 1).then(|| format!("{heading:?} in {holders:?}"))
        })
        .collect();
    assert!(
        misplaced.is_empty(),
        "headings not in exactly one file: {misplaced:#?}"
    );
}

/// §FS-init.5.3: the citation-directions copy is only in `citations`, the
/// chapter-rules copy only in `rules`, and neither is in the default output.
#[test]
fn each_marked_copy_is_only_in_its_own_section() {
    let copies = [
        ("<!-- BEGIN citation-directions -->", "citations"),
        ("<!-- BEGIN chapter-rules -->", "rules"),
    ];
    for (marker, owner) in copies {
        assert!(
            !core().contains(marker),
            "the core carries the copy {marker}"
        );
        for name in SECTIONS {
            assert_eq!(
                section(name).contains(marker),
                name == owner,
                "{marker} in references/{name}.md"
            );
        }
    }
}

/// A fresh directory under the system temp root, holding nothing but `git init`.
fn empty_repository() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("grund-guide-fresh-path-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create the empty repository");
    let status = Command::new("git")
        .args(["init", "-q"])
        .current_dir(&dir)
        .status()
        .expect("spawn git init");
    assert!(status.success(), "git init failed in {}", dir.display());
    dir
}

/// Run one printed command line: drop a trailing `# …` comment, fill `<name>`, and
/// run the words with the built binary in place of `grund`.
fn run_printed(line: &str, cwd: &Path) -> Output {
    let command = match line.find(" #") {
        Some(comment) => &line[..comment],
        None => line,
    };
    let command = command.replace("<name>", "fresh-demo");
    let mut words = command.split_whitespace();
    assert_eq!(words.next(), Some("grund"), "{line}");
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(words)
        .current_dir(cwd)
        .output()
        .expect("spawn grund")
}

/// §FS-init.5.1, §FS-init.5.6: the three commands of Fresh repository, as the front
/// page prints them, run in order on an empty repository; each exits `0`, and `grund check` warns that nothing is
/// recognized until the first declaration exists (§FS-check.4.5).
#[test]
fn fresh_path_runs_as_printed_on_an_empty_repository() {
    let printed = print(&[]);
    let sections = h2_sections(&printed);
    let commands = fenced_commands(h2_body(&sections, "Fresh repository"));
    assert_eq!(
        commands.len(),
        3,
        "Fresh repository's commands: {commands:?}"
    );
    let repo = empty_repository();
    for line in &commands {
        let output = run_printed(line, &repo);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(
            output.status.code(),
            Some(0),
            "`{line}` failed:\n{stdout}{stderr}"
        );
        if line.starts_with("grund check") {
            assert!(
                format!("{stdout}{stderr}").contains("nothing recognized"),
                "`{line}` on a fresh scaffold does not warn that nothing is recognized:\n\
                 {stdout}{stderr}"
            );
        }
    }
    let _ = fs::remove_dir_all(&repo);
}

/// The core's `## Fresh repository` section, from its heading up to the next `##`
/// heading, read from the skill file rather than from the command.
fn fresh_section_of_the_core() -> String {
    let core = core();
    let start = core
        .find("\n## Fresh repository\n")
        .expect("the core has a Fresh repository section")
        + 1;
    let end = core[start..]
        .find("\n## ")
        .map_or(core.len(), |at| start + at + 1);
    core[start..end].to_string()
}

/// A size as §FS-init.5.6 writes it: kilobytes of 1,000 bytes, rounded to the
/// nearest whole and never below 1.
fn kilobytes(bytes: usize) -> String {
    format!("{} KB", ((bytes + 500) / 1000).max(1))
}

/// The phrase a reference section's Contents line ends with, without the period.
fn contents_summary(name: &str) -> String {
    let entry = format!("- `grund guide {name}` (`references/{name}.md`): ");
    core()
        .lines()
        .find_map(|line| line.strip_prefix(entry.as_str()))
        .unwrap_or_else(|| panic!("Contents has no line for {name}"))
        .trim_end_matches('.')
        .to_string()
}

/// §FS-init.5.6: the front page is the title, the core's Fresh repository section
/// byte for byte, then the Sections heading.
#[test]
fn front_page_opens_with_the_cores_fresh_path() {
    let printed = print(&[]);
    let opening = format!(
        "# grund guide\n\n{}## Sections\n\n",
        fresh_section_of_the_core()
    );
    assert!(
        printed.starts_with(&opening),
        "the front page does not open with the title and the core's fresh path:\n{printed}"
    );
}

/// §FS-init.5.6, §FS-init.5.5: the front page lists every section in `--full`
/// order with the size it prints and its summary, a reference section's being the
/// phrase its Contents line ends with, and names the size of the whole guide.
#[test]
fn front_page_lists_every_section_with_its_size_and_summary() {
    let printed = print(&[]);
    let full = print(&["--full"]);
    assert!(
        printed.contains(&format!(
            "`grund guide --full` prints every section, {} in all.",
            kilobytes(full.len())
        )),
        "the front page does not give the whole guide's size:\n{printed}"
    );
    let fence = printed
        .split("```text\n")
        .nth(1)
        .and_then(|rest| rest.split("```\n").next())
        .expect("the front page lists the sections in a text fence");
    let rows: Vec<Vec<&str>> = fence
        .lines()
        .map(|line| line.splitn(2, "  ").map(str::trim).collect())
        .collect();
    let names: Vec<&str> = rows.iter().map(|row| row[0]).collect();
    let mut expected_names = vec!["setup"];
    expected_names.extend(SECTIONS);
    assert_eq!(
        names, expected_names,
        "the front page lists the sections out of order"
    );
    for (line, name) in fence.lines().zip(&expected_names) {
        let text = if *name == "setup" {
            core()
        } else {
            section(name)
        };
        let rest = line[name.len()..].trim_start();
        let (size, summary) = rest
            .split_once("  ")
            .unwrap_or_else(|| panic!("no size and summary for {name}: {line:?}"));
        assert_eq!(size.trim(), kilobytes(text.len()), "the size of {name}");
        if *name == "setup" {
            assert!(!summary.is_empty(), "setup has no summary");
        } else {
            assert_eq!(summary, contents_summary(name), "the summary of {name}");
        }
    }
}

/// §FS-init.5.7: the deprecated name is gone from grund 0.20.0, so it is refused as
/// any first word that is no subcommand is: alone or with `--full`, the failed ID
/// query of §FS-cli.1.2 (exit `1`), and as a help request the unknown command of
/// §FS-cli.1.3 (exit `2`). No form prints the guide or the deprecation warning, which
/// would come first; the e2e goldens pin the whole refusal.
#[test]
fn removed_name_is_refused_as_any_unknown_word() {
    let invalid = "invalid ID `agent-setup-instructions`\nhint: ";
    let unknown = "error: unknown command: agent-setup-instructions\nknown commands: ";
    for (args, code, opening) in [
        (&["agent-setup-instructions"][..], 1, invalid),
        (&["agent-setup-instructions", "--full"], 1, invalid),
        (&["help", "agent-setup-instructions"], 2, unknown),
        (&["agent-setup-instructions", "-h"], 2, unknown),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(manifest_dir())
            .output()
            .expect("spawn grund");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(code), "grund {args:?}: {stderr}");
        assert!(output.stdout.is_empty(), "grund {args:?} printed a page");
        assert!(stderr.starts_with(opening), "grund {args:?}: {stderr}");
    }
}
