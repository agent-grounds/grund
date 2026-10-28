"""§AR-ci-self-links — a `tree/main/<path>` self-link is resolved in the checkout as the
`blob/main` form is (§AR-ci.3.1): a directory the branch adds passes with no network
request, and a missing one fails the internal pass before any is made (#1085).

The gate runs as the hook runs it, `LINK_BASE=<base>` and no `--all`, with a lychee
that hands local calls to the real one and records every URL it would have sent to
the network, answering each 200, so a red here never depends on GitHub.
"""

import importlib.util
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from link_gate_fixture import LYCHEE, ROOT, LinkFixture  # noqa: E402


SPEC = importlib.util.spec_from_file_location("check_links", ROOT / "scripts" / "check_links.py")
check_links = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(check_links)

SELF = "https://github.com/agent-grounds/grund"
ADDED = "examples/grund1085-added-by-branch"
ABSENT = "examples/grund1085-absent"
MISSING = "canonical self-link target is missing"

# red until #1085 collects the tree/main form; implement removes each marker below.
RED = unittest.expectedFailure


class SelfLinkFormsTests(unittest.TestCase):
    @RED
    def test_self_links_collects_both_forms(self):
        blob = f"{SELF}/blob/main/{ADDED}/README.md"
        tree = f"{SELF}/tree/main/{ADDED}"
        other = "https://github.com/agent-grounds/rhei/tree/main/docs"
        self.assertEqual(
            sorted([blob, tree]), check_links.self_links(f"{blob}\n{tree}\n{other}\n")
        )

    @RED
    def test_a_tree_link_maps_to_its_directory(self):
        root = ROOT
        expected = (root / "examples").resolve().as_uri()
        self.assertEqual(expected, check_links._local_url(f"{SELF}/tree/main/examples", root))
        self.assertEqual(expected, check_links._local_url(f"{SELF}/tree/main/examples/", root))

    @RED
    def test_a_fragment_on_a_tree_link_is_malformed(self):
        with self.assertRaisesRegex(
            check_links.SelfLinkError, "malformed canonical self-link.*fragment"
        ):
            check_links._local_url(f"{SELF}/tree/main/examples#rules", ROOT)


@unittest.skipUnless(LYCHEE, "Link gate fixture requires CI-pinned lychee 0.23.0 on PATH")
class TreeSelfLinkGateTests(unittest.TestCase):
    def gate(self, line: str) -> tuple[int, str, list[str]]:
        """The branch adds `ADDED/` and `line` to docs/guide.md; the gate's exit, output,
        and the URLs it sent to the network."""
        fixture = LinkFixture()
        self.addCleanup(fixture.close)
        base = fixture.commit_main({"docs/guide.md": "# Guide\n\nIntro.\n"})
        fixture.write(f"{ADDED}/README.md", "# Added by the branch\n")
        fixture.append("docs/guide.md", f"\n{line}\n")
        fixture.commit("branch")
        lychee = fixture.recording_lychee()
        code, output = fixture.run("--lychee", str(lychee), "docs", link_base=base)
        return code, output, fixture.sent()

    def test_blob_link_to_a_file_the_branch_adds_stays_local(self):
        code, output, sent = self.gate(f"See [its readme]({SELF}/blob/main/{ADDED}/README.md).")
        self.assertEqual((0, []), (code, sent), output)

    def test_blob_link_to_a_missing_file_fails_the_internal_pass(self):
        code, output, sent = self.gate(f"See [nothing]({SELF}/blob/main/{ABSENT}/README.md).")
        self.assertEqual([], sent)
        self.assertNotEqual(0, code)
        self.assertIn(MISSING, output)

    @RED
    def test_tree_link_to_a_directory_the_branch_adds_stays_local(self):
        for url in (f"{SELF}/tree/main/{ADDED}", f"{SELF}/tree/main/{ADDED}/"):
            with self.subTest(url=url):
                code, output, sent = self.gate(f"See [the member]({url}).")
                self.assertEqual([], sent, "the gate sent a self-link to the network")
                self.assertEqual(0, code, output)

    @RED
    def test_tree_link_to_a_missing_directory_fails_the_internal_pass(self):
        code, output, sent = self.gate(f"See [nothing]({SELF}/tree/main/{ABSENT}).")
        self.assertEqual([], sent, "the gate sent a self-link to the network")
        self.assertNotEqual(0, code, output)
        self.assertIn(f"{MISSING}: {ABSENT}", output)

    @RED
    def test_tree_link_to_a_file_fails_as_missing(self):
        code, output, sent = self.gate(f"See [a file]({SELF}/tree/main/{ADDED}/README.md).")
        self.assertEqual([], sent, "the gate sent a self-link to the network")
        self.assertNotEqual(0, code, output)
        self.assertIn(MISSING, output)

    @RED
    def test_tree_link_with_a_fragment_is_malformed(self):
        code, output, sent = self.gate(f"See [a heading]({SELF}/tree/main/{ADDED}#added).")
        self.assertEqual([], sent, "the gate sent a self-link to the network")
        self.assertNotEqual(0, code, output)
        self.assertRegex(output, "malformed canonical self-link.*fragment")


if __name__ == "__main__":
    unittest.main()
