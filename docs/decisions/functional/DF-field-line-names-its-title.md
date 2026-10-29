# DF-field-line-names-its-title: A field line names the heading title the field is held to

**Date:** 2026-10-29

Settles how `grund schema` and the managed block's `### Declaration shapes` name a field's configured `title`, as [§FS-schema.4.1](../../functional-spec/FS-schema.md#41-fields-are-lines-inside-their-kinds-node) and [§FS-init.2.3.5.12.2](../../functional-spec/FS-init.md#235122-declaration-shapes) specify, and why that moves the block's bytes under the pairs [§FS-init.2.3.7.1](../../functional-spec/FS-init.md#2371-the-current-version) already advanced. Serves [§FS-fields.requirements.record](../../functional-spec/FS-fields.md#requirementsrecord-the-agent-and-a-schema-view-read-the-shape-the-checks-read--realized): the title is a slot the checks compare byte for byte ([§FS-fields.identity.match](../../functional-spec/FS-fields.md#identitymatch-titles-and-values-match-byte-for-byte)), so a view that omits it tells an agent the shape only after the fact. Reported as [grund#1070](https://github.com/agent-grounds/grund/issues/1070).

## status: Accepted

## 1. Context

A kind whose fields set `title`, the shape `examples/rust-rfc` uses, has `grund check` refuse a heading that differs from the title by one byte (`Guide-level Explanation` for `Guide-level explanation`). The field lines of `grund schema` and of the block named the field's presence, placement and content, never its title, so an agent that read the block learned the handles and the order and guessed the heading text.

## 2. Decision

### 2.1 The title is the last fact on the line

A titled field's line ends with ` · titled ` and the title, after every fact it carried before: ``field summary  must · first · titled `Summary` ``. The table aligns one facts column across its sections, so the title goes there rather than beside the name, where it would push the places column of the whole table right. Putting it last keeps every changed line's prefix as it was. The word is `check`'s own for the same fact (`kind RFC titles it "Guide-level explanation"`).

### 2.2 The title is a code span

The title is written as a Markdown code span: its bytes between fences one backtick longer than the longest run inside it, padded with one space inside each fence where it begins or ends with a backtick. A code span is the one delimiter that keeps a title's `*`, `&` or `"` literal in the block and still reads as exact text in a terminal, and it keeps a title's own `·`, `:` or `,` from reading as a separator. A control character is written as the table already writes one in any cell, its TOML basic-string escape, so a bullet stays one line. Mermaid and DOT add only the escapes they already apply to every label ([§FS-schema.11](../../functional-spec/FS-schema.md#11-escaping)).

### 2.3 Untitled lines keep their bytes

A field with no `title`, and every slot that cannot carry one (a scalar, a `front-matter` field and its keys, a value kind's `field <number>`), prints the line it printed before. `grund check` and its findings do not change.

### 2.4 What it costs

Every block whose config declares a titled field changes bytes. Those configs render v45 or v46, and no release has written v43–v46, so the change shares those pairs rather than adding one ([§FS-init.2.3.7.1](../../functional-spec/FS-init.md#2371-the-current-version)).

## 3. Alternatives considered

| Option | Why rejected |
|---|---|
| `field summary: Summary`, the issue's sketch | Mermaid and DOT already write `field <name>: <facts>`, so the title would collide with that separator. |
| The full heading, `## summary: Summary` | Its depth depends on the declaration, so it is not a fact about the field, and the block already teaches the `## name: Title` form. |
| A double-quoted title, as `check` writes it | Markdown inside quotes still renders in the block, and a `"` in the title would need a rule of its own. |

## release-note: Release note

- [§FS-schema.4.1](../../functional-spec/FS-schema.md#41-fields-are-lines-inside-their-kinds-node), [§FS-init.2.3.5.12.2](../../functional-spec/FS-init.md#235122-declaration-shapes): **`grund schema` and the managed block name a field's configured title.** A field that sets `title` now ends its line with ``· titled `<title>` `` in `grund schema`'s table, its Mermaid and its DOT label lines, and in the `### Declaration shapes` bullets `grund init` writes, so an agent writes the heading `grund check` holds it to before `check` runs. A field with no title prints the same line as before. A script that matches a titled field's whole line in `grund schema` output sees one fact appended; `grund config show` remains the machine-readable reader. The block's change shares v43/v44 and v45/v46, so `grund check` reports a block 0.20.0 wrote as outdated, and a block an unreleased build wrote at v45 or v46 for a config with a titled field as stale in `declaration shapes`. **What an upgrader does:** run `grund init` once to re-render the block; unmanaged content is kept. No `grund check` verdict changes. (agent-grounds/grund#1070)
