"""§AR-config.3 — every v1 key and every implicit default lands in exactly one
record field, and the table that says where is held three ways: its key set is
the set the reader's parse sites accept and the §DF-config-concerns inventory
classifies (§AR-config.3.3 item 1); each key's record agrees with its concern
but for the one exception §AR-config.1.4 names (item 2); and the code's table
in `config/v1/mapping.rs` names exactly the page's rows, each at the page's path
or a field below it (item 3).

The parse sites and the inventory are read by `test_config_concern_inventory`,
which this test imports rather than re-reads, so the three sets are compared
from one parse.
"""

import re
import unittest
from pathlib import Path

from test_config_concern_inventory import inventory_rows, reader_keys


REPO_ROOT = Path(__file__).resolve().parents[2]
PAGE = REPO_ROOT / "docs" / "architecture" / "AR-config.md"
MAPPING = REPO_ROOT / "crates" / "grund-core" / "src" / "config" / "v1" / "mapping.rs"

# A table row of the page: a backticked name, then a backticked record path.
ROW = re.compile(r"^\|\s*`([^`]+)`\s*\|\s*`([^`]+)`\s*\|", re.MULTILINE)
# One `("<name>", "<record path>")` entry of a `mapping.rs` constant.
ENTRY = re.compile(r'\(\s*"((?:[^"\\]|\\.)*)"\s*,\s*"([^"]+)"\s*,?\s*\)')

# §AR-config.3.3 item 2: the record each concern lowers into.
CONCERN_ROOTS = {
    "schema": ("schema",),
    "rules": ("rules",),
    "presentation": ("presentation",),
    "envelope": ("name", "version", "workspace"),
}
# §AR-config.1.4: spelled on the kind, judged by rules. The only one.
SPELLED_ELSEWHERE = {"[[kinds]] index"}

# The implicit defaults the ticket names (agent-grounds/grund#453): the page may
# add rows, but it may not lose one of these.
REQUIRED_DEFAULTS = {
    "built-in kinds",
    "legacy FS home",
    "name-keyed index",
    "strict",
    "named sections",
    "scan exclusions",
    "nesting",
    "complement name",
}


def _section(text, start, end):
    body = text[text.index(start):]
    return body[: body.index(end)] if end in body else body


def page_rows(start, end):
    """The `(name, path)` rows of one table of §AR-config.3."""
    text = PAGE.read_text(encoding="utf-8")
    return ROW.findall(_section(text, start, end))


def page_keys():
    return page_rows("### 3.1 ", "### 3.2 ")


def page_defaults():
    return page_rows("### 3.2 ", "### 3.3 ")


def _constant(source, name):
    match = re.search(rf"\b{name}\s*:[^=]*=\s*&\[(.*?)\];", source, re.DOTALL)
    if match is None:
        raise AssertionError(f"{MAPPING.relative_to(REPO_ROOT)} declares no `{name}` table")
    return ENTRY.findall(match.group(1))


def code_table(name):
    """One constant of `config/v1/mapping.rs`, as `(name, path)` entries."""
    if not MAPPING.is_file():
        raise AssertionError(
            f"{MAPPING.relative_to(REPO_ROOT)} does not exist: the v1 reader has no "
            "lowering table (§AR-config.3)"
        )
    lines = MAPPING.read_text(encoding="utf-8").splitlines()
    source = "\n".join("" if line.lstrip().startswith("//") else line for line in lines)
    return _constant(source, name)


def _under(path, prefix):
    return path == prefix or path.startswith(prefix + ".")


def _disagreements(code, page):
    """Where the code's table and the page's disagree, one line each."""
    page_paths = dict(page)
    code_paths = {}
    lines = []
    for name, path in code:
        if name in code_paths:
            lines.append(f"{name}: listed twice in mapping.rs")
        code_paths[name] = path
    for name in sorted(set(page_paths) - set(code_paths)):
        lines.append(f"{name}: no row in mapping.rs")
    for name in sorted(set(code_paths) - set(page_paths)):
        lines.append(f"{name}: in mapping.rs but not on the page")
    for name in sorted(set(page_paths) & set(code_paths)):
        if not _under(code_paths[name], page_paths[name]):
            lines.append(f"{name}: mapping.rs says {code_paths[name]}, the page {page_paths[name]}")
    return lines


class ConfigLoweringPageTests(unittest.TestCase):
    """The page's table, against the reader and the inventory. These hold today."""

    def test_the_page_carries_both_tables(self):
        self.assertGreaterEqual(len(page_keys()), 60, "the key table no longer parses")
        self.assertNotEqual([], page_defaults(), "the default table no longer parses")

    def test_every_key_the_reader_accepts_has_one_page_row(self):
        names = [name for name, _ in page_keys()]
        self.assertEqual(len(names), len(set(names)), "a key is listed twice")
        self.assertEqual(sorted(set(reader_keys())), sorted(names))

    def test_the_page_and_the_inventory_name_the_same_keys(self):
        self.assertEqual(
            sorted(key for key, _ in inventory_rows()),
            sorted(name for name, _ in page_keys()),
        )

    def test_every_key_lowers_into_the_record_its_concern_names(self):
        concern = dict(inventory_rows())
        crossing = set()
        for key, path in page_keys():
            roots = CONCERN_ROOTS[concern[key]]
            if not any(_under(path, root) for root in roots):
                crossing.add(key)
        self.assertEqual(SPELLED_ELSEWHERE, crossing)

    def test_the_page_keeps_every_implicit_default_the_ticket_names(self):
        names = {name for name, _ in page_defaults()}
        self.assertEqual(set(), REQUIRED_DEFAULTS - names)


class ConfigLoweringCodeTests(unittest.TestCase):
    """The code's table, against the page. These fail until `mapping.rs` exists."""

    def test_mapping_rs_lowers_every_key_where_the_page_says(self):
        self.assertEqual([], _disagreements(code_table("KEYS"), page_keys()))

    def test_mapping_rs_lowers_every_default_where_the_page_says(self):
        self.assertEqual([], _disagreements(code_table("DEFAULTS"), page_defaults()))


if __name__ == "__main__":
    unittest.main()
