"""The license source in §REL-code-queries.work survives the reported HTML 404.

Adapt issue-748's triage transport through the production runner injection:
the HTML viewer answers 404, the underlying raw file answers 200. The actual
document chooses the URL. Both §AR-ci.3.1.4 and §AR-ci.3.1.5 exercise it, while
§FS-non-goals.1.1 still requires success and rejects a missing target.
"""

import contextlib
import io
import json
import os
import re
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from test_check_links import ROOT, check_links, lychee_report


SOURCE = "docs/related-work/REL-code-queries.md"
LABEL = "GitHub CodeQL Terms and Conditions"
HTML = "https://github.com/github/codeql-cli-binaries/blob/main/LICENSE.md"
RAW = "https://raw.githubusercontent.com/github/codeql-cli-binaries/main/LICENSE.md"
REASON = 'Rejected status code: 404 Not Found (configurable with "accept" option)'


class LicenseTransport:
    """Stub only lychee's transport; parsing, scope and settlement remain production code."""

    def __init__(self, url, status=None):
        self.url = url
        self.status = status if status is not None else (200 if url == RAW else 404)
        self.network = 0
        self.successes = 0

    def __call__(self, command, **kwargs):
        if command[0] != "lychee":
            raise AssertionError(f"Unexpected transport request: {command!r}")
        if "--dump" in command:
            suffix = f" ({SOURCE})" if "--verbose" in command else ""
            return subprocess.CompletedProcess(command, 0, stdout=self.url + suffix + "\n")
        if "--offline" in command:
            return subprocess.CompletedProcess(command, 0)
        if "--output" not in command:
            raise AssertionError(f"Expected JSON network report: {command!r}")
        if command[-1] == "-":
            text = kwargs["input"]
        else:
            if command[-1] != SOURCE:
                raise AssertionError(f"Unexpected source input: {command!r}")
            text = (Path(kwargs["cwd"]) / SOURCE).read_text(encoding="utf-8")
        if self.url not in text:
            raise AssertionError("Network input did not contain the document's license URL")
        self.network += 1
        self.successes += int(self.status == 200)
        errors = {} if self.status == 200 else {
            SOURCE: [{"url": self.url, "status": {
                "code": self.status,
                "text": REASON if self.status == 404 else "Service Unavailable",
            }}]
        }
        report = lychee_report(errors)
        report["successful"] = int(self.status == 200)
        Path(command[command.index("--output") + 1]).write_text(
            json.dumps(report), encoding="utf-8"
        )
        return subprocess.CompletedProcess(command, 0 if self.status == 200 else 2)


class CodeqlLicenseSourceTests(unittest.TestCase):
    def setUp(self):
        scratch = Path.home() / "ag/tmp"
        scratch.mkdir(parents=True, exist_ok=True)
        temp = tempfile.TemporaryDirectory(prefix="grund-codeql-source-", dir=scratch)
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)
        # Keep the production wrapper's reports under the same scratch root.
        tempdir = patch.object(tempfile, "tempdir", str(self.root))
        tempdir.start()
        self.addCleanup(tempdir.stop)
        pause = patch.object(check_links, "RECHECK_PAUSE", 0)
        pause.start()
        self.addCleanup(pause.stop)
        self.git("init", "-q")
        self.document = self.root / SOURCE
        self.document.parent.mkdir(parents=True)
        self.document.write_text("# License source\n", encoding="utf-8")
        self.git("add", ".")
        self.git("commit", "-qm", "Before the license source line")
        self.base = self.git("rev-parse", "HEAD")

    def git(self, *args):
        env = {k: v for k, v in os.environ.items()
               if k not in ("GIT_CONFIG_PARAMETERS", "GIT_CONFIG_COUNT")}
        env.update(GIT_CONFIG_GLOBAL=os.devnull, GIT_CONFIG_NOSYSTEM="1")
        return subprocess.run(
            ["git", "-c", "core.hooksPath=/dev/null", "-c", "commit.gpgSign=false",
             "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", *args],
            cwd=self.root, env=env, capture_output=True, text=True, check=True,
        ).stdout.strip()

    def license_url(self):
        text = (ROOT / SOURCE).read_text(encoding="utf-8")
        urls = re.findall(r"\[" + re.escape(LABEL) + r"\]\(([^)]+)\)", text)
        self.assertEqual(1, len(urls), "Expected exactly one CodeQL license source hyperlink")
        return urls[0]

    def run_source(self, mode, url, status=None):
        self.document.write_text(f"# License source\n\n[{LABEL}]({url})\n", encoding="utf-8")
        transport = LicenseTransport(url, status)
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            if mode == "sweep":
                code = check_links.check_links([SOURCE], self.root, runner=transport)
            else:
                code = check_links.check_changed(
                    [SOURCE], self.root, runner=transport, link_base=self.base
                )
        detail = (f"{mode}: {url}; wrapper exit={code}; network passes={transport.network}; "
                  f"successful responses={transport.successes}\n{output.getvalue()}")
        return code, transport, detail

    def assert_document_source_passes(self, mode):
        url = self.license_url()
        code, transport, detail = self.run_source(mode, url)
        self.assertEqual(0, code, detail)
        self.assertEqual(RAW, url, "License source must preserve the publisher, ref and file")
        self.assertEqual(1, transport.network, detail)
        self.assertEqual(1, transport.successes, detail)

    def test_document_license_source_passes_full_sweep(self):
        self.assert_document_source_passes("sweep")

    def test_document_license_source_passes_changed_line_gate(self):
        self.assert_document_source_passes("changed-line gate")

    def test_missing_license_target_still_fails(self):
        for source in (HTML, RAW):
            url = source.replace("LICENSE.md", "grund-issue-748-missing-control.md")
            for mode in ("sweep", "changed-line gate"):
                with self.subTest(source=source, mode=mode):
                    code, transport, detail = self.run_source(mode, url)
                    self.assertEqual(2, code, detail)
                    self.assertEqual(1, transport.network, detail)
                    self.assertEqual(0, transport.successes, detail)
                    self.assertIn(f"[404] {url}", detail)

    def test_raw_license_failure_cannot_pass_without_success(self):
        for mode in ("sweep", "changed-line gate"):
            for status in (404, 503):
                with self.subTest(mode=mode, status=status):
                    code, transport, detail = self.run_source(mode, RAW, status)
                    self.assertEqual(2, code, detail)
                    self.assertGreater(transport.network, 0, detail)
                    self.assertEqual(0, transport.successes, detail)
                    self.assertIn(f"[{status}] {RAW}", detail)


if __name__ == "__main__":
    unittest.main()
