# DF-fmt-preserves-value-attempts: formatting preserves every refused value-binding attempt

## status: Accepted

## 1. Decision

Preserve each citation occurrence belonging to a split or malformed attempted value
binding during the default formatter link pass, using the classification that
already makes `check` refuse it ([§FS-values.8](../../functional-spec/FS-values.md#8-formatting-stability)).
This restores the existing agree-or-fail guarantee
([§FS-values.requirements.agree-or-fail](../../functional-spec/FS-values.md#requirementsagree-or-fail-a-binding-passes-only-by-agreeing--realized-one-case-deferred)):
formatting must not turn a refused attempt with literal `9999` against declared
`1200` into ordinary linked prose that passes unchecked. The accepted grammar,
attempt boundary, authority rules and checker diagnostics do not change.

## 2. Compatibility

The observable verdict that moves is `fmt --check`: it can now exit `0` where its
only proposed edit used to be linking a refused attempt. Dry runs propose fewer
links and `--write` writes fewer lines. The authored tree's `check` verdict and
message do not change; the same refusal survives formatting. An already linked
form is not reinterpreted as a binding by this correction.

This is a recorded formatter correction under
[§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered),
as with [§FS-values.9.1](../../functional-spec/FS-values.md#91-the-one-marked-root-formatting-behavior-that-moved).
No previously passing `check` tree acquires a finding, so the verdict-correction
route in section 5 of [§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly)
is not invoked. A deprecation period retaining the destructive link rewrite
would keep erasing an existing refusal; a mechanical migration is inapplicable
because there is no new accepted spelling or configuration to migrate to
([§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path),
[§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)).
The author fixes the already invalid binding form and then its literal if it
differs from the declaration. Ordinary citations continue to link, including a
separate occurrence naming the same target as a refused attempt.

## release-note: Release note

- [§DF-fmt-preserves-value-attempts](DF-fmt-preserves-value-attempts.md#df-fmt-preserves-value-attempts-formatting-preserves-every-refused-value-binding-attempt), [§FS-values.8](../../functional-spec/FS-values.md#8-formatting-stability): **Formatting preserves split and malformed value-binding attempts.** The default link pass no longer links their citation bytes and erases `check`'s existing refusal. `fmt --check` can now pass when that link was its only proposed change; previews and `--write` report fewer links. `check` keeps its existing grammar, diagnostics and verdict on the authored tree, and still refuses the attempt after formatting. Correct the binding's form and literal as its finding requests. Already linked prose is unchanged. Closes [issue #1433](https://github.com/agent-grounds/grund/issues/1433).
