# DF-glob-citation: a glob operator ends no citation, and the prefix it left behind is removed

**Status:** Accepted
**Date:** 2026-10-10

## 1. Context and decision

The scanner ended a marked token at the first character outside the ID grammar, so a glob
operator after or inside a component cut the token short and the surviving prefix was recorded
as a citation. `<§>FS-login.*` became a citation of `FS-login`, `<§>FS-login.requirements.**` one
of `FS-login.requirements`, and `<§>FS-log*in.requirements`, where `FS-log` is declared, a
citation of `FS-log`: an edge to a declaration the author never named, which resolves, so
`check` passed. Where no prefix matched, as in `<§>FS-*.requirements` or `<§>FS-[a,b]`, the token
was dropped without a word. `grund fmt` then linked the prefix and left the rest of the pattern
dangling after the link. Patterns are about to be taught as coordinate globs, so writing one
where a citation belongs becomes a common mistake.

[§FS-check.1.1.11](../../functional-spec/FS-check.md#1111-a-glob-operator-never-shortens-a-marked-citation) settles it: a marked candidate is read whole. A declared ID is a citation
whatever it holds, a candidate the address grammar accepts keeps the ordinary dangling and
missing-section verdicts, and any other candidate holding an operator is a pattern: no edge is
recorded, and [§FS-check.checks.glob-citation](../../functional-spec/FS-check.md#checksglob-citation-glob-citation) reports it. Terminal emphasis, a sentence-final
`?`, a footnote reference, and a trailing `*` after a complete ID stay punctuation. No
configured ID grammar loses a character: a format or slug pattern that admits `*` is read by the
catalog and the grammar first, so [§FS-declarations.line.configured-slug](../../functional-spec/FS-declarations.md#lineconfigured-slug-characters-admitted-by-the-slug-pattern-belong-to-the-canonical-id) is unchanged.

Rejected: keeping the prefix edge and adding only a warning, which leaves the wrong edge in
`refs`, `cover`, inbound counts, and grounding for a whole release; and banning operator
characters from configured grammars, which removes a configuration contract rather than
correcting a reading.

## 2. Verdict correction and compatibility

Two verdicts move, and they take different routes. The `glob-citation` finding is new and is a
warning first: its promotion to an error in grund 0.19.0 is the deprecation path of
[§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), recorded in [§FS-check.checks.glob-citation.1](../../functional-spec/FS-check.md#checksglob-citation1-a-warning-before-0190-an-error-in-it). Removing the prefix
edge is not ramped. Without it a declaration cited only through a pattern earns the `unused`
warning, and a unit grounded or satisfying a `must` direction only through one can now fail
`ungrounded`, `missing-citation`, or a chapter rule; a pattern whose prefix dangled or named a
missing section stops failing on that prefix and earns the warning instead.

The applicable route for the edge is [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids):

1. **Prior prohibition.** The old prefix verdict violated [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) by substituting a near miss for the point the citation named, which the resolver must never do.
   That prohibition already applied when the old verdict shipped, in every release from 0.10.1 through 0.17.0.
2. **Accepted proof.** This record documents that conflict.
   [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) does not fit, because a release that kept the wrong edge for a window would ship the violation it exists to end.
   [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) does not fit either, because no single rewrite can decide whether a pattern meant one exact point, a different point, or an illustration to escape.
3. **Named release.** The correcting release must include the compatibility notice below.
4. **Actionable findings.** Each pattern is reported at its location by
   [§FS-check.checks.glob-citation](../../functional-spec/FS-check.md#checksglob-citation-glob-citation), whose message names the two actions: cite the point, or
   escape the illustration. Findings the removed edge exposes keep their existing located
   messages and remedies.
5. **No new licence.** This route cannot justify ordinary policy tightening or a new prohibition.
   It removes an edge the author never wrote; the new finding itself takes the deprecation path
   rather than this route, and no configured grammar is restricted.

## release-note: Release note

- A marked citation holding a glob operator, such as `<§>FS-login.*` or `<§>FS-log*in.requirements`, is no longer cut to the prefix before the operator: no citation of that prefix is recorded, `grund fmt` no longer links it, and `check` reports the pattern as the `glob-citation` warning, which becomes an error in grund 0.19.0. A declaration cited only through such a pattern may now be reported unused, and a unit grounded or satisfying a citation rule only through one may now fail that rule. Cite the exact point, or escape the illustration with the configured marker wrapped in angle brackets. This verdict change is the correction recorded in [§DF-glob-citation.2](DF-glob-citation.md#2-verdict-correction-and-compatibility), under [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), for the prior violation of [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution). Every finding names its location and an action the maintainer can take.
