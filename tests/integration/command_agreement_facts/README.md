# Authored command agreement facts

This directory is the input-authored oracle for
[§REQ-one-answer.proof.evidence](../../../docs/requirements/REQ-one-answer.md#proofevidence-execution-and-independent-truth),
with exact question accounting under
[§REQ-one-answer.proof.matrix](../../../docs/requirements/REQ-one-answer.md#proofmatrix-the-complete-comparison-matrix).
The loader and consumers are in the CLI test support modules. They follow
[§AR-system.principle](../../../docs/architecture/README.md#principle-one-fact-one-point-one-implementation).

**Construction status:** every case the acceptance asks has a manifest here:
the seed's sampled first projects and every case of `basic`, `cover`, `scan` and
`stub`, and of tranche 3's `agent`, `cross`, `declare`, `deterministic`, `guide`,
`haskell`, `json`, `markdown`, `marked`, `named`, `report` and `rust`. The basic
tranche and tranche 3 each add their dynamically
discovered families whole
([§REQ-one-answer.proof.matrix.family](../../../docs/requirements/REQ-one-answer.md#proofmatrixfamily-a-family-is-covered-whole-or-not-at-all)).
`symlink` stays a row of `WIDENING`: its sampled project,
`symlink-alias-read-once`, has facts, and its fourteen other cases do not yet.
`line`, `e2e`, `remote`, `relative` and `path` stay rows of `WIDENING`
because tranche 3 did not author their facts.
`external` stays a row because no point fixes the `details` of an invalid-id
failure at a workspace root:
[§FS-distribution.3.3.1](../../../docs/functional-spec/FS-distribution.md#331-typed-immutable-results)
keeps every engine output field without naming them.
The nineteen cases without input repositories keep input-pinned applicability
manifests. A manifest that exists is validated whether or not its case is
asked, and the matrix rejects every gap before native preparation. Query
consumers require physical-fact partitions. The regression-envelope JSON
files in the CLI fixtures are recorded comparator inputs, not oracle manifests.
This directory and the loader do not discharge S4.

Each original corpus directory needs one `<case>.json` file with these fields:

- `version`: `1`; `case`: the exact original directory name.
- `inputFiles`: an exact physical inventory of the original case, including
  repository files, directories, symlinks and command/golden manifests. Each
  relative path maps to one of `{"bytes": [0, ...]}`, `{"directory": true}` or
  `{"symlink": "literal target"}`. Input bytes may be read from disk; expected
  answers must be authored from the inputs and defining points.
- `variants`: exactly `whole-tree`, `narrowed`, `path-argument`, `editor-overlay`.
  Each applicable variant has an explicit nullable `absent`, `context`, a
  `queryInventory`, a nonempty unique `coordinates` array, `calls` and
  `writtenChanges`.

The invocation context contains exactly `relative`, `path`, `cwd`, `editorPath`,
`explicit`, `overlays`, `overlayRelative`, `heldBytes`, `bufferOnly` and `scopes`.
Missing optional context fields are explicitly null. Physical copied roots use
`<repo>` when there is one copy. An overlay context distinguishes `<held>` and
`<saved>`, including overlay map keys and each scope's filesystem fields. Native,
slash and Windows verbatim spellings are normalized only in those filesystem
fields; held text and arbitrary map keys retain their bytes. Answer paths still
use their common `<repo>` domain through the separate answer normalizer.
The held edit is the input-authored file plus the materializer's deterministic
newline, or its explicitly authored buffer-only file. Each question's scope uses
its actual producer/report base; section-map paths remain section coordinates.

`queryInventory.files` maps every physical input file under `repo/` to its exact
`bytes` and an authored `segments` array. The saved newline edit (or buffer-only
control) replaces that file's bytes before comparison with the inventory. Each
segment has byte offsets `start` and `end`, exactly one `source` UTF-8 string or
`sourceBytes` array, and a `role`. Segments partition all bytes consecutively,
without gaps or overlaps. Empty files have an empty partition. Physical file
membership is compared to the pinned original inputs independently of the
partition; removing a file from both the facts and queries cannot shrink it.

Each role carries `point` and `authoredFrom`, plus these defining facts:

- `declaration`: `id`, explicit nullable `project`, and `sectionSeparator`.
  Its site identity is `<file>:<start-byte-offset>`.
- `section`: a `declaration` site reference, its physically written `component`,
  and a nullable `parent` section site. Relative components inherit their parent's
  path; missing parents, wrong owners and cycles are errors.
- `citation`: its physically written `query`, including undeclared coordinates.
- `alias`: physically written `alias` and explicit nullable target `project`.
- `excluded`: a defining-point `reason` for bytes that contribute no query fact.
  Exclusion classifications need independent semantic review; a complete byte
  partition alone does not prove that those classifications are correct.

The closure includes all declaration and section questions, qualified alias
questions, cited coordinates and the explicit `absentQuery` control (also through
each alias). `coordinates` must equal that closure. The same derivation supplies
both the driver and receipt obligations; matching deleted calls cannot certify
a smaller supplied list. Deleting an inventoried fact leaves an invalid physical
partition. A manifest may author this representation through a compact `facts` block
instead; see below.

`calls` has these exact keys:

- `check`, `list`, `list-size`, `cover`, `cover-lines`, `fmt`, `show-batch`,
  `fmt-write`, each with `cli`, `python` and `node` expected answers.
- `show:<coordinate>` and `refs:<coordinate>` for every authored coordinate,
  each with all three expected answers. Include an authored absent coordinate
  and its expected answers for a legitimately empty catalog.
- `editor:snapshot` with an `editor` expected answer, and
  `editor:show:<coordinate>` for every coordinate when the snapshot succeeds.
  A pinned refused snapshot instead has
  `editorUnavailable` with reason `snapshot refuses` and point
  [§FS-lsp.2.2.1](../../../docs/functional-spec/FS-lsp.md#221-workspace-folders-anchor-discovery)
  (the point that defines loading the snapshot).

Every call includes its most-specific `point` and an `authoredFrom` explanation
of the physical input sites and rules that settle its expected answer. CLI
answers include the exact `exit`, normalized `stdout` and `stderr`: all-JSON
streams are arrays of complete records (including explicitly empty arrays);
other streams are strings. Host and editor answers retain complete structured
envelopes, including warnings/refusals. Serialized show JSON is decoded for path
normalization then serialized canonically. Bodies, spans/maps/anchors,
measurements and empty/refused answers must be pinned, not inferred from hits
or a returned declaration catalog. Node-only metadata remains asserted as well.

Before common projection, `agreement_truth::host` compares Node's entire
envelope with these facts. Successful ordinary show and wholly successful batch
envelopes with all metadata fields, null `manifest` and empty cautions no longer
need the extra native-peer scan. Their `id`, `section` and `kind_title` derive
from the authored declaration/section and owning config; `manifest` is null for
these non-E2E declarations. Bodies, paths, lines, JSON and section maps retain
the exact comparison. Refusals, mixed batches, nonempty cautions, manifests and
unrecognized schemas retain the extra peer pending an independent audit
([§REQ-one-answer.proof.evidence](../../../docs/requirements/REQ-one-answer.md#proofevidence-execution-and-independent-truth)).

`writtenChanges` is an authored map of output paths to resulting byte-inventory
entries (or `{"remove": true}`). The independent saved-input inventory supplies
the unchanged entries. Explicit `{}` pins entirely unchanged output; an omitted
map is an error. Every CLI/Python/Node writer is independently checked against
the resulting full inventory before peer equality. Output anchors are pinned
in these resulting bytes. Reuse an original `expected.repo` only after establishing
that its authored invocation, options and variant domain match this writer.

An inapplicable variant names one of the materializer's precise physical reasons
in `absent`. Materialization still checks that reason against the actual input.
Windows symlink privilege refusals are separately verified against the manifest
and actual error code; they cannot be authored as a general skip.

## Compact facts

A manifest may pin its input once, as a top-level `facts` block, and leave each
applicable variant only its `absent` and what that variant reads differently.
The loader expands it, after holding `inputFiles` to the fixture and before
validation, into every variant's context, physical query inventory, coordinates,
calls and written changes for the CLI, Python, Node and the editor. The block
carries `point` and `authoredFrom`, and then:

- `scanned`: the files the walk reads, by the spelling it keeps;
  `kinds` when the config declares its own, and `workspace` and `aliases` for
  a workspace root.
- `declarations`, `sections` and `citations`: each by its file and its written
  heading or token, with the title, anchor, body range, enclosing declaration
  and the stub, duplicate or value-root facts its point defines. A front-matter
  coordinate is a section marked `frontMatter`, with its `slice`
  ([§FS-fields.front-matter.surfaces](../../../docs/functional-spec/FS-fields.md#front-mattersurfaces-what-each-surface-does-with-them)). A section's
  optional `file` selects its physical owner when `of` names duplicate homes;
  ambiguous ownership is rejected. Size rows interleave those homes by
  coordinate, then site, keeping every measurement local to its body
  ([§FS-list.2.7](../../../docs/functional-spec/FS-list.md#27-duplicate-sections)).
- `findings` and `refusals`: each finding as `check` publishes it, and each
  refused query with its code, message and the questions it refuses.
- `fmt`: the changes a dry run lists and a write makes, and the written lines.
- `absentQuery`, `editor` and `coverLines` where the case needs them.

Lines, columns, byte offsets, bodies and measurements are read from the input
bytes, so the authored part is what the defining points decide. A variant may
pin `findings`, `refusals`, `scanned`, `declarations`, `sections`, `citations`,
`fmt` or `coverLines` for itself: a member run of a workspace reads its member
alone (`member`), and an explicit path names a linked file by the spelling it
gives. A link the case's `symlinks` manifest declares reads as its target, and
the editor names its sites by that target, the file's filesystem identity.
`inputs.py <case>` refreshes a manifest's `inputFiles` from the fixture and
reads nothing else.

There is no answer-regeneration command. Do not copy engine output into these
files, replace input provenance with another surface, or use the original
manifest's one golden for unrelated questions. Execution receipts are separate
artifacts; they compare against these authored facts and account for every exact
question/producer obligation. Missing-manifest readiness diagnostics identify
unfinished construction and never claim executed coverage or payment.
