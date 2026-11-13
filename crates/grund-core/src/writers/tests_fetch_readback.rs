//! Focused tests for the home as admission reads it after the integration (§FS-fetch.6.1,
//! §FS-fetch.admission.3): a finding the home already held is matched by the lines it records,
//! never by a number in the text it quotes, and the reading after the integration reads the
//! candidate and nothing else, so what the integration writes elsewhere plays no part.

use super::fetch_admit::admit;
use super::fetch_target::{Before, Home, Request, place};
use super::tests_fetch_admit::{FOLDER_TOML, file_toml, into_file};
use crate::config::load_config;
use crate::grammar::parse_id_arg;
use crate::scanner::scan_tree;
use crate::testing::{test_root, write};

/// Where a case's home is, relative to its root.
pub(super) enum At {
    File(&'static str),
    Folder(&'static str),
}

/// What admission makes of `output` for `local` in a tree of `files`, with `during` written
/// after the scan and before admission, as an integration writes while it runs: the bytes it
/// would install, or the refusal's line.
pub(super) fn admit_during(
    name: &str,
    toml: &str,
    at: At,
    files: &[(&str, &str)],
    during: &[(&str, &str)],
    local: &str,
    output: &str,
) -> Result<String, String> {
    let root = test_root(name);
    write(&root.join("grund.toml"), toml);
    for (path, text) in files {
        write(&root.join(path), text);
    }
    let config = load_config(&root).expect("load config");
    let (id, _) = parse_id_arg(local, &config.grammar).expect("parse the requested ID");
    let (catalog, errors) =
        scan_tree(config.schema(), config.frame(), None, false).expect("scan the tree");
    let before = Before { catalog, errors };
    let home = match at {
        At::File(file) => Home::File(config.root.join(file)),
        At::Folder(folder) => Home::Folder(config.root.join(folder)),
    };
    let request = Request {
        config: &config,
        id: &id,
        local,
        report_base: &config.root,
        home,
    };
    let mut placing = None;
    let placed = place(&request, &before, &mut placing).map_err(|failure| (failure, placing));
    for (path, text) in during {
        write(&root.join(path), text);
    }
    let mut diagnostic = None;
    admit(&request, &before, placed, output, &mut diagnostic)
        .map(|admitted| admitted.bytes)
        .map_err(|failure| failure.message)
}

// §FS-fetch.admission.3: a held finding keeps its key however its own text reads.

#[test]
fn a_number_in_a_held_finding_s_text_is_no_line_of_the_target() {
    let output = "## AR-001-thing: Thing\n\nnew\nlonger\nbody\n";
    // Each heading is a finding the home already holds, below the replaced range: `Pipeline 3`
    // holds `line 3`, `99` is past the end of the file, and `2024–2025` reads as a range.
    for (index, heading) in [
        "Pipeline 3",
        "Pipeline 2",
        "Timeline 99",
        "Releases 2024–2025",
        "Phases 1–3",
    ]
    .into_iter()
    .enumerate()
    {
        let tail = format!("## AR-002-other: Other\n\nother\n\n### {heading}\n\na\n");
        let home = format!("## AR-001-thing: Thing\n\nold\n\n{tail}");
        assert_eq!(
            into_file(&format!("readback_quoted_number_{index}"), &home, output),
            Ok(format!("{output}\n{tail}")),
            "{heading}"
        );
    }
}

#[test]
fn a_held_duplicate_names_its_sites_below_the_output_however_far_they_move() {
    // The duplicate's other site, a duplicate section and a conflict block are all named by
    // line, below the range the output lengthens.
    let tail = "## AR-002-other: Other\n\n### 1. One\n\n### 1. Again\n\n<<<<<<< ours\na\n\
                =======\nb\n>>>>>>> theirs\n\n## AR-002-other: Twice\n";
    let home = format!("## AR-001-thing: Thing\n\nold\n\n{tail}");
    let output = "## AR-001-thing: Thing\n\nnew\nlonger\nbody\n";
    assert_eq!(
        into_file("readback_held_sites", &home, output),
        Ok(format!("{output}\n{tail}"))
    );
}

// §FS-fetch.6.1: what the integration writes beside the target is never read.

#[test]
fn a_file_outside_a_file_home_written_during_the_integration_plays_no_part() {
    let home = "## AR-001-thing: Thing\n\nold\n\n## AR-002-other: Other\n\nother\n";
    let output = "## AR-001-thing: Thing\n\nnew\n";
    let installed = admit_during(
        "readback_outside_file_home",
        &file_toml(true),
        At::File("docs/arch.md"),
        &[("docs/arch.md", home)],
        &[("docs/zzz.md", "## AR-002-other: Copy\n\ncopy\n")],
        "AR-001-thing",
        output,
    );
    assert_eq!(
        installed,
        Ok("## AR-001-thing: Thing\n\nnew\n\n## AR-002-other: Other\n\nother\n".to_string())
    );
}

#[test]
fn another_file_of_a_folder_home_changed_or_created_during_the_integration_plays_no_part() {
    let output = "# AR-001-thing: Thing\n\nnew\n";
    let files = [
        ("docs/ar/AR-001-thing.md", "# AR-001-thing: Old\n\nold\n"),
        (
            "docs/ar/AR-002-other.md",
            "# AR-002-other: Other\n\nother\n",
        ),
    ];
    let conflicted = "# AR-002-other: Other\n\n<<<<<<< ours\na\n=======\nb\n>>>>>>> theirs\n";
    let copy = "# AR-001-thing: Copy\n\ncopy\n";
    for (index, during) in [
        ("docs/ar/AR-002-other.md", conflicted),
        ("docs/ar/AR-009-copy.md", copy),
    ]
    .into_iter()
    .enumerate()
    {
        let installed = admit_during(
            &format!("readback_folder_sibling_{index}"),
            FOLDER_TOML,
            At::Folder("docs/ar"),
            &files,
            &[during],
            "AR-001-thing",
            output,
        );
        assert_eq!(installed.as_deref(), Ok(output), "{}", during.0);
    }
}
