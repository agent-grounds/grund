"""§AR-ci.3.1.5 — the scheduled sweep checks every external URL of the tree
through `scripts/check_links.py --all`, reports each failure with its file and
line, and files one `bug` issue; it is never a check of a pull request.

The wrapper cases run real lychee against a loopback host that counts requests.
"""

import re
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from link_gate_fixture import LYCHEE, ROOT, LinkFixture  # noqa: E402


WORKFLOW = ROOT / ".github" / "workflows" / "links-nightly.yml"
TITLE = "Nightly link sweep: external links failing"
MAIN = {
    "docs/a.md": "# A\n\nSee [the old dead page](<{host}/dead-old>).\n\nSee [a live page](<{host}/ok>).\n",
    "docs/b.md": "# B\n\nBack to [A](a.md#a).\n\nAgain [the old dead page](<{host}/dead-old>).\n",
}


@unittest.skipUnless(LYCHEE, "Link gate fixture requires CI-pinned lychee 0.23.0 on PATH")
class SweepTests(unittest.TestCase):
    def fixture(self, files=MAIN):
        gate = LinkFixture()
        self.addCleanup(gate.close)
        gate.commit_main(files)
        return gate

    def test_all_requests_every_external_url_on_an_unchanged_tree(self):
        gate = self.fixture()

        code, output = gate.run("--all", "docs")

        self.assertEqual(["/dead-old", "/ok"], sorted(gate.requested()), output)
        self.assertNotEqual(0, code, output)

    def test_all_passes_a_clean_tree(self):
        gate = self.fixture({"docs/a.md": "# A\n\nSee [a live page](<{host}/ok>).\n"})

        code, output = gate.run("--all", "docs")

        self.assertEqual(["/ok"], sorted(gate.requested()), output)
        self.assertEqual(0, code, output)

    def test_report_names_every_file_and_line_that_carries_a_failing_url(self):
        gate = self.fixture()
        report = gate.root / "sweep-report.md"

        code, output = gate.run("--all", "--report", str(report), "docs")

        self.assertNotEqual(0, code, output)
        self.assertTrue(report.is_file(), output)
        lines = report.read_text(encoding="utf-8").splitlines()
        dead = re.escape(f"{gate.base}/dead-old")
        for location in ("docs/a.md:3", "docs/b.md:5"):
            self.assertTrue(
                any(re.fullmatch(rf"- {dead} - {re.escape(location)} - 404\b.*", line)
                    for line in lines),
                f"no report line for {location}:\n" + "\n".join(lines),
            )
        self.assertFalse(any("/ok" in line for line in lines), "\n".join(lines))


class SweepWorkflowTests(unittest.TestCase):
    """§AR-ci.3.1.5 — the workflow's triggers, command, permissions and issue."""

    def workflow(self):
        self.assertTrue(WORKFLOW.is_file(), f"{WORKFLOW.relative_to(ROOT)} is missing")
        return WORKFLOW.read_text(encoding="utf-8")

    def hook_inputs(self):
        config = (ROOT / ".pre-commit-config.yaml").read_text(encoding="utf-8")
        entry = re.search(r"- id: lychee\n(?:\s+.*\n)*?\s+entry: python scripts/check_links\.py (.+)\n",
                          config)
        self.assertIsNotNone(entry, "no lychee hook entry")
        return entry[1].split()

    def test_runs_on_a_schedule_and_on_dispatch_only(self):
        text = self.workflow()
        triggers = re.search(r"(?ms)^on:\n(.*?)(?=^\S)", text)
        self.assertIsNotNone(triggers, "no `on:` block")
        names = set(re.findall(r"(?m)^  ([a-z_]+):", triggers[1]))
        self.assertEqual({"schedule", "workflow_dispatch"}, names)

    def test_runs_the_full_pass_over_the_hooks_inputs(self):
        command = re.search(r"python3? scripts/check_links\.py (.+)", self.workflow())
        self.assertIsNotNone(command, "the workflow does not run scripts/check_links.py")
        self.assertIn("--all", command[1].split())
        self.assertIn(" ".join(self.hook_inputs()), command[1])

    def test_files_one_bug_issue(self):
        text = self.workflow()
        self.assertRegex(text, r"\n\s+issues: write\n")
        self.assertIn(TITLE, text)
        self.assertRegex(text, r"\bbug\b")


if __name__ == "__main__":
    unittest.main()
