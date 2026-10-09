# DF-stub-target-declared-twice: a stub's target that declares its ID twice is two homes, scanned or not

**Status:** Accepted
**Date:** 2026-10-09

## 1. Context

[§DF-stub-pairs-with-unscanned-target](DF-stub-pairs-with-unscanned-target.md#df-stub-pairs-with-unscanned-target-a-stub-pairs-with-its-target-whether-or-not-the-scan-reaches-it) paired a stub with its target whether or not the scan
reaches it, and decided that each unscanned tree answers as its scanned control does. The count
of homes it built read a stub's target only for an ID the walk recorded more than once, and kept
the first line of the target that declares the ID. So a lone stub to a target outside
`[scan] include` that declares the stub's ID twice was one home: `check` passed the tree with
`success`, while `show` and `refs` refused the ID as `ambiguous ID`, naming both declarations
([§FS-show.2.3.7](../../functional-spec/FS-show.md#237-a-stubs-target-is-found-by-its-id)), and the same tree with the target scanned reported `duplicate declaration`.
Where a scanned second declaration of the ID made the walk read the target, the error named the
target's first declaration and left its second out. The pull request behind that record had set
the twice-declared target aside for the `refs` fix, which reached only the query `show` and `refs`
share. Reported as [issue #556](https://github.com/agent-grounds/grund/issues/556).

## 2. Decision

The home a stub stands for is every declaration of its ID in its target that is not itself a
stub, each read as the scan would read it were the target scanned
([§FS-declarations.checks.duplicate.1](../../functional-spec/FS-declarations.md#checksduplicate1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)). A lone stub's target is read as any stub's is, and a target
that declares the ID twice is two homes, so the ID is a duplicate. Stubs to one such target stand
for both declarations once between them ([§FS-declarations.checks.duplicate.2](../../functional-spec/FS-declarations.md#checksduplicate2-stubs-to-one-target-are-one-home)), and the error
names each declaration at the target's own line ([§FS-declarations.checks.duplicate.3](../../functional-spec/FS-declarations.md#checksduplicate3-a-home-reached-through-stubs-is-named-at-its-target)). This
completes [§DF-stub-pairs-with-unscanned-target](DF-stub-pairs-with-unscanned-target.md#df-stub-pairs-with-unscanned-target-a-stub-pairs-with-its-target-whether-or-not-the-scan-reaches-it) rather than revising it: its decision already names
the answer, the scanned control's, and `show` already gives it.

## 3. Rejected alternative: read the target only where the walk recorded the ID twice

Leaving a lone stub unread spares one read per unscanned target and per scan. It also leaves
`check` passing a tree `show` and `refs` refuse, and lets the scan scope decide how many homes the
ID has, which [§FS-declarations.checks.duplicate.1](../../functional-spec/FS-declarations.md#checksduplicate1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned) rules out by name: narrowing `[scan] include` to
leave out a file a stub points at would hide a duplicate inside that file, with no warning.

## 4. Rejected alternative: count the target once however often it declares the ID

Taking the target's first declaration as its one home, as the old run did once it read the
target, is a ranking by file order: every citation of the ID resolves to one of two declarations
for no reason the author wrote down. [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) forbids exactly that, "duplicate
declarations are reported rather than ranked", and the scanned control reports both.

## 5. Consequences

The verdict moves in three ways, and no message template changes.

- **Pass to fail.** A plain `grund check` over a tree where a stub's target outside the scan
  declares the stub's ID twice now reports the `duplicate declaration` the scanned control
  reports, at the target's two lines. The editor's diagnostics report it from the target's unsaved
  text where it is open in one ([§FS-declarations.checks.broken-stub.1](../../functional-spec/FS-declarations.md#checksbroken-stub1-the-target-is-read-as-a-save-would-write-it)), and `list` tags the stub's
  row as a duplicate ([§FS-list.3.1.1](../../functional-spec/FS-list.md#311-row-notes)).

  ```console
  $ grund check     # docs/a.md stubs FS-a to ../notes/a.md, outside the scan; notes/a.md declares FS-a at lines 1 and 5
  notes/a.md:1: error: duplicate declaration of FS-a (also declared at notes/a.md:5)
  ```

  This exited 0 with `success` before, and exits 1 now.
- **Sites added.** A duplicate that already involved such a target, through a second declaration
  the walk reached, named only the target's first declaration. It now names each, in the message
  and in the JSON `sites` ([§FS-errors.5](../../functional-spec/FS-errors.md#5-json-format)), so
  `docs/b.md:1: error: duplicate declaration of FS-a (also declared at notes/a.md:1)` becomes
  `docs/b.md:1: error: duplicate declaration of FS-a (also declared at notes/a.md:1, notes/a.md:5)`.
- **Both ways, for a selected run.** A run that leaves the new duplicate out of its report while it
  reads a citation of such an ID: a path that holds the citation and none of the duplicate's sites
  ([§FS-check.1.3.6.1](../../functional-spec/FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution), [§FS-check.1.3.6.2](../../functional-spec/FS-check.md#1362-a-finding-that-spans-several-sites-is-in-scope-at-any-of-them)), `--ignore duplicate`, `--only` with a code a rule produces
  such as `citation-cardinality`, or a trial `--rule "<sentence>" --only-rule`. The citation now
  reads as ambiguous, as it does on the scanned control and to `show`, and a rule counts no edge to
  an ambiguous declaration ([§FS-rules.5.1](../../functional-spec/FS-rules.md#51-facts-and-identity)). So an `at least` count can newly fail, and a
  prohibition or an `at most N` or `exactly N` count can newly pass.

Five things do not move. A target that declares the ID once is one home, as before. A heading
inside a fence of a Markdown target, and a stub-shaped line in the target, declare nothing there,
as they declare nothing to the broken-stub rule ([§FS-declarations.checks.broken-stub.2](../../functional-spec/FS-declarations.md#checksbroken-stub2-a-heading-inside-a-fence-of-the-target-declares-nothing)). A broken
stub, and a stub that links to its own file, stay homes of their own. A section citation of such
an ID reads as it did: a target that declares the ID twice lends no section ([§FS-check.3.2.1](../../functional-spec/FS-check.md#321-a-stubs-sections-are-its-targets-scanned-or-not)), so
its `missing section` stays, beside the new duplicate. And how `fmt` links a citation of such an
ID is not this record's to decide.

## 6. The correction route

[§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) licenses the pass-to-fail move, and each of its five conditions
holds.

1. *Prior prohibition.* The old verdict violated [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution), which already applied when that verdict shipped: it is present in both released tags, v0.16.0 and v0.16.1.
   That section binds resolution to [§FS-declarations.checks.duplicate](../../functional-spec/FS-declarations.md#checksduplicate-duplicate-declaration) by name, "duplicate declarations are reported rather than ranked", and a duplicate there is "any two declarations that are not stubs", "whether in one file or several", in both tags too.
   In both tags the stub is accepted only because its target holds a declaration of its ID ([§FS-declarations.checks.broken-stub](../../functional-spec/FS-declarations.md#checksbroken-stub-broken-inline-spec-stub)), which that rule reads wherever the target lies; here the target holds two. The old run counted one home and resolved every citation of the ID to it: to the stub's own record, or, where the walk read the target since [§DF-stub-pairs-with-unscanned-target](DF-stub-pairs-with-unscanned-target.md#df-stub-pairs-with-unscanned-target-a-stub-pairs-with-its-target-whether-or-not-the-scan-reaches-it), to the first of its two lines. That is a ranking the specification forbids.
   The plain run's `success`, and each edge a selected run counted to that one home, are that one misreading.
   This reaches only the misreading. A target that declares the ID once keeps every verdict on both builds.
2. *Accepted proof.* This record. The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path does not fit, because there is no old form to keep beside a new one, only a wrong answer, and nothing a user could write selects it: the stub is right as written.
   The [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical migration has no command to point at, because only the author can say which of the two declarations is the ID's.
3. *Named release.* The release note below names the verdict move in both directions, and the runs it reaches by path and by selection.
4. *Actionable findings.* Every finding that replaces the old verdict is an existing one: `duplicate declaration`, located at the first of its sites and naming every other ([§FS-check.2.1](../../functional-spec/FS-check.md#21-report-format)), which says where the declarations to rename or remove are, or a rule's count at the unit it judges, naming the rule, the count found and the count required ([§FS-rules.7.3](../../functional-spec/FS-rules.md#73-outbound-citation-cardinality), [§FS-rules.7.4](../../functional-spec/FS-rules.md#74-inbound-citation-cardinality)).
5. *No new licence.* This is not a licence for anything else: no rule, finding or prohibition is new, the homes now counted are the declarations [§FS-declarations.checks.duplicate](../../functional-spec/FS-declarations.md#checksduplicate-duplicate-declaration) counts with the target scanned, and a target that declares the ID once is one home as before.

## release-note: Release note

- [§FS-declarations.checks.duplicate.1](../../functional-spec/FS-declarations.md#checksduplicate1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned), [§FS-declarations.checks.duplicate.3](../../functional-spec/FS-declarations.md#checksduplicate3-a-home-reached-through-stubs-is-named-at-its-target), [§DF-stub-target-declared-twice](DF-stub-target-declared-twice.md#df-stub-target-declared-twice-a-stubs-target-that-declares-its-id-twice-is-two-homes-scanned-or-not), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution): **a stub's target that declares its ID twice is two homes, whether or not `[scan] include` reaches it.** A plain `grund check` over a tree where a stub points outside the scan at a file that declares the stub's ID twice no longer passes: it reports the `duplicate declaration` the same tree with the target scanned reports, named at both of the target's lines, where `show` and `refs` already refused the ID as ambiguous; `list` tags the stub, and the editor reports the duplicate from the target's unsaved text. A duplicate that already involved such a target now names each of its declarations rather than the first. **Who this breaks:** the verdict moves from pass to fail for a plain `grund check` over such a tree, and both ways for a run that leaves the new duplicate out of its report while it reads a citation of the ID, by a path or by a selection: `--ignore duplicate`, `--only` with a code a rule produces such as `citation-cardinality`, or a trial `--rule "<sentence>" --only-rule`. Such a run now reads the citation as ambiguous and counts no edge to it, so an `at least` count can fail where the run exited 0, and a prohibition or an `at most` or `exactly` count can pass. Every finding that replaces the old verdict is an existing error naming its location and the action to take. Closes [issue #556](https://github.com/agent-grounds/grund/issues/556).
