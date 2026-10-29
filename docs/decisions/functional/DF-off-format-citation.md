# DF-off-format-citation: a marked token its kind's format does not spell, and nothing declares, is reported rather than dropped

**Date:** 2026-10-29

## status: Accepted

## 1. Context

Under the default `slug_pattern = "[a-z][a-z0-9-]*"`, a tree whose `docs/FS/FS-a.md` reads
`See <§>REQ-999 and <§>REQ-12a and <§>REQ-a12.`, with no requirement declared, got one finding
from `check`: `unknown reference REQ-a12`. The two tokens whose slug starts with a digit got
none, in text or in JSON, and with `<§>REQ-a12` deleted `check` passed
([#1073](https://github.com/agent-grounds/grund/issues/1073), reproduced on `main` at
377ff0f2e9). Whether a citation of a missing requirement passed depended on the first
character of its slug, and a numeric slug is what an author used to ticket numbers writes
first. `grund show REQ-999` already answered `invalid ID`.

The scanner kept the token as a candidate. The off-grammar pass of [§FS-config.3.2.5](../../functional-spec/FS-config.md#325-off-grammar-declarations-stay-readable) promotes a
candidate that an exact declaration backs, and it dropped every other candidate without a word.
[§DF-off-grammar-declaration-compatibility](DF-off-grammar-declaration-compatibility.md#df-off-grammar-declaration-compatibility-persisted-declarations-remain-readable-without-relaxing-the-authoring-grammar) gave an unbacked candidate no compatibility meaning,
which settles that the candidate resolves to nothing, but neither says what `check` reports about
it. [§REQ-no-missed-citation](../../requirements/REQ-no-missed-citation.md#req-no-missed-citation-every-citation-the-run-reads-is-checked) says every marked citation a run reads is recognized and checked, and
its list of blind spots, [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded), does not name this one.

## 2. Decision

A marked token is an off-format citation when it opens with one configured kind, its kind's
effective format does not spell it, whole or before a section path, no other reading claims its
marker, and nothing declares it ([§FS-citations.recognized.off-format](../../functional-spec/FS-citations.md#recognizedoff-format-a-marked-token-its-kinds-format-does-not-spell-is-reported-never-dropped)). It still cites nothing, so
the catalog-backed boundary of [§DF-off-grammar-declaration-compatibility](DF-off-grammar-declaration-compatibility.md#df-off-grammar-declaration-compatibility-persisted-declarations-remain-readable-without-relaxing-the-authoring-grammar) stays where it is. The
backed side does not move either: once a declaration of exactly that spelling exists, the token
is a citation and only `declaration-near-miss` is reported, at the declaration.

`check` reports the token at its marker as `off-format-citation`
([§FS-check.checks.off-format-citation](../../functional-spec/FS-check.md#checksoff-format-citation-off-format-citation)). The message names the whole token as written, its
section path included, then the kind's template with the key that owns it and the pattern of
each component the template uses, because the template alone does not say why `999` fails. Last
come the two remedies: cite an ID in that format, or escape the illustration. It guesses no
corrected ID. Under `--full` an out-of-scope site is `out-of-scope-off-format-citation`
([§FS-check.3.14.5](../../functional-spec/FS-check.md#3145-a-compound-code-per-rule)).

It fires wherever a dangling citation fires: in prose, inline code, comments, doc-comments,
string literals and raw strings. It is silent exactly where a dangling citation is. An earlier
draft exempted source strings and inline code. It was dropped because the exemption is a skip
[§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded) would have had to declare, and its only purpose was illustrations,
which the escape `<§>` and a spelled-out marker such as `\u{a7}` already write without a finding.

## 3. Rejected alternatives

- **Report it as `dangling`.** `unknown reference REQ-999` tells the author to declare
  `REQ-999`, and that declaration is itself an error, `declaration-near-miss`, so the advice
  fails when it is followed. It would also carry `dangling`'s fetch and near-ID tails, and fall
  under `--ignore dangling`.
- **A warning release first.** The deprecation path ramps a verdict that had a meaning. This
  construct had none, and a warning would keep `check` green over a citation of nothing for one
  more release.
- **Declare the silence as a blind spot.** That would turn a false negative, which
  [§GOAL-no-dangling-refs](../../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) calls a bug, into a documented one.

## 4. Compatibility

A tree with such a token, in any text where a dangling citation is reported, exits `1` where it
exited `0`. The route is [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise). The construct had no defined meaning
and produced no output: [§FS-config.3.2.5](../../functional-spec/FS-config.md#325-off-grammar-declarations-stay-readable) says such a candidate "gains no compatibility meaning".
Giving it one is therefore not a break. The precedent is a marker followed by a bare section
number, which went from silent to an error the same way ([§DF-declaration-local-section-shorthand.4](DF-declaration-local-section-shorthand.md#4-consequences)).

The route is not <§>REQ-backwards-compatibility.5. That route corrects a verdict a prior
requirement forbade. Here nothing `check` reported was wrong; what it left out was, and no prior
requirement named a verdict to correct.

The finding names the releases it moved between, `— unchecked in grund 0.20.0, an error in
0.21.0`, as the local-section citation's does (§FS-check.3.24.1), and §FS-check.checks.off-format-citation.1 records
the flip. No single command fixes it. Only the author can tell whether a token meant a missing
ID, a mistyped one, or an illustration, so `grund fmt` rewrites nothing. grund's own tree is
rewritten in the change that lands this rule. In a Rust string the marker is spelled `\u{a7}`, in
a Python string `\u00a7`, and a comment or Markdown illustration takes `<§>`.

## release-note: Release note

- A marked token that opens with a configured kind but does not match that kind's ID format, such as `<§>REQ-999` or `<§>REQ-12a` under the default `slug_pattern`, and that nothing declares, was dropped without a word, so a citation of a missing ID written this way passed `check`. It is now the `off-format-citation` error, and `out-of-scope-off-format-citation` under `--full`. It is reported wherever a dangling citation is: in prose, inline code, comments, doc-comments and source strings. A tree that carries such a token exits `1` where it exited `0`. There are two fixes: cite an ID in the kind's format, or, if the token is an illustration, escape it with the configured marker wrapped in angle brackets, `<§>REQ-999`. A token that a declaration of exactly that spelling backs is unchanged: it resolves, and only its declaration is reported. The construct had no defined meaning, so giving it one is the change licensed by [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise), as recorded in [§DF-off-format-citation.4](DF-off-format-citation.md#4-compatibility).
