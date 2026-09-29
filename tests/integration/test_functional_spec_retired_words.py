"""§FS-terms.terms — a word the shared vocabulary retires does not stand in
functional-spec prose. One row per swept word: for each, neither spelling of the
word occurs in any `docs/functional-spec/*.md` outside the senses the vocabulary's
lead excludes — inside a fenced block, a link target, a heading anchor, or a frozen
code name. The file claims nothing about a retired word it does not name, and a
word joins it as a row when its own sweep is required.
"""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
FUNCTIONAL_SPEC = REPO_ROOT / "docs" / "functional-spec"
# A fence opens and closes the excluded region; a link target carries the anchor a
# heading rename moves, and a backticked span carries the frozen code names the
# vocabulary keeps. Blanked rather than dropped, so a column is never invented.
FENCE = re.compile(r"^\s*(?:```|~~~)")
LINK_TARGET = re.compile(r"\]\([^)]*\)")
CODE_SPAN = re.compile(r"`[^`]*`")


class RetiredWord:
    """One swept word: the row that retires it, what displaces it, how it reads."""

    def __init__(self, word, row, displaced_by, pattern):
        self.word = word
        self.row = row
        self.displaced_by = displaced_by
        self.pattern = re.compile(pattern, re.IGNORECASE)


# The row ids are written without the marker on purpose: a literal citation from
# `tests/integration/` is coverage evidence, and these leaves are excepted there.
SWEPT_WORDS = (
    RetiredWord(
        word="point size",
        row="FS-terms.terms.1",
        displaced_by="coordinate-size",
        # `per-point` is an occurrence of *point*, which this row also retires but
        # this sweep does not carry; the compound is what is pinned here.
        pattern=r"point[-\s]sizes?",
    ),
    RetiredWord(
        word="ref",
        row="FS-terms.terms.2",
        displaced_by="citation",
        # The standalone word cannot be pinned: every bare `ref`/`refs` left in this
        # prose is a name the row keeps or a sense it never reached - the `refs`
        # command, the `--cross-refs` pass, a `FS-refs` or `GOAL-no-dangling-refs`
        # ID in link text, or git's own `ref`. The one compound where *ref* stands
        # for a citation is what is pinned here, and each anchor earns its place.
        # The leading guard keeps the `GOAL-no-dangling-refs` ID out, which the
        # `FS-check` lean line carries unbackticked. The trailing one keeps
        # *dangling reference* out: *reference* is swept on its own and this row
        # claims nothing about it. And the compound is what lets the row that
        # retires the word survive its own sweep - `FS-terms.terms.2` has to name
        # *ref* to retire it.
        pattern=r"(?<![-\w])dangling[-\s]refs?\b",
    ),
)


def _spec_files():
    return sorted(FUNCTIONAL_SPEC.glob("*.md"))


def _prose(path):
    """Every (line number, line) of a spec file, the excluded senses blanked out."""
    fenced = False
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if FENCE.match(line):
            fenced = not fenced
            continue
        if fenced:
            continue
        yield number, CODE_SPAN.sub("``", LINK_TARGET.sub("]()", line))


class FunctionalSpecRetiredWordsTests(unittest.TestCase):
    def test_the_spec_files_are_found(self):
        self.assertGreaterEqual(len(_spec_files()), 20, "spec files not found; parser broken?")

    def test_no_swept_retired_word_stands_in_functional_spec_prose(self):
        standing = []
        for retired in SWEPT_WORDS:
            for path in _spec_files():
                for number, line in _prose(path):
                    for hit in retired.pattern.finditer(line):
                        standing.append(
                            f"{path.relative_to(REPO_ROOT)}:{number}: {hit.group(0)!r} — "
                            f"say {retired.displaced_by} ({retired.row})"
                        )
        self.assertEqual(
            [],
            standing,
            f"{len(standing)} retired word(s) still standing in functional-spec prose:\n"
            + "\n".join(standing),
        )


if __name__ == "__main__":
    unittest.main()
