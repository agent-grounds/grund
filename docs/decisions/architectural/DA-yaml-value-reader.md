# DA-yaml-value-reader: one YAML event parser builds one span tree, and value rules are a pass over it

**Date:** 2026-10-27

## status: Accepted

## 1. Context

[§FS-values.2.2.3](../../functional-spec/FS-values.md#223-how-a-yaml-source-is-read) makes YAML the second format of a value source, and [§FS-values.2.2.2](../../functional-spec/FS-values.md#222-reader-fidelity) asks the same fidelity of it as of JSON: member order, repeated keys, raw spellings and exact spans, kept before any lookup map is built. A YAML library that loads a document into values gives none of these. It keeps the last of two equal keys, resolves `1.10` to a float and `yes` to a boolean, expands an alias into a copy, and forgets where each scalar began.

Front matter is the other YAML grund will read ([issue #1048](https://github.com/agent-grounds/grund/issues/1048)), and it needs what value sources refuse: nested mappings, block scalars, scalars that span lines. Two readers of one language would drift apart on the first case one of them handles and the other does not ([§AR-scanner.2.1.3](../../architecture/AR-scanner.md#213-value-declarations)).

## 2. Decision

### 2.1 One event parser

`crates/grund-core/src/scanner/yaml.rs` drives the event parser of `yaml-rust2` 0.13 (MIT or Apache-2.0, pure Rust). Its events carry each node's style, anchor, tag and alias as written, which is what the refusals of [§FS-values.2.2.3](../../functional-spec/FS-values.md#223-how-a-yaml-source-is-read) are decided from, and no type resolution is applied to them.

### 2.2 One span tree, with no value rules in it

From the events and the source text the module builds a tree of documents, mappings, sequences, scalars and aliases. Each node keeps its style, anchor, tag and alias as the events give them, a start byte, and an end byte. Mapping keys keep their order, a repeated key included, and the tree keeps document boundaries. The tree accepts everything the parser accepts but one thing; it refuses nothing a front-matter reader would need. The one thing is depth: building the tree recurses once per nested collection, so it refuses a collection nested more than 128 deep with a `YamlError` at that collection's line, `collections are nested more than 128 deep`, before it recurses into it. That bound is the tree's, so a value source and a front-matter block meet the same one ([§FS-values.5.3](../../functional-spec/FS-values.md#53-incomplete-input-and-deterministic-output), [§FS-fields.front-matter.reading](../../functional-spec/FS-fields.md#front-matterreading-reading-the-block)), and no input runs the reader out of stack.

The value subset of [§FS-values.2.2.3](../../functional-spec/FS-values.md#223-how-a-yaml-source-is-read) is a separate pass over that tree, in the value pass (`scanner/value_json.rs`), which picks the reader by a source's format. Both readers feed the one enrollment path (`value_json_enrollment.rs`), so duplicates, ownership and component validity are written once for both formats ([§FS-values.2.2.1](../../functional-spec/FS-values.md#221-source-shape), [§FS-values.2.3](../../functional-spec/FS-values.md#23-duplicates-and-ownership)).

### 2.3 Where a node starts

The parser's positions are not byte offsets, and the tree never takes one as such. In 0.13, `Marker::index()` counts characters, one per character skipped, except over a block scalar's content lines, where it adds the line's byte length; after a non-ASCII comment it falls short of the byte offset, and after a non-ASCII block scalar it matches neither count. `Marker::col()` counts characters from 0. So:

- A node's start byte is read from `line()` and `col()`: the line's start in the source, plus the byte length of its first `col()` characters. `index()` is never used, and a parser message's "at byte N" is never passed on as a byte.
- A mapping starts at its first key. `MappingStart`, and an implicit `DocumentStart`, are positioned at that key's `:`.
- An empty block element's `Scalar("")` is positioned on the following line, so the element is located from its `-`.
- The parser keeps a leading byte order mark as part of the first key, so the mark is stripped before parsing and the positions are shifted past it; it belongs to no node.

### 2.4 Where a node ends

A single-line scalar's end is found by rescanning the source from its start: a plain scalar ends after its own text, a quoted one at its closing quote. Any other node ends where the next event starts, trimmed back over whitespace and comments. A line ends at LF or CRLF, and no end falls on a carriage return.

## 3. Rejected alternatives

### 3.1 A hand-written reader of the value subset

It is the smaller reader for this ticket alone. #1048 would then have to grow it into a full YAML parser, or add a second one beside it, and the second reader is what [§DA-yaml-value-reader.2.2](DA-yaml-value-reader.md#22-one-span-tree-with-no-value-rules-in-it) exists to prevent.

### 3.2 A loading library such as `serde_yaml`

It answers with values, not sites: no repeated key, no raw spelling, no span, and YAML's own type resolution already applied. Each of those is something [§FS-values.2.2.2](../../functional-spec/FS-values.md#222-reader-fidelity) forbids losing.

### 3.3 Trusting `Marker::index()`

It reads as a byte offset and agrees with one on ASCII input, which is what every first test is written in. On the first non-ASCII comment above a value it puts every later span a few bytes early, so `show` would print the wrong slice and a finding would name the wrong column.

## 4. Cost

One dependency, which adds `arraydeque` and `hashlink` to the tree; `hashbrown` is already in it. A tree module of about three hundred lines, held by unit tests that put a non-ASCII comment, a non-ASCII quoted element and a non-ASCII block scalar before a node, and that keep a multi-line block scalar's span, the shape #1048 reads.
