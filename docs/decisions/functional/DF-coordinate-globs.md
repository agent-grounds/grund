# DF-coordinate-globs: coordinates are selected by a glob of their own, and a brace before a separator opens a pattern

**Date:** 2026-10-18

## status: Accepted

## 1. Context and decision

A reviewer who wanted every point of a checklist — every requirement of a chapter, every Terms
chapter of a kind — had one way to get their bodies: `list --selector … --format json`, a `jq`
program to turn its rows into queries, and `show --batch` to read them. The selector could not
grow into the answer, because its grammar is rule subjects, which express neither a union nor
a depth. So issue #503 gives coordinates a glob of their own, the one shape both humans and
agents already read: [§FS-coordinate-globs](../../functional-spec/FS-coordinate-globs.md#fs-coordinate-globs-a-coordinate-glob-selects-catalog-coordinates-by-pattern). `list --glob` selects rows beside `--selector`,
`list --total` counts them, `show --batch --glob` reads their bodies in one scan, and the
bindings take the same patterns as `globs`. The owner's review step is the shipped
`cover FILE --lines N-M` ([§FS-cover.6](../../functional-spec/FS-cover.md#6-line-ownership)).

Rejected: extending the selector, which would have changed what a rule subject admits;
matching through the `exclude` matcher with coordinates spelled as `/` paths, which would
import a dialect whose departures ([§FS-coordinate-globs.grammar.departures](../../functional-spec/FS-coordinate-globs.md#grammardepartures-where-it-departs-from-the-exclude-dialect)) are each wrong
for coordinates; a `total` keyword on the bindings, where a host already counts `entries`;
and `refs --total`'s sentence for `list --total`, which is a hover's words for a site and file
pair that `list` has neither of, where a bare count compares with `wc -l` ([§FS-list.total](../../functional-spec/FS-list.md#total---total-a-count-rung)).

## 2. The rulings it carries

The grammar is the issue's, settled by a two-family discussion and the owner's rulings on its
disputes, with these changes:

- **V-D03, amended.** The first ruling read every pattern with one dotted grammar. It was
  reversed because a project's separator is its own: inside one project the first unescaped
  occurrence of its outer separator ends the ID atom, and with none the first unescaped `.`
  does ([§FS-coordinate-globs.grammar.components](../../functional-spec/FS-coordinate-globs.md#grammarcomponents-where-the-id-atom-ends)). A pasted `#` citation is therefore a valid
  pattern in its project, and the escaped dotted spelling keeps working
  ([§FS-coordinate-globs.workspace.escapes](../../functional-spec/FS-coordinate-globs.md#workspaceescapes-two-spellings-of-one-coordinate)).
- **V-A09, dropped.** The issue refused a named literal after a numbered one, as in
  `FS-00x.1.**.requirements`. A mixed path such as `1.goals` is a legal coordinate on `main`
  ([§FS-config.3.3.1](../../functional-spec/FS-config.md#331-named-components)), so the pattern is valid, matches like any other, and exits `0`.
- **The provably-empty message is a note.** It never changes stdout or the exit, and it is
  printed on every `--glob` surface ([§FS-coordinate-globs.errors.note](../../functional-spec/FS-coordinate-globs.md#errorsnote-a-pattern-no-kind-can-start-gets-a-note)).
- **Messages take the house quoting.** A quoted fragment and every accepted input are
  double-quoted, several accepted inputs are joined with `or`, the note's suggestion is the
  `; did you mean …?` clause, and the PATH hint puts the command to paste in backticks
  ([§FS-coordinate-globs.errors](../../functional-spec/FS-coordinate-globs.md#errors-refusals-notes-and-hints)). Each spelling freezes at 1.0.
- **The block is taught from the format.** The managed block's recipe builds its pattern from
  the project's `[id] format`, so a `{kind}{number}` project is never taught a `-` its IDs do
  not have ([§FS-init.2.3.4.3.1](../../functional-spec/FS-init.md#23431-structural-queries)).

`--selector` and every rule sentence are unchanged, and no rule admission moves ([§FS-rules.2](../../functional-spec/FS-rules.md#2-subject-selectors)).

## 3. The slice this change ships

This change is the first slice: the grammar and matcher, `list --glob`, `list --total`,
`show --batch --glob`, the bindings' `globs` and `note`, the citation guard's widening, and the
teaching. `refs --glob`, `refs --from` and `cover --glob` follow in an issue of their own,
carrying this issue's design and its two open questions, and the relation flags `--cites`,
`--cited-by` and `--cited-from` stay designed and unbuilt until a consumer asks for them.
`cover --at` is not built: `cover FILE --lines N-M` already answers it.

Teaching the glob recipe in place of the `jq` pipe changes the managed block's fixed text, so
both current version pairs advance: v23/v24 becomes v27/v28 and v25/v26 becomes v29/v30, and
one `grund init` re-renders an outdated block ([§FS-init.2.3.7.1](../../functional-spec/FS-init.md#2371-the-current-version)).

## 4. The citation guard's verdict correction

Teaching brace globs makes one more prefix edge common. The citation guard opened a `{` only
before an address character or `,`, so in prose `<§>FS-x{.1,.2}` — a valid coordinate glob —
was recorded as a citation of `FS-x`, with no finding. The guard now opens a `{` before the
section separator too ([§FS-citations.recognized.globs](../../functional-spec/FS-citations.md#recognizedglobs-a-glob-operator-never-shortens-a-marked-citation)): the candidate is read whole, is a
pattern, and is reported as [§FS-check.checks.glob-citation](../../functional-spec/FS-check.md#checksglob-citation-glob-citation). A declaration cited only through
such a pattern may now be reported unused, and a unit grounded or satisfying a `must` direction
only through one can now fail it.

The applicable route is [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids):

1. **Prior prohibition.** The old brace verdict violated [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) by recording the declaration before the brace as the point the citation named, a near miss the resolver must never substitute.
   That prohibition already applied when the old verdict shipped, in every release from 0.10.1 through 0.18.0.
2. **Accepted proof.** This record documents that conflict, which is the one [§DF-glob-citation](DF-glob-citation.md#df-glob-citation-a-glob-operator-ends-no-citation-and-the-prefix-it-left-behind-is-removed) corrected for every other operator.
   [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) does not fit, because a release that kept the wrong edge for a window would ship the violation it exists to end.
   [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) does not fit either, because no single rewrite can decide whether a pattern meant one exact point, several, or an illustration to escape.
3. **Named release.** The correcting release must include the compatibility notice below.
4. **Actionable findings.** Each pattern is reported at its location by
   [§FS-check.checks.glob-citation](../../functional-spec/FS-check.md#checksglob-citation-glob-citation), whose message names the two actions: cite the point, or
   escape the illustration. Findings the removed edge exposes keep their existing located
   messages and remedies.
5. **No new licence.** This route cannot justify ordinary policy tightening or a new prohibition.
   It removes an edge the author never wrote, and the finding it reports is the one
   [§DF-glob-citation](DF-glob-citation.md#df-glob-citation-a-glob-operator-ends-no-citation-and-the-prefix-it-left-behind-is-removed) already put on its deprecation path.
   That path ends in grund 0.19.0, the release this correction ships in, so the pattern is
   reported as an error from the start ([§DF-glob-citation-lands-in-0-19](DF-glob-citation-lands-in-0-19.md#df-glob-citation-lands-in-0-19-a-glob-citation-is-an-error-from-0190)).

## release-note: Release note

- The managed block now teaches coordinate globs in place of the `jq` pipe, so both current block versions move: v23/v24 becomes v27/v28 and v25/v26 becomes v29/v30, `grund check` and `grund init --check` report every earlier block outdated, and one `grund init` re-renders it, keeping unmanaged content. A marked citation whose brace group opens before the section separator, such as `<§>FS-login{.1,.2}`, is no longer recorded as a citation of `FS-login`: `check` reports the pattern as the `glob-citation` error, which fails the run, so a declaration cited only through one may now be reported unused, and a unit grounded or satisfying a citation rule only through one may now fail that rule. Cite each exact point, or escape the illustration with the configured marker wrapped in angle brackets. This verdict change is the correction recorded in [§DF-coordinate-globs.4](DF-coordinate-globs.md#4-the-citation-guards-verdict-correction), under [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), for the prior violation of [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution). Every finding names its location and an action the maintainer can take.
