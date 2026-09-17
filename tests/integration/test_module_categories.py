"""§AR-core-module-layout.1 — the source layout matches the category
boundaries the page names: every implementation file under
`crates/grund-core/src/` belongs to exactly one category, by the module
directory it sits under or, until its category has one, by its file-name
prefix; every prefix the page lists owns at least one file; every module
directory the page names exists and has emptied the top level of its
category but for the prefixes its row still lists; and `lib.rs` is the one
file outside the categories, as the crate entrypoint."""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PAGE = REPO_ROOT / "docs" / "architecture" / "AR-core-module-layout.md"
CORE = REPO_ROOT / "crates" / "grund-core" / "src"
ROW = re.compile(r"^\|\s*\*\*([a-z]+)\*\*\s*\|\s*([^|]*?)\s*\|")
PREFIX = re.compile(r"`([a-z][a-z0-9_]*)`")
MODULE = re.compile(r"`([a-z][a-z0-9_]*)/`")


def _table():
    """Every category row: category -> (module directories, file-name prefixes).

    A cell names a module directory as `model/` and a file-name prefix as
    `model`; the trailing slash is what tells the two apart. A row that has not
    become a module lists no directory, and one that has lists only the
    prefixes whose files it deliberately left flat.
    """
    table = {}
    for line in PAGE.read_text(encoding="utf-8").splitlines():
        match = ROW.match(line)
        if match:
            cell = match.group(2)
            table[match.group(1)] = (MODULE.findall(cell), PREFIX.findall(cell))
    return table


def _categories():
    return {category: prefixes for category, (_, prefixes) in _table().items()}


def _modules():
    return {
        category: directories
        for category, (directories, _) in _table().items()
        if directories
    }


def _implementation_stems():
    return sorted(
        path.stem
        for path in CORE.glob("*.rs")
        if not path.name.startswith("tests") and path.name != "lib.rs"
    )


def _claims(stem, names):
    return any(stem == name or stem.startswith(name + "_") for name in names)


def _category_of(stem, table):
    """The categories that claim a top-level file.

    A category claims a stem by one of its listed prefixes, and a category that
    has become a module directory still claims the flat files named after it —
    so a leftover `model_headings.rs` is reported by the module test as a file
    that failed to move, rather than by the orphan test as a file nobody owns.
    """
    owners = set()
    for category, (directories, prefixes) in table.items():
        if _claims(stem, prefixes) or _claims(stem, directories):
            owners.add(category)
    return owners


class ModuleCategoryTests(unittest.TestCase):
    def test_the_page_carries_the_category_table(self):
        self.assertGreaterEqual(len(_table()), 14, "category table not found on the page")

    def test_every_implementation_file_has_exactly_one_category(self):
        table = _table()
        problems = []
        for stem in _implementation_stems():
            owners = _category_of(stem, table)
            if len(owners) != 1:
                problems.append(f"{stem}.rs -> {sorted(owners) or 'no category'}")
        self.assertEqual([], problems, "\n".join(problems))

    def test_every_listed_prefix_owns_a_file(self):
        stems = _implementation_stems()
        stale = []
        for category, prefixes in _categories().items():
            for prefix in prefixes:
                if not any(stem == prefix or stem.startswith(prefix + "_") for stem in stems):
                    stale.append(f"{category}: `{prefix}`")
        self.assertEqual([], stale, "\n".join(stale))

    def test_every_module_directory_holds_its_category(self):
        problems = []
        for category, directories in _modules().items():
            for directory in directories:
                path = CORE / directory
                if not path.is_dir():
                    problems.append(f"{category}: `{directory}/` is not a directory")
                elif not any(path.glob("*.rs")):
                    problems.append(f"{category}: `{directory}/` holds no .rs file")
        self.assertEqual([], problems, "\n".join(problems))

    def test_a_module_directory_leaves_no_file_of_its_category_flat(self):
        """A file of a category that has a directory and is still flat failed to
        move — unless the row lists its prefix beside the directory, which is
        how the page records a file the move deliberately left behind (today, a
        deprecated renderer waiting for `compat/`)."""
        stale = []
        for category, (directories, prefixes) in _table().items():
            if not directories:
                continue
            for stem in _implementation_stems():
                if _claims(stem, directories) and not _claims(stem, prefixes):
                    stale.append(f"{category}: {stem}.rs is still at the crate root")
        self.assertEqual([], stale, "\n".join(stale))

    def test_lib_rs_is_the_entrypoint_outside_the_categories(self):
        self.assertTrue((CORE / "lib.rs").is_file())
        named = {name for names, prefixes in _table().values() for name in (*names, *prefixes)}
        self.assertNotIn("lib", named)


if __name__ == "__main__":
    unittest.main()
