//! The number-only shorthand resolved against the **catalog of a run**
//! (§FS-fmt.2.4, §FS-workspace.1.2, §DF-number-only-citation-shorthand): what one
//! `grund fmt` walk indexes to expand a shorthand, the one declared ID a typed
//! token expands to, the rewrite that writes the canonical form, and the
//! cross-namespace pass that resolves `§<alias>/FS-042` once every project has
//! been scanned.
//!
//! Recognition is `grammar/shorthand.rs` — the shape a token wears, the
//! `(kind, number)` candidate rule, and the index a pass builds out of one
//! declaration set. What is here is everything that needs the *loaded project
//! set*: whose grammar parses a qualified token's tail, whose `[id] format`
//! renders the canonical ID, whose `[reference] shorthand` policy lets a
//! persisted spelling stand, and which project's declarations an alias points
//! at (§AR-system.2.10, §AR-resolver.4). A per-project scan cannot answer any of
//! them — it sees only its own declaration set — which is what put this half
//! above the scanner rather than in the grammar under it (§AR-system.4).
//!
//! The finding a shorthand site earns is not here either: that is a rule, and
//! rules are the checker's (`checker/shorthand.rs`, §FS-check.3.13).

use std::collections::BTreeMap;

use super::context::{WorkspaceContext, WorkspaceProject};
use crate::config::{Config, ShorthandPolicy};
use crate::grammar::{
    DocstringContent, ParsedId, QUALIFIED_CITATION_PREFIX, ShorthandIndex,
    never_rewrite_context_in, parse_id, parse_id_arg, parse_id_arg_with_shorthand, render_id,
    shorthand_names,
};
use crate::model::{Findings, Id};

/// Everything one `grund fmt` walk needs to expand a shorthand: this project's
/// declaration index, plus one per workspace alias for the qualified form
/// (§FS-fmt.2.4, §FS-workspace.8.5).
///
/// Built once per walk rather than per line. `fmt` visits every marker of every
/// scanned file, so resolving each against a linear scan of the declaration set
/// is quadratic on exactly the tree this rewrite exists to clean up
/// (§GOAL-fast-feedback).
pub(crate) struct ShorthandTargets<'a> {
    /// `None` until the walk has a declaration set — §FS-fmt.2.4.5 defers that scan
    /// until a shorthand is actually met, so a repo without one never pays for it.
    pub(crate) local: Option<ShorthandIndex<'a>>,
    pub(crate) by_alias: BTreeMap<&'a str, ShorthandAliasTarget<'a>>,
}

/// One aliased project's half of `ShorthandTargets`: its declarations, and the
/// config the canonical ID renders under (a workspace may mix `[id] format`s).
pub(crate) struct ShorthandAliasTarget<'a> {
    pub(crate) config: &'a Config,
    pub(crate) index: ShorthandIndex<'a>,
}

impl<'a> ShorthandTargets<'a> {
    pub(crate) fn new(
        config: &Config,
        findings: Option<&'a Findings>,
        workspace: Option<&'a WorkspaceContext>,
    ) -> Self {
        Self {
            local: findings
                .map(|found| ShorthandIndex::build(&config.grammar, found.declarations.keys())),
            by_alias: workspace
                .map(|workspace| {
                    workspace
                        .projects
                        .iter()
                        .map(|project| {
                            (
                                project.alias.as_str(),
                                ShorthandAliasTarget {
                                    config: &project.config,
                                    index: ShorthandIndex::build(
                                        &project.config.grammar,
                                        project.findings.declarations.keys(),
                                    ),
                                },
                            )
                        })
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}

/// §FS-fmt.2.4: expand every number-only shorthand citation on the line that
/// resolves to exactly one declaration. An ambiguous or unknown shorthand is left
/// exactly as written — `fmt` normalizes, it does not guess, and §FS-check.3.13
/// is where those two outcomes are reported.
///
/// Qualified `§<alias>/FS-042` resolves against the aliased project's
/// declarations, which is why `workspace` is threaded in: §FS-fmt.2.4 promises the
/// rewrite preserves the `<alias>/` namespace, and a rule that reported the site
/// but never fixed it would leave the §FS-check.3.13 error with no bulk remedy.
/// A member-local run carries no workspace context and leaves the citation alone
/// — there the alias resolves nowhere and `check` says so instead
/// (§FS-workspace.8.5.1).
///
/// Whose grammar parses a token: the scanner routes qualified citations to the
/// target project's grammar (`scan_workspace_qualified_pass`) and this pass has to
/// agree with it byte for byte — matching the tail with the citing project's
/// shorthand instead would rewrite tokens `check` never saw and skip the ones it
/// reported, in a workspace that mixes `[id] format`s. A project alias is
/// lower-case-initial and a kind is not, which is why one byte can skip the
/// qualified pattern for essentially every citation in a real tree; `fmt` runs this
/// per marker of every scanned line.
///
/// Why the gates are in this order: testing a claimable token with the *anchored*
/// full-ID pattern rather than an unanchored search is what keeps `fmt --check`
/// cheap — a canonical citation, nearly every marker in a real tree, never reaches
/// the shorthand pattern at all. That pattern is anchored only at its start, so
/// `§FS-042-User-Login`, a full ID whose slug this grammar rejects, matches on its
/// `FS-042` prefix; rewriting it would splice the canonical slug into the middle of
/// the author's token, leave the tail glued on, and silently corrupt the file. And
/// declarations are reached for last because `fmt --check` has no other reason to
/// scan: the walk starts without them, this pass reports the first candidate it
/// actually meets, the caller scans then and re-runs the single file, and a repo
/// that never writes a shorthand pays nothing at all. The expansion report is what
/// makes a rewrite reviewable before it is written.
#[cfg(test)]
pub(crate) fn expand_shorthand_citations(
    line: &str,
    docstring: DocstringContent<'_>,
    config: &Config,
    is_md: bool,
    targets: &ShorthandTargets<'_>,
    saw_candidate: &mut bool,
    expansions: &mut Vec<(String, String)>,
) -> Option<String> {
    expand_shorthand_citations_with_origins(
        line,
        docstring,
        config,
        is_md,
        targets,
        &[],
        saw_candidate,
        expansions,
    )
}

/// The formatter entry point for §FS-fmt.2.4.3, carrying the byte offsets of
/// markers produced from triggers so accepted persisted forms and authoring
/// sugar remain distinct even when they share one line.
#[allow(clippy::too_many_arguments)]
pub(crate) fn expand_shorthand_citations_with_origins(
    line: &str,
    docstring: DocstringContent<'_>,
    config: &Config,
    is_md: bool,
    targets: &ShorthandTargets<'_>,
    trigger_marker_starts: &[usize],
    saw_candidate: &mut bool,
    expansions: &mut Vec<(String, String)>,
) -> Option<String> {
    // The local grammar is only one of the grammars in play: a qualified citation
    // is parsed with the *target's*, so a citing project with no shorthand of its
    // own can still hold one that needs expanding.
    if config.marker.is_empty()
        || !line.contains(&config.marker)
        || (!config.grammar.has_shorthand() && targets.by_alias.is_empty())
    {
        return None;
    }
    let mut output = String::new();
    let mut cursor = 0;
    // Driven from marker positions, like the scan pass and for the same reason: the
    // shorthand prefixes every full ID under the default format, so a line sweep
    // costs a candidate and a rejection per citation (§AR-scanner.2.6, §GOAL-fast-feedback).
    for (marker_start, _) in line.match_indices(&config.marker) {
        if marker_start < cursor {
            continue;
        }
        let token_start = marker_start + config.marker.len();
        let Some(rest) = line.get(token_start..) else {
            continue;
        };
        // §FS-workspace.1.2: an `<alias>/` prefix decides *whose* grammar parses the
        // rest of the token, and a lower-case initial is the one byte that skips the
        // qualified pattern for nearly every citation (§GOAL-fast-feedback).
        let alias = rest
            .starts_with(|ch: char| ch.is_ascii_lowercase())
            .then(|| {
                QUALIFIED_CITATION_PREFIX
                    .captures(rest)
                    .and_then(|caps| Some((caps.name("namespace")?.as_str(), caps.get(0)?.end())))
            })
            .flatten();
        let target = match alias {
            Some((alias, _)) => match targets.by_alias.get(alias) {
                Some(target) => Some(target),
                None => continue,
            },
            None => None,
        };
        let target_config = target.map_or(config, |target| target.config);
        let alias_len = alias.map_or(0, |(_, len)| len);
        let tail = &rest[alias_len..];
        let Some(shorthand) = target_config.grammar.shorthand_for(tail) else {
            continue;
        };
        // §DF-number-only-citation-shorthand.2.6: a token the full-ID pattern can
        // claim is already canonical and must not be touched; anchored, the test
        // costs one match bounded by the token (§GOAL-fast-feedback).
        if shorthand.full_prefix_re().is_match(tail) {
            continue;
        }
        let Some(caps) = shorthand.prefix_re().captures(tail) else {
            continue;
        };
        let match_end = caps.get(0).map_or(0, |found| found.end());
        if target_config
            .grammar
            .has_reserved_named_tail(rest, alias_len + match_end)
        {
            continue;
        }
        // §DF-number-only-citation-shorthand.2.6: the pattern is anchored only at
        // the start, so rewriting `§FS-042-User-Login` on its `FS-042` prefix would
        // corrupt the file — see the gate order above.
        if !target_config.grammar.id_token_ends_cleanly(tail, match_end) {
            continue;
        }
        // §FS-fmt.2.4.1.1: `§SPEC-001→SPEC-003` is a renumbering table, not a citation.
        // The marker is the *citing* project's — what the author typed — while the
        // number shape is the target's, the same split the rewrite below uses.
        if target_config
            .grammar
            .shorthand_sits_in_numeric_run(&config.marker, tail, match_end)
        {
            continue;
        }
        // §FS-fmt.2.3: the same exclusions the other rewrites honour — inline code,
        // a link destination, a runtime string — asked of the docstring's content on
        // a docstring line, exactly as the scanner asks it (§FS-fmt.2.3.1.1).
        if never_rewrite_context_in(docstring, line, is_md, marker_start) {
            continue;
        }
        let Some(id) = parse_id(&caps, &target_config.grammar) else {
            continue;
        };
        // §FS-fmt.2.4.5: only *now* are declarations needed — every gate above rejects
        // on the line text alone, and reaching for them earlier was a measured 79%
        // regression on the benchmark fixture (§GOAL-fast-feedback, §AR-ci.5).
        let index = match target {
            Some(target) => &target.index,
            None => match targets.local.as_ref() {
                Some(index) => index,
                None => {
                    *saw_candidate = true;
                    continue;
                }
            },
        };
        let Some(unique) = index.unique(&id) else {
            continue;
        };
        // Exact persisted shorthand-shaped IDs are read compatibility, not an
        // authoring rewrite. With a conforming neighbor they made `unique`
        // return `None`; alone they already have the right written spelling.
        if unique.legacy_spelling().is_some() {
            continue;
        }
        // §FS-fmt.2.4.3 / §FS-workspace.4.2: an accepted project preserves only
        // marker-origin shorthand. A marker created from this line's trigger is
        // still authoring input and always expands to the canonical full ID.
        if target_config.shorthand == ShorthandPolicy::Accepted
            && !trigger_marker_starts.contains(&marker_start)
        {
            continue;
        }
        let namespace = alias.map(|(alias, _)| alias);
        let match_end = alias_len + match_end;
        output.push_str(&line[cursor..token_start]);
        // §FS-fmt.3.6: the written and canonical forms are recorded as the line is
        // built, because this is the only point that holds both — and expanding is
        // the one rewrite here whose mistakes no later pass can see.
        let written_start = output.len();
        if let Some(alias) = namespace {
            output.push_str(alias);
            output.push('/');
        }
        // The ID renders under the *target* project's `[id] format`, which is what
        // makes the rewrite correct in a mixed-format workspace.
        output.push_str(&render_id(&target_config.grammar, unique));
        if let Some(section) = caps.name("sec") {
            // The target's separator, matching the form §FS-check.3.13 names —
            // the section belongs to the target's ID, not the citing project's.
            output.push_str(&target_config.section_separator);
            output.push_str(section.as_str());
        }
        expansions.push((
            line[marker_start..token_start + match_end].to_string(),
            format!("{}{}", config.marker, &output[written_start..]),
        ));
        cursor = token_start + match_end;
    }
    // `cursor` moves only when something was rewritten, so this is the
    // "unchanged" signal — and returning `None` lets the caller move the
    // original line through instead of allocating a copy of it.
    if cursor == 0 {
        return None;
    }
    output.push_str(&line[cursor..]);
    Some(output)
}

/// §AR-scanner.2.6.6: resolve `§<alias>/FS-042` against the aliased project's
/// declarations. A per-project scan cannot do this — it sees only its own
/// declaration set — so the cross-namespace half of the shorthand rule lands
/// here, once every project has been scanned. Unqualified shorthands were
/// already resolved inside each project's own walk.
pub(crate) fn resolve_qualified_shorthand_citations(projects: &mut [WorkspaceProject]) {
    let pending = |project: &WorkspaceProject| {
        project
            .findings
            .citations
            .iter()
            .any(|cite| cite.shorthand && cite.namespace.is_some() && cite.id.slug.is_none())
    };
    if !projects.iter().any(pending) {
        return;
    }
    let declared: BTreeMap<String, Vec<Id>> = projects
        .iter()
        .map(|project| {
            (
                project.alias.clone(),
                project.findings.declarations.keys().cloned().collect(),
            )
        })
        .collect();
    let indexes: BTreeMap<&str, ShorthandIndex<'_>> = declared
        .iter()
        .filter_map(|(alias, ids)| {
            let grammar = projects
                .iter()
                .find(|project| project.alias == *alias)
                .map(|project| &project.config.grammar)?;
            Some((alias.as_str(), ShorthandIndex::build(grammar, ids.iter())))
        })
        .collect();
    for project in projects.iter_mut() {
        for cite in &mut project.findings.citations {
            if !cite.shorthand || cite.id.slug.is_some() {
                continue;
            }
            let Some(index) = cite
                .namespace
                .as_deref()
                .and_then(|alias| indexes.get(alias))
            else {
                continue;
            };
            if let Some(unique) = index.unique(&cite.id) {
                cite.id = unique.clone();
            }
        }
    }
}

/// The canonical ID a typed shorthand token expands to, or `None` when the token
/// is not a shorthand or names other than exactly one declared ID
/// (§FS-lsp.1.4). Candidates are matched by re-parsing each declared ID under
/// the same grammar, so the editor and `grund fmt` agree on what resolves
/// (§FS-fmt.2.4).
pub(crate) fn shorthand_token_expansion(
    config: &Config,
    token: &str,
    declared_ids: &[&str],
) -> Option<String> {
    if !config.grammar.has_shorthand() {
        return None;
    }
    let parsed = parse_id_arg_with_shorthand(token, &config.grammar).ok()?;
    if !parsed.shorthand {
        return None;
    }
    let unique = unique_shorthand_expansion_target(config, token, &parsed, declared_ids)?;
    let section = parsed
        .section
        .map(|section| format!("{}{}", config.section_separator, section))
        .unwrap_or_default();
    Some(format!("{unique}{section}"))
}

fn unique_shorthand_expansion_target<'a>(
    config: &Config,
    token: &str,
    parsed: &ParsedId,
    declared_ids: &[&'a str],
) -> Option<&'a str> {
    let exact = parsed.section.as_ref().map_or(token, |section| {
        token
            .strip_suffix(&format!("{}{}", config.section_separator, section))
            .unwrap_or(token)
    });
    let mut matches = declared_ids.iter().copied().filter(|declared| {
        *declared == exact
            || parse_id_arg(declared, &config.grammar)
                .is_ok_and(|(id, section)| section.is_none() && shorthand_names(&id, &parsed.id))
    });
    let unique = matches.next()?;
    (matches.next().is_none() && unique != exact).then_some(unique)
}
