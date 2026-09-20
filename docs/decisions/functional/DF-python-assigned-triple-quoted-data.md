# DF-python-assigned-triple-quoted-data: module-level assigned triple-quoted strings are data, not docstrings

**Status:** Accepted
**Date:** 2026-09-20

## 1. Context

Python uses the same triple-quote tokens for docstrings and for runtime string data. The shared
line reader recognized only a delimiter at the first non-whitespace column. A module-level
assignment such as `SAMPLE = """..."""` therefore supplied no opening state; its closing delimiter
could be mistaken for a new docstring opener, hiding citations in later real docstrings. At the
same time, a marked unqualified citation stored in the assigned value remained a graph edge under
the former “marker is the whole signal” rule.

Restoring later docstrings alone would leave runtime data able to declare graph intent and would
let `check`, `refs`, `cover`, formatting, and editor transforms disagree if each consumer patched
the ambiguity independently. Treating every triple-quoted expression as data would instead remove
real documentation and require Python scope analysis that [§FS-non-goals.3](../../functional-spec/FS-non-goals.md#3-code-ast-parsing)
explicitly rejects.

## 2. Decision

### 2.1 A bounded assignment is lexical evidence of data

With Python docstring scanning enabled, the simple unindented assignment form in
[§FS-check.1.1.3.1](../../functional-spec/FS-check.md#1131-assigned-python-triple-quoted-data)
opens an assigned-data span. Both triple-quote delimiters, the listed case-insensitive Python
prefixes, an optional annotation, one-line and multiline values, escaped would-be closes, and
same-line recovery are included. Parenthesized, destructuring, computed, and indented assignments
remain outside the rule. This deliberately distinguishes one high-confidence lexical form; it is
not a claim about Python scope or expression semantics.

### 2.2 Assigned data contributes no scanner meaning

No marked, bare, or namespace-qualified citation and no declaration, section, value, or inline
site inside the span enters the shared model. `check`, `refs`, `cover`, body resolution, formatting,
and editor consumers therefore see the same absence. The raw bytes remain available to writers
only as a protected span, while source after the close and later real docstrings resume their
existing treatment ([§FS-fmt.2.3.1.1](../../functional-spec/FS-fmt.md#2311-a-python-docstring-is-walked-for-its-content),
[§FS-refs.2](../../functional-spec/FS-refs.md#2-behaviour)).

### 2.3 The existing gate owns the distinction

`[scan] docstring_python = false` retains its raw-line behavior. Other Python strings, other
languages, Markdown, bare-token strictness, and qualified-source-string suppression are unchanged.
No new configuration, CLI option, output field, or exit code is introduced.

## 3. Rejected alternatives

### 3.1 Filter only `refs`

The symptom was visible in `refs`, but a query-only filter would leave the citation graph, unused
counts, grounding, formatting, and the LSP with different answers. Recognition belongs in the
shared reader and block classifier described by
[§AR-scanner.4.4](../../architecture/AR-scanner.md#44-comment-lines-are-normalized-before-detection).

### 3.2 Parse Python

An AST could distinguish more assignment and docstring contexts, but it would add language-version
and parser dependencies to a scanner designed around deterministic lexical forms. The bounded
assignment covers the reported false state without claiming semantic completeness.

### 3.3 Preserve assigned marked citations for one release

That would knowingly preserve the false edge this decision is meant to remove and make the shared
reader carry a temporary exception after the behavior was approved. Before 1.0, output and verdict
changes may correct behavior that was never promised ([§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise)); the release note provides the migration instead.

## 4. Consequences

Repositories that intentionally stored marked citations in qualifying assigned data lose those
edges. Move an intentional edge to a `#` comment or a real docstring; no configuration migration
is required. Recovered later docstrings may expose genuine dangling, shorthand, direction, value,
or grounding findings that the false quote state had hidden. Formatter and editor transforms leave
the assigned bytes unchanged while continuing to transform real docstrings.
