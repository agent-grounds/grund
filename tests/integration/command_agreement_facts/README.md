# Authored command agreement facts

This directory is the input-authored oracle for
[§REQ-one-answer.proof.evidence](../../../docs/requirements/REQ-one-answer.md#proofevidence-execution-and-independent-truth),
with exact question accounting under
[§REQ-one-answer.proof.matrix](../../../docs/requirements/REQ-one-answer.md#proofmatrix-the-complete-comparison-matrix).
The loader and consumers are in the CLI test support modules. They follow
[§AR-system.principle](../../../docs/architecture/README.md#principle-one-fact-one-point-one-implementation).

**Construction status:** the twelve cases without input repositories have
input-pinned applicability manifests here. The 1,511 input cases still lack
complete generic fact manifests. The matrix rejects that gap before native
preparation. The regression-envelope JSON
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
  nonempty unique `coordinates` array, `calls` and `writtenChanges`.

The invocation context contains exactly `relative`, `path`, `cwd`, `editorPath`,
`explicit`, `overlays`, `overlayRelative`, `heldBytes`, `bufferOnly` and `scopes`.
Missing optional context fields are explicitly null. Physical copied roots use
`<repo>`; owner-relative paths, field values and buffer bytes retain their meaning.
The held edit is the input-authored file plus the materializer's deterministic
newline, or its explicitly authored buffer-only file. Each question's scope uses
its actual producer/report base; section-map paths remain section coordinates.

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

There is no answer-regeneration command. Do not copy engine output into these
files, replace input provenance with another surface, or use the original
manifest's one golden for unrelated questions. Execution receipts are separate
artifacts; they compare against these authored facts and account for every exact
question/producer obligation. Missing-manifest readiness diagnostics identify
unfinished construction and never claim executed coverage or payment.
