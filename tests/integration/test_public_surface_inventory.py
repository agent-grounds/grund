"""§AR-core-module-layout.1.2 — `lib.rs` holds the explicit `pub use` list that
is the crate's public surface, so the public-surface audit is complete exactly
when its inventory rows are that list: every root name has one row, no row names
something the root does not export, and no name is listed twice."""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
LIB_RS = REPO_ROOT / "crates" / "grund-core" / "src" / "lib.rs"
INVENTORY = (
    REPO_ROOT
    / "docs"
    / "discussions"
    / "proposals"
    / "2026-09-22-grund-core-public-surface-inventory.md"
)

USE = re.compile(r"^\s*pub use\s+([^;]+);", re.MULTILINE | re.DOTALL)
# An unqualified `pub` at the crate root; `pub(crate) mod testing;` is not one.
ROOT_ITEM = re.compile(
    r"^pub\s+(?:unsafe\s+|async\s+|extern\s+\"[^\"]*\"\s+)*"
    r"(?:mod|fn|struct|enum|trait|union|type|const|static)\s+([A-Za-z_][A-Za-z0-9_]*)",
    re.MULTILINE,
)
# A row of the inventory: its first cell is the exported name in backticks.
ROW = re.compile(r"^\|\s*`([A-Za-z_][A-Za-z0-9_]*)`\s*\|", re.MULTILINE)

# The spine of the data-returning surface §FS-distribution.3.1 documents. A
# parser that stopped seeing these has stopped seeing the export list, and every
# set comparison below would agree with an empty inventory.
ENTRY_POINTS = (
    "check",
    "show",
    "scan",
    "refs",
    "list",
    "cover",
    "format_references",
    "propose_id",
    "complete_ids",
    "effective_config",
    "init",
)


class GlobReExport(AssertionError):
    pass


def _source():
    """`lib.rs` without its comments, which quote the syntax below in prose."""
    lines = LIB_RS.read_text(encoding="utf-8").splitlines()
    return "\n".join("" if line.lstrip().startswith("//") else line for line in lines)


def _top_level_items(inner):
    depth = 0
    item = ""
    for character in inner:
        if character == "{":
            depth += 1
        elif character == "}":
            depth -= 1
        if character == "," and depth == 0:
            yield item
            item = ""
        else:
            item += character
    yield item


def _exported_names(tree):
    """The names one `pub use` tree puts at the crate root."""
    tree = " ".join(tree.split())
    if tree.endswith("}"):
        head, inner = tree.split("{", 1)
        names = []
        for item in _top_level_items(inner[: inner.rindex("}")]):
            item = item.strip()
            if not item:
                continue
            if item == "self":
                names.append(head.strip().rstrip(":").split("::")[-1])
            else:
                names.extend(_exported_names(item))
        return names
    if tree.endswith("*"):
        raise GlobReExport(f"glob re-export at the crate root: pub use {tree};")
    if " as " in tree:
        return [tree.split(" as ")[-1].strip()]
    return [tree.split("::")[-1].strip()]


def public_root_names():
    """Every distinct spelling reachable as `grund_core::<name>`."""
    source = _source()
    names = []
    for tree in USE.findall(source):
        names.extend(_exported_names(tree))
    names.extend(ROOT_ITEM.findall(source))
    return names


def inventory_names():
    """The names the audit's table claims to have classified.

    An inventory that is absent classifies nothing, which is the same finding as
    one that stops short: the names it does not carry are the ones missing."""
    if not INVENTORY.is_file():
        return []
    return ROW.findall(INVENTORY.read_text(encoding="utf-8"))


def _sample(names, limit=8):
    names = sorted(names)
    shown = ", ".join(names[:limit])
    return shown if len(names) <= limit else f"{shown}, … ({len(names)} in all)"


class PublicSurfaceInventoryTests(unittest.TestCase):
    def test_the_crate_root_still_reads_as_an_explicit_export_list(self):
        """The guard on every comparison below: a parse that found nothing would
        agree with an inventory that classified nothing."""
        names = set(public_root_names())
        missing = sorted(name for name in ENTRY_POINTS if name not in names)
        self.assertEqual(
            [],
            missing,
            f"{LIB_RS} no longer exports the documented entry points, or the "
            f"export list no longer parses: {missing}",
        )

    def test_the_crate_root_re_exports_nothing_by_glob(self):
        """A glob makes the public surface unenumerable, so an audit of it could
        only be a guess (§AR-core-module-layout.1.2)."""
        try:
            public_root_names()
        except GlobReExport as glob:
            self.fail(str(glob))

    def test_every_public_root_name_has_an_inventory_row(self):
        missing = set(public_root_names()) - set(inventory_names())
        self.assertEqual(
            "",
            _sample(missing),
            f"public root names with no row in {INVENTORY.relative_to(REPO_ROOT)}",
        )

    def test_the_inventory_names_only_public_root_names(self):
        extra = set(inventory_names()) - set(public_root_names())
        self.assertEqual(
            "",
            _sample(extra),
            "inventory rows naming what the crate root does not export",
        )

    def test_no_public_root_name_is_listed_twice(self):
        counted = {}
        for name in inventory_names():
            counted[name] = counted.get(name, 0) + 1
        duplicated = {name for name, count in counted.items() if count > 1}
        self.assertEqual(
            "",
            _sample(duplicated),
            "inventory rows repeating one name, so one name has two dispositions",
        )


if __name__ == "__main__":
    unittest.main()
