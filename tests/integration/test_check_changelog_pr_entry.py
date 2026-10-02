"""§AR-ci.7 — the changelog gate, exercised the way its two halves run it: a
branch must add an entry under `docs/changelog/unreleased/` whose slug the merge
base does not hold, and may not write another pull request's number into one
(§FS-distribution.4.6, in the format §FS-distribution.4.12 adopts); the script
reads the number, the base and the branch the way the event supplies them.

The end-to-end class at the bottom is the gate's own parity proof, and the only
place §AR-ci.1.1's "the same verdict on the same tree" is actually checked: it
reads the pre-push invocation out of `.pre-commit-config.yaml` and the CI
invocation out of `.github/workflows/ci.yml`, runs both against one fixture
repository, and asserts they agree. It reads the two invocations rather than
spelling them so that it measures the arrangement this repository ships rather
than the one it happened to have when the test was written.

`ChangelogGateRulesTests` above it drives one half at a time over the same
fixture shape (`changelog_gate_fixture.py`): what counts as an added entry and
what does not, the three skips with the fallback, and the file the refusal
suggests. What the gate reads inside an entry — its name and shape, its number,
and whether it was moved from `## Unreleased` — is
`test_check_changelog_pr_entry_contents.py`."""

import importlib.util
import json
import os
import re
import shlex
import subprocess
import sys
import tempfile
import unittest
from subprocess import CompletedProcess
from unittest.mock import patch
from pathlib import Path

from changelog_gate_fixture import (
    BRANCH,
    EARLIER,
    ENTRIES,
    FIXTURE_PR,
    NEW,
    POINTER,
    REFUSED,
    REPO_ROOT,
    SCRIPT_PATH,
    GitFixture,
    environment,
    pre_push_environment,
)

PRE_COMMIT_CONFIG = REPO_ROOT / ".pre-commit-config.yaml"
CI_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "ci.yml"
HOOK_ID = "changelog-pr-entry"
SCRIPT_NAME = SCRIPT_PATH.name
EXPRESSION = re.compile(r"\$\{\{[^}]*\}\}")
HOOK_START = re.compile(r"- id:\s*(\S+)$")

SPEC = importlib.util.spec_from_file_location("check_changelog_pr_entry", SCRIPT_PATH)
check_changelog_pr_entry = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(check_changelog_pr_entry)

# Section 1 of the agent-grounds/grund#379 proposal, verbatim: what a push that forgot its entry reads.
PROPOSED_REFUSAL = """error: docs/changelog/unreleased/ gains no entry against origin/main.
  Add one file for this change:
      docs/changelog/unreleased/fix-release-record-reads-latest-release.<category>.md
  <category> is one of: added, changed, deprecated, removed, fixed, security.
  The file holds one bullet. A number is optional: the release writes it.
  If this branch is not becoming a pull request:
      SKIP=changelog-pr-entry git push
"""


class ChangelogEventTests(unittest.TestCase):
    """What the `pull_request` event supplies: the number the number check judges
    against, and the base commit the merge base is taken from."""

    def write_event(self, payload: dict) -> Path:
        event_path = Path(self.tempdir.name) / "event.json"
        event_path.write_text(json.dumps(payload), encoding="utf-8")
        return event_path

    def setUp(self) -> None:
        self.tempdir = tempfile.TemporaryDirectory()
        self.addCleanup(self.tempdir.cleanup)

    def test_reads_pull_request_number_from_event_file(self) -> None:
        event_path = self.write_event({"pull_request": {"number": 15, "base": {"sha": "abc123"}}})
        self.assertEqual(check_changelog_pr_entry.pr_number_from_event(event_path), 15)

    def test_reads_base_sha_from_event_file(self) -> None:
        event_path = self.write_event({"pull_request": {"number": 15, "base": {"sha": "abc123"}}})
        self.assertEqual(check_changelog_pr_entry.base_sha_from_event(event_path), "abc123")

    def test_non_pull_request_event_supplies_neither(self) -> None:
        event_path = self.write_event({"ref": "refs/heads/main"})
        self.assertIsNone(check_changelog_pr_entry.pr_number_from_event(event_path))
        self.assertIsNone(check_changelog_pr_entry.base_sha_from_event(event_path))

    def test_rejects_an_invalid_number_in_the_event(self) -> None:
        event_path = self.write_event({"pull_request": {"number": "fifteen"}})
        with self.assertRaises(check_changelog_pr_entry.ChangelogPrError):
            check_changelog_pr_entry.pr_number_from_event(event_path)

    def test_pushed_ref_is_the_head_the_stage_names(self) -> None:
        with patch.dict(os.environ, {"PRE_COMMIT_TO_REF": "deadbeef"}, clear=False):
            self.assertEqual(check_changelog_pr_entry.head_ref_for_pre_push(), "deadbeef")

    def test_a_branch_deletion_has_no_head_and_nothing_to_require(self) -> None:
        # pre-commit hands the all-zero sha when the push deletes the ref: there
        # is no head to list entries from, and nothing is being added.
        with patch.dict(os.environ, {"PRE_COMMIT_TO_REF": "0" * 40}, clear=False):
            self.assertIsNone(check_changelog_pr_entry.head_ref_for_pre_push())

    def test_an_unset_pushed_ref_falls_back_to_the_checkout(self) -> None:
        with patch.dict(os.environ, {"PRE_COMMIT_TO_REF": ""}, clear=False):
            self.assertEqual(check_changelog_pr_entry.head_ref_for_pre_push(), "HEAD")


def _hook_entry() -> str:
    """The `entry:` line of the `changelog-pr-entry` hook, read as text — the CI
    Python has no YAML parser, so §AR-ci.1.2's parity test reads these two files
    line-shaped and this one does the same."""
    current = None
    for raw in PRE_COMMIT_CONFIG.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        started = HOOK_START.match(line)
        if started:
            current = started.group(1)
            continue
        if current == HOOK_ID and line.startswith("entry:"):
            return line[len("entry:") :].strip()
    raise AssertionError(f"{PRE_COMMIT_CONFIG} declares no `entry:` for the {HOOK_ID} hook")


def _ci_run() -> str:
    """The workflow's own invocation of the gate, wherever it sits — a step of
    the test matrix or a job of its own."""
    runs = [
        line.strip()[len("run:") :].strip()
        for line in CI_WORKFLOW.read_text(encoding="utf-8").splitlines()
        if line.strip().startswith("run:")
    ]
    named = [run for run in runs if SCRIPT_NAME in run]
    if len(named) != 1:
        raise AssertionError(f"{CI_WORKFLOW} has {len(named)} steps running {SCRIPT_NAME}, expected 1")
    return named[0]


def _ci_expressions(base: str, head: str) -> dict[str, str]:
    """Every value a `pull_request` event offers the step, the head branch under
    both of the names the workflow may read it by."""
    return {
        "github.event.pull_request.number": str(FIXTURE_PR),
        "github.event.pull_request.base.sha": base,
        "github.event.pull_request.head.sha": head,
        "github.event.pull_request.base.ref": "main",
        "github.event.pull_request.head.ref": BRANCH,
        "github.base_ref": "main",
        "github.head_ref": BRANCH,
    }


def _ci_environment() -> dict[str, str]:
    """What the runner itself sets on a `pull_request` event, beside the step's own line."""
    return environment({"GITHUB_BASE_REF": "main", "GITHUB_HEAD_REF": BRANCH})


def _argv(command: str, expressions: dict[str, str]) -> list[str]:
    def expand(match: re.Match[str]) -> str:
        key = match.group(0)[3:-2].strip()
        if key not in expressions:
            raise AssertionError(f"the workflow reads `{key}`, which this test supplies no value for")
        return expressions[key]

    tokens = shlex.split(EXPRESSION.sub(expand, command))
    return [
        sys.executable if token == "python" else str(SCRIPT_PATH) if token.endswith(SCRIPT_NAME) else token
        for token in tokens
    ]


class MergeCheckout(GitFixture):
    def _merge_checkout(self, root: Path, theirs: dict[str, str], mine: dict[str, str]) -> tuple[Path, str, str]:
        """The tree `actions/checkout` leaves on a `pull_request` event: `refs/pull/N/merge`,
        the head merged with a base tip that moved after the branch point."""
        repo, branch_point, head = self._fixture(root, {**EARLIER, **mine})
        self._git(repo, "checkout", "-q", "-B", "their-main", branch_point)
        self._write(repo, {**EARLIER, **theirs}, POINTER)
        base = self._commit(repo, "Their change, merged after the branch point")
        self._git(repo, "update-ref", "refs/remotes/origin/main", base)
        identity = ("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid")
        self._git(repo, *identity, "merge", "--no-commit", "--no-ff", "-s", "ours", head)
        self._write(repo, {**EARLIER, **theirs, **mine}, POINTER)
        self._commit(repo, "Merge the head into the base tip")
        return repo, base, head

    def _no_pull_request_yet(self, root: Path) -> Path:
        """A `gh` that answers the way it answers on a branch with no pull
        request, so the local half meets the condition without the network."""
        binaries = root / "bin"
        binaries.mkdir()
        shell = binaries / "gh"
        shell.write_text('#!/bin/sh\necho \'no pull requests found for branch\' >&2\nexit 1\n', encoding="utf-8")
        shell.chmod(0o755)
        (binaries / "gh.bat").write_text("@echo no pull requests found 1>&2\r\n@exit /b 1\r\n", encoding="utf-8")
        return binaries


class ChangelogGateRulesTests(MergeCheckout, unittest.TestCase):
    """§FS-distribution.4.6, one half at a time: what counts as an added entry,
    what a malformed one is, what a number in one may be, the move rule, the
    skips, and the file the refusal suggests."""

    # What passes: a slug the merge base does not hold, and no number needed.
    def test_an_added_entry_needs_no_number_and_no_pull_request(self) -> None:
        result = self._pre_push({**EARLIER, **NEW})
        self.assertPassed(result)
        self.assertFalse(self.gh_was_run, "the gate ran `gh`; both halves read local git alone")

    def test_a_wrapped_entry_is_one_bullet(self) -> None:
        self.assertPassed(self._ci({**EARLIER, "the-change.added.md": "- One bullet that\n  wraps onto a second line.\n"}))

    def test_a_branch_may_add_more_than_one_entry(self) -> None:
        self.assertPassed(self._ci({**EARLIER, **NEW, "the-change-too.fixed.md": "- A second entry.\n"}))

    # What does not count: only an added slug does (the narrowing of agent-grounds/grund#379).
    def test_a_branch_that_adds_no_entry_is_refused(self) -> None:
        self.assertRefused(self._ci(EARLIER), REFUSED, f"{ENTRIES}/")

    def test_a_bullet_under_unreleased_is_not_an_entry(self) -> None:
        habit = "### Changed\n\n- A bullet written where bullets used to go."
        self.assertRefused(self._ci(EARLIER, head_unreleased=habit), REFUSED, f"{ENTRIES}/")

    def test_editing_an_entry_is_not_adding_one(self) -> None:
        edits = {
            "reworded": "- An earlier pull request's entry, reworded here. (PR #1)\n",
            "rewrapped": "- An earlier pull request's\n  entry. (PR #1)\n",
            "numbered again": f"- An earlier pull request's entry. (PR #1) (PR #{FIXTURE_PR})\n",
        }
        for edit, text in edits.items():
            with self.subTest(edit=edit):
                self.assertRefused(self._ci({"earlier-change.fixed.md": text}), REFUSED)

    def test_changing_an_entrys_category_is_not_adding_one(self) -> None:
        self.assertRefused(self._ci({"earlier-change.changed.md": EARLIER["earlier-change.fixed.md"]}), REFUSED)

    def test_editing_the_readme_is_not_an_entry(self) -> None:
        self.assertRefused(self._ci({**EARLIER, "README.md": "# Unreleased changelog entries\n\nReworded.\n"}), REFUSED)

    def test_an_uncommitted_entry_is_not_a_pushed_one(self) -> None:
        # A push carries commits: the head side is listed from the head commit, so
        # an entry still only in the working tree is not what this branch adds.
        with tempfile.TemporaryDirectory() as tmp:
            repo, base, head = self._fixture(Path(tmp), EARLIER)
            (repo / ENTRIES / "the-change.added.md").write_text(NEW["the-change.added.md"], encoding="utf-8")
            result = subprocess.run(
                [sys.executable, str(SCRIPT_PATH), "--base-sha", base, "--head-sha", head],
                cwd=repo,
                capture_output=True,
                text=True,
                env=environment({}),
            )
        self.assertRefused(result, REFUSED)

    def test_an_entry_the_base_gained_after_the_branch_point_is_not_this_branchs(self) -> None:
        # Driven as `ci.yml` drives it, on the merge tree the event leaves. List the
        # directory in the checkout rather than at the head commit and the entry the
        # base gained since the branch point counts as this branch's: it would pass a
        # branch that added nothing, and refuse one for a number nobody here wrote.
        theirs = {"their-change.fixed.md": "- Somebody else's freshly merged entry. (PR #340)\n"}
        for mine, verdict in ((NEW, 0), ({}, 1)):
            with self.subTest(adds=bool(mine)), tempfile.TemporaryDirectory() as tmp:
                repo, base, head = self._merge_checkout(Path(tmp), theirs, mine)
                result = subprocess.run(
                    _argv(_ci_run(), _ci_expressions(base, head)),
                    cwd=repo,
                    capture_output=True,
                    text=True,
                    env=_ci_environment(),
                )
                self.assertEqual(verdict, result.returncode, self._output(result))
                self.assertNotIn("PR #340", self._output(result))
                if verdict:
                    self.assertIn(REFUSED, self._output(result))

    def test_an_absolute_changelog_path_reaches_the_same_verdict_as_a_relative_one(self) -> None:
        # `git show <commit>:/abs/path` is refused, and reading that refusal as
        # "the directory was empty" made every entry count as added — and #1 foreign.
        with tempfile.TemporaryDirectory() as tmp:
            repo, base, head = self._fixture(Path(tmp), EARLIER)
            result = self._run_ci(repo, base, head, "--changelog", str(repo / "docs" / "changelog.md"))
        self.assertRefused(result, REFUSED)
        self.assertNotIn("PR #1", self._output(result))

    # The three skips, all by condition, and the fallback.
    def test_a_head_the_base_already_contains_skips(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo, base, _head = self._fixture(Path(tmp), EARLIER)
            result = self._run_ci(repo, base, base)
        self.assertPassed(result)
        self.assertIn("already contains this head", self._output(result))

    def test_an_unresolvable_base_falls_back_to_one_entry_and_warns(self) -> None:
        result = self._ci({**EARLIER, **NEW}, base_sha="0" * 40)
        self.assertPassed(result)
        self.assertIn("no base ref resolves", self._output(result))
        self.assertIn(ENTRIES, self._output(result))

    def test_a_base_that_resolves_but_shares_no_history_says_so(self) -> None:
        # The same skip, reached the other way: a shallow clone's grafted base
        # resolves and still has no merge base, and telling the contributor no
        # ref resolves would send them to fetch a ref they already have.
        with tempfile.TemporaryDirectory() as tmp:
            repo, _base, head = self._fixture(Path(tmp), {**EARLIER, **NEW})
            self._git(repo, "checkout", "-q", "--orphan", "another-history")
            (repo / "README.md").write_text("Another history.\n", encoding="utf-8")
            stranger = self._commit(repo, "The root of an unrelated history")
            result = self._run_ci(repo, stranger, head)
        self.assertPassed(result)
        self.assertIn("share no history", self._output(result))
        self.assertNotIn("no base ref resolves", self._output(result))

    def test_the_fallback_still_refuses_a_head_with_no_entry(self) -> None:
        self.assertRefused(self._ci({}, {}, base_sha="0" * 40), ENTRIES)

    # The refusal names the file to add, built from the branch.
    def test_the_pre_push_refusal_is_the_one_the_proposal_shows(self) -> None:
        result = self._pre_push(EARLIER, branch="refs/heads/fix-release-record-reads-latest-release")
        self.assertEqual(1, result.returncode, self._output(result))
        self.assertEqual(PROPOSED_REFUSAL, result.stderr)

    def test_the_suggested_slug_is_the_branch_name_made_a_slug(self) -> None:
        branches = {
            "refs/heads/fix/issue-379": "fix-issue-379",
            "refs/heads/Feature/Big_Change": "feature-big-change",
            "refs/heads/_odd__name_": "odd-name",
            BRANCH: "fix-the-change",
        }
        with tempfile.TemporaryDirectory() as tmp:
            repo, base, head = self._fixture(Path(tmp), EARLIER)
            for branch, slug in branches.items():
                with self.subTest(branch=branch):
                    result = self._run_pre_push(repo, base, head, branch)
                    self.assertRefused(result, f"{ENTRIES}/{slug}.<category>.md")


class ChangelogGateParityTests(MergeCheckout, unittest.TestCase):
    """Both halves of one gate, on one tree, from the repository's own configuration."""

    def _both_halves(self, head: dict[str, str]) -> tuple[CompletedProcess, CompletedProcess]:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            repo, base, head_sha = self._fixture(root, head)
            local = subprocess.run(
                _argv(_hook_entry(), {}),
                cwd=repo,
                capture_output=True,
                text=True,
                env=pre_push_environment(base, head_sha, f"refs/heads/{BRANCH}", self._no_pull_request_yet(root)),
            )
            remote = subprocess.run(
                _argv(_ci_run(), _ci_expressions(base, head_sha)),
                cwd=repo,
                capture_output=True,
                text=True,
                env=_ci_environment(),
            )
            return local, remote

    def _report(self, half: str, result: CompletedProcess) -> str:
        return f"{half}: exit {result.returncode}\n{(result.stdout + result.stderr).strip()}"

    def _verdicts(self, local: CompletedProcess, remote: CompletedProcess) -> None:
        self.assertEqual(
            local.returncode,
            remote.returncode,
            "the two halves of one gate disagree on one tree\n"
            f"{self._report('pre-push', local)}\n{self._report('pull-request CI', remote)}",
        )

    def test_both_halves_refuse_a_branch_that_added_no_entry(self) -> None:
        local, remote = self._both_halves(EARLIER)
        self._verdicts(local, remote)
        self.assertEqual(1, local.returncode, self._report("pre-push", local))
        for half, result in (("pre-push", local), ("pull-request CI", remote)):
            with self.subTest(half=half):
                self.assertIn(f"{ENTRIES}/fix-the-change.<category>.md", result.stdout + result.stderr)
        self.assertIn("SKIP=changelog-pr-entry", local.stdout + local.stderr)

    def test_both_halves_accept_an_added_entry_that_names_no_number(self) -> None:
        local, remote = self._both_halves({**EARLIER, **NEW})
        self._verdicts(local, remote)
        self.assertEqual(0, local.returncode, self._report("pre-push", local))

    def test_both_halves_refuse_an_entry_in_a_category_grund_does_not_take(self) -> None:
        local, remote = self._both_halves({**EARLIER, **NEW, "a-note.note.md": "- A bullet.\n"})
        self._verdicts(local, remote)
        self.assertEqual(1, local.returncode, self._report("pre-push", local))


if __name__ == "__main__":
    unittest.main()
