"""§AR-system.principle — the refusal rules have one home, §FS-rules.3.5 and
§FS-rules.8.1 moved out to the FS-rule-refusals declaration, and the shape a
refused input is printed in is stated once, at §FS-errors.2.5. For each move
this holds three things: the destination chapter holds the moved text, the
source section no longer does, and the source keeps its heading, title and
all, as a pointer that cites the destination
(§REQ-spec-section-names.permanence). It also holds the new page's chapter
map: one Terms chapter (§RULE-terms), no numbered top-level chapter
(§REQ-spec-section-names.shape), and both refusal chapters citing the shape
rather than restating it."""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
FS = REPO_ROOT / "docs" / "functional-spec"
SOURCE = FS / "FS-rules.md"
DESTINATION = FS / "FS-rule-refusals.md"
ERRORS = FS / "FS-errors.md"
INDEX = FS / "README.md"
HEADING = re.compile(r"^(#{1,6}) (\S+?)(?::? )(.*)$")

CHAPTERS = (
    "terms",
    "sentences",
    "sentences.whitespace", "sentences.named-sections", "sentences.chapter-path",
    "sentences.rewrite",
    "sentences.rewrite.replaced", "sentences.rewrite.recovered",
    "sentences.rewrite.reparsed", "sentences.rewrite.no-form",
    "sentences.rewrite.by-refusal", "sentences.rewrite.fs-only",
    "sentences.conjunction",
    "selectors",
)

# (FS-rules label, its title, which the pointer keeps so the anchor survives,
#  the chapter it points at, fragments that moved out of it into that chapter)
MOVES = (
    ("3.5", "Strict refusals", "sentences",
     ("No paraphrase or unlisted production is accepted.",
      '`modality "may not" is not accepted; accepted form: Each FS must not cite any AR.`')),
    ("3.5.1", "Presence-name whitespace refusal", "sentences.whitespace",
     ("A presence `NAME` containing whitespace anywhere",)),
    ("3.5.2", "A subject that needs named sections is answered with one they make valid",
     "sentences.named-sections",
     ("so a reader who follows the suggestion is not refused again",)),
    ("3.5.3", "A chapter path is refused for the component that failed",
     "sentences.chapter-path",
     ("opens with the reason that names what is wrong with the path",)),
    ("3.5.4", "An accepted form is the typed sentence with the failed part replaced",
     "sentences.rewrite",
     ("is the sentence the author typed with the production that failed rewritten",)),
    ("3.5.4.1", "Only the failed part is replaced", "sentences.rewrite.replaced",
     ("only the production that failed is spelled canonically",)),
    ("3.5.4.2", "Only what can be recovered is supplied", "sentences.rewrite.recovered",
     ("An object kind is never guessed.", "A chapter name is never invented")),
    ("3.5.4.3", "A form is parsed before it is offered", "sentences.rewrite.reparsed",
     ("A form is offered only once the parser accepts it.",)),
    ("3.5.4.4", "Where no form is offered", "sentences.rewrite.no-form",
     ("Where no form is offered, the refusal is its reason alone.",)),
    ("3.5.4.5", "The rewrite each refusal makes", "sentences.rewrite.by-refusal",
     ("| a per-target count without `times` |",)),
    ("3.5.4.6", "Exact rows where `FS` is the only kind", "sentences.rewrite.fs-only",
     ('`per-target counts must end in "times"; accepted form:'
      ' FS-login must cite each FS exactly 2 times.`',)),
    ("3.5.5", "Any clauses joined by `and` are refused as a conjunction",
     "sentences.conjunction",
     ("A sentence is a conjunction when it contains ` and ` immediately followed",)),
    ("8.1", "A refused selector is answered with a selector", "selectors",
     ("The suggestion is built from what was typed and from the configured kinds",
      "`named chapter subjects require [id] named_sections = true;"
      " accepted selector after enabling it: FS.requirements`")),
)
# What the move leaves behind on purpose: the `--only-rule` invocation error is
# not a refusal of an input, so FS-rules.8 keeps it, and its pointer to 8.1 now
# names the new home.
KEPT = (("8.", "`error: --only-rule requires\n--rule`"),)
RETARGETED = (("8.", "selectors"),)


def _flat(text):
    return " ".join(text.split()).lower()


def _headings(path):
    """Map each heading label to (its title, the text under it up to the next heading)."""
    sections, label, title, lines = {}, None, None, []
    fenced = False
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith("```"):
            fenced = not fenced
        match = None if fenced else HEADING.match(line)
        if match:
            if label is not None:
                sections.setdefault(label, (title, "\n".join(lines)))
            label, title, lines = match.group(2).rstrip(":"), match.group(3), []
        elif label is not None:
            lines.append(line)
    if label is not None:
        sections.setdefault(label, (title, "\n".join(lines)))
    return sections


def _body(sections, label):
    return sections.get(label, (None, ""))[1]


def _cites(text, target):
    return re.search(r"§" + re.escape(target) + r"(?![\w.-])", text)


class RuleRefusalsMoveTests(unittest.TestCase):
    def destination(self):
        self.assertTrue(DESTINATION.is_file(),
                        f"{DESTINATION.relative_to(REPO_ROOT)} is not there")
        return _headings(DESTINATION)

    def test_every_chapter_the_new_page_promises_is_declared(self):
        declared = self.destination()
        self.assertIn("FS-rule-refusals", declared)
        self.assertEqual([], [c for c in CHAPTERS if c not in declared], "chapters missing")

    def test_the_new_page_has_one_terms_chapter_and_no_numbered_chapter(self):
        self.destination()
        top = re.findall(r"^## (\S+?):? ", DESTINATION.read_text(encoding="utf-8"), re.M)
        self.assertEqual(1, top.count("terms"), "FS-rule-refusals needs exactly one Terms chapter")
        self.assertEqual([], [label for label in top if label[0].isdigit()],
                         "a new declaration has no numbered top-level chapter")

    def test_each_moved_fragment_lives_at_its_destination_only(self):
        declared = _headings(DESTINATION) if DESTINATION.is_file() else {}
        source = _headings(SOURCE)
        for label, title, chapter, fragments in MOVES:
            self.assertIn(label, source, f"FS-rules.{label} no longer resolves")
            for fragment in fragments:
                self.assertFalse(_flat(fragment) in _flat(_body(source, label)),
                                 f"FS-rules.{label} still holds what moved to {chapter}")
                self.assertTrue(_flat(fragment) in _flat(_body(declared, chapter)),
                                f"FS-rule-refusals.{chapter} does not hold the moved text")

    def test_each_old_coordinate_is_a_pointer_that_keeps_its_title(self):
        source = _headings(SOURCE)
        for label, title, chapter, _ in MOVES:
            self.assertIn(label, source, f"FS-rules.{label} no longer resolves")
            kept, body = source[label]
            self.assertEqual(title, kept, f"FS-rules.{label} changed its title and its anchor")
            self.assertTrue(_cites(body, f"FS-rule-refusals.{chapter}"),
                            f"FS-rules.{label} does not point at FS-rule-refusals.{chapter}")

    def test_what_the_move_leaves_behind_stays_and_points_at_the_new_home(self):
        source = _headings(SOURCE)
        for label, fragment in KEPT:
            self.assertIn(_flat(fragment), _flat(_body(source, label)), f"FS-rules.{label}")
        for label, chapter in RETARGETED:
            self.assertTrue(_cites(_body(source, label), f"FS-rule-refusals.{chapter}"),
                            f"FS-rules.{label} does not point at FS-rule-refusals.{chapter}")

    def test_the_refused_input_shape_is_stated_once(self):
        errors = _headings(ERRORS)
        self.assertIn("2.5", errors, "FS-errors.2.5 is not declared")
        shape = _body(errors, "2.5")
        self.assertIn("error: <reason>; accepted <form|selector|glob>: <input>", shape)
        for chapter in ("sentences", "selectors"):
            self.assertTrue(_cites(shape, f"FS-rule-refusals.{chapter}"),
                            f"FS-errors.2.5 does not name FS-rule-refusals.{chapter}")
        declared = self.destination()
        for chapter in ("sentences", "selectors"):
            self.assertTrue(_cites(_body(declared, chapter), "FS-errors.2.5"),
                            f"FS-rule-refusals.{chapter} does not cite FS-errors.2.5")
        self.assertNotIn("error: <reason>; accepted selector: <selector>",
                         DESTINATION.read_text(encoding="utf-8"),
                         "the selector refusal restates the line FS-errors.2.5 states")
        rules = _headings(SOURCE)
        self.assertNotIn("error: <reason>; accepted form:", SOURCE.read_text(encoding="utf-8"),
                         "FS-rules restates the line FS-errors.2.5 states")
        for label in ("4.", "7.1"):
            self.assertTrue(_cites(_body(rules, label), "FS-errors.2.5"),
                            f"FS-rules.{label.rstrip('.')} does not cite FS-errors.2.5 for the shape")

    def test_the_new_page_is_indexed(self):
        self.assertTrue(re.search(r"\(FS-rule-refusals\.md[#)]", INDEX.read_text(encoding="utf-8")),
                        "docs/functional-spec/README.md does not index FS-rule-refusals")


if __name__ == "__main__":
    unittest.main()
