"""§FS-config.concerns — every key of the config belongs to exactly one concern,
and the complete assignment is derived from the reader's parse sites rather than
written by hand. The three parse sites §DF-config-scope-override.2.2 enumerates
are the candidate set, and they are one per grammar because `[[kinds]]` and
`[citations]` read their own keys (§AR-core-module-layout.1): so the inventory is
complete exactly when its first column is that set — every key the reader accepts
has one row, no row names a key no parser accepts, no key is listed twice, and
every row carries exactly one classification from the closed set.
"""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CONFIG = REPO_ROOT / "crates" / "grund-core" / "src" / "config"


def _parse_site(name):
    """A parse site, in the v1 reader's directory once it has moved there
    (§AR-config.2), and beside discovery until then."""
    moved = CONFIG / "v1" / name
    return moved if moved.is_file() else CONFIG / name


PARSE = _parse_site("parse.rs")
KIND_TABLE = _parse_site("kind_table.rs")
CITATIONS = _parse_site("citations.rs")
INVENTORY = (
    REPO_ROOT / "docs" / "decisions" / "functional" / "DF-config-concerns.md"
)

# The closed set of classifications. Three concerns, and the envelope, which is
# what the concerns are read inside rather than a fourth one.
CLASSIFICATIONS = ("schema", "rules", "presentation", "envelope")

# `("<section>", "<key>")` and `("<section>", key @ ("<a>" | "<b>"))`, the two
# shapes the reader's section-and-key match is written in. The catch-all arms
# that delegate to the two grammars below carry no literal and match neither.
PAIR = re.compile(r'^\s*\("([a-z._]*)",\s*"([a-z_]+)"\)\s*=>', re.MULTILINE)
ALTERNATIVES = re.compile(
    r'^\s*\("([a-z._]*)",\s*key @ \(([^)]*)\)\)\s*=>', re.MULTILINE
)
LITERAL = re.compile(r'"([a-z_-]+)"')
# One `match key` arm of a grammar's own reader: at the function's own depth, so
# a match on a *value* nested inside an arm is not read as a key.
KIND_ARM = re.compile(r'^ {8}"([a-z_]+)" =>', re.MULTILINE)
CITATION_ARM = re.compile(r'^\s+"([a-z-]+)" =>', re.MULTILINE)
# A row of the inventory: a backticked key, then its classification.
ROW = re.compile(r"^\|\s*`([^`]+)`\s*\|\s*([a-z]*)\s*\|", re.MULTILINE)

# Keys the reader recognizes only in order to refuse them: they are not keys of
# the format in force, so they are not the inventory's. Each is held to its
# refusal below, so the exclusion is checked rather than asserted.
REFUSED = {"[[kinds]] prefix": "was removed in"}

# One key per parse site and per shape, so a parser that stopped seeing its
# arms fails here rather than agreeing with an inventory that classifies
# nothing. `[id] format` and `[[kinds]] format` are both here on purpose: one
# name, two keys, and an inventory that carries one row for the two is wrong.
SPINE = (
    "grund_config_version",
    "[reference] marker",
    "[id] format",
    "[id] named_sections",
    "[[kinds]] require_grounding",
    "[[kinds]] folder",
    "[[kinds]] format",
    "[scan] exclude",
    "[output] color",
    "[fmt] exclude",
    "[fmt.cross_refs] enabled",
    "[workspace] members",
    "[citations] default",
    "[citations.<KIND>] must",
)


def _source(path):
    """The file without its comments, which quote key names in prose."""
    lines = path.read_text(encoding="utf-8").splitlines()
    return "\n".join("" if line.lstrip().startswith("//") else line for line in lines)


def _qualify(section, key):
    """How one arm's section and key are spelled as a row of the inventory."""
    if section == "":
        return key
    if section == "kinds":
        # An array of tables in the file, whichever reader answers for the key:
        # the section walk hands the two grounding keys to `grounding.rs` and
        # everything else to `kind_table.rs`, and both write the same row.
        return f"[[kinds]] {key}"
    return f"[{section}] {key}"


def project_keys():
    """Every key `parse.rs` accepts, by the table it is written under."""
    source = _source(PARSE)
    keys = [_qualify(section, key) for section, key in PAIR.findall(source)]
    for section, alternatives in ALTERNATIVES.findall(source):
        keys.extend(_qualify(section, key) for key in LITERAL.findall(alternatives))
    return keys


def _function(path, name):
    """One function's body: from its signature to the next item at column 0."""
    source = _source(path)
    start = source.index(f"fn {name}(")
    rest = source[start:]
    end = re.search(r"^\}", rest, re.MULTILINE)
    return rest[: end.end()]


def kind_row_keys():
    """Every key a `[[kinds]]` row accepts, from its own reader."""
    return [f"[[kinds]] {key}" for key in KIND_ARM.findall(_function(KIND_TABLE, "parse_kinds_key"))]


def citation_keys():
    """`[citations]`'s own key, and the keys of a `[citations.<KIND>]` table.

    The reader answers for the two tables in one function, splitting at the
    point it stops being about `[citations]`, so the split is read here from
    that same line rather than from the arms' indentation."""
    body = _function(CITATIONS, "parse_citation_entry")
    boundary = body.index('strip_prefix("citations.")')
    return [f"[citations] {key}" for key in CITATION_ARM.findall(body[:boundary])] + [
        f"[citations.<KIND>] {key}" for key in CITATION_ARM.findall(body[boundary:])
    ]


def reader_keys():
    """Every key of the format in force, as the three parse sites accept it."""
    keys = project_keys() + kind_row_keys() + citation_keys()
    return [key for key in keys if key not in REFUSED]


def inventory_rows():
    """The rows the record's inventory claims to have classified.

    An inventory that is absent classifies nothing, which is the same finding
    as one that stops short: the keys it does not carry are the ones missing."""
    if not INVENTORY.is_file():
        return []
    return ROW.findall(INVENTORY.read_text(encoding="utf-8"))


def _sample(names, limit=8):
    names = sorted(names)
    shown = ", ".join(names[:limit])
    return shown if len(names) <= limit else f"{shown}, … ({len(names)} in all)"


class ConfigConcernInventoryTests(unittest.TestCase):
    def test_the_three_parse_sites_still_read_as_key_literals(self):
        """The guard on every comparison below: a parse that found nothing
        would agree with an inventory that classified nothing."""
        keys = set(reader_keys())
        missing = sorted(key for key in SPINE if key not in keys)
        self.assertEqual(
            [],
            missing,
            f"the config reader under {CONFIG.relative_to(REPO_ROOT)} no longer "
            f"spells these keys where this test reads them: {missing}",
        )

    def test_every_refused_key_is_still_refused_rather_than_read(self):
        """The exclusion list is a claim about the reader, so it is checked:
        a key dropped from the inventory must be one no config may write."""
        arms = _function(KIND_TABLE, "parse_kinds_key")
        for key, refusal in REFUSED.items():
            self.assertIn(
                refusal,
                arms,
                f"{key} is excluded from the inventory as a key the reader only "
                f"refuses, but its refusal is gone from {KIND_TABLE.name}",
            )

    def test_the_record_carries_an_inventory(self):
        """The rows are the classification: without them the comparisons below
        hold an empty set against an empty set and prove nothing."""
        self.assertNotEqual(
            [],
            inventory_rows(),
            f"{INVENTORY.relative_to(REPO_ROOT)} carries no key -> concern table, "
            "so no key is classified and the four checks below are vacuous",
        )

    def test_every_key_the_reader_accepts_has_an_inventory_row(self):
        missing = set(reader_keys()) - {key for key, _ in inventory_rows()}
        self.assertEqual(
            "",
            _sample(missing),
            f"config keys with no row in {INVENTORY.relative_to(REPO_ROOT)}",
        )

    def test_the_inventory_names_only_keys_the_reader_accepts(self):
        extra = {key for key, _ in inventory_rows()} - set(reader_keys())
        self.assertEqual(
            "",
            _sample(extra),
            "inventory rows naming what no parse site accepts",
        )

    def test_every_row_carries_exactly_one_of_the_three_concerns(self):
        wrong = {
            f"{key} → {classification or '(blank)'}"
            for key, classification in inventory_rows()
            if classification not in CLASSIFICATIONS
        }
        self.assertEqual(
            "",
            _sample(wrong),
            f"inventory rows whose classification is not one of {CLASSIFICATIONS}",
        )

    def test_no_key_is_listed_twice(self):
        counted = {}
        for key, _ in inventory_rows():
            counted[key] = counted.get(key, 0) + 1
        duplicated = {key for key, count in counted.items() if count > 1}
        self.assertEqual(
            "",
            _sample(duplicated),
            "inventory rows repeating one key, so one key has two concerns",
        )


if __name__ == "__main__":
    unittest.main()
