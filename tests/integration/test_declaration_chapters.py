"""§AR-system.principle.inventory — what a heading's rendered text is, where a
declaration's body starts and ends, what a section and the section map are, and what
cuts a lead are defined at one point each, chapters of the `FS-declarations` page, and
the command specs they were moved out of, `FS-show` and `FS-fmt`, cite them rather than
restate them.
For each move this holds three things: the destination chapter holds the moved
sentence, the source section no longer does, and the source cites the destination,
so a restatement that grows back fails here. It also holds the page's chapter map,
that what `show` prints and how `fmt` slugifies stay with those commands, that
`FS-show.2.5` stays where it is and is cited, that the page keeps one Terms chapter
and `FS-terms` gains no second gloss, and that each source spec ends smaller."""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
FS = REPO_ROOT / "docs" / "functional-spec"
DECLARATIONS = FS / "FS-declarations.md"
TERMS = FS / "FS-terms.md"
ONE_ANSWER = REPO_ROOT / "docs" / "requirements" / "REQ-one-answer.md"
# The marker, spelled so that this file cites nothing the page has not declared yet.
MARKER = "\N{SECTION SIGN}"
CITATION = MARKER + "FS-declarations"
HEADING = re.compile(r"^(#{1,6}) (\S+?)(?::? )(.*)$")

# The chapters the page already has, and the ones this move adds beside them.
CHAPTERS = (
    "terms", "line", "stubs", "checks",
    "heading", "heading.rendered-text",
    "body",
    "section", "section.map",
    "lead", "lead.section",
)

# (source file, source heading label, destination chapter it must cite,
#  fragments that moved out of it into that chapter)
MOVES = (
    ("FS-show.md", "2.1", "lead",
     ("the prose between the declaration heading and the first child citable section heading",
      "a plain heading remains prose and does not cut it")),
    ("FS-show.md", "2.1.5", "lead.section",
     ("the prose between that heading and the first *child* heading",)),
    ("FS-show.md", "2.1.2.1", "body",
     ("The span, not the most recently scanned declaration, owns the coordinate",
      "and a stub's single declaration line each end ownership")),
    ("FS-show.md", "2.1.2.1", "section.map",
     ("A coordinate enters the section map only when its heading lies inside that declaration's existing body span",
      "one after it is absent from the map")),
    ("FS-show.md", "2.1.3", "body",
     ("whether or not it declares an ID",)),
    ("FS-show.md", "2.2", "section",
     ("nested deeper headings included",)),
    ("FS-show.md", "2.2.3", "section.map",
     ("so none answers for a heading it rejects",)),
    ("FS-fmt.md", "6.2.2", "heading.rendered-text",
     ("the destination URL is not part of the text",
      "exactly as a Markdown renderer treats them")),
    ("FS-fmt.md", "6.2.4", "heading.rendered-text",
     ("the rendered text includes the name and colon",)),
)
# A source whose every definition moved is left a pointer: its heading and a line.
POINTERS = (("FS-show.md", "2.1.2.1"),)
POINTER_WORDS = 60

# What the move leaves where it is: what each `show` mode prints, how `fmt` names the
# cited heading and slugifies it, and FS-show.2.5, which this step does not move.
KEPT = (
    ("FS-show.md", "2.1", "The opening heading is omitted in `text` format and included in `md`"),
    ("FS-show.md", "2.1.3", "Every subsection and sub-subsection body is included"),
    ("FS-show.md", "2.1.5", "If the section opens directly with a sub-subsection, the output is just the section heading line"),
    ("FS-show.md", "2.2", "The flag determines how much of it is printed"),
    ("FS-show.md", "2.2.3", "emits none of the outside heading's body"),
    ("FS-show.md", "2.5", "is content, never structure"),
    ("FS-fmt.md", "6.2.1", "for a bare-ID citation it is the declaration's own heading"),
    ("FS-fmt.md", "6.2.2", "slugified per the configured renderer profile"),
    ("FS-fmt.md", "6.2.4", "produces `#goals-scope`"),
)
# Behaviour the page must not take with it: show's output form, fmt's slug profile,
# and the fence rule, which stays at FS-show.2.5.
NOT_MOVED = (
    "The opening heading is omitted",
    "slugified per the configured renderer profile",
    "is content, never structure",
)
# (chapter, point it cites rather than restates)
CITED = (
    ("heading", "FS-show.2.5"),
    ("section", "FS-config.3.3"),
    ("section.map", "REQ-one-answer.sections.map"),
)
# Bytes on main at 3613f4e9fd; every source the move takes text from ends below it.
# FS-fmt.md grew past its move baseline by one point of fmt's own, not a restatement:
# §FS-fmt.6.3.3 (grund#642).

# It grew again by the 217-byte clause of §FS-fmt.2.5.2.2: a directive inside a
# multi-line string literal is data (grund#702).

# Its lead grew by 404 LF bytes teaching existing link and eligible-lock maintenance
# via §FS-fmt.6.3 and §FS-section-lock.write.rules (grund#824).

# Each adds the bytes of the requirements chapter its spec gained since (#634: 5295
# and 4680), so the room a restatement could grow back into is what it was.

# FS-show.md grew by the 463 bytes of §FS-show.3.2.1's paragraph on where a wrap
# ends, which cites the wrap test rather than restating it (grund#655).

# FS-show.md grew by the 1006 bytes of `show --batch --glob`: §FS-show.1.9's paragraph
# and §FS-show.2.6.2's sentence on filtering the generation, which cite
# §FS-coordinate-globs.errors and §FS-show.2.6.3 rather than restating them (grund#503).

# FS-show.md grew 449 bytes in §FS-show.3.2.1 and FS-fmt.md 959 in §FS-fmt.6.4, each on a
# citation in a link its author wrote, citing §FS-citations.wrap.enclosing rather than
# restating it (grund#825).

# FS-fmt.md grew by the 3881 bytes of fmt's own refusal of a conflict block, in §FS-fmt.3
# and §FS-fmt.7.4 and the new §FS-fmt.7.4.3, citing §FS-declarations.checks.conflict-marker
# for the block and its message rather than restating them (grund#837).

# FS-fmt.md grew by the 223 bytes of the lock pass's place in the list of
# §FS-fmt.requirements.fixed-point, citing §FS-section-lock.write.same-run for which tree
# the pass judges rather than restating it (grund#847).

# FS-fmt.md grew by 162 bytes in §FS-fmt.1 and §FS-fmt.3.2, citing §FS-scan.path.none for
# what a run with no path covers rather than restating it (grund#854).

# FS-show.md grew 118 bytes in §FS-show.3.2.1 and FS-fmt.md 345 in §FS-fmt.6.4, citing
# §FS-citations.wrap.enclosing.scope and .images rather than restating them (grund#839).

# FS-show.md grew by 1498 bytes in §FS-show.requirements.authored-bytes, .2.2, .3.1.1,
# .3.1.2 and .3.2.2, each citing §FS-declarations.section.relative.shown.selected for a
# selected section's canonical heading rather than restating it (grund#1009).

# FS-show.md grew by 119 bytes in §FS-show.3.1.1, saying a selected section's text read opens
# with its heading line by citing §FS-show.2.2 rather than restating it (grund#1026).

# FS-fmt.md grew by 3697 bytes in §FS-fmt.3.5, .3.5.1, .3.5.2, .3.6, .3.7, .7.3 and
# .requirements.next-class: a dry-run row now names every kind its line receives and the
# write summary totals them, new behaviour no other point states (grund#1021).

# FS-show.md grew by 421 bytes in §FS-show.2.1.2.1, .2.2 and .3.1.3.1, saying the section
# map, a section read and a JSON anchor answer for front-matter coordinates by citing
# §FS-fields.front-matter.coordinates, .surfaces and §FS-output-shapes.4.4 (grund#1048).

# FS-show.md grew by 1228 bytes in §FS-show.3.5.1 and .3.5.2, saying an invalid-ID hint in a
# run that loaded no config names the project config below it, as §FS-check.2.2.2.5.1 and
# .5.2 choose it, in place of the default format (grund#1079).

# FS-fmt.md grew by 42 bytes in §FS-fmt.2.4, .2.4.1.2, .2.4.2, .2.4.3, .2.4.6 and .3.6: its
# shorthand illustrations take the <§> escape, so none is an off-format citation (grund#1073).
BASELINES = {"FS-show.md": 71777, "FS-fmt.md": 103210}


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
            label, lines = match.group(2).rstrip(":").rstrip("."), []
        elif label is not None:
            lines.append(line)
    if label is not None:
        sections.setdefault(label, "\n".join(lines))
    return sections


def _cites(text, target):
    return re.search(re.escape(MARKER + target) + r"(?![\w.-])", text)


class DeclarationChapterTests(unittest.TestCase):
    def test_every_chapter_the_page_promises_is_declared(self):
        declared = _sections(DECLARATIONS)
        self.assertEqual([], [c for c in CHAPTERS if c not in declared], "chapters missing")

    def test_the_page_keeps_one_terms_chapter_and_fs_terms_no_second_gloss(self):
        text = DECLARATIONS.read_text(encoding="utf-8")
        self.assertEqual(1, len(re.findall(r"^## terms: Terms$", text, re.M)), "Terms chapters")
        terms = _flat(TERMS.read_text(encoding="utf-8"))
        for _, _, chapter, fragments in MOVES:
            for fragment in fragments:
                self.assertNotIn(_flat(fragment), terms, f"FS-terms glosses {chapter} again")

    def test_each_moved_sentence_lives_at_its_destination_only(self):
        declared = _sections(DECLARATIONS)
        for source, label, chapter, fragments in MOVES:
            body = _sections(FS / source).get(label)
            self.assertIsNotNone(body, f"{source} {label} is not there")
            self.assertTrue(_cites(body, f"FS-declarations.{chapter}"),
                            f"{source} {label} does not cite {CITATION}.{chapter}")
            for fragment in fragments:
                self.assertNotIn(_flat(fragment), _flat(body), f"{source} {label} still restates {chapter}")
                self.assertIn(_flat(fragment), _flat(declared.get(chapter, "")),
                              f"FS-declarations.{chapter} does not hold the moved sentence")

    def test_a_source_left_with_nothing_is_a_pointer(self):
        for source, label in POINTERS:
            body = _sections(FS / source)[label]
            self.assertLessEqual(len(body.split()), POINTER_WORDS, f"{source} {label} is not a pointer")

    def test_the_page_cites_what_it_does_not_restate(self):
        declared = _sections(DECLARATIONS)
        for chapter, target in CITED:
            self.assertTrue(_cites(declared.get(chapter, ""), target),
                            f"FS-declarations.{chapter} does not cite {target}")
        page = _flat(DECLARATIONS.read_text(encoding="utf-8"))
        for fragment in NOT_MOVED:
            self.assertNotIn(_flat(fragment), page, f"FS-declarations states {fragment!r}")

    def test_the_requirement_cites_the_map_where_it_is_defined(self):
        body = _sections(ONE_ANSWER)["sections.map"]
        self.assertTrue(_cites(body, "FS-declarations.section.map"),
                        "REQ-one-answer.sections.map still cites the map inside show's spec")

    def test_every_source_spec_ends_smaller(self):
        for name, baseline in BASELINES.items():
            # Baselines are LF sizes; a Windows checkout reads CRLF.
            size = len((FS / name).read_bytes().replace(b"\r\n", b"\n"))
            self.assertLess(size, baseline, name)

    def test_what_the_move_leaves_behind_stays(self):
        for source, label, fragment in KEPT:
            self.assertIn(_flat(fragment), _flat(_sections(FS / source)[label]), f"{source} {label}")


if __name__ == "__main__":
    unittest.main()
