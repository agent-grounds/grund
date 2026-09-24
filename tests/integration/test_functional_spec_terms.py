"""§FS-terms.terms — every spec the functional-spec index links carries the
vocabulary's Terms chapter: for each `FS-` ID the index at
`docs/functional-spec/README.md` links, the file the link names has exactly one
`## terms: Terms` named section, and that section's first non-blank line opens
the lean line that cites the shared groups. `FS-terms` is the vocabulary itself
— its chapter opens with the one-definition rule rather than with a lean line —
and is the one page exempt, as the architecture index page is exempt from the
placement chapter this test is modelled on."""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
FUNCTIONAL_SPEC = REPO_ROOT / "docs" / "functional-spec"
INDEX = FUNCTIONAL_SPEC / "README.md"
VOCABULARY = "FS-terms"
ENTRY = re.compile(r"\[§(FS-[a-z][a-z0-9-]*)(?:\.[^\]]*)?\]\(([^)#]+)(?:#[^)]*)?\)")
TERMS = re.compile(r"^## terms: Terms\s*$")
MARKER = "§"
# `[fmt.cross_refs]` is on, so the stored lean line carries the linked form of the
# citation; accept both, because the assertion is about the lean line and not about
# whether `grund fmt --write` has run. The marker stays out of any literal a scan
# would read as a citation of a group: such a citation from tests/integration/ is
# coverage evidence for that leaf and turns its listed exception into a failure.
LEAN = re.compile(r"^Leans on \[?" + MARKER + r"FS-terms\.terms\.\d")
LEAN_SHAPE = "Leans on " + MARKER + "FS-terms.terms.<N>"
HEADING = re.compile(r"^#{1,2} ")


def _specs():
    """Every (id, file) the index links, once per ID, the vocabulary left out."""
    specs = {}
    for match in ENTRY.finditer(INDEX.read_text(encoding="utf-8")):
        ident, target = match.groups()
        if ident != VOCABULARY:
            specs.setdefault(ident, (INDEX.parent / target).resolve())
    return specs


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


def _first_line(chapter):
    return next((line for line in chapter if line.strip()), "").strip()


class FunctionalSpecTermsTests(unittest.TestCase):
    def test_the_index_links_the_specs(self):
        self.assertGreaterEqual(len(_specs()), 20, "index entries not found; parser broken?")

    def test_every_spec_carries_one_terms_chapter_that_leans_on_the_vocabulary(self):
        specs = _specs()
        problems = []
        for ident, path in sorted(specs.items()):
            if not path.is_file():
                problems.append(f"{ident}: the index links {path}, which is not a file")
                continue
            chapters = _terms_chapters(path.read_text(encoding="utf-8").splitlines())
            if not chapters:
                problems.append(f"{ident}: no `## terms: Terms` chapter in {path.name}")
            elif len(chapters) > 1:
                problems.append(f"{ident}: {len(chapters)} `## terms: Terms` chapters in {path.name}")
            elif not LEAN.match(_first_line(chapters[0])):
                problems.append(
                    f"{ident}: the Terms chapter does not open with `{LEAN_SHAPE}…`, "
                    f"it opens with {_first_line(chapters[0])!r}"
                )
        carried = len(specs) - len(problems)
        self.assertEqual(
            [],
            problems,
            f"{carried} of {len(specs)} specs carry the Terms chapter:\n" + "\n".join(problems),
        )


if __name__ == "__main__":
    unittest.main()
