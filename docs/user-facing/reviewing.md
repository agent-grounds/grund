# Reviewing code

Before changing or removing a declaration, see what leans on it:

```bash
$ grund refs FS-check.3.2 --summary
crates/grund-cli/tests/index_entry_round_trip.rs: 1 (line 229)
crates/grund-core/src/checker/index.rs: 2 (lines 152, 258)
crates/grund-core/src/checker/references.rs: 2 (lines 2, 378)
crates/grund-core/src/checker/report.rs: 2 (lines 85, 486)
docs/decisions/functional/DF-duplicate-section-path.md: 1 (line 26)
docs/decisions/functional/DF-require-grounding.md: 1 (line 8)
docs/requirements/REQ-no-wrong-citation.md: 1 (line 7)
```

A section's children break when it moves, so before a move, rename or delete ask the same question of the whole subtree: `grund refs FS-check.3.2 --descendants --summary` folds that section *and every section beneath it* into the same one-line-per-file shape, which is the blast radius of the change rather than of the coordinate. Swap `--summary` for `--total` and the same subtree folds one rung further, to the pair `cited at <n> sites across <m> files` — the size of the move, without the rows ([§FS-refs.3.4](../functional-spec/FS-refs.md#34---total)).

Before reviewing a diff, group the citation graph by file so you can join changed files to the specs they touch:

```bash
$ grund cover --format json | jq -c 'select(.path == "crates/grund-core/src/checker/references.rs") | .citations |= map(select(.id == "FS-check" and .section == "3.2"))'
{"path":"crates/grund-core/src/checker/references.rs","citations":[{"path":"crates/grund-core/src/checker/references.rs","line":2,"column":23,"id":"FS-check","section":"3.2","marker":true,"text":"§FS-check.3.2","enclosing_declaration":null,"enclosing_section":null},{"path":"crates/grund-core/src/checker/references.rs","line":255,"column":12,"id":"FS-check","section":"3.2","marker":true,"text":"§FS-check.3.2","enclosing_declaration":null,"enclosing_section":null}]}
```

`id` and `section` name what each site points *at*; `enclosing_declaration` and `enclosing_section` name the unit it sits *in*, so the record carries both ends of the `cites` edge and a report over those relations needs no second read of the tree. Both are `null` above because this file declares no ID of its own — in a spec file they read `"enclosing_declaration":"FS-cover","enclosing_section":"terms"` ([§FS-cover.3.2](../functional-spec/FS-cover.md#32---format-json)).

For an agent reviewing a code change, the loop is mechanical: list the `§…` citations in the changed files, run `grund <ID>` on each, and ask "does the code still match what the spec claims?"
