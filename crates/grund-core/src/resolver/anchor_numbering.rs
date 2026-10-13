//! A repeated heading's anchor (§DF-repeated-heading-anchor.decision.count): the
//! renderer numbers a slug it has already handed out in the same file, so the
//! anchor of a heading is its slug numbered against every heading the file
//! renders before it. The headings are the scan's record of the home, which holds
//! none inside a fence, so the count is the renderer's and no file is read again.

use std::collections::{BTreeMap, BTreeSet};

use super::settled_headings::{SettledLines, settled_heading_text};
use crate::grammar::{anchor_slug, reduce_heading_text, without_closing_sequence};
use crate::model::FileRecord;

/// `base`, the slug of the heading on `line` of `record`, numbered as the
/// renderer numbers it: unchanged for the first heading of that slug in the
/// file, `<base>-1`, `<base>-2`, … under `github`, `gitlab` and `pandoc`, and
/// `<base>_1`, `<base>_2`, … under `mkdocs` (§FS-fmt.6.2.3,
/// §DF-repeated-heading-anchor.decision.count). Without a record, or where the
/// record has no heading on `line`, `base` comes back as it is. Each heading is
/// counted with the text the run leaves it (`settled`, §FS-fmt.6.3.3).
pub(crate) fn numbered_anchor(
    record: Option<&FileRecord>,
    settled: Option<&SettledLines>,
    line: usize,
    base: String,
    profile: &str,
) -> String {
    let Some(record) = record.filter(|record| record.markdown) else {
        return base;
    };
    let headings = record.headings.iter().filter(|heading| heading.atx);
    // The first heading of a file takes its slug as it is: nothing to slug.
    if headings
        .clone()
        .next()
        .is_none_or(|first| first.line >= line)
    {
        return base;
    }
    let mut slugs = Slugs::new(profile);
    for heading in headings {
        let slug = anchor_slug(
            without_closing_sequence(&reduce_heading_text(settled_heading_text(
                record, settled, heading,
            ))),
            profile,
        );
        let numbered = slugs.take(slug.clone());
        if heading.line == line {
            // The suffix the count gave this heading, on the slug its caller derived.
            return match numbered.strip_prefix(slug.as_str()) {
                Some(suffix) => format!("{base}{suffix}"),
                None => base,
            };
        }
    }
    base
}

/// The slugs a renderer has handed out so far in one file.
struct Slugs {
    mkdocs: bool,
    counts: BTreeMap<String, usize>,
    taken: BTreeSet<String>,
}

impl Slugs {
    fn new(profile: &str) -> Self {
        Self {
            mkdocs: profile == "mkdocs",
            counts: BTreeMap::new(),
            taken: BTreeSet::new(),
        }
    }

    /// Hand out `slug`, numbered past every slug already taken.
    fn take(&mut self, slug: String) -> String {
        if self.mkdocs {
            return self.take_mkdocs(slug);
        }
        // `github-slugger`: a repeat counts up from its last number, skipping
        // any numbered form a literal heading already took.
        let Some(count) = self.counts.get(&slug).copied() else {
            self.counts.insert(slug.clone(), 0);
            self.taken.insert(slug.clone());
            return slug;
        };
        let mut count = count;
        let numbered = loop {
            count += 1;
            let candidate = format!("{slug}-{count}");
            if !self.taken.contains(&candidate) {
                break candidate;
            }
        };
        self.counts.insert(slug, count);
        self.counts.insert(numbered.clone(), 0);
        self.taken.insert(numbered.clone());
        numbered
    }

    /// Python-Markdown's `unique`: an id already taken gets `_1`, and one that
    /// already ends in `_<n>` counts that number up, until the id is free.
    fn take_mkdocs(&mut self, slug: String) -> String {
        let mut id = slug;
        while self.taken.contains(&id) || id.is_empty() {
            id = match id.rsplit_once('_') {
                Some((stem, number))
                    if !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()) =>
                {
                    let next = number.parse::<u64>().map_or(1, |n| n + 1);
                    format!("{stem}_{next}")
                }
                _ => format!("{id}_1"),
            };
        }
        self.taken.insert(id.clone());
        id
    }
}

#[cfg(test)]
mod tests {
    use super::Slugs;

    #[test]
    fn github_numbers_a_repeat_and_skips_a_literal_numbered_slug() {
        let mut slugs = Slugs::new("github");
        let handed: Vec<String> = ["a", "a", "a-2", "a", "a"]
            .into_iter()
            .map(|slug| slugs.take(slug.to_string()))
            .collect();
        assert_eq!(handed, ["a", "a-1", "a-2", "a-3", "a-4"]);
    }

    #[test]
    fn mkdocs_numbers_with_an_underscore() {
        let mut slugs = Slugs::new("mkdocs");
        let handed: Vec<String> = ["1-overview", "1-overview", "1-overview"]
            .into_iter()
            .map(|slug| slugs.take(slug.to_string()))
            .collect();
        assert_eq!(handed, ["1-overview", "1-overview_1", "1-overview_2"]);
    }
}
