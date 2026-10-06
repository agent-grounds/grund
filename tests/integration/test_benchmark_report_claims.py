"""§AR-ci.5.2 — the benchmark job records an instruction-count comparison
against the pull request's base branch and does **not** fail the build on
growth: "the PR rerun passes no limit flag and the harness sets no
`RegressionConfig`". The generated report and the generator that writes it must
say that, because a reader who believes the gate is live stops looking for the
regression nobody blocked (§AR-benchmarks).

§REQ-readme.evidence makes the report an archive: new measurements must write
elsewhere. The existing generator guard remains, but this correction does not
change the generator. Text detectors require factual review as well.
"""

import hashlib
import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]

GENERATOR = REPO_ROOT / "scripts/local-benchmark-report.py"
REPORT = REPO_ROOT / "docs/benchmarks.md"
README = REPO_ROOT / "README.md"

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

# Match positive assertions independently of a later sentence disclaiming them.
# A release-blocking meter followed by "limits are not wired" still contradicts itself.
INSTRUCTION = re.compile(r"instruction[- ]count|Callgrind|`Ir`", re.IGNORECASE)
GATE = re.compile(
    r"release[- ]blocking|\bfail(?:s|ed|ing|ure)?\b|"
    r"\b(?:enforces?|enforced|blocks?|blocking)\b|\b(?:active|enforced) regression gate\b",
    re.IGNORECASE,
)
NEGATED_GATE = re.compile(
    r"\b(?:not|never|no|without)\b(?:\s+[\w`-]+){0,3}\s*$|"
    r"\b(?:doesn't|isn't|don't)\s*$",
    re.IGNORECASE,
)


def _instruction_gate_claims(text):
    offenders = []
    for sentence in re.split(r"(?<=[.!?])\s+", text.replace("\n", " ")):
        if INSTRUCTION.search(sentence) and any(
            not NEGATED_GATE.search(sentence[:match.start()])
            for match in GATE.finditer(sentence)
        ):
            offenders.append(sentence.strip())
    return offenders


def _missing_evidence_links(text, targets):
    """Markdown destinations only; a bare filename is not a reader path."""
    destinations = re.findall(r"(?:\]\(|\]:\s*)([^\s)]+)", text)
    return [target for target in targets if not any(
        destination.split("#", 1)[0].endswith(target) for destination in destinations
    )]


def _archival_measurement_digest(text):
    """Pin measured cells and raw samples, permitting table whitespace changes."""
    rows = []
    for line in text.splitlines():
        if line.startswith("|"):
            cells = [cell.strip() for cell in line.strip("|").split("|")]
            if not all(re.fullmatch(r":?-+:?", cell) for cell in cells):
                rows.append("|".join(cells))
        elif re.match(r"^- `(?:grund check|grund fmt --check|lychee)`:", line):
            rows.append(line)
    return hashlib.sha256("\n".join(rows).encode()).hexdigest()


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

    def test_public_copy_has_no_active_instruction_regression_gate(self):
        for path in (REPORT, README):
            with self.subTest(path=path.name):
                offenders = _instruction_gate_claims(path.read_text(encoding="utf-8"))
                self.assertEqual([], offenders, "active instruction gate claim: " + " | ".join(offenders))

    def test_readme_retires_the_throughput_badge(self):
        text = README.read_text(encoding="utf-8")
        self.assertFalse(
            re.search(r"!\[[^\]]*(?:LoC/s|lines.*second|throughput)[^\]]*\]", text, re.I),
            "README still publishes the undated throughput badge",
        )

    def test_readme_links_to_performance_evidence_and_current_reporting(self):
        missing = _missing_evidence_links(README.read_text(encoding="utf-8"), (
            "docs/benchmarks.md", "docs/architecture/AR-ci.md",
        ))
        self.assertEqual([], missing, "README missing performance paths: " + ", ".join(missing))

    def test_report_separates_historical_measurements_from_current_reporting(self):
        text = REPORT.read_text(encoding="utf-8")
        checks = {
            "dated local wall-clock snapshot": r"2026-05-20",
            "historical instruction snapshot": r"(?:historical|archiv\w*)[^.\n]*instruction|instruction[^.\n]*(?:historical|archiv\w*)",
            "current generated fixtures": r"generated.{0,30}fixtures?",
            "base-branch comparison": r"base branch",
            "workload-cost proxy": r"\bproxy\b",
            "binary comparability": r"\bbinar(?:y|ies)\b",
            "build comparability": r"\bbuild\w*\b",
            "PGO comparability": r"\bPGO\b",
            "different workloads": r"different.{0,30}workloads?|unlike.{0,30}workloads?|not.{0,30}(?:equivalent|interchangeable)",
        }
        for label, pattern in checks.items():
            with self.subTest(evidence=label):
                self.assertTrue(re.search(pattern, text, re.I | re.S), f"report missing {label}")
        self.assertEqual([], _missing_evidence_links(text, (
            "architecture/AR-benchmarks.md", "architecture/AR-ci.md",
        )))

    def test_future_measurements_do_not_overwrite_the_archive(self):
        text = REPORT.read_text(encoding="utf-8")
        self.assertFalse(
            re.search(r"--out\s+docs/benchmarks\.md", text),
            "future measurement recipe still overwrites docs/benchmarks.md",
        )

    def test_archival_measurements_and_provenance_are_preserved(self):
        # All measured tables, provenance cells and warm samples at df506acfae.
        self.assertEqual(
            "f9dbeb07560f3b5c4c3a524c2884319d7fa2d25c0f54ad9996bb74e4ccd88174",
            _archival_measurement_digest(REPORT.read_text(encoding="utf-8")),
            "historical measurement/provenance cells or raw samples changed",
        )


class PerformanceDetectorProbes(unittest.TestCase):
    def test_positive_instruction_gate_claims_are_rejected(self):
        for text in (
            "The release-blocking meter is Callgrind instruction count.",
            "CI fails when `Ir` grows by more than 5%.",
            "Instruction-count regression limits are enforced.",
            "Instruction count has an active regression gate.",
            "Instruction count blocks the release. Limits are not wired up.",
        ):
            with self.subTest(text=text):
                self.assertTrue(_instruction_gate_claims(text))

    def test_negated_gate_claims_and_unrelated_checks_are_allowed(self):
        for text in (
            "Instruction count is not a release-blocking meter.",
            "Instruction-count limits are not enforced.",
            "Growth in `Ir` does not yet fail the build.",
            "Callgrind never blocks a release.",
            "CI doesn't fail on `Ir` growth.",
            "Instruction count does not have an active regression gate.",
            "grund check fails on a dangling citation.",
        ):
            with self.subTest(text=text):
                self.assertEqual([], _instruction_gate_claims(text))

    def test_missing_links_are_detected_including_bare_paths(self):
        target = "docs/benchmarks.md"
        for text in ("", target, "[report](docs/other.md)"):
            with self.subTest(text=text):
                self.assertEqual([target], _missing_evidence_links(text, (target,)))
        for text in (
            "[report](docs/benchmarks.md#results)",
            "[report]: https://github.com/agent-grounds/grund/blob/main/docs/benchmarks.md",
        ):
            with self.subTest(text=text):
                self.assertEqual([], _missing_evidence_links(text, (target,)))


if __name__ == "__main__":
    unittest.main()
