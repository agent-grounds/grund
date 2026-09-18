//! The one finding a number-only shorthand site earns (§FS-check.3.13,
//! §AR-checker.2.12), and the three shapes it wears: no declaration, several, or
//! one whose canonical form the author should write instead.
//!
//! It sits here rather than with the shape it judges because this is a *rule*,
//! and §AR-system.2.1 says the grammar holds none. What a shorthand token is and
//! which declarations it names are read downward — the shape from
//! `grammar/shorthand.rs`, the candidate set from the index it builds, and the
//! project a qualified citation resolves against from `resolver/` — and the
//! verdict is taken here, beside `references.rs`, the only caller
//! (§AR-core-module-layout.1).

use std::collections::BTreeMap;

use super::references::ReferenceTier;
use crate::config::{Config, ShorthandPolicy};
use crate::grammar::{ShorthandIndex, render_id, render_qualified_id};
use crate::model::{CheckReport, Citation, Diagnostic, Id};
use crate::resolver::WorkspaceCheckTarget;

/// §FS-check.3.13 / §AR-checker.2.12: the one finding a number-only shorthand
/// site earns. The candidate set is re-derived here rather than read off the
/// citation, so the message is right whether or not the scanner's resolution
/// pass has run — a synthetic `Findings` fed straight to the checker gets the
/// same three shapes.
///
/// The marker comes from the *citing* project (it is what the author types) while
/// the ID renders under the *target* project's `[id] format`, so the replacement
/// in a mixed-format workspace is pasteable as printed.
///
/// `None` for a resolving shorthand at a site `fmt` may not rewrite
/// (§FS-fmt.2.3): the citation is real and counts everywhere, but the only fix
/// this error knows how to name is one the formatter declines to apply, so
/// reporting it would leave a repository permanently red with nothing to run.
/// A shorthand that resolves to zero or several declarations is still reported
/// there — that is a dangling reference, not a formatting nit. The out-of-scope
/// tier of `check --full` withholds it for the same reason: `fmt` scopes by
/// `[scan] include` too (§FS-check.3.14).
fn shorthand_diagnostic(
    config: &Config,
    cite: &Citation,
    target_config: &Config,
    tier: ReferenceTier,
    candidates: &[&Id],
) -> Option<Diagnostic> {
    let written = cite.text.trim();
    let message = match candidates {
        [] => format!("shorthand citation {written} matches no declaration"),
        [unique] => {
            // An exact persisted spelling is already canonical for compatibility;
            // there is no different text for `fmt` to write (§FS-config.3.2).
            if unique.legacy_spelling().is_some() {
                return None;
            }
            // §FS-check.3.14: outside the configured scope `fmt` will not rewrite
            // the site either, so the same withholding applies for the same reason.
            if !cite.shorthand_rewritable || tier == ReferenceTier::OutOfScope {
                return None;
            }
            let section = cite
                .section
                .as_ref()
                .map(|section| format!("{}{}", target_config.section_separator, section))
                .unwrap_or_default();
            let canonical = format!(
                "{}{}{}",
                config.marker,
                render_qualified_id(&target_config.grammar, cite.namespace.as_deref(), unique),
                section
            );
            // §FS-check.3.15: the same site, a different verdict — `fmt` will not
            // rewrite a numeral in a run, so naming only the canonical form would
            // advise the edit that corrupts the line. Both exits; the author picks.
            if cite.numeric_run {
                return Some(Diagnostic {
                    code: "shorthand-numeric-run",
                    path: Some(cite.file.clone()),
                    line: Some(cite.line),
                    column: Some(cite.column),
                    message: format!(
                        "shorthand {written} sits in a numeric run and was not rewritten; \
                         write {canonical}, or <{}>{} if these are old numbers",
                        config.marker,
                        written
                            .strip_prefix(config.marker.as_str())
                            .unwrap_or(written),
                    ),
                    sites: Vec::new(),
                });
            }
            // §FS-check.3.13 / §FS-workspace.4: only the unique persisted-form
            // finding is policy-gated, and the policy belongs to the project
            // whose catalog resolved the shorthand.
            if target_config.shorthand == ShorthandPolicy::Accepted {
                return None;
            }
            format!("shorthand citation {written}; write {canonical}")
        }
        many => format!(
            "shorthand citation {written} is ambiguous: {}",
            many.iter()
                .map(|id| render_id(&target_config.grammar, id))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    };
    Some(Diagnostic {
        code: "shorthand-citation",
        path: Some(cite.file.clone()),
        line: Some(cite.line),
        column: Some(cite.column),
        message,
        sites: Vec::new(),
    })
}

/// Declaration indexes for the checker's shorthand pass, keyed by the citation's
/// target namespace and populated on first use — so a tree without shorthands
/// builds none (§FS-check.3.13).
pub(super) type ShorthandIndexes<'a> = BTreeMap<Option<String>, ShorthandIndex<'a>>;

/// §FS-check.3.13 / §AR-checker.2.12: report one citation's shorthand finding, if
/// it earns one. Returns `true` when the citation resolved to nothing and the
/// caller should skip its remaining rules — §3.1 in particular, which would
/// otherwise name a token that is not a full ID.
///
/// "Resolved" is read off the candidate set this check can see, not off the
/// scanner's rewrite: under `--full` the scan resolved every shorthand against
/// the whole walk and the findings were then narrowed back to `[scan] include`
/// (§FS-check.1.3), so a site can arrive here holding a canonical ID whose
/// declaration is no longer in the target's set. Judging it by the rewrite alone
/// would earn that site a shorthand error *and* a dangling error for one cause,
/// which §FS-check.3.13 says never happens. The qualified cross-member form is
/// resolved in a pass of the resolver's own (§AR-resolver.4), so this is the only
/// place both forms meet.
///
/// The index is built per target namespace rather than per site: re-deriving the
/// candidate set by walking every declaration each time is quadratic on a tree
/// mid-migration, which is precisely the tree this rule asks people to run
/// (§GOAL-fast-feedback).
pub(super) fn report_shorthand_citation<'a>(
    cite: &Citation,
    config: &Config,
    target: &WorkspaceCheckTarget<'a>,
    tier: ReferenceTier,
    indexes: &mut ShorthandIndexes<'a>,
    report: &mut CheckReport,
) -> bool {
    let index = indexes.entry(cite.namespace.clone()).or_insert_with(|| {
        ShorthandIndex::build(&target.config.grammar, target.findings.declarations.keys())
    });
    let candidates = index.candidates(&cite.id);
    let resolved = cite.id.slug.is_some() && candidates.len() == 1;
    if let Some(diagnostic) = shorthand_diagnostic(config, cite, target.config, tier, candidates) {
        report.errors.push(diagnostic);
    }
    !resolved
}
