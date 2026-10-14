"""§AR-ci.3.1.3 — after the network pass, the link gate checks each failure of a
transient shape once more with lychee, against a stubbed lychee report; the
real-lychee loopback cases are in test_check_links_network.py."""

import re
import unittest

from test_check_links import EMACS_BLOB, EMACS_CONTENTS, StubbedGate, check_links, failure


RECHECK_LINE = re.compile(
    r"(\d+) transient failures re-checked with lychee: (\d+) passed, (\d+) failed"
)
TRANSIENT = {
    "https://publikationen.bibliothek.kit.edu/1000123": failure(
        "https://publikationen.bibliothek.kit.edu/1000123", 503),
    "https://code.visualstudio.com/docs/editing/intellisense": failure(
        "https://code.visualstudio.com/docs/editing/intellisense", 403),
    "https://docs.github.com/en/rest/pulls/pulls": failure(
        "https://docs.github.com/en/rest/pulls/pulls", 502),
    "https://github.com/agent-grounds/grund/pull/1": failure(
        "https://github.com/agent-grounds/grund/pull/1", 429),
    "https://example.org/slow": failure("https://example.org/slow", 408),
    "https://www.cs.tufts.edu/~nr/a.pdf": failure(
        "https://www.cs.tufts.edu/~nr/a.pdf", reason="Timeout"),
    "https://doi.org/10.1609/aaai.v35i1.16119": failure(
        "https://doi.org/10.1609/aaai.v35i1.16119",
        reason="Network error: HTTP/2 protocol error. Server may not support HTTP/2 properly"),
}
SETTLED = [
    failure("https://example.org/gone", 404),
    failure("https://example.org/removed", 410),
    failure("https://example.org/private", 401),
    failure("file:///repo/docs/missing.md"),
    failure("https://example.org/page#absent", reason="Cannot find fragment"),
]


class TransientRecheckTests(unittest.TestCase):
    """§AR-ci.3.1.3 — a failure of a transient shape from any host is checked
    once more with lychee, and passes only on that check's success
    (§FS-non-goals.1.1); nothing else is checked again."""

    def assertReported(self, output, checked, passed, failed):
        match = RECHECK_LINE.search(output)
        self.assertIsNotNone(match, f"no re-check report line in:\n{output}")
        self.assertEqual((checked, passed, failed), tuple(int(g) for g in match.groups()))

    def test_the_pause_is_five_seconds(self):
        self.assertEqual(5, getattr(check_links, "RECHECK_PAUSE", None))

    def test_only_transient_shapes_are_rechecked_in_one_lychee_call(self):
        gate = StubbedGate({
            "docs/related-work/a.md": [*TRANSIENT.values(), *SETTLED],
            "docs/user-facing/lsp.md": [failure(EMACS_BLOB, 503)],
        }, recovers=TRANSIENT)
        code, output = gate.run()
        self.assertNotEqual(0, code, output)
        self.assertEqual(1, len(gate.rechecked), gate.passes)
        self.assertEqual(sorted(TRANSIENT), sorted(gate.rechecked[0]))
        self.assertEqual([EMACS_CONTENTS], gate.endpoints)
        self.assertReported(output, len(TRANSIENT), len(TRANSIENT), 0)
        for settled in SETTLED:
            self.assertIn(settled["url"], output)

    def test_every_transient_failure_the_recheck_answers_passes(self):
        gate = StubbedGate({
            "docs/related-work/a.md": list(TRANSIENT.values()),
            "docs/user-facing/lsp.md": [failure(EMACS_BLOB, 503)],
        }, recovers=TRANSIENT)
        code, output = gate.run()
        self.assertEqual(0, code, output)
        self.assertReported(output, len(TRANSIENT), len(TRANSIENT), 0)

    def test_the_recheck_runs_with_fragments_and_a_json_report(self):
        url = "https://code.visualstudio.com/docs/editing/intellisense"
        gate = StubbedGate({"docs/user-facing/lsp.md": [TRANSIENT[url]]}, recovers=[url])
        code, output = gate.run()
        self.assertEqual(0, code, output)
        self.assertEqual(2, len(gate.passes), gate.passes)
        recheck = gate.passes[1]
        self.assertIn("--include-fragments", recheck)
        self.assertIn("json", recheck)
        self.assertTrue({"-o", "--output"} & set(recheck), recheck)

    def test_a_url_failing_in_two_files_is_rechecked_once_and_settles_both(self):
        url = "https://publikationen.bibliothek.kit.edu/1000123"
        gate = StubbedGate({
            "docs/related-work/a.md": [TRANSIENT[url]],
            "docs/related-work/b.md": [TRANSIENT[url]],
        }, recovers=[url])
        code, output = gate.run()
        self.assertEqual(0, code, output)
        self.assertEqual([[url]], gate.rechecked)
        self.assertReported(output, 1, 1, 0)

    def test_a_transient_failure_the_recheck_repeats_fails_and_is_named(self):
        kit = "https://publikationen.bibliothek.kit.edu/1000123"
        code_vs = "https://code.visualstudio.com/docs/editing/intellisense"
        gate = StubbedGate({
            "docs/related-work/a.md": [TRANSIENT[kit]],
            "docs/user-facing/lsp.md": [TRANSIENT[code_vs]],
        }, recovers=[code_vs])
        code, output = gate.run()
        self.assertNotEqual(0, code, output)
        self.assertReported(output, 2, 1, 1)
        self.assertIn(f"[503] {kit} (docs/related-work/a.md)", output)
        self.assertNotIn(f"{code_vs} (docs/user-facing/lsp.md)", output)

    def test_a_404_beside_a_recovered_transient_failure_still_fails(self):
        kit = "https://publikationen.bibliothek.kit.edu/1000123"
        gate = StubbedGate(
            {"docs/related-work/a.md": [TRANSIENT[kit], SETTLED[0]]}, recovers=[kit]
        )
        code, output = gate.run()
        self.assertNotEqual(0, code, output)
        self.assertEqual([[kit]], gate.rechecked)
        self.assertReported(output, 1, 1, 0)
        self.assertIn(SETTLED[0]["url"], output)

    def test_nothing_transient_runs_no_recheck(self):
        gate = StubbedGate({"docs/related-work/a.md": SETTLED})
        code, output = gate.run()
        self.assertNotEqual(0, code, output)
        self.assertEqual(1, len(gate.passes), gate.passes)
        self.assertIsNone(RECHECK_LINE.search(output), output)


if __name__ == "__main__":
    unittest.main()
