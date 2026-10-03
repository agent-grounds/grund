"""§FS-cli.2.3 — every `github.com/agent-grounds/grund/{blob,tree}/main/<path>`
address the CLI prints names a file (`blob`) or a directory (`tree`) in this
tree, so a printed link cannot go stale. The top-level page's footer is one of
them (§FS-cli.2.2).

The addresses are read from the string literals of the CLI frontend's sources,
with the literal reader the shipped-surfaces guard already uses, so a page with
no e2e golden is covered too. Each address resolves through the same function
the Markdown link gate uses (`scripts/check_links.py`, §AR-ci.3.1), so the two
cannot disagree about what a self-link names.
"""

import importlib.util
import re
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CLI_SOURCES = REPO_ROOT / "crates" / "grund-cli" / "src"

# §FS-cli.2.2: the footer's public address of the guides and examples.
FOOTER_URL = "https://github.com/agent-grounds/grund/tree/main/docs/user-facing"

SELF_URL = re.compile(
    r"https://github\.com/agent-grounds/grund/(?:blob|tree)/main/[^\s)\]\"'`<>]+"
)


def _load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


check_links = _load("check_links", REPO_ROOT / "scripts" / "check_links.py")
shipped_surfaces = _load(
    "shipped_surfaces", REPO_ROOT / "tests" / "integration" / "test_shipped_surfaces.py"
)


def self_link_target(url, repo_root):
    """The path a `blob/main` or `tree/main` self-link names, via the link gate."""
    resolve = getattr(check_links, "self_link_target", None)
    if resolve is None:
        raise AssertionError(
            "scripts/check_links.py has no self_link_target(url, repo_root): the "
            "shared blob/tree resolution §FS-cli.2.3 relies on is not written yet"
        )
    return resolve(url, repo_root)


def self_urls_in(source):
    """Every self-link address in the string literals of one Rust source text."""
    for literal in shipped_surfaces.string_literals(source):
        for match in SELF_URL.finditer(literal):
            yield match.group(0).rstrip(".,;:")


def printed_self_urls(sources):
    """Every self-link address the given Rust files print, with the first file."""
    found = {}
    for path in sources:
        for url in self_urls_in(path.read_text(encoding="utf-8")):
            found.setdefault(url, path.relative_to(REPO_ROOT).as_posix())
    return found


def _cli_sources():
    for path in sorted(CLI_SOURCES.rglob("*.rs")):
        parts = path.relative_to(CLI_SOURCES).parts
        if "tests" in parts or parts[-1].startswith("tests"):
            continue
        yield path


class PrintedLinksExist(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.urls = printed_self_urls(_cli_sources())

    def test_the_scan_finds_the_footer_url(self):
        # A pattern that matched nothing would pass every other test here.
        self.assertIn(
            FOOTER_URL,
            self.urls,
            "the top-level help footer's public address is not among the "
            f"addresses printed by {CLI_SOURCES.relative_to(REPO_ROOT)} (§FS-cli.2.2)",
        )

    def test_every_printed_self_link_names_an_existing_path(self):
        stale = []
        for url, source in sorted(self.urls.items()):
            try:
                self_link_target(url, REPO_ROOT)
            except check_links.SelfLinkError as exc:
                stale.append(f"{source}: {exc}")
        self.assertEqual([], stale, "printed links that do not open (§FS-cli.2.3)")

    def test_a_link_to_a_missing_path_is_refused(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "docs").mkdir()
            (root / "docs" / "guide.md").write_text("# Guide\n", encoding="utf-8")
            base = "https://github.com/agent-grounds/grund"
            self.assertEqual(
                (root / "docs" / "guide.md").resolve(),
                Path(self_link_target(f"{base}/blob/main/docs/guide.md", root)).resolve(),
            )
            self.assertEqual(
                (root / "docs").resolve(),
                Path(self_link_target(f"{base}/tree/main/docs", root)).resolve(),
            )
            for url in (
                f"{base}/blob/main/docs/missing.md",  # no such file
                f"{base}/tree/main/examples",  # no such directory
                f"{base}/blob/main/docs",  # blob names a file, not a directory
                f"{base}/tree/main/docs/guide.md",  # tree names a directory, not a file
            ):
                with self.subTest(url=url), self.assertRaises(check_links.SelfLinkError):
                    self_link_target(url, root)

    def test_the_scan_reads_a_made_up_literal_and_refuses_its_link(self):
        source = (
            'println!("Guide:    https://github.com/agent-grounds/grund/blob/main/'
            'docs/user-facing/no-such-guide.md");\n'
            "// https://github.com/agent-grounds/grund/tree/main/a-comment-is-not-printed\n"
        )
        urls = list(self_urls_in(source))
        self.assertEqual(
            ["https://github.com/agent-grounds/grund/blob/main/docs/user-facing/no-such-guide.md"],
            urls,
        )
        with self.assertRaises(check_links.SelfLinkError):
            self_link_target(urls[0], REPO_ROOT)


if __name__ == "__main__":
    unittest.main()
