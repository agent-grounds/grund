"""§AR-ci.3.1 — canonical links to this repository's default branch are
validated against the current checkout before the remaining links go to the
network, so a target introduced on the current branch is valid but a missing
target or fragment still fails."""

import contextlib
import importlib.util
import io
import json
import os
import re
import stat
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from subprocess import CompletedProcess
from unittest.mock import Mock, patch


ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = ROOT / "scripts" / "check_links.py"
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


EMACS_BLOB = "https://github.com/emacs-mirror/emacs/blob/master/doc/misc/eglot.texi"
EMACS_CONTENTS = "repos/emacs-mirror/emacs/contents/doc/misc/eglot.texi?ref=master"
GRUND_PULL = "https://github.com/agent-grounds/grund/pull/387"
GRUND_ISSUE = "https://github.com/agent-grounds/grund/issues/552"
REPORT_LINE = re.compile(
    r"(\d+) GitHub 5xx re-checked through the API: (\d+) confirmed, (\d+) broken, (\d+) unsettled"
)


def failure(url, code=None, reason="Rejected status code"):
    status = {"text": reason if code is None else f"{reason}: {code}"}
    if code is not None:
        status["code"] = code
    return {"url": url, "status": status}


def lychee_report(error_map):
    """The shape lychee 0.23.0 writes for `--format json` (§AR-ci.3.1.2)."""
    errors = sum(len(entries) for entries in error_map.values())
    return {
        "total": errors, "successful": 0, "unknown": 0, "unsupported": 0,
        "timeouts": sum(1 for entries in error_map.values() for e in entries
                        if "code" not in e["status"]),
        "redirects": 0, "excludes": 0, "errors": errors, "cached": 0,
        "success_map": {}, "error_map": error_map, "suggestion_map": {},
        "redirect_map": {}, "excluded_map": {}, "duration_secs": 0, "detailed_stats": False,
    }


def gh_answer(status):
    """What `gh api` does for an HTTP status, as gh 2.x does it."""
    if status is None:
        return FileNotFoundError(2, "No such file or directory", "gh")
    if status == "no-auth":
        return CompletedProcess(
            args=[], returncode=4, stdout="",
            stderr="To get started with GitHub CLI, please run:  gh auth login\n",
        )
    if status == 200:
        return CompletedProcess(args=[], returncode=0, stdout='{"type": "file"}', stderr="")
    reason = {404: "Not Found", 500: "Server Error", 502: "Bad Gateway",
              503: "Service Unavailable"}[status]
    return CompletedProcess(
        args=[], returncode=1,
        stdout=json.dumps({"message": reason, "status": str(status)}),
        stderr=f"gh: {reason} (HTTP {status})\n",
    )


class StubbedGate:
    """One run of the wrapper with lychee's network pass and `gh` stubbed.

    The network pass exits 2 and leaves `report` where `--output` names it (or
    on stdout without one); `gh api <endpoint>` answers `api[endpoint]`, or
    `api_default`. `gh` runs through the injected runner (§AR-ci.3.1.2), and a
    missing `gh` is the runner raising FileNotFoundError, as subprocess.run does.
    """

    def __init__(self, error_map, api=None, api_default=200, write_report=True, recovers=()):
        self.report = lychee_report(error_map)
        self.statuses = {e["url"]: e["status"] for es in error_map.values() for e in es}
        self.api = api or {}
        self.api_default = api_default
        self.write_report = write_report
        self.recovers = set(recovers)
        self.endpoints = []
        self.passes = []
        self.rechecked = []

    def _requested(self, command, kwargs):
        """The failing URLs a later lychee call names, on its command line, its
        stdin, or in a file it reads (§AR-ci.3.1.3)."""
        texts = [kwargs.get("input") or ""]
        for part in command:
            path = Path(kwargs.get("cwd") or ".", part)
            if path.is_file() and path.stat().st_size < 1 << 20:
                texts.append(path.read_text(encoding="utf-8", errors="replace"))
        return [
            url for url in self.statuses
            if url in command or any(
                re.search(re.escape(url) + r"(?=[\s)>\]\"']|$)", text) for text in texts
            )
        ]

    def runner(self, command, *args, **kwargs):
        command = [str(part) for part in command]
        if Path(command[0]).stem == "gh":
            endpoint = next(part for part in command if part.startswith("repos/"))
            self.endpoints.append(endpoint)
            answer = gh_answer(self.api.get(endpoint, self.api_default))
            if isinstance(answer, BaseException):
                raise answer
            return answer
        if "--dump" in command:
            return CompletedProcess(args=command, returncode=0, stdout="")
        output = next(
            (command[i + 1] for i, part in enumerate(command) if part in ("-o", "--output")),
            None,
        )
        self.passes.append(command)
        report = json.dumps(self.report)
        if len(self.passes) > 1:
            # A re-check: every named URL fails again unless it recovers.
            urls = self._requested(command, kwargs)
            self.rechecked.append(urls)
            still = [{"url": u, "status": self.statuses[u]} for u in urls
                     if u not in self.recovers]
            report = json.dumps(lychee_report({"-": still} if still else {}))
            if output is not None:
                Path(kwargs.get("cwd") or ".", output).write_text(report, encoding="utf-8")
            return CompletedProcess(args=command, returncode=2 if still else 0, stdout="")
        if output is not None:
            if self.write_report:
                Path(kwargs.get("cwd") or ".", output).write_text(report, encoding="utf-8")
            return CompletedProcess(args=command, returncode=2, stdout="", stderr="")
        return CompletedProcess(
            args=command, returncode=2, stdout=report if self.write_report else "", stderr=""
        )

    def run(self):
        out = io.StringIO()
        with tempfile.TemporaryDirectory() as tmp, \
                patch.object(check_links, "RECHECK_PAUSE", 0, create=True):
            with contextlib.redirect_stdout(out), contextlib.redirect_stderr(out):
                code = check_links.check_links(["docs"], Path(tmp), runner=self.runner)
        return code, out.getvalue()


class GitHub5xxRecheckTests(unittest.TestCase):
    """§AR-ci.3.1.2 — a github.com 5xx is re-checked through the API, and passes
    only on the API's successful answer (§FS-non-goals.1.1)."""

    def assertReported(self, output, checked, confirmed, broken, unsettled):
        match = REPORT_LINE.search(output)
        self.assertIsNotNone(match, f"no re-check report line in:\n{output}")
        self.assertEqual(
            (checked, confirmed, broken, unsettled), tuple(int(g) for g in match.groups())
        )

    # Confirmed: the API's 200 is the successful response, so the hook passes.

    def test_blob_503_the_contents_api_confirms_passes_and_is_reported(self):
        gate = StubbedGate({"docs/user-facing/lsp.md": [failure(EMACS_BLOB, 503)]})
        code, output = gate.run()
        self.assertEqual(0, code, output)
        self.assertEqual([EMACS_CONTENTS], gate.endpoints)
        self.assertReported(output, 1, 1, 0, 0)

    def test_blob_503_with_a_line_fragment_rechecks_the_file(self):
        gate = StubbedGate({"docs/user-facing/lsp.md": [failure(EMACS_BLOB + "#L10", 503)]})
        code, output = gate.run()
        self.assertEqual(0, code, output)
        self.assertEqual([EMACS_CONTENTS], gate.endpoints)

    def test_pull_500_the_pulls_api_confirms_passes(self):
        gate = StubbedGate({"docs/changelog.md": [failure(GRUND_PULL, 500)]})
        code, output = gate.run()
        self.assertEqual(0, code, output)
        self.assertEqual(["repos/agent-grounds/grund/pulls/387"], gate.endpoints)
        self.assertReported(output, 1, 1, 0, 0)

    def test_issue_502_the_issues_api_confirms_passes(self):
        gate = StubbedGate({"docs/changelog.md": [failure(GRUND_ISSUE, 502)]})
        code, output = gate.run()
        self.assertEqual(0, code, output)
        self.assertEqual(["repos/agent-grounds/grund/issues/552"], gate.endpoints)
        self.assertReported(output, 1, 1, 0, 0)

    def test_several_files_all_confirmed_pass_with_one_report_line(self):
        gate = StubbedGate({
            "docs/user-facing/lsp.md": [failure(EMACS_BLOB, 503)],
            "docs/changelog.md": [failure(GRUND_PULL, 504), failure(GRUND_ISSUE, 500)],
        })
        code, output = gate.run()
        self.assertEqual(0, code, output)
        self.assertEqual(1, len(REPORT_LINE.findall(output)), output)
        self.assertReported(output, 3, 3, 0, 0)

    # Broken and unsettled: no successful answer, so the hook fails as today.

    def test_blob_503_the_api_answers_404_fails_as_broken(self):
        gate = StubbedGate(
            {"docs/user-facing/lsp.md": [failure(EMACS_BLOB, 503)]}, api={EMACS_CONTENTS: 404}
        )
        code, output = gate.run()
        self.assertNotEqual(0, code)
        self.assertReported(output, 1, 0, 1, 0)
        self.assertIn(EMACS_BLOB, output)
        self.assertIn("docs/user-facing/lsp.md", output)

    def test_github_503_the_api_answers_5xx_fails_as_unsettled(self):
        gate = StubbedGate({"docs/changelog.md": [failure(GRUND_PULL, 503)]}, api_default=502)
        code, output = gate.run()
        self.assertNotEqual(0, code)
        self.assertReported(output, 1, 0, 0, 1)
        self.assertIn(GRUND_PULL, output)

    def test_github_500_without_gh_fails_as_unsettled(self):
        gate = StubbedGate({"docs/changelog.md": [failure(GRUND_PULL, 500)]}, api_default=None)
        code, output = gate.run()
        self.assertNotEqual(0, code)
        self.assertReported(output, 1, 0, 0, 1)

    def test_github_500_with_gh_unauthenticated_fails_as_unsettled(self):
        gate = StubbedGate(
            {"docs/changelog.md": [failure(GRUND_PULL, 500)]}, api_default="no-auth"
        )
        code, output = gate.run()
        self.assertNotEqual(0, code)
        self.assertReported(output, 1, 0, 0, 1)

    def test_github_5xx_on_an_unmapped_shape_fails_without_a_recheck(self):
        url = "https://github.com/agent-grounds/grund/releases/tag/v0.16.1"
        gate = StubbedGate({"README.md": [failure(url, 503)]})
        code, output = gate.run()
        self.assertNotEqual(0, code)
        self.assertEqual([], gate.endpoints)
        self.assertReported(output, 1, 0, 0, 1)

    # Not a candidate: never re-checked, even when the API would confirm it.

    def test_non_github_503_fails_without_a_recheck(self):
        url = "https://www.gnu.org/software/emacs/manual/html_node/eglot/index.html"
        gate = StubbedGate({"docs/user-facing/lsp.md": [failure(url, 503)]})
        code, _ = gate.run()
        self.assertNotEqual(0, code)
        self.assertEqual([], gate.endpoints)

    def test_github_subdomain_503_fails_without_a_recheck(self):
        url = "https://docs.github.com/en/rest/pulls/pulls"
        gate = StubbedGate({"README.md": [failure(url, 503)]})
        code, _ = gate.run()
        self.assertNotEqual(0, code)
        self.assertEqual([], gate.endpoints)

    def test_github_timeout_fails_without_a_recheck(self):
        gate = StubbedGate({"docs/changelog.md": [failure(GRUND_PULL, reason="Timeout")]})
        code, _ = gate.run()
        self.assertNotEqual(0, code)
        self.assertEqual([], gate.endpoints)

    def test_github_404_fails_without_a_recheck(self):
        gate = StubbedGate({"docs/changelog.md": [failure(GRUND_PULL, 404)]})
        code, _ = gate.run()
        self.assertNotEqual(0, code)
        self.assertEqual([], gate.endpoints)

    def test_a_confirmed_5xx_beside_a_real_404_fails_and_names_the_404(self):
        gone = "https://github.com/agent-grounds/grund/pull/999999/files"
        gate = StubbedGate({
            "docs/user-facing/lsp.md": [failure(EMACS_BLOB, 503)],
            "docs/changelog.md": [failure(gone, 404)],
        })
        code, output = gate.run()
        self.assertNotEqual(0, code)
        self.assertEqual([EMACS_CONTENTS], gate.endpoints)
        self.assertReported(output, 1, 1, 0, 0)
        self.assertIn(gone, output)

    def test_a_kept_code_less_failure_prints_lychee_s_reason(self):
        missing = {"url": "file:///repo/docs/b.md#nosuch", "status": {
            "text": "Cannot find fragment",
            "details": "Fragment not found in document. Check if fragment exists or page structure",
        }}
        refused = {"url": "http://127.0.0.1:9/x", "status": {
            "text": "Network error: Connection refused - server may be down or port blocked",
            "details": "Connection refused - server may be down or port blocked",
        }}
        gate = StubbedGate({"docs/a.md": [missing, refused]})
        code, output = gate.run()
        self.assertNotEqual(0, code)
        self.assertIn(
            "[ERROR] file:///repo/docs/b.md#nosuch (docs/a.md) | Cannot find fragment: "
            "Fragment not found in document.", output,
        )
        self.assertIn(
            "[ERROR] http://127.0.0.1:9/x (docs/a.md) | Network error: Connection refused"
            " - server may be down or port blocked\n", output,
        )

    def test_a_failed_network_pass_without_a_report_fails(self):
        gate = StubbedGate({"docs/changelog.md": [failure(GRUND_PULL, 500)]}, write_report=False)
        code, _ = gate.run()
        self.assertNotEqual(0, code)


@unittest.skipIf(os.name == "nt", "the stub lychee and gh are POSIX scripts")
class GitHub5xxRecheckScriptTests(unittest.TestCase):
    """§AR-ci.3.1.2 end to end — the #581 reproducer: the script, run as the hook
    runs it, passes a GitHub 503 on a blob whose file the contents API confirms."""

    def test_hook_passes_a_blob_503_the_contents_api_confirms(self):
        scratch = Path.home() / "ag" / "tmp"
        scratch.mkdir(parents=True, exist_ok=True)
        report = lychee_report({"docs/user-facing/lsp.md": [failure(EMACS_BLOB, 503)]})
        with tempfile.TemporaryDirectory(prefix="grund-581-", dir=scratch) as tmp:
            bin_dir = Path(tmp)
            lychee = bin_dir / "lychee"
            lychee.write_text(
                f"#!{sys.executable}\n"
                "import sys\n"
                "args = sys.argv[1:]\n"
                f"if '--dump' in args:\n    print({EMACS_BLOB!r}); sys.exit(0)\n"
                f"report = {json.dumps(report)!r}\n"
                "out = next((args[i + 1] for i, a in enumerate(args) if a in ('-o', '--output')), None)\n"
                "if out:\n    open(out, 'w').write(report)\n"
                "else:\n    print(report)\n"
                "sys.exit(2)\n",
                encoding="utf-8",
            )
            gh = bin_dir / "gh"
            gh.write_text(
                f"#!{sys.executable}\n"
                "import sys\n"
                f"if {EMACS_CONTENTS!r} in sys.argv:\n"
                "    print('{\"type\": \"file\"}'); sys.exit(0)\n"
                "print('gh: Not Found (HTTP 404)', file=sys.stderr); sys.exit(1)\n",
                encoding="utf-8",
            )
            for stub in (lychee, gh):
                stub.chmod(stub.stat().st_mode | stat.S_IEXEC)
            env = dict(os.environ, PATH=f"{bin_dir}{os.pathsep}{os.environ['PATH']}")
            proc = subprocess.run(
                [sys.executable, str(SCRIPT_PATH), "--lychee", str(lychee),
                 "docs/user-facing/lsp.md"],
                cwd=ROOT, env=env, capture_output=True, text=True, timeout=60,
            )
        self.assertEqual(0, proc.returncode, proc.stdout + proc.stderr)
        self.assertRegex(proc.stdout + proc.stderr, REPORT_LINE)


class CiPassesTheTokenTests(unittest.TestCase):
    """§AR-ci.3.1.2 — CI gives the pre-commit step the workflow token as GH_TOKEN."""

    def test_pre_commit_step_sets_gh_token(self):
        workflow = (ROOT / ".github" / "workflows" / "ci.yml").read_text(encoding="utf-8")
        step = re.search(
            r"- name: pre-commit\n(?P<body>(?:[ \t]+.*\n|\n)*?)[ \t]+run: pre-commit run --all-files",
            workflow,
        )
        self.assertIsNotNone(step, "no pre-commit step running `pre-commit run --all-files`")
        self.assertRegex(step["body"], r"\n\s+GH_TOKEN: \$\{\{ secrets\.GITHUB_TOKEN \}\}\n")


if __name__ == "__main__":
    unittest.main()
