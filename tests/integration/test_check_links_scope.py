"""§AR-ci.3.1.4 — the gate sends an external URL to the network only where the
change adds or modifies its line, and checks every internal link of the tree
offline (§AR-ci.3.1, §FS-non-goals.1.1).

Real lychee against a git fixture and a loopback host that counts requests: no
subprocess result is mocked, so a request count of zero means the wrapper made
no network request at all.
"""

import importlib.util
import os
import re
import sys
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
from link_gate_fixture import LYCHEE, ROOT, LinkFixture, isolated_env  # noqa: E402

SPEC = importlib.util.spec_from_file_location("link_scope", ROOT / "scripts" / "link_scope.py")
link_scope = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(link_scope)

WINDOWS = os.name == "nt"
POSIX_NAME = unittest.skipIf(WINDOWS, "Windows allows no such character in a file name")
RED_UNTIL_1261 = unittest.skipUnless(
    os.environ.get("GRUND_1261_RUN_RED_TESTS") == "1",
    "red until #1261 has the link gate read each changed file by the name the file system "
    "holds; implement removes this",
)
LINE = "See [a page](https://example.invalid/page)."
KEPT = "one\ntwo\n"


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


class ChangedLinesWhateverTheNameTests(unittest.TestCase):
    """§AR-ci.3.1.4.1 — the scope returns a changed file with exactly its changed lines, and
    every line of a new or untracked one, whatever its name holds. Each test makes one change,
    so that one file is the whole scope."""

    def scope(self, change, files=None):
        gate = LinkFixture()
        self.addCleanup(gate.close)
        base = gate.commit_main(files or {"docs/kept.md": KEPT})
        change(gate)
        with mock.patch.dict(os.environ, isolated_env(), clear=True):
            return link_scope.changed_lines(gate.root, base, ["docs"])

    def assert_new(self, name, state="committed"):
        """A new file of one line: committed since the base, staged, or untracked."""
        def change(gate):
            gate.write(name, LINE + "\n")
            if state != "untracked":
                gate.git("add", "-A")
            if state == "committed":
                gate.commit()
        self.assertEqual({os.path.normpath(name): {1: LINE}}, self.scope(change))

    def assert_edited(self, name, state):
        """`one`, `two` in the base, the second line replaced: only line 2 changed."""
        def change(gate):
            gate.write(name, "one\n" + LINE + "\n")
            if state != "unstaged":
                gate.git("add", "-A")
            if state == "committed":
                gate.commit()
        self.assertEqual({os.path.normpath(name): {2: LINE}}, self.scope(change, {name: KEPT}))

    def assert_deleted(self, name):
        def change(gate):
            (gate.root / name).unlink()
            gate.commit()
        self.assertEqual({}, self.scope(change, {name: LINE + "\n", "docs/kept.md": KEPT}))

    def test_a_plain_name_new_edited_and_untracked(self):
        for state in ("committed", "staged", "untracked"):
            with self.subTest(new=state):
                self.assert_new("docs/plain.md", state)
        for state in ("committed", "staged", "unstaged"):
            with self.subTest(edited=state):
                self.assert_edited("docs/plain.md", state)

    @unittest.skipIf(WINDOWS, "on Windows the scope may decode git's UTF-8 by the code page")
    def test_a_non_ascii_name(self):
        self.assert_new("docs/été.md")

    def test_a_deleted_file_yields_nothing(self):
        self.assert_deleted("docs/gone.md")

    @POSIX_NAME
    def test_a_deleted_file_named_with_a_space_and_a_double_quote_yields_nothing(self):
        self.assert_deleted('docs/gone "q.md')

    def test_an_untracked_name_with_a_space(self):
        self.assert_new("docs/u b.md", "untracked")

    @RED_UNTIL_1261
    def test_a_new_file_named_with_a_space(self):
        for state in ("committed", "staged"):
            with self.subTest(state=state):
                self.assert_new("docs/a b.md", state)

    @RED_UNTIL_1261
    def test_an_edited_file_named_with_a_space_gives_only_its_changed_line(self):
        for state in ("committed", "staged", "unstaged"):
            with self.subTest(state=state):
                self.assert_edited("docs/kept space.md", state)

    @POSIX_NAME
    @RED_UNTIL_1261
    def test_a_name_ending_in_a_space(self):
        self.assert_new("docs/trailing .md")

    @POSIX_NAME
    @RED_UNTIL_1261
    def test_a_name_with_a_double_quote(self):
        self.assert_new('docs/a"b.md')

    @POSIX_NAME
    @RED_UNTIL_1261
    def test_a_name_with_a_space_and_a_double_quote(self):
        self.assert_new('docs/a "b.md')

    @POSIX_NAME
    @RED_UNTIL_1261
    def test_a_name_with_a_tab(self):
        self.assert_new("docs/a\tb.md")

    @POSIX_NAME
    @RED_UNTIL_1261
    def test_a_name_with_a_backslash(self):
        self.assert_new("docs/a\\b.md")

    @POSIX_NAME
    @RED_UNTIL_1261
    def test_a_name_with_a_newline(self):
        self.assert_new("docs/a\nb.md")

    @POSIX_NAME
    @RED_UNTIL_1261
    def test_a_name_with_a_carriage_return(self):
        self.assert_new("docs/a\rb.md")

    @POSIX_NAME
    @RED_UNTIL_1261
    def test_a_name_with_another_control_character(self):
        self.assert_new("docs/a\x01b.md")

    @unittest.skipIf(WINDOWS, "on Windows the scope may decode git's UTF-8 by the code page")
    @RED_UNTIL_1261
    def test_a_name_with_non_ascii_text_and_a_space(self):
        self.assert_new("docs/été b.md")

    @unittest.skipIf(WINDOWS, "on Windows the scope may decode git's UTF-8 by the code page")
    @RED_UNTIL_1261
    def test_a_name_with_a_line_separator(self):
        self.assert_new("docs/a b.md")

    @POSIX_NAME
    @RED_UNTIL_1261
    def test_an_untracked_name_with_a_double_quote(self):
        self.assert_new('docs/u"b.md', "untracked")

    @POSIX_NAME
    @RED_UNTIL_1261
    def test_an_untracked_name_with_a_newline(self):
        self.assert_new("docs/u\nb.md", "untracked")

    @POSIX_NAME
    @RED_UNTIL_1261
    def test_an_untracked_name_with_a_carriage_return(self):
        self.assert_new("docs/u\rb.md", "untracked")

    @unittest.skipIf(WINDOWS, "on Windows the scope may decode git's UTF-8 by the code page")
    @RED_UNTIL_1261
    def test_an_untracked_name_with_a_line_separator(self):
        self.assert_new("docs/u b.md", "untracked")


@unittest.skipUnless(LYCHEE, "Link gate fixture requires CI-pinned lychee 0.23.0 on PATH")
@unittest.skipIf(WINDOWS, "Windows allows none of these names but the space, and has no lychee")
class ChangedLineGateWhateverTheNameTests(unittest.TestCase):
    """§AR-ci.3.1.4.1 — a dead URL added on a changed line fails the gate, and is the one URL
    requested, whatever the name of the file that carries it, as it does in `docs/a.md`."""

    def gate_on(self, name, path, *flags):
        gate = LinkFixture()
        self.addCleanup(gate.close)
        gate.commit_main(MAIN)
        gate.write(name, f"# Named\n\nSee [a dead page](<{{host}}{path}>).\n")
        gate.commit()
        code, output = gate.run(*flags, "docs")
        return (sorted(gate.requested()), code), output

    def assert_fails(self, name, path):
        verdict, output = self.gate_on(name, path)
        self.assertEqual(([path], 2), verdict, output)

    def test_the_sweep_requests_a_dead_url_in_a_file_named_with_a_newline(self):
        """The newline the gate cannot follow is left to the sweep, which does (§AR-ci.3.1.5)."""
        (requested, code), output = self.gate_on("docs/a\nb.md", "/dead-newline", "--all")
        self.assertEqual((True, 2), ("/dead-newline" in requested, code), output)

    @RED_UNTIL_1261
    def test_a_space(self):
        self.assert_fails("docs/a b.md", "/dead-space")

    @RED_UNTIL_1261
    def test_a_double_quote(self):
        self.assert_fails('docs/a"b.md', "/dead-quote")

    @RED_UNTIL_1261
    def test_a_tab(self):
        self.assert_fails("docs/a\tb.md", "/dead-tab")

    @RED_UNTIL_1261
    def test_a_backslash(self):
        self.assert_fails("docs/a\\b.md", "/dead-backslash")

    @RED_UNTIL_1261
    def test_a_carriage_return(self):
        self.assert_fails("docs/a\rb.md", "/dead-return")

    @RED_UNTIL_1261
    def test_a_line_separator(self):
        self.assert_fails("docs/a b.md", "/dead-separator")


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
