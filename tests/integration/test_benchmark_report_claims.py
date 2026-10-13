"""§AR-ci.5.2 — the benchmark job compares instruction counts against the pull
request's base branch and **fails the build** on growth past the harness's
limits: 5% `Ir` on the generated 10k-file fixture, 12.5% on every other
benchmark. The current copy a reader acts on — the report's current-reporting
paragraph — must name that gate, and none of it may still say the
limits are not enforced, because a reader who believes either the wrong way
reads a red bench job, or a green one, wrongly (§AR-benchmarks).

§AR-benchmarks.3 retires the last copy that called the limits "intended", and
§AR-ci.5.2 holds every limit at no less than five times the same-binary bound of
§AR-benchmarks.7.3, so jitter alone never crosses one.

§REQ-readme.evidence makes the report an archive: new measurements must write
elsewhere, and the archived rows stay byte for byte. Text detectors require
factual review as well.
"""

import hashlib
import json
import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]

GENERATOR = REPO_ROOT / "scripts/local-benchmark-report.py"
REPORT = REPO_ROOT / "docs/benchmarks.md"
README = REPO_ROOT / "README.md"
# §AR-ci.5.2: each scenario's relative limit is a row of the catalog, not a harness constant.
CATALOG = REPO_ROOT / "crates/grund-cli/benches/scenarios.json"
AR_BENCHMARKS = REPO_ROOT / "docs/architecture/AR-benchmarks.md"
# Every document that describes the benchmark limits to a reader.
BENCHMARK_DOCS = (
    REPORT, GENERATOR, AR_BENCHMARKS, REPO_ROOT / "docs/architecture/AR-ci.md",
    REPO_ROOT / "docs/decisions/architectural/DA-benchmark-instruction-counting.md",
)
INTENDED_LIMITS = re.compile(r"\bintended\b(?:\W+\w+){0,3}\W+limits?\b|"
                             r"\blimits?\b(?:\W+\w+){0,3}\W+intended\b", re.IGNORECASE)
BOUND = re.compile(r"The same-binary bound is \*\*([0-9.]+)%\*\* `Ir`")

# One limit stated for every benchmark, which §AR-ci.5.2 contradicts: 5% holds the
# 10k-file fixture alone.
FAILING_GATE_CLAIM = "fails when `Ir` grows by more than 5%"

# A sentence is what a reader takes as one statement. Read on the generated
# report rather than on the generator, where the prose is wrapped across
# adjacent string literals and no sentence boundary survives.
SENTENCE_SPLIT = re.compile(r"(?<=\.)\s")
FAIL_WORD = re.compile(r"\bfail(s|ed|ing|ure)?\b", re.IGNORECASE)

# A negated "fail" is the denial §AR-ci.5.2 retired: the gate is live.
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


def _instruction_gate_claims(text, denied=False):
    """Sentences about instruction counts that claim a gate, or with `denied`, deny one."""
    offenders = []
    for sentence in re.split(r"(?<=[.!?])\s+", text.replace("\n", " ")):
        if INSTRUCTION.search(sentence) and any(
            bool(NEGATED_GATE.search(sentence[:match.start()])) == denied
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


def _denies_a_failure(sentence):
    """Whether `sentence` says something does not fail."""
    return any(
        NEGATED.search(sentence[: match.start()])
        for match in FAIL_WORD.finditer(sentence)
    )


def _names_both_limits(sentence):
    return "5%" in sentence and "12.5%" in sentence


class TheBenchmarkComparisonGatesTheBuild(unittest.TestCase):
    def test_the_generator_does_not_state_one_limit_for_every_benchmark(self):
        self.assertNotIn(
            FAILING_GATE_CLAIM,
            GENERATOR.read_text(encoding="utf-8"),
            f"{GENERATOR.relative_to(REPO_ROOT)} states a 5% limit for every "
            "benchmark; §AR-ci.5.2 holds only the 10k-file fixture to 5%",
        )

    def test_the_report_does_not_state_one_limit_for_every_benchmark(self):
        self.assertNotIn(
            FAILING_GATE_CLAIM,
            REPORT.read_text(encoding="utf-8"),
            f"{REPORT.relative_to(REPO_ROOT)} states a 5% limit for every "
            "benchmark; §AR-ci.5.2 holds only the 10k-file fixture to 5%",
        )

    def test_no_sentence_about_ir_denies_the_build_failure(self):
        offenders = [
            sentence
            for sentence in _sentences_about_instruction_counts(
                REPORT.read_text(encoding="utf-8")
            )
            if _denies_a_failure(sentence)
        ]
        self.assertEqual(
            [],
            offenders,
            f"a sentence about `Ir` in {REPORT.relative_to(REPO_ROOT)} still "
            "says growth does not fail the build: " + " | ".join(offenders),
        )

    def test_the_report_names_both_limits_and_the_failure(self):
        sentences = [
            sentence
            for sentence in _sentences_about_instruction_counts(
                REPORT.read_text(encoding="utf-8")
            )
            if _names_both_limits(sentence) and FAIL_WORD.search(sentence)
            and "#52-regression-limits" in sentence
        ]
        self.assertTrue(
            sentences,
            f"{REPORT.relative_to(REPO_ROOT)} has no sentence that names the 5% and "
            "12.5% `Ir` limits, says growth past them fails, and links §AR-ci.5.2",
        )

    def test_public_copy_names_the_instruction_regression_gate(self):
        text = REPORT.read_text(encoding="utf-8")
        self.assertTrue(_instruction_gate_claims(text),
                        f"{REPORT.name} does not say instruction-count growth fails the build")
        denials = _instruction_gate_claims(text, denied=True)
        self.assertEqual([], denials, "stale denial of the gate: " + " | ".join(denials))

    def test_readme_retires_the_throughput_badge(self):
        text = README.read_text(encoding="utf-8")
        self.assertFalse(
            re.search(r"!\[[^\]]*(?:LoC/s|lines.*second|throughput)[^\]]*\]", text, re.I),
            "README still publishes the undated throughput badge",
        )

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


class TheLimitsAreEnforcedAndClearOfJitter(unittest.TestCase):
    """§AR-benchmarks.3, §AR-ci.5.2, §AR-benchmarks.7.3"""

    def test_no_benchmark_doc_calls_the_limits_intended(self):
        for path in BENCHMARK_DOCS:
            with self.subTest(doc=path.relative_to(REPO_ROOT).as_posix()):
                found = [m.group(0) for m in INTENDED_LIMITS.finditer(path.read_text(encoding="utf-8"))]
                self.assertEqual([], found, f"{path.name} still calls the enforced limits intended")

    def test_every_ir_limit_is_five_times_the_same_binary_bound(self):
        bound = BOUND.search(AR_BENCHMARKS.read_text(encoding="utf-8"))
        self.assertIsNotNone(bound, "§AR-benchmarks.7.3 states no same-binary bound")
        scenarios = json.loads(CATALOG.read_text(encoding="utf-8"))["scenarios"]
        self.assertTrue(scenarios, f"{CATALOG.name} lists no scenario")
        for row in scenarios:
            with self.subTest(scenario=row["name"]):
                self.assertGreaterEqual(float(row["limit_pct"]), 5 * float(bound.group(1)),
                                        f"{row['name']}'s limit is within five times the "
                                        "same-binary bound")

    def test_the_intended_detector_finds_the_retired_wording(self):
        self.assertTrue(INTENDED_LIMITS.search(
            "the intended limits and their current wiring are §AR-ci.5.2"))
        self.assertTrue(INTENDED_LIMITS.search("limits that were only intended"))
        self.assertFalse(INTENDED_LIMITS.search("growth past a limit fails the build"))

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

    def test_negated_gate_claims_and_unrelated_checks_are_not_claims(self):
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

    def test_negated_gate_claims_are_denials(self):
        for text in (
            "Instruction-count limits are not enforced.",
            "Growth in `Ir` does not yet fail the build.",
            "CI doesn't fail on `Ir` growth.",
        ):
            with self.subTest(text=text):
                self.assertTrue(_instruction_gate_claims(text, denied=True))
        self.assertEqual([], _instruction_gate_claims(
            "grund check fails on a dangling citation.", denied=True))

    def test_a_denial_of_the_build_failure_is_found(self):
        self.assertTrue(_denies_a_failure("Growth in `Ir` does not fail the build."))
        self.assertFalse(_denies_a_failure("Growth in `Ir` past 5% fails the build."))

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
