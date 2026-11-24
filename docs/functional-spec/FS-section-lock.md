# FS-section-lock: a cited relative or mixed section keeps its text, or check says where the text went

A relative heading takes its address from its parent ([§FS-declarations.section.relative](FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds)), and a mixed path puts a name under the number of a section above it ([§FS-config.3.3.1](FS-config.md#331-named-components)), so renumbering that section moves it; a complete-path heading with no name under a number carries its address written on it, and the address moves with it. So a relative or mixed address can come to hold another section's text while `grund check` stays green: swap the `### .1` blocks under `## format:` and `## parse:`, and every citation of `FS-login.format.1` reads the other section. The section lock is what catches it: a committed file that records, for every locked address the tree cites, a fingerprint of the text it held, so `check` can tell an edit in place from text that left for another address ([§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution), [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration)). Decided in [§DF-relative-section-headings.lock](../decisions/functional/DF-relative-section-headings.md#lock-a-cited-relative-address-is-locked-to-its-text), and for a mixed path in [§DF-number-name-sections.decision.lock](../decisions/functional/DF-number-name-sections.md#decisionlock-the-lock-is-what-answers-178).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, body, section, coordinate, relative heading, canonical path, section lock), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (citation), and [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, config root, workspace, member, alias).

- **locked address** — A coordinate some citation in the project resolves to, whose section is a relative heading or whose canonical path is a mixed path ([§FS-config.3.3.1](FS-config.md#331-named-components)), whichever spelling declared it.
- **entry** — One line of a lock file: a locked address and its fingerprint.
- **fingerprint** — Sixteen lowercase hex digits that stand for the text a section held ([§FS-section-lock.holds.fingerprint](FS-section-lock.md#holdsfingerprint-the-fingerprint)).
- **recorded address** — The coordinate an entry names.

## holds: What the lock holds

### holds.entry: One line per locked address

A lock file holds one entry per locked address of one declaration, `<coordinate> <fingerprint>`, one space between them, one entry per line, sorted bytewise by coordinate, each line ending in LF, and nothing else: no title, no line number, no history.

```text
FS-login.format.1 b2405f19c414fb23
FS-login.parse.2 8f332b6fe87bb346
```

An address is enrolled exactly when its heading is relative or its canonical path has a numeric component followed, at any later depth, by a named one, a test of the path's parsed components and never of digits or letters in the ID. So `FS-x.1.goals`, `FS-x.1.goals.3` and `FS-x.goals.3.latency` are entries once cited, written `### 1.goals: Goals` or `### .goals: Goals` alike, and so is a relative `### .3:` under `## goals:`; a complete-path `FS-x.goals.3`, `FS-x.1.2` or `FS-x.format.goals` is not an entry, cited or not, and neither is an address nothing cites. A locked address in a stub's target is entered like one in a scanned file, even where the target lies outside `[scan] include` ([§FS-declarations.stubs.target.5](FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)).

### holds.fingerprint: The fingerprint

The fingerprint is the first 16 hex digits, lowercase, of the SHA-256 of the section's text: what `grund <coordinate> --full` prints in text format below the heading line, which is the section and everything nested in it. Before hashing, line endings are made LF, trailing spaces and tabs are removed from each line, leading and trailing empty lines are dropped, and the lines are joined with LF with no final LF; a section with no text hashes the empty string. Because the text is `show`'s, cross-reference links are already flattened ([§FS-show.3.2](FS-show.md#32-cross-reference-links-are-flattened-in-text-and-json)), so wrapping a citation or re-deriving its anchor leaves the fingerprint as it was, and two installs agree on it ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)).

Also before hashing, every line the declaration's section map records as the heading of a section nested in the fingerprinted one is written with its path relative to that section: the `#` run, the punctuation after the path and the title stay as written, and the path becomes `.` followed by the components it has below the fingerprinted section. Take `FS-x.1.1`, a `### .1. Goals` under `## 1. Overview`: a `#### 1.1.1. Latency` in it hashes as `#### .1. Latency`, exactly as a `#### .1. Latency` does, and a `##### .1. P95` below either hashes as `##### .1.1. P95`. A line is matched to the map by its position, never by its shape, so a heading-shaped line in a fence or in prose hashes as written. Renumbering an ancestor therefore leaves the fingerprint as it was, and the move is reported whichever way the nested headings are written ([§FS-section-lock.checks.section-address-changed](FS-section-lock.md#checkssection-address-changed-a-cited-sections-text-sits-at-another-address)). Decided in [§DF-section-lock-relative-fingerprint](../decisions/functional/DF-section-lock-relative-fingerprint.md#df-section-lock-relative-fingerprint-a-locked-sections-nested-headings-are-fingerprinted-relative-to-it-so-a-renumber-is-still-caught).

### holds.repeated: A text the declaration repeats

When another section of the same declaration has the same fingerprint, the entry is written with `-` in place of the fingerprint (`FS-login.format.1 -`) until the texts differ, when `fmt --write` records the fingerprint.

## file: Where the lock lives

### file.shard: One file per cited declaration

The entries for the locked addresses of declaration `<ID>` live in `.grund/sections/<ID>.lock` under the config root of the project whose files hold the citations: one sorted file per declaration, so the lock is no file every change edits ([§DF-relative-section-headings.lock](../decisions/functional/DF-relative-section-headings.md#lock-a-cited-relative-address-is-locked-to-its-text)). The file is named by the ID, not the declaring file, so moving or renaming a spec file leaves its lock alone. A file with no entry does not exist, and neither does an empty `.grund/sections/`.

### file.members: Citations into another project

A citation into another workspace member or a remote, `§<alias>/<ID>.<path>`, is entered in `.grund/sections/<alias>/<ID>.lock` of the citing project, its coordinate `<ID>` and its path as the target project spells them, without the alias, and is judged against the target's sections and texts like any other entry. A finding about it names the coordinate with the alias, `api/FS-login.format.1`, and [§FS-section-lock.checks.section-address-changed](FS-section-lock.md#checkssection-address-changed-a-cited-sections-text-sits-at-another-address) sits at the target's heading.

A run that does not load the alias — a member's own run, where a sibling's alias is unknown ([§FS-workspace.5.1](FS-workspace.md#51-a-member-run)), or any run once no project has that alias — judges no entry in `.grund/sections/<alias>/`, so `check` reports nothing from it and `fmt` adds or refreshes none of its entries; for a citation into a sibling member only the workspace root's run writes them ([§FS-scan.projects.workspace-root](FS-scan.md#projectsworkspace-root-a-run-at-the-workspace-root-loads-every-project)). Such a run leaves the directory alone while a citation in the project still names the alias; once none does, `fmt --write` drops its entries as it drops any uncited entry ([§DF-section-lock-unnamed-alias](../decisions/functional/DF-section-lock-unnamed-alias.md#df-section-lock-unnamed-alias-fmt-drops-a-lock-shard-of-an-alias-nothing-names-any-more)).

## write: Who writes the lock

### write.rules: Add, refresh, drop

Only `grund fmt --write` writes lock files, and only when its scope is a project's config root or the workspace root; `check`, the editor and every other command write none. Given a file, or a directory that is neither a project's config root nor the workspace root, `fmt` leaves every lock alone and says nothing of it, since telling that an entry is pending takes the whole declaration set. A run brings each lock file to what the tree needs as the run leaves it, not as it read it:

- **add** an entry for each locked address that has none;
- **refresh** an entry whose section was edited in place — its recorded fingerprint is found at no address of the declaration, so its text changed where it stands — to the section's current fingerprint;
- **drop** an entry whose address nothing cites any more, or whose address is no longer a locked address.

A citation the run itself writes, marking a trigger ([§FS-fmt.2.1](FS-fmt.md#21-trigger-to-marker)) or expanding a shorthand or a declaration-local token ([§FS-fmt.2.4](FS-fmt.md#24-shorthand-to-canonical)), gets its entry in that run, and a locked section whose text the run rewrites, in this project or, from the workspace root, in another, is fingerprinted as rewritten, so `fmt --check` passes straight after a `--write` ([§FS-fmt.requirements.fixed-point](FS-fmt.md#requirementsfixed-point-a-tree-fmt-has-written-is-a-fixed-point)). A stale entry is never a `check` finding, only a row `fmt --write` drops ([§DF-relative-section-headings.lock](../decisions/functional/DF-relative-section-headings.md#lock-a-cited-relative-address-is-locked-to-its-text)). An unreadable path or a conflict block in a declaration body ([§FS-declarations.body.conflict](FS-declarations.md#bodyconflict-a-body-still-in-the-middle-of-a-merge)) refuses the pass, and the run writes no lock and no other file ([§FS-fmt.3.2](FS-fmt.md#32-a-run-that-needs-the-whole-declaration-set-rewrites-nothing)).

### write.never-a-move: `fmt` never approves a move

`fmt` never refreshes an entry whose recorded fingerprint now sits at another address of the declaration, because that is text that moved, and recording the new text at the old address would silence [§FS-section-lock.checks.section-address-changed](FS-section-lock.md#checkssection-address-changed-a-cited-sections-text-sits-at-another-address) ([§DF-number-name-sections.decision.lock](../decisions/functional/DF-number-name-sections.md#decisionlock-the-lock-is-what-answers-178)). The one exception is a move already acknowledged by a pointer ([§FS-section-lock.move.pointer](FS-section-lock.md#movepointer-leave-a-pointer)), whose pointer text it records. A lock whose ID uses an equivalent numeric spelling ([§FS-config.3.2.8](FS-config.md#328-numeric-spelling-survives-reading-and-reporting)) still records the same declaration and coordinates, and the spelling alone changes neither the lock's spelling nor its recorded fingerprints, so it never sets a fresh baseline.

### write.report: What `fmt` reports

The dry run, `fmt --check`, lists one row per entry `--write` would add, refresh or drop, `<lock file>:<line>: section lock: <add|refresh|drop> <coordinate>` in the [§FS-fmt.3.5](FS-fmt.md#35-the-dry-run-report) shape, after every source row, ordered by path, then line, then the row's text. An add is reported at the line the entry will occupy after the write, a refresh or drop at the line it occupies now, and a pending row makes `fmt --check` exit `1` ([§FS-fmt.3](FS-fmt.md#3-outputs)). `--write` lists each lock file it changed ([§FS-fmt.3.7](FS-fmt.md#37-the---write-report)), its count being the number of rows the dry run lists for it.

A write that creates a lock file, or deletes one by leaving no entry in it ([§FS-section-lock.file.shard](FS-section-lock.md#fileshard-one-file-per-cited-declaration)), says so: the file's first dry-run row ends `; creates the file` or `; deletes the file`, appended once the rows are in the order above and without moving any of them, and its `--write` line ends ` created` or ` deleted` after the count. A lock file that exists before the write and after it carries neither suffix. A rename that moves a declaration's entries to a lock file of the new name reads:

```text
$ grund fmt
docs/notes.md:1: markdown link: [§FS-sign-in.code.1](functional-spec/FS-login.md#1-a-code-expires-after-ten-minutes) → [§FS-sign-in.code.1](functional-spec/FS-sign-in.md#1-a-code-expires-after-ten-minutes)
.grund/sections/FS-login.lock:1: section lock: drop FS-login.code.1; deletes the file
.grund/sections/FS-sign-in.lock:1: section lock: add FS-sign-in.code.1; creates the file
$ grund fmt --write
rewrote 3 lines: 1 link written, 2 section-lock entries
  .grund/sections/FS-login.lock (1) deleted
  .grund/sections/FS-sign-in.lock (1) created
  docs/notes.md (1)
```

### write.rebuild: A lock grund cannot read is rebuilt

A lock line that is not `<coordinate> <fingerprint>`, such as a hand edit or the conflict markers of a merge, is a `check` error ([§FS-section-lock.checks.invalid-section-lock](FS-section-lock.md#checksinvalid-section-lock-a-lock-line-grund-cannot-read)), and `fmt --check` lists it as `section lock: rebuild` at its line. `fmt --write` rebuilds the file: it keeps every entry it can read, from either side of a conflict, takes for a coordinate entered twice the entry whose fingerprint the section holds now, or else the first, and then adds, refreshes and drops as [§FS-section-lock.write.rules](FS-section-lock.md#writerules-add-refresh-drop) says. Two changes that each cite a new section of one declaration therefore merge with one `fmt --write`.

## move: Moving a cited section on purpose

A cited section that moves changes its address, and [§REQ-spec-section-names.permanence](../requirements/REQ-spec-section-names.md#permanence-a-name-is-permanent) asks that the old address not silently become something else, so a move on purpose takes its citations along or leaves a pointer.

### move.citations: Update every citation

Change every citation of the old address to the new one and run `fmt --write`: the old entry, cited by nothing now, is dropped, and the new address is added. Where a cited section takes the old address, as in a swap or a shift of cited sections, the old address stays cited, so this does not clear the move: undo it or leave a pointer.

### move.pointer: Leave a pointer

Keep the old address cited and leave a pointer there: a section at the recorded address whose text cites the address where the recorded text now sits (`See <§>FS-login.parse.2.`). That acknowledges the move: `check` does not report it, and `fmt --write` records the pointer's text as the entry's new fingerprint. Retiring the pointer once nothing cites the old address is a change of its own, and it drops the entry.

### move.copy: Copy, then edit the original

Copying a cited section to a new address and then editing the original is the one case the lock reports though nothing moved: the recorded text is then found only at the copy, which is what a move looks like ([§DF-relative-section-headings.lock](../decisions/functional/DF-relative-section-headings.md#lock-a-cited-relative-address-is-locked-to-its-text)). The finding names the way out: edit the copy, so the original reads as edited in place, or cite the copy from the original, which acknowledges it as a pointer does.

## limits: What the lock does not catch

### limits.edit-and-move: A section moved and edited in one change

When a cited section moves and its text is edited in the same change, its recorded fingerprint is found nowhere, so the lock reads it as edited in place: `check` passes, and `fmt --write` refreshes the entry. Swap the `### .1` blocks under `## format:` and `## parse:` and edit both: `fmt --check` lists `section lock: refresh FS-login.format.1`, and after `--write` the citation of `FS-login.format.1` reads the other section's edited text. A move that also updates the section's own citations of its subsections is this limit too, though nothing but the move was meant, because citation text is hashed as written, never normalized as nested headings are ([§DF-section-lock-relative-fingerprint.rejected](../decisions/functional/DF-section-lock-relative-fingerprint.md#rejected-rejected-alternatives)): `check` reports only the updated citation, as [§FS-section-lock.checks.unrecorded-section-citation](FS-section-lock.md#checksunrecorded-section-citation-a-locked-address-the-lock-does-not-hold), and the `fmt --write` that clears it also refreshes the old entry to the text now at the old address. The only trace is the refreshed line in the lock diff `fmt --write` produces. The limit is accepted, and no second key, such as the title, narrows it ([§DF-relative-section-headings.lock](../decisions/functional/DF-relative-section-headings.md#lock-a-cited-relative-address-is-locked-to-its-text)).

### limits.identical: Identical texts

Sections with byte-identical text cannot be told apart, so an entry whose text the declaration repeats guards nothing until they differ ([§FS-section-lock.holds.repeated](FS-section-lock.md#holdsrepeated-a-text-the-declaration-repeats)).

### limits.across: Across declarations

The lock compares a section only with the other sections of its own declaration. A section moved into another declaration is a risk complete-path headings already carry, and it is not covered here.

### limits.baseline: A baseline the lock never saw

The lock can report a move only from a text it recorded before the move. Where it holds no such record, `fmt --write` records the text the address holds now, and `check` has nothing older to compare it with: on a citation added for the first time, on the first `fmt --write` after an upgrade that changed how a fingerprint is computed, on a lock file deleted and written again, and on an entry dropped because its citation was removed for a while ([§FS-section-lock.write.rules](FS-section-lock.md#writerules-add-refresh-drop)). The only trace is the added or changed line in the lock diff, which is why the lock is committed with the citations it records and reviewed with them. No check refuses a new baseline: telling a first record from a lost one needs history, and `check` reads none ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)).

### limits.gate: A lock nobody refreshes

An edit that `fmt --write` never recorded leaves the old fingerprint in place, which weakens the guard for that entry until the next `fmt --write`. A repository that runs `fmt --check` in its gate never carries one.

## checks: Checks

Each is a `grund check` error with its code in [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog), located, selected and rendered as [§FS-check.2.1](FS-check.md#21-report-format) says, a lock file its message names being a report path ([§FS-errors.3](FS-errors.md#3-message-text)). [§FS-section-lock.checks.vanished](FS-section-lock.md#checksvanished-a-cited-address-that-is-gone-and-where-its-text-went) has no code of its own: it is what the lock adds to the `missing-section` finding of [§FS-check.3.2](FS-check.md#32-missing-section).

### checks.section-address-changed: A cited section's text sits at another address

When an entry's recorded address is cited and is a locked address, the section there no longer has the recorded fingerprint, and another address of the same declaration does, the text that was cited has moved and the address now holds something else. The error is located at the heading that now holds the recorded address and names the address that holds the recorded text, the first in document order where several do:

```text
docs/functional-spec/FS-login.md:5: error: FS-login.format.1 is cited, and what .grund/sections/FS-login.lock recorded for it now sits at FS-login.parse.1; move it back, cite FS-login.parse.1 instead, or leave a pointer to it at FS-login.format.1; if FS-login.parse.1 is a copy, edit the copy or cite it from FS-login.format.1; FS-login.format.1 is cited at docs/notes.md:3, docs/notes.md:7, docs/use.md:3
```

After the filled-in *Fix* cell, the finding's own words name each site that cites the recorded address ([§FS-errors.5.5.1](FS-errors.md#551-how-a-row-is-read)): every one, with no cap, because each is an edit the remedy asks for. Two citations on one line are one site, sites are sorted by path, then line, and JSON `sites` carries the same `{ path, line }` pairs in the same order ([§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output)). The sites are read from the whole tree, as *is cited* is, so a run narrowed to the heading's file still names citations outside it, and a narrowed run reports the finding, whole, when its heading or any of its sites is in scope ([§FS-check.1.3.6](FS-check.md#136-an-explicit-path-still-narrows)). It is not reported when the section at the recorded address cites the address it names ([§FS-section-lock.move.pointer](FS-section-lock.md#movepointer-leave-a-pointer)), nor for an entry written `-` ([§FS-section-lock.holds.repeated](FS-section-lock.md#holdsrepeated-a-text-the-declaration-repeats)).

### checks.vanished: A cited address that is gone, and where its text went

A citation of an address that holds no section is [§FS-check.3.2](FS-check.md#32-missing-section)'s `missing-section` error, and the lock may still know where that section's text went. It does exactly when all three hold:

- the citation's coordinate is no section of its declaration;
- the lock of that declaration ([§FS-section-lock.file.shard](FS-section-lock.md#fileshard-one-file-per-cited-declaration)) holds an entry whose recorded address is exactly that coordinate, with a fingerprint and not `-`, the first where the coordinate is entered twice;
- another section of the same declaration has that fingerprint now.

Then the message gains one clause, in the words [§FS-section-lock.checks.section-address-changed](FS-section-lock.md#checkssection-address-changed-a-cited-sections-text-sits-at-another-address) uses for the same fact, naming the lock file and the section that holds the recorded text, the first in document order where several do; where the clause sits in each form of the message is [§FS-errors.3.11](FS-errors.md#311-the-missing-section-remedy)'s. Rename `### .borrow-a-book:` under `## requirements:` to `### .check-out-a-book:`, its text unchanged, and a citation of `SPEC-loans.requirements.borrow-a-book` reports:

```text
src/loans.py:5: error: section not found: SPEC-loans.requirements.borrow-a-book; write <§> before it to show the shape without citing it; what .grund/sections/SPEC-loans.lock recorded for it now sits at SPEC-loans.requirements.check-out-a-book
```

The finding stays one finding at the citation, and the lock and the declaration are read whatever the run is narrowed to, so a run narrowed to the citing file carries the clause. The clause reads the lock as it stands: a `fmt --write` run before the citations follow the move drops the entry, because its coordinate is no longer a locked address ([§FS-section-lock.write.rules](FS-section-lock.md#writerules-add-refresh-drop)), and every later `check` reports the citation without the clause, the dropped line in that run's lock diff the only trace.

### checks.unrecorded-section-citation: A locked address the lock does not hold

A citation that resolves to a locked address with no entry in the lock is an error at the citation's site, because without an entry the address is unguarded:

```text
docs/use.md:3: error: FS-login.format.1 is cited but has no entry in .grund/sections/FS-login.lock; run grund fmt --write with no path at the workspace root, or at the project root outside a workspace
```

The remedy names where to run the command, and with no path, because a run given a file, or a directory that is neither a project's config root nor the workspace root, writes no lock ([§FS-section-lock.write.rules](FS-section-lock.md#writerules-add-refresh-drop)) and still reports `rewrote 0 lines` ([§FS-fmt.3.7](FS-fmt.md#37-the---write-report)). Outside a workspace, a run with no path from anywhere in the project, or given its root, is the run that writes it ([§FS-fmt.7.1](FS-fmt.md#71-scope-equivalence)); in a workspace the remedy names the root, because a member's own run writes no entry for a citation into a sibling ([§FS-section-lock.file.members](FS-section-lock.md#filemembers-citations-into-another-project)).

### checks.invalid-section-lock: A lock line grund cannot read

A line of a lock file that is not an entry, as [§FS-section-lock.holds.entry](FS-section-lock.md#holdsentry-one-line-per-locked-address) defines one, is an error at that line. The entries `check` can read are still judged, and `fmt --write` rebuilds the file ([§FS-section-lock.write.rebuild](FS-section-lock.md#writerebuild-a-lock-grund-cannot-read-is-rebuilt)).

```text
.grund/sections/FS-login.lock:1: error: section lock line is not <coordinate> <fingerprint>; to rebuild the file, run grund fmt --write with no path at the workspace root, or at the project root outside a workspace
```
