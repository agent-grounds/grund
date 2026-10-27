# DA-toml-front-matter-reader: one TOML event parser builds the span tree a TOML front-matter block is read through

**Date:** 2026-10-27

## status: Accepted

## 1. Context

[§FS-fields.front-matter.reading](../../functional-spec/FS-fields.md#front-matterreading-reading-the-block) reads a `+++` block in a kind that declares the `front-matter` field as TOML, with the same keys, coordinates and findings a `---` block gets ([issue #1057](https://github.com/agent-grounds/grund/issues/1057)). That needs from a TOML reader what [§DA-yaml-value-reader](DA-yaml-value-reader.md#da-yaml-value-reader-one-yaml-event-parser-builds-one-span-tree-and-value-rules-are-a-pass-over-it) needed from a YAML one: each top-level key's span, its raw scalar text, and a key written twice kept rather than refused or overwritten, because [§FS-fields.front-matter.keys](../../functional-spec/FS-fields.md#front-matterkeys-its-keys-are-its-sections) reports it as `duplicate-section`. A TOML library that loads a document into values gives none of these.

grund already reads one TOML file, `grund.toml`, through a walk of its own (`config/v2/walk.rs`, `config/v1/parse.rs`). It reads grund's closed config dialect, its refusals are worded contracts ([§FS-config-v2.reader](../../functional-spec/FS-config-v2.md#reader-the-reader)), and it keeps no spans.

## 2. Decision

### 2.1 One event parser, at TOML 1.1

`crates/grund-core/src/scanner/toml.rs` drives the event parser of `toml_parser` 1.1 (MIT OR Apache-2.0, pure Rust), the layer `toml_edit` and `toml` are built on. It is TOML's counterpart of the `yaml-rust2` event parser [§DA-yaml-value-reader.2.1](DA-yaml-value-reader.md#21-one-event-parser) chose: it emits table headers, keys, key separators, scalars and comments, each with a byte span, and decodes strings and their escapes. It validates the order of events and not what they mean, so a redefined key reaches the tree as an event rather than an error, and the tree decides it.

The 1.1 series reads TOML 1.1, which admits newlines and a trailing comma inside an inline table that a TOML 1.0 reader refuses. The dependency is pinned `~1.1`, so moving to a later TOML specification is a decision of its own rather than a `cargo update`.

### 2.2 A span tree that folds contributions into top-level keys

From the events and the source text the module builds the list of the block's top-level keys that the YAML branch builds: each key's decoded name, its line, its slice, and its one-line value where it has one. Everything after it reads that list unchanged, the section map, `show`, `cover`, rule subjects and every check, which is why the findings of a TOML block are those of a YAML one rather than merely similar ([§FS-fields.front-matter.coordinates](../../functional-spec/FS-fields.md#front-mattercoordinates-its-coordinates)).

The tree folds every contribution, a key-value, a dotted key, a `[table]` or `[[array]]` header, into the top-level key its first component names. It keeps, for each key, what its contributions have made it so far: a value, a table of dotted keys, a table its own header defines, a table only a header below it has opened, or an array of tables. It judges each later contribution against that as TOML does, and records a refused one as a second site of the key instead of failing ([§FS-fields.front-matter.keys](../../functional-spec/FS-fields.md#front-matterkeys-its-keys-are-its-sections)). It models nothing below the top level, so a conflict there is part of a value and is not judged.

A string's value is the decoder's output. Every other scalar's is the source text its span covers, because the decoder normalizes a number's spelling, `1_000` to `1000` and `0x1F` to `1F`, and [§FS-fields.front-matter.values](../../functional-spec/FS-fields.md#front-mattervalues-a-keys-value-is-its-decoded-text) keeps the spelling.

### 2.3 Spans are bytes

`toml_parser`'s spans are byte offsets into the text it is given, and the tree is given the block's content alone, so every span is shifted by the content's start in the file before it is reported. No offset is counted in characters, so a non-ASCII comment or string above a key moves no later span, the trap [§DA-yaml-value-reader.3.3](DA-yaml-value-reader.md#33-trusting-markerindex) records for YAML.

### 2.4 One nesting bound, the YAML tree's

The parser recurses once per nested array or inline table. It is wrapped in its `RecursionGuard`, built from the YAML tree's `MAX_NESTING` rather than from a second number. The guard counts arrays and inline tables and the YAML tree counts the root mapping too, so the guard is set one below the bound, and a TOML value is refused exactly where the same value written in YAML's flow style is ([§FS-fields.front-matter.reading](../../functional-spec/FS-fields.md#front-matterreading-reading-the-block), [§REQ-never-crashes.1](../../requirements/REQ-never-crashes.md#1-every-failure-is-a-diagnostic)). The guard's own message, `cannot recurse further; max recursion depth met`, is replaced by the shared reason, so [§FS-fields.checks.unreadable-front-matter](../../functional-spec/FS-fields.md#checksunreadable-front-matter-unreadable-front-matter) keeps one list of reasons.

## 3. Rejected alternatives

### 3.1 `toml_edit`

It brings six crates, two `winnow` versions among them, and builds about five times as long. It refuses a key written twice as a parse error, so a duplicate would be `unreadable-front-matter` rather than the `duplicate-section` a YAML block gets.

### 3.2 A hand-written subset reader

It is the second hand-written reader of one language that [§DA-yaml-value-reader.3.1](DA-yaml-value-reader.md#31-a-hand-written-reader-of-the-value-subset) argues against, and the first page that writes a construct it skipped would read differently from the same page in YAML.

### 3.3 Moving the `grund.toml` walk onto the new tree in this change

The walk reads grund's own closed dialect, and every refusal it gives is a worded contract. Moving it would re-prove every config error, which is a change of its own, not part of reading front matter.

## 4. One fact, one implementation

[§AR-system.principle](../../architecture/README.md#principle-one-fact-one-point-one-implementation) asks for one implementation of each fact. The two readers answer different facts: what `grund.toml` configures, and which keys a page's front matter has. The new tree is the one general TOML reader: a later general TOML need, such as a TOML value source, goes through it. Moving the config walk onto it is a possible later step, and this decision does not promise it.

## 5. Cost

Two crates join the tree, `toml_parser` 1.1.5 (MIT OR Apache-2.0) and `winnow` 1.0.4 (MIT), and neither was in it. A serial release build of the two took about 0.9 s on the machine the proposal was measured on. The parse runs only for a file whose first declaration's kind declares the field, so a repository that does not opt in pays nothing at run time. A tree module of about 250 lines, held by unit tests that fold dotted keys, headers and arrays of tables into one key at its first site, agree on quoted and bare names, keep spans after a non-ASCII comment and a non-ASCII string, keep raw spellings, end a slice before a trailing comment, carry the first parse error's file line, refuse nesting past the bound without overflowing, and read a CRLF block as its LF form.
