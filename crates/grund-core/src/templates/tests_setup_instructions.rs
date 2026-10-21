//! Test module: the setup skill's core and its reference sections, the parts
//! `grund guide` prints, and what its front page lists of them (§FS-init.5,
//! §FS-init.5.5, §FS-init.5.6).

use super::{
    AGENT_SETUP_INSTRUCTIONS, AGENT_SETUP_SECTIONS, SETUP_CORE_SECTION, SetupPart,
    canonical_template_text, guide_sections, setup_fresh_path, setup_instructions,
};

/// §FS-init.5.5: the section `setup` is the core, `SKILL.md`, and the core stays
/// within its budget.
#[test]
fn core_is_skill_md_within_its_budget() {
    let core = setup_instructions(SetupPart::Section(SETUP_CORE_SECTION)).expect("the core");
    assert_eq!(core, canonical_template_text(AGENT_SETUP_INSTRUCTIONS));
    assert!(core.len() <= 16_000, "the core is {} bytes", core.len());
}

/// §FS-init.5: each section name prints its own file, and any other name prints
/// nothing, which the caller refuses with the table's names.
#[test]
fn a_section_prints_its_file_and_an_unknown_name_is_none() {
    for (name, text) in AGENT_SETUP_SECTIONS {
        assert_eq!(
            setup_instructions(SetupPart::Section(name)).as_deref(),
            Some(text),
            "{name}"
        );
    }
    for unknown in ["rule", "Rules", "", "SKILL", "references/rules.md"] {
        assert_eq!(
            setup_instructions(SetupPart::Section(unknown)),
            None,
            "{unknown:?}"
        );
    }
}

/// §FS-init.5.5: `--full` is the core, then each section in table order, one
/// newline between consecutive files.
#[test]
fn full_is_the_core_then_every_section_in_order() {
    let mut expected =
        setup_instructions(SetupPart::Section(SETUP_CORE_SECTION)).expect("the core");
    for (name, _) in AGENT_SETUP_SECTIONS {
        expected.push('\n');
        expected.push_str(&setup_instructions(SetupPart::Section(name)).expect(name));
    }
    assert_eq!(setup_instructions(SetupPart::Full), Some(expected));
}

/// §FS-init.5.5: every file ends in one newline and no blank line, so the one
/// newline `--full` puts between two files reads as one blank line.
#[test]
fn every_file_ends_in_exactly_one_newline() {
    let files = std::iter::once(("SKILL.md", AGENT_SETUP_INSTRUCTIONS)).chain(AGENT_SETUP_SECTIONS);
    for (name, text) in files {
        assert!(text.ends_with('\n') && !text.ends_with("\n\n"), "{name}");
    }
}

/// §FS-init.5.1, §FS-init.5.5: the core's hand-written Contents names the table's
/// sections in the table's order, so the list a reader is shown is the list the
/// command answers to (§AR-system.principle).
#[test]
fn contents_names_the_table_in_order() {
    let contents = AGENT_SETUP_INSTRUCTIONS
        .split_once("\n## Contents\n")
        .expect("the core has a Contents section")
        .1
        .split("\n## ")
        .next()
        .unwrap();
    let mut named: Vec<&str> = Vec::new();
    for occurrence in contents.split("`grund guide ").skip(1) {
        let name = occurrence
            .split(|c: char| !(c.is_ascii_lowercase() || c == '-'))
            .next()
            .unwrap();
        if !name.is_empty() && !name.starts_with('-') && !named.contains(&name) {
            named.push(name);
        }
    }
    let table: Vec<&str> = AGENT_SETUP_SECTIONS.iter().map(|(name, _)| *name).collect();
    assert_eq!(named, table);
}

/// §FS-init.5.1: Contents opens by naming the core's own `##` sections, in order,
/// so a section moved out under §FS-init.5.5's budget cannot stay listed as core.
#[test]
fn contents_names_the_cores_own_sections_in_order() {
    let mut headings: Vec<&str> = Vec::new();
    let mut fence: Option<&str> = None;
    for line in AGENT_SETUP_INSTRUCTIONS.lines() {
        let trimmed = line.trim();
        let marker = trimmed.len() - trimmed.trim_start_matches(['`', '~']).len();
        if marker >= 3 {
            match fence {
                None => fence = Some(&trimmed[..marker]),
                Some(open) if trimmed == open => fence = None,
                Some(_) => {}
            }
        } else if fence.is_none()
            && let Some(title) = line.strip_prefix("## ")
        {
            headings.push(title.trim());
        }
    }
    let list = match headings.split_last() {
        Some((last, [])) => last.to_string(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
        None => panic!("the core has no `##` heading"),
    };
    let contents = AGENT_SETUP_INSTRUCTIONS
        .split_once("\n## Contents\n")
        .expect("the core has a Contents section")
        .1;
    let sentence = format!("This core, the section `{SETUP_CORE_SECTION}`, holds {list}.");
    assert!(
        contents.trim_start().starts_with(&sentence),
        "Contents does not open with `{sentence}`"
    );
}

/// §FS-init.5.5, §FS-init.5.6: the guide's sections are `setup` and then the table,
/// each with the bytes it prints and a summary, a reference section's being the
/// phrase its Contents line ends with.
#[test]
fn guide_sections_are_setup_then_the_table_with_their_summaries() {
    let sections = guide_sections();
    let names: Vec<&str> = sections.iter().map(|section| section.name).collect();
    let mut expected = vec![SETUP_CORE_SECTION];
    expected.extend(AGENT_SETUP_SECTIONS.iter().map(|(name, _)| *name));
    assert_eq!(names, expected);
    for section in &sections {
        let printed = setup_instructions(SetupPart::Section(section.name)).expect(section.name);
        assert_eq!(section.bytes, printed.len(), "{}", section.name);
        assert!(
            !section.summary.is_empty(),
            "{} has no summary",
            section.name
        );
        assert!(!section.summary.ends_with('.'), "{}", section.name);
        if section.name != SETUP_CORE_SECTION {
            let line = format!(
                "- `grund guide {0}` (`references/{0}.md`): {1}.",
                section.name, section.summary
            );
            assert!(
                AGENT_SETUP_INSTRUCTIONS.lines().any(|l| l == line),
                "{line}"
            );
        }
    }
}

/// §FS-init.5.6: the front page's fresh path is the core's Fresh repository section,
/// from its heading up to the next `##` heading.
#[test]
fn fresh_path_is_the_cores_fresh_repository_section() {
    let fresh = setup_fresh_path();
    assert!(fresh.starts_with("## Fresh repository\n"), "{fresh}");
    assert!(fresh.contains("grund init --scaffold"), "{fresh}");
    assert!(!fresh.contains("\n## "), "{fresh}");
    let core = canonical_template_text(AGENT_SETUP_INSTRUCTIONS);
    assert!(core.contains(&format!("{fresh}## Contents\n")), "{fresh}");
}
