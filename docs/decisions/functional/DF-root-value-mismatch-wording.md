# DF-root-value-mismatch-wording: a root-aimed value mismatch names the cited root and the literal as written, after one release that appends it

**Date:** 2026-10-25
**Supersedes:** The reporting choice of [§DA-root-aimed-value-bindings.2](../architectural/DA-root-aimed-value-bindings.md#2-decision), that a root-aimed mismatch whose part count equals the component count reports the first unequal component by its own coordinate in the canonical text, and the reason [§DA-root-aimed-value-bindings.3](../architectural/DA-root-aimed-value-bindings.md#3-why) gives for it. The binding itself, the space rule and the report of a part-count disagreement stand, and so does that record.

Amends [§FS-values.5.2.2](../../functional-spec/FS-values.md#522-a-root-aimed-mismatch-names-the-first-unequal-component), [§FS-values.3.1.2](../../functional-spec/FS-values.md#312-a-binding-aimed-at-the-root) and [§FS-values.5.2.1](../../functional-spec/FS-values.md#521-a-mixed-kind-mismatch-names-each-sides-kind), and stages the change in [§FS-errors.3.9](../../functional-spec/FS-errors.md#39-the-root-aimed-value-mismatch-wording-migration). Serves [§GOAL-friendliness-first.1](../../goals.md#1-hard-requirements): a finding should send its reader to the edit that fixes it. Reported as [grund#1008](https://github.com/agent-grounds/grund/issues/1008).

## status: Accepted

## 1. Context

A spec declares an idle timeout as one value of two components, `45` and `minutes`, under `FS-session.1`. An operations document binds the whole value, `30 minutes`, to that root; a code comment binds the amount alone, `30`, to the component `FS-session.1.1`. Both are stale, and `grund check` reported both with the same line:

```text
docs/operations.md:3: error: value mismatch for FS-session.1.1: bound `30`, declared `45` at requirements.md:11
src/sessions/limits.py:3: error: value mismatch for FS-session.1.1: bound `30`, declared `45` at requirements.md:11
```

The first site cites `FS-session.1` and wrote `30 minutes`, but its finding names a coordinate the site does not cite and quotes a literal it does not contain. The two sites need different edits, `30 minutes` to `45 minutes` and `30` to `45`, and the message cannot tell them apart. An agent reading the first line concluded the citation was wrong. It re-cited the site to `FS-session.1.1` keeping `45 minutes`, which failed on a number against a string, then took the whole-value binding apart and re-cited an untouched line to match. The binding [§DA-root-aimed-value-bindings](../architectural/DA-root-aimed-value-bindings.md#da-root-aimed-value-bindings-a-value-binding-may-aim-at-the-whole-root-its-components-joined-by-one-ascii-space) was made to allow was undone because of the message that reports on it.

That cost was named in advance. The clarification round of grund#375 put the message to the owner as its third question, and the reading taken, the component by its own coordinate, was known to quote "a slice of the author's literal rather than the literal they wrote". Its gain was that the canonical text, the one `sites` entry and the one editor span stayed unchanged to the byte. The same site already got the root and the literal as written whenever the part count differed.

## 2. Decision

A root-aimed mismatch whose part count equals the component count names what the site wrote, and still names the one component that differs:

```text
value mismatch for <root> (component <section-path>): bound `<literal>`, declared `<joined>` at <site>
```

`<root>` is rendered as the part-count branch renders it, `<literal>` is the literal as authored, `<joined>` the components joined by one ASCII space, `<section-path>` the first unequal component's section path after the ID, and `<site>` that component's one declaration site ([§FS-values.5.2.2](../../functional-spec/FS-values.md#522-a-root-aimed-mismatch-names-the-first-unequal-component)). Where that pair differs in kind, the kind clause names it as `component <section-path>` ([§FS-values.5.2.1](../../functional-spec/FS-values.md#521-a-mixed-kind-mismatch-names-each-sides-kind)). The finding keeps one declaration site, so `sites` keeps its one entry and the editor its one span. A component-aimed mismatch and a part-count mismatch keep their bytes.

It ships across two releases ([§FS-errors.3.9](../../functional-spec/FS-errors.md#39-the-root-aimed-value-mismatch-wording-migration)). In 0.20.1 the complete message earlier releases printed stays in front, followed by ` — whole binding: `, the new message, and `; this wording changes in grund 0.21.0`. In 0.21.0 the new message stands alone. The release guard reads that suffix and holds 0.21.0 until it is gone ([§FS-distribution.4.2.2](../../functional-spec/FS-distribution.md#422-the-release-guard-reads-the-releases-the-tree-names)).

## 3. Why

The cited root and the literal as written are what the reader can find on their own line, so the line says the citation is right and the literal is stale. `(component <section-path>)` keeps what the earlier reading was right to keep: the one part that differs, at the one site an editor jumps to. The cost is that the root-aimed line no longer matches the canonical text word for word.

A single-component root stays equivalent to its `.1` spelling in verdict, and only in verdict ([§FS-values.3.1.2](../../functional-spec/FS-values.md#312-a-binding-aimed-at-the-root)): byte identity was the property that made the two sites indistinguishable.

The message is bytes a consumer may match ([§FS-errors.requirements.prefix-windows](../../functional-spec/FS-errors.md#requirementsprefix-windows-a-wording-change-keeps-the-old-message-whole-until-its-window-closes--realized-directional-for-the-next-change)), so the old message is kept whole for a release and the new one is appended after it, which a prefix or `code` consumer cannot see.

## 4. Compatibility

No verdict moves. The code `value-mismatch`, the error severity, the primary location, the one `sites` entry, `--only` and `--ignore`, and the exit status are what they were for every tree, so this is a wording change under [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), not a verdict correction, and the five-condition route of section 5 of `REQ-backwards-compatibility` does not apply.

In 0.20.1 an exact-line consumer of a root-aimed mismatch stops matching, in the text line, the `--format=json` `message`, and the editor diagnostic alike. A consumer of the legacy prefix keeps matching until 0.21.0. Both migrate to `code == "value-mismatch"`.

## 5. Alternatives considered

| Option | Why not |
|---|---|
| Keep the component-only message | It is the cost reported in grund#1008: a right citation reads as the thing to change. |
| Report the whole literal against the joined declaration with every component's site | It makes `sites` plural and needs a new go-to target, as [§DA-root-aimed-value-bindings.6](../architectural/DA-root-aimed-value-bindings.md#6-alternatives-considered) found, and it leaves the kind clause with no pair to describe. |
| Name the root and the literal with no component | The reader still has to work out which part differs, and the one site the finding carries is a component's, which the line would no longer name. |
| Replace the message outright in 0.20.1 | It breaks the stable phrasing of [§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text) with no window. |

## release-note: Release note

- [§FS-errors.3.9](../../functional-spec/FS-errors.md#39-the-root-aimed-value-mismatch-wording-migration): **a value mismatch at a binding aimed at a whole value names the cited root and the literal as written.** For `` `30 minutes` `` bound to the root `FS-session.1`, `grund check` printed ``value mismatch for FS-session.1.1: bound `30`, declared `45` ``, the line a binding of `30` to the component earns. From 0.20.1 that message is kept whole and followed by `` — whole binding: value mismatch for FS-session.1 (component 1.1): bound `30 minutes`, declared `45 minutes` at requirements.md:11; this wording changes in grund 0.21.0 ``; in 0.21.0 the part before ` — whole binding: ` and the suffix go. A component-aimed mismatch and one whose part count differs are unchanged. **Who this affects:** a script matching a root-aimed `value-mismatch` line whole in text, in the `message` of `--format=json`, or in the `message` of a Python or Node finding, now; one matching its prefix, from 0.21.0. Match `code == "value-mismatch"` instead. The code, the severity, the line, the `sites`, the exit status and the JSON shape are unchanged. Closes [issue #1008](https://github.com/agent-grounds/grund/issues/1008).
