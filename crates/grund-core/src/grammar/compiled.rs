use anyhow::{Result, anyhow};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use super::id_format::{
    IdElement, ShorthandGrammar, id_pattern, literal_after_kind_placeholder, parse_id_format,
    shorthand_elements,
};
use super::id_rules::{
    id_grammar_literal_slash_error, id_grammar_pattern_slash_error, section_separator_slash_error,
};
use super::near_miss::{LegacyGrammar, NearMissGrammar};
use super::settings::{GrammarKind, GrammarSource};
use super::source_line::comment_prefix_regex;
use crate::model::Id;

const NUMERIC_SECTION_PATTERN: &str = r"\d+(?:\.\d+)*";
const NAMED_SECTION_PATTERN: &str = r"[a-z][a-z0-9-]*(?:\.[a-z][a-z0-9-]*)*(?:\.\d+)*";
pub(crate) static STUB_LINK_HEADING: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*:\s*\[[^\]]*\]\(\s*(?P<path>[^)\s]+)\s*\)\s*$").unwrap());
/// The explicit managed-block delimiters (§FS-init.2.3.9,
/// §DF-managed-block-delimiters): standard `BEGIN`/`END` HTML-comment lines
/// bound the managed region from block v4 on. Legacy v3-and-earlier blocks have
/// no delimiters and are found by `AGENTS_BLOCK_H2` alone.
pub(crate) static AGENTS_BLOCK_BEGIN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^<!-- BEGIN GRUND MANAGED BLOCK -->[ \t]*\r?$").unwrap());
pub(crate) static AGENTS_BLOCK_END: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^<!-- END GRUND MANAGED BLOCK -->[ \t]*\r?$").unwrap());
/// The managed block's version marker: an H2 heading carrying the block
/// version. Inside a delimited block it names the schema version; for a legacy
/// block it is also the begin marker, and the block runs until the next H1/H2
/// or EOF (§FS-init.2.3.1).
pub(crate) static AGENTS_BLOCK_H2: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^##[ \t]+Grounding with grund[ \t]+\(v(?P<version>\d+)\)[ \t]*\r?$").unwrap()
});
/// The next H1 or H2 heading after a position — the implicit end of a legacy
/// managed section. A legacy block ends at this line's start, or at EOF if no
/// such line follows.
pub(crate) static AGENTS_SECTION_BOUNDARY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^#{1,2}[ \t]+\S").unwrap());
/// ID grammar compiled from [id].format + [[kinds]] — the single place that knows the
/// shape of a declaration heading or a citation. Built once per config load.
/// Realizes §FS-config.3.1, §FS-config.3.2, §FS-config.3.3 and the regex-not-a-parser
/// stance of §AR-scanner.5.
/// The pattern one alias segment must match (§FS-workspace.1.1, §AR-workspace.2).
/// One canonical place — also referenced by the config-load alias validator
/// (`is_valid_project_alias` in `config/workspace_block.rs`).
const PROJECT_ALIAS_PATTERN: &str = "[a-z][a-z0-9-]*";
/// The namespace a qualified citation carries: one alias segment per workspace
/// level, so a project nested inside a member workspace is named by its whole
/// chain (§FS-workspace.1.1, §FS-workspace.6.1). Greedy by construction and the
/// ID that follows never contains `/`, so the last `/` in the token is always
/// the boundary between the project path and the ID.
static PROJECT_PATH_PATTERN: Lazy<String> =
    Lazy::new(|| format!("{PROJECT_ALIAS_PATTERN}(?:/{PROJECT_ALIAS_PATTERN})*"));
pub(crate) static QUALIFIED_CITATION_PREFIX: Lazy<Regex> =
    Lazy::new(|| Regex::new(&format!(r"^(?P<namespace>{})/", *PROJECT_PATH_PATTERN)).unwrap());

#[derive(Clone)]
pub struct Grammar {
    pub(super) decl_re: Regex,
    pub(super) docstring_decl_re: Regex,
    /// `\A`, the configured `[id] section_separator` and a section path the section
    /// grammar admits: what follows the ID of a coordinate, read by
    /// [`Grammar::opens_section_suffix`] (§FS-declarations.line.section-suffix.3).
    section_suffix_re: Regex,
    pub(crate) section_re: Regex,
    /// Bare-token detection retains its word boundary (§FS-check.1.1.10).
    /// Both full-ID patterns capture the optional `<namespace>/` prefix
    /// (§FS-workspace.1, §AR-workspace.3.1).
    pub(crate) citation_re: Regex,
    /// A full-ID prefix directly after an explicit marker needs no intervening
    /// word boundary (§FS-check.1.1.10); `citation_captures` merges both paths.
    pub(super) citation_prefix_re: Regex,
    pub(super) id_input_re: Regex,
    /// The compiled grammar remembers the gate so every token consumer can
    /// enforce the same whole-token suppression rule (§AR-scanner.2.3.4).
    pub(crate) named_sections: bool,
    /// The per-kind near-miss patterns (§FS-declarations.checks.declaration-near-miss.1): a heading that opens
    /// with a configured kind and the literal its effective ID format puts
    /// after it, without parsing as an ID. Absent for a kind whose format puts
    /// no literal there — then "looks like a declaration" cannot be told from
    /// prose beginning with the kind name, and the rule declines rather than guess.
    pub(super) near_misses: Vec<NearMissGrammar>,
    pub(super) legacy: LegacyGrammar,
    /// The number-only shorthand patterns (§FS-check.1.2.1, §AR-scanner.2.6),
    /// present only when `[id] format` carries both `{number}` and `{slug}`.
    /// `None` is the whole opt-out: every shorthand pass downstream is gated on
    /// this being `Some`, so a `{kind}-{slug}` repo like `grund` itself compiles
    /// nothing extra and pays nothing (§FS-id.4.1).
    pub(super) shorthand: Option<ShorthandGrammar>,
    /// Number-only shorthand grammars for kinds whose override carries both a
    /// number and a slug (§FS-config.3.2). Kept beside the legacy/default
    /// shorthand so all consumers can select by the token's kind.
    pub(super) override_shorthands: Vec<ShorthandGrammar>,
    /// The parsed `[id] format`. Kept so `render_id` reduces a partial `Id` by
    /// the same rule the shorthand pattern was derived from, rather than a
    /// second interpretation of the template (§AR-scanner.2.6.11).
    pub(super) elements: Vec<IdElement>,
    /// Per-kind full-ID parsers and render templates. Detection uses one union
    /// regex, then parsing selects the already-known kind's exact grammar
    /// (§FS-config.3.2, §FS-config.3.4.10.1).
    kind_parsers: Vec<(String, Regex)>,
    pub(super) kind_elements: BTreeMap<String, Vec<IdElement>>,
    pub(super) overridden_kinds: BTreeSet<String>,
    /// What `build` was handed, for [`Grammar::with_named_sections`].
    pub(super) source: Arc<GrammarSource>,
}

impl Grammar {
    /// Compile the four regexes from the effective config. An empty `kinds` compiles
    /// a grammar that recognizes no ID; refusing that is the v1 reader's call
    /// ([`super::require_kinds`]). The validation rejections
    /// here (`{kind}` required, at least one of `{number}`/`{slug}`, separator must be
    /// lexically distinct) are §FS-config.3.2; the optional `§`-marker prefix on a
    /// citation is §FS-config.3.1 / §DF-reference-marker; the comment-prefix wrapper
    /// on declaration/section regexes is §AR-scanner.4.3 (declarations live in code
    /// doc-comments too).
    ///
    /// The `/` rejections repeat `config/v1/parse.rs` because the whole namespace grammar
    /// rests on "an ID never contains `/`". Each component pattern is compiled on
    /// its own because two patterns whose parentheses balance only against each
    /// other — `number_pattern = "("` with `slug_pattern = "a)"` — compile fine as
    /// one ID pattern and fall apart the moment a pass builds one from a subset of
    /// the elements, as `shorthand_elements` does; rejecting them here lets every
    /// derived pattern be built by string surgery and still be sure to compile.
    ///
    /// A comment prefix stands in for the `#`, so `/// AR-foo: title` matches while
    /// a bare prose line `AR-foo: title` in markdown does not. And a namespace only
    /// ever precedes a citation, never being the second number of a run, which is
    /// why the shorthand's unqualified pattern is a separate string, not a reuse.
    pub(crate) fn build(
        format: &str,
        kinds: &[GrammarKind],
        number_pattern: &str,
        slug_pattern: &str,
        section_separator: &str,
        named_sections: bool,
        comment_prefixes: &[String],
    ) -> Result<Self> {
        // §FS-config.3.2.3: the "an ID never contains `/`" invariant, enforced over
        // every component an ID is built from. `config/v1/parse.rs` rejects each key at its
        // own line first; this is the backstop for a `Config` assembled in code.
        if let Some(message) = id_grammar_literal_slash_error("[id].format", format) {
            return Err(anyhow!("{message}"));
        }
        for (label, pattern) in [
            ("[id].number_pattern", number_pattern),
            ("[id].slug_pattern", slug_pattern),
        ] {
            if let Some(message) = id_grammar_pattern_slash_error(label, pattern) {
                return Err(anyhow!("{message}"));
            }
        }
        for kind in kinds.iter().map(|kind| &kind.name) {
            if let Some(message) =
                id_grammar_literal_slash_error(&format!("[[kinds]] kind `{kind}`"), kind)
            {
                return Err(anyhow!("{message}"));
            }
        }

        let elements = parse_id_format(format)?;
        let mut kind_parsers = Vec::new();
        let mut kind_elements = BTreeMap::new();
        let mut overridden_kinds = BTreeSet::new();
        let mut detection_patterns = Vec::new();
        for kind in kinds {
            let effective = kind.format.as_deref().unwrap_or(format);
            if let Some(message) = id_grammar_literal_slash_error(
                &format!("[[kinds]] kind `{}` format", kind.name),
                effective,
            ) {
                return Err(anyhow!("{message}"));
            }
            let kind_format = parse_id_format(effective)
                .map_err(|err| anyhow!("[[kinds]] kind `{}` format: {err}", kind.name))?;
            let literal_kind = regex::escape(&kind.name);
            let detection = id_pattern(
                &kind_format,
                &literal_kind,
                &format!("(?:{})", number_pattern),
                &format!("(?:{})", slug_pattern),
            );
            let parser = id_pattern(
                &kind_format,
                &literal_kind,
                &format!("(?P<num>{})", number_pattern),
                &format!("(?P<slug>{})", slug_pattern),
            );
            detection_patterns.push(detection);
            kind_parsers.push((kind.name.clone(), Regex::new(&format!("^{parser}$"))?));
            kind_elements.insert(kind.name.clone(), kind_format);
            if kind.format.is_some() {
                overridden_kinds.insert(kind.name.clone());
            }
        }
        // §FS-config-v2.defaults.1: a v2 file with no citable row has no kinds, so
        // its ID pattern is one that never matches (`\b\B`), not an empty one.
        let detection = if detection_patterns.is_empty() {
            r"\b\B".to_string()
        } else {
            detection_patterns.join("|")
        };
        let id_pat = format!("(?P<id>(?:{detection}))");
        let literals: Vec<&String> = elements
            .iter()
            .filter_map(|element| match element {
                IdElement::Literal(text) => Some(text),
                _ => None,
            })
            .collect();
        let has_number = kind_elements
            .values()
            .any(|elements| elements.contains(&IdElement::Number));

        // §FS-config.3.2.2: the section separator must be lexically distinguishable
        // from the ID grammar — otherwise a citation like `FS-foo<sep>bar` could
        // not be split into ID and section unambiguously.
        if section_separator.is_empty() {
            return Err(anyhow!("[id].section_separator must not be empty"));
        }
        if let Some(message) = section_separator_slash_error(section_separator) {
            return Err(anyhow!("{message}"));
        }
        if literals.iter().any(|lit| lit.contains(section_separator)) {
            return Err(anyhow!(
                "[id].section_separator `{section_separator}` collides with a literal in [id].format"
            ));
        }
        for (kind, elements) in &kind_elements {
            if elements.iter().any(|element| {
                matches!(element, IdElement::Literal(literal) if literal.contains(section_separator))
            }) {
                return Err(anyhow!(
                    "[[kinds]] kind `{kind}` format collides with [id].section_separator `{section_separator}`"
                ));
            }
        }
        // §FS-config.3.2.4: each component pattern must be a valid regex *on its
        // own*, not merely valid once spliced into `id_pat` — the number-only
        // shorthand builds one from a subset of the elements (§FS-check.1.2.1).
        let slug_re = Regex::new(slug_pattern)
            .map_err(|err| anyhow!("[id].slug_pattern is not a valid regex: {err}"))?;
        if slug_re.is_match(section_separator) {
            return Err(anyhow!(
                "[id].section_separator `{section_separator}` is matched by [id].slug_pattern"
            ));
        }
        if has_number {
            let number_re = Regex::new(number_pattern)
                .map_err(|err| anyhow!("[id].number_pattern is not a valid regex: {err}"))?;
            if number_re.is_match(section_separator) {
                return Err(anyhow!(
                    "[id].section_separator `{section_separator}` is matched by [id].number_pattern"
                ));
            }
        }

        let sep_quoted = regex::escape(section_separator);
        let section_pattern = if named_sections {
            format!(r"(?:{NUMERIC_SECTION_PATTERN}|{NAMED_SECTION_PATTERN})")
        } else {
            NUMERIC_SECTION_PATTERN.to_string()
        };
        let sec_group = format!(r"(?P<sec>{section_pattern})");
        let sec_suffix = format!(r"(?:{}{})?", sep_quoted, sec_group);

        let comment_prefix = comment_prefix_regex(comment_prefixes);
        // Declaration grammar (§AR-scanner.2.1): Markdown-form is `#+` then ID,
        // with the `#` mandatory in `.md`; code-form requires a comment prefix and
        // then the ID directly (§DF-code-declarations-drop-hash).

        // §FS-declarations.line.configured-slug: the ID need not end in a word character.
        // Match its delimiter explicitly so regex alternatives cannot claim a shorter prefix.
        let decl_re = Regex::new(&format!(
            r"^\s*(?:{prefix}\s+|(?P<mdhashes>#+)\s+){id}(?:[:\s`]|$)",
            prefix = comment_prefix,
            id = id_pat
        ))?;
        let docstring_decl_re = Regex::new(&format!(r"^\s*{id}(?:[:\s`]|$)", id = id_pat))?;
        // §FS-declarations.line.section-suffix: a coordinate can match an ID prefix;
        // `opens_section_suffix` refuses it in the shared capture reader.
        let section_suffix_re = Regex::new(&format!(r"\A{sep_quoted}(?:{section_pattern})"))?;
        // §FS-config.3.3.1: name-bearing headings require the explicit colon form;
        // numeric headings retain optional full stops. Rust regexes lack lookahead, so
        // punctuation stays captured and `section_path` removes it (§AR-scanner.2.2).
        let section_heading = if named_sections {
            format!(r"(?P<sec>(?:{NUMERIC_SECTION_PATTERN}\.?|{NAMED_SECTION_PATTERN}:))")
        } else {
            format!(r"(?P<sec>{NUMERIC_SECTION_PATTERN})\.?")
        };
        let section_re = Regex::new(&format!(
            r"^\s*(?:{})?\s*(?P<hashes>#+)\s+{}\s+\S",
            comment_prefix, section_heading
        ))?;
        // §FS-workspace.1: the optional `<alias>/` namespace prefix is part of the
        // citation grammar, not a separate parser pass, and the scanner gates it on
        // the marker (§AR-workspace.3.1, §FS-workspace.6.1).
        let namespace_prefix = format!(r"(?:(?P<namespace>{})/)?", *PROJECT_PATH_PATTERN);
        let citation_re = Regex::new(&format!(r"\b{}{}{}", namespace_prefix, id_pat, sec_suffix))?;
        // §FS-check.1.1.10: only the explicit-marker path bypasses the bare boundary.
        let citation_prefix_re =
            Regex::new(&format!(r"\A{}{}{}", namespace_prefix, id_pat, sec_suffix))?;
        let id_input_re = Regex::new(&format!(r"^{}{}$", id_pat, sec_suffix))?;
        let legacy = LegacyGrammar::build(kinds, format, &section_pattern, &comment_prefix)?;

        // §FS-check.1.2.1: the same two shapes over the slug-less element list.
        // Compiled only where the format has a shorthand at all, so `has_shorthand`
        // is the single gate the scanner, checker, `fmt`, and the LSP all read.
        let global_kinds = kinds
            .iter()
            .filter(|kind| kind.format.is_none())
            .map(|kind| regex::escape(&kind.name))
            .collect::<Vec<_>>();
        let shorthand = (!global_kinds.is_empty())
            .then(|| shorthand_elements(&elements))
            .flatten()
            .map(|short| {
                let kind_group = format!("(?P<kind>{})", global_kinds.join("|"));
                let num_group = format!("(?P<num>{})", number_pattern);
                let slug_group = format!("(?P<slug>{})", slug_pattern);
                let global_id_pat = id_pattern(&elements, &kind_group, &num_group, &slug_group);
                let short_pat = id_pattern(&short, &kind_group, &num_group, &slug_group);
                ShorthandGrammar {
                    full_prefix_pattern: format!(
                        r"\A{}{}{}",
                        namespace_prefix, global_id_pat, sec_suffix
                    ),
                    prefix_pattern: format!(r"\A{}{}{}", namespace_prefix, short_pat, sec_suffix),
                    // §FS-fmt.2.4.1 clause 2: the same shorthand shape with no
                    // `<alias>/` in front of it — reusing `prefix_pattern` here would
                    // count every path ending in an ID-shaped segment.
                    unqualified_prefix_pattern: format!(r"\A{}{}", short_pat, sec_suffix),
                    // Non-capturing: this one is only ever asked `is_match`, and a
                    // second `(?P<num>…)` beside the one in `short_pat` would be a
                    // duplicate group name if the two ever met in one pattern.
                    number_prefix_pattern: format!(r"\A(?:{})", number_pattern),
                    full_prefix_re: once_cell::sync::OnceCell::new(),
                    prefix_re: once_cell::sync::OnceCell::new(),
                    unqualified_prefix_re: once_cell::sync::OnceCell::new(),
                    number_prefix_re: once_cell::sync::OnceCell::new(),
                }
            });
        let override_shorthands = kinds
            .iter()
            .filter_map(|kind| {
                let format = kind.format.as_deref()?;
                let elements = parse_id_format(format).ok()?;
                let short = shorthand_elements(&elements)?;
                let kind_group = format!("(?P<kind>{})", regex::escape(&kind.name));
                let num_group = format!("(?P<num>{})", number_pattern);
                let slug_group = format!("(?P<slug>{})", slug_pattern);
                let full = id_pattern(&elements, &kind_group, &num_group, &slug_group);
                let short = id_pattern(&short, &kind_group, &num_group, &slug_group);
                Some(ShorthandGrammar {
                    full_prefix_pattern: format!(r"\A{}{}{}", namespace_prefix, full, sec_suffix),
                    prefix_pattern: format!(r"\A{}{}{}", namespace_prefix, short, sec_suffix),
                    unqualified_prefix_pattern: format!(r"\A{}{}", short, sec_suffix),
                    number_prefix_pattern: format!(r"\A(?:{})", number_pattern),
                    full_prefix_re: once_cell::sync::OnceCell::new(),
                    prefix_re: once_cell::sync::OnceCell::new(),
                    unqualified_prefix_re: once_cell::sync::OnceCell::new(),
                    number_prefix_re: once_cell::sync::OnceCell::new(),
                })
            })
            .collect();

        let near_misses = kinds
            .iter()
            .filter_map(|kind| {
                let effective = kind.format.as_deref().unwrap_or(format);
                literal_after_kind_placeholder(effective)
                    .filter(|literal| !literal.is_empty())
                    .map(|literal| {
                        NearMissGrammar::build(
                            &regex::escape(&kind.name),
                            &comment_prefix,
                            literal,
                            effective,
                        )
                    })
            })
            .collect();
        Ok(Self {
            decl_re,
            docstring_decl_re,
            section_suffix_re,
            section_re,
            citation_re,
            citation_prefix_re,
            id_input_re,
            named_sections,
            near_misses,
            legacy,
            shorthand,
            override_shorthands,
            elements,
            kind_parsers,
            kind_elements,
            overridden_kinds,
            source: Arc::new(GrammarSource {
                format: format.into(),
                kinds: kinds.to_vec(),
                number_pattern: number_pattern.into(),
                slug_pattern: slug_pattern.into(),
                section_separator: section_separator.into(),
                comment_prefixes: comment_prefixes.to_vec(),
            }),
        })
    }

    /// Parse one already-delimited full ID using the exact grammar configured
    /// for its kind (§FS-config.3.2).
    pub(crate) fn parse_token(&self, token: &str) -> Option<Id> {
        self.kind_parsers.iter().find_map(|(kind, parser)| {
            let caps = parser.captures(token)?;
            let num = caps
                .name("num")
                .map(|value| value.as_str().parse())
                .transpose()
                .ok()?;
            let slug = caps.name("slug").map(|value| value.as_str().to_string());
            Some(Id {
                kind: kind.clone(),
                num,
                slug,
            })
        })
    }

    pub(super) fn shorthands(&self) -> impl Iterator<Item = &ShorthandGrammar> {
        self.shorthand.iter().chain(self.override_shorthands.iter())
    }

    pub(crate) fn shorthand_for(&self, token: &str) -> Option<&ShorthandGrammar> {
        self.shorthands()
            .find(|shorthand| shorthand.prefix_re().is_match(token))
    }

    /// A name-shaped section matched by the opted-in grammar (§FS-check.1.1.2).
    pub(crate) fn is_named_section(&self, section: Option<&str>) -> bool {
        self.named_sections
            && section.is_some_and(|path| {
                path.split('.')
                    .any(|part| part.as_bytes().first().is_some_and(u8::is_ascii_lowercase))
            })
    }

    /// Whether a valid prefix is followed by the reserved `number.name` order.
    /// The regex crate has no lookahead, so this whole-token rejection is the
    /// post-match half of the grammar (§AR-scanner.2.3.4, §FS-config.3.3.1).
    pub(crate) fn has_reserved_named_tail(&self, text: &str, end: usize) -> bool {
        self.named_sections
            && text[end..]
                .strip_prefix('.')
                .and_then(|tail| tail.as_bytes().first())
                .is_some_and(u8::is_ascii_lowercase)
    }

    /// Whether `rest`, the text directly after an ID-shaped token in declaration
    /// position, opens with the configured separator and a section. Such a token is a
    /// coordinate, and the line declares nothing (§FS-declarations.line.section-suffix).
    /// The regex crate has no lookahead, so this is the post-match half of the
    /// declaration patterns, and the one check every reading of a declaration line
    /// makes (§AR-scanner.2.1.1). The separator and the sections are the project's
    /// (§FS-declarations.line.section-suffix.3).
    pub(crate) fn opens_section_suffix(&self, rest: &str) -> bool {
        self.section_suffix_re.is_match(rest)
    }

    pub(crate) fn is_section_path(&self, section: &str) -> bool {
        self.legacy.section_path_re.is_match(section)
    }
}

/// The normalized complete path from a citable section heading. In named mode
/// the grammar captures the discriminating `:`; numeric `.` remains optional.
pub(crate) fn section_path<'a>(caps: &'a regex::Captures<'a>) -> Option<&'a str> {
    let raw = caps.name("sec")?.as_str();
    Some(
        raw.strip_suffix('.')
            .or_else(|| raw.strip_suffix(':'))
            .unwrap_or(raw),
    )
}
