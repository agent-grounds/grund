"""§AR-system.principle.inventory, held as the pointer guard of §AR-system.5:
a restatement that was turned into a pointer stays a pointer. Every row names a
coordinate, the defining points it must cite, the words its own lead may hold as
`grund list --size` measures them, and the restating fragments that must not
grow back. The list only shrinks from both sides: a pointer that grows past its
bound, stops citing its point or restates again fails, and so does a row whose
coordinate is gone or whose bound is looser than what it measures. A section
whose lead is only `Moved to §…` is a pointer by its shape, so one the rows do
not name fails too."""

import functools
import json
import os
import re
import subprocess
import unittest
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
DOCS = REPO_ROOT / "docs"
EXE = "grund.exe" if os.name == "nt" else "grund"

# (coordinate, defining points it must cite, lead-word bound, fragments gone)
POINTERS = (
    # The citation pass over the restatements step 0a of #576 left in place.
    ("FS-declarations.checks.duplicate.1",
     ("REQ-one-answer.text.targets", "REQ-one-answer.stubs.outside-include"), 188,
     ("the editor reports the duplicate, or its absence",
      "the editor's unsaved text where the target is open in one",
      "decides nothing about how many declarations its ID has")),
    ("FS-fmt.6.2.1.1",
     ("REQ-one-answer.stubs.outside-include", "REQ-one-answer.writes.anchors"), 175,
     ("the target is read as `check` and `show` read it",)),
    ("FS-fmt.2.4.6", ("REQ-one-answer.sections.map",), 269,
     ("so the two readings of that one fact cannot disagree",)),
    ("FS-workspace.8.5",
     ("REQ-one-answer.identity.declarations", "REQ-one-answer.writes.anchors"), 98,
     ("can never disagree", "target_for_citation")),
    ("FS-rules.8.1", ("FS-rule-refusals.selectors",), 13, ()),
    # The moves of #576's step 0 left a pointer at each source. Where a source's lead
    # is mostly its pointer (110 words or fewer) it is a row here; a longer one keeps
    # facts of its own, and its move slice alone holds it.
    # The partial pointers the move into REQ-one-answer, held by test_one_answer_moves.py.
    ("FS-show.1.1",
     ("REQ-one-answer.identity.declarations",), 84,
     ("the complete ID printed by `list` is the same ID",)),
    ("FS-show.2.2.3",
     ("REQ-one-answer.sections.map",), 58,
     ("can therefore neither resolve nor be suggested",)),
    ("FS-declarations.checks.duplicate-section.4",
     ("REQ-one-answer.sections.ambiguity",), 18,
     ("a coordinate `check` calls clean but `show` will not resolve",)),
    ("FS-declarations.checks.broken-stub.4",
     ("REQ-one-answer.stubs.verdict", "REQ-one-answer.identity.declarations",
      "REQ-one-answer.stubs.outside-include", "REQ-one-answer.text.targets"), 32,
     ("none reaches an answer of its own",
      "`list` and `list --size` mark its rows duplicate",
      "counted, measured and linked exactly as it is with the target scanned",
      "an unsaved edit to the target moves every answer at once")),
    ("FS-lsp.4", ("REQ-one-answer.text.parity",), 75, ("same hover body",)),
    # The partial pointers the move into FS-citations, held by test_citations_moves.py.
    ("FS-check.1.1",
     ("FS-citations.recognized",), 16,
     ("a citation is the marker followed by an ID",)),
    ("FS-check.1.1.1",
     ("FS-citations.recognized.off-grammar",), 17,
     ("participates in section and dangling checks, inbound counts",)),
    ("FS-check.1.1.2",
     ("FS-citations.recognized.named-candidates",), 12,
     ("the scanner consumes a whole ID-and-dot-tail candidate before deciding what it means",)),
    ("FS-check.1.1.3",
     ("FS-citations.recognized.string-literals",), 19,
     ("an ID-shaped substring inside runtime data does not raise a false dangling citation",)),
    ("FS-check.1.1.3.1",
     ("FS-citations.recognized.string-literals.assigned-data",), 17,
     ("opens an **assigned-data span**, not a docstring",)),
    ("FS-check.1.1.4",
     ("FS-citations.recognized.link-destinations",), 17,
     ("the parallel carve-out is the link destination",)),
    ("FS-check.1.1.5",
     ("FS-citations.recognized.contexts",), 35,
     ("this is what makes an example ID safe to write in documentation",)),
    ("FS-check.1.1.6", ("FS-citations.recognized.value-bindings",), 15, ("remains one citation",)),
    ("FS-check.1.1.7", ("FS-citations.recognized.snapshots",), 12, ("is not a special input",)),
    ("FS-check.1.1.8",
     ("FS-citations.recognized.local-sections",), 19,
     ("is a declaration-local section candidate",)),
    ("FS-check.1.1.9",
     ("FS-citations.recognized.escapes",), 19,
     ("begins immediately after the literal escape",)),
    ("FS-check.1.1.10",
     ("FS-citations.recognized.marker-start",), 16,
     ("establishes a citation start even when the marker ends in a word character",)),
    ("FS-check.1.1.11",
     ("FS-citations.recognized.globs",), 22,
     ("never ends a marked token early to leave a citation of its prefix",)),
    ("FS-check.1.2",
     ("FS-citations.shorthand",), 17,
     ("the number alone already identifies a declaration within its kind",)),
    ("FS-check.1.2.1",
     ("FS-citations.shorthand.shape",), 10,
     ("with the `{slug}` placeholder and one adjacent literal separator removed",)),
    ("FS-check.1.2.2",
     ("FS-citations.shorthand.marker",), 14,
     ("so the marker is what supplies the intent",)),
    ("FS-check.1.2.3",
     ("FS-citations.shorthand.full-id-wins",), 20,
     ("the shorthand pass only sees what is left",)),
    ("FS-check.1.2.4",
     ("FS-citations.shorthand.edge",), 21,
     ("the citation participates in the graph like any other",)),
    ("FS-check.1.2.5", ("FS-citations.shorthand.numeric-run",), 20, ("is a numeral in a run",)),
    ("FS-check.1.2.6",
     ("FS-citations.shorthand.cli",), 22,
     ("where nothing is persisted and the caller gets the declaration",)),
    ("FS-check.3.13.1",
     ("FS-citations.rewrite-exemption",), 23,
     ("where the text is legitimate as it stands",)),
    ("FS-check.3.17.2", ("FS-citations.wrap.shape",), 19, ("the citation wrapped as",)),
    ("FS-fmt.2.3.4",
     ("FS-citations.zones.link-destination",), 26,
     ("rewriting it would change the URL rather than the visible citation",)),
    ("FS-fmt.2.3.5",
     ("FS-citations.zones.inline-code",), 33,
     ("it closes at the next run of **exactly** *n* backticks",
      "One reading serves every caller, because they have to agree")),
    ("FS-fmt.6.3.1",
     ("FS-citations.wrap.recognized",), 26,
     ("its immediately-preceding character is `[` and its immediately-following text begins `](`",)),
    # The partial pointers the move into FS-scan, held by test_scan_moves.py.
    ("FS-config.3.5.1",
     ("FS-scan.scope.links",), 11,
     ("is followed only while its canonical target remains inside the independently checked project's canonical root",)),
    ("FS-config.3.5.2",
     ("FS-scan.identity.link-path",), 11,
     ("is reported at the in-tree link path, never the target's",)),
    ("FS-config.3.5.2.1",
     ("FS-scan.identity.root-path",), 14,
     ("is scanned and reported under the path the run was handed, never the physical one it resolves to",)),
    ("FS-config.3.5.3",
     ("FS-scan.scope.links.names",), 12,
     ("is excluded exactly as a real directory of that name would be",)),
    ("FS-config.3.5.4",
     ("FS-scan.identity.read-once",), 10,
     ("file is read once however many spellings reach it",)),
    ("FS-config.3.5.5",
     ("FS-scan.scope.links.unresolvable",), 16,
     ("a loop is pruned where it is met",)),
    ("FS-config.3.5.6",
     ("FS-scan.scope.links.owed",), 11,
     ("That report is owed only where the scan would otherwise have read through the link",)),
    ("FS-config.3.5.6.1",
     ("FS-scan.scope.links.blind-spot",), 15,
     ("would have been a directory to descend into had it resolved",)),
    ("FS-config.3.5.7",
     ("FS-scan.scope.include",), 11,
     ("never a subdirectory that merely happened to be passed as `grund`'s path argument",
      "intentionally names external content and is still scanned")),
    ("FS-config.3.5.8",
     ("FS-scan.scope.homes",), 10,
     ("A home that does not exist scans as nothing and earns no finding",)),
    ("FS-config.3.5.9",
     ("FS-scan.scope.roots",), 12,
     ("no scan root is pruned by `exclude`, an ignore file, or the hidden-directory rule",)),
    ("FS-config.3.5.10",
     ("FS-scan.scope.homes.why",), 9,
     ("outside `include` was never scanned, so its declarations did not exist",)),
    ("FS-config.3.5.11",
     ("FS-scan.scope.not-a-fence",), 12,
     ("brings into the scan is invisible rather than merely unchecked",)),
    ("FS-config.3.5.12",
     ("FS-scan.scope.hidden",), 17,
     ("it skips a file whose own name begins with `.` by the name that file wears",)),
    ("FS-config.3.5.13",
     ("FS-scan.scope.extensions",), 12,
     ("its native heading syntax still declares nothing",)),
    ("FS-config.3.5.15",
     ("FS-scan.scope.ignore-files",), 9,
     ("makes the scanner honor every form of ignore file the `ignore` crate recognizes",)),
    ("FS-workspace.5",
     ("FS-scan.projects.workspace-root", "FS-scan.projects.narrowed"), 45,
     ("and all configured members, aggregates findings",
      "narrower than the config root is one narrowed scan of the enclosing project")),
    ("FS-workspace.5.1",
     ("FS-scan.projects.member",), 103,
     ("discovers the member's own config first and, for a member that declares no `[workspace]` block of its own, validates it as an independent project",)),
    ("FS-workspace.8.6.1",
     ("FS-scan.path.cover",), 11,
     ("draws the aggregate/narrow line where `grund check` draws it and not where `list` does",)),
    ("FS-workspace.8.8",
     ("FS-scan.projects.discovery", "FS-scan.path.discovery"), 11,
     ("A scope inside a member with no config of its own is member-local all the same",
      "behaves as if the command were invoked from that path")),
    ("FS-show.1.4",
     ("FS-scan.path.show",), 31,
     ("is the directory or file whose tree is scanned to resolve the ID",)),
    # The partial pointers the move into FS-languages, held by test_languages_moves.py.
    ("FS-show.2.3.1", ("FS-languages.comment-block",), 42, ()),
    ("FS-show.2.3.1.1",
     ("FS-languages.comment-block.open",), 12,
     ("Scan **backwards** from the declaration line over consecutive lines",
      "matches the same comment prefix family",
      "For Python triple-quoted docstrings: scan backward until the opening")),
    ("FS-show.2.3.1.2",
     ("FS-languages.comment-block.close",), 12,
     ("Scan **forwards** from the declaration line by the symmetric rules",
      "a declaration written as a one-line `/* … */` closes on its own line")),
    ("FS-show.2.3.1.3",
     ("FS-languages.comment-block.early-end",), 17,
     ("if another declaration line of any ID is encountered, the block ends at the line before it",)),
    ("FS-show.2.3.2",
     ("FS-languages.comment-wrapper", "FS-languages.docstring"), 71,
     ("The whitespace in front of the comment prefix belongs to the comment's envelope",
      "a leading ` * ` (with surrounding spaces) is removed if present",
      "Trailing comment-close tokens (`*/`) on their own line are dropped entirely",
      "Blank lines inside the block are preserved",
      "For Python docstrings, no prefix is stripped")),
    ("FS-inline-citation-style.1.2",
     ("FS-languages.comment-block.forms",), 31,
     ("a run of adjacent lines whose first non-whitespace token is the same line-comment prefix",
      "Adjacency is broken by any line that is not part of the same block",
      "An empty comment line — the prefix alone")),
    ("FS-config.3.5.14",
     ("FS-languages.prefixes",), 21,
     ("The two lists compose",
      "`;` pairs with Lisp, Scheme, and Clojure extensions",
      "Any line whose first non-whitespace run is a configured prefix is eligible to host")),
    ("FS-values.2.4.1",
     ("FS-languages.comment-wrapper", "FS-languages.docstring"), 42,
     ("`/* ... */` and `*` continuation lines used by Javadoc/JSDoc",
      "content inside enabled Python")),
    # Sections that are only their `Moved to` pointer, found by their shape.
    ("FS-check.3.3", ("FS-declarations.checks.duplicate",), 19, ()),
    ("FS-check.3.4", ("FS-declarations.checks.broken-stub",), 20, ()),
    ("FS-check.3.7", ("FS-declarations.checks.misplaced-declaration",), 22, ()),
    ("FS-check.3.9", ("FS-declarations.checks.section-heading-level",), 21, ()),
    ("FS-check.3.16", ("FS-declarations.checks.duplicate-section",), 20, ()),
    ("FS-check.3.19", ("FS-declarations.checks.orphan-section",), 21, ()),
    ("FS-check.3.23", ("FS-declarations.checks.section-outside-declaration",), 21, ()),
    ("FS-check.4.6", ("FS-declarations.checks.declaration-near-miss",), 20, ()),
    ("FS-check.4.7", ("FS-check.3.30",), 34, ()),
    ("FS-check.4.13", ("FS-declarations.checks.oversized-lead",), 20, ()),
    ("FS-check.4.14", ("FS-declarations.checks.unmarked-heading",), 20, ()),
    ("FS-rules.3.5", ("FS-rule-refusals.sentences",), 7, ()),
    ("FS-rules.3.5.1", ("FS-rule-refusals.sentences.whitespace",), 8, ()),
    ("FS-rules.3.5.2", ("FS-rule-refusals.sentences.named-sections",), 18, ()),
    ("FS-rules.3.5.3", ("FS-rule-refusals.sentences.chapter-path",), 15, ()),
    ("FS-rules.3.5.4", ("FS-rule-refusals.sentences.rewrite",), 17, ()),
    ("FS-rules.3.5.4.1", ("FS-rule-refusals.sentences.rewrite.replaced",), 11, ()),
    ("FS-rules.3.5.4.2", ("FS-rule-refusals.sentences.rewrite.recovered",), 12, ()),
    ("FS-rules.3.5.4.3", ("FS-rule-refusals.sentences.rewrite.reparsed",), 13, ()),
    ("FS-rules.3.5.4.4", ("FS-rule-refusals.sentences.rewrite.no-form",), 10, ()),
    ("FS-rules.3.5.4.5", ("FS-rule-refusals.sentences.rewrite.by-refusal",), 10, ()),
    ("FS-rules.3.5.4.6", ("FS-rule-refusals.sentences.rewrite.fs-only",), 13, ()),
    ("FS-rules.3.5.5", ("FS-rule-refusals.sentences.conjunction",), 15, ()),
)
# A pointer that is nothing but its pointer: `Moved to` and the citation.
MOVED = re.compile(r"^Moved to \[?§([A-Za-z0-9./_-]+?)\]?(?:\(|\.?$)")
DECLARATION = re.compile(r"^# ([A-Z]+-[a-z0-9-]+):")
SECTION = re.compile(r"^#{2,6} ((?:[a-z][a-z0-9-]*|\d+)(?:\.(?:[a-z][a-z0-9-]*|\d+))*)\.?(?::| )")


@functools.lru_cache(maxsize=None)
def _grund():
    """The binary under test: the gate's build, or this checkout's built once."""
    built = os.environ.get("GRUND_BUILT")
    if built:
        return built
    target = Path(os.environ.get("CARGO_TARGET_DIR") or REPO_ROOT / "target")
    subprocess.run([os.environ.get("CARGO", "cargo"), "build", "-p", "grund", "--locked"],
                   cwd=REPO_ROOT, check=True)
    return str(target / "debug" / EXE)


@functools.lru_cache(maxsize=None)
def _sizes():
    """Every coordinate's measurement, keyed `ID` or `ID.section`."""
    out = subprocess.run([_grund(), "list", "--size=words", "--format", "json"], cwd=REPO_ROOT,
                         capture_output=True, text=True, check=True).stdout
    rows = {}
    for line in out.splitlines():
        row = json.loads(line)
        key = row["id"] + (f".{row['section']}" if row["section"] else "")
        rows[key] = row
    return rows


def _leads(coordinates):
    """Each coordinate's own text, up to its first child, as `grund <ID>` prints it,
    from one `grund show --batch` rather than one run per row."""
    queries = "".join(json.dumps({"id": c.partition(".")[0], "section": c.partition(".")[2] or None})
                      + "\n" for c in coordinates)
    out = subprocess.run([_grund(), "show", "--batch", "--format", "json"], cwd=REPO_ROOT,
                         input=queries, capture_output=True, text=True).stdout
    leads = {}
    for coordinate, line in zip(coordinates, out.splitlines()):
        record = json.loads(line)
        if record["ok"]:
            body = record["result"]["body"]
            leads[coordinate] = body.split("\n", 1)[1] if body.startswith("#") else body
    return leads


def _flat(text):
    return " ".join(text.split()).lower()


def _cites(text, point):
    return re.search(r"§" + re.escape(point) + r"(?![\w-]|\.\w)", text)


def moved_pointers(paths):
    """Every coordinate whose lead is one `Moved to §…` line, with the point it names."""
    found = {}
    for path in paths:
        declaration, section, lead = None, None, []
        for line in path.read_text(encoding="utf-8").splitlines() + ["## end"]:
            heading = DECLARATION.match(line) or SECTION.match(line)
            if heading or line.startswith("#"):
                body = [text for text in lead if text.strip()]
                if declaration and section and len(body) == 1 and MOVED.match(body[0]):
                    found[f"{declaration}.{section}"] = MOVED.match(body[0]).group(1)
                lead = []
                if DECLARATION.match(line):
                    declaration, section = DECLARATION.match(line).group(1), None
                elif SECTION.match(line):
                    section = SECTION.match(line).group(1)
                continue
            lead.append(line)
    return found


def problems(rows, measure, discovered):
    """What is wrong with `rows` against what the tree measures, both ways round.

    `measure(coordinate)` is `(lead words, lead text)`, or `None` where the
    coordinate does not resolve; `discovered` maps each shape-found pointer to
    the point it names."""
    wrong = []
    named = {row[0] for row in rows}
    for coordinate, points, bound, gone in rows:
        measured = measure(coordinate)
        if measured is None:
            wrong.append(f"{coordinate}: the row names a coordinate that is gone; take the row out")
            continue
        words, text = measured
        if words > bound:
            wrong.append(f"{coordinate}: {words} words in its lead, over the bound {bound}: "
                         "the pointer is growing back into a restatement")
        elif words < bound:
            wrong.append(f"{coordinate}: {words} words in its lead, under the bound {bound}: "
                         "lower the row's bound to what it measures")
        for point in points:
            if not _cites(text, point):
                wrong.append(f"{coordinate}: does not cite §{point}, its defining point")
        for fragment in gone:
            if _flat(fragment) in _flat(text):
                wrong.append(f"{coordinate}: restates again: {fragment!r}")
    for coordinate, point in sorted(discovered.items()):
        if coordinate not in named:
            wrong.append(f"{coordinate}: a `Moved to §{point}` pointer no row names; add its row")
    return wrong


class PointerGuardTests(unittest.TestCase):
    def test_every_pointer_cites_its_point_and_stays_a_pointer(self):
        discovered = moved_pointers(sorted(DOCS.rglob("*.md")))
        with ThreadPoolExecutor(max_workers=2) as pool:
            sizes = pool.submit(_sizes)
            leads = pool.submit(_leads, [row[0] for row in POINTERS])
            sizes, leads = sizes.result(), leads.result()

        def measure(coordinate):
            row = sizes.get(coordinate)
            return None if row is None else (row["lead_words"], leads.get(coordinate, ""))

        wrong = problems(POINTERS, measure, discovered)
        self.assertEqual([], wrong, "\n" + "\n".join(wrong))


class PointerGuardHarnessTests(unittest.TestCase):
    """The guard fails on a new violation and on a row that no longer applies."""

    ROW = ("FS-x.1", ("REQ-y.z",), 10, ("restated words",))

    def check(self, measured, discovered=None, rows=None):
        return problems(rows or (self.ROW,), lambda c: measured.get(c), discovered or {})

    def test_a_clean_pointer_passes(self):
        self.assertEqual([], self.check({"FS-x.1": (10, "See \u00a7REQ-y.z.")}))

    def test_a_pointer_that_grows_fails(self):
        self.assertIn("growing back", " ".join(self.check({"FS-x.1": (11, "See \u00a7REQ-y.z.")})))

    def test_a_pointer_that_stops_citing_fails(self):
        self.assertIn("does not cite", " ".join(self.check({"FS-x.1": (10, "See \u00a7REQ-y.zz.")})))

    def test_a_restatement_that_grows_back_fails(self):
        wrong = self.check({"FS-x.1": (10, "See \u00a7REQ-y.z. Restated  words.")})
        self.assertIn("restates again", " ".join(wrong))

    def test_a_row_whose_coordinate_is_gone_fails(self):
        self.assertIn("is gone", " ".join(self.check({})))

    def test_a_row_looser_than_its_pointer_fails(self):
        self.assertIn("lower the row's bound", " ".join(self.check({"FS-x.1": (9, "\u00a7REQ-y.z")})))

    def test_an_unlisted_moved_pointer_fails(self):
        wrong = self.check({"FS-x.1": (10, "\u00a7REQ-y.z")}, {"FS-x.2": "FS-w"})
        self.assertIn("FS-x.2: a `Moved to \u00a7FS-w` pointer no row names", " ".join(wrong))

    def test_the_shape_finds_a_moved_section(self):
        import tempfile
        with tempfile.TemporaryDirectory() as scratch:
            page = Path(scratch) / "FS-x.md"
            page.write_text("# FS-x: X\n\nLead.\n\n## 1. One\n\nMoved to [\u00a7FS-w.2](FS-w.md#2-two).\n\n"
                            "## 2. Two\n\nMoved to \u00a7FS-w.3, and more.\n", encoding="utf-8")
            self.assertEqual({"FS-x.1": "FS-w.2"}, moved_pointers([page]))


if __name__ == "__main__":
    unittest.main()
