"""§FS-terms.senses.4 — the one reader of a spec's prose, so the exclusion that
makes a token a name rather than a use has a single implementation and a single
place to drift from. A fenced block is dropped, a link target and a backticked
span are blanked, and what is left is what a shared word has to stand in to count.

Two callers read through it: `test_functional_spec_retired_words.py`, for a word
the vocabulary retires, and `terms_vocabulary.py`, for a word it shares. Beside
them rather than in either, and named for what it holds rather than `test_*`, so
`unittest discover -p 'test_*.py'` does not collect it.
"""

import re


# A fence opens and closes the excluded region; a link target carries the anchor a
# heading rename moves, and a backticked span carries the frozen code names the
# vocabulary keeps. Blanked rather than dropped, so a column is never invented.
FENCE = re.compile(r"^\s*(?:```|~~~)")
LINK_TARGET = re.compile(r"\]\([^)]*\)")
CODE_SPAN = re.compile(r"`[^`]*`")


def prose(path):
    """Every (line number, line) of a Markdown file, the excluded senses blanked out."""
    fenced = False
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if FENCE.match(line):
            fenced = not fenced
            continue
        if fenced:
            continue
        yield number, CODE_SPAN.sub("``", LINK_TARGET.sub("]()", line))
