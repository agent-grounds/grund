//! A grammar settled for the thread that searches with it (§AR-lsp.2.1.1): every pattern that
//! compiles on first use compiled, and every pattern's search cache taken for the calling thread.
//! The language server settles each snapshot it installs, on its event loop and outside the
//! keystroke it counts, so the first key after an install builds nothing the key before it did
//! not. `Grammar::build` never settles: most commands never meet the lazy patterns, and compiling
//! them up front cost every read command 3-5% (§GOAL-fast-feedback).

use once_cell::sync::OnceCell;
use regex::Regex;

use super::compiled::Grammar;

impl Grammar {
    /// Compile every pattern still to compile, and take each pattern's search cache for this
    /// thread. A search iterator takes the cache from the pattern's pool before it searches
    /// anything, and gives it back for this thread's next search when it drops, so the empty
    /// haystack still settles a pattern no empty haystack can match.
    pub(crate) fn settle(&self) {
        for regex in self.eager().chain(self.lazy()) {
            drop(regex.find_iter(""));
        }
    }

    /// How many of the patterns that compile on first use have not compiled yet: none once
    /// settled.
    pub(crate) fn unsettled(&self) -> usize {
        self.cells().filter(|cell| cell.get().is_none()).count()
    }

    /// Every pattern `build` compiled.
    fn eager(&self) -> impl Iterator<Item = &Regex> {
        let legacy = &self.legacy;
        [
            &self.decl_re,
            &self.docstring_decl_re,
            &self.section_suffix_re,
            &self.section_re,
            &self.citation_re,
            &self.citation_prefix_re,
            &self.citation_whole_re,
            &self.id_input_re,
            &legacy.decl_re,
            &legacy.docstring_decl_re,
            &legacy.section_path_re,
        ]
        .into_iter()
        .chain(self.kind_parsers.iter().map(|(_, regex)| regex))
    }

    /// Every pattern that compiles on first use, compiled.
    fn lazy(&self) -> impl Iterator<Item = &Regex> {
        let shorthands = self.shorthand.iter().chain(&self.override_shorthands);
        shorthands
            .flat_map(|short| {
                [
                    short.full_prefix_re(),
                    short.prefix_re(),
                    short.unqualified_prefix_re(),
                    short.number_prefix_re(),
                ]
            })
            .chain(
                self.near_misses
                    .iter()
                    .flat_map(|near| [near.decl_re(), near.docstring_re()]),
            )
    }

    /// The cells the patterns that compile on first use are kept in.
    fn cells(&self) -> impl Iterator<Item = &OnceCell<Regex>> {
        let shorthands = self.shorthand.iter().chain(&self.override_shorthands);
        shorthands
            .flat_map(|short| {
                [
                    &short.full_prefix_re,
                    &short.prefix_re,
                    &short.unqualified_prefix_re,
                    &short.number_prefix_re,
                ]
            })
            .chain(
                self.near_misses
                    .iter()
                    .flat_map(|near| [&near.decl_re, &near.docstring_re]),
            )
    }
}
