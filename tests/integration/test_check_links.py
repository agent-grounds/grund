"""§AR-ci.3.1 — canonical links to this repository's default branch are
validated against the current checkout before the remaining links go to the
network, so a target introduced on the current branch is valid but a missing
target or fragment still fails."""

import importlib.util
import re
import tempfile
import unittest
from pathlib import Path
from subprocess import CompletedProcess
from unittest.mock import Mock


SCRIPT_PATH = Path(__file__).resolve().parents[2] / "scripts" / "check_links.py"
SPEC = importlib.util.spec_from_file_location("check_links", SCRIPT_PATH)
check_links = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(check_links)


class CheckLinksTests(unittest.TestCase):
    def test_new_branch_only_target_passes_locally_and_is_exactly_excluded(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            target = root / "docs" / "new-on-this-branch.md"
            target.parent.mkdir()
            target.write_text("# New on this branch\n", encoding="utf-8")
            url = (
                "https://github.com/agent-grounds/grund/blob/main/"
                "docs/new-on-this-branch.md#new-on-this-branch"
            )
            source = root / "SKILL.md"
            source.write_text(f"See [the new guide]({url}).\n", encoding="utf-8")
            runner = Mock(
                side_effect=[
                    CompletedProcess(args=[], returncode=0, stdout=f"{url}\n"),
                    CompletedProcess(args=[], returncode=0),
                    CompletedProcess(args=[], returncode=0),
                ]
            )

            self.assertEqual(0, check_links.check_links(["SKILL.md"], root, runner=runner))

            dump_call, local_call, network_call = runner.call_args_list
            self.assertEqual(["lychee", "--dump", "SKILL.md"], dump_call.args[0])
            local_document = local_call.kwargs["input"]
            self.assertIn(target.resolve().as_uri() + "#new-on-this-branch", local_document)
            self.assertEqual(
                ["--exclude", f"^{re.escape(url)}$"],
                network_call.args[0][3:5],
            )
            self.assertEqual("SKILL.md", network_call.args[0][-1])

    def test_missing_self_link_target_fails_before_lychee(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = root / "SKILL.md"
            source.write_text(
                "See [the missing guide](https://github.com/agent-grounds/grund/blob/main/"
                "docs/missing.md).\n",
                encoding="utf-8",
            )
            url = "https://github.com/agent-grounds/grund/blob/main/docs/missing.md"
            runner = Mock(return_value=CompletedProcess(args=[], returncode=0, stdout=f"{url}\n"))

            with self.assertRaisesRegex(check_links.SelfLinkError, "target is missing"):
                check_links.check_links(["SKILL.md"], root, runner=runner)
            runner.assert_called_once()

    def test_bad_local_fragment_stops_before_the_network_pass(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            target = root / "docs" / "guide.md"
            target.parent.mkdir()
            target.write_text("# Present\n", encoding="utf-8")
            source = root / "SKILL.md"
            source.write_text(
                "See [the absent chapter](https://github.com/agent-grounds/grund/blob/main/"
                "docs/guide.md#absent).\n",
                encoding="utf-8",
            )
            url = "https://github.com/agent-grounds/grund/blob/main/docs/guide.md#absent"
            runner = Mock(
                side_effect=[
                    CompletedProcess(args=[], returncode=0, stdout=f"{url}\n"),
                    CompletedProcess(args=[], returncode=2),
                ]
            )

            self.assertEqual(2, check_links.check_links(["SKILL.md"], root, runner=runner))
            self.assertEqual(2, runner.call_count)


if __name__ == "__main__":
    unittest.main()
