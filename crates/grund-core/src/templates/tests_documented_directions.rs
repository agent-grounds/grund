//! Test module: the citation-directions guide's examples stay true to the
//! production renderer (§FS-init.2.3.5). The page is what `grund guide citations`
//! prints, so an example that drifted from the render would teach a section no
//! `grund init` writes.

use std::path::Path;

use super::*;
use crate::config::load_config;
use crate::grammar::markdown_fence_delimiter;
use crate::testing::{test_root, write};

/// §FS-init.2.3.5: the documented config/render pair must stay an exact
/// example of the production Citation directions renderer. The extractor
/// follows the shared fence state machine so another fence style or a
/// shorter closing run cannot make the test compare the wrong block.
#[test]
fn documented_citation_directions_example_matches_production_render() {
    let page_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/user-facing/citation-directions.md");
    let page = std::fs::read_to_string(&page_path).expect("read canonical page");
    let documented_config = fenced_example(&page, "toml");
    let documented_render = fenced_example(&page, "markdown");

    let root = test_root("documented_citation_directions_example_matches_production_render");
    // The page intentionally shows only the citation tables. Add the
    // three homes needed to give those tables their configured meanings;
    // the citation fragment itself is read verbatim from the page above.
    write(
        &root.join("grund.toml"),
        &format!(
            "[[kinds]]\nkind = \"GOAL\"\nfile = \"docs/goals.md\"\n\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs/functional-spec\"\n\n[[kinds]]\nkind = \"AR\"\nfolder = \"docs/architecture\"\n\n[[kinds]]\nkind = \"DA\"\nfolder = \"docs/decisions/architectural\"\n\n[[kinds]]\nkind = \"skill\"\nfolder = \"skills\"\ncitable = false\n\n{documented_config}"
        ),
    );
    let config = load_config(&root).expect("load documented config");
    let rendered = citation_directions_section(config.project(), config.run())
        .expect("the documented config declares rules");
    let expected = documented_render
        .strip_suffix('\n')
        .expect("render example ends with a newline");
    assert_eq!(rendered, expected);
}

fn fenced_example(page: &str, language: &str) -> String {
    let mut open = None;
    let mut target = false;
    let mut body = String::new();
    for raw_line in page.split_inclusive('\n') {
        let line = raw_line
            .strip_suffix('\n')
            .unwrap_or(raw_line)
            .strip_suffix('\r')
            .unwrap_or_else(|| raw_line.strip_suffix('\n').unwrap_or(raw_line));
        let was_open = open.is_some();
        let delimiter = markdown_fence_delimiter(&mut open, line);
        if !was_open && delimiter {
            target = fence_language(line) == Some(language);
            continue;
        }
        if was_open && delimiter {
            if target {
                return body;
            }
            target = false;
            continue;
        }
        if target {
            body.push_str(raw_line);
        }
    }
    panic!("missing fenced {language} example");
}

fn fence_language(line: &str) -> Option<&str> {
    let bytes = line.as_bytes();
    let indent = bytes.iter().take_while(|byte| **byte == b' ').count();
    if indent > 3 || indent == bytes.len() {
        return None;
    }
    let delimiter = bytes[indent];
    if delimiter != b'`' && delimiter != b'~' {
        return None;
    }
    let run = bytes[indent..]
        .iter()
        .take_while(|byte| **byte == delimiter)
        .count();
    (run >= 3).then(|| line[indent + run..].trim())
}
