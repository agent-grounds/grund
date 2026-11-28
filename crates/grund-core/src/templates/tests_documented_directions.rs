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
    // The page shows one complete version-2 config, homes included, so the
    // test writes it verbatim as the repository's grund.toml.
    write(&root.join("grund.toml"), &documented_config);
    let config = load_config(&root).expect("load documented config");
    let rendered = citation_directions_section(config.project(), config.run())
        .expect("the documented config declares rules");
    let expected = documented_render
        .strip_suffix('\n')
        .expect("render example ends with a newline");
    assert_eq!(rendered, expected);
}

/// §FS-init.2.3.5: the page's closed-row example, its
/// `[citations.fixture]` table, renders exactly the one bullet the page shows
/// beside it, so the page cannot teach a closed place a sentence no `grund init`
/// writes (§FS-config.3.9.4.1).
#[test]
fn documented_closed_row_example_matches_production_render() {
    let page_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/user-facing/citation-directions.md");
    let page = std::fs::read_to_string(&page_path).expect("read canonical page");
    let table = fenced_examples(&page, "toml")
        .into_iter()
        .find(|example| example.starts_with("[citations.fixture]"))
        .expect("a closed-row example");
    let documented = fenced_examples(&page, "markdown")
        .into_iter()
        .find(|example| example.contains("**tests/fixtures/**"))
        .expect("the closed row's bullet");
    let root = test_root("documented_closed_row_example_matches_production_render");
    write(
        &root.join("grund.toml"),
        &format!(
            "[[kinds]]\nkind = \"FS\"\nfolder = \"docs/functional-spec\"\n\n[[kinds]]\n\
             kind = \"fixture\"\nfolder = \"tests/fixtures\"\ncitable = false\n\n{table}"
        ),
    );
    let config = load_config(&root).expect("load the closed-row example");
    let rendered = citation_directions_section(config.project(), config.run())
        .expect("the example declares a rule");
    let bullet = documented
        .strip_suffix('\n')
        .expect("bullet example ends with a newline");
    assert!(rendered.lines().any(|line| line == bullet), "{rendered}");
}

/// §FS-init.2.3.5: the climbing convention the block no longer states is the
/// page's recommended starting ruleset, its `toml` example that opens with the
/// GOAL table; later sections carry toml examples of their own. Over the kinds
/// the starter config declares (§FS-init.2.4) it loads, and every table it
/// writes renders a bullet, so the page cannot teach a rule `grund init` drops.
#[test]
fn the_recommended_starting_ruleset_loads_over_the_starter_kinds() {
    let page_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/user-facing/citation-directions.md");
    let page = std::fs::read_to_string(&page_path).expect("read canonical page");
    let ruleset = fenced_examples(&page, "toml")
        .into_iter()
        .find(|example| example.starts_with("[rules.citations.GOAL]"))
        .expect("a recommended starting ruleset");
    let root = test_root("the_recommended_starting_ruleset_loads_over_the_starter_kinds");
    let starter = render_grund_toml(Template::SoftwareDevelopment, "demo", None);
    write(&root.join("grund.toml"), &format!("{starter}\n{ruleset}"));
    let config = load_config(&root).expect("load the starter config with the ruleset");
    let rendered = citation_directions_section(config.project(), config.run())
        .expect("the ruleset declares rules");
    let tables = ruleset.matches("[rules.citations.").count();
    let bullets = rendered
        .lines()
        .filter(|line| line.starts_with("- "))
        .count();
    assert_eq!(bullets, tables, "one bullet per table:\n{rendered}");
    assert!(
        rendered.contains("- Each **FS** declaration should cite GOAL or FS."),
        "{rendered}"
    );
}

/// The first fenced `language` example on `page`, the one a reader meets first.
fn fenced_example(page: &str, language: &str) -> String {
    fenced_examples(page, language)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("missing fenced {language} example"))
}

/// Every fenced `language` example on `page`, in page order.
fn fenced_examples(page: &str, language: &str) -> Vec<String> {
    let mut open = None;
    let mut target = false;
    let mut body = String::new();
    let mut examples = Vec::new();
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
                examples.push(std::mem::take(&mut body));
            }
            target = false;
            continue;
        }
        if target {
            body.push_str(raw_line);
        }
    }
    examples
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
