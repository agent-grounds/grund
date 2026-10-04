# DF-off-grammar-declaration-compatibility: persisted declarations remain readable without relaxing the authoring grammar

**Status:** Accepted
**Date:** 2026-09-07

Exact read compatibility serves [§GOAL-no-dangling-refs](../../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) when a repository's configured ID format and its persisted declarations drift apart.

## 1. Context

A repository can change `[id].format`, or add a per-kind override, while a
declaration and its citations keep an older spelling. Rejecting that spelling
at the query boundary makes a fact visibly present in the tree unreadable and
leaves `show`, `check`, `list`, `refs`, completion, formatting, coverage, and
the LSP disagreeing about whether it exists. Silently accepting every
kind-prefixed token instead would replace the configured grammar with an
unbounded second grammar and turn malformed prose into citations.

## 2. Decision

The effective format remains the grammar for authoring and conformance. The
shared scanner retains a declaration-position, colon-terminated token that
unambiguously begins with a configured citable kind as a catalog declaration,
including its raw spelling, body, sections, and source site. Exact
marker-prefixed candidates rejected by normal grammar are promoted only after
catalog merge, and only when an exact declaration in the selected project
backs them. Every reader consumes that shared result ([§FS-config.3.2](../../functional-spec/FS-config.md#32-id--id-grammar)).

Configured full IDs retain precedence. All remaining shorthand, duplicate,
section, and workspace interpretations are considered without guessing: one
target resolves, zero preserves the current invalid or dangling outcome, and
multiple targets fail with sorted candidates or sites. A whole-token exact
declaration wins before the token is split as an inline section.

The format mismatch stays located and visible as `declaration-near-miss`. It is
a warning through 0.14.x and becomes an error in 0.16.0 under
[§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path),
without changing lookup or citation resolution at that boundary.

## 3. Rejected alternatives

**A `show`-only fallback** was rejected because retrieval would claim the ID
exists while graph and editor surfaces omit it. **Relaxing the configured
grammar globally** was rejected because unmarked candidates, unbacked marked
candidates, and malformed input would acquire meaning without repository
evidence. **Automatic rename** was rejected because the tool cannot choose
between changing the declaration and changing the configured format, nor can
it know every external citation.

## 4. Consequences

Repositories regain consistent reads immediately and receive one migration
diagnostic per mismatched declaration. Newly authored mismatches are readable
but diagnosed just like old ones; no history database distinguishes them. The
scanner/catalog is the compatibility boundary, so downstream commands and the
LSP do not grow parallel parsers. Existing configuration, authoring commands,
and JSON schemas remain unchanged. The scheduled severity change landed in
[§FS-declarations.checks.declaration-near-miss.5](../../functional-spec/FS-declarations.md#checksdeclaration-near-miss5-a-warning-before-0160-an-error-in-it).

The warning window closed in grund 0.16.0, on the schedule this record named:
`declaration-near-miss` is an error, a retained finding exits `1`, and the
message's last clause reads `this became an error in grund 0.16.0`. The
catalog-backed compatibility this record decided did not move: the declaration
and its exact marked citations stay readable through every reader. The
reasoning above stands as written and is not revised.

## release-note: Release note

- [§FS-declarations.checks.declaration-near-miss](../../functional-spec/FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss), [§FS-declarations.checks.declaration-near-miss.5](../../functional-spec/FS-declarations.md#checksdeclaration-near-miss5-a-warning-before-0160-an-error-in-it), [§FS-errors.5.5](../../functional-spec/FS-errors.md#55-the-check-code-catalog), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **a declaration off its `[id] format` now fails `grund check`.** The ramp every `0.14.x` and `0.15.x` binary announced in its own output lands on the release it named: ``docs/FS-billing.md:1: warning: `FS-billing` resolves for compatibility but does not match [id] format = "{kind}-{number}-{slug}" — rename it or change the effective format; this warning becomes an error in grund 0.16.0`` at exit `0` becomes the same line at `error:` ending `; this became an error in grund 0.16.0` at exit `1`, and JSON and the LSP report `"severity":"error"`. [§RM-off-grammar-declaration-error](../../roadmap.md#rm-off-grammar-declaration-error-make-off-grammar-declarations-a-check-error-in-0160) loses its plan but keeps its address and heading text as a pointer, because released changelogs cite it. Two e2e cases move from exit `0` to exit `1`: `check-declaration-near-miss` and `config-id-format-per-kind-override`. **What does not move:** the `code`, the location at the declaration line, the facts the message states, and the catalog-backed compatibility of [§FS-config.3.2](../../functional-spec/FS-config.md#32-id--id-grammar) — `show`, `refs`, `list`, `cover`, `fmt` and LSP navigation still resolve the ID and its exact marked citations exactly as they did. **Who this breaks:** a repository holding a declaration whose ID does not match its effective `[id] format`, including one that resolves only for compatibility — its `check` exits `1` where it exited `0`. The fixes are the ones the message names: rename the declaration and its citations, or change the effective format; `--ignore declaration-near-miss` removes the finding and the exit together. A consumer matching the message's last clause exactly sees it change; a `code` consumer reads the same code. Closes [issue #443](https://github.com/agent-grounds/grund/issues/443). (PR #446)
