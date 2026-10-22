//! A managed `AGENTS.md` block cut into the parts a stale finding names
//! (§FS-check.3.5.3): the citation scheme, which is everything above the first
//! `###` heading, then one part per `###` heading up to the next or to the
//! `<!-- END GRUND MANAGED BLOCK -->` line. The cut is lexical and reads both
//! sides alike, the render `Expected` carries and the block on disk, so the
//! comparison of §FS-check.3.5.4 covers every byte between the delimiters.

use super::managed_block::AGENTS_BLOCK_END;

/// The noun of the part above the first `###` heading (§FS-init.2.3.4.2).
pub(crate) const CITATION_SCHEME: &str = "citation scheme";

/// The two sections only the outermost root of a full checkout compares
/// (§FS-check.3.5.5), by their nouns.
pub(crate) const WORKSPACE_SECTIONS: [&str; 2] = ["project namespaces", "workspace members"];

/// One part of a managed block: the `###` heading line that opens it, empty for
/// the citation scheme, and its bytes, the heading and its trailing blank lines
/// included.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BlockSection<'a> {
    pub(crate) heading: &'a str,
    pub(crate) bytes: &'a str,
}

impl BlockSection<'_> {
    /// The lower-case noun a stale finding names this part by (§FS-check.3.5.3).
    pub(crate) fn noun(&self) -> String {
        match self.heading.strip_prefix("### ") {
            Some(title) => title.trim().to_lowercase(),
            None => CITATION_SCHEME.to_string(),
        }
    }
}

/// `block`, from its begin delimiter to its end delimiter, cut into its parts
/// in order (§FS-check.3.5.3). The parts partition every byte before the end
/// delimiter, so two blocks whose parts agree in order and in bytes are equal.
pub(crate) fn block_sections(block: &str) -> Vec<BlockSection<'_>> {
    let end = AGENTS_BLOCK_END
        .find(block)
        .map_or(block.len(), |found| found.start());
    let body = &block[..end];
    let mut starts = vec![0];
    let mut offset = 0;
    for line in body.split_inclusive('\n') {
        if offset > 0 && line.starts_with("### ") {
            starts.push(offset);
        }
        offset += line.len();
    }
    starts.push(body.len());
    starts
        .windows(2)
        .map(|span| {
            let bytes = &body[span[0]..span[1]];
            let heading = if span[0] == 0 && !bytes.starts_with("### ") {
                ""
            } else {
                bytes.lines().next().unwrap_or_default()
            };
            BlockSection { heading, bytes }
        })
        .collect()
}
