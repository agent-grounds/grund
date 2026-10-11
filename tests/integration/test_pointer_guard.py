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


def _lead(coordinate):
    """The coordinate's own text, up to its first child, as `grund <ID>` prints it."""
    return subprocess.run([_grund(), coordinate], cwd=REPO_ROOT, capture_output=True,
                          text=True, check=True).stdout


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


def _measure(coordinate):
    row = _sizes().get(coordinate)
    if row is None:
        return None
    return row["lead_words"], _lead(coordinate)


class PointerGuardTests(unittest.TestCase):
    def test_every_pointer_cites_its_point_and_stays_a_pointer(self):
        discovered = moved_pointers(sorted(DOCS.rglob("*.md")))
        wrong = problems(POINTERS, _measure, discovered)
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
