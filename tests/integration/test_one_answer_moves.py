"""§AR-system.principle.inventory — a promise that two commands agree lives at
one point, §REQ-one-answer, and the specs it was moved out of cite it rather
than restate it. For each move this holds three things: the destination
chapter holds the moved sentence, the source section no longer does, and the
source cites the destination, so a restatement that grows back fails here.
It also holds the requirement's chapter map and its closed list of
differences, and the rule's own homes: the `AR-system` principle, its
decision record, and the one `AGENTS.md` line that points at it."""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
DOCS = REPO_ROOT / "docs"
REQUIREMENT = DOCS / "requirements" / "REQ-one-answer.md"
FS = DOCS / "functional-spec"
ARCHITECTURE = DOCS / "architecture" / "README.md"
DECISION = DOCS / "decisions" / "architectural" / "DA-one-fact-one-point-one-implementation.md"
AGENTS = REPO_ROOT / "AGENTS.md"
HEADING = re.compile(r"^(#{1,6}) (\S+?)(?::? )(.*)$")

CHAPTERS = (
    "contract",
    "identity", "identity.declarations",
    "sections", "sections.map", "sections.ambiguity",
    "stubs", "stubs.verdict", "stubs.outside-include",
    "text", "text.targets", "text.parity",
    "scope", "scope.narrowed",
    "writes", "writes.anchors",
    "differences",
    "differences.identity", "differences.sections", "differences.stubs",
    "differences.text", "differences.scope",
    "proof", "why",
)
DIFFERENCE_ROWS = {
    "identity": (),
    "sections": ("P01", "P02", "P03", "P07", "P10", "P12", "P16"),
    "stubs": (),
    "text": ("R06a", "R06b", "R06c", "R06d"),
    "scope": ("R01", "R02"),
}

# (source file, source heading label, destination chapters it must cite,
#  fragments that moved out of it into the first destination listed with them)
MOVES = (
    ("FS-show.md", "1.1", ("identity.declarations",),
     {"identity.declarations": ("the complete ID printed by `list` is the same ID",)}),
    ("FS-show.md", "2.2.3", ("sections.map",),
     {"sections.map": ("can therefore neither resolve nor be suggested",)}),
    ("FS-declarations.md", "checks.duplicate-section.4", ("sections.ambiguity",),
     {"sections.ambiguity": ("a coordinate `check` calls clean but `show` will not resolve",)}),
    ("FS-declarations.md", "stubs.target.1", ("text.targets",),
     {"text.targets": ("every answer is the one the same text gives once it is saved",)}),
    ("FS-declarations.md", "checks.broken-stub.4",
     ("stubs.verdict", "identity.declarations", "stubs.outside-include", "text.targets"),
     {"stubs.verdict": ("none reaches an answer of its own",),
      "identity.declarations": ("`list` and `list --size` mark its rows duplicate",),
      "stubs.outside-include": ("counted, measured and linked exactly as it is with the target scanned",),
      "text.targets": ("an unsaved edit to the target moves every answer at once",)}),
    ("FS-check.md", "1.3.6.1", ("scope.narrowed",),
     {"scope.narrowed": ("never disagree about a citation they both read",)}),
    ("FS-lsp.md", "4.", ("text.parity",),
     {"text.parity": ("same hover body",)}),
    ("FS-show.md", "3.1.3.1", ("writes.anchors",),
     {"writes.anchors": ("`grund fmt --cross-refs` derives for a citation of the same coordinate",)}),
)
# What a move leaves behind on purpose: the show-only sentence of 2.2.3 (V-D02 i)
# and the formatter's form 3.1.3.1 still cites (V-C07).
KEPT = (
    ("FS-show.md", "2.2.3", "A query for its coordinate has the ordinary `section not found` result"),
    ("FS-show.md", "3.1.3.1", "§FS-fmt.6.2]"),
)


def _flat(text):
    return " ".join(text.split()).lower()


def _sections(path):
    """Map each heading label to the text under it, up to the next heading."""
    sections, label, lines = {}, None, []
    for line in path.read_text(encoding="utf-8").splitlines():
        match = HEADING.match(line)
        if match:
            if label is not None:
                sections.setdefault(label, "\n".join(lines))
            label, lines = match.group(2).rstrip(":"), []
        elif label is not None:
            lines.append(line)
    if label is not None:
        sections.setdefault(label, "\n".join(lines))
    return sections


def _cites(text, chapter):
    return re.search(r"§REQ-one-answer\." + re.escape(chapter) + r"(?![\w.-])", text)


class OneAnswerMoveTests(unittest.TestCase):
    def requirement(self):
        self.assertTrue(REQUIREMENT.is_file(), f"{REQUIREMENT.relative_to(REPO_ROOT)} is not there")
        return _sections(REQUIREMENT)

    def test_every_chapter_the_requirement_promises_is_declared(self):
        declared = self.requirement()
        self.assertIn("REQ-one-answer", declared)
        self.assertEqual([], [c for c in CHAPTERS if c not in declared], "chapters missing")

    def test_the_differences_list_has_its_rows(self):
        declared = self.requirement()
        for group, rows in DIFFERENCE_ROWS.items():
            body = declared.get(f"differences.{group}", "")
            found = tuple(re.findall(r"^- \*\*([\w-]+?)\.\*\*", body, re.M))
            self.assertEqual(rows, found, f"rows of differences.{group}")
        # R04 and R05 closed with DF-a-buffer-is-the-file (#595); U01 and U05 closed and
        # U07 and U08 were found with #597; U04, U07 and U08 closed with #651; G06
        # closed with #671; G09 closed with #680; Placement, U03a to U03d and U06 closed
        # with #650.
        self.assertEqual(13, sum(len(rows) for rows in DIFFERENCE_ROWS.values()))

    def test_each_moved_sentence_lives_at_its_destination_only(self):
        declared = _sections(REQUIREMENT) if REQUIREMENT.is_file() else {}
        for source, label, cited, moved in MOVES:
            body = _sections(FS / source).get(label)
            self.assertIsNotNone(body, f"{source} {label} is not there")
            for chapter in cited:
                self.assertTrue(_cites(body, chapter),
                                f"{source} {label} does not cite REQ-one-answer.{chapter}")
            for chapter, fragments in moved.items():
                for fragment in fragments:
                    self.assertNotIn(_flat(fragment), _flat(body),
                                     f"{source} {label} still restates {chapter}")
                    self.assertIn(_flat(fragment), _flat(declared.get(chapter, "")),
                                  f"REQ-one-answer.{chapter} does not hold the moved sentence")

    def test_what_a_move_leaves_behind_stays(self):
        for source, label, fragment in KEPT:
            self.assertIn(_flat(fragment), _flat(_sections(FS / source)[label]), f"{source} {label}")

    def test_the_rule_has_its_principle_its_record_and_its_line(self):
        architecture = _sections(ARCHITECTURE)
        for chapter in ("principle", "principle.inventory"):
            self.assertTrue(chapter in architecture, f"AR-system.{chapter} is not declared")
        self.assertTrue(re.search(r"§REQ-one-answer(?![\w.-])", architecture["principle"]))
        self.assertTrue(DECISION.is_file(), "the decision record is not there")
        record = _sections(DECISION)
        for chapter in ("context", "decision", "alternatives", "consequences", "why"):
            self.assertTrue(chapter in record, f"the decision record has no {chapter} chapter")
        agents = AGENTS.read_text(encoding="utf-8")
        outside = re.sub(r"<!-- BEGIN (\w+) MANAGED BLOCK -->.*?<!-- END \1 MANAGED BLOCK -->",
                         "", agents, flags=re.S)
        self.assertIn("- One fact, one point, one implementation: settle a fact's defining point"
                      " in the spec before the code, have other specs cite it rather than restate"
                      " it, and derive it once in code that every other place reads"
                      " ([§AR-system.principle]", outside)


if __name__ == "__main__":
    unittest.main()
