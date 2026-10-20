//! Test module: the agent block's version heading in its two spellings
//! (§FS-init.2.3.9.3). Inside the delimiters `(block vN)` and the unlabelled `(vN)`
//! of v30 and earlier are both the version heading and carry the same version; outside
//! them only the unlabelled spelling opens a legacy block (§FS-init.2.3.9.2). The CLI
//! suite `init_block_revision_label.rs` pins what `init` and `check` make of it.

use super::*;

fn delimited(heading: &str, eol: &str) -> String {
    format!(
        "# Title{eol}{eol}<!-- BEGIN GRUND MANAGED BLOCK -->{eol}{heading}{eol}{eol}body{eol}\
         <!-- END GRUND MANAGED BLOCK -->{eol}after{eol}"
    )
}

fn found(text: &str) -> (usize, usize, u32) {
    match find_agents_block(text) {
        AgentsBlockLookup::Found(block) => (block.start, block.end, block.version),
        AgentsBlockLookup::Absent => panic!("no block in {text:?}"),
        AgentsBlockLookup::Malformed { message, .. } => panic!("malformed: {message}"),
    }
}

#[test]
fn both_spellings_inside_the_delimiters_carry_the_same_version_and_span() {
    for eol in ["\n", "\r\n"] {
        let old = delimited("## Grounding with grund (v30)", eol);
        let new = delimited("## Grounding with grund (block v30)", eol);
        let (old_start, old_end, old_version) = found(&old);
        let (new_start, new_end, new_version) = found(&new);
        assert_eq!((old_version, new_version), (30, 30), "{eol:?}");
        assert_eq!(new_start, old_start, "{eol:?}");
        // The labelled heading is six bytes longer; the span still ends past END's line.
        assert_eq!(new_end, old_end + "block ".len(), "{eol:?}");
        assert!(new[..new_end].ends_with(&format!("MANAGED BLOCK -->{eol}")));
        assert!(new[new_end..].starts_with("after"), "{eol:?}");
    }
    let (.., future) = found(&delimited("## Grounding with grund (block v99)", "\n"));
    assert_eq!(future, 99, "a newer version is read, so it can be refused");
}

#[test]
fn a_labelled_heading_outside_the_delimiters_opens_no_legacy_block() {
    let text = "# Title\n\n## Grounding with grund (block v3)\n\nmine\n";
    assert!(matches!(find_agents_block(text), AgentsBlockLookup::Absent));
    let (.., legacy) = found("# Title\n\n## Grounding with grund (v3)\n\nlegacy\n");
    assert_eq!(legacy, 3, "the unlabelled legacy form is still recognized");
}

#[test]
fn a_region_with_neither_spelling_stays_malformed_with_its_message() {
    let text = delimited("## Grounding with grund (release v31)", "\n");
    let AgentsBlockLookup::Malformed { message, at } = find_agents_block(&text) else {
        panic!("a region with no version heading is malformed");
    };
    assert_eq!(
        message,
        "no `## Grounding with grund (vN)` heading between the delimiters"
    );
    assert_eq!(at, text.find("<!-- BEGIN").unwrap());
}
