"""§GOAL-configurable.2 — what a project's committed `grund.toml` may move, and
what is frozen for every project and every install alike.

Three classes, doing three different jobs.

`FrozenListScopeAgreement` is the reproducer of agent-grounds/grund#294 ported
into this suite: the frozen list — severity, exit codes, report ordering — has
to be stated at one scope, not scoped to install-local choices in
§GOAL-configurable.2 and unqualified in §GOAL-friendliness-first.2 and
§FS-non-goals.9 at the same time. It is necessary and not sufficient: it asserts
on wording, so a rewording that decided nothing would clear it.

`ShippedCommittedLevers` derives the shipped verdict-moving committed settings
by *running* them rather than by reading about them — one fixture repository per
setting, `grund check` before and after the one-word committed edit — and holds
each run to the frozen vocabulary: the two severities, the fixed exit-code
mapping of §FS-cli.5, and the bytewise `(path, line, message)` report order of
§FS-errors.4.1. It is green today and is meant to be. It is not a red-to-green
pin but the invariant guard beside one: no repair of the goal may turn a shipped
committed setting into a specification violation.

`GoalAdmitsEveryShippedLever` holds the goal to what that half found. The goal
does not name keys (§GOAL-configurable.1), so "admits" is read through the three
clauses of the ruling — the trio frozen unqualified, the reach test stated
separately about install-local state, and the citation of the authoring
procedure — plus the decision record that carries the admissibility test, which
must name every lever key the first half exercised. It is red on every clause
until the change lands.
"""

from __future__ import annotations

import functools
import json
import os
import re
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
CASES = REPO_ROOT / "tests" / "e2e" / "cases"
DECISIONS = REPO_ROOT / "docs" / "decisions" / "functional"
RECORD = DECISIONS / "DF-verdict-vocabulary-freeze.md"
DECISIONS_INDEX = DECISIONS / "README.md"
UNMARKED_HEADINGS = DECISIONS / "DF-unmarked-markdown-headings.md"
EXE = "grund.exe" if os.name == "nt" else "grund"

# The frozen vocabulary itself, as the three points that own it state it.
SEVERITIES = {"error", "warning"}


# --------------------------------------------------------------------------
# Reading points out of the tree, and running checks over fixture repositories.
# --------------------------------------------------------------------------


@functools.lru_cache(maxsize=None)
def _grund() -> str:
    """The binary under test, built from this checkout once per process.

    It is built rather than found. CI restores `target` from a prefix-keyed
    cache and runs this suite before the workspace build, so a
    `target/*/grund` that exists may have been built at an older commit, and
    these tests would then read this tree's `grund.toml` with a binary that
    does not know its keys. `-p grund` builds only what is needed, and the
    later workspace build reuses it.
    """
    target = Path(os.environ.get("CARGO_TARGET_DIR") or REPO_ROOT / "target")
    cargo = os.environ.get("CARGO", "cargo")
    subprocess.run([cargo, "build", "-p", "grund", "--locked"], cwd=REPO_ROOT, check=True)
    built = target / "debug" / EXE
    if not built.is_file():
        raise AssertionError(f"no grund binary at {built} after cargo build -p grund")
    return str(built)


def _point(point_id: str) -> str:
    """`grund <ID> --full`, the way the reproducer fetches a point."""
    result = subprocess.run(
        [_grund(), point_id, "--full"],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    if result.returncode != 0 or not result.stdout.strip():
        raise AssertionError(f"grund {point_id} --full exited {result.returncode}: {result.stderr}")
    return result.stdout


def _check(fixture: Path):
    """`grund check . --format json` in `fixture`: (exit code, findings)."""
    result = subprocess.run(
        [_grund(), "check", ".", "--format", "json"],
        cwd=fixture,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    if result.returncode not in (0, 1):
        raise AssertionError(f"grund check exited {result.returncode}: {result.stderr}")
    findings = [json.loads(line) for line in result.stdout.splitlines() if line.strip()]
    return result.returncode, findings


def _sort_key(finding):
    """§FS-errors.4.1 — JSON findings keep a global bytewise (path, line, message) order."""
    return (
        finding["path"].encode("utf-8"),
        finding["line"] if finding["line"] is not None else -1,
        finding["message"].encode("utf-8"),
    )


def _locations(findings):
    return [(finding["path"], finding["line"]) for finding in findings]


# --------------------------------------------------------------------------
# Sentence predicates. The first three are the reproducer's, ported verbatim so
# the ported test is the same gate; `_names_trio` below is the tolerant one the
# sufficient test needs, because the repair rewords the sentence it reads.
# --------------------------------------------------------------------------

_SEVERITY = re.compile(r"severit", re.I)
_EXIT_CODE = re.compile(r"exit[ -]code", re.I)
_REPORT_ORDER = re.compile(r"report[ -]order|ordering of the report|order(?:ing)? of the report", re.I)
_INSTALL_LOCAL = re.compile(
    r"install-local|machine-local|two (?:correctly-configured )?installs|same configuration", re.I
)
_LINK = re.compile(r"\[([^\]]*)\]\([^)]*\)")


def _sentences(body: str):
    text = " ".join(line for line in body.splitlines() if not line.startswith("#"))
    return [sentence for sentence in text.replace(". ", ".\n").split("\n") if sentence.strip()]


def _names_frozen_list(sentence: str) -> bool:
    return bool(
        _SEVERITY.search(sentence) and _EXIT_CODE.search(sentence) and _REPORT_ORDER.search(sentence)
    )


def _install_local_scoped(sentence: str) -> bool:
    return bool(_INSTALL_LOCAL.search(sentence))


def _normalize(text: str) -> str:
    """Link text without its target, no emphasis, no apostrophes, one space, lowercase."""
    text = _LINK.sub(r"\1", text)
    for noise in ("*", "`", "’", "'"):
        text = text.replace(noise, "")
    return re.sub(r"\s+", " ", text).strip().lower()


def _names_trio(sentence: str) -> bool:
    """The frozen trio named in one sentence, however the sentence phrases it."""
    normalized = _normalize(sentence)
    return bool(
        "severit" in normalized
        and re.search(r"exit[ -]code", normalized)
        and re.search(r"\border", normalized)
    )


# --------------------------------------------------------------------------
# The reproducer, ported.
# --------------------------------------------------------------------------


class FrozenListScopeAgreement(unittest.TestCase):
    """runtime/repro/…ticket.triage/run.sh, in this project's framework.

    Exit 1 there is a failure here: the defect stands while every sentence
    in the goal that names the frozen list is install-local scoped and some
    standing point still freezes that same list for every project.
    """

    STANDING = ("GOAL-friendliness-first.2", "FS-non-goals.9")

    def test_the_frozen_list_is_stated_at_one_scope(self):
        goal = _sentences(_point("GOAL-configurable.2"))
        frozen = [sentence for sentence in goal if _names_frozen_list(sentence)]
        scoped = [sentence for sentence in frozen if _install_local_scoped(sentence)]

        elsewhere = []
        for point_id in self.STANDING:
            for sentence in _sentences(_point(point_id)):
                if _names_frozen_list(sentence) and not _install_local_scoped(sentence):
                    elsewhere.append(f"{point_id}: {sentence.strip()}")

        goal_is_install_local_only = bool(frozen) and len(scoped) == len(frozen)
        self.assertFalse(
            goal_is_install_local_only and elsewhere,
            "GOAL-configurable.2 scopes severity, exit codes and report ordering to "
            "install-local choices while the same list is frozen for every project, "
            "unqualified, by:\n  " + "\n  ".join(elsewhere) + "\nGOAL-configurable.2 says:\n  "
            + "\n  ".join(sentence.strip() for sentence in scoped),
        )


# --------------------------------------------------------------------------
# Half one: the shipped committed levers, derived by running them.
# --------------------------------------------------------------------------

# Each lever is one fixture repository and one committed word. `case` names the
# e2e repo it is grown from, `before`/`after` are the exact substitution, and
# `key` is the `grund.toml` key the lever is — the same list the decision record
# has to decide, which is what makes the two halves agree.
LEVERS = (
    {
        "name": "citation direction standing",
        "key": "[citations.FS]",
        "case": "check-citation-levels-default",
        "before": 'should = ["GOAL"]',
        "after": 'must = ["GOAL"]',
    },
    {
        "name": "persisted number-only shorthand",
        "key": "shorthand",
        "case": "check-shorthand-citation-resolvable",
        "before": 'shorthand = "canonical"',
        "after": 'shorthand = "accepted"',
    },
    {
        "name": "inline note layout channel",
        "key": "inline_note_layout_check",
        "case": "inline-note-layout-error",
        "before": 'inline_note_layout_check = "error"',
        "after": 'inline_note_layout_check = "warn"',
    },
    {
        "name": "section heading level channel",
        "key": "section_heading_levels",
        "case": "section-heading-level-warn",
        "before": 'section_heading_levels = "warn"',
        "after": 'section_heading_levels = "strict"',
    },
)

# The control on the other side: a committed key that conjures warnings out of a
# clean tree and leaves the exit code where it was (§FS-config.3.1.2).
CONTROL = {
    "name": "lead size warning",
    "key": "lead_size_warning",
    "case": "check-citation-levels-default",
    "before": "[reference]\n",
    "after": '[reference]\nlead_size_warning = { max = 2, unit = "words" }\n',
}

LEVER_KEYS = tuple(lever["key"] for lever in LEVERS) + (CONTROL["key"],)


class ShippedCommittedLevers(unittest.TestCase):
    """Green by design: the guard that no repair may make a shipped lever a violation."""

    def _fixture(self, lever, flipped: bool) -> Path:
        root = Path(self.enterContext(tempfile.TemporaryDirectory()))
        fixture = root / "repo"
        shutil.copytree(CASES / lever["case"] / "repo", fixture)
        config = fixture / "grund.toml"
        text = config.read_text(encoding="utf-8")
        self.assertIn(lever["before"], text, f"{lever['case']}/repo/grund.toml no longer carries the lever")
        if flipped:
            config.write_text(text.replace(lever["before"], lever["after"], 1), encoding="utf-8")
        return fixture

    def _run(self, lever, flipped: bool):
        exit_code, findings = _check(self._fixture(lever, flipped))
        where = f"{lever['name']} ({'after' if flipped else 'before'} the committed edit)"

        severities = {finding.get("severity") for finding in findings}
        self.assertLessEqual(
            severities, SEVERITIES, f"{where}: a severity outside the frozen set {sorted(SEVERITIES)}"
        )

        expected = 1 if any(finding["severity"] == "error" for finding in findings) else 0
        self.assertEqual(expected, exit_code, f"{where}: exit code is not the fixed mapping of the report")

        self.assertEqual(
            sorted(findings, key=_sort_key), findings, f"{where}: report is not in bytewise (path, line, message) order"
        )
        return exit_code, findings

    def test_each_committed_lever_moves_a_verdict_inside_the_frozen_vocabulary(self):
        for lever in LEVERS:
            with self.subTest(lever=lever["name"]):
                before_exit, before = self._run(lever, flipped=False)
                after_exit, after = self._run(lever, flipped=True)

                verdict_moved = (before_exit, [_sort_key(f) + (f["severity"],) for f in before]) != (
                    after_exit,
                    [_sort_key(f) + (f["severity"],) for f in after],
                )
                self.assertTrue(verdict_moved, f"{lever['name']}: the committed edit moved no verdict")

                kept = set(_locations(before)) & set(_locations(after))
                self.assertEqual(
                    [location for location in _locations(before) if location in kept],
                    [location for location in _locations(after) if location in kept],
                    f"{lever['name']}: the committed edit moved the report order",
                )

    def test_lead_size_warning_is_the_control(self):
        before_exit, before = self._run(CONTROL, flipped=False)
        after_exit, after = self._run(CONTROL, flipped=True)
        self.assertEqual(0, before_exit)
        self.assertEqual([], before)
        self.assertEqual(0, after_exit, "a warning-only key moved the exit code")
        self.assertTrue(after, "lead_size_warning reported nothing; the control proves nothing")
        self.assertEqual({"warning"}, {finding["severity"] for finding in after})


# --------------------------------------------------------------------------
# Half two: the goal has to admit every one of them.
# --------------------------------------------------------------------------


class GoalAdmitsEveryShippedLever(unittest.TestCase):
    """Red on all five clauses until the ruling on #294 is written down."""

    @classmethod
    def setUpClass(cls):
        cls.body = _point("GOAL-configurable.2")
        cls.sentences = _sentences(cls.body)

    def test_the_frozen_trio_is_asserted_without_an_install_local_qualifier(self):
        naming = [sentence for sentence in self.sentences if _names_trio(sentence)]
        self.assertTrue(naming, "GOAL-configurable.2 no longer states the frozen trio at all")
        unqualified = [sentence for sentence in naming if not _install_local_scoped(sentence)]
        self.assertTrue(
            unqualified,
            "every sentence naming the frozen trio is scoped to install-local choices:\n  "
            + "\n  ".join(sentence.strip() for sentence in naming),
        )

    def test_the_reach_test_is_about_install_local_state_and_not_about_the_trio(self):
        verdict = [sentence for sentence in self.sentences if "verdict" in _normalize(sentence)]
        self.assertTrue(verdict, "GOAL-configurable.2 no longer states the reach test")
        self.assertEqual(
            [],
            [sentence.strip() for sentence in verdict if _names_trio(sentence)],
            "the sentence about verdicts still names the frozen trio: the two are different axes",
        )
        self.assertTrue(
            [sentence for sentence in verdict if _install_local_scoped(sentence)],
            "no sentence puts install-local state as the subject of the reach test",
        )

    def test_the_point_cites_the_install_local_authoring_procedure(self):
        self.assertIn(
            "FS-config.principle.install-local",
            self.body,
            "the ruling names this point for the authoring procedure; citing it is the ruling",
        )

    def test_the_decision_record_carries_the_admissibility_test_and_is_reachable(self):
        self.assertIn("DF-verdict-vocabulary-freeze", self.body, "the goal does not send a reader to the record")
        self.assertTrue(RECORD.is_file(), f"no decision record at {RECORD.relative_to(REPO_ROOT)}")
        record = RECORD.read_text(encoding="utf-8")
        self.assertTrue(
            record.startswith("# DF-verdict-vocabulary-freeze:"),
            "the record does not declare DF-verdict-vocabulary-freeze on its first line",
        )
        self.assertIn(
            "DF-verdict-vocabulary-freeze.md",
            DECISIONS_INDEX.read_text(encoding="utf-8"),
            "the record has no index line in docs/decisions/functional/README.md",
        )
        normalized = _normalize(record)
        self.assertIn("admissib", normalized, "the record does not carry the admissibility test")
        self.assertTrue(
            re.search(r"^#+ .*\b(cost|consequence)", record, re.I | re.M),
            "the record does not say what the ruling costs",
        )
        for key in LEVER_KEYS:
            self.assertIn(
                _normalize(key),
                normalized,
                f"the record does not decide {key}, which the shipped-lever half exercises",
            )

    def test_unmarked_markdown_headings_no_longer_grounds_its_refusal_on_the_goal(self):
        self.assertNotIn(
            "GOAL-configurable",
            UNMARKED_HEADINGS.read_text(encoding="utf-8"),
            "DF-unmarked-markdown-headings still claims the goal as authority for refusing a "
            "committed severity selector; the refusal stands on its own rule-specific reason",
        )


if __name__ == "__main__":
    unittest.main()
