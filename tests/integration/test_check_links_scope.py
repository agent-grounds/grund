"""§AR-ci.3.1.4 — the gate sends an external URL to the network only where the
change adds or modifies its line, and checks every internal link of the tree
offline (§AR-ci.3.1, §FS-non-goals.1.1).

Real lychee against a git fixture and a loopback host that counts requests: no
subprocess result is mocked, so a request count of zero means the wrapper made
no network request at all.
"""

import re
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from link_gate_fixture import LYCHEE, ROOT, LinkFixture  # noqa: E402


MAIN = {
    "docs/a.md": "# A\n\nSee [the old dead page](<{host}/dead-old>).\n\nSee [a live page](<{host}/ok>).\n",
    "docs/b.md": "# B\n\nBack to [A](a.md#a).\n",
    "docs/c.md": "# C\n\nNo links here.\n",
}


@unittest.skipUnless(LYCHEE, "Link gate fixture requires CI-pinned lychee 0.23.0 on PATH")
class ChangedLineGateTests(unittest.TestCase):
    def fixture(self, files=MAIN):
        gate = LinkFixture()
        self.addCleanup(gate.close)
        main = gate.commit_main(files)
        return gate, main

    def test_unchanged_dead_url_is_never_requested(self):
        gate, _ = self.fixture()
        gate.append("docs/c.md", "\nA sentence without a link.\n")
        gate.commit()

        code, output = gate.run("docs")

        self.assertEqual({}, gate.requested(), output)
        self.assertEqual(0, code, output)

    def test_added_dead_url_fails_and_is_the_only_one_requested(self):
        gate, _ = self.fixture()
        gate.append("docs/a.md", "\nSee [a new dead page](<{host}/dead-new>).\n")
        gate.commit()

        code, output = gate.run("docs")

        self.assertNotEqual(0, code, output)
        self.assertIn("/dead-new", output)
        self.assertEqual(["/dead-new"], sorted(gate.requested()), output)

    def test_edited_line_is_checked(self):
        gate, _ = self.fixture()
        text = (gate.root / "docs/a.md").read_text(encoding="utf-8")
        gate.write("docs/a.md", text.replace("/ok>", "/dead-edited>"))
        gate.commit()

        code, output = gate.run("docs")

        self.assertNotEqual(0, code, output)
        self.assertEqual(["/dead-edited"], sorted(gate.requested()), output)

    def test_unstaged_and_untracked_lines_count_as_changed(self):
        gate, _ = self.fixture()
        gate.append("docs/c.md", "\nSee [an unstaged dead page](<{host}/dead-unstaged>).\n")
        gate.write("docs/new.md", "# New\n\nSee [an untracked dead page](<{host}/dead-untracked>).\n")

        code, output = gate.run("docs")

        self.assertNotEqual(0, code, output)
        self.assertEqual(["/dead-unstaged", "/dead-untracked"], sorted(gate.requested()), output)

    def test_broken_internal_link_in_an_untouched_file_fails_offline(self):
        files = dict(MAIN)
        files["docs/b.md"] = "# B\n\n[Gone](missing.md) and [no such part](a.md#absent).\n"
        gate, _ = self.fixture(files)
        gate.append("docs/c.md", "\nA sentence without a link.\n")
        gate.commit()

        code, output = gate.run("docs")

        self.assertEqual({}, gate.requested(), output)
        self.assertNotEqual(0, code, output)
        self.assertIn("b.md", output)

    def test_link_base_and_merge_base_reach_the_same_verdict(self):
        """§AR-ci.1.1: the variable CI sets and the local fallback judge one tree alike."""
        gate, main = self.fixture()
        gate.append("docs/a.md", "\nSee [a new dead page](<{host}/dead-new>).\n")
        gate.commit()

        fallback_code, fallback_output = gate.run("docs")
        fallback = sorted(gate.requested())
        gate.reset()
        gate.git("update-ref", "-d", "refs/remotes/origin/main")
        variable_code, variable_output = gate.run("docs", link_base=main)
        variable = sorted(gate.requested())

        self.assertEqual(["/dead-new"], fallback, fallback_output)
        self.assertEqual(
            (fallback_code != 0, fallback), (variable_code != 0, variable),
            f"fallback:\n{fallback_output}\nLINK_BASE:\n{variable_output}",
        )
        self.assertNotEqual(0, variable_code, variable_output)

    def test_diff_prefix_config_does_not_change_the_verdict(self):
        # R1-01: a user's diff prefixes must not hide an added dead link locally (§AR-ci.1.1).
        gate, _ = self.fixture()
        gate.append("docs/a.md", "\nSee [a new dead page](<{host}/dead-new>).\n")
        for setting in ("'diff.mnemonicprefix=true'", "'diff.noprefix=true'"):
            with self.subTest(setting=setting):
                gate.reset()
                code, output = gate.run("docs", git_config=setting)
                self.assertNotEqual(0, code, output)
                self.assertEqual(["/dead-new"], sorted(gate.requested()), output)

    def test_no_diff_base_checks_no_external_url_and_says_so(self):
        gate, _ = self.fixture()
        gate.append("docs/a.md", "\nSee [a new dead page](<{host}/dead-new>).\n")
        gate.commit()
        gate.git("update-ref", "-d", "refs/remotes/origin/main")

        code, output = gate.run("docs")

        self.assertEqual({}, gate.requested(), output)
        self.assertEqual(0, code, output)
        self.assertEqual(1, len(re.findall(r"(?m)^no diff base:", output)), output)


class CiGivesTheGateItsBaseTests(unittest.TestCase):
    """§AR-ci.3.1.4 — CI's pre-commit job sets LINK_BASE and fetches the history it names."""

    def pre_commit_job(self):
        workflow = (ROOT / ".github" / "workflows" / "ci.yml").read_text(encoding="utf-8")
        job = re.search(r"(?ms)^  pre-commit:\n(?P<body>.*?)(?=^  [a-z][a-z0-9-]*:\n)", workflow)
        self.assertIsNotNone(job, "ci.yml has no pre-commit job")
        return job["body"]

    def test_pre_commit_checks_out_the_full_history(self):
        self.assertRegex(
            self.pre_commit_job(),
            r"uses: actions/checkout@v\d+\n\s+with:\n(?:\s+.*\n)*?\s+fetch-depth: 0\n",
        )

    def test_pre_commit_sets_link_base_from_the_event(self):
        job = self.pre_commit_job()
        self.assertRegex(job, r"\n\s+LINK_BASE:")
        self.assertIn("github.event.pull_request.base.sha", job)
        self.assertIn("github.event.merge_group.base_sha", job)


if __name__ == "__main__":
    unittest.main()
