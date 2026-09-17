"""§AR-system.5 — every page the architecture index links opens with a
`placement` chapter that names its box in the system: for each `AR-` ID the
index at `docs/architecture/README.md` links, the file the link names — a
Markdown page under the folder, or the source file an inline declaration lives
in — has a `## placement:` named section, and that section cites `AR-system`.
The index page is the system itself and is the one page exempt."""

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
HEADING = re.compile(r"^#{1,2} ")


def _pages():
    """Every (id, file) the index links, once per ID, the system page left out."""
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


def _placement_chapter(lines):
    """The lines from the placement heading up to the next H1 or H2, or None."""
    for start, line in enumerate(lines):
        if PLACEMENT.match(line):
            body = []
            for line in lines[start + 1 :]:
                if HEADING.match(line):
                    break
                body.append(line)
            return body
    return None


class ArchitecturePlacementTests(unittest.TestCase):
    def test_the_index_links_the_pages(self):
        self.assertGreaterEqual(len(_pages()), 8, "index entries not found; parser broken?")

    def test_every_page_opens_with_a_placement_chapter_that_cites_the_system(self):
        problems = []
        for ident, path in sorted(_pages().items()):
            if not path.is_file():
                problems.append(f"{ident}: index links {path}, which is not a file")
                continue
            chapter = _placement_chapter(_markdown_lines(path))
            if chapter is None:
                problems.append(f"{ident}: no `## placement:` chapter in {path.name}")
            elif f"§{SYSTEM}" not in "\n".join(chapter):
                problems.append(f"{ident}: the placement chapter does not cite §{SYSTEM}")
        self.assertEqual([], problems, "\n".join(problems))


if __name__ == "__main__":
    unittest.main()
