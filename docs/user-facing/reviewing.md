# Reviewing code

Before changing or removing a declaration, see what leans on it:

```bash
$ grund refs FS-check.3.2 --summary
crates/grund-cli/tests/index_entry_round_trip.rs: 1 (line 232)
crates/grund-core/src/checker/index.rs: 2 (lines 204, 354)
crates/grund-core/src/checker/judge.rs: 1 (line 89)
crates/grund-core/src/checker/references.rs: 2 (lines 2, 333)
crates/grund-core/src/checker/report.rs: 1 (line 185)
crates/grund-core/src/checker/tests_local_section_citations.rs: 3 (lines 2, 241, 356)
crates/grund-core/src/model/codes.rs: 1 (line 105)
crates/grund-core/src/model/remedies/check.rs: 1 (line 352)
crates/grund-core/src/model/stub_resolution.rs: 1 (line 176)
docs/decisions/functional/DF-duplicate-section-path.md: 1 (line 27)
docs/decisions/functional/DF-local-section-absent-target-escape.md: 1 (line 13)
docs/decisions/functional/DF-require-grounding.md: 1 (line 9)
docs/functional-spec/FS-check.md: 8 (lines 500, 508, 938, 950, 1059, 1171, 1223, 1269)
docs/functional-spec/FS-citations.md: 3 (lines 151, 198, 402)
docs/functional-spec/FS-declarations.md: 1 (line 1006)
docs/functional-spec/FS-errors.md: 1 (line 760)
docs/functional-spec/FS-fmt.md: 3 (lines 205, 210, 539)
docs/functional-spec/FS-languages.md: 1 (line 94)
docs/functional-spec/FS-rules.md: 1 (line 795)
docs/functional-spec/FS-section-lock.md: 7 (lines 155, 175, 177, 189, 191, 195, 211)
docs/related-work/REL-doc-reference-checkers.md: 1 (line 147)
docs/related-work/REL-documentation-decay.md: 1 (line 75)
docs/related-work/REL-traceability-tools.md: 1 (line 46)
docs/requirements/REQ-no-wrong-citation.md: 1 (line 7)
docs/roadmap.md: 1 (line 287)
examples/quickstart/README.md: 1 (line 6)
examples/rust-rfc/README.md: 1 (line 351)
examples/spec-kit/README.md: 1 (line 311)
```

A section's children break when it moves, so before a move, rename or delete ask the same question of the whole subtree: `grund refs FS-check.3.2 --descendants --summary` folds that section *and every section beneath it* into the same one-line-per-file shape, which is the blast radius of the change rather than of the coordinate. Swap `--summary` for `--total` and the same subtree folds one rung further, to the pair `cited at <n> sites across <m> files` — the size of the move, without the rows ([§FS-refs.3.4](../functional-spec/FS-refs.md#34---total)).

After an authored declaration rename or file move, use the
[maintenance workflow](structure.md#maintaining-declarations): edit affected
authored citation IDs, run `grund fmt --write` from the config/project or workspace
root, then verify with `grund check` and `grund fmt --check`. Review and commit
the generated link and eligible section-lock changes with the authored edits
([§FS-init.2.3.4.9](../functional-spec/FS-init.md#2349-declaration-blast-radius)).

Before reviewing a diff, group the citation graph by file so you can join changed files to the specs they touch. Name the changed file and a pattern, and `cover` keeps only the citations the pattern matches ([§FS-cover.globs](../functional-spec/FS-cover.md#globs---glob-keeps-the-citations-a-pattern-matches)):

```bash
$ grund cover crates/grund-core/src/checker/references.rs --glob 'FS-check.3.2' --format json
{"path":"crates/grund-core/src/checker/references.rs","citations":[{"path":"crates/grund-core/src/checker/references.rs","line":2,"column":23,"id":"FS-check","section":"3.2","marker":true,"text":"§FS-check.3.2","enclosing_declaration":null,"enclosing_section":null},{"path":"crates/grund-core/src/checker/references.rs","line":255,"column":12,"id":"FS-check","section":"3.2","marker":true,"text":"§FS-check.3.2","enclosing_declaration":null,"enclosing_section":null}]}
```

`id` and `section` name what each site points *at*; `enclosing_declaration` and `enclosing_section` name the unit it sits *in*, so the record carries both ends of the `cites` edge and a report over those relations needs no second read of the tree. Both are `null` above because this file declares no ID of its own — in a spec file they read `"enclosing_declaration":"FS-cover","enclosing_section":"terms"` ([§FS-cover.3.2](../functional-spec/FS-cover.md#32---format-json)).

[Querying grund](querying.md) sets these beside the other structural queries.

**Which spec points does this hunk edit?** A citation's `enclosing_section` answers for the lines that carry a citation. A hunk also edits headings, blank lines and uncited sentences, so ask `cover` about the hunk's own line numbers — from the new side of the diff, because they are read against the tree on disk ([§FS-cover.6](../functional-spec/FS-cover.md#6-line-ownership)):

```console
$ grund cover docs/functional-spec/FS-config.md --lines 115-124
docs/functional-spec/FS-config.md:115-124
  115  FS-config.requirements.5
  116-119  FS-config.requirements.6
  120-123  FS-config.requirements.7
  124  FS-config.requirements.8
```

Each row is a run of lines and the unit that owns it, by the rules the scan itself uses, so named sections, fenced headings and the end of a doc-comment come out as `check` sees them. Pass one `--lines` per hunk to answer a whole file in one scan, and `--format json` for the records of [§FS-output-shapes.5.3](../functional-spec/FS-output-shapes.md#53-cover---lines---formatjson). This is ownership, not coverage: it says which unit a line lies in, never that the line is grounded.

Lines removed by a hunk belong to the base tree: run
`grund cover <file> --lines <old range>` in a checkout of the base, for example
a `git worktree add` of it. A hunk with no new-side lines has nothing to ask
the head checkout about.

For an agent reviewing a code change, the loop is mechanical: list the `§…` citations in the changed files, run `grund <ID>` on each, and ask "does the code still match what the spec claims?"

A changed line in `.grund/sections/<ID>.lock` (`-<address> <old>` / `+<address> <new>`) means the text at that address changed. For each changed address, run `grund refs <address>` and reread every citer against `grund <address>`, including citers outside the diff that may now read different text.
