# FS-section-lock: a cited relative section keeps its text, or check says where the text went

A relative heading takes its address from its parent ([§FS-declarations.section.relative](FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds)), so a section can change address without a byte of its heading changing. Swap the `### .1` blocks under `## format:` and `## parse:` and every citation of `FS-login.format.1` silently reads the other section's text, while `grund check` stays green. A complete-path heading whose path holds no name under a number cannot do this, because its address is written on it and moves with it. A mixed path ([§FS-config.3.3.1](FS-config.md#331-named-components)) can, however it is written: renumber `## 1. Overview` to `## 2.` and insert a new `## 1.` with its own `1.goals`, and a citation of `FS-x.1.goals` reads the new chapter's goals. The section lock is what catches both: a committed file that records, for every locked address the tree cites, a fingerprint of the text it held, so `check` can tell an edit in place from text that left for another address ([§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution), [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration)). Decided in [§DF-relative-section-headings.lock](../decisions/functional/DF-relative-section-headings.md#lock-a-cited-relative-address-is-locked-to-its-text). A mixed address is enrolled since [§DF-number-name-sections.decision.lock](../decisions/functional/DF-number-name-sections.md#decisionlock-the-lock-is-what-answers-178).

The lock is part of the tree. `check` reads it and no history, so its verdict stays a function of the tree ([§FS-check.3.6.4](FS-check.md#364-a-pure-function-of-the-tree)); only `grund fmt --write` writes it ([§FS-section-lock.write](FS-section-lock.md#write-who-writes-the-lock)). A repository that cites no locked address has no lock and reads none, and a complete-path heading is enrolled only when its path is mixed ([§FS-section-lock.holds.entry](FS-section-lock.md#holdsentry-one-line-per-locked-address)).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, body, section, coordinate, relative heading, canonical path, section lock), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (citation), and [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, config root, workspace, member, alias).

- **locked address** — A coordinate some citation in the project resolves to, whose section is a relative heading or whose canonical path is a mixed path ([§FS-config.3.3.1](FS-config.md#331-named-components)), whichever spelling declared it.
- **entry** — One line of a lock file: a locked address and its fingerprint.
- **fingerprint** — Sixteen lowercase hex digits that stand for the text a section held ([§FS-section-lock.holds.fingerprint](FS-section-lock.md#holdsfingerprint-the-fingerprint)).
- **recorded address** — The coordinate an entry names.

## holds: What the lock holds

### holds.entry: One line per locked address

A lock file holds one entry per locked address of one declaration, `<coordinate> <fingerprint>`, one space between them, one entry per line, sorted bytewise by coordinate, each line ending in LF:

```text
FS-login.format.1 8b1a5e0c2f9d7a64
FS-login.parse.user 0d3c9e51b7f2a816
```

Nothing else is recorded: no title, no line number, no history. An address is enrolled exactly when its heading is relative or its canonical path has a numeric component followed, at any later depth, by a named one, a test of the path's parsed components and never of digits or letters in the ID. So `FS-x.1.goals`, `FS-x.1.goals.3`, `FS-x.1.goals.latency` and `FS-x.goals.3.latency` are entries once cited, written `### 1.goals: Goals` or `### .goals: Goals` alike, and so is a relative `### .3:` under `## goals:`; a complete-path `FS-x.goals.3`, `FS-x.1.2` or `FS-x.format.goals` is not an entry, cited or not, and neither is an address nothing cites. A locked address in a stub's target is entered like one in a scanned file, wherever the target lies: one outside `[scan] include` still lends the stub its sections ([§FS-declarations.stubs.target.5](FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)), so a citation of one is a locked address.

### holds.fingerprint: The fingerprint

The fingerprint is the first 16 hex digits, lowercase, of the SHA-256 of the section's text: what `grund <coordinate> --full` prints in text format below the heading line, which is the section and everything nested in it. Before hashing, line endings are made LF, trailing spaces and tabs are removed from each line, leading and trailing empty lines are dropped, and the lines are joined with LF with no final LF; a section with no text hashes the empty string. Because the text is `show`'s, cross-reference links are already flattened ([§FS-show.3.2](FS-show.md#32-cross-reference-links-are-flattened-in-text-and-json)): `fmt` re-deriving an anchor, a wrapped against a bare citation, a CRLF checkout and trailing whitespace all give one fingerprint, so two installs agree on it ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)). The digest is grund's one SHA-256, the algorithm [§FS-remote-projects.bytes.digest](FS-remote-projects.md#bytesdigest-the-digest-is-over-exact-bytes) names.

Before hashing, every line the declaration's section map records as the heading of a section nested in the fingerprinted one is written with its path relative to that section: the `#` run, the punctuation after the path and the title stay as written, and the path becomes `.` followed by the components it has below the fingerprinted section. Take `FS-x.1.1`, a `### .1. Goals` under `## 1. Overview`: a `#### 1.1.1. Latency` in it hashes as `#### .1. Latency`, exactly as a `#### .1. Latency` does, and a `##### .1. P95` below either hashes as `##### .1.1. P95`. Under `FS-x.1.goals`, `#### 1.goals.latency: Latency` and `#### .latency: Latency` both hash as `#### .latency: Latency`, and a `##### .p95: P95` below either as `##### .latency.p95: P95`. A line is matched to the map by its position, never by its shape, so a heading-shaped line in a fence or in prose hashes as written. The fingerprint is therefore what the section says and not where its parent stands: renumbering an ancestor changes the spelling of every complete-path heading nested in the section, and without this a renumber alone would make the moved section's text read as edited in place ([§FS-section-lock.limits.edit-and-move](FS-section-lock.md#limitsedit-and-move-a-section-moved-and-edited-in-one-change)), so `check` would pass and `fmt --write` would record the replacement at the old address. With it the move is reported ([§FS-section-lock.checks.section-address-changed](FS-section-lock.md#checkssection-address-changed-a-cited-sections-text-sits-at-another-address)) whichever way the nested headings are written. What `show` prints, and what the file says, do not change. Decided in [§DF-section-lock-relative-fingerprint](../decisions/functional/DF-section-lock-relative-fingerprint.md#df-section-lock-relative-fingerprint-a-locked-sections-nested-headings-are-fingerprinted-relative-to-it-so-a-renumber-is-still-caught).

### holds.repeated: A text the declaration repeats

When another section of the same declaration has the same fingerprint, the two cannot be told apart, so the entry is written with `-` in place of the fingerprint (`FS-login.format.1 -`). Such an entry guards nothing — it is never a [§FS-section-lock.checks.section-address-changed](FS-section-lock.md#checkssection-address-changed-a-cited-sections-text-sits-at-another-address) finding — until the texts differ, when `fmt --write` records the fingerprint.

## file: Where the lock lives

### file.shard: One file per cited declaration

The entries for the locked addresses of declaration `<ID>` live in `.grund/sections/<ID>.lock` under the config root of the project whose files hold the citations. Two changes that cite sections of different declarations never touch the same file, and two changes that cite different sections of one declaration touch different lines of a sorted file, so the lock is no file every change edits. The file is named by the ID, not the declaring file, so moving or renaming a spec file leaves its lock alone. A file with no entry does not exist, and neither does an empty `.grund/sections/`. The scan never enters `.grund/` ([§FS-scan](FS-scan.md#fs-scan-what-a-path-argument-means-which-projects-a-run-loads-and-which-files-it-reads)), so a lock file is never read as a source.

### file.members: Citations into another project

Each project keeps the entries for the citations its own files hold. A citation into another workspace member or a remote, `§<alias>/<ID>.<path>`, is entered in `.grund/sections/<alias>/<ID>.lock` of the citing project, so a member's own run and a run from the workspace root agree on what it cites and where that is recorded.

Such an entry's coordinate is the one the citation names, `<ID>` and its path as the target project spells them, without the alias, and it is judged against the target's sections and texts like any other entry. A finding about it names the coordinate with the alias, `api/FS-login.format.1`, and [§FS-section-lock.checks.section-address-changed](FS-section-lock.md#checkssection-address-changed-a-cited-sections-text-sits-at-another-address) sits at the target's heading. A run that does not load the alias — a member's own run, where a sibling's alias is unknown ([§FS-workspace.5.1](FS-workspace.md#51-a-member-run)) — leaves `.grund/sections/<alias>/` alone: it reports nothing from it and writes nothing to it.

## write: Who writes the lock

### write.rules: Add, refresh, drop

Only `grund fmt --write` writes lock files, and only when its scope is a project's config root or the workspace root; `fmt` narrowed to a file or directory leaves every lock alone, which is why `check` names that scope beside the command ([§FS-section-lock.checks.remedy](FS-section-lock.md#checksremedy-the-run-that-writes-the-lock)), and neither `check`, the editor, nor any other command writes one. A run brings each lock file to what the tree it leaves needs ([§FS-section-lock.write.same-run](FS-section-lock.md#writesame-run-an-entry-records-the-tree-as-the-same-run-leaves-it)):

- **add** an entry for each locked address that has none;
- **refresh** an entry whose section was edited in place — its recorded fingerprint is found at no address of the declaration, so its text changed where it stands — to the section's current fingerprint;
- **drop** an entry whose address nothing cites any more, or whose address is no longer a locked address.

An entry that already matches is left alone, so a second run writes nothing. A stale entry is never a `check` finding: it is something `fmt --write` drops. A conflict block in a declaration body ([§FS-declarations.body.conflict](FS-declarations.md#bodyconflict-a-body-still-in-the-middle-of-a-merge)) refuses the pass, and the run writes no lock and no other file ([§FS-fmt.3.2](FS-fmt.md#32-a-run-that-needs-the-whole-declaration-set-rewrites-nothing)), because neither side of a conflict is a text an entry may record.

### write.same-run: An entry records the tree as the same run leaves it

A run that rewrites files and brings the locks along judges every entry against the tree as it leaves it, not as it read it. A citation the run writes — a trigger it marks ([§FS-fmt.2.1](FS-fmt.md#21-trigger-to-marker)), a shorthand or a declaration-local token it expands ([§FS-fmt.2.4](FS-fmt.md#24-shorthand-to-canonical)) — is a citation when the lock is judged, so the locked address it names is added in that run. A locked section whose text the run rewrites is fingerprinted as rewritten, so its entry is refreshed to that text, or left alone where it already records it. An entry for a citation into another project is judged the same way against that project's text as the same workspace-root run leaves it ([§FS-section-lock.file.members](FS-section-lock.md#filemembers-citations-into-another-project)). The dry run lists exactly those rows ([§FS-fmt.7.3](FS-fmt.md#73-preview-equivalence)). Otherwise the next run adds or refreshes the entry again, and `fmt --check` fails straight after a `--write`, on a lock file the change's author never touched ([§FS-fmt.requirements.fixed-point](FS-fmt.md#requirementsfixed-point-a-tree-fmt-has-written-is-a-fixed-point--realized)).

### write.never-a-move: `fmt` never approves a move

`fmt` never refreshes an entry whose recorded fingerprint now sits at another address of the declaration, because that is text that moved, and recording the new text at the old address would turn the report of [§FS-section-lock.checks.section-address-changed](FS-section-lock.md#checkssection-address-changed-a-cited-sections-text-sits-at-another-address) into silence. The one exception is a move already acknowledged by a pointer ([§FS-section-lock.move.pointer](FS-section-lock.md#movepointer-leave-a-pointer)), whose pointer text it records.

An existing lock whose ID uses an equivalent numeric spelling still records the
same declaration and coordinates ([§FS-config.3.2.8](FS-config.md#328-numeric-spelling-survives-reading-and-reporting)).
An equivalent spelling alone changes neither the lock's spelling nor its recorded
fingerprints; a report-spelling correction never establishes a fresh baseline for
its addresses.

### write.report: What `fmt` reports

The dry run, `fmt --check`, lists one row per entry `--write` would add, refresh or drop, in the [§FS-fmt.3.5](FS-fmt.md#35-the-dry-run-report) shape, after every source row, ordered by path, then line, then the row's text:

```text
.grund/sections/FS-login.lock:1: section lock: add FS-login.format.1
.grund/sections/FS-login.lock:2: section lock: refresh FS-login.parse.1
.grund/sections/FS-login.lock:3: section lock: drop FS-login.parse.2
```

An add is reported at the line the entry will occupy after the write, a refresh or drop at the line it occupies now. A pending row makes `fmt --check` exit `1` as any pending rewrite does ([§FS-fmt.3](FS-fmt.md#3-outputs)). `--write` lists each lock file it changed in its report like any other file ([§FS-fmt.3.7](FS-fmt.md#37-the---write-report)), its count being the number of rows the dry run lists for it, so a move that drops one entry and adds another is `(2)`, and the lock's rows are its `section-lock entries` among the summary's totals.

A write can create a lock file, where the file does not exist and the write leaves entries in it, and it can delete one, where the file exists and the write leaves no entry in it ([§FS-section-lock.file.shard](FS-section-lock.md#fileshard-one-file-per-cited-declaration)). Both are said. The lock file's first dry-run row ends `; creates the file` or `; deletes the file`, appended once the rows are in the order above and without moving any of them, and its `--write` line ends ` created` or ` deleted` after the count. A rename that moves a declaration's entries to a lock file of the new name reads:

```text
$ grund fmt
docs/notes.md:1: markdown link
.grund/sections/FS-login.lock:1: section lock: drop FS-login.code.1; deletes the file
.grund/sections/FS-sign-in.lock:1: section lock: add FS-sign-in.code.1; creates the file
$ grund fmt --write
rewrote 3 lines: 1 link written, 2 section-lock entries
  .grund/sections/FS-login.lock (1) deleted
  .grund/sections/FS-sign-in.lock (1) created
  docs/notes.md (1)
```

A lock file that exists before the write and after it — refreshed, rebuilt ([§FS-section-lock.write.rebuild](FS-section-lock.md#writerebuild-a-lock-grund-cannot-read-is-rebuilt)), or given one entry and relieved of another — carries neither suffix. The deleted file and the created one are not paired as a move: that each happened is certain, and which deletion goes with which creation is a guess the report does not make.

Like the cross-reference pass, the lock pass needs the whole declaration set, so an unreadable path refuses it ([§FS-fmt.3.2](FS-fmt.md#32-a-run-that-needs-the-whole-declaration-set-rewrites-nothing)).

### write.rebuild: A lock grund cannot read is rebuilt

A lock line that is not `<coordinate> <fingerprint>` — a hand edit, or the conflict markers of a merge — is a `check` error ([§FS-section-lock.checks.invalid-section-lock](FS-section-lock.md#checksinvalid-section-lock-a-lock-line-grund-cannot-read)), and `fmt --check` lists it as `section lock: rebuild` at its line. `fmt --write` rebuilds the file: it keeps every entry it can read, from either side of a conflict, takes for a coordinate entered twice the entry whose fingerprint the section holds now, or else the first, and then adds, refreshes and drops as above. Two changes that each cite a new section of one declaration therefore merge with one `fmt --write`. A merge that also left a conflict block in a declaration body is refused until that body is resolved ([§FS-declarations.checks.conflict-marker](FS-declarations.md#checksconflict-marker-unresolved-merge-conflict)), and the lock is then rebuilt as above.

## move: Moving a cited section on purpose

A cited section that moves changes its address, and [§REQ-spec-section-names.permanence](../requirements/REQ-spec-section-names.md#permanence-a-name-is-permanent) asks that the old address not silently become something else. There are two ways to move one, and both pass `check`.

### move.citations: Update every citation

Change every citation of the old address to the new one and run `fmt --write`. The old entry is no longer cited, so it is dropped; the new address is added. Until `fmt --write` runs, `check` reports each citation of the new address as [§FS-section-lock.checks.unrecorded-section-citation](FS-section-lock.md#checksunrecorded-section-citation-a-locked-address-the-lock-does-not-hold).

### move.pointer: Leave a pointer

Keep the old address cited and leave a pointer there: a section at the recorded address whose text cites the address where the recorded text now sits (`See <§>FS-login.parse.2.`). That acknowledges the move: `check` does not report it, and `fmt --write` records the pointer's text as the entry's new fingerprint. Retiring the pointer once nothing cites the old address is a change of its own, and it drops the entry.

### move.copy: Copy, then edit the original

Copying a cited section to a new address and then editing the original is the one case the lock reports though nothing moved. After the edit the recorded text is found only at the copy, which is exactly what a move looks like. The way out is the one the finding names: edit the copy, so its text is no longer the recorded one and the original reads as edited in place, or cite the copy from the original, which acknowledges it as a pointer does.

## limits: What the lock does not catch

### limits.edit-and-move: A section moved and edited in one change

When a cited section moves and its text is edited in the same change, its recorded fingerprint is found nowhere, so the lock reads it as edited in place: `check` passes, and `fmt --write` refreshes the entry. Take `FS-login` with `## format:` holding `### .1: Username` and `## parse:` holding `### .1: Tokens`, `FS-login.format.1` cited and locked. Swap the two `### .1` blocks and edit both texts: `grund check` succeeds, `grund fmt --check` lists `section lock: refresh FS-login.format.1`, and after `fmt --write` the citation of `FS-login.format.1` reads the edited Tokens text. The only trace is that refreshed line in the lock diff `fmt --write` produces. This limit is accepted: a second key beside the fingerprint, such as the title, would report a section retitled and edited in place whenever another section carries its old title.

A move that also updates a section's citations of its own subsections is this limit too, though nothing but the move was meant. Take `FS-x` with `## 1. Overview` holding `### .1. Goals`, whose text says `See <§>FS-x.1.1.`, and `docs/use.md` citing `FS-x.1.1`, locked. Insert a new `## 1. Setup` with its own `### .1. Goals`, so Overview becomes `## 2.`, and rewrite the self-citation to `See <§>FS-x.2.1.` as the move asks. The moved text now differs from the recorded one, so it is found nowhere: `grund check` reports only `FS-x.2.1 is cited but has no entry`, `fmt --write` adds that entry and refreshes `FS-x.1.1` to Setup's text, and the next `check` passes while `grund FS-x.1.1 --full` prints Setup's goals. The citation text inside a section is not normalized the way its nested headings are ([§FS-section-lock.holds.fingerprint](FS-section-lock.md#holdsfingerprint-the-fingerprint)): it is prose, and rewriting it before hashing would also hide a self-citation that genuinely changed its target. The trace is the refreshed line in the lock diff.

### limits.identical: Identical texts

Sections with byte-identical text cannot be told apart, so an entry whose text the declaration repeats guards nothing until they differ ([§FS-section-lock.holds.repeated](FS-section-lock.md#holdsrepeated-a-text-the-declaration-repeats)).

### limits.across: Across declarations

The lock compares a section only with the other sections of its own declaration. A section moved into another declaration is a risk complete-path headings already carry, and it is not covered here.

### limits.baseline: A baseline the lock never saw

The lock can report a move only from a text it recorded before the move. Where it holds no such record, `fmt --write` records the text the address holds now, and `check` has nothing older to compare it with. That happens on a citation added for the first time, on the first `fmt --write` after an upgrade that changed how a fingerprint is computed, on a lock file deleted and written again, and on an entry dropped because its citation was removed for a while ([§FS-section-lock.write.rules](FS-section-lock.md#writerules-add-refresh-drop)). Take `FS-x` citing only `FS-x` itself; chapter `1` is renumbered `2` and a new chapter `1` inserted, and only then does someone write `<§>FS-x.1.1`. `check` reports it unrecorded ([§FS-section-lock.checks.unrecorded-section-citation](FS-section-lock.md#checksunrecorded-section-citation-a-locked-address-the-lock-does-not-hold)), and after `fmt --write` it passes with the new chapter's text recorded. The only trace is the added or changed line in the lock diff, which is why the lock is committed with the citations it records and reviewed with them.

A lock that survives keeps its records. `fmt --write` alone never overwrites an entry whose text moved ([§FS-section-lock.write.never-a-move](FS-section-lock.md#writenever-a-move-fmt-never-approves-a-move)), and a lock grund cannot read is rebuilt from the entries it still can ([§FS-section-lock.write.rebuild](FS-section-lock.md#writerebuild-a-lock-grund-cannot-read-is-rebuilt)). No check refuses a new baseline: telling a first record from a lost one needs history, and `check` reads none ([§FS-check.3.6.4](FS-check.md#364-a-pure-function-of-the-tree)).

### limits.gate: A lock nobody refreshes

An edit that `fmt --write` never recorded leaves the old fingerprint in place, which weakens the guard for that entry until the next `fmt --write`. A repository that runs `fmt --check` in its gate never carries one.

## checks: Checks

Each is a located `grund check` error with its code in [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog), selected by `--only` and `--ignore` like any other, and rendered in text, JSON and the LSP as [§FS-check.2.1](FS-check.md#21-report-format) says. JSON `sites` is `null` for each but `section-address-changed`, which carries the sites that cite the recorded address ([§FS-section-lock.checks.section-address-changed](FS-section-lock.md#checkssection-address-changed-a-cited-sections-text-sits-at-another-address)). None fires in a repository with no lock file and no citation of a locked address. A message names the lock file as the report names every path, so from the workspace root a member's lock is `packages/api/.grund/sections/FS-login.lock`.

### checks.section-address-changed: A cited section's text sits at another address

When an entry's recorded address is cited and is a locked address, the section there no longer has the recorded fingerprint, and another address of the same declaration does, the text that was cited has moved and the address now holds something else. The error is located at the heading that now holds the recorded address, and names every line that still cites it:

```text
FS-login.format.1 is cited, and what .grund/sections/FS-login.lock recorded for it now sits at FS-login.parse.1; move it back, cite FS-login.parse.1 instead, or leave a pointer to it at FS-login.format.1; if FS-login.parse.1 is a copy, edit the copy or cite it from FS-login.format.1; FS-login.format.1 is cited at src/auth/normalize.py:5, src/session.rs:12
```

The last clause, `; <cited address> is cited at <path>:<line>, …`, names each site that cites the recorded address: every one, with no cap, because each is an edit the remedy asks for and a count would send the reader back to `grund refs`. Two citations on one line are one site; sites are sorted by path, then line, as [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration) sorts its sites; and each path is named as the report names every path. JSON `sites` carries the same `{ path, line }` pairs in the same order. The heading is the finding's location and not a site: the text does not name it as a citation, as a value mismatch's `sites` holds what its text names rather than its location ([§FS-errors.5.4](FS-errors.md#54-value-findings)). The sites are read from the whole tree, as *is cited* is, so a run narrowed to the heading's file still names citations outside it, and a narrowed run keeps or drops the finding by its location alone.

The clause is the finding's own words after the remedy segment, which a message may carry ([§FS-errors.5.5.1](FS-errors.md#551-how-a-row-is-read)), and not part of the remedy: the *Fix* cell, the *Note* and `grund config explain section-address-changed` stay as they are, and the clause is not to be moved into the cell. Appended after the whole of the 0.20 message, it keeps that message a verbatim contiguous prefix, so it ships in one release with no warning window ([§FS-errors.requirements.prefix-windows](FS-errors.md#requirementsprefix-windows-a-wording-change-keeps-the-old-message-whole-until-its-window-closes--realized-directional-for-the-next-change)).

The remedy is the *Fix* cell of the code's row in the check code catalog ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)), which the message carries filled in and `grund config explain section-address-changed` prints with its slots unfilled. Its three fixes are the move undone, [§FS-section-lock.move.citations](FS-section-lock.md#movecitations-update-every-citation) and [§FS-section-lock.move.pointer](FS-section-lock.md#movepointer-leave-a-pointer); the last clause is the way out of [§FS-section-lock.move.copy](FS-section-lock.md#movecopy-copy-then-edit-the-original). When the fingerprint sits at more than one other address, the first in document order is named. It is reported for a swap of two sections with or without the same title, for a parent rename after which a sibling takes the old address, and for a section moved under another parent whose old address another section took. It is not reported when the section at the recorded address cites the address it names, nor for an entry written `-`.

The row's *Note* says what the check compares and what writes the lock, from [§FS-section-lock.write.never-a-move](FS-section-lock.md#writenever-a-move-fmt-never-approves-a-move): only `fmt --write` writes a lock, it never approves a move, and a lock is not edited by hand, because an edit that records the new text at the old address, or a citation dropped and restored around a `fmt --write`, silences this report without undoing, following or acknowledging the move.

### checks.unrecorded-section-citation: A locked address the lock does not hold

A citation that resolves to a locked address with no entry in the lock is an error at the citation's site:

```text
FS-login.format.1 is cited but has no entry in .grund/sections/FS-login.lock; run grund fmt --write with no path at the workspace root, or at the project root outside a workspace
```

The same immediate error and repair command apply to mixed addresses,
including complete headings, relative headings and stub targets. From 0.19.0
there is no versioned mixed-path clause ([§DF-mixed-path-citation-errors.decision](../decisions/functional/DF-mixed-path-citation-errors.md#decision-decision)).
The one command, run where [§FS-section-lock.checks.remedy](FS-section-lock.md#checksremedy-the-run-that-writes-the-lock) says, repairs the missing entry ([§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations));
enrollment and fingerprints remain [§FS-section-lock.holds.entry](FS-section-lock.md#holdsentry-one-line-per-locked-address)'s.

Without an entry the address is unguarded, so the citation is not taken as checked until `fmt --write` records it. A citation that resolves to no section at all is [§FS-check.3.2](FS-check.md#32-missing-section) and never this.

### checks.invalid-section-lock: A lock line grund cannot read

A line of a lock file that is not an entry, as [§FS-section-lock.holds.entry](FS-section-lock.md#holdsentry-one-line-per-locked-address) defines one, is an error at that line:

```text
section lock line is not <coordinate> <fingerprint>; to rebuild the file, run grund fmt --write with no path at the workspace root, or at the project root outside a workspace
```

The entries `check` can read are still judged. `fmt --write` rebuilds the file ([§FS-section-lock.write.rebuild](FS-section-lock.md#writerebuild-a-lock-grund-cannot-read-is-rebuilt)).

### checks.remedy: The run that writes the lock

The two errors a `fmt --write` clears, [§FS-section-lock.checks.unrecorded-section-citation](FS-section-lock.md#checksunrecorded-section-citation-a-locked-address-the-lock-does-not-hold) and [§FS-section-lock.checks.invalid-section-lock](FS-section-lock.md#checksinvalid-section-lock-a-lock-line-grund-cannot-read), end in one remedy, worded once: the *Fix* cell of `unrecorded-section-citation`'s row in the check code catalog ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)), which `invalid-section-lock`'s row takes by reference ([§FS-errors.5.5.1](FS-errors.md#551-how-a-row-is-read)), so both messages and both `grund config explain` answers print the one cell.

It says where to run the command, and with no path, because only a run whose scope is a project's config root or the workspace root writes a lock ([§FS-section-lock.write.rules](FS-section-lock.md#writerules-add-refresh-drop)). Outside a workspace, `grund fmt --write` with no path, from anywhere in the project, is that run, and so is one given the root as its path ([§FS-fmt.7.1](FS-fmt.md#71-scope-equivalence)). Given a file, or a directory that is no project's config root, the same command exits `0`, reports `rewrote 0 lines` ([§FS-fmt.3.7](FS-fmt.md#37-the---write-report)) and leaves every lock as it was: a remedy that named the command alone would read as satisfied by a run that wrote nothing.

In a workspace it names the workspace root, because that run writes the locks of every project it loads ([§FS-scan.projects.workspace-root](FS-scan.md#projectsworkspace-root-a-run-at-the-workspace-root-loads-every-project)). A member run ([§FS-scan.projects.member](FS-scan.md#projectsmember-a-member-run-loads-the-member)), with no path from inside the member or given the member's config root, writes the member's entries for citations of its own declarations, but none of its `.grund/sections/<alias>/` entries for a citation into a sibling, which only the workspace root's run writes ([§FS-section-lock.file.members](FS-section-lock.md#filemembers-citations-into-another-project)). A remedy that named the member's root would send such a citation to a run that reports `rewrote 0 lines`, so the one remedy names the run that writes every entry.

The narrowed run itself says nothing about the lock it left alone. Telling that an entry is pending takes the whole declaration set ([§FS-section-lock.write.report](FS-section-lock.md#writereport-what-fmt-reports)), which a narrowed run need not read, so the scope is named where the missing entry is found.
