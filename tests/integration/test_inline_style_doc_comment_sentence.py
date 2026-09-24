"""§FS-inline-citation-style.5.4 — the sentence that closes every rendered
managed block is the one the specification quotes, verbatim. The specification
is the only copy: the renderer holds it as a literal and every `grund init`
block in every repository carries it, so a paraphrase teaches one rule while
the gate enforces another (§FS-inline-citation-style.1.1).

Three copies are compared because they drift independently: the quote in the
specification, the literal the templates component renders (§AR-system.2.11),
and this repository's own block in `AGENTS.md`, which `grund check` does not
byte-compare — only `grund init --check` does (§REQ-agents-md.1).
"""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]

SPEC = REPO_ROOT / "docs/functional-spec/FS-inline-citation-style.md"
RENDERER = REPO_ROOT / "crates/grund-core/src/templates/agents_block.rs"
AGENTS = REPO_ROOT / "AGENTS.md"

SECTION = "### 5.4 The doc-comment sentence"

# The section states the sentence as one double-backtick span on a line of its
# own, because the sentence itself contains single backticks.
QUOTE = re.compile(r"^``(?P<sentence>.+)``$")

DOC_COMMENT_SENTENCE = re.compile(
    r'^const DOC_COMMENT_SENTENCE: &str = "(?P<value>.*)";$', re.MULTILINE
)


def _specified_sentence():
    """The sentence §FS-inline-citation-style.5.4 quotes."""
    lines = SPEC.read_text(encoding="utf-8").splitlines()
    start = lines.index(SECTION)
    for line in lines[start + 1 :]:
        if line.startswith("### "):
            break
        quoted = QUOTE.match(line)
        if quoted:
            return quoted.group("sentence")
    raise AssertionError(f"{SECTION} quotes no sentence in {SPEC}")


class TheRenderedSentenceIsTheSpecifiedOne(unittest.TestCase):
    def setUp(self):
        self.sentence = _specified_sentence()

    def test_the_renderer_holds_the_specified_sentence(self):
        literal = DOC_COMMENT_SENTENCE.search(RENDERER.read_text(encoding="utf-8"))
        self.assertIsNotNone(
            literal, f"no DOC_COMMENT_SENTENCE literal in {RENDERER}"
        )
        self.assertEqual(
            f" {self.sentence}",
            literal.group("value"),
            "DOC_COMMENT_SENTENCE is not §FS-inline-citation-style.5.4's "
            "sentence, so every rendered block teaches a rule the "
            "specification does not state",
        )

    def test_this_repositorys_block_carries_the_specified_sentence(self):
        self.assertTrue(
            self.sentence in AGENTS.read_text(encoding="utf-8"),
            "AGENTS.md does not carry §FS-inline-citation-style.5.4's "
            "sentence; `grund init` rewrites the block",
        )


if __name__ == "__main__":
    unittest.main()
