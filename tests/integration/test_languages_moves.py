"""§AR-system.principle.inventory — how a source file's lines are read, the comment
block, the comment wrapper, docstrings, fences and comment prefixes, is defined at
one point, the `FS-languages` page, and the specs it was moved out of, `FS-show`,
`FS-inline-citation-style`, `FS-check`, `FS-config`, `FS-rules` and `FS-values`,
cite it rather than restate it. For each move this holds three things: the
destination chapter holds the moved sentence, the source section no longer does,
and the source keeps its heading, title and all, as a pointer that cites the
destination (§REQ-spec-section-names.permanence). It also holds the page's chapter
map, one Terms chapter and no numbered top-level chapter, and the two readings the
move must not settle ahead of the ruling that settles them: the extent of a run of
line comments is still stated both ways, and the closing `*/` is envelope only on a
heading's own line."""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
FS = REPO_ROOT / "docs" / "functional-spec"
LANGUAGES = FS / "FS-languages.md"
INDEX = FS / "README.md"
# The marker, spelled so that this file cites nothing the page has not declared yet.
CITATION = "\N{SECTION SIGN}FS-languages"
HEADING = re.compile(r"^(#{1,6}) (\S+?)(?::? )(.*)$")

CHAPTERS = (
    "terms",
    "comment-block",
    "comment-block.open", "comment-block.close", "comment-block.early-end",
    "comment-block.forms",
    "comment-wrapper", "comment-wrapper.headings",
    "docstring",
    "fence",
    "prefixes",
)

# (source file, source heading label, its title, which the pointer keeps so the
#  anchor survives, the chapters it must cite, and per chapter the fragments that
#  moved out of the source into that chapter)
MOVES = (
    ("FS-show.md", "2.3.1", 'What counts as the "comment block"', {"comment-block": ()}),
    ("FS-show.md", "2.3.1.1", "Find the open boundary", {"comment-block.open": (
        "Scan **backwards** from the declaration line over consecutive lines",
        "matches the same comment prefix family",
        "For Python triple-quoted docstrings: scan backward until the opening",
    )}),
    ("FS-show.md", "2.3.1.2", "Find the close boundary", {"comment-block.close": (
        "Scan **forwards** from the declaration line by the symmetric rules",
        "a declaration written as a one-line `/* … */` closes on its own line",
    )}),
    ("FS-show.md", "2.3.1.3", "Terminate early on another declaration", {"comment-block.early-end": (
        "if another declaration line of any ID is encountered, the block ends at the line before it",
    )}),
    ("FS-show.md", "2.3.2", "Stripping comment prefixes", {
        "comment-wrapper": (
            "The whitespace in front of the comment prefix belongs to the comment's envelope",
            "a leading ` * ` (with surrounding spaces) is removed if present",
            "Trailing comment-close tokens (`*/`) on their own line are dropped entirely",
            "Blank lines inside the block are preserved",
        ),
        "docstring": (
            "For Python docstrings, no prefix is stripped",
        ),
    }),
    ("FS-inline-citation-style.md", "1.2", "Comment blocks", {"comment-block.forms": (
        "a run of adjacent lines whose first non-whitespace token is the same line-comment prefix",
        "Adjacency is broken by any line that is not part of the same block",
        "An empty comment line — the prefix alone",
    )}),
    ("FS-check.md", "1.1.5", "Contexts read as neither prose nor code", {"fence": (
        "A fence opens with at most three leading spaces",
        "it closes only on a run of the **same character** at least as long as the opener",
        "A backtick opener cannot carry a backtick in its info string",
        "An unclosed fence runs to end of file",
    )}),
    ("FS-config.md", "3.5.14", "`comment_prefixes` compose with `extensions`", {"prefixes": (
        "The two lists compose",
        "`;` pairs with Lisp, Scheme, and Clojure extensions",
        "Any line whose first non-whitespace run is a configured prefix is eligible to host",
    )}),
    ("FS-rules.md", "5.1.1", "A chapter's display name is the label its author wrote",
     {"comment-wrapper.headings": (
         "sits behind the comment that carries it",
         "The envelope has a closing half too",
         "Only the C-family closer is envelope",
         "A Python docstring's heading is read as Markdown, and a Markdown heading is never trimmed",
     )}),
)
# A restatement with nothing to move: it leaves its source and the source cites the
# chapters that already say it.
DROPPED = (
    ("FS-values.md", "2.4.1", "Recognition in source doc-comments", ("comment-wrapper", "docstring"), (
        "`/* ... */` and `*` continuation lines used by Javadoc/JSDoc",
        "content inside enabled Python",
    )),
)
# A sibling page that named the old coordinate of the fence grammar names the new one.
REPOINTS = (
    ("FS-citations.md", "recognized.contexts", "fence"),
)

# What each source keeps: the fact that is its own rather than the language's.
KEPT = (
    ("FS-show.md", "2.3.1", "sits inside a comment or docstring"),
    ("FS-show.md", "2.3.2", "This is the property that makes"),
    ("FS-inline-citation-style.md", "1.2", "not a verdict on which blocks are sites"),
    ("FS-check.md", "1.1.5.1", "No line of the block, its opening and closing lines included, is a heading"),
    ("FS-config.md", "3.5", "comment_prefixes"),
    ("FS-rules.md", "5.1.1", "A `chapter` fact's `display_name` is the label the author wrote"),
    ("FS-rules.md", "5.1.1", "a presence rule naming `Terms` counts either"),
    ("FS-rules.md", "5.1.1", "section map gives that chapter is the same label"),
    ("FS-values.md", "2.4.1", "The language adds no alternate spelling"),
    ("FS-fmt.md", "2.3.1", "Track an open-quote state per"),
    ("FS-fmt.md", "2.3.1.1", "the line this scan reads is the docstring's **content**"),
    ("FS-fmt.md", "2.3.3", "Every pass stops at a Markdown fence"),
)

# Today `show` reads a run of line comments by prefix family and the scanner by the
# exact prefix; the ruling that settles it changes behaviour in a later step, so the
# page states both and adopts neither the ruling's wording nor a wider `*/` clause.
LINE_RUN_READINGS = ("the same comment prefix family", "the same line-comment prefix")
CLOSER_ON_HEADING = "A block comment's closing `*/` on the heading's own line"
NOT_YET = (
    "are separate comment blocks",
    "a trailing `*/` is part of the comment wrapper",
)


def _flat(text):
    return " ".join(text.split()).lower()


def _read(path):
    """Map each heading label to its title and the text under it, up to the next heading."""
    sections, label, title, lines, fence = {}, None, None, [], None
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
                sections.setdefault(label, (title, "\n".join(lines)))
            label, title, lines = match.group(2).rstrip(":"), match.group(3), []
        elif label is not None:
            lines.append(line)
    if label is not None:
        sections.setdefault(label, (title, "\n".join(lines)))
    return sections


def _sections(path):
    return {label: body for label, (_, body) in _read(path).items()}


def _cites(text, chapter):
    return re.search(re.escape(CITATION) + r"\." + re.escape(chapter) + r"(?![\w.-])", text)


def _chapter_and_children(declared, chapter):
    return " ".join(body for label, body in declared.items()
                    if label == chapter or label.startswith(chapter + "."))


class LanguagesMoveTests(unittest.TestCase):
    def languages(self):
        self.assertTrue(LANGUAGES.is_file(), f"{LANGUAGES.relative_to(REPO_ROOT)} is not there")
        return _sections(LANGUAGES)

    @unittest.expectedFailure
    def test_every_chapter_the_page_promises_is_declared(self):
        declared = self.languages()
        self.assertIn("FS-languages", declared)
        self.assertEqual([], [c for c in CHAPTERS if c not in declared], "chapters missing")
        self.assertRegex(INDEX.read_text(encoding="utf-8"), re.escape(f"[{CITATION}](FS-languages.md#"))

    @unittest.expectedFailure
    def test_the_page_has_named_chapters_and_one_terms_chapter(self):
        text = self.languages() and LANGUAGES.read_text(encoding="utf-8")
        tops = re.findall(r"^## (\S+?):? ", text, re.M)
        self.assertEqual([], [t for t in tops if t[0].isdigit()], "numbered top-level chapters")
        self.assertEqual(1, len(re.findall(r"^## terms: Terms$", text, re.M)), "Terms chapters")

    @unittest.expectedFailure
    def test_each_moved_sentence_lives_at_its_destination_only(self):
        declared = _sections(LANGUAGES) if LANGUAGES.is_file() else {}
        for source, label, title, chapters in MOVES:
            entry = _read(FS / source).get(label)
            self.assertIsNotNone(entry, f"{source} {label} is not there")
            self.assertEqual(title, entry[0], f"{source} {label} lost its title")
            for chapter, fragments in chapters.items():
                self.assertTrue(_cites(entry[1], chapter),
                                f"{source} {label} does not cite FS-languages.{chapter}")
                for fragment in fragments:
                    self.assertNotIn(_flat(fragment), _flat(entry[1]),
                                     f"{source} {label} still restates {chapter}")
                    self.assertIn(_flat(fragment), _flat(declared.get(chapter, "")),
                                  f"FS-languages.{chapter} does not hold the moved sentence")

    @unittest.expectedFailure
    def test_a_restatement_leaves_and_cites_the_chapters_that_say_it(self):
        for source, label, title, chapters, fragments in DROPPED:
            entry = _read(FS / source)[label]
            self.assertEqual(title, entry[0], f"{source} {label} lost its title")
            for chapter in chapters:
                self.assertTrue(_cites(entry[1], chapter),
                                f"{source} {label} does not cite FS-languages.{chapter}")
            for fragment in fragments:
                self.assertNotIn(_flat(fragment), _flat(entry[1]), f"{source} {label} still restates it")

    @unittest.expectedFailure
    def test_the_fence_grammar_is_cited_at_its_new_home(self):
        for source, label, chapter in REPOINTS:
            body = _sections(FS / source)[label]
            self.assertTrue(_cites(body, chapter), f"{source} {label} does not cite FS-languages.{chapter}")
            self.assertNotIn("FS-check.1.1.5]", body, f"{source} {label} still names the old coordinate")

    @unittest.expectedFailure
    def test_both_readings_of_a_line_comment_run_are_still_stated(self):
        declared = self.languages()
        block = _flat(_chapter_and_children(declared, "comment-block"))
        for reading in LINE_RUN_READINGS:
            self.assertIn(_flat(reading), block, f"comment-block no longer states {reading!r}")
        page = _flat(LANGUAGES.read_text(encoding="utf-8"))
        for wording in NOT_YET:
            self.assertNotIn(_flat(wording), page, "a pending ruling's wording arrived early")

    @unittest.expectedFailure
    def test_the_closing_comment_token_is_envelope_only_on_a_heading(self):
        self.languages()
        titled = _read(LANGUAGES)
        holders = [label for label, (_, body) in titled.items() if _flat(CLOSER_ON_HEADING) in _flat(body)]
        self.assertEqual(["comment-wrapper.headings"], holders, "the heading-line `*/` clause moved scope")
        self.assertIn("heading", titled["comment-wrapper.headings"][0].lower())

    def test_what_the_move_leaves_behind_stays(self):
        for source, label, fragment in KEPT:
            self.assertIn(_flat(fragment), _flat(_sections(FS / source)[label]), f"{source} {label}")


if __name__ == "__main__":
    unittest.main()
