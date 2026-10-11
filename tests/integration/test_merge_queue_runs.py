"""§AR-ci.12 — a merge queue lands every pull request, so CI runs on its groups.

The queue tests each group under the checks a pull request must pass and lands
exactly the commit it tested, which is only true while `ci.yml` answers the
`merge_group` event. On a group the commit-message job scans what the group puts
on `main` (§AR-ci.8.2), and the benchmark job stays out (§AR-ci.5.1). Read as
text, the way `test_release_workflow.py` reads the workflows."""

import re
import unittest

from test_release_workflow import WORKFLOWS, jobs, squash, step_name, steps

CI = WORKFLOWS / "ci.yml"


class MergeQueueRunTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.text = CI.read_text(encoding="utf-8")
        cls.jobs = jobs(cls.text)

    def test_ci_runs_on_merge_queue_groups(self):
        triggers = self.text.split("\non:\n", 1)[1].split("\nenv:", 1)[0]
        self.assertRegex(triggers, r"(?m)^  merge_group:", "ci.yml does not run on merge-queue groups")

    def test_the_test_matrix_runs_on_every_event(self):
        header = self.jobs["test"].split("steps:", 1)[0]
        self.assertNotIn("if:", header, "the required checks must report on a queue group too")

    def test_the_benchmark_job_is_skipped_on_merge_queue_groups(self):
        header = self.jobs["bench"].split("steps:", 1)[0]
        condition = re.search(r"(?m)^    if:\s*(.+)$", header)
        self.assertIsNotNone(condition, "the bench job runs on merge-queue groups")
        self.assertEqual("github.event_name != 'merge_group'", condition.group(1).strip())

    def test_the_commit_message_range_covers_the_group(self):
        (scan,) = [step for step in steps(self.jobs["commit-messages"])
                   if step_name(step) == "scan commit messages"]
        flat = squash(scan)
        self.assertIn("github.event_name == 'merge_group' && format('{0}..{1}', "
                      "github.event.merge_group.base_sha, github.event.merge_group.head_sha)", flat)
        self.assertIn('--range "$RANGE"', flat)


if __name__ == "__main__":
    unittest.main()
