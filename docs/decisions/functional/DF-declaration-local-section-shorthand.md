# DF-declaration-local-section-shorthand: local numeric section citations are recognized but never canonical

**Status:** Accepted
**Date:** 2026-09-20

## 1. Context

A configured marker followed only by a numeric section path is visually a citation but was
outside the citation grammar. Persisted forms such as `<§>2` and `<§>2.1` therefore vanished
from `check`, graph queries, formatting, and editor navigation. That silent verdict conflicts
with [§GOAL-no-dangling-refs](../../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration)
and [§FS-check.1.1](../../functional-spec/FS-check.md#11-recognized-citations).

The apparent target depends on where the prose sits. Accepting the short spelling as canonical
would make moving prose silently retarget it and would leave a reader outside the declaration
unable to identify the edge. Rejecting the token without first recognizing it would preserve
the original blind spot.

## 2. Decision

### 2.1 Recognition precedes canonicalization

A configured marker plus one or more decimal components separated by literal dots is a
declaration-local section candidate ([§FS-check.1.1.8](../../functional-spec/FS-check.md#118-declaration-local-numeric-section-candidates)).
The candidate is recognized under strict and non-strict scanning. An unmarked numeric path is
ordinary prose.

Recognition is deliberately broader than acceptance. A candidate inside one unambiguous
enclosing declaration becomes an ordinary edge to that declaration and section, while retaining
its authored local spelling. It then receives the canonical-form error in
[§FS-check.3.24](../../functional-spec/FS-check.md#324-declaration-local-section-citation).
An ownerless or otherwise ambiguous candidate receives an actionable error and no guessed edge.

### 2.2 Existing body ownership is the only owner

Ownership is the declaration body already assigned by the scanner: Markdown ends at the next
same-or-higher heading; a source declaration ends at its comment or docstring boundary; and the
nearest preceding declaration owns the remainder of a multi-declaration source comment. A
neighboring citation and a declaration in another file are never ownership evidence
([§AR-scanner.2.4](../../architecture/AR-scanner.md#24-citing-side-classification)).

### 2.3 Whole tokens receive one verdict

Full-ID citations and number-only ID shorthand keep precedence. A local candidate ends only at a
whole-token boundary. A digit-starting tail such as `<§>2.goals` or `<§>2abc` is rejected as one
unsupported local token; it is never truncated to section `2`. Named and mixed declaration-local
shorthand are not introduced by this decision.

### 2.4 Resolved local forms are real edges

An owned candidate participates in missing-section checking, `refs`, `cover`, unused-declaration
counting, grounding, citation directions, and every LSP graph surface as the same ordinary edge
([§FS-check.3.24](../../functional-spec/FS-check.md#324-declaration-local-section-citation)).
Form and target existence are independent: a missing local section receives both the canonical-
form error and the ordinary missing-section error.

### 2.5 Stored citations carry their full ID

The formatter expands an owned local form to its full citation only when the owner is unique and
the existing writer rules permit that location ([§FS-fmt.2.4](../../functional-spec/FS-fmt.md#24-shorthand-to-canonical)).
Suppressed scopes, external symlink targets, fences, inline code, Markdown link destinations,
declaration headings, and source strings keep their existing protections. The diagnostic still
names the manual full replacement for an owned protected site. An ownerless site is left unchanged
and told to write a full citation or escape the illustration. No LSP quick-fix and no `$$2` typing
expansion are added.

## 3. Rejected alternatives

### 3.1 Accept the persisted local form

This makes a citation's identity depend on its current container and lets an ordinary move change
what it means without changing its bytes. It also splits repository search between full and local
spellings. Canonical stored citations avoid both costs.

### 3.2 Diagnose without adding an edge

That would make `check` loud while leaving `refs`, `cover`, unused counting, grounding, citation
directions, and editor queries blind to a target the scanner already resolved. All citation
surfaces must share the verdict.

## 4. Consequences

Repositories that persisted marker-plus-numeric local paths become newly red. Unambiguous owned
sites have a mechanical migration; cross-document prose, ownerless navigation labels,
hypotheticals, and broken section paths require review rather than guessed rewriting. The change
is permitted by [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise)
because the formerly silent form was never accepted syntax and now has this explicit decision and
migration path.
