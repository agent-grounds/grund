"""§AR-system.principle.inventory — what a citation is, and where its text is
never rewritten, is defined at one point, the `FS-citations` page, and the command specs
it was moved out of, `FS-check` and `FS-fmt`, cite it rather than restate it.
For each move this holds three things: the destination chapter holds the moved
sentence, the source section no longer does, and the source cites the
destination, so a restatement that grows back fails here. It also holds the
page's chapter map, the three-zone exemption `check` keeps, the cross-reference
exception `fmt` keeps, and the text its sibling moves leave where it is."""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
FS = REPO_ROOT / "docs" / "functional-spec"
CITATIONS = FS / "FS-citations.md"
INDEX = FS / "README.md"
# The marker, spelled so that this file cites nothing the page has not declared yet.
CITATION = "\N{SECTION SIGN}FS-citations"
HEADING = re.compile(r"^(#{1,6}) (\S+?)(?::? )(.*)$")

CHAPTERS = (
    "terms",
    "recognized",
    "recognized.off-grammar", "recognized.named-candidates",
    "recognized.string-literals", "recognized.string-literals.assigned-data",
    "recognized.link-destinations", "recognized.contexts",
    "recognized.value-bindings", "recognized.snapshots", "recognized.local-sections",
    "recognized.escapes", "recognized.marker-start", "recognized.globs",
    "shorthand",
    "shorthand.shape", "shorthand.marker", "shorthand.full-id-wins",
    "shorthand.edge", "shorthand.numeric-run", "shorthand.cli",
    "zones", "zones.link-destination", "zones.inline-code",
    "rewrite-exemption",
    "wrap", "wrap.shape", "wrap.recognized",
)

# (source file, source heading label, destination chapter it must cite,
#  fragments that moved out of it into that chapter)
MOVES = (
    ("FS-check.md", "1.1", "recognized",
     ("a citation is the marker followed by an ID",)),
    ("FS-check.md", "1.1.1", "recognized.off-grammar",
     ("participates in section and dangling checks, inbound counts",)),
    ("FS-check.md", "1.1.2", "recognized.named-candidates",
     ("the scanner consumes a whole ID-and-dot-tail candidate before deciding what it means",)),
    ("FS-check.md", "1.1.3", "recognized.string-literals",
     ("an ID-shaped substring inside runtime data does not raise a false dangling citation",)),
    ("FS-check.md", "1.1.3.1", "recognized.string-literals.assigned-data",
     ("opens an **assigned-data span**, not a docstring",)),
    ("FS-check.md", "1.1.4", "recognized.link-destinations",
     ("the parallel carve-out is the link destination",)),
    ("FS-check.md", "1.1.5", "recognized.contexts",
     ("this is what makes an example ID safe to write in documentation",)),
    ("FS-check.md", "1.1.5.1", "recognized.contexts",
     ("Citations inside the block stay live",)),
    ("FS-check.md", "1.1.6", "recognized.value-bindings",
     ("remains one citation",)),
    ("FS-check.md", "1.1.7", "recognized.snapshots",
     ("is not a special input",)),
    ("FS-check.md", "1.1.8", "recognized.local-sections",
     ("is a declaration-local section candidate",)),
    ("FS-check.md", "1.1.9", "recognized.escapes",
     ("begins immediately after the literal escape",)),
    ("FS-check.md", "1.1.10", "recognized.marker-start",
     ("establishes a citation start even when the marker ends in a word character",)),
    ("FS-check.md", "1.1.11", "recognized.globs",
     ("never ends a marked token early to leave a citation of its prefix",)),
    ("FS-check.md", "1.2", "shorthand",
     ("the number alone already identifies a declaration within its kind",)),
    ("FS-check.md", "1.2.1", "shorthand.shape",
     ("with the `{slug}` placeholder and one adjacent literal separator removed",)),
    ("FS-check.md", "1.2.2", "shorthand.marker",
     ("so the marker is what supplies the intent",)),
    ("FS-check.md", "1.2.3", "shorthand.full-id-wins",
     ("the shorthand pass only sees what is left",)),
    ("FS-check.md", "1.2.4", "shorthand.edge",
     ("the citation participates in the graph like any other",)),
    ("FS-check.md", "1.2.5", "shorthand.numeric-run",
     ("is a numeral in a run",)),
    ("FS-check.md", "1.2.6", "shorthand.cli",
     ("where nothing is persisted and the caller gets the declaration",)),
    ("FS-check.md", "3.13.1", "rewrite-exemption",
     ("where the text is legitimate as it stands",)),
    ("FS-check.md", "3.17.2", "wrap.shape",
     ("the citation wrapped as",)),
    ("FS-fmt.md", "2.3.4", "zones.link-destination",
     ("rewriting it would change the URL rather than the visible citation",)),
    ("FS-fmt.md", "2.3.5", "zones.inline-code",
     ("it closes at the next run of **exactly** *n* backticks",
      "One reading serves every caller, because they have to agree")),
    ("FS-fmt.md", "6.3.1", "wrap.recognized",
     ("its immediately-preceding character is `[` and its immediately-following text begins `](`",)),
)
# FS-fmt.2.3 keeps its list but its zone entries now name the zones chapter.
ZONE_LIST = ("FS-fmt.md", "2.3", "zones",
             ("where rewriting would change a literal command, path, or example",))

# What the move leaves where it is: the fence and raw-HTML grammar and the lexical
# string and fence rules, which a later step moves to a languages page (#589), the
# wrap form of 6.2, which another moves to the declarations page, and the
# writer-only entries of the never-rewritten list.
KEPT = (
    ("FS-check.md", "1.1.5", "it closes only on a run of the **same character** at least as long as the opener"),
    ("FS-check.md", "1.1.5.1", "No line of the block, its opening and closing lines included, is a heading"),
    ("FS-fmt.md", "2.3", "Declaration headings (the line that names the ID)"),
    ("FS-fmt.md", "2.3", "Everything in a **suppressed scope**"),
    ("FS-fmt.md", "2.3.1", "Track an open-quote state per"),
    ("FS-fmt.md", "2.3.1.1", "the line this scan reads is the docstring's **content**"),
    ("FS-fmt.md", "2.3.3", "Every pass stops at a Markdown fence"),
    ("FS-fmt.md", "6.2", "The citation text inside the brackets is preserved verbatim"),
)
# The grammar the sibling steps own must not arrive here ahead of them.
NOT_MOVED = (
    "at least as long as the opener",
    "Track an open-quote state per",
)
CROSS_REF_EXCEPTION = "the destination of a cross-reference wrap `fmt` itself maintains"


def _flat(text):
    return " ".join(text.split()).lower()


def _sections(path):
    """Map each heading label to the text under it, up to the next heading."""
    sections, label, lines, fence = {}, None, [], None
    for line in path.read_text(encoding="utf-8").splitlines():
        opener = re.match(r"^ {0,3}(`{3,}|~{3,})", line)
        if fence:
            if opener and opener.group(1)[0] == fence[0] and len(opener.group(1)) >= len(fence):
                fence = None
        elif opener:
            fence = opener.group(1)
        match = None if fence or opener else HEADING.match(line)
        if match:
            if label is not None:
                sections.setdefault(label, "\n".join(lines))
            label, lines = match.group(2).rstrip(":"), []
        elif label is not None:
            lines.append(line)
    if label is not None:
        sections.setdefault(label, "\n".join(lines))
    return sections


def _cites(text, chapter, children=False):
    tail = r"(?:\.[\w-]+)*" if children else ""
    return re.search(re.escape(CITATION) + r"\." + re.escape(chapter) + tail + r"(?![\w.-])", text)


class CitationsMoveTests(unittest.TestCase):
    def citations(self):
        self.assertTrue(CITATIONS.is_file(), f"{CITATIONS.relative_to(REPO_ROOT)} is not there")
        return _sections(CITATIONS)

    def test_every_chapter_the_page_promises_is_declared(self):
        declared = self.citations()
        self.assertIn("FS-citations", declared)
        self.assertEqual([], [c for c in CHAPTERS if c not in declared], "chapters missing")
        self.assertRegex(INDEX.read_text(encoding="utf-8"), re.escape(f"[{CITATION}](FS-citations.md#"))

    def test_the_page_has_named_chapters_and_one_terms_chapter(self):
        text = self.citations() and CITATIONS.read_text(encoding="utf-8")
        tops = re.findall(r"^## (\S+?):? ", text, re.M)
        self.assertEqual([], [t for t in tops if t[0].isdigit()], "numbered top-level chapters")
        self.assertEqual(1, len(re.findall(r"^## terms: Terms$", text, re.M)), "Terms chapters")

    def test_each_moved_sentence_lives_at_its_destination_only(self):
        declared = _sections(CITATIONS) if CITATIONS.is_file() else {}
        for source, label, chapter, fragments in MOVES + (ZONE_LIST,):
            body = _sections(FS / source).get(label)
            self.assertIsNotNone(body, f"{source} {label} is not there")
            self.assertTrue(_cites(body, chapter, children=label == "2.3"),
                            f"{source} {label} does not cite FS-citations.{chapter}")
            for fragment in fragments:
                self.assertNotIn(_flat(fragment), _flat(body), f"{source} {label} still restates {chapter}")
                self.assertIn(_flat(fragment), _flat(declared.get(chapter, "")),
                              f"FS-citations.{chapter} does not hold the moved sentence")

    def test_the_rewrite_exemption_names_exactly_three_zones(self):
        body = self.citations().get("rewrite-exemption", "")
        flat = _flat(body)
        for zone in ("inline code", "link destination", "string literal", "three"):
            self.assertIn(zone, flat, f"the exemption does not name {zone}")
        self.assertIn(_flat("A suppressed scope and an external file-symlink target are not among them"), flat)
        self.assertIsNone(_cites(body, "zones"), "the exemption cites the whole zones chapter")

    def test_the_cross_reference_exception_is_still_stated(self):
        fmt = _sections(FS / "FS-fmt.md")["2.3.4"]
        zone = self.citations().get("zones.link-destination", "")
        self.assertIn(_flat(CROSS_REF_EXCEPTION), _flat(fmt) + " " + _flat(zone))

    def test_what_the_move_leaves_behind_stays(self):
        for source, label, fragment in KEPT:
            self.assertIn(_flat(fragment), _flat(_sections(FS / source)[label]), f"{source} {label}")
        if CITATIONS.is_file():
            page = _flat(CITATIONS.read_text(encoding="utf-8"))
            for fragment in NOT_MOVED:
                self.assertNotIn(_flat(fragment), page, "a sibling step's text moved here")


if __name__ == "__main__":
    unittest.main()
