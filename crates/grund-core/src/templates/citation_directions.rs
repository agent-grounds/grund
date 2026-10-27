use crate::config::{
    CitationDisjunction, CitationLevel, CitationTarget, KindCitationRules, NamespaceMatch, Project,
    Row, Run, grounding_setting, row_grounds_at_any_strength,
};

use super::citation_grounding::citation_grounding_sentence;

/// The public citation-directions page the scaffold config and the no-`[citations]`
/// fallback sentence hand a reader instead of an ID of this repository
/// (§FS-config.3.9, §REQ-shipped-surfaces.1).
const CITATION_DIRECTIONS_URL: &str =
    "https://github.com/agent-grounds/grund/blob/main/docs/user-facing/citation-directions.md";

/// The `### Citation directions` renderer of the managed block (§FS-init.2.3.5),
/// in a file of its own so the payload constants and the renderer that composes
/// them do not share one (§AR-core-module-layout.1).
///
/// The section is what an agent reads *instead of* `grund.toml`, so every line
/// here is held to the rule it states: a bullet names the unit the rule is
/// checked per, a conjunction of alternatives is grouped so it has one reading,
/// rule grammar never reaches the prose, and the levels that gate are told apart
/// from the levels that suggest. The wording is pinned by
/// §DF-directions-render against the canonical config that exercises every
/// branch.
///
/// Rendering stays deterministic — `[[kinds]]` order, the homeless kind last,
/// fixed phrases — because `grund check` re-renders it and byte-compares for
/// drift (§FS-check.3.5): the render *is* the hash.

/// The legend (§FS-init.2.3.5.1): which of the five levels are `grund check`
/// errors and which are suggestions. Fixed text, rendered whenever `[citations]`
/// is declared, because a bullet's verb is unreadable without it — an agent that
/// cannot tell `should` from `must` treats every rule as one or the other.
pub(crate) const CITATION_LEVEL_LEGEND: &str = "`must`/`never` are `grund check` errors; `should`/`avoid` are suggestions (`grund check --suggestions`).";

/// Render the `### Citation directions` managed-block section (§FS-init.2.3.5)
/// from the effective config, every row in it: the root block of a config whose
/// guidance all renders at the root (§FS-init.2.3.5.11.5).
pub(crate) fn citation_directions_section(project: &Project, run: &Run) -> String {
    root_citation_directions(project, run, &|_| true)
}

/// The root block's `### Citation directions` (§FS-init.2.3.5.1), holding the
/// rows `here` says land at the root. When no `[citations]` section is
/// declared, the static climbing-rule sentence stands in, so a config that
/// predates the feature keeps a stable block (§FS-init.2.3.4.10); the grounding
/// sentence renders under either, because it is generated from each row's
/// effective grounding and not from the direction rules (§FS-check.3.6).
pub(crate) fn root_citation_directions(
    project: &Project,
    run: &Run,
    here: &dyn Fn(&str) -> bool,
) -> String {
    // Built as lines joined with `\n` and returned without a trailing newline:
    // the `{CITATION_DIRECTIONS}` placeholder in the template supplies the single
    // block-final newline, so `grund init` stays idempotent on re-run.
    let mut lines = vec!["### Citation directions".to_string(), String::new()];
    let grounding = citation_grounding_sentence(project, run, here);
    if !project.rules.citations.declared {
        // §FS-init.2.3.4.4.2: the climbing-rule sentence names kinds; a config
        // with no citable kind is told that no direction applies yet instead.
        let paragraph = if project.kind_configs().iter().any(|kind| kind.citable) {
            format!(
                "Specs cite goals, architecture cites specs, code and executable tests cite the specs they realize. In a citation rule array, entries are all required; `|` inside one entry means any one alternative. See {CITATION_DIRECTIONS_URL} for the levels and examples."
            )
        } else {
            format!(
                "No kind is declared yet, so no citation direction applies. Once kinds exist, the config's citation rules say which kinds each may cite; see {CITATION_DIRECTIONS_URL} for the levels and examples."
            )
        };
        lines.push(join_sentences(&paragraph, grounding.as_deref()));
        return lines.join("\n");
    }
    lines.push(join_sentences(CITATION_LEVEL_LEGEND, grounding.as_deref()));
    lines.push(String::new());
    lines.extend(citation_bullets(project, run, here));
    // Load-bearing (§FS-init.2.3.5.6): silence is open only when the global default
    // leaves it open, and a per-kind default never reaches it — that one is
    // folded into its own bullet, which is *listed above*.
    lines.push(citation_closing_line(
        project.rules.citations.global_default,
    ));
    lines.join("\n")
}

/// A scoped block's `### Citation directions` (§FS-init.2.3.5.12.1): the legend
/// and this destination's grounding sentence, its rows' bullets and the closing
/// line for the global default — or nothing, when no bullet and no grounding
/// clause lands here.
pub(crate) fn scoped_citation_directions(
    project: &Project,
    run: &Run,
    here: &dyn Fn(&str) -> bool,
) -> Option<String> {
    let grounding = citation_grounding_sentence(project, run, here);
    let bullets = citation_bullets(project, run, here);
    if grounding.is_none() && bullets.is_empty() {
        return None;
    }
    let mut lines = vec![
        "### Citation directions".to_string(),
        String::new(),
        join_sentences(CITATION_LEVEL_LEGEND, grounding.as_deref()),
        String::new(),
    ];
    lines.extend(bullets);
    lines.push(citation_closing_line(
        project.rules.citations.global_default,
    ));
    Some(lines.join("\n"))
}

/// The rows a bullet is rendered for, in the order §FS-init.2.3.5.1 lays them
/// out: `[[kinds]]` order, then the homeless kind last — wherever in the table a
/// project happened to declare it, because it is the complement of every row
/// above it and reads as the closing case. Placement walks the same order, so
/// the pointer lists destinations as the bullets would (§FS-init.2.3.5.13).
pub(crate) fn bullet_row_order(project: &Project) -> Vec<&str> {
    let homeless = project.schema.complement_name();
    let mut rows: Vec<&str> = project
        .schema
        .rows
        .iter()
        .map(|row| row.name.as_str())
        .filter(|name| *name != homeless)
        .collect();
    rows.push(homeless);
    rows
}

/// Whether `row` renders a citation-direction bullet (§FS-init.2.3.5.1).
pub(crate) fn row_has_bullet(project: &Project, row: &str) -> bool {
    project.rules.citations.declared
        && project
            .rules
            .citations
            .per_kind
            .get(row)
            .and_then(|rules| citation_direction_clauses(project, rules))
            .is_some()
}

/// The bullets of the rows `here` admits, in [`bullet_row_order`].
fn citation_bullets(project: &Project, run: &Run, here: &dyn Fn(&str) -> bool) -> Vec<String> {
    if !project.rules.citations.declared {
        return Vec::new();
    }
    let homeless = project.schema.complement_name();
    bullet_row_order(project)
        .into_iter()
        .filter(|kind| here(kind))
        .filter_map(|kind| {
            let rules = project.rules.citations.per_kind.get(kind)?;
            let clauses = citation_direction_clauses(project, rules)?;
            let (subject, uncited) = citation_direction_subject(project, run, kind, homeless);
            Some(match uncited {
                Some(uncited) => format!("- {subject} {clauses}. {uncited}"),
                None => format!("- {subject} {clauses}."),
            })
        })
        .collect()
}

/// Two sentences of one paragraph, space-joined, with the second optional.
fn join_sentences(first: &str, second: Option<&str>) -> String {
    match second {
        Some(second) => format!("{first} {second}"),
        None => first.to_string(),
    }
}

/// What one bullet's rules are checked *per* (§FS-init.2.3.5.2) — the unit the
/// ticket's first defect was that no bullet stated — and, for a non-citable home
/// whose row grounds nothing, the sentence that follows the bullet.
///
/// A citable kind's unit is the top-level declaration (§FS-check.3.11). A
/// non-citable kind's is every scanned file in its home, so the bullet names the
/// place, as the Project map does (§FS-init.2.3.4.4) — naming the kind would name
/// something an agent can never write. The homeless kind has no place, so it
/// keeps its name and says what it covers: its `title` where the project wrote
/// one. Its unit is narrower again — a *source* file (not `.md`) that already
/// cites something (§FS-config.3.9.2.4) — because the obligation constrains what a
/// file cites and never whether it cites at all; a util that cites nothing is
/// not a unit. In a non-citable home the opposite holds — a skill without a spec
/// is the defect — so the subject is unnarrowed exactly when the row's grounding
/// reaches its files and `grund check` judges that defect. Where the row grounds
/// nothing, a file there that cites nothing is no unit either (§FS-check.3.11), so
/// the subject is narrowed as the homeless kind's is and the second sentence says
/// so and names the setting that makes it a finding (§FS-init.2.3.5.2.2).
fn citation_direction_subject(
    project: &Project,
    run: &Run,
    kind: &str,
    homeless: &str,
) -> (String, Option<String>) {
    let kinds = project.kind_configs();
    if kind == homeless {
        let scope = kinds
            .iter()
            .find(|configured| configured.kind == kind)
            .and_then(|configured| configured.title.as_deref())
            .map_or_else(String::new, |title| format!(": {title}"));
        return (
            format!(
                "Each source file outside the Project map (**{kind}**{scope}) that cites anything"
            ),
            None,
        );
    }
    let configured = project
        .schema
        .rows
        .iter()
        .find(|row| row.name == kind && row.kind.is_none());
    // §FS-config-v2.schema.places: a row with several places names every one.
    let places = configured.and_then(Row::place_labels).unwrap_or_default();
    let Some(row) = configured else {
        return (format!("Each **{kind}** declaration"), None);
    };
    let grounds = row_grounds_at_any_strength(&project.rules, run, kind);
    // A single-file non-citable home is one file, so "each file in" would
    // promise a directory that is not there.
    let (subject, narrowed, uncited) = if row.folders().next().is_some() {
        (
            format!("Each file in **{places}**"),
            " that cites anything",
            "A file there that cites nothing is not checked",
        )
    } else if row.files().nth(1).is_some() {
        (
            format!("Each of the files **{places}**"),
            " that cites anything",
            "One that cites nothing is not checked",
        )
    } else {
        (
            format!("The file **{places}**"),
            ", if it cites anything,",
            "If it cites nothing, it is not checked",
        )
    };
    if grounds {
        return (subject, None);
    }
    // §FS-init.2.3.5.2.2: the one setting that closes the hole, in the live version.
    let setting = grounding_setting(kind, project.version, |fragment| format!("`{fragment}`"));
    (
        format!("{subject}{narrowed}"),
        Some(format!("{uncited}; {setting} makes it a finding.")),
    )
}

/// The verb-phrase clauses for one citing kind's rules, joined by "; " in the
/// order obligations, permissions, prohibitions, then the default
/// (§FS-init.2.3.5.3). `None` when the kind has no renderable rule.
///
/// A prohibition leads with the modal (`must not cite`, the wording
/// §FS-check.3.12 already uses in its findings) and follows with the short form
/// (`never cite`) the legend names: the subject is a noun phrase, so a bullet
/// opening on a bare `never cite` is not a sentence.
fn citation_direction_clauses(project: &Project, rules: &KindCitationRules) -> Option<String> {
    let folded = citation_permission_is_closed(rules);
    let mut clauses = Vec::new();
    if !rules.must.is_empty() {
        clauses.push(format!("must cite {}", citation_rule_targets(&rules.must)));
    }
    // §FS-config-v2.rules.strengths: `warn` is the `must` obligation on the
    // warning channel, so it is rendered as one, marked with its channel.
    if !rules.warn.is_empty() {
        clauses.push(format!(
            "must cite {} (a warning)",
            citation_rule_targets(&rules.warn)
        ));
    }
    if !rules.should.is_empty() {
        clauses.push(format!(
            "should cite {}",
            citation_rule_targets(&rules.should)
        ));
    }
    if !rules.may.is_empty() {
        // §FS-init.2.3.5.5: a closed per-kind default plus a `may` list is one
        // rule — "only these" — and takes one clause, not a permission followed
        // by a prohibition of everything else.
        let only = if folded { "only " } else { "" };
        clauses.push(format!(
            "may cite {only}{}",
            citation_rule_targets(&rules.may)
        ));
    }
    if !rules.must_not.is_empty() {
        let verb = if clauses.is_empty() {
            "must not cite"
        } else {
            "never cite"
        };
        clauses.push(format!(
            "{verb} {}",
            citation_prohibition_targets(&rules.must_not)
        ));
    }
    if !rules.warn_not.is_empty() {
        let verb = if clauses.is_empty() {
            "must not cite"
        } else {
            "never cite"
        };
        clauses.push(format!(
            "{verb} {} (a warning)",
            citation_prohibition_targets(&rules.warn_not)
        ));
    }
    if !rules.should_not.is_empty() {
        let verb = if clauses.is_empty() {
            "should not cite"
        } else {
            "avoid citing"
        };
        clauses.push(format!(
            "{verb} {}",
            citation_prohibition_targets(&rules.should_not)
        ));
    }
    if !folded && let Some(clause) = citation_default_clause(project, rules, clauses.is_empty()) {
        clauses.push(clause);
    }
    if clauses.is_empty() {
        return None;
    }
    Some(clauses.join("; "))
}

/// Whether this kind's `may` list is the whole of what it permits, so a closed
/// per-kind default folds into it as `may cite only …` (§FS-init.2.3.5.5). With a
/// `must` or a `should` beside it the permitted set is wider than the `may`
/// list, and "only" would name the wrong set — those bullets keep the explicit
/// closing clause instead.
fn citation_permission_is_closed(rules: &KindCitationRules) -> bool {
    rules.default == Some(CitationLevel::MustNot)
        && !rules.may.is_empty()
        && rules.must.is_empty()
        && rules.warn.is_empty()
        && rules.should.is_empty()
}

/// The clause a per-kind `default` adds to its bullet (§FS-init.2.3.5.5), or
/// `None` when it changes nothing a reader could act on.
///
/// Only `must-not` and `should-not` defaults are load-bearing: §FS-config.3.9.4
/// makes a default of `must` / `should` / `may` invent no obligation and forbid
/// nothing, so those say something only where they punch a hole in a **closed
/// global** default — and then what they say is that this kind is open.
fn citation_default_clause(
    project: &Project,
    rules: &KindCitationRules,
    first: bool,
) -> Option<String> {
    let anything = if first { "anything" } else { "anything else" };
    match rules.default? {
        CitationLevel::MustNot if first => Some(format!("must not cite {anything}")),
        CitationLevel::MustNot => Some(format!("never cite {anything}")),
        CitationLevel::ShouldNot if first => Some(format!("should not cite {anything}")),
        CitationLevel::ShouldNot => Some(format!("avoid citing {anything}")),
        CitationLevel::WarnNot if first => Some(format!("must not cite {anything} (a warning)")),
        CitationLevel::WarnNot => Some(format!("never cite {anything} (a warning)")),
        CitationLevel::May | CitationLevel::Must | CitationLevel::Warn | CitationLevel::Should => {
            citation_default_is_closed(project.rules.citations.global_default)
                .then(|| format!("may cite {anything}"))
        }
    }
}

/// The section's closing line (§FS-init.2.3.5.6) — load-bearing either way, so an
/// agent neither over-infers prohibitions from silence nor misses a closed
/// world. A global default of `must` or `should` closes as open: it obliges
/// nothing and forbids nothing (§FS-config.3.9.4).
fn citation_closing_line(global_default: Option<CitationLevel>) -> String {
    match global_default {
        Some(CitationLevel::MustNot) => "Any citation not listed above is forbidden.".to_string(),
        Some(CitationLevel::ShouldNot) => {
            "Any citation not listed above is discouraged.".to_string()
        }
        Some(CitationLevel::WarnNot) => {
            "Any citation not listed above is reported as a warning.".to_string()
        }
        _ => "Anything not listed above is allowed.".to_string(),
    }
}

/// Whether a default level closes the world — the only two that do anything to
/// an unlisted pair (§FS-config.3.9.4).
fn citation_default_is_closed(level: Option<CitationLevel>) -> bool {
    matches!(
        level,
        Some(CitationLevel::MustNot | CitationLevel::WarnNot | CitationLevel::ShouldNot)
    )
}

/// Render an obligation's or a permission's rule array as a target phrase
/// (§FS-init.2.3.5.4): alternatives inside one entry are joined with "or",
/// conjunctive entries with "and". When there is
/// more than one entry, an entry that has alternatives of its own is
/// parenthesised — `must = ["FS|GOAL", "AR"]` is *(FS or GOAL) and AR*, and the
/// ungrouped prose said the opposite.
fn citation_rule_targets(disjunctions: &[CitationDisjunction]) -> String {
    let grouped = disjunctions.len() > 1;
    let entries = disjunctions
        .iter()
        .map(|disjunction| {
            let targets = disjunction
                .targets
                .iter()
                .map(citation_target_phrase)
                .collect::<Vec<_>>();
            let phrase = join_english_list(&targets, "or");
            if grouped && targets.len() > 1 {
                format!("({phrase})")
            } else {
                phrase
            }
        })
        .collect::<Vec<_>>();
    join_english_list(&entries, "and")
}

/// Render a prohibition's rule array as a target phrase (§FS-init.2.3.5.4,
/// §DF-prohibition-targets-join-with-or): each target is banned on its own, so every entry's
/// alternatives join one "or" list, unparenthesised — `must-not = ["FS", "AR"]`
/// is `FS or AR`, as `["FS|AR"]` is, where "and" read as banning only the pair.
fn citation_prohibition_targets(disjunctions: &[CitationDisjunction]) -> String {
    let targets = disjunctions
        .iter()
        .flat_map(|disjunction| disjunction.targets.iter().map(citation_target_phrase))
        .collect::<Vec<_>>();
    join_english_list(&targets, "or")
}

/// One rule target as prose (§FS-init.2.3.5.4). A pinned alias stays exactly as
/// spelled, because that is how the citation itself is written; `*/K` is rule
/// grammar that is never a citation (§FS-config.3.9.3), so it is said in words
/// instead of leaked into the entrypoint.
fn citation_target_phrase(target: &CitationTarget) -> String {
    match &target.namespace {
        NamespaceMatch::Local => target.kind.clone(),
        NamespaceMatch::Any => format!("{} in any project", target.kind),
        NamespaceMatch::Alias(alias) => format!("{alias}/{}", target.kind),
    }
}

/// An English list: `A`, `A or B`, `A, B, or C` — the Oxford comma from three
/// items on, so a three-way rule cannot be read as a two-way one (§FS-init.2.3.5.4).
pub(super) fn join_english_list(items: &[String], conjunction: &str) -> String {
    match items {
        [] => String::new(),
        [only] => only.clone(),
        [first, second] => format!("{first} {conjunction} {second}"),
        [head @ .., last] => format!("{}, {conjunction} {last}", head.join(", ")),
    }
}
