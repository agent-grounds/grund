//! Test module: one answer to what a relative heading is (§FS-declarations.section.relative.parent),
//! given by the parent pass to a Markdown body and a doc-comment body alike
//! (§FS-declarations.section.relative.where).

use super::relative_sections::RelativeAnswer;
use crate::config::load_config;
use crate::grammar::relative_heading_form;
use crate::model::Catalog;
use crate::testing::{scan_findings, test_root, write};

/// Every kind of parent and refusal the rule knows, one heading each.
const BODY: &str = "## format: Formatting\n\n### .1: Username\n\n#### .2: Deeper\n\n\
                    ## .notes: Notes\n\n#### .9. Orphan\n\n### .x. Wrong punctuation\n\n\
                    ### .a: Child of a relative parent\n";

/// `BODY` under `FS-login` in Markdown and under `FS-logout` in a Rust doc-comment,
/// each body line on the same file line, scanned together.
fn scan() -> Catalog {
    let root = test_root("doc_comment_relative_one_answer");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n[reference]\nstrict = true\n\
         require_grounding = false\n\n[id]\nformat = \"{kind}-{slug}\"\n\
         named_sections = true\n\n[[kinds]]\nkind = \"FS\"\n\
         folder = \"docs\"\nindex = false\n\n[scan]\ninclude = [\"docs\", \"src\"]\n",
    );
    write(
        &root.join("docs/FS-login.md"),
        &format!("# FS-login: Login\n\n{BODY}"),
    );
    let comment: String = BODY
        .lines()
        .map(|line| format!("///{}{line}\n", if line.is_empty() { "" } else { " " }))
        .collect();
    write(
        &root.join("src/lib.rs"),
        &format!("/// FS-logout: Logout\n///\n{comment}pub struct Logout;\n"),
    );
    let config = load_config(&root).expect("load config");
    scan_findings(&config, &root)
}

/// The answer as `line: answer`.
fn shown(line: usize, answer: &RelativeAnswer) -> String {
    format!("{line}: {answer:?}")
}

#[test]
fn a_doc_comment_body_gets_the_markdown_answer() {
    let findings = scan();
    let declaration = |slug: &str| {
        findings
            .declarations
            .values()
            .flatten()
            .find(|decl| decl.id.slug.as_deref() == Some(slug))
            .expect("declaration")
    };
    // Each relative line's answer as the scan recorded it on `decl`.
    let answers = |decl: &crate::model::Declaration| -> Vec<String> {
        BODY.lines()
            .enumerate()
            .filter(|(_, line)| {
                relative_heading_form(line.trim_start_matches('#').trim()).is_some()
            })
            .map(|(index, _)| {
                let line = index + 3;
                let answer = if let Some((path, _)) = decl
                    .sections
                    .iter()
                    .find(|(_, info)| info.relative && info.line == line)
                {
                    RelativeAnswer::Section(path.clone())
                } else if findings
                    .relative_section_parents
                    .iter()
                    .any(|failed| failed.line == line && failed.owner == decl.id)
                {
                    RelativeAnswer::NoParent
                } else {
                    RelativeAnswer::Refused
                };
                shown(line, &answer)
            })
            .collect()
    };
    let markdown = answers(declaration("login"));
    assert_eq!(
        markdown,
        [
            "5: Section(\"format.1\")",
            "7: Section(\"format.1.2\")",
            "9: Section(\"notes\")",
            "11: NoParent",
            "13: Refused",
            "15: Section(\"notes.a\")",
        ]
    );
    assert_eq!(answers(declaration("logout")), markdown);
}
