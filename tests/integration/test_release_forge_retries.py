"""§FS-distribution.4.6.3.1 — a forge call the release makes is tried three times,
waiting one second and then two between failed attempts; a non-zero `gh` and an
answer that is not JSON share that budget, the last attempt's diagnostic is the
refusal, and a missing `gh` or an answer of the wrong shape is refused at once.

`release_pull_requests` is called in-process, `gh` is a scripted
`subprocess.run` and the wait is a recorded `time.sleep`, so every case runs
in no time; the few cases that run the whole script against a stub `gh`, and
sleep for real, are in `test_prepare_changelog_release_list.py`."""

import importlib.util
import json
import subprocess
import unittest
from pathlib import Path
from unittest import mock


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "release_pull_requests.py"

_spec = importlib.util.spec_from_file_location("release_pull_requests", SCRIPT)
release_pull_requests = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(release_pull_requests)

ChangelogError = release_pull_requests.ChangelogError
Commit, Range = release_pull_requests.Commit, release_pull_requests.Range

TOP = Path(".")
SHA = "edb62f698bf2" + "0" * 28
COMMIT = Commit(SHA, "A change")
REPOSITORY = {"full_name": "agent-grounds/grund", "html_url": "https://github.com/agent-grounds/grund"}
PULL = {
    "number": 751,
    "title": "A change",
    "html_url": "https://github.com/agent-grounds/grund/pull/751",
    "merged_at": "2026-10-15T12:00:00Z",
    "base": {"ref": "main"},
    "merge_commit_sha": SHA,
}

# One attempt each: how `gh` exits, and what it prints.
TRUNCATED = (1, "", "unexpected end of JSON input\n")
SERVER = (1, "", "gh: Server Error (HTTP 502)\n")
PARTIAL = (0, '[{"number": 751, "title": "A ch', "")


def answer(value: object) -> tuple[int, str, str]:
    return (0, json.dumps(value), "")


class ForgeScript:
    """`gh`, as a scripted `subprocess.run`: each call to an endpoint takes the
    next outcome of that endpoint's sequence, and the last one repeats."""

    def __init__(self, **sequences: list[tuple[int, str, str]]) -> None:
        self.sequences = sequences
        self.calls: list[str] = []

    def __call__(self, arguments, **_options) -> subprocess.CompletedProcess:
        endpoint = arguments[-1]
        self.calls.append(endpoint)
        sequence = self.sequences["commit" if "/commits/" in endpoint else "repo"]
        code, out, err = sequence[min(self.calls.count(endpoint), len(sequence)) - 1]
        return subprocess.CompletedProcess(arguments, code, out, err)


class RetryCase(unittest.TestCase):
    def _run(self, call, gh: str | None = "gh", **sequences):
        """Run `call` against the scripted forge; returns its result or its
        refusal, the endpoints asked, and the waits between attempts."""
        self.forge, self.waits = ForgeScript(**sequences), []
        with mock.patch.object(release_pull_requests.shutil, "which", return_value=gh), mock.patch.object(
            release_pull_requests.subprocess, "run", self.forge
        ), mock.patch("time.sleep", self.waits.append):
            try:
                return call()
            except ChangelogError as refusal:
                return refusal

    def _pulls(self, *outcomes):
        return self._run(lambda: release_pull_requests._pulls_for(TOP, COMMIT), commit=list(outcomes))

    def assertListed(self, result, calls: int, waits: list[int]) -> None:
        self.assertNotIsInstance(result, ChangelogError, f"refused after {len(self.forge.calls)} call(s): {result}")
        self.assertEqual([751], [pull.number for pull in result])
        self.assertEqual(calls, len(self.forge.calls))
        self.assertEqual(waits, self.waits)

    def assertRefused(self, result, needle: str, calls: int, waits: list[int]) -> None:
        self.assertIsInstance(result, ChangelogError, result)
        self.assertIn(needle, str(result))
        self.assertEqual(calls, len(self.forge.calls), str(result))
        self.assertEqual(waits, self.waits)


class RecoveryTests(RetryCase):
    # Red until #781 retries the forge call; implement removes this marker.
    @unittest.expectedFailure
    def test_a_call_that_fails_once_is_asked_again_after_one_second(self) -> None:
        self.assertListed(self._pulls(TRUNCATED, answer([PULL])), calls=2, waits=[1])

    # Red until #781 retries the forge call; implement removes this marker.
    @unittest.expectedFailure
    def test_an_answer_that_is_not_json_is_asked_again(self) -> None:
        self.assertListed(self._pulls(PARTIAL, answer([PULL])), calls=2, waits=[1])

    # Red until #781 retries the forge call; implement removes this marker.
    @unittest.expectedFailure
    def test_the_third_attempt_still_counts(self) -> None:
        self.assertListed(self._pulls(SERVER, SERVER, answer([PULL])), calls=3, waits=[1, 2])

    # Red until #781 retries the forge call; implement removes this marker.
    @unittest.expectedFailure
    def test_both_kinds_of_failure_share_one_budget(self) -> None:
        self.assertListed(self._pulls(TRUNCATED, PARTIAL, answer([PULL])), calls=3, waits=[1, 2])

    def test_an_answer_on_the_first_attempt_waits_for_nothing(self) -> None:
        self.assertListed(self._pulls(answer([PULL])), calls=1, waits=[])

    # Red until #781 retries the forge call; implement removes this marker.
    @unittest.expectedFailure
    def test_the_repository_call_is_asked_again_too(self) -> None:
        result = self._run(lambda: release_pull_requests._repository_url(TOP), repo=[TRUNCATED, answer(REPOSITORY)])
        self.assertEqual("https://github.com/agent-grounds/grund", result, result)
        self.assertEqual(2, len(self.forge.calls))
        self.assertEqual([1], self.waits)


class ExhaustionTests(RetryCase):
    # Red until #781 retries the forge call; implement removes this marker.
    @unittest.expectedFailure
    def test_a_call_that_keeps_failing_is_refused_with_the_third_attempt_s_diagnostic(self) -> None:
        last = (1, "", "gh: Bad Gateway (HTTP 502)\n")
        result = self._pulls(TRUNCATED, SERVER, last)
        self.assertRefused(result, "the forge did not answer for commit edb62f698bf2 (A change): gh: Bad Gateway", 3, [1, 2])

    # Red until #781 retries the forge call; implement removes this marker.
    @unittest.expectedFailure
    def test_an_answer_that_is_never_json_is_refused_after_three_attempts(self) -> None:
        result = self._pulls(PARTIAL)
        self.assertRefused(result, "the forge's answer for commit edb62f698bf2 (A change) is not JSON", 3, [1, 2])

    # Red until #781 retries the forge call; implement removes this marker.
    @unittest.expectedFailure
    def test_alternating_failures_are_refused_with_the_last_one_s_diagnostic(self) -> None:
        result = self._pulls(PARTIAL, TRUNCATED, PARTIAL)
        self.assertRefused(result, "is not JSON", 3, [1, 2])

    # Red until #781 retries the forge call; implement removes this marker.
    @unittest.expectedFailure
    def test_a_third_attempt_is_never_followed_by_a_fourth(self) -> None:
        result = self._pulls(TRUNCATED, TRUNCATED, TRUNCATED, answer([PULL]))
        self.assertRefused(result, "unexpected end of JSON input", 3, [1, 2])


class NoRetryTests(RetryCase):
    """What the forge answered, however wrongly, or a `gh` that is not there, is
    refused on the spot: asking again would not change it."""

    def test_a_missing_gh_is_refused_before_any_attempt(self) -> None:
        result = self._run(lambda: release_pull_requests._pulls_for(TOP, COMMIT), gh=None, commit=[answer([PULL])])
        self.assertRefused(result, "gh is not on PATH", 0, [])

    def test_a_commit_answered_with_something_other_than_a_list_is_refused_at_once(self) -> None:
        self.assertRefused(self._pulls(answer({"message": "odd"}), answer([PULL])), "other than a list", 1, [])

    def test_a_repository_answer_with_no_address_is_refused_at_once(self) -> None:
        result = self._run(lambda: release_pull_requests._repository_url(TOP), repo=[answer([]), answer(REPOSITORY)])
        self.assertRefused(result, "did not say which repository", 1, [])

    def test_a_pull_request_linking_elsewhere_is_refused_at_once(self) -> None:
        elsewhere = dict(PULL, html_url="https://github.com/someone/else/pull/751")
        span = Range(TOP, SHA, "v0.17.0", (COMMIT,))
        result = self._run(
            lambda: release_pull_requests.pull_request_lines(span),
            repo=[answer(REPOSITORY)],
            commit=[answer([elsewhere]), answer([PULL])],
        )
        self.assertRefused(result, "someone/else", 2, [])


if __name__ == "__main__":
    unittest.main()
