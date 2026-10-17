# DF-qualified-comment-quotes: recognize qualified comment citations after nonliteral single quotes

**Date:** 2026-10-17

## status: Accepted

## 1. Context and decision

[§FS-citations.recognized.contexts](../../functional-spec/FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code)
excludes qualified citations inside source literals and inline code. Reusing the formatter's
raw quote toggle silently extends that exclusion to a comment after `Don't`, a Rust
`&'static` lifetime, or a Haskell `f'` identifier. None opens a literal containing the
comment citation. Restore recognition in these supported comment sites, for unknown
aliases and configured workspace aliases alike, with the usual edge and target findings.

Keep the separate conservative formatter policy of
[§FS-fmt.2.3.1](../../functional-spec/FS-fmt.md#231-string-literal-exclusion-rule)
and the bare-token policy that adopts it
([§FS-citations.recognized.string-literals](../../functional-spec/FS-citations.md#recognizedstring-literals-string-literals-in-source-files)).
Genuine strings, inline-code spans, carried multi-line literals and Markdown retain their
existing recognition rules. A global single-quote exemption or broader lexer policy is
not this decision.

## 2. Verdict correction and compatibility

Recovering these marked citations may turn a passing check into an unknown-alias,
dangling-target, missing-section or existing citation-policy error. The immediate
correction follows
[§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids):

1. **Prior prohibition.** The old passing verdict violated [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded) by silently missing an explicitly marked citation outside its bounded exclusion.
   That prohibition already applied when the old verdict shipped. It bounded the
   qualified blind spot to source inline code and string literals, without excluding
   supported comments following nonliteral single quotes.
2. **Accepted proof.** This accepted record proves the conflict.
   [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) does not fit: no syntax or configuration is being retired, and delaying recognition
   would preserve a forbidden silent skip of already marked citations. A mechanical
   migration under [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) cannot fit either: no rewrite is needed to make these citations intentional, and no
   single command can choose whether to correct an alias, restore a target, or escape
   an illustration. Moving comments around is a workaround for the defect, not migration.
3. **Named release.** The correcting release includes the notice below through the
   decision release-note mechanism.
4. **Actionable findings.** Recovered sites receive the existing located findings.
   Maintainers correct or configure an unknown alias, correct or declare a target or
   section, satisfy the named policy, or escape an intended illustration. No finding
   format changes.
5. **No new licence.** This route cannot justify ordinary policy tightening or a new
   prohibition. It restores the existing qualified-comment promise, removes no feature
   and does not authorize changing formatter, bare-token or genuine-literal policy.

## release-note: Release note

- Qualified citations in supported source comments after prose apostrophes, Rust
  lifetimes and Haskell identifier primes are now recognized. Repositories may fail
  existing checks for previously hidden unknown aliases, missing targets or sections,
  or citation-policy violations. Follow the located finding to correct/configure the
  alias, correct/declare the target or section, satisfy the policy, or escape an
  illustration with the marker in angle brackets. Formatter and bare-token behavior
  are unchanged. This verdict change follows
  [§DF-qualified-comment-quotes.2](DF-qualified-comment-quotes.md#2-verdict-correction-and-compatibility)
  under [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids)
  for the prior violation of [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded).
  Every finding names its location and an action the maintainer can take; the existing
  remedies apply to recovered citations.
