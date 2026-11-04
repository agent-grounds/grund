//! Focused tests for the bare-heading refusal of fetch's output validator (§FS-fetch.3.3): hashes
//! that end the line are a heading to the next scan (§FS-declarations.checks.unmarked-heading.1),
//! so each case asks the validator and the scanner about the same bytes, and shows what the scan
//! would do with the text after the line.

use super::fetch::validate_fetched_declaration;
use crate::config::load_config;
use crate::testing::{scan_findings, test_root, write};

#[derive(Clone, Copy, Debug)]
enum Home {
    File,
    Folder,
}
use Home::{File, Folder};

/// What the validator says of a declaration holding `line`, and what the next scan makes of
/// those bytes: whether the text after `line` is still in the body, and the unmarked headings.
struct Reading {
    verdict: Result<(), String>,
    text_stays: bool,
    unmarked: usize,
}

fn reading(name: &str, home: Home, line: &str) -> Reading {
    let root = test_root(name);
    let (home_key, depth, path, hashes) = match home {
        File => ("file = \"docs/arch.md\"", 2, "docs/arch.md", "##"),
        Folder => (
            "folder = \"docs/arch\"",
            1,
            "docs/arch/AR-001-thing.md",
            "#",
        ),
    };
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 1\n\n[id]\nnamed_sections = true\n\n\
             [[kinds]]\nkind = \"AR\"\n{home_key}\n\n\
             [scan]\ninclude = [\"docs\"]\nrespect_gitignore = false\n"
        ),
    );
    let output = format!("{hashes} AR-001-thing: Thing\nkept\n{line}\nprose after\n");
    write(&root.join(path), &output);
    let config = load_config(&root).expect("load config");
    let verdict = validate_fetched_declaration(&output, "AR-001-thing", depth, &config.grammar)
        .map_err(|failure| failure.message);
    let findings = scan_findings(&config, &root);
    let declaration = findings
        .declarations
        .values()
        .flatten()
        .next()
        .expect("the output declares the requested ID");
    Reading {
        verdict,
        text_stays: declaration.body_end >= output.lines().count(),
        unmarked: findings.unmarked_headings.len(),
    }
}

fn bare_refusal(heading: &str) -> String {
    format!(
        "fetch output line 3: heading `{heading}` has no text and cannot stay in `AR-001-thing`; \
         fence it or drop it to keep the text after it"
    )
}

fn validate(name: &str, depth: usize, text: &str) -> Result<(), String> {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n[id]\nnamed_sections = true\n\n\
         [[kinds]]\nkind = \"AR\"\nfile = \"docs/arch.md\"\n",
    );
    let config = load_config(&root).expect("load config");
    validate_fetched_declaration(text, "AR-001-thing", depth, &config.grammar)
        .map_err(|failure| failure.message)
}

/// At or above the native depth: the declaration ends at the line.
const CLOSERS: [(Home, &str); 7] = [
    (File, "##"),
    (File, "#"),
    (File, "   ##"),
    (File, "  #"),
    // A provider's CRLF: the line ending is no text, to the scan or to the validator.
    (File, "##\r"),
    (Folder, "#"),
    (Folder, "   #"),
];

/// Deeper than subsection depth: the text stays, and the next scan calls the line unmarked.
const BELOW_SUBSECTION: [(Home, &str); 6] = [
    (File, "####"),
    (File, "#####"),
    (File, "######"),
    (File, "   ####"),
    (Folder, "###"),
    (Folder, "######"),
];

/// Why the refusal is owed, asked of the scanner alone, so it holds before the fix and after.
#[test]
fn the_next_scan_reads_each_bare_heading_as_a_heading() {
    for (index, (home, line)) in CLOSERS.into_iter().enumerate() {
        let read = reading(&format!("fetch_bare_closer_{index}"), home, line);
        assert!(
            !read.text_stays,
            "the next scan keeps the text after {line:?} under {home:?}"
        );
    }
    for (index, (home, line)) in BELOW_SUBSECTION.into_iter().enumerate() {
        let read = reading(&format!("fetch_bare_deep_{index}"), home, line);
        assert!(
            read.text_stays,
            "the next scan drops the text after {line:?} under {home:?}"
        );
        assert_eq!(read.unmarked, 1, "the next scan of {line:?} under {home:?}");
    }
}

/// Every row is asked before deciding, so a failure lists each line the validator gets wrong.
#[test]
#[ignore = "red until #1163 refuses a bare heading in fetch output; implement removes this"]
fn a_bare_heading_is_refused_naming_its_line_and_the_way_to_keep_the_text() {
    let wrong = CLOSERS
        .into_iter()
        .chain(BELOW_SUBSECTION)
        .enumerate()
        .filter_map(|(index, (home, line))| {
            let verdict = reading(&format!("fetch_bare_refused_{index}"), home, line).verdict;
            (verdict != Err(bare_refusal(line.trim_end())))
                .then(|| format!("{home:?} {line:?}: {verdict:?}"))
        })
        .collect::<Vec<_>>();
    assert!(
        wrong.is_empty(),
        "not refused as §FS-fetch.3.3 says: {wrong:#?}"
    );
}

/// The slice that reads a declaration's title starts one byte past the hashes, which a bare
/// heading at the native depth does not have: reaching it would be a panic and not a refusal.
#[test]
#[ignore = "red until #1163 refuses a bare heading in fetch output; implement removes this"]
fn a_bare_heading_that_ends_the_output_is_refused_without_a_panic() {
    assert_eq!(
        validate(
            "fetch_bare_last_line",
            2,
            "## AR-001-thing: Thing\nkept\n##"
        ),
        Err(bare_refusal("##"))
    );
}

#[test]
fn a_line_the_next_scan_reads_as_content_stays_accepted_with_the_text_after_it() {
    for (index, (home, line)) in [
        (File, "    ##"),
        (File, "\t##"),
        (File, "#######"),
        (File, "##foo"),
        (File, "```\n##\n```"),
        (File, "~~~\n#\n~~~"),
        (File, "<pre>\n##\n</pre>"),
        (Folder, "    #"),
        (Folder, "\t#"),
        (Folder, "#######"),
        (Folder, "#foo"),
        (Folder, "```\n#\n```"),
    ]
    .into_iter()
    .enumerate()
    {
        let read = reading(&format!("fetch_bare_content_{index}"), home, line);
        assert_eq!(read.verdict, Ok(()), "{line:?}");
        assert!(
            read.text_stays,
            "the next scan drops the text after {line:?}"
        );
        assert_eq!(read.unmarked, 0, "the next scan of {line:?}");
    }
}

/// §FS-fetch.3.2 owns a bare heading at subsection depth, and the next scan calls it unmarked.
#[test]
fn a_bare_heading_at_subsection_depth_keeps_its_refusal() {
    for (index, (home, line)) in [(File, "###"), (Folder, "##")].into_iter().enumerate() {
        let read = reading(&format!("fetch_bare_subsection_{index}"), home, line);
        assert_eq!(read.unmarked, 1, "the next scan of {line:?}");
        assert_eq!(
            read.verdict,
            Err(format!(
                "fetch output line 3: heading `{line}` is not a section of `AR-001-thing`; \
                 number it, fence it, or make it a bold label"
            ))
        );
    }
}

/// Hashes followed by whitespace were never prose to the validator: it reads them as a
/// declaration heading with no title, and refuses that (§FS-fetch.3).
#[test]
fn hashes_followed_only_by_whitespace_stay_refused() {
    for (index, line) in ["## ", "##\t", "##   "].into_iter().enumerate() {
        let name = format!("fetch_bare_whitespace_{index}");
        let refusal = reading(&name, File, line).verdict;
        assert_eq!(
            refusal,
            Err("fetch output declaration needs `: <title>`".to_string()),
            "{line:?}"
        );
    }
}

/// These are why the fix adds to the validator's heading test and does not replace it with
/// `markdown_heading_level`, which reads none of them as a heading. A replacement would accept
/// `    #### Foo` and `####### foo`, and would read `    ## AR-002-other: Other` as prose though
/// the scanner's declaration pattern, which opens with `^\s*`, still records a second one.
#[test]
fn lines_a_wholesale_swap_to_the_scanners_reading_would_accept_stay_refused() {
    for (index, (line, message)) in [
        (
            "    ## AR-002-other: Other",
            "fetch output must contain exactly one declaration and no leading content",
        ),
        (
            "    #### Foo",
            "fetch output heading depth 4 is invalid for a depth-2 declaration",
        ),
        (
            "####### foo",
            "fetch output heading depth 7 is invalid for a depth-2 declaration",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let name = format!("fetch_bare_swap_{index}");
        let text = format!("## AR-001-thing: Thing\nkept\n{line}\nprose after\n");
        assert_eq!(
            validate(&name, 2, &text),
            Err(message.to_string()),
            "{line:?}"
        );
    }
}

/// Ahead of the declaration a bare heading is leading content (§FS-fetch.3.3). At the native
/// depth it reaches the same slice as above, so this also pins that it stays a refusal.
#[test]
fn a_bare_heading_ahead_of_the_declaration_stays_refused_without_a_panic() {
    for (index, (depth, text)) in [
        (2, "##\n## AR-001-thing: Thing\nkept\n"),
        (2, "##"),
        (2, "#\n## AR-001-thing: Thing\n"),
        (2, "   ##\n## AR-001-thing: Thing\n"),
        (2, "###\n## AR-001-thing: Thing\n"),
        (2, "\n\n##\n## AR-001-thing: Thing\n"),
        (1, "#\n# AR-001-thing: Thing\n"),
        (1, "##\n# AR-001-thing: Thing\n"),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(
            validate(&format!("fetch_bare_leading_{index}"), depth, text),
            Err("fetch output must begin with one Markdown declaration".to_string()),
            "{text:?}"
        );
    }
}
