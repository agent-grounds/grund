"""§AR-ci.5.2 — the benchmark job records an instruction-count comparison
against the pull request's base branch and does **not** fail the build on
growth: "the PR rerun passes no limit flag and the harness sets no
`RegressionConfig`". The generated report and the generator that writes it must
say that, because a reader who believes the gate is live stops looking for the
regression nobody blocked (§AR-benchmarks).

The report and its generator are checked together: `docs/benchmarks.md` is
written by `scripts/local-benchmark-report.py`, so a sentence corrected in the
tree alone is undone by the next regeneration.
"""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]

GENERATOR = REPO_ROOT / "scripts/local-benchmark-report.py"
REPORT = REPO_ROOT / "docs/benchmarks.md"

# The claim §AR-ci.5.2 contradicts, as the generator spells it today.
FAILING_GATE_CLAIM = "fails when `Ir` grows by more than 5%"

# A sentence is what a reader takes as one statement. Read on the generated
# report rather than on the generator, where the prose is wrapped across
# adjacent string literals and no sentence boundary survives.
SENTENCE_SPLIT = re.compile(r"(?<=\.)\s")
FAIL_WORD = re.compile(r"\bfail(s|ed|ing|ure)?\b", re.IGNORECASE)

# Saying the gate does *not* fail the build is the correction, not the
# claim, so a negated "fail" is what §AR-ci.5.2 asks the sentence to say.
NEGATED = re.compile(r"\b(not|never|no|without)\b(\s+\w+){0,2}\s*$", re.IGNORECASE)


def _sentences_about_instruction_counts(text):
    """Every sentence in `text` that speaks about the `Ir` measurement."""
    return [
        sentence.strip()
        for sentence in SENTENCE_SPLIT.split(text.replace("\n", " "))
        if "`Ir`" in sentence
    ]


def _promises_a_failure(sentence):
    """Whether `sentence` states that something fails, negations excluded."""
    return any(
        not NEGATED.search(sentence[: match.start()])
        for match in FAIL_WORD.finditer(sentence)
    )


class TheBenchmarkComparisonIsRecordedNotGating(unittest.TestCase):
    def test_the_generator_does_not_claim_a_failing_gate(self):
        self.assertNotIn(
            FAILING_GATE_CLAIM,
            GENERATOR.read_text(encoding="utf-8"),
            f"{GENERATOR.relative_to(REPO_ROOT)} still generates the claim that "
            "pull-request CI fails on `Ir` growth; §AR-ci.5.2 says the "
            "comparison is recorded and the limits are not wired up",
        )

    def test_the_report_does_not_claim_a_failing_gate(self):
        self.assertNotIn(
            FAILING_GATE_CLAIM,
            REPORT.read_text(encoding="utf-8"),
            f"{REPORT.relative_to(REPO_ROOT)} still says pull-request CI fails "
            "on `Ir` growth; §AR-ci.5.2 says it records the comparison",
        )

    def test_no_sentence_about_ir_promises_a_build_failure(self):
        offenders = [
            sentence
            for sentence in _sentences_about_instruction_counts(
                REPORT.read_text(encoding="utf-8")
            )
            if _promises_a_failure(sentence)
        ]
        self.assertEqual(
            [],
            offenders,
            f"a sentence about `Ir` in {REPORT.relative_to(REPO_ROOT)} still "
            "promises a build failure: " + " | ".join(offenders),
        )


if __name__ == "__main__":
    unittest.main()
