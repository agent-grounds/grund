"""§FS-distribution.4.4.1.1 — a release helper whose `main` the merge queue moves
rebuilds its candidates on the new tip and runs all three candidate runs again,
and `main` still takes only commits those runs passed.

Each case runs one helper's whole job - `release-minor.yml` or `auto-bump.yml` -
against a local bare origin and a stub forge that plays the merge queue
(`release_queue_fixture.py`, `release_queue_forge.py`), so no case reaches the
network. The queue lands pull request #975 while the first round's runs are going,
as it did under run 39962684811 (agent-grounds/grund#983).

The bounds of §FS-distribution.4.4.1.2 are shortened through the controller's
waits (§AR-ci.11): `GRUND_RELEASE_SETTLE`, `GRUND_RELEASE_BUDGET`,
`GRUND_RELEASE_ROUND_MIN` and `GRUND_RELEASE_RESERVE`, in seconds."""

import shutil
import sys
import tempfile
import unittest
from pathlib import Path

from release_queue_fixture import LATER, MINOR, PATCH, Case



@unittest.skipIf(sys.platform == "win32", "the helpers' steps are bash scripts with shell hooks")
@unittest.skipUnless(shutil.which("bash") and shutil.which("git") and shutil.which("jq"), "needs bash, git and jq")
class FollowsMain:
    """The cases both helpers share; a subclass names the helper and its versions."""

    WORKFLOW = INPUTS = NEXT = DEV = None

    def setUp(self):
        scratch = tempfile.TemporaryDirectory(prefix="grund-983-")
        self.addCleanup(scratch.cleanup)
        self.scratch = Path(scratch.name)

    def case(self, scenario=None, *, inputs=None, knobs=None, timeout=600, push=None):
        case = Case(self.scratch, self.WORKFLOW, scenario, {**self.INPUTS, **(inputs or {})}, knobs, timeout)
        if push:
            case.push_main(*push)
        return case.run()

    def assertReleased(self, case, round_, base):
        """Published and advanced, on exactly the commits `round_`'s runs passed, over `base`."""
        report = case.report()
        self.assertTrue(case.ok, report)
        (publish,) = case.published() or [None]
        self.assertIsNotNone(publish, f"release.yml was not dispatched to publish\n{report}")
        release, dev = publish["sha"], case.main()
        self.assertEqual(self.NEXT, case.version(release), report)
        self.assertEqual(base, case.parent(release), f"the release is not built on {base[:10]}\n{report}")
        self.assertTrue(case.passed(release, "release.yml", round_), f"no dry run of round {round_} passed it\n{report}")
        self.assertTrue(case.passed(release, "ci.yml", round_), f"no CI of round {round_} passed the release\n{report}")
        self.assertEqual(release, case.parent(dev), f"main is not the advance over the release\n{report}")
        self.assertEqual(self.DEV, case.version(dev), report)
        self.assertTrue(case.passed(dev, "ci.yml", round_), f"no CI of round {round_} passed the advance\n{report}")

    def assertNothingPublished(self, case, main):
        report = case.report()
        self.assertFalse(case.ok, report)
        self.assertEqual([], case.published(), report)
        self.assertEqual(main, case.main(), report)

    def assertCancelled(self, case, round_):
        """Every run of `round_` was cancelled or had finished; none is left going."""
        runs = case.runs(round_)
        self.assertEqual(3, len(runs), case.report())
        self.assertEqual([], [r["id"] for r in runs if r["status"] != "completed"], case.report())
        self.assertTrue(any(r.get("cancelled") for r in runs), f"no run of round {round_} was cancelled\n"
                        + case.report())

    def test_an_unmoved_main_publishes_and_advances(self):
        case = self.case()
        self.assertReleased(case, 1, case.shas["a"])
        self.assertEqual(1, case.rounds(), case.report())

    def test_a_move_cancels_the_round_and_publishes_what_the_next_round_passed(self):
        case = self.case({"move": {"1": "m1"}})
        self.assertReleased(case, 2, case.shas["m1"])
        self.assertEqual(2, case.rounds(), case.report())
        self.assertCancelled(case, 1)
        self.assertTrue(all(run["main"] == case.shas["m1"] for run in case.runs(2)), case.report())
        first = {run["branch"] for run in case.runs(1)}
        self.assertEqual(set(), first & set(case.branches()), f"round 1's branches remain\n{case.report()}")
        self.assertEqual(set(), first & {run["branch"] for run in case.runs(2)}, "round 2 reused a branch name")
        # The `no new branches` ruleset's `release-candidate/**` does not cross a `/`.
        for branch in case.created():
            self.assertNotIn("/", branch.removeprefix("release-candidate/"), branch)

    def test_the_publisher_is_dispatched_on_the_tested_release_sha(self):
        case = self.case()
        (publish,) = case.published()
        self.assertEqual(publish["sha"], publish["fields"].get("expected_release_sha"), case.report())
        self.assertTrue(publish["branch"].startswith("release-candidate/"), case.report())

    def test_a_failed_run_in_a_refreshed_round_keeps_only_that_rounds_branches(self):
        """§FS-distribution.4.4.1.4: a failed run keeps only its own round's branches."""
        case = self.case({"move": {"1": "m1"}, "fail": {"2": "ci.yml"}})
        self.assertNothingPublished(case, case.shas["m1"])
        self.assertEqual(2, case.rounds(), case.report())
        self.assertCancelled(case, 2)
        self.assertEqual(sorted({run["branch"] for run in case.runs(2)}), case.branches(), case.report())

    def test_six_moving_rounds_exhaust_the_helper(self):
        moves = {str(n): f"m{n}" for n in range(1, 7)}
        case = self.case({"move": moves})
        self.assertNothingPublished(case, case.shas["m6"])
        self.assertEqual(6, case.rounds(), case.report())
        for round_ in range(1, 7):
            self.assertCancelled(case, round_)
        self.assertEqual([], case.branches(), "exhaustion leaves no branch behind")

    def test_the_time_guard_starts_no_round_it_cannot_finish(self):
        knobs = {"GRUND_RELEASE_BUDGET": 65, "GRUND_RELEASE_ROUND_MIN": 60, "GRUND_RELEASE_RESERVE": 1,
                 "GRUND_RELEASE_SETTLE": 8}
        case = self.case({"move": {"1": "m1"}}, knobs=knobs)
        self.assertNothingPublished(case, case.shas["m1"])
        self.assertEqual(1, case.rounds(), case.report())
        self.assertCancelled(case, 1)
        self.assertEqual([], case.branches(), "exhaustion leaves no branch behind")

    def test_the_watch_deadline_cancels_a_slow_round_and_stops(self):
        knobs = {"GRUND_RELEASE_BUDGET": 6, "GRUND_RELEASE_ROUND_MIN": 1, "GRUND_RELEASE_RESERVE": 2,
                 "CANDIDATE_RUNS_WATCH": "0.2"}
        case = self.case({"slow": True}, knobs=knobs, timeout=12)
        self.assertNothingPublished(case, case.shas["a"])
        self.assertNotIn(-1, [code for _name, code, _out, _err in case.ran], "the job's timeout killed it")
        self.assertEqual(1, case.rounds(), case.report())
        self.assertCancelled(case, 1)

    def test_a_ramp_the_queue_landed_fails_the_rebuild_before_any_run(self):
        """§FS-distribution.4.4.1.4: a refused rebuild stops before any branch or run."""
        case = self.case({"move": {"1": "ramp"}})
        self.assertNothingPublished(case, case.shas["ramp"])
        self.assertEqual(1, case.rounds(), case.report())
        self.assertEqual(2, len(case.created()), "the rebuild pushed a branch")
        self.assertIn(f"became an error in {LATER}", case.stderr())

    def test_a_target_tag_that_appeared_stops_before_anything_is_pushed(self):
        case = self.case({"move": {"1": "m1"}, "tag": {"1": f"v{self.NEXT}"}})
        self.assertNothingPublished(case, case.shas["m1"])
        self.assertEqual(1, case.rounds(), case.report())
        self.assertCancelled(case, 1)
        self.assertEqual(2, len(case.created()), case.report())
        self.assertIn(f"v{self.NEXT}", case.stderr())
        self.assertIn("tag", case.stderr())

    def test_version_sources_that_moved_stop_the_helper(self):
        case = self.case({"move": {"1": "versions"}})
        self.assertNothingPublished(case, case.shas["versions"])
        self.assertEqual(1, case.rounds(), case.report())
        self.assertCancelled(case, 1)
        self.assertEqual(2, len(case.created()), case.report())
        self.assertIn(PATCH[1], case.stderr())

    def test_main_settles_before_the_next_round(self):
        case = self.case({"move": {"1": "m1"}, "lander": {"1": "m2"}}, knobs={"GRUND_RELEASE_SETTLE": 4})
        self.assertReleased(case, 2, case.shas["m2"])
        self.assertEqual(2, case.rounds(), case.report())

    def test_a_push_the_queue_beat_is_movement(self):
        """§FS-distribution.4.4.1.3: a main the read-back finds elsewhere is a move."""
        case = self.case(push=("race", "m1"))
        self.assertReleased(case, 2, case.shas["m1"])
        self.assertEqual([], [r for r in case.runs(1) if r["status"] != "completed"], case.report())

    def test_a_push_that_landed_though_git_reported_failure_goes_on(self):
        """§FS-distribution.4.4.1.3: a read-back that finds the release commit goes on."""
        case = self.case(push=("land",))
        self.assertReleased(case, 1, case.shas["a"])
        self.assertEqual(1, case.rounds(), case.report())

    def test_a_refused_push_onto_an_unmoved_main_names_gits_error(self):
        """§FS-distribution.4.4.1.3: a read-back that finds the round base stops with git's error."""
        case = self.case(push=("refuse",))
        self.assertNothingPublished(case, case.shas["a"])
        self.assertEqual(1, case.rounds(), case.report())
        self.assertIn("simulated rejection: the push to main was refused", case.stderr())
        self.assertEqual(sorted({run["branch"] for run in case.runs(1)}), case.branches(), case.report())

    def test_a_failed_run_wins_over_a_move_in_the_same_poll(self):
        """§FS-distribution.4.4.1.4: a failure wins over a move seen on the same look."""
        case = self.case({"move": {"1": "m1"}, "move_at": "dispatch", "fail": {"1": "ci.yml"}})
        self.assertNothingPublished(case, case.shas["m1"])
        self.assertEqual(1, case.rounds(), case.report())
        self.assertCancelled(case, 1)
        self.assertEqual(sorted({run["branch"] for run in case.runs(1)}), case.branches(), case.report())


class ReleaseMinorFollowsMain(FollowsMain, unittest.TestCase):
    WORKFLOW, (NEXT, DEV) = "release-minor.yml", MINOR
    INPUTS = {"version": "next-minor", "expected_main_sha": ""}

    def test_a_pinned_run_refuses_movement_and_names_the_tip_to_pin(self):
        """§FS-distribution.4.4.2: a run given `expected_main_sha` does not follow `main`."""
        case = Case(self.scratch, self.WORKFLOW, {"move": {"1": "m1"}}, dict(self.INPUTS))
        case.inputs["expected_main_sha"] = case.shas["a"]
        case.run()
        self.assertNothingPublished(case, case.shas["m1"])
        self.assertEqual(1, case.rounds(), case.report())
        self.assertCancelled(case, 1)
        self.assertEqual([], case.branches(), case.report())
        self.assertIn(case.shas["m1"], case.stderr())
        self.assertIn("expected_main_sha", case.stderr())


class AutoBumpFollowsMain(FollowsMain, unittest.TestCase):
    WORKFLOW, (NEXT, DEV), INPUTS = "auto-bump.yml", PATCH, {}


if __name__ == "__main__":
    unittest.main()
