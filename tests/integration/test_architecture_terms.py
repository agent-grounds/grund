"""§AR-system.5 — every page the architecture index links carries the shared
vocabulary's Terms chapter: for each `AR-` ID the index at
`docs/architecture/README.md` links, the file the link names — a Markdown page
under the folder, or the source file a source declaration lives in — has exactly
one `## terms: Terms` named section, that section's first non-blank line opens
the lean line that cites the shared groups, and the section comes after the
`placement` chapter, which keeps the opening position. `AR-system` is the index
page and is the one page exempt, as it is exempt from the placement chapter.
Modelled on `test_functional_spec_terms.py`, pointed at the architecture index,
and borrowing `_markdown_lines()` from `test_architecture_placement.py` so a
source declaration's comment markers are stripped before the headings are read."""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ARCHITECTURE = REPO_ROOT / "docs" / "architecture"
INDEX = ARCHITECTURE / "README.md"
SYSTEM = "AR-system"
ENTRY = re.compile(r"\[§(AR-[a-z][a-z0-9-]*)(?:\.[^\]]*)?\]\(([^)#]+)(?:#[^)]*)?\)")
COMMENT_PREFIX = re.compile(r"^\s*(?://[/!]?|#|\*)\s?")
PLACEMENT = re.compile(r"^## placement: \S")
TERMS = re.compile(r"^## terms: Terms\s*$")
HEADING = re.compile(r"^#{1,2} ")
MARKER = "§"
# `[fmt.cross_refs]` is on, so the stored lean line carries the linked form of the
# citation; accept both, because the assertion is about the lean line and not about
# whether `grund fmt --write` has run. The marker stays out of any literal a scan
# would read as a citation of a group: such a citation from tests/integration/ is
# coverage evidence for that leaf and turns its listed exception into a failure.
LEAN = re.compile(r"^Leans on \[?" + MARKER + r"FS-terms\.terms\.\d")
LEAN_SHAPE = "Leans on " + MARKER + "FS-terms.terms.<N>"


def _pages():
    """Every (id, file) the index links, once per ID, the index page left out."""
    pages = {}
    for match in ENTRY.finditer(INDEX.read_text(encoding="utf-8")):
        ident, target = match.groups()
        if ident != SYSTEM:
            pages.setdefault(ident, (INDEX.parent / target).resolve())
    return pages


def _markdown_lines(path):
    """The page's lines, with a source file's doc-comment markers stripped."""
    text = path.read_text(encoding="utf-8")
    if path.suffix == ".md":
        return text.splitlines()
    return [COMMENT_PREFIX.sub("", line) for line in text.splitlines()]


def _terms_chapters(lines):
    """Every `## terms: Terms` chapter body, each cut at the next H1 or H2."""
    chapters = []
    for start, line in enumerate(lines):
        if not TERMS.match(line):
            continue
        body = []
        for line in lines[start + 1 :]:
            if HEADING.match(line):
                break
            body.append(line)
        chapters.append(body)
    return chapters


def _heading_line(lines, pattern):
    """The 1-based line of the first heading matching the pattern, or None."""
    for number, line in enumerate(lines, 1):
        if pattern.match(line):
            return number
    return None


def _first_line(chapter):
    return next((line for line in chapter if line.strip()), "").strip()


class ArchitectureTermsTests(unittest.TestCase):
    def test_the_index_links_the_pages(self):
        self.assertGreaterEqual(len(_pages()), 8, "index entries not found; parser broken?")

    def test_every_page_carries_one_terms_chapter_that_leans_on_the_vocabulary(self):
        pages = _pages()
        problems = []
        for ident, path in sorted(pages.items()):
            if not path.is_file():
                problems.append(f"{ident}: the index links {path}, which is not a file")
                continue
            chapters = _terms_chapters(_markdown_lines(path))
            if not chapters:
                problems.append(f"{ident}: no `## terms: Terms` chapter in {path.name}")
            elif len(chapters) > 1:
                problems.append(f"{ident}: {len(chapters)} `## terms: Terms` chapters in {path.name}")
            elif not LEAN.match(_first_line(chapters[0])):
                problems.append(
                    f"{ident}: the Terms chapter does not open with `{LEAN_SHAPE}…`, "
                    f"it opens with {_first_line(chapters[0])!r}"
                )
        carried = len(pages) - len(problems)
        self.assertEqual(
            [],
            problems,
            f"{carried} of {len(pages)} architecture pages carry the Terms chapter:\n"
            + "\n".join(problems),
        )

    def test_the_terms_chapter_follows_the_placement_chapter(self):
        pages = _pages()
        problems = []
        for ident, path in sorted(pages.items()):
            if not path.is_file():
                problems.append(f"{ident}: the index links {path}, which is not a file")
                continue
            lines = _markdown_lines(path)
            placement = _heading_line(lines, PLACEMENT)
            terms = _heading_line(lines, TERMS)
            if placement is None:
                problems.append(f"{ident}: no `## placement:` chapter in {path.name}")
            elif terms is None:
                problems.append(f"{ident}: no `## terms: Terms` chapter to order in {path.name}")
            elif terms < placement:
                problems.append(
                    f"{ident}: the Terms chapter is at line {terms}, ahead of the placement "
                    f"chapter at line {placement}; placement keeps the opening position"
                )
        ordered = len(pages) - len(problems)
        self.assertEqual(
            [],
            problems,
            f"{ordered} of {len(pages)} architecture pages order the two chapters:\n"
            + "\n".join(problems),
        )


if __name__ == "__main__":
    unittest.main()
